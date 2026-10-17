//! 설치된 프로그램 탐색.
//!
//! 레지스트리 Uninstall 키(HKLM 64/32비트, HKCU)를 읽고 MSI 정보로 보강한다.
//! Microsoft Store·MSIX 앱은 `PackageManager`로 따로 읽는다(목록만, 제거는 Windows 설정에서).
//! 읽기 전용이며 아무것도 수정하지 않는다.

use robin_core::{InstalledApplication, StoreApp};

pub mod command;
#[cfg(windows)]
mod icon;
mod icon_source;
#[cfg(windows)]
mod msi;
pub mod package_info;
#[cfg(windows)]
mod packages;
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

/// 현재 사용자의 Microsoft Store·MSIX 앱을 이름순으로 반환한다 (Windows 구성 요소는 뺀다).
pub fn list_store_apps() -> Vec<StoreApp> {
    #[cfg(windows)]
    let mut apps = packages::scan();
    #[cfg(not(windows))]
    let mut apps: Vec<StoreApp> = Vec::new();

    apps.sort_by_key(|a| a.display_name.to_lowercase());
    apps
}

/// Store 앱 로고(PNG)를 data URL로 읽는다. 너무 크거나 PNG가 아니면 `None`.
pub fn store_app_icon(app: &StoreApp) -> Option<String> {
    #[cfg(windows)]
    {
        use base64::Engine;
        const MAX_BYTES: u64 = 1024 * 1024;
        let path = app.logo.as_deref()?;
        if std::fs::metadata(path).ok()?.len() > MAX_BYTES {
            return None;
        }
        let bytes = std::fs::read(path).ok()?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return None;
        }
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        None
    }
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

/// 파일(실행 파일 등)의 아이콘을 PNG data URL로 추출한다. `path,index` 형식도 받는다.
pub fn file_icon(location: &str) -> Option<String> {
    #[cfg(windows)]
    return icon::extract_data_url(location);
    #[cfg(not(windows))]
    {
        let _ = location;
        None
    }
}
