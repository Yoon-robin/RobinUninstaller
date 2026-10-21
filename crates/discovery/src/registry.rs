use std::path::Path;

use robin_core::{
    InstallScope, InstallTechnology, InstalledApplication, UninstallerStatus, make_app_id,
};

use crate::command::{executable_path, expand_env, find_product_code, is_product_code};
use crate::msi;
use winreg::RegKey;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};

const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

const SOURCES: [(winreg::HKEY, u32, InstallScope); 3] = [
    (HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY, InstallScope::Machine64),
    (HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY, InstallScope::Machine32),
    (HKEY_CURRENT_USER, 0, InstallScope::User),
];

pub fn scan_all() -> Vec<InstalledApplication> {
    SOURCES
        .into_iter()
        .flat_map(|(hive, view, scope)| scan(hive, view, scope))
        .collect()
}

/// ID(`hkcu:키이름` 등)의 등록 정보 하나만 읽는다. 목록 전체를 읽는 것과 같은 기준으로 거른다
/// (시스템 구성 요소·업데이트는 없음으로 본다). 제거 뒤 "아직 설치돼 있나" 확인처럼 하나만 볼 때 쓴다.
pub fn find(id: &str) -> Option<InstalledApplication> {
    let (prefix, key_name) = id.split_once(':')?;
    let (hive, view, scope) = SOURCES
        .into_iter()
        .find(|(_, _, s)| s.key_prefix() == prefix)?;
    let root = RegKey::predef(hive)
        .open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | view)
        .ok()?;
    // ID는 키 이름을 소문자로 바꾼 것이라, 원래 이름(MSI 제품 코드는 대문자)을 찾아 쓴다
    let name = root
        .enum_keys()
        .filter_map(Result::ok)
        .find(|n| n.eq_ignore_ascii_case(key_name))?;
    let key = root.open_subkey_with_flags(&name, KEY_READ | view).ok()?;
    read_entry(&key, &name, scope)
}

fn scan(hive: winreg::HKEY, view: u32, scope: InstallScope) -> Vec<InstalledApplication> {
    let Ok(root) = RegKey::predef(hive).open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | view)
    else {
        return Vec::new();
    };

    root.enum_keys()
        .filter_map(Result::ok)
        .filter_map(|name| {
            let key = root.open_subkey_with_flags(&name, KEY_READ | view).ok()?;
            read_entry(&key, &name, scope)
        })
        .collect()
}

fn read_entry(key: &RegKey, key_name: &str, scope: InstallScope) -> Option<InstalledApplication> {
    let display_name = get_str(key, "DisplayName")?;

    // Windows 설정 앱 목록과 같은 기준: 시스템 구성요소, 업데이트, 상위 제품에 딸린 항목은 숨긴다.
    if get_u32(key, "SystemComponent") == Some(1)
        || get_str(key, "ParentKeyName").is_some()
        || get_str(key, "ReleaseType").is_some_and(|t| t.contains("Update") || t == "Hotfix")
    {
        return None;
    }

    let is_msi = get_u32(key, "WindowsInstaller") == Some(1);
    let product_code = (is_msi && is_product_code(key_name)).then(|| key_name.to_string());
    let uninstall_command = get_str(key, "UninstallString");

    let mut install_location = get_str(key, "InstallLocation");
    if install_location.is_none()
        && let Some(code) = &product_code
    {
        install_location = msi::install_location(code, scope == InstallScope::User);
    }

    let uninstaller = uninstaller_status(product_code.as_deref(), uninstall_command.as_deref());

    Some(InstalledApplication {
        id: make_app_id(scope, key_name),
        display_name,
        publisher: get_str(key, "Publisher"),
        version: get_str(key, "DisplayVersion"),
        install_location,
        install_date: get_str(key, "InstallDate"),
        estimated_size_kb: get_u32(key, "EstimatedSize").map(u64::from),
        display_icon: get_str(key, "DisplayIcon"),
        uninstall_command,
        quiet_uninstall_command: get_str(key, "QuietUninstallString"),
        product_code,
        technology: if is_msi {
            InstallTechnology::Msi
        } else {
            InstallTechnology::Exe
        },
        scope,
        uninstaller,
    })
}

fn uninstaller_status(product_code: Option<&str>, command: Option<&str>) -> UninstallerStatus {
    if let Some(code) = product_code {
        return if msi::is_installed(code) {
            UninstallerStatus::Available
        } else {
            UninstallerStatus::Missing
        };
    }
    let Some(command) = command else {
        return UninstallerStatus::NotRegistered;
    };
    let command = expand_env(command, |name| std::env::var(name).ok());
    match executable_path(&command) {
        None => match find_product_code(&command) {
            Some(code) if msi::is_installed(code) => UninstallerStatus::Available,
            Some(_) => UninstallerStatus::Missing,
            None => UninstallerStatus::Unknown,
        },
        Some(path) if Path::new(&path).is_file() => UninstallerStatus::Available,
        // 절대 경로인데 파일이 없을 때만 Missing. 상대 경로(rundll32 등)는 판단하지 않는다.
        Some(path) if Path::new(&path).is_absolute() => UninstallerStatus::Missing,
        _ => UninstallerStatus::Unknown,
    }
}

fn get_str(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn get_u32(key: &RegKey, name: &str) -> Option<u32> {
    key.get_value::<u32, _>(name).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_matches_the_full_scan() {
        // 읽기만 한다: 전체 목록의 앞쪽 몇 개를 하나씩 다시 읽어 같은지 본다
        for app in super::scan_all().iter().take(15) {
            let one = super::find(&app.id).expect("목록에 있는 ID는 하나만 읽어도 있어야 함");
            assert_eq!(one.id, app.id);
            assert_eq!(one.display_name, app.display_name);
            assert_eq!(one.product_code, app.product_code);
        }
        assert!(super::find("hkcu:robinx-test-nonexistent-key").is_none());
        assert!(super::find("nope").is_none());
    }

    #[test]
    fn scan_returns_named_entries() {
        let apps = super::scan_all();
        assert!(apps.iter().all(|a| !a.display_name.is_empty()));
    }
}
