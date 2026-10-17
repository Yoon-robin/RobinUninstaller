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
    for change in traces.iter().flat_map(|t| &t.changes) {
        let location = change.location.clone();
        let value = change.value_name.clone();
        let item = match change.kind {
            ChangeKind::Folder | ChangeKind::File => {
                let path = Path::new(&location);
                if !path.exists() || ctx.is_self(&location) {
                    continue;
                }
                if protected_path_reason(&location, &ctx.known).is_some() {
                    *protected += 1;
                    continue;
                }
                let mut evidence = vec![Evidence::TrackedInstall];
                if let Some(other) = ctx.other_app_sharing_path(&location) {
                    evidence.push(Evidence::SharedWithOtherApp { app_name: other });
                }
                let is_dir = path.is_dir();
                let (size, docs) = if is_dir {
                    measure(path)
                } else {
                    (std::fs::metadata(path).map(|m| m.len()).unwrap_or(0), 0)
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
                    continue;
                }
                let path = location
                    .split_once('\\')
                    .map(|(_, p)| p)
                    .unwrap_or_default();
                // 값은 그 키 자체를 지우는 게 아니라서 키 보호 규칙만 본다 (Run 키의 값은 허용)
                if value.is_none() && protected_key_reason(path, false).is_some() {
                    *protected += 1;
                    continue;
                }
                let kind = if value.is_some() {
                    LeftoverKind::RegistryValue
                } else {
                    LeftoverKind::RegistryKey
                };
                let mut evidence = vec![Evidence::TrackedInstall];
                let last = path.rsplit('\\').next().unwrap_or_default();
                if value.is_none()
                    && let Some(other) = ctx.other_app_named(last)
                {
                    evidence.push(Evidence::SharedWithOtherApp { app_name: other });
                }
                Leftover::new(kind, location, value, evidence)
            }
        };

        // 이미 찾은 항목이면 근거만 더한다
        if let Some(existing) = out.iter_mut().find(|i| i.id.eq_ignore_ascii_case(&item.id)) {
            existing.add_evidence(Evidence::TrackedInstall);
        } else {
            out.push(item);
        }
    }
}
