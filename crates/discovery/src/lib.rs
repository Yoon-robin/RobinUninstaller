//! 설치된 프로그램 탐색.
//!
//! 지금은 레지스트리 Uninstall 키(HKLM 64/32비트, HKCU)만 읽는다. 읽기 전용이며 아무것도 수정하지 않는다.
//! MSI API, AppX 탐색은 Phase 1에서 추가한다.

use robin_core::InstalledApplication;

#[cfg(windows)]
mod registry;

/// 설치된 프로그램 목록을 이름순으로 반환한다.
pub fn list_installed() -> Vec<InstalledApplication> {
    #[cfg(windows)]
    let mut apps = registry::scan_all();
    #[cfg(not(windows))]
    let mut apps: Vec<InstalledApplication> = Vec::new();

    apps.sort_by_key(|a| a.display_name.to_lowercase());
    apps
}
