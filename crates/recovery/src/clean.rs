//! 정리 실행과 복원.
//!
//! 항목마다: 안전 재검사 → 백업 저장 → 원래 자리에서 제거 → 매니페스트 저장.
//! 백업 저장에 실패하면 절대 지우지 않는다.

use std::path::{Path, PathBuf};

use robin_scanner::LeftoverKind;
use robin_scanner::protect::{
    KnownFolders, is_within, protected_key_reason, protected_path_reason,
};

use crate::files::{self, FsError};
use crate::model::{Backup, ItemStatus, Operation, OperationItem, safe_file_name};
use crate::registry::{self, RegError, RegistryBackup};
use crate::store::Store;

pub struct Guard {
    pub known: KnownFolders,
    /// 백업 저장소 자신은 절대 건드리지 않는다
    pub store_root: PathBuf,
}

impl Guard {
    /// 지우기 직전 다시 확인. 스캔 때와 상황이 달라졌을 수 있다.
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
                protected_path_reason(&item.location, &self.known).map(str::to_string)
            }
            LeftoverKind::RegistryKey | LeftoverKind::RegistryValue => {
                let Some((_, sub)) = item.location.split_once('\\') else {
                    return Some("알 수 없는 레지스트리 위치예요".into());
                };
                let sub_lower = sub.to_lowercase();
                if sub_lower.starts_with("system\\currentcontrolset\\services\\") {
                    return Some(
                        "Windows 서비스는 아직 정리할 수 없어요 (서비스를 먼저 중지·제거해야 해요)"
                            .into(),
                    );
                }
                // 값은 키 안의 값 하나만 지우므로 키 자체(Run 등)가 보호 대상이어도 괜찮다
                if item.kind == LeftoverKind::RegistryKey {
                    protected_key_reason(sub, false).map(str::to_string)
                } else {
                    None
                }
            }
        }
    }
}

/// `indices`의 항목을 처리한다(이미 Removed/Restored인 항목은 건너뜀). `on_item`은 진행 상황 알림.
pub fn clean(
    store: &Store,
    op: &mut Operation,
    guard: &Guard,
    mut on_item: impl FnMut(&OperationItem),
) {
    // 레지스트리 값 → 키 → 파일·폴더 순서
    let mut order: Vec<usize> = (0..op.items.len()).collect();
    order.sort_by_key(|&i| match op.items[i].kind {
        LeftoverKind::RegistryValue => 0,
        LeftoverKind::RegistryKey => 1,
        LeftoverKind::File => 2,
        LeftoverKind::Folder => 3,
    });

    let op_dir = store.dir(&op.id);
    for i in order {
        if !matches!(
            op.items[i].status,
            ItemStatus::Pending | ItemStatus::NeedsAdmin | ItemStatus::Failed
        ) {
            continue;
        }
        let item = &mut op.items[i];
        item.error = None;

        if let Some(reason) = guard.blocked_reason(item) {
            item.status = ItemStatus::Blocked;
            item.error = Some(reason);
        } else {
            let result = match item.kind {
                LeftoverKind::Folder | LeftoverKind::File => clean_fs(item, &op_dir, i),
                _ => clean_registry(item, &op_dir, i),
            };
            match result {
                Ok(()) => item.status = ItemStatus::Removed,
                Err((status, msg)) => {
                    item.status = status;
                    item.error = msg;
                }
            }
        }
        let snapshot = item.clone();
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

fn clean_fs(item: &mut OperationItem, op_dir: &Path, index: usize) -> Result<(), ItemError> {
    let src = PathBuf::from(&item.location);
    files::check(&src).map_err(fs_error)?;
    let name = src
        .file_name()
        .map(|n| safe_file_name(&n.to_string_lossy()))
        .unwrap_or_else(|| "item".into());
    let rel = format!("files\\{index}\\{name}");
    files::move_item(&src, &op_dir.join(&rel)).map_err(fs_error)?;
    item.backup = Some(Backup::Quarantine { path: rel });
    Ok(())
}

fn clean_registry(item: &mut OperationItem, op_dir: &Path, index: usize) -> Result<(), ItemError> {
    let backup = match (&item.kind, &item.value_name) {
        (LeftoverKind::RegistryValue, Some(v)) => registry::backup_value(&item.location, v),
        _ => registry::backup_key(&item.location),
    }
    .map_err(reg_error)?;

    // 백업을 디스크에 저장한 뒤에만 지운다
    let rel = format!("registry\\{index}.json");
    let path = op_dir.join(&rel);
    let write = std::fs::create_dir_all(path.parent().unwrap()).and_then(|_| {
        std::fs::write(
            &path,
            serde_json::to_vec_pretty(&backup).unwrap_or_default(),
        )
    });
    if let Err(e) = write {
        return Err((
            ItemStatus::Failed,
            Some(format!("백업을 저장하지 못해 지우지 않았어요: {e}")),
        ));
    }
    item.backup = Some(Backup::Registry { path: rel });

    registry::delete(&backup).map_err(reg_error)
}

/// 제거한 항목을 되돌린다. `ids`가 None이면 전부.
pub fn restore(
    store: &Store,
    op: &mut Operation,
    ids: Option<&[String]>,
    mut on_item: impl FnMut(&OperationItem),
) {
    let op_dir = store.dir(&op.id);
    for i in 0..op.items.len() {
        let item = &mut op.items[i];
        if item.status != ItemStatus::Removed || ids.is_some_and(|ids| !ids.contains(&item.id)) {
            continue;
        }
        let result = match &item.backup {
            Some(Backup::Quarantine { path }) => {
                files::move_item(&op_dir.join(path), Path::new(&item.location)).map_err(|e| match e
                {
                    FsError::Conflict => "원래 자리에 같은 이름이 이미 있어요".to_string(),
                    FsError::AccessDenied => "관리자 권한이 필요해요".to_string(),
                    other => format!("{other:?}"),
                })
            }
            Some(Backup::Registry { path }) => std::fs::read(op_dir.join(path))
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<RegistryBackup>(&bytes).map_err(|e| e.to_string())
                })
                .and_then(|b| {
                    registry::restore(&b).map_err(|e| match e {
                        RegError::Conflict(m) | RegError::Other(m) => m,
                        RegError::AccessDenied(_) => "관리자 권한이 필요해요".into(),
                        other => format!("{other:?}"),
                    })
                }),
            None => Err("백업 정보가 없어요".into()),
        };
        match result {
            Ok(()) => {
                item.status = ItemStatus::Restored;
                item.error = None;
            }
            Err(m) => item.error = Some(format!("복원 실패: {m}")),
        }
        let snapshot = item.clone();
        let _ = store.save(op);
        on_item(&snapshot);
    }
}
