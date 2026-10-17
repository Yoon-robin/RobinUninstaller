//! 설치 추적 기록에 있는 항목 (읽기 전용).
//!
//! 기록은 "이 프로그램을 설치할 때 새로 생긴 것"이라 가장 강한 근거 중 하나다. 그래도 보호 규칙은 그대로 적용하고,
//! 다른 설치된 프로그램과 겹치면 감점한다. 이미 다른 방법으로 찾은 항목이면 근거만 더한다.

use std::path::Path;

use robin_tracker::{ChangeKind, Trace};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};

use crate::context::ScanContext;
use crate::files::measure;
use crate::model::{Evidence, Leftover, LeftoverKind};
use crate::protect::{protected_key_reason, protected_path_reason};

/// `HKCU\Software\X` → (루트, `Software\X`)
fn split(location: &str) -> Option<(RegKey, &str)> {
    let (hive, path) = location.split_once('\\')?;
    match hive.to_ascii_uppercase().as_str() {
        "HKCU" => Some((RegKey::predef(HKEY_CURRENT_USER), path)),
        "HKLM" => Some((RegKey::predef(HKEY_LOCAL_MACHINE), path)),
        _ => None,
    }
}

/// 레지스트리 키(값 이름이 있으면 값)가 지금 있는지. HKLM은 표기 그대로(64비트 뷰) 연다.
fn registry_exists(location: &str, value: Option<&str>) -> bool {
    let Some((root, path)) = split(location) else {
        return false;
    };
    let Ok(key) = root.open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY) else {
        return false;
    };
    value.is_none_or(|v| key.get_raw_value(v).is_ok())
}

pub fn scan(ctx: &ScanContext, traces: &[Trace], out: &mut Vec<Leftover>, protected: &mut usize) {
    for trace in traces {
        // 같은 설치에서 함께 생긴 다른 프로그램이 아직 설치돼 있으면, 이 기록의 항목은 그 프로그램 것일 수도 있다
        let sibling = trace
            .app_ids
            .iter()
            .filter(|id| !id.eq_ignore_ascii_case(&ctx.app_id))
            .find_map(|id| ctx.other_app_by_id(id))
            .map(|o| o.name.clone());
        for change in &trace.changes {
            if let Some(mut item) = tracked_item(ctx, change, protected) {
                if let Some(name) = &sibling {
                    item.add_evidence(Evidence::SharedWithOtherApp {
                        app_name: name.clone(),
                    });
                }
                out.push(item);
            }
        }
    }
}

/// 기록 항목 하나. 지금 없거나, 보호 대상이거나, 늘 바뀌는 위치면 None.
fn tracked_item(
    ctx: &ScanContext,
    change: &robin_tracker::Change,
    protected: &mut usize,
) -> Option<Leftover> {
    // 예전 기록에는 지금 기준으로 늘 바뀌는 위치(Microsoft 영역 등)가 들어 있을 수 있다
    if robin_tracker::model::is_noise(&change.location) {
        return None;
    }
    let location = change.location.clone();
    let value = change.value_name.clone();
    let item = match change.kind {
        ChangeKind::Folder | ChangeKind::File => {
            let path = Path::new(&location);
            if !path.exists() || ctx.is_self(&location) {
                return None;
            }
            if protected_path_reason(&location, &ctx.known).is_some() {
                *protected += 1;
                return None;
            }
            let mut evidence = vec![Evidence::TrackedInstall];
            if let Some(other) = ctx.other_app_sharing_path(&location) {
                evidence.push(Evidence::SharedWithOtherApp { app_name: other });
            }
            let is_dir = path.is_dir();
            let (size, docs) = if is_dir {
                measure(path)
            } else {
                (
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                    usize::from(crate::files::is_document(path)),
                )
            };
            if docs > 0 {
                evidence.push(Evidence::ContainsUserDocuments { count: docs });
            }
            let kind = if is_dir {
                LeftoverKind::Folder
            } else {
                LeftoverKind::File
            };
            let mut item = Leftover::new(kind, location, None, evidence);
            item.size_bytes = Some(size);
            item
        }
        ChangeKind::RegistryKey | ChangeKind::RegistryValue => {
            if !registry_exists(&location, value.as_deref()) || ctx.is_self(&location) {
                return None;
            }
            let path = location
                .split_once('\\')
                .map(|(_, p)| p)
                .unwrap_or_default();
            // 값은 그 키 자체를 지우는 게 아니라서 키 보호 규칙만 본다 (Run 키의 값은 허용)
            if value.is_none() && protected_key_reason(path, false).is_some() {
                *protected += 1;
                return None;
            }
            let kind = if value.is_some() {
                LeftoverKind::RegistryValue
            } else {
                LeftoverKind::RegistryKey
            };
            let mut evidence = vec![Evidence::TrackedInstall];
            let last = path.rsplit('\\').next().unwrap_or_default();
            let lower = path.to_lowercase();
            let shared = if value.is_some() {
                None
            } else if lower.contains("\\currentversion\\uninstall\\") {
                // 다른 프로그램의 등록 정보(Uninstall 키)
                ctx.uninstall_key_of_other(last)
            } else {
                ctx.other_app_named(last)
            };
            if let Some(other) = shared {
                evidence.push(Evidence::SharedWithOtherApp { app_name: other });
            }
            let service = crate::protect::is_service_key(path);
            let mut item = Leftover::new(kind, location.clone(), value, evidence);
            if service {
                item.note =
                    Some("Windows 서비스예요. 지우기 전에 서비스를 중지하고 제거해야 해요.".into());
            }
            item
        }
    };
    // 같은 항목을 다른 방법으로도 찾았으면 finalize가 하나로 합친다
    Some(item)
}
