//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

pub mod worker;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use robin_core::InstalledApplication;
use robin_execution::{UninstallEvent, UninstallOutcome};
use robin_recovery::{Guard, Operation, Store};
use robin_scanner::{Leftover, LeftoverKind, ScanReport};
use robin_startup::{SetError, StartupItem, StartupSource};
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
    /// 마지막으로 읽은 시작 항목. 아이콘·탐색기 요청은 여기 있는 경로만 쓴다.
    startup: Mutex<Vec<StartupItem>>,
}

impl AppCache {
    fn get(&self, id: &str) -> Option<InstalledApplication> {
        self.apps.lock().ok()?.get(id).cloned()
    }

    fn get_startup(&self, id: &str) -> Option<StartupItem> {
        self.startup
            .lock()
            .ok()?
            .iter()
            .find(|i| i.id == id)
            .cloned()
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

fn store() -> Result<Store, String> {
    Store::default_location().ok_or_else(|| "백업 위치를 찾을 수 없어요".to_string())
}

fn guard(store: &Store) -> Guard {
    Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
    }
}

/// 마지막 스캔 결과에서 고른 항목을 백업·격리한 뒤 원래 자리에서 없앤다.
/// 권한이 부족한 항목은 `needsAdmin`으로 남는다 → `clean_leftovers_elevated`.
#[tauri::command]
async fn clean_leftovers(
    app_name: String,
    item_ids: Vec<String>,
    cache: State<'_, AppCache>,
) -> Result<Operation, String> {
    let items: Vec<Leftover> = cache
        .last_scan
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|i| item_ids.contains(&i.id))
        .cloned()
        .collect();
    if items.is_empty() {
        return Err("정리할 항목이 없어요. 다시 찾은 뒤 시도해 주세요".into());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        let mut op = robin_recovery::new_operation(&app_name, &items);
        store
            .save(&op)
            .map_err(|e| format!("작업 기록을 만들지 못했어요: {e}"))?;
        robin_recovery::clean(&store, &mut op, &guard(&store), |_| {});
        Ok(op)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 관리자 권한 작업자를 띄워(UAC) 남은 항목을 처리하고, 갱신된 기록을 돌려준다.
fn run_worker(mode: &str, op_id: &str) -> Result<Operation, String> {
    let store = store()?;
    if !robin_recovery::store::is_valid_id(op_id) {
        return Err("잘못된 작업이에요".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = format!("{mode} {op_id} --store \"{}\"", store.root().display());
    match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
        Ok(_) => store.load(op_id).map_err(|e| e.to_string()),
        Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
        Err(Some(m)) => Err(m),
    }
}

#[tauri::command]
async fn clean_leftovers_elevated(op_id: String) -> Result<Operation, String> {
    tauri::async_runtime::spawn_blocking(move || run_worker(worker::CLEAN, &op_id))
        .await
        .map_err(|e| e.to_string())?
}

/// 복구 기록 (최근 작업부터).
#[tauri::command]
async fn list_backups() -> Result<Vec<Operation>, String> {
    tauri::async_runtime::spawn_blocking(|| Ok(store()?.list()))
        .await
        .map_err(|e| e.to_string())?
}

/// 작업에서 지운 항목을 되돌린다. `item_ids`가 없으면 전부.
#[tauri::command]
async fn restore_backup(op_id: String, item_ids: Option<Vec<String>>) -> Result<Operation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        let mut op = store.load(&op_id).map_err(|e| e.to_string())?;
        robin_recovery::restore(&store, &mut op, item_ids.as_deref(), |_| {});
        Ok(op)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn restore_backup_elevated(op_id: String) -> Result<Operation, String> {
    tauri::async_runtime::spawn_blocking(move || run_worker(worker::RESTORE, &op_id))
        .await
        .map_err(|e| e.to_string())?
}

/// 백업을 완전히 지운다. 이후에는 복원할 수 없다 (사용자가 확인한 뒤에만 부른다).
#[tauri::command]
async fn delete_backup(op_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || store()?.delete(&op_id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 시작 항목을 다시 읽어 캐시에 넣는다.
fn reload_startup(cache: &AppCache) -> Vec<StartupItem> {
    let items = robin_startup::list(&robin_discovery::list_installed());
    if let Ok(mut c) = cache.startup.lock() {
        *c = items.clone();
    }
    items
}

/// 시작프로그램 목록 (읽기 전용).
#[tauri::command]
async fn list_startup_items(app: tauri::AppHandle) -> Result<Vec<StartupItem>, String> {
    tauri::async_runtime::spawn_blocking(move || reload_startup(&app.state::<AppCache>()))
        .await
        .map_err(|e| e.to_string())
}

/// 시작 항목을 켜거나 끈다. 모든 사용자 항목은 관리자 권한 작업자(UAC)를 거친다.
/// 항목을 지우지 않고 켜짐·꺼짐 표시만 바꾸므로 언제든 되돌릴 수 있다. 바뀐 항목을 돌려준다.
#[tauri::command]
async fn set_startup_enabled(
    id: String,
    enabled: bool,
    app: tauri::AppHandle,
) -> Result<StartupItem, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let cache = app.state::<AppCache>();
        if !robin_startup::is_valid_id(&id) {
            return Err("잘못된 항목이에요".into());
        }
        match robin_startup::set_enabled(&id, enabled) {
            Ok(()) => {}
            Err(SetError::NotFound) => {
                return Err("이미 없는 항목이에요. 목록을 새로고침해 주세요".into());
            }
            Err(SetError::AccessDenied) => set_startup_elevated(&id, enabled)?,
            Err(SetError::Other(m)) => return Err(m),
        }
        reload_startup(&cache)
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| "바꾼 뒤 항목을 찾지 못했어요. 목록을 새로고침해 주세요".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn set_startup_elevated(id: &str, enabled: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // ID는 is_valid_id로 검사해 소문자·숫자·`-`·`:`만 들어 있다
    let args = format!(
        "{} {} {id}",
        worker::STARTUP,
        if enabled { "on" } else { "off" }
    );
    match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
        Ok(Some(code)) if code == worker::STARTUP_OK as u32 => Ok(()),
        Ok(Some(code)) if code == worker::STARTUP_NOT_FOUND as u32 => {
            Err("이미 없는 항목이에요. 목록을 새로고침해 주세요".into())
        }
        Ok(_) => Err("관리자 권한으로도 바꾸지 못했어요".into()),
        Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
        Err(Some(m)) => Err(m),
    }
}

/// 시작 항목 실행 파일의 아이콘 (PNG data URL).
#[tauri::command]
async fn get_startup_icon(
    id: String,
    cache: State<'_, AppCache>,
) -> Result<Option<String>, String> {
    let Some(path) = cache.get_startup(&id).and_then(|i| i.file_path) else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || robin_discovery::file_icon(&path))
        .await
        .map_err(|e| e.to_string())
}

/// 시작 항목의 실행 파일(시작 폴더 항목은 바로 가기 파일)을 탐색기에서 보여 준다.
#[tauri::command]
fn reveal_startup_item(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let item = cache
        .get_startup(&id)
        .ok_or("목록에 없는 항목이에요. 새로고침해 주세요")?;
    let path = match item.source {
        StartupSource::FolderUser | StartupSource::FolderCommon => Path::new(&item.location)
            .join(&item.name)
            .to_string_lossy()
            .into_owned(),
        _ => item.file_path.ok_or("실행 파일 위치를 알 수 없어요")?,
    };
    if !Path::new(&path).exists() {
        return Err("파일이 없어요".into());
    }
    if path.contains('"') {
        return Err("열 수 없는 경로예요".into());
    }
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{path}\""))
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
            clean_leftovers,
            clean_leftovers_elevated,
            list_backups,
            restore_backup,
            restore_backup_elevated,
            delete_backup,
            list_startup_items,
            set_startup_enabled,
            get_startup_icon,
            reveal_startup_item,
            window_material
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
