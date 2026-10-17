//! 크기 기록이 없는 프로그램의 설치 폴더 크기를 직접 잰다 (읽기 전용).
//!
//! 잘못 재면 엉뚱한 크기가 보이므로 보수적으로 고른다: 보호 경로(드라이브 루트, Windows, Program Files 자체,
//! 공유 폴더)와 다른 프로그램과 같이 쓰거나 다른 프로그램 폴더를 품은 위치는 재지 않는다.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use robin_core::InstalledApplication;

use crate::protect::{KnownFolders, is_within, protected_path_reason};

/// 한 폴더에서 세는 항목 수 상한. 넘으면 크기를 보여 주지 않는다 (일부만 센 값은 틀린 값이다).
const ENTRY_LIMIT: usize = 300_000;

/// 크기를 잴 (앱 ID, 설치 폴더) 목록. 크기 기록이 없는 앱만 고른다.
pub fn size_targets(apps: &[InstalledApplication], known: &KnownFolders) -> Vec<(String, String)> {
    let locations: Vec<(&str, String)> = apps
        .iter()
        .filter_map(|a| {
            let loc = a.install_location.as_deref()?.trim().trim_matches('"');
            (!loc.is_empty()).then(|| (a.id.as_str(), loc.to_string()))
        })
        .collect();

    apps.iter()
        .filter(|a| a.estimated_size_kb.unwrap_or(0) == 0)
        .filter_map(|a| {
            let (_, loc) = locations.iter().find(|(id, _)| *id == a.id)?;
            if protected_path_reason(loc, known).is_some() {
                return None;
            }
            // 같은 폴더를 쓰는 다른 프로그램이 있거나, 이 폴더 안에 다른 프로그램이 있으면 누구 크기인지 알 수 없다
            let shared = locations
                .iter()
                .any(|(id, other)| *id != a.id && is_within(other, loc));
            (!shared).then(|| (a.id.clone(), normalize_display(loc)))
        })
        .collect()
}

/// 비교용 소문자 경로가 아니라 원래 대소문자를 살린 경로 (끝 구분자만 정리)
fn normalize_display(loc: &str) -> String {
    let p = loc.trim_end_matches(['\\', '/']);
    if p.len() == 2 && p.ends_with(':') {
        format!("{p}\\")
    } else {
        p.to_string()
    }
}

/// 폴더 크기(바이트). 폴더가 없거나 항목이 너무 많으면 `None`.
/// 심볼릭 링크·정션은 따라가지 않는다 (다른 위치를 잘못 세는 것 방지).
pub fn measure_dir(dir: &Path) -> Option<u64> {
    if !std::fs::symlink_metadata(dir).ok()?.is_dir() {
        return None;
    }
    let mut size = 0u64;
    let mut visited = 0usize;
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > ENTRY_LIMIT {
                return None;
            }
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(entry.path());
            } else {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    Some(size)
}

/// 크기 기록이 없는 프로그램의 설치 폴더 크기 (앱 ID → KB, 올림). 여러 폴더를 동시에 잰다.
pub fn install_sizes(apps: &[InstalledApplication], known: &KnownFolders) -> HashMap<String, u64> {
    const WORKERS: usize = 4;
    let targets = size_targets(apps, known);
    if targets.is_empty() {
        return HashMap::new();
    }
    let chunk = targets.len().div_ceil(WORKERS);
    std::thread::scope(|s| {
        let handles: Vec<_> = targets
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    part.iter()
                        .filter_map(|(id, loc)| {
                            let bytes = measure_dir(Path::new(loc))?;
                            (bytes > 0).then(|| (id.clone(), bytes.div_ceil(1024)))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .flatten()
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::{InstallScope, InstallTechnology, UninstallerStatus};

    fn app(id: &str, location: Option<&str>, size: Option<u64>) -> InstalledApplication {
        InstalledApplication {
            id: id.into(),
            display_name: id.into(),
            publisher: None,
            version: None,
            install_location: location.map(String::from),
            install_date: None,
            estimated_size_kb: size,
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Exe,
            scope: InstallScope::Machine64,
            uninstaller: UninstallerStatus::Available,
        }
    }

    fn known() -> KnownFolders {
        KnownFolders {
            windows: r"C:\Windows".into(),
            program_files: vec![r"C:\Program Files".into(), r"C:\Program Files (x86)".into()],
            program_data: r"C:\ProgramData".into(),
            users_root: r"C:\Users".into(),
            user_profile: r"C:\Users\me".into(),
            app_data: r"C:\Users\me\AppData\Roaming".into(),
            local_app_data: r"C:\Users\me\AppData\Local".into(),
            others: vec![],
        }
    }

    fn ids(targets: &[(String, String)]) -> Vec<&str> {
        targets.iter().map(|(id, _)| id.as_str()).collect()
    }

    #[test]
    fn only_apps_without_recorded_size_are_measured() {
        let apps = [
            app("has-size", Some(r"C:\Program Files\A"), Some(1234)),
            app("zero", Some(r"C:\Program Files\B\"), Some(0)),
            app("none", Some(r#""C:\Program Files\C""#), None),
            app("no-location", None, None),
            app("blank", Some("  "), None),
        ];
        let t = size_targets(&apps, &known());
        assert_eq!(ids(&t), ["zero", "none"]);
        assert_eq!(t[0].1, r"C:\Program Files\B", "끝 구분자 정리");
        assert_eq!(t[1].1, r"C:\Program Files\C", "따옴표 정리");
    }

    #[test]
    fn protected_and_shared_locations_are_skipped() {
        let apps = [
            app("root", Some(r"C:\"), None),
            app("pf", Some(r"C:\Program Files"), None),
            app("win", Some(r"C:\Windows\System32"), None),
            app("common", Some(r"C:\Program Files\Common Files\Foo"), None),
            app("profile", Some(r"C:\Users\me"), None),
            app("odd", Some(r"C:\Program Files\..\Windows"), None),
        ];
        assert!(size_targets(&apps, &known()).is_empty());
    }

    #[test]
    fn folders_used_by_several_apps_are_skipped() {
        let apps = [
            // 같은 폴더를 쓰는 두 항목 (구성 요소가 여러 개 등록된 제품)
            app("office-a", Some(r"C:\Program Files\Office"), None),
            app("office-b", Some(r"c:\program files\office\"), Some(10)),
            // 다른 프로그램 폴더를 품은 폴더는 재지 않고, 안쪽 프로그램은 잰다
            app("suite", Some(r"D:\Suite"), None),
            app("plugin", Some(r"D:\Suite\Plugin"), None),
            app("alone", Some(r"D:\Alone"), None),
        ];
        assert_eq!(ids(&size_targets(&apps, &known())), ["plugin", "alone"]);
    }

    #[test]
    fn measures_files_recursively() {
        let dir = std::env::temp_dir().join(format!("robin-size-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.bin"), vec![0u8; 1500]).unwrap();
        std::fs::write(dir.join("sub").join("b.bin"), vec![0u8; 600]).unwrap();
        assert_eq!(measure_dir(&dir), Some(2100));
        assert_eq!(
            measure_dir(&dir.join("a.bin")),
            None,
            "파일은 폴더가 아니다"
        );
        assert_eq!(measure_dir(&dir.join("missing")), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
