//! Microsoft Store·MSIX 앱 목록 (현재 사용자, 읽기 전용).
//!
//! `PackageManager`로 현재 사용자에게 등록된 기본(Main) 패키지만 읽는다. 관리자 권한이 필요 없다.

use std::collections::HashSet;
use std::path::Path;

use robin_core::{PackageSource, StoreApp, make_store_app_id};
use windows::ApplicationModel::{Package, PackageSignatureKind};
use windows::Management::Deployment::{PackageManager, PackageTypes};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::core::HSTRING;

use crate::package_info::{
    file_uri_to_path, is_infrastructure_package, pick_logo, publisher_from_dn, ticks_to_date,
    usable_display_name,
};

/// 이 스레드에서 WinRT를 쓰는 동안만 초기화해 둔다.
struct Com(bool);

impl Com {
    fn init() -> Self {
        // 이미 다른 방식으로 초기화된 스레드면 그대로 쓴다 (짝이 맞는 경우에만 해제)
        Com(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

/// 현재 사용자의 Store·MSIX 앱. 목록을 읽지 못하면(예: 지원하지 않는 환경) 빈 목록.
pub fn scan() -> Vec<StoreApp> {
    let _com = Com::init();
    let Ok(manager) = PackageManager::new() else {
        return Vec::new();
    };
    // 빈 SID = 현재 사용자
    let Ok(packages) =
        manager.FindPackagesByUserSecurityIdWithPackageTypes(&HSTRING::new(), PackageTypes::Main)
    else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    packages
        .into_iter()
        .filter_map(|p| to_store_app(&p))
        .filter(|a| seen.insert(a.id.clone()))
        .collect()
}

fn to_store_app(p: &Package) -> Option<StoreApp> {
    // 프레임워크·리소스 패키지와 Windows 구성 요소(시스템 서명)는 사용자가 지울 앱이 아니다
    if p.IsFramework().unwrap_or(true) || p.IsResourcePackage().unwrap_or(true) {
        return None;
    }
    let source = match p.SignatureKind().ok()? {
        PackageSignatureKind::Store => PackageSource::Store,
        PackageSignatureKind::System => return None,
        _ if p.IsDevelopmentMode().unwrap_or(false) => PackageSource::Developer,
        _ => PackageSource::Sideloaded,
    };

    let id = p.Id().ok()?;
    let family_name = id.FamilyName().ok()?.to_string();
    let package_name = id.Name().ok()?.to_string();
    if is_infrastructure_package(&package_name) {
        return None;
    }
    let display_name = p
        .DisplayName()
        .ok()
        .and_then(|n| usable_display_name(&n.to_string()))
        .unwrap_or(package_name);
    let publisher = p
        .PublisherDisplayName()
        .ok()
        .and_then(|n| usable_display_name(&n.to_string()))
        .or_else(|| {
            id.Publisher()
                .ok()
                .and_then(|dn| publisher_from_dn(&dn.to_string()))
        });
    let version = id
        .Version()
        .ok()
        .map(|v| format!("{}.{}.{}.{}", v.Major, v.Minor, v.Build, v.Revision));
    let install_date = p
        .InstalledDate()
        .ok()
        .and_then(|d| ticks_to_date(d.UniversalTime));
    let install_location = p
        .InstalledPath()
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let logo = p
        .Logo()
        .ok()
        .and_then(|uri| uri.AbsoluteUri().ok())
        .and_then(|uri| file_uri_to_path(&uri.to_string()))
        .and_then(|path| resolve_logo(Path::new(&path)));

    Some(StoreApp {
        id: make_store_app_id(&family_name),
        display_name,
        publisher,
        version,
        install_date,
        install_location,
        family_name,
        source,
        logo,
    })
}

/// 매니페스트의 로고 경로를 실제 파일로 바꾼다 (배율이 붙은 파일 중에서 고른다).
fn resolve_logo(path: &Path) -> Option<String> {
    if path.is_file() {
        return Some(path.to_string_lossy().into_owned());
    }
    let dir = path.parent()?;
    let file_name = path.file_name()?.to_str()?;
    let names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    let picked = pick_logo(file_name, &names)?;
    Some(dir.join(picked).to_string_lossy().into_owned())
}
