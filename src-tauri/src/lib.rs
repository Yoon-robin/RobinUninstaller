//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use robin_core::InstalledApplication;
use robin_execution::{UninstallEvent, UninstallOutcome};
use robin_scanner::{Leftover, LeftoverKind, ScanReport};
use tauri::ipc::Channel;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{Manager, State};

#[derive(Default)]
struct AppCache {
    apps: Mutex<HashMap<String, InstalledApplication>>,
    icons: Mutex<HashMap<String, Option<String>>>,
    /// 이번 실행 중에 제거한 프로그램 (제거된 뒤에도 잔여물을 찾을 수 있도록 정보를 남겨 둔다)
    removed: Mutex<Vec<InstalledApplication>>,
    /// 마지막 스캔 결과의 항목. 탐색기로 열기 같은 요청은 여기 있는 경로만 허용한다.
    last_scan: Mutex<Vec<Leftover>>,
}

impl AppCache {
    fn get(&self, id: &str) -> Option<InstalledApplication> {
        self.apps.lock().ok()?.get(id).cloned()
    }

    fn get_removed(&self, id: &str) -> Option<InstalledApplication> {
        self.removed
            .lock()
            .ok()?
            .iter()
            .find(|a| a.id == id)
            .cloned()
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
/// 사용자가 "기다리지 않고 확인"을 눌렀는지.
static STOP_WAITING: AtomicBool = AtomicBool::new(false);

/// 언인스톨러 프로세스를 더 기다리지 않고 바로 제거 여부를 확인한다.
#[tauri::command]
fn stop_waiting_uninstall() {
    STOP_WAITING.store(true, Ordering::SeqCst);
}

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
    let snapshot = app.clone();
    if UNINSTALLING.swap(true, Ordering::SeqCst) {
        return Err("이미 다른 프로그램을 제거하고 있어요".into());
    }
    STOP_WAITING.store(false, Ordering::SeqCst);

    let result = tauri::async_runtime::spawn_blocking(move || {
        robin_execution::run_uninstaller(
            &app,
            &STOP_WAITING,
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
    let outcome = result.map_err(|e| e.to_string())?;

    if matches!(
        outcome,
        UninstallOutcome::Removed | UninstallOutcome::RemovedRebootRequired
    ) && let Ok(mut removed) = cache.removed.lock()
    {
        removed.retain(|a| a.id != snapshot.id);
        removed.insert(0, snapshot);
    }
    Ok(outcome)
}

/// 이번 실행 중에 제거한 프로그램 (최근 것부터).
#[tauri::command]
fn recently_removed(cache: State<'_, AppCache>) -> Vec<InstalledApplication> {
    cache.removed.lock().map(|r| r.clone()).unwrap_or_default()
}

/// 남은 파일·레지스트리 후보를 찾는다 (읽기 전용). 제거된 프로그램과 설치된 프로그램 모두 가능.
#[tauri::command]
async fn scan_leftovers(id: String, cache: State<'_, AppCache>) -> Result<ScanReport, String> {
    let app = cache
        .get_removed(&id)
        .or_else(|| cache.get(&id))
        .ok_or("알 수 없는 프로그램이에요")?;

    let report = tauri::async_runtime::spawn_blocking(move || {
        robin_scanner::scan(&app, &robin_discovery::list_installed())
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Ok(mut last) = cache.last_scan.lock() {
        *last = report.items.clone();
    }
    Ok(report)
}

/// 스캔 결과의 파일·폴더를 탐색기에서 선택해 보여 준다. 마지막 스캔에 있던 항목만 허용한다.
#[tauri::command]
fn reveal_leftover(item_id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let item = cache
        .last_scan
        .lock()
        .ok()
        .and_then(|items| items.iter().find(|i| i.id == item_id).cloned())
        .ok_or("스캔 결과에 없는 항목이에요")?;
    if !matches!(item.kind, LeftoverKind::Folder | LeftoverKind::File) {
        return Err("파일이나 폴더만 열 수 있어요".into());
    }
    if !Path::new(&item.location).exists() {
        return Err("이미 없는 항목이에요".into());
    }
    // explorer는 `/select,"경로"` 형식만 제대로 해석한다. 경로는 스캔 결과에서 온 것이라 따옴표가 들어갈 수 없다.
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", item.location))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
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
            // 창은 프론트엔드가 첫 화면을 그린 뒤 보여 준다. 프론트엔드가 실패해도 창이 사라지지 않도록 안전장치.
            if let Some(window) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let _ = window.show();
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_installed_apps,
            get_app_icon,
            open_install_location,
            uninstall_app,
            stop_waiting_uninstall,
            recently_removed,
            scan_leftovers,
            reveal_leftover,
            window_material
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
