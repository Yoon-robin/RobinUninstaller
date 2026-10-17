//! 정리 실행과 복원.
//!
//! 항목마다: 안전 재검사 → 백업 위치를 기록에 먼저 저장 → 백업 → 원래 자리에서 제거 → 기록 저장.
//! 백업 저장에 실패하면 절대 지우지 않는다. 한 번 만든 백업은 덮어쓰지 않는다.
//!
//! 매니페스트(`manifest.json`)는 일반 사용자 권한으로 고칠 수 있는 곳에 있다. 그래서 관리자 작업자도 이 크레이트도
//! 매니페스트를 믿지 않는다: 작업 ID·백업 경로 모양·백업 내용의 위치를 확인하고, 복원할 때도 보호 규칙을 다시 적용한다.

use std::path::{Path, PathBuf};

use robin_scanner::LeftoverKind;
use robin_scanner::protect::{
    KnownFolders, is_service_key, is_within, protected_key_reason, protected_path_reason,
};

use crate::files::{self, Direction, FsError};
use crate::model::{Backup, ItemStatus, Operation, OperationItem, safe_file_name};
use crate::registry::{self, RegError, RegistryBackup};
use crate::store::Store;

#[derive(Default)]
pub struct Guard {
    pub known: KnownFolders,
    /// 백업 저장소 자신은 절대 건드리지 않는다
    pub store_root: PathBuf,
    /// 관리자 작업자로 실행 중. Windows·Microsoft 영역에서는 우리가 실제로 정리하는 위치만 허용한다.
    pub elevated: bool,
    /// 관리자 작업자가 정리를 요청한 사용자와 다른 계정으로 실행 중 (다른 관리자 계정으로 UAC를 허용한 경우).
    /// 이때 HKCU와 사용자 폴더는 요청한 사용자의 것이 아니므로 건드리지 않는다.
    pub foreign_user: bool,
}

/// 관리자 권한으로 다뤄도 되는 Windows·Microsoft 영역 안의 위치 (하이브 제외, 소문자).
/// 잔여물 스캐너가 실제로 찾는 곳들이다: 등록 정보, App Paths, Run, 켜짐·꺼짐 상태, MSI 제품 등록.
fn elevated_allowed(sub: &str, is_value: bool) -> bool {
    let k = sub.to_lowercase();
    let k = k
        .strip_prefix("software\\wow6432node\\")
        .map(|rest| format!("software\\{rest}"))
        .unwrap_or(k);
    let one_below = |prefix: &str| {
        k.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('\\'))
    };
    if is_value {
        matches!(
            k.as_str(),
            "software\\microsoft\\windows\\currentversion\\run"
                | "software\\microsoft\\windows\\currentversion\\explorer\\startupapproved\\run"
                | "software\\microsoft\\windows\\currentversion\\explorer\\startupapproved\\run32"
                | "software\\microsoft\\windows\\currentversion\\explorer\\startupapproved\\startupfolder"
        )
    } else {
        one_below("software\\microsoft\\windows\\currentversion\\uninstall\\")
            || one_below("software\\microsoft\\windows\\currentversion\\app paths\\")
            || one_below("software\\classes\\installer\\products\\")
            || k.strip_prefix("software\\microsoft\\windows\\currentversion\\installer\\userdata\\")
                .and_then(|rest| rest.split_once("\\products\\"))
                .is_some_and(|(sid, product)| {
                    // SYSTEM(S-1-5-18) 또는 사용자 SID 아래, 제품 키 하나만
                    sid.starts_with("s-1-5-")
                        && !sid.contains('\\')
                        && !product.is_empty()
                        && !product.contains('\\')
                })
    }
}

/// Windows·Microsoft 설정 영역인지 (관리자 작업자에서 따로 허용 목록을 적용할 곳)
fn is_system_area(sub: &str) -> bool {
    let k = sub.to_lowercase();
    [
        "software\\microsoft\\",
        "software\\wow6432node\\microsoft\\",
        "software\\classes\\",
        "software\\wow6432node\\classes\\",
        "software\\policies\\",
        "software\\wow6432node\\policies\\",
        "system\\",
    ]
    .iter()
    .any(|p| k.starts_with(p))
}

impl Guard {
    /// 지우거나 되돌리기 직전 다시 확인. 스캔 때와 상황이 달라졌을 수 있고, 기록이 바뀌었을 수도 있다.
    fn blocked_reason(&self, item: &OperationItem) -> Option<String> {
        let lower = item.location.to_lowercase();
        if lower.contains("robinuninstaller") && !lower.contains("robin test labs") {
            return Some("RobinUninstaller 자신의 데이터예요".into());
        }
        match item.kind {
            LeftoverKind::Folder | LeftoverKind::File => {
                if is_within(&item.location, &self.store_root.to_string_lossy()) {
                    return Some("백업 저장소예요".into());
                }
                if self.foreign_user && is_within(&item.location, &self.known.users_root) {
                    return Some(
                        "다른 계정의 관리자 권한으로는 사용자 폴더를 다루지 않아요. 정리를 시작한 계정의 관리자 권한으로 다시 시도해 주세요"
                            .into(),
                    );
                }
                protected_path_reason(&item.location, &self.known).map(str::to_string)
            }
            LeftoverKind::RegistryKey | LeftoverKind::RegistryValue => {
                let Some((hive, sub)) = item.location.split_once('\\') else {
                    return Some("알 수 없는 레지스트리 위치예요".into());
                };
                if self.foreign_user && hive.eq_ignore_ascii_case("hkcu") {
                    return Some(
                        "다른 계정의 관리자 권한으로는 현재 사용자 레지스트리(HKCU)를 다루지 않아요"
                            .into(),
                    );
                }
                if is_service_key(sub) {
                    return Some(
                        "Windows 서비스는 아직 정리할 수 없어요 (서비스를 먼저 중지·제거해야 해요)"
                            .into(),
                    );
                }
                let is_value = item.kind == LeftoverKind::RegistryValue;
                if self.elevated && is_system_area(sub) && !elevated_allowed(sub, is_value) {
                    return Some("Windows 설정 영역이라 관리자 권한으로 다루지 않아요".into());
                }
                // 값은 키 안의 값 하나만 지우므로 키 자체(Run 등)가 보호 대상이어도 괜찮다
                if is_value {
                    protected_key_reason(sub, false)
                        .filter(|r| r.contains("형태"))
                        .map(str::to_string)
                } else {
                    protected_key_reason(sub, false).map(str::to_string)
                }
            }
        }
    }
}

/// 다시 시도할 수 있는 상태
fn retryable(status: ItemStatus) -> bool {
    matches!(
        status,
        ItemStatus::Pending | ItemStatus::NeedsAdmin | ItemStatus::Failed | ItemStatus::Partial
    )
}

/// 처리할 수 있는 모든 항목을 정리한다 (이미 Removed/Restored 등인 항목은 건너뜀). `on_item`은 진행 상황 알림.
pub fn clean(
    store: &Store,
    op: &mut Operation,
    guard: &Guard,
    on_item: impl FnMut(&OperationItem),
) {
    let all: Vec<usize> = (0..op.items.len()).collect();
    clean_only(store, op, guard, &all, on_item);
}

/// `indices`의 항목만 정리한다 (관리자 작업자는 관리자 권한이 필요했던 항목만 넘긴다).
pub fn clean_only(
    store: &Store,
    op: &mut Operation,
    guard: &Guard,
    indices: &[usize],
    mut on_item: impl FnMut(&OperationItem),
) {
    // 레지스트리 값 → 키 → 파일 → 폴더 순서
    let mut order: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|&i| i < op.items.len())
        .collect();
    order.sort_by_key(|&i| match op.items[i].kind {
        LeftoverKind::RegistryValue => 0,
        LeftoverKind::RegistryKey => 1,
        LeftoverKind::File => 2,
        LeftoverKind::Folder => 3,
    });
    order.dedup();

    let op_dir = store.dir(&op.id);
    for i in order {
        if !retryable(op.items[i].status) {
            continue;
        }
        if !op.items[i].backup_path_is_valid(i) {
            op.items[i].status = ItemStatus::Blocked;
            op.items[i].error = Some("작업 기록이 올바르지 않아요".into());
        } else if let Some(reason) = guard.blocked_reason(&op.items[i]) {
            op.items[i].status = ItemStatus::Blocked;
            op.items[i].error = Some(reason);
        } else {
            let result = match op.items[i].kind {
                LeftoverKind::Folder | LeftoverKind::File => clean_fs(store, op, &op_dir, i),
                _ => clean_registry(store, op, &op_dir, i),
            };
            let item = &mut op.items[i];
            match result {
                Ok(()) => {
                    item.status = ItemStatus::Removed;
                    item.error = None;
                }
                Err((status, msg)) => {
                    item.status = status;
                    item.error = msg;
                }
            }
        }
        let snapshot = op.items[i].clone();
        let _ = store.save(op);
        on_item(&snapshot);
    }
}

type ItemError = (ItemStatus, Option<String>);

fn fs_error(e: FsError) -> ItemError {
    match e {
        FsError::NotFound => (ItemStatus::AlreadyGone, None),
        FsError::AccessDenied => (
            ItemStatus::NeedsAdmin,
            Some("관리자 권한이 필요해요".into()),
        ),
        FsError::InUse => (
            ItemStatus::Failed,
            Some("다른 프로그램이 사용 중이에요. 그 프로그램을 닫고 다시 시도하세요".into()),
        ),
        FsError::ReparsePoint => (
            ItemStatus::Blocked,
            Some("다른 위치를 가리키는 링크(정션)라 건드리지 않았어요".into()),
        ),
        FsError::Conflict => (
            ItemStatus::Failed,
            Some("격리 위치에 같은 이름이 있어요".into()),
        ),
        FsError::Partial(m) => (ItemStatus::Partial, Some(m)),
        FsError::Other(m) => (ItemStatus::Failed, Some(m)),
    }
}

fn reg_error(e: RegError) -> ItemError {
    match e {
        RegError::NotFound => (ItemStatus::AlreadyGone, None),
        RegError::AccessDenied(detail) => (
            ItemStatus::NeedsAdmin,
            Some(if detail.is_empty() {
                "관리자 권한이 필요해요".into()
            } else {
                format!("관리자 권한이 필요해요 ({detail})")
            }),
        ),
        RegError::TooLarge => (
            ItemStatus::Blocked,
            Some("하위 항목이 너무 많아 안전하게 백업할 수 없어요".into()),
        ),
        RegError::Conflict(m) | RegError::Other(m) => (ItemStatus::Failed, Some(m)),
    }
}

/// 지우기 전에 백업 위치를 기록에 먼저 남긴다. 저장하지 못하면 지우지 않는다.
fn record_backup(
    store: &Store,
    op: &mut Operation,
    i: usize,
    backup: Backup,
) -> Result<(), ItemError> {
    op.items[i].backup = Some(backup);
    store.save(op).map_err(|e| {
        op.items[i].backup = None;
        (
            ItemStatus::Failed,
            Some(format!("작업 기록을 저장하지 못해 지우지 않았어요: {e}")),
        )
    })
}

fn clean_fs(store: &Store, op: &mut Operation, op_dir: &Path, i: usize) -> Result<(), ItemError> {
    let src = PathBuf::from(&op.items[i].location);

    // 전에 다른 드라이브로 복사까지 끝내고 원본 일부만 남은 항목: 복사본은 온전하니 남은 원본만 지운다
    if let (ItemStatus::Partial, Some(Backup::Quarantine { path })) =
        (op.items[i].status, &op.items[i].backup)
    {
        if !op_dir.join(path).exists() {
            return Err((
                ItemStatus::Failed,
                Some("격리 폴더의 백업이 없어져 다시 정리하지 않았어요".into()),
            ));
        }
        return match files::remove_rest(&src) {
            Ok(()) | Err(FsError::NotFound) => Ok(()),
            Err(e) => Err(match fs_error(e) {
                // 백업은 이미 있으므로 "일부만 정리됨"으로 남긴다
                (ItemStatus::NeedsAdmin, m) => (ItemStatus::Partial, m),
                other => other,
            }),
        };
    }
    // 백업 위치가 기록돼 있는데 백업이 이미 있으면, 지난번에 중간에 멈춘 것이다. 내용이 온전한지 알 수 없으니 손대지 않는다.
    if let Some(Backup::Quarantine { path }) = &op.items[i].backup
        && op_dir.join(path).exists()
    {
        return Err((
            ItemStatus::Failed,
            Some("지난번 정리가 중간에 멈췄어요. 복구 기록에서 확인해 주세요".into()),
        ));
    }

    files::check(&src).map_err(fs_error)?;
    let name = src
        .file_name()
        .map(|n| safe_file_name(&n.to_string_lossy()))
        .unwrap_or_else(|| "item".into());
    let rel = format!("files\\{i}\\{name}");
    record_backup(store, op, i, Backup::Quarantine { path: rel.clone() })?;
    match files::move_item(&src, &op_dir.join(&rel), Direction::Clean) {
        Ok(()) => Ok(()),
        // 복사본이 있으므로 백업 기록은 남긴다
        Err(e @ FsError::Partial(_)) => Err(fs_error(e)),
        Err(e) => {
            if !op_dir.join(&rel).exists() {
                op.items[i].backup = None;
            }
            Err(fs_error(e))
        }
    }
}

fn clean_registry(
    store: &Store,
    op: &mut Operation,
    op_dir: &Path,
    i: usize,
) -> Result<(), ItemError> {
    let rel = format!("registry\\{i}.json");
    let path = op_dir.join(&rel);
    let item = &op.items[i];

    // 이미 백업 파일이 있으면(지난번에 지우다 멈춤, 관리자 권한으로 다시 시도) 그 백업을 그대로 쓴다.
    // 지금 상태를 다시 백업하면 지난번에 지운 부분이 백업에서 사라진다.
    let backup = if path.exists() {
        let existing = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<RegistryBackup>(&b).ok())
            .ok_or((
                ItemStatus::Failed,
                Some("이전 백업을 읽지 못해 지우지 않았어요".into()),
            ))?;
        if existing.target() != (item.location.as_str(), item.value_name.as_deref()) {
            return Err((
                ItemStatus::Blocked,
                Some("작업 기록과 백업 내용이 맞지 않아요".into()),
            ));
        }
        existing
    } else {
        let fresh = match (&item.kind, &item.value_name) {
            (LeftoverKind::RegistryValue, Some(v)) => registry::backup_value(&item.location, v),
            _ => registry::backup_key(&item.location),
        }
        .map_err(reg_error)?;
        // 백업을 디스크에 저장한 뒤에만 지운다 (이미 있는 파일은 덮어쓰지 않는다)
        let write = std::fs::create_dir_all(op_dir.join("registry")).and_then(|_| {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            f.write_all(&serde_json::to_vec_pretty(&fresh).unwrap_or_default())?;
            f.sync_all()
        });
        if let Err(e) = write {
            return Err((
                ItemStatus::Failed,
                Some(format!("백업을 저장하지 못해 지우지 않았어요: {e}")),
            ));
        }
        fresh
    };
    record_backup(store, op, i, Backup::Registry { path: rel })?;

    match registry::delete(&backup) {
        Ok(()) | Err(RegError::NotFound) => Ok(()),
        Err(e) => {
            let (mut status, msg) = reg_error(e);
            // 일부 하위 키만 지워졌으면 "일부만 정리됨" (백업은 온전하다)
            if op.items[i].kind == LeftoverKind::RegistryKey && !registry::matches_backup(&backup) {
                status = ItemStatus::Partial;
            }
            Err((status, msg))
        }
    }
}

/// 복원 순서: 상위 위치부터 (함께 정리한 하위 항목이 상위 키·폴더를 먼저 만들어 충돌하지 않게), 키 다음에 값.
fn restore_order(op: &Operation) -> Vec<usize> {
    let mut order: Vec<usize> = (0..op.items.len()).collect();
    order.sort_by_key(|&i| {
        let item = &op.items[i];
        let kind_rank = match item.kind {
            LeftoverKind::Folder | LeftoverKind::File => 0,
            LeftoverKind::RegistryKey => 1,
            LeftoverKind::RegistryValue => 2,
        };
        (
            kind_rank,
            item.location.trim_end_matches('\\').matches('\\').count(),
        )
    });
    order
}

/// 제거한 항목을 되돌린다. `ids`가 None이면 전부. 일부만 정리된 항목은 없는 부분만 채워 넣는다.
pub fn restore(
    store: &Store,
    op: &mut Operation,
    guard: &Guard,
    ids: Option<&[String]>,
    mut on_item: impl FnMut(&OperationItem),
) {
    let op_dir = store.dir(&op.id);
    for i in restore_order(op) {
        let item = &op.items[i];
        let restorable = matches!(item.status, ItemStatus::Removed | ItemStatus::Partial);
        if !restorable || ids.is_some_and(|ids| !ids.contains(&item.id)) {
            continue;
        }
        let partial = item.status == ItemStatus::Partial;
        let result = if !item.backup_path_is_valid(i) {
            Err("작업 기록이 올바르지 않아요".to_string())
        } else if let Some(reason) = guard.blocked_reason(item) {
            Err(reason)
        } else {
            restore_item(item, &op_dir, partial)
        };
        let item = &mut op.items[i];
        match result {
            Ok(kept) if kept.is_empty() => {
                item.status = ItemStatus::Restored;
                item.error = None;
            }
            Ok(kept) => {
                item.status = ItemStatus::Restored;
                item.error = Some(format!(
                    "되돌렸지만 이미 다른 내용이 있는 {}개는 덮어쓰지 않았어요: {}",
                    kept.len(),
                    kept.join(", ")
                ));
            }
            Err(m) => item.error = Some(format!("복원 실패: {m}")),
        }
        let snapshot = item.clone();
        let _ = store.save(op);
        on_item(&snapshot);
    }
}

/// 항목 하나를 되돌린다. 덮어쓰지 않고 남긴 것의 목록을 돌려준다.
fn restore_item(item: &OperationItem, op_dir: &Path, partial: bool) -> Result<Vec<String>, String> {
    let reg_message = |e: RegError| match e {
        RegError::Conflict(m) | RegError::Other(m) => m,
        RegError::AccessDenied(_) => "관리자 권한이 필요해요".into(),
        other => format!("{other:?}"),
    };
    match &item.backup {
        Some(Backup::Quarantine { path }) => {
            let backup = op_dir.join(path);
            let original = Path::new(&item.location);
            if partial {
                files::merge_back(&backup, original).map_err(|e| format!("{e:?}"))
            } else {
                files::move_item(&backup, original, Direction::Restore)
                    .map(|()| Vec::new())
                    .map_err(|e| match e {
                        FsError::Conflict => "원래 자리에 같은 이름이 이미 있어요".to_string(),
                        FsError::AccessDenied => "관리자 권한이 필요해요".to_string(),
                        other => format!("{other:?}"),
                    })
            }
        }
        Some(Backup::Registry { path }) => {
            let backup = std::fs::read(op_dir.join(path))
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<RegistryBackup>(&bytes).map_err(|e| e.to_string())
                })?;
            // 백업 파일이 기록과 다른 곳을 가리키면 (기록이나 백업이 바뀜) 되돌리지 않는다
            if backup.target() != (item.location.as_str(), item.value_name.as_deref()) {
                return Err("작업 기록과 백업 내용이 맞지 않아요".into());
            }
            match registry::restore(&backup) {
                Ok(()) => Ok(Vec::new()),
                // 일부만 지워진 항목: 남아 있는 키에 없는 것만 채운다. 그 밖의 충돌(다시 설치 등)은 덮어쓰지 않고 알린다.
                Err(RegError::Conflict(_)) if partial => {
                    registry::restore_merge(&backup).map_err(reg_message)
                }
                Err(e) => Err(reg_message(e)),
            }
        }
        None => Err("백업 정보가 없어요".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevated_allowlist() {
        let key = |p: &str| elevated_allowed(p, false);
        assert!(key(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Acme"
        ));
        assert!(key(
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Acme"
        ));
        assert!(key(r"SOFTWARE\Classes\Installer\Products\00ABC"));
        assert!(key(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\UserData\S-1-5-21-1-2-3-1001\Products\00ABC"
        ));
        // Uninstall 키 자체, 더 깊은 곳, 다른 Microsoft 영역은 안 된다
        assert!(!key(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"));
        assert!(!key(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Acme\Sub"
        ));
        assert!(!key(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\x.exe"
        ));
        assert!(!key(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\UserData\S-1-5-18\Products\00ABC\InstallProperties"
        ));
        assert!(elevated_allowed(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            true
        ));
        assert!(!elevated_allowed(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
            true
        ));
    }
}
