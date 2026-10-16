//! 아이콘을 어디서 가져올지 고른다.
//!
//! 순서: 레지스트리 DisplayIcon → MSI 제품 아이콘 → 설치 폴더의 메인 실행 파일 → 언인스톨러 실행 파일.

use std::path::{Path, PathBuf};

use robin_core::{InstallScope, InstalledApplication};

use crate::command::{executable_path, expand_env};

/// 메인 실행 파일로 보기 어려운 이름들.
const NON_MAIN_EXE: &[&str] = &[
    "unins",
    "uninst",
    "update",
    "setup",
    "install",
    "crash",
    "helper",
    "elevat",
    "report",
    "service",
    "svc",
    "notif",
    "broker",
    "squirrel",
    "vc_redist",
];

pub fn icon_sources(app: &InstalledApplication) -> Vec<String> {
    let mut sources = Vec::new();

    if let Some(icon) = &app.display_icon {
        sources.push(icon.clone());
    }

    #[cfg(windows)]
    if let Some(code) = &app.product_code
        && let Some(icon) = crate::msi::product_icon(code, app.scope == InstallScope::User)
    {
        sources.push(icon);
    }
    #[cfg(not(windows))]
    let _ = InstallScope::User;

    if let Some(dir) = &app.install_location
        && let Some(exe) = main_executable(Path::new(dir.trim_matches('"')), &app.display_name)
    {
        sources.push(exe.to_string_lossy().into_owned());
    }

    if let Some(cmd) = &app.uninstall_command
        && let Some(exe) = executable_path(&expand_env(cmd, |n| std::env::var(n).ok()))
    {
        sources.push(exe);
    }

    sources
}

/// 설치 폴더(와 한 단계 아래 폴더)에서 프로그램 이름과 가장 잘 맞는 exe를 고른다.
fn main_executable(dir: &Path, display_name: &str) -> Option<PathBuf> {
    let mut candidates: Vec<(PathBuf, u64)> = Vec::new();
    collect_exes(dir, &mut candidates);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for sub in entries.flatten().take(64) {
            if sub.file_type().is_ok_and(|t| t.is_dir()) {
                collect_exes(&sub.path(), &mut candidates);
            }
        }
    }

    candidates
        .into_iter()
        .filter(|(p, _)| !is_non_main(p))
        .max_by_key(|(p, size)| (name_score(p, display_name), *size))
        .map(|(p, _)| p)
}

fn collect_exes(dir: &Path, out: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten().take(256) {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
            && let Ok(meta) = entry.metadata()
        {
            out.push((path, meta.len()));
        }
    }
}

fn is_non_main(path: &Path) -> bool {
    let stem = file_stem_lower(path);
    NON_MAIN_EXE.iter().any(|w| stem.contains(w))
}

/// 파일 이름이 프로그램 이름의 단어와 얼마나 겹치는지. 높을수록 메인 실행 파일일 가능성이 크다.
pub(crate) fn name_score(path: &Path, display_name: &str) -> u32 {
    let stem: String = file_stem_lower(path)
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    if stem.is_empty() {
        return 0;
    }
    display_name
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !w.chars().all(|c| c.is_ascii_digit()))
        .filter(|w| stem.contains(w))
        .map(|w| w.len() as u32)
        .sum()
}

fn file_stem_lower(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_exe_matching_program_name() {
        let blender = name_score(Path::new(r"C:\x\blender.exe"), "Blender");
        let launcher = name_score(Path::new(r"C:\x\blender-launcher.exe"), "Blender");
        let other = name_score(Path::new(r"C:\x\python.exe"), "Blender");
        assert!(blender > 0 && launcher > 0);
        assert_eq!(other, 0);
    }

    #[test]
    fn ignores_version_numbers_and_short_words() {
        assert_eq!(name_score(Path::new(r"C:\x\5.0.exe"), "Blender 5.0"), 0);
        assert_eq!(name_score(Path::new(r"C:\x\ai.exe"), "Topaz Video AI"), 0);
    }

    #[test]
    fn filters_uninstallers_and_updaters() {
        assert!(is_non_main(Path::new(r"C:\x\unins000.exe")));
        assert!(is_non_main(Path::new(r"C:\x\Update.exe")));
        assert!(!is_non_main(Path::new(r"C:\x\Discord.exe")));
    }
}
