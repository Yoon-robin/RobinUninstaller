//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

mod settings;
mod tracking;
pub mod worker;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use robin_core::{InstalledApplication, StoreApp};
use robin_execution::{UninstallEvent, UninstallOutcome};
use robin_recovery::{Guard, Operation, Store};
use robin_scanner::{Leftover, LeftoverKind, ScanReport};
use robin_startup::{StartupEntry, ToggleError};
use tauri::ipc::Channel;
use tauri::window::{Effect, EffectsBuilder};
use tauri::{Manager, State};

/// 스캔 한 번의 결과. 정리 요청은 같은 프로그램의 스캔 결과에 있는 항목만 받는다
/// (다른 프로그램을 스캔한 뒤 이전 화면의 정리 요청이 들어와도 섞이지 않게).
#[derive(Default)]
pub(crate) struct ScanSession {
    app_id: String,
    items: Vec<Leftover>,
}

#[derive(Default)]
pub(crate) struct AppCache {
    apps: Mutex<HashMap<String, InstalledApplication>>,
    icons: Mutex<HashMap<String, Option<String>>>,
    /// 마지막으로 읽은 Store·MSIX 앱 (아이콘 요청은 여기 있는 앱만 허용)
    store_apps: Mutex<HashMap<String, StoreApp>>,
    /// 마지막으로 직접 잰 설치 폴더 크기 (앱 ID → KB, 목록 내보내기에 쓴다)
    sizes: Mutex<HashMap<String, u64>>,
    /// 이번 실행 중에 제거한 프로그램 (제거된 뒤에도 잔여물을 찾을 수 있도록 정보를 남겨 둔다)
    removed: Mutex<Vec<InstalledApplication>>,
    /// 마지막 스캔(어느 프로그램을 스캔했는지 + 결과 항목). 정리·탐색기로 열기는 여기 있는 항목만 허용한다.
    pub(crate) last_scan: Mutex<ScanSession>,
    /// 마지막으로 읽은 시작 항목. 아이콘·탐색기 요청은 여기 있는 항목만 허용한다.
    startup: Mutex<Vec<StartupEntry>>,
    /// 폴더 분석에서 화면에 보낸 항목 ID → 경로. 화면은 경로 대신 이 ID만 넘긴다.
    folders: Mutex<HashMap<String, PathBuf>>,
    next_folder: AtomicU64,
    /// 진행 중인 폴더 분석의 취소 신호 (새 분석을 시작하면 이전 것을 멈춘다)
    analysis_cancel: Mutex<Option<Arc<AtomicBool>>>,
    /// 마지막 저장공간 스캔 결과 (탐색기로 열기는 여기 있는 위치만 허용)
    storage: Mutex<Vec<robin_storage::Category>>,
}

impl AppCache {
    fn get(&self, id: &str) -> Option<InstalledApplication> {
        self.apps.lock().ok()?.get(id).cloned()
    }

    fn get_startup(&self, id: &str) -> Option<StartupEntry> {
        self.startup
            .lock()
            .ok()?
            .iter()
            .find(|e| e.id == id)
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

/// 크기 기록이 없는 프로그램의 설치 폴더 크기 (앱 ID → KB, 읽기 전용).
/// 보호 경로와 여러 프로그램이 같이 쓰는 폴더는 재지 않는다.
#[tauri::command]
async fn measure_app_sizes(cache: State<'_, AppCache>) -> Result<HashMap<String, u64>, String> {
    let apps: Vec<InstalledApplication> = cache
        .apps
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    let sizes = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        return robin_scanner::install_size::install_sizes(
            &apps,
            &robin_scanner::protect::KnownFolders::from_env(),
        );
        #[cfg(not(windows))]
        {
            let _ = apps;
            HashMap::new()
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    if let Ok(mut map) = cache.sizes.lock() {
        *map = sizes.clone();
    }
    Ok(sizes)
}

/// 프로그램 목록(데스크톱 + Store 앱)을 CSV로 저장한다. 저장 위치는 사용자가 저장 창에서 고른다.
/// 저장했으면 파일 이름, 취소했으면 `None`.
#[tauri::command]
async fn export_app_list(
    app: tauri::AppHandle,
    cache: State<'_, AppCache>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let mut apps: Vec<InstalledApplication> = cache
        .apps
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    if apps.is_empty() {
        return Err("프로그램 목록을 먼저 불러와 주세요".into());
    }
    apps.sort_by_key(|a| a.display_name.to_lowercase());
    let mut store: Vec<StoreApp> = cache
        .store_apps
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    store.sort_by_key(|a| a.display_name.to_lowercase());
    let sizes = cache.sizes.lock().map(|m| m.clone()).unwrap_or_default();
    let csv = robin_discovery::export::programs_csv(&apps, &sizes, &store);

    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| {
            robin_discovery::package_info::ticks_to_date(
                d.as_secs() as i64 * 10_000_000 + 116_444_736_000_000_000,
            )
        })
        .unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(picked) = app
            .dialog()
            .file()
            .set_title("프로그램 목록 저장")
            .set_file_name(format!("프로그램 목록 {today}.csv").trim())
            .add_filter("CSV (Excel)", &["csv"])
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|e| e.to_string())?;
        std::fs::write(&path, csv).map_err(|e| format!("저장하지 못했어요: {e}"))?;
        Ok(path.file_name().map(|n| n.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 프로그램 이름으로 웹 검색을 연다 (기본 브라우저).
#[tauri::command]
fn search_app_web(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let (name, publisher) = match cache.get(&id) {
        Some(a) => (a.display_name, a.publisher),
        None => cache
            .store_apps
            .lock()
            .ok()
            .and_then(|m| {
                m.get(&id)
                    .map(|a| (a.display_name.clone(), a.publisher.clone()))
            })
            .ok_or("알 수 없는 프로그램이에요")?,
    };
    let url = robin_discovery::export::search_url(&name, publisher.as_deref());
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Microsoft Store·MSIX 앱 목록 (읽기 전용).
#[tauri::command]
async fn list_store_apps(cache: State<'_, AppCache>) -> Result<Vec<StoreApp>, String> {
    let apps = tauri::async_runtime::spawn_blocking(robin_discovery::list_store_apps)
        .await
        .map_err(|e| e.to_string())?;

    if let Ok(mut map) = cache.store_apps.lock() {
        *map = apps.iter().map(|a| (a.id.clone(), a.clone())).collect();
    }
    Ok(apps)
}

/// Store 앱 로고 (PNG data URL). 아이콘 캐시는 프로그램 아이콘과 같이 쓴다 (ID 앞부분이 달라 겹치지 않는다).
#[tauri::command]
async fn get_store_app_icon(
    id: String,
    cache: State<'_, AppCache>,
) -> Result<Option<String>, String> {
    if let Some(hit) = cache.icons.lock().ok().and_then(|m| m.get(&id).cloned()) {
        return Ok(hit);
    }
    let Some(app) = cache
        .store_apps
        .lock()
        .ok()
        .and_then(|m| m.get(&id).cloned())
    else {
        return Ok(None);
    };
    let icon = tauri::async_runtime::spawn_blocking(move || robin_discovery::store_app_icon(&app))
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(mut map) = cache.icons.lock() {
        map.insert(id, icon.clone());
    }
    Ok(icon)
}

/// Windows 설정의 "설치된 앱" 화면을 연다. Store 앱 제거는 거기서 한다.
#[tauri::command]
fn open_apps_settings() -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg("ms-settings:appsfeatures")
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
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
        *last = ScanSession {
            app_id: id,
            items: report.items.clone(),
        };
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
        .and_then(|scan| scan.items.iter().find(|i| i.id == item_id).cloned())
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

pub(crate) fn store() -> Result<Store, String> {
    Store::default_location().ok_or_else(|| "백업 위치를 찾을 수 없어요".to_string())
}

fn guard(store: &Store) -> Guard {
    Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    }
}

/// 마지막 스캔 결과에서 고른 항목을 백업·격리한 뒤 원래 자리에서 없앤다.
/// 권한이 부족한 항목은 `needsAdmin`으로 남는다 → `clean_leftovers_elevated`.
#[tauri::command]
async fn clean_leftovers(
    app_id: String,
    app_name: String,
    item_ids: Vec<String>,
    force_app_id: Option<String>,
    cache: State<'_, AppCache>,
) -> Result<Operation, String> {
    let items: Vec<Leftover> = {
        let scan = cache.last_scan.lock().map_err(|e| e.to_string())?;
        if scan.app_id != app_id {
            return Err(
                "다른 프로그램의 스캔 결과예요. 남은 흔적을 다시 찾은 뒤 시도해 주세요".into(),
            );
        }
        scan.items
            .iter()
            .filter(|i| item_ids.contains(&i.id))
            .cloned()
            .collect()
    };
    if items.is_empty() {
        return Err("정리할 항목이 없어요. 다시 찾은 뒤 시도해 주세요".into());
    }
    // 강제 제거는 화면에서 이미 확인했어도 여기서 다시 확인한다 (보안 프로그램·실행 중인 프로그램).
    // 대상을 찾지 못하면 확인을 건너뛰지 않고 멈춘다.
    if let Some(force_id) = force_app_id.as_deref() {
        if force_id != app_id {
            return Err("강제 제거 대상이 스캔한 프로그램과 달라요".into());
        }
        let app = cache
            .get(force_id)
            .ok_or("강제 제거할 프로그램을 찾지 못했어요. 목록을 새로고침해 주세요")?;
        let check =
            tauri::async_runtime::spawn_blocking(move || robin_execution::force::check(&app))
                .await
                .map_err(|e| e.to_string())?;
        if let Some(reason) = check.blocked {
            return Err(reason);
        }
        if !check.running.is_empty() {
            return Err("프로그램이 실행 중이에요. 닫은 뒤 다시 시도해 주세요".into());
        }
    }

    let op = tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        let mut op = robin_recovery::new_operation(&app_name, &items);
        store
            .save(&op)
            .map_err(|e| format!("작업 기록을 만들지 못했어요: {e}"))?;
        robin_recovery::clean(&store, &mut op, &guard(&store), |_| {})?;
        Ok::<_, String>(op)
    })
    .await
    .map_err(|e| e.to_string())??;

    note_force_removed(&cache, force_app_id).await;
    Ok(op)
}

/// 강제 제거로 등록 정보가 사라졌으면 "제거한 프로그램"으로 옮겨, 나중에 남은 흔적을 다시 찾을 수 있게 한다.
async fn note_force_removed(cache: &AppCache, force_app_id: Option<String>) {
    let Some(app) = force_app_id.and_then(|id| cache.get(&id)) else {
        return;
    };
    let id = app.id.clone();
    let still_installed = tauri::async_runtime::spawn_blocking(move || {
        robin_discovery::list_installed().iter().any(|a| a.id == id)
    })
    .await
    .unwrap_or(true);
    if !still_installed && let Ok(mut removed) = cache.removed.lock() {
        removed.retain(|a| a.id != app.id);
        removed.insert(0, app);
    }
}

/// 강제 제거 전 확인: 대상에서 빼야 하는 프로그램인지, 설치 폴더 안에서 실행 중인 프로세스가 있는지.
#[tauri::command]
async fn force_uninstall_check(
    id: String,
    cache: State<'_, AppCache>,
) -> Result<robin_execution::force::ForceCheck, String> {
    let app = cache
        .get(&id)
        .ok_or("알 수 없는 프로그램이에요. 목록을 새로고침해 주세요")?;
    tauri::async_runtime::spawn_blocking(move || robin_execution::force::check(&app))
        .await
        .map_err(|e| e.to_string())
}

/// 관리자 권한 작업자를 띄워(UAC) 남은 항목을 처리하고, 갱신된 기록을 돌려준다.
fn run_worker(mode: &str, op_id: &str, extra: &str) -> Result<Operation, String> {
    let store = store()?;
    if !robin_recovery::store::is_valid_id(op_id) {
        return Err("잘못된 작업이에요".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = format!(
        "{mode} {op_id} --store \"{}\"{extra}",
        store.root().display()
    );
    match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
        // 일부 항목만 실패: 기록에 항목별 이유가 있으니 그대로 보여 준다
        Ok(Some(code)) if code == worker::EXIT_SOME_FAILED as u32 => {
            store.load(op_id).map_err(|e| e.to_string())
        }
        Ok(Some(code)) if code == worker::EXIT_SAVE_FAILED as u32 => Err(
            "관리자 권한 작업 중 기록을 저장하지 못해 멈췄어요. 복구 기록을 확인해 주세요".into(),
        ),
        // 작업자가 기록을 읽지 못하거나 인자를 거부하면 0이 아닌 코드로 끝난다
        Ok(Some(code)) if code != 0 => Err(format!(
            "관리자 권한 작업이 끝나지 못했어요 (코드 {code}). 복구 기록을 확인해 주세요"
        )),
        Ok(_) => store.load(op_id).map_err(|e| e.to_string()),
        Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
        Err(Some(m)) => Err(m),
    }
}

#[tauri::command]
async fn clean_leftovers_elevated(
    op_id: String,
    force_app_id: Option<String>,
    cache: State<'_, AppCache>,
) -> Result<Operation, String> {
    let op = tauri::async_runtime::spawn_blocking(move || run_worker(worker::CLEAN, &op_id, ""))
        .await
        .map_err(|e| e.to_string())??;
    note_force_removed(&cache, force_app_id).await;
    Ok(op)
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
        robin_recovery::restore(&store, &mut op, &guard(&store), item_ids.as_deref(), |_| {})?;
        Ok(op)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 관리자 권한으로 되돌린다. `item_ids`가 있으면 그 항목만 (작업자에게는 항목 번호만 넘긴다).
#[tauri::command]
async fn restore_backup_elevated(
    op_id: String,
    item_ids: Option<Vec<String>>,
) -> Result<Operation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let extra = match item_ids {
            None => String::new(),
            Some(ids) => {
                let op = store()?.load(&op_id).map_err(|e| e.to_string())?;
                let indices: Vec<String> = op
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| ids.contains(&item.id))
                    .map(|(i, _)| i.to_string())
                    .collect();
                if indices.is_empty() {
                    return Err("되돌릴 항목이 없어요".to_string());
                }
                format!(" {} {}", worker::ITEMS, indices.join(","))
            }
        };
        run_worker(worker::RESTORE, &op_id, &extra)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 백업을 완전히 지운다. 이후에는 복원할 수 없다 (사용자가 확인한 뒤에만 부른다).
#[tauri::command]
async fn delete_backup(op_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        match store.delete(&op_id) {
            Ok(()) => Ok(()),
            // 관리자 권한으로 격리한 폴더는 원래 권한을 그대로 갖고 있어 일반 권한으로 못 지울 수 있다
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let args = format!(
                    "{} {op_id} --store \"{}\"",
                    worker::DELETE,
                    store.root().display()
                );
                match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
                    Ok(Some(0)) | Ok(None) if !store.dir(&op_id).exists() => Ok(()),
                    Ok(_) => Err("관리자 권한으로도 백업을 다 지우지 못했어요".into()),
                    Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
                    Err(Some(m)) => Err(m),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

fn startup_history() -> Result<robin_startup::History, String> {
    robin_startup::History::default_location()
        .ok_or_else(|| "시작프로그램 기록 위치를 찾을 수 없어요".to_string())
}

/// 시작프로그램 목록 (읽기 전용).
#[tauri::command]
async fn list_startup_items(cache: State<'_, AppCache>) -> Result<Vec<StartupEntry>, String> {
    let items = tauri::async_runtime::spawn_blocking(|| {
        startup_history().map(|history| robin_startup::list(&history))
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Ok(mut startup) = cache.startup.lock() {
        *startup = items.clone();
    }
    Ok(items)
}

/// 실행 파일이 없는 시작 항목(Run 값·시작 폴더 파일)을 짝인 꺼짐 상태 값과 함께 백업·격리한 뒤 정리한다.
/// 항목은 ID로만 받고, 엔진이 목록을 다시 읽어 실행 파일이 정말 없는지 확인한다. 예약 작업은 정리하지 않는다.
/// 관리자 권한이 필요한 항목은 `needsAdmin`으로 남는다 → `clean_leftovers_elevated`.
#[tauri::command]
async fn clean_missing_startup(ids: Vec<String>) -> Result<Operation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let history = startup_history()?;
        let entries = robin_startup::list(&history);
        let items: Vec<Leftover> = ids
            .iter()
            .filter_map(|id| entries.iter().find(|e| &e.id == id))
            .filter_map(|e| robin_recovery::startup::leftovers_for(e).ok())
            .flatten()
            .collect();
        if items.is_empty() {
            return Err("정리할 항목이 없어요. 목록을 새로고침해 주세요".to_string());
        }
        let store = store()?;
        let mut op = robin_recovery::new_operation(robin_recovery::startup::OPERATION_NAME, &items);
        store
            .save(&op)
            .map_err(|e| format!("작업 기록을 만들지 못했어요: {e}"))?;
        robin_recovery::clean(&store, &mut op, &guard(&store), |_| {})?;
        Ok(op)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 시작 항목이 실행하는 파일의 아이콘. 없으면 `None`.
#[tauri::command]
async fn get_startup_icon(
    id: String,
    cache: State<'_, AppCache>,
) -> Result<Option<String>, String> {
    let key = format!("startup:{id}");
    if let Some(hit) = cache.icons.lock().ok().and_then(|m| m.get(&key).cloned()) {
        return Ok(hit);
    }
    let Some(target) = cache
        .get_startup(&id)
        .filter(|e| !e.target_missing)
        .and_then(|e| e.target_path)
    else {
        return Ok(None);
    };

    let icon = tauri::async_runtime::spawn_blocking(move || robin_discovery::file_icon(&target))
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(mut map) = cache.icons.lock() {
        map.insert(key, icon.clone());
    }
    Ok(icon)
}

/// 시작 항목을 끄거나 다시 켠다. HKLM·공용 폴더 항목과 다른 계정의 예약 작업은 관리자 권한 작업자(UAC)로 처리한다.
#[tauri::command]
async fn set_startup_enabled(
    id: String,
    enabled: bool,
    cache: State<'_, AppCache>,
) -> Result<StartupEntry, String> {
    let entry = tauri::async_runtime::spawn_blocking(move || {
        let history = startup_history()?;
        match robin_startup::set_enabled(&history, &id, enabled) {
            Ok(entry) => Ok(entry),
            // ID는 set_enabled가 이미 검사했다 (형식이 틀리면 NotFound)
            Err(ToggleError::NeedsAdmin) => {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let args = format!(
                    "{} {} {id} --store \"{}\"",
                    worker::STARTUP,
                    if enabled { "on" } else { "off" },
                    history.root().display()
                );
                match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
                    Ok(_) => robin_startup::list(&history)
                        .into_iter()
                        .find(|e| e.id == id && e.enabled == enabled)
                        .ok_or_else(|| "관리자 권한으로도 바꾸지 못했어요".to_string()),
                    Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
                    Err(Some(m)) => Err(m),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Ok(mut startup) = cache.startup.lock()
        && let Some(slot) = startup.iter_mut().find(|e| e.id == entry.id)
    {
        *slot = entry.clone();
    }
    Ok(entry)
}

/// 시작 항목을 탐색기에서 보여 준다: 시작 폴더 항목은 그 파일, Run 항목과 예약 작업은 실행 파일.
#[tauri::command]
fn reveal_startup_item(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let entry = cache
        .get_startup(&id)
        .ok_or("시작프로그램 목록에 없는 항목이에요")?;
    let path = match entry.source {
        robin_startup::StartupSource::FolderUser | robin_startup::StartupSource::FolderCommon => {
            Path::new(&entry.location)
                .join(&entry.approved_name)
                .to_string_lossy()
                .into_owned()
        }
        _ => entry
            .target_path
            .filter(|_| !entry.target_missing)
            .ok_or("실행 파일을 찾을 수 없어요")?,
    };
    if !Path::new(&path).exists() {
        return Err("파일이 이미 없어요".into());
    }
    // 경로는 엔진이 읽은 목록에서 온 것이다. 파일 이름에는 따옴표가 들어갈 수 없다.
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{path}\""))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 시스템 드라이브 (`C:\`)
fn system_drive() -> String {
    format!(
        "{}\\",
        std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into())
    )
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DiskUsage {
    drive: String,
    total: u64,
    free: u64,
}

/// 시스템 드라이브 용량
#[tauri::command]
fn disk_usage() -> Result<DiskUsage, String> {
    let drive = system_drive();
    let (total, free) =
        robin_storage::system::disk_space(&drive).ok_or("드라이브 용량을 읽지 못했어요")?;
    Ok(DiskUsage { drive, total, free })
}

/// 정리할 수 있는 항목 (읽기 전용).
#[tauri::command]
async fn scan_storage(cache: State<'_, AppCache>) -> Result<Vec<robin_storage::Category>, String> {
    let items = tauri::async_runtime::spawn_blocking(robin_storage::scan)
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(mut storage) = cache.storage.lock() {
        *storage = items.clone();
    }
    Ok(items)
}

/// 고른 항목을 정리한다. 다시 만들어지는 항목만 기본이고, 되돌릴 수 없는 항목은 `allow_irreversible`이 있어야 한다.
/// 관리자 권한이 필요한 항목은 `needsAdmin`으로 돌아온다 → `clean_storage_elevated`.
#[tauri::command]
async fn clean_storage(
    ids: Vec<String>,
    allow_irreversible: bool,
) -> Result<Vec<robin_storage::CleanResult>, String> {
    tauri::async_runtime::spawn_blocking(move || robin_storage::clean(&ids, allow_irreversible))
        .await
        .map_err(|e| e.to_string())
}

/// 관리자 권한 작업자를 띄워(UAC) 정리한다. 작업자는 규칙 ID만 받고 결과를 파일로 남긴다.
#[tauri::command]
async fn clean_storage_elevated(
    ids: Vec<String>,
    allow_irreversible: bool,
) -> Result<Vec<robin_storage::CleanResult>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if ids.is_empty() || !ids.iter().all(|id| robin_storage::rules::is_valid_id(id)) {
            return Err("알 수 없는 항목이에요".to_string());
        }
        let dir = PathBuf::from(std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?)
            .join(robin_core::APP_DIR)
            .join("storage");
        let result_file = dir.join(worker::STORAGE_RESULT);
        let _ = std::fs::remove_file(&result_file);

        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let args = format!(
            "{} {} {} --store \"{}\"",
            worker::STORAGE,
            if allow_irreversible { "1" } else { "0" },
            ids.join(","),
            dir.display()
        );
        match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
            Ok(_) => {
                let text = std::fs::read_to_string(&result_file)
                    .map_err(|_| "관리자 권한으로 정리하지 못했어요".to_string())?;
                let _ = std::fs::remove_file(&result_file);
                serde_json::from_str(&text).map_err(|e| e.to_string())
            }
            Err(None) => Err("관리자 권한 요청이 거절됐어요".into()),
            Err(Some(m)) => Err(m),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 정리 항목의 첫 위치를 탐색기로 연다. 마지막 스캔에 있던 위치만 허용한다.
#[tauri::command]
fn reveal_storage_category(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let location = cache
        .storage
        .lock()
        .ok()
        .and_then(|items| {
            items
                .iter()
                .find(|c| c.id == id)?
                .locations
                .first()
                .cloned()
        })
        .ok_or("열 수 있는 위치가 없어요")?;
    open_in_explorer(Path::new(&location))
}

/// Windows 설정의 저장소 화면을 연다 (Windows.old 등 직접 정리 안내용)
#[tauri::command]
fn open_storage_settings() -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg("ms-settings:storagesense")
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 탐색기로 연다. 폴더면 그 폴더를, 파일이면 파일을 선택한 채로.
fn open_in_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("이미 없는 항목이에요".into());
    }
    let mut cmd = std::process::Command::new("explorer.exe");
    if path.is_dir() {
        cmd.arg(path);
    } else {
        // 경로는 엔진이 읽은 것이다. 파일 이름에는 따옴표가 들어갈 수 없다.
        use std::os::windows::process::CommandExt;
        cmd.raw_arg(format!("/select,\"{}\"", path.display()));
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderItem {
    id: String,
    #[serde(flatten)]
    entry: robin_storage::FolderEntry,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderAnalysis {
    id: String,
    path: String,
    /// 상위 폴더 ID (드라이브 루트면 없음)
    parent_id: Option<String>,
    items: Vec<FolderItem>,
    cancelled: bool,
}

impl AppCache {
    fn folder_id(&self, path: &Path) -> String {
        let mut map = self.folders.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((id, _)) = map.iter().find(|(_, p)| p.as_path() == path) {
            return id.clone();
        }
        let id = format!("f{}", self.next_folder.fetch_add(1, Ordering::Relaxed));
        map.insert(id.clone(), path.to_path_buf());
        id
    }

    fn folder_path(&self, id: &str) -> Option<PathBuf> {
        self.folders.lock().ok()?.get(id).cloned()
    }
}

/// 폴더 바로 아래 항목의 크기 (읽기 전용). `id`가 없으면 시스템 드라이브.
#[tauri::command]
async fn analyze_folder(
    id: Option<String>,
    cache: State<'_, AppCache>,
) -> Result<FolderAnalysis, String> {
    let path = match &id {
        Some(id) => cache.folder_path(id).ok_or("알 수 없는 폴더예요")?,
        None => PathBuf::from(system_drive()),
    };
    if !path.is_dir() {
        return Err("폴더가 없어요".into());
    }

    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut slot) = cache.analysis_cancel.lock()
        && let Some(previous) = slot.replace(cancel.clone())
    {
        previous.store(true, Ordering::Relaxed);
    }

    let (entries, cancelled) = {
        let path = path.clone();
        let cancel = cancel.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let entries = robin_storage::analyze(&path, &cancel);
            (entries, cancel.load(Ordering::Relaxed))
        })
        .await
        .map_err(|e| e.to_string())?
    };

    let items = entries
        .into_iter()
        .map(|entry| FolderItem {
            id: cache.folder_id(Path::new(&entry.path)),
            entry,
        })
        .collect();
    Ok(FolderAnalysis {
        id: cache.folder_id(&path),
        path: path.display().to_string(),
        parent_id: path.parent().map(|p| cache.folder_id(p)),
        items,
        cancelled,
    })
}

/// 진행 중인 폴더 분석을 멈춘다.
#[tauri::command]
fn cancel_folder_analysis(cache: State<'_, AppCache>) {
    if let Ok(slot) = cache.analysis_cancel.lock()
        && let Some(cancel) = slot.as_ref()
    {
        cancel.store(true, Ordering::Relaxed);
    }
}

/// 분석 결과의 항목을 탐색기로 연다.
#[tauri::command]
fn reveal_folder_item(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    open_in_explorer(&cache.folder_path(&id).ok_or("알 수 없는 항목이에요")?)
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
        .plugin(tauri_plugin_dialog::init())
        .manage(AppCache::default())
        .manage(tracking::TrackingState::default())
        .setup(|app| {
            // 사용자가 보관 기간을 골랐으면 기간이 지난 백업을 정리한다 (기본은 계속 보관이라 아무것도 하지 않는다)
            std::thread::spawn(|| {
                let _ = settings::prune_expired();
            });
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
            measure_app_sizes,
            export_app_list,
            search_app_web,
            list_store_apps,
            get_store_app_icon,
            open_apps_settings,
            open_install_location,
            uninstall_app,
            stop_waiting_uninstall,
            recently_removed,
            scan_leftovers,
            reveal_leftover,
            clean_leftovers,
            clean_leftovers_elevated,
            force_uninstall_check,
            list_backups,
            restore_backup,
            restore_backup_elevated,
            delete_backup,
            list_startup_items,
            get_startup_icon,
            set_startup_enabled,
            clean_missing_startup,
            settings::get_settings,
            settings::set_backup_retention,
            settings::add_exclusion,
            settings::remove_exclusion,
            settings::backup_usage,
            settings::prune_backups,
            tracking::tracking_status,
            tracking::start_tracking,
            tracking::cancel_tracking,
            tracking::finish_tracking,
            tracking::save_trace,
            tracking::list_traces,
            tracking::delete_trace,
            reveal_startup_item,
            disk_usage,
            scan_storage,
            clean_storage,
            clean_storage_elevated,
            reveal_storage_category,
            open_storage_settings,
            analyze_folder,
            cancel_folder_analysis,
            reveal_folder_item,
            window_material
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
