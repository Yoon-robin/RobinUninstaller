//! 설치된 프로그램 탐색.
//!
//! 레지스트리 Uninstall 키(HKLM 64/32비트, HKCU)를 읽고 MSI 정보로 보강한다.
//! 읽기 전용이며 아무것도 수정하지 않는다. AppX/MSIX는 이후 버전에서 추가한다.

use robin_core::InstalledApplication;

pub mod command;
#[cfg(windows)]
mod icon;
mod icon_source;
#[cfg(windows)]
mod msi;
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

/// 프로그램 아이콘을 PNG data URL로 추출한다. 여러 후보 중 처음 성공한 것을 쓴다.
pub fn app_icon(app: &InstalledApplication) -> Option<String> {
    #[cfg(windows)]
    return icon_source::icon_sources(app)
        .iter()
        .find_map(|source| icon::extract_data_url(source));
    #[cfg(not(windows))]
    {
        let _ = app;
        None
    }
}
