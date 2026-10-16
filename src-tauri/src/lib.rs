//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use robin_core::InstalledApplication;
use robin_execution::{UninstallEvent, UninstallOutcome};
use tauri::ipc::Channel;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{Manager, State};

#[derive(Default)]
struct AppCache {
    apps: Mutex<HashMap<String, InstalledApplication>>,
    icons: Mutex<HashMap<String, Option<String>>>,
}

impl AppCache {
    fn get(&self, id: &str) -> Option<InstalledApplication> {
        self.apps.lock().ok()?.get(id).cloned()
    }
}

/// 설치된 프로그램 목록 (읽기 전용).
#[tauri::command]
async fn list_installed_apps(
    cache: State<'_, AppCache>,
) -> Result<Vec<InstalledApplication>, String> {
    let apps = tauri::async_runtime::spawn_blocking(robin_discovery::list_installed)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(mut map) = cache.apps.lock() {
        *map = apps.iter().map(|a| (a.id.clone(), a.clone())).collect();
    }
    Ok(apps)
}

/// 프로그램 아이콘 (PNG data URL). 없으면 `None`.
#[tauri::command]
async fn get_app_icon(id: String, cache: State<'_, AppCache>) -> Result<Option<String>, String> {
    if let Some(hit) = cache.icons.lock().ok().and_then(|m| m.get(&id).cloned()) {
        return Ok(hit);
    }
    let Some(app) = cache.get(&id) else {
        return Ok(None);
    };

    let icon = tauri::async_runtime::spawn_blocking(move || robin_discovery::app_icon(&app))
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(mut map) = cache.icons.lock() {
        map.insert(id, icon.clone());
    }
    Ok(icon)
}

/// 한 번에 하나만 제거한다 (Windows Installer도 동시에 하나만 처리한다).
static UNINSTALLING: AtomicBool = AtomicBool::new(false);

/// 기본 언인스톨러를 실행하고 끝날 때까지 기다린다. 진행 상황은 `on_event`로 보낸다.
#[tauri::command]
async fn uninstall_app(
    id: String,
    on_event: Channel<UninstallEvent>,
    cache: State<'_, AppCache>,
) -> Result<UninstallOutcome, String> {
    let app = cache
        .get(&id)
        .ok_or("알 수 없는 프로그램이에요. 목록을 새로고침해 주세요")?;
    if UNINSTALLING.swap(true, Ordering::SeqCst) {
        return Err("이미 다른 프로그램을 제거하고 있어요".into());
    }

    let result = tauri::async_runtime::spawn_blocking(move || {
        robin_execution::run_uninstaller(
            &app,
            |event| {
                let _ = on_event.send(event);
            },
            || {
                robin_discovery::list_installed()
                    .iter()
                    .any(|a| a.id == app.id)
            },
        )
    })
    .await;

    UNINSTALLING.store(false, Ordering::SeqCst);
    result.map_err(|e| e.to_string())
}

/// 설치 폴더를 탐색기로 연다.
#[tauri::command]
fn open_install_location(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let app = cache.get(&id).ok_or("알 수 없는 프로그램이에요")?;
    let dir = app
        .install_location
        .ok_or("설치 폴더가 기록돼 있지 않아요")?;
    let dir = dir.trim_matches('"');
    if !Path::new(dir).is_dir() {
        return Err("설치 폴더가 존재하지 않아요".into());
    }

    std::process::Command::new("explorer.exe")
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Mica 배경이 켜졌는지. 프론트엔드는 이 값을 보고 배경을 반투명하게 그린다.
static MICA_ENABLED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn window_material() -> &'static str {
    if MICA_ENABLED.load(Ordering::Relaxed) {
        "mica"
    } else {
        "solid"
    }
}

/// Mica는 Windows 11(빌드 22000 이상)에서만 지원된다.
#[cfg(windows)]
fn supports_mica() -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|k| k.get_value::<String, _>("CurrentBuildNumber"))
        .ok()
        .and_then(|b| b.parse::<u32>().ok())
        .is_some_and(|build| build >= 22000)
}

#[cfg(not(windows))]
fn supports_mica() -> bool {
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppCache::default())
        .setup(|app| {
            if supports_mica()
                && let Some(window) = app.get_webview_window("main")
                && window
                    .set_effects(EffectsBuilder::new().effect(Effect::Mica).build())
                    .is_ok()
            {
                MICA_ENABLED.store(true, Ordering::Relaxed);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_installed_apps,
            get_app_icon,
            open_install_location,
            uninstall_app,
            window_material
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
