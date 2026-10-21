//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

mod removed;
mod settings;
#[cfg(windows)]
pub mod setup;

pub mod worker;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use robin_core::progress::{Progress, ProgressSnapshot};
use robin_core::{InstalledApplication, StoreApp};
use robin_execution::{UninstallEvent, UninstallOutcome};
use robin_recovery::{Guard, Operation, Store};
use robin_scanner::{Leftover, LeftoverKind, ScanReport};
use robin_startup::{StartupEntry, ToggleError};
use tauri::ipc::Channel;
use tauri::{Manager, State};

/// 프로그램별 마지막 스캔 결과. 정리 요청은 그 프로그램의 스캔 결과에 있는 항목만 받는다
/// (다른 프로그램을 스캔한 뒤 이전 화면의 정리 요청이 들어와도 섞이지 않게). 프로그램별로 두는 이유:
/// 닫은 창의 느린 스캔이 나중에 끝나도 지금 보는 프로그램의 결과를 덮어쓰지 않게.
#[derive(Default)]
pub(crate) struct ScanSession {
    by_app: HashMap<String, Vec<Leftover>>,
}

impl ScanSession {
    /// 어느 스캔 결과에든 있는 항목 (탐색기로 열기, 제외 목록에 더하기)
    pub(crate) fn find(&self, item_id: &str) -> Option<Leftover> {
        self.by_app
            .values()
            .flat_map(|items| items.iter())
            .find(|i| i.id == item_id)
            .cloned()
    }
}

#[derive(Default)]
pub(crate) struct AppCache {
    apps: Mutex<HashMap<String, InstalledApplication>>,
    icons: Mutex<HashMap<String, Option<String>>>,
    /// 마지막으로 읽은 Store·MSIX 앱 (아이콘 요청은 여기 있는 앱만 허용)
    store_apps: Mutex<HashMap<String, StoreApp>>,
    /// 마지막으로 직접 잰 설치 폴더 크기 (앱 ID → KB, 목록 내보내기에 쓴다)
    sizes: Mutex<HashMap<String, u64>>,
    /// 제거한 프로그램 (제거된 뒤에도 잔여물을 찾을 수 있도록 정보를 남겨 둔다).
    /// 데이터 폴더의 removed.json에도 저장해 앱을 다시 켜도 보인다.
    removed: Mutex<Vec<removed::RemovedApp>>,
    /// 마지막 스캔(어느 프로그램을 스캔했는지 + 결과 항목). 정리·탐색기로 열기는 여기 있는 항목만 허용한다.
    pub(crate) last_scan: Mutex<ScanSession>,
    /// 마지막으로 읽은 시작 항목. 아이콘·탐색기 요청은 여기 있는 항목만 허용한다.
    startup: Mutex<Vec<StartupEntry>>,
    /// 폴더 분석에서 화면에 보낸 항목 ID → 경로. 화면은 경로 대신 이 ID만 넘긴다.
    folders: Mutex<HashMap<String, PathBuf>>,
    next_folder: AtomicU64,
    /// 진행 중인 폴더 분석의 취소 신호 (새 분석을 시작하면 이전 것을 멈춘다)
    analysis_cancel: Mutex<Option<Arc<AtomicBool>>>,
    /// 진행 중인 큰 파일 찾기의 취소 신호
    large_files_cancel: Mutex<Option<Arc<AtomicBool>>>,
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
            .find(|r| r.app.id == id)
            .map(|r| r.app.clone())
    }

    /// 제거한 프로그램 목록을 고치고 파일에도 저장한다 (바뀐 게 있을 때만).
    fn update_removed(&self, f: impl FnOnce(&mut Vec<removed::RemovedApp>) -> bool) {
        if let Ok(mut list) = self.removed.lock()
            && f(&mut list)
        {
            removed::save(&list);
        }
    }

    fn new() -> Self {
        AppCache {
            removed: Mutex::new(removed::load()),
            ..Default::default()
        }
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
    // 제거했던 프로그램을 다시 설치했으면 "제거한 프로그램"에서 뺀다. 남겨 두면 다시 설치한 파일·등록 정보가
    // 옛 기록 기준으로 "남은 흔적"이 된다.
    cache.update_removed(|removed| {
        let before = removed.len();
        removed.retain(|r| !apps.iter().any(|a| a.id == r.app.id));
        removed.len() != before
    });
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
async fn measure_app_sizes(
    on_progress: Channel<ProgressSnapshot>,
    cache: State<'_, AppCache>,
) -> Result<HashMap<String, u64>, String> {
    // 한 번에 하나만 잰다. 새로고침·정리·복원 뒤마다 요청이 와도 동시에 여러 번 돌지 않게,
    // 기다리는 동안 그 요청 뒤에 시작한 측정이 끝났으면 그 결과를 같이 받는다.
    static REQUESTS: AtomicU64 = AtomicU64::new(0);
    static COVERED: AtomicU64 = AtomicU64::new(0);
    static RUNNING: tauri::async_runtime::Mutex<()> = tauri::async_runtime::Mutex::const_new(());
    let mine = REQUESTS.fetch_add(1, Ordering::SeqCst) + 1;
    let _turn = RUNNING.lock().await;
    if COVERED.load(Ordering::SeqCst) >= mine {
        return Ok(cache.sizes.lock().map(|m| m.clone()).unwrap_or_default());
    }
    // 이번 측정은 지금까지 들어온 요청을 모두 맡는다 (모두 지금 목록으로 잰다)
    let covers = REQUESTS.load(Ordering::SeqCst);
    let apps: Vec<InstalledApplication> = cache
        .apps
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    let sizes = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        return robin_scanner::install_size::install_sizes_with(
            &apps,
            &robin_scanner::protect::KnownFolders::from_env(),
            &progress,
        );
        #[cfg(not(windows))]
        {
            let _ = (apps, progress);
            HashMap::new()
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    if let Ok(mut map) = cache.sizes.lock() {
        *map = sizes.clone();
    }
    COVERED.store(covers, Ordering::SeqCst);
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

    let today = robin_discovery::local_today();
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

/// RobinX 저장소·문제 알리기 페이지를 기본 브라우저로 연다. 주소는 여기 정해 둔 것만 쓴다.
#[tauri::command]
fn open_project_page(page: String) -> Result<(), String> {
    let url = match page.as_str() {
        "home" => "https://github.com/Yoon-robin/RobinX",
        "issues" => "https://github.com/Yoon-robin/RobinX/issues",
        _ => return Err("알 수 없는 페이지예요".into()),
    };
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Windows 작업 스케줄러를 연다 (숨은 예약 작업은 RobinX가 바꾸지 않고, 거기서 사용자가 직접 끈다).
#[tauri::command]
fn open_task_scheduler() -> Result<(), String> {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    std::process::Command::new("explorer.exe")
        .arg(PathBuf::from(root).join("System32").join("taskschd.msc"))
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

/// 창이 지금 모니터의 작업 영역(작업 표시줄을 뺀 곳)보다 크면 맞게 줄이고 가운데에 둔다.
fn fit_to_work_area(window: &tauri::WebviewWindow) {
    let (Ok(Some(monitor)), Ok(size)) = (window.current_monitor(), window.outer_size()) else {
        return;
    };
    let area = monitor.work_area();
    // 작업 영역에 여유(가로 16, 세로 16 물리 px)를 둔다
    let max_w = area.size.width.saturating_sub(16);
    let max_h = area.size.height.saturating_sub(16);
    if size.width <= max_w && size.height <= max_h {
        return;
    }
    let w = size.width.min(max_w);
    let h = size.height.min(max_h);
    let _ = window.set_size(tauri::PhysicalSize::new(w, h));
    let _ = window.set_position(tauri::PhysicalPosition::new(
        area.position.x + ((area.size.width - w) / 2) as i32,
        area.position.y + ((area.size.height - h) / 2) as i32,
    ));
}

/// 일반 제거 확인 창 안내: 보안 프로그램·드라이버·런타임처럼 다른 프로그램이 함께 쓸 수 있는지 (막지 않음).
#[tauri::command]
fn uninstall_may_be_shared(id: String, cache: State<'_, AppCache>) -> bool {
    cache
        .get(&id)
        .is_some_and(|app| robin_execution::force::may_be_shared(&app))
}

/// 제거 확인 창에 보여 줄 "실제로 실행할 명령". 화면이 따로 흉내 내지 않고 실행할 때와 같은 계획을 쓴다
/// (EXE로 분류된 `MsiExec.exe /I{GUID}`도 실제처럼 `/x`, `%ProgramFiles%` 같은 변수도 펼친 모양).
#[tauri::command]
fn uninstall_command_preview(id: String, cache: State<'_, AppCache>) -> Result<String, String> {
    let app = cache
        .get(&id)
        .ok_or("알 수 없는 프로그램이에요. 목록을 새로고침해 주세요")?;
    let plan = robin_execution::plan_for(&app, |name| std::env::var(name).ok())
        .map_err(|e| e.to_string())?;
    let (file, args) = plan.file_and_args();
    let file = if file.contains(' ') {
        format!("\"{file}\"")
    } else {
        file
    };
    Ok(if args.is_empty() {
        file
    } else {
        format!("{file} {args}")
    })
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
    ) {
        cache.update_removed(|removed| {
            removed::push(removed, snapshot);
            true
        });
    }
    Ok(outcome)
}

/// 제거한 프로그램 (최근 것부터, 보관 기간 안의 것).
#[tauri::command]
fn recently_removed(cache: State<'_, AppCache>) -> Vec<removed::RemovedApp> {
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
        let installed = robin_discovery::list_installed();
        // 다시 설치된 프로그램이면 옛 제거 기록 대신 지금 등록 정보로 본다
        let app = installed
            .iter()
            .find(|a| a.id == app.id)
            .cloned()
            .unwrap_or(app);
        robin_scanner::scan(&app, &installed)
    })
    .await
    .map_err(|e| e.to_string())?;

    // 제거한 프로그램이면 "찾아봄"으로 표시 (사이드바 알림을 끈다)
    cache.update_removed(|removed| {
        removed
            .iter_mut()
            .find(|r| r.app.id == id && !r.scanned)
            .map(|r| r.scanned = true)
            .is_some()
    });
    if let Ok(mut last) = cache.last_scan.lock() {
        last.by_app.insert(id, report.items.clone());
    }
    Ok(report)
}

/// 스캔 결과의 파일·폴더를 탐색기에서 선택해 보여 준다. 마지막 스캔에 있던 항목만 허용한다.
// 경로가 연결 안 된 네트워크 드라이브면 확인이 오래 걸려 창이 멈추지 않게 메인 스레드 밖에서 돈다
#[tauri::command(async)]
fn reveal_leftover(item_id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    let item = cache
        .last_scan
        .lock()
        .ok()
        .and_then(|scan| scan.find(&item_id))
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
// 경로가 연결 안 된 네트워크 드라이브면 확인이 오래 걸려 창이 멈추지 않게 메인 스레드 밖에서 돈다
#[tauri::command(async)]
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
    on_progress: Channel<ProgressSnapshot>,
    cache: State<'_, AppCache>,
) -> Result<Operation, String> {
    let items: Vec<Leftover> = {
        let scan = cache.last_scan.lock().map_err(|e| e.to_string())?;
        let Some(scanned) = scan.by_app.get(&app_id) else {
            return Err(
                "이 프로그램의 스캔 결과가 없어요. 남은 흔적을 다시 찾은 뒤 시도해 주세요".into(),
            );
        };
        scanned
            .iter()
            .filter(|i| item_ids.contains(&i.id))
            .cloned()
            .collect()
    };
    if items.is_empty() {
        return Err("정리할 항목이 없어요. 다시 찾은 뒤 시도해 주세요".into());
    }
    // 아직 설치된 프로그램은 강제 제거로만 정리한다. "흔적 미리 보기"에는 지금 쓰는 설치 폴더·등록 정보가
    // 들어 있어서, 여기서 정리하면 강제 제거의 확인(보안 프로그램·실행 중)을 건너뛰게 된다.
    if force_app_id.is_none() {
        let id = app_id.clone();
        let installed = tauri::async_runtime::spawn_blocking(move || {
            robin_discovery::list_installed().iter().any(|a| a.id == id)
        })
        .await
        .map_err(|e| e.to_string())?;
        if installed {
            return Err(
                "아직 설치된 프로그램이에요. 프로그램을 제거한 뒤 남은 흔적을 정리해 주세요".into(),
            );
        }
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

    // 다른 드라이브의 폴더는 복사해서 옮기느라 오래 걸릴 수 있어 진행 상황을 보낸다
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    let op = tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        let mut op = robin_recovery::new_operation(&app_name, &items);
        store
            .save(&op)
            .map_err(|e| format!("작업 기록을 만들지 못했어요: {e}"))?;
        robin_recovery::progress::with(progress, || {
            robin_recovery::clean(&store, &mut op, &guard(&store), |_| {})
        })?;
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
    if !still_installed {
        // 강제 제거는 흔적을 찾아 정리한 뒤라 이미 "찾아봄"
        cache.update_removed(|removed| {
            removed::push(removed, app);
            removed[0].scanned = true;
            true
        });
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
    let _lock = BackupLock::acquire(&op_id)?;
    let op = tauri::async_runtime::spawn_blocking(move || run_worker(worker::CLEAN, &op_id, ""))
        .await
        .map_err(|e| e.to_string())??;
    note_force_removed(&cache, force_app_id).await;
    Ok(op)
}

/// 지금 되돌리거나 지우고 있는 백업. 같은 백업을 동시에 다루지 않게 한다: 복구 기록 화면을 떠났다 돌아와
/// 다시 누르거나(화면은 진행 중인 것을 잊는다), 앱을 켤 때 도는 오래된 백업 정리와 겹치면 지우는 중인 백업에서
/// 되돌리게 된다.
static BUSY_BACKUPS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub(crate) struct BackupLock(String);

impl BackupLock {
    pub(crate) fn acquire(op_id: &str) -> Result<Self, String> {
        let mut busy = BUSY_BACKUPS.lock().unwrap_or_else(|e| e.into_inner());
        if busy.iter().any(|id| id == op_id) {
            return Err("이 백업으로 다른 작업을 하고 있어요. 끝난 뒤 다시 시도해 주세요".into());
        }
        busy.push(op_id.to_string());
        Ok(Self(op_id.to_string()))
    }
}

impl Drop for BackupLock {
    fn drop(&mut self) {
        let mut busy = BUSY_BACKUPS.lock().unwrap_or_else(|e| e.into_inner());
        busy.retain(|id| *id != self.0);
    }
}

/// 복구 기록 (최근 작업부터).
#[tauri::command]
async fn list_backups() -> Result<Vec<Operation>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let store = store()?;
        // 저장소 폴더를 읽지 못하면 "기록 없음"이 아니라 오류로 알린다
        store
            .unreadable()
            .map_err(|e| format!("백업 폴더를 읽지 못했어요: {e}"))?;
        Ok(store.list())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 기록을 읽지 못한 백업 폴더 이름 (매니페스트가 깨졌거나 권한이 없음). 백업은 그대로 두고 알리기만 한다.
#[tauri::command]
async fn unreadable_backups() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(|| store()?.unreadable().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 백업 폴더(`%LOCALAPPDATA%\RobinX\backups`)를 탐색기로 연다.
#[tauri::command(async)]
fn open_backup_folder() -> Result<(), String> {
    let root = store()?.root().to_path_buf();
    if !root.is_dir() {
        return Err("아직 백업 폴더가 없어요".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(&root)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 한 작업의 백업 폴더를 탐색기로 연다 (충돌로 백업에만 남은 파일을 직접 꺼낼 때).
#[tauri::command(async)]
fn open_backup_operation(op_id: String) -> Result<(), String> {
    if !robin_recovery::store::is_valid_id(&op_id) {
        return Err("잘못된 작업 ID예요".into());
    }
    let dir = store()?.dir(&op_id);
    if !dir.is_dir() {
        return Err("이 작업의 백업 폴더가 없어요".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(&dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 작업에서 지운 항목을 되돌린다. `item_ids`가 없으면 전부.
#[tauri::command]
async fn restore_backup(
    op_id: String,
    item_ids: Option<Vec<String>>,
    on_progress: Channel<ProgressSnapshot>,
) -> Result<Operation, String> {
    let _lock = BackupLock::acquire(&op_id)?;
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        let mut op = store.load(&op_id).map_err(|e| e.to_string())?;
        robin_recovery::progress::with(progress, || {
            robin_recovery::restore(&store, &mut op, &guard(&store), item_ids.as_deref(), |_| {})
        })?;
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
    let _lock = BackupLock::acquire(&op_id)?;
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
async fn delete_backup(
    op_id: String,
    on_progress: Channel<ProgressSnapshot>,
) -> Result<(), String> {
    let _lock = BackupLock::acquire(&op_id)?;
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    tauri::async_runtime::spawn_blocking(move || {
        let store = store()?;
        match robin_recovery::progress::with(progress, || store.delete(&op_id)) {
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

/// 일반 사용자에게 안 보이는 예약 작업(SYSTEM 등)을 관리자 권한(UAC)으로 읽어 온다. 읽기만 한다.
/// 이미 목록에 있는 작업은 빼고, 관리자 권한으로만 보이는 것만 돌려준다.
#[tauri::command]
async fn list_hidden_startup_tasks() -> Result<Vec<StartupEntry>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let history = startup_history()?;
        let result_file = history.root().join(worker::TASKS_RESULT);
        let _ = std::fs::remove_file(&result_file);
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let args = format!("{} --store \"{}\"", worker::TASKS, history.root().display());
        match robin_execution::run_elevated_and_wait(&exe.to_string_lossy(), &args) {
            Ok(_) => {}
            Err(None) => return Err("관리자 권한 요청이 거절됐어요".into()),
            Err(Some(m)) => return Err(m),
        }
        let text = std::fs::read_to_string(&result_file)
            .map_err(|_| "관리자 권한으로 작업 목록을 읽지 못했어요".to_string())?;
        let _ = std::fs::remove_file(&result_file);
        let all: Vec<StartupEntry> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let visible: std::collections::HashSet<String> = robin_startup::list_tasks(&history)
            .into_iter()
            .map(|e| e.id)
            .collect();
        Ok(all
            .into_iter()
            .filter(|e| !visible.contains(&e.id))
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 시작 항목을 탐색기에서 보여 준다: 시작 폴더 항목은 그 파일, Run 항목과 예약 작업은 실행 파일.
// 경로가 연결 안 된 네트워크 드라이브면 확인이 오래 걸려 창이 멈추지 않게 메인 스레드 밖에서 돈다
#[tauri::command(async)]
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
async fn scan_storage(
    on_item: Channel<robin_storage::Category>,
    on_progress: Channel<ProgressSnapshot>,
    cache: State<'_, AppCache>,
) -> Result<Vec<robin_storage::Category>, String> {
    // 항목을 다 세는 데 몇 초 걸리므로, 찾는 대로 화면에 먼저 보내고 지금 재는 항목도 알린다
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    let items = tauri::async_runtime::spawn_blocking(move || {
        robin_storage::scan_with(
            &|c| {
                let _ = on_item.send(c.clone());
            },
            &progress,
        )
    })
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
    on_event: Channel<robin_storage::CleanEvent>,
) -> Result<Vec<robin_storage::CleanResult>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // 화면이 닫혀 소식을 못 받아도 정리는 끝까지 한다
        robin_storage::clean_with_progress(&ids, allow_irreversible, &mut |event| {
            let _ = on_event.send(event);
        })
    })
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
// 경로가 연결 안 된 네트워크 드라이브면 확인이 오래 걸려 창이 멈추지 않게 메인 스레드 밖에서 돈다
#[tauri::command(async)]
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

/// 오래 걸리는 작업 동안 [`Progress`]를 짧은 간격으로 화면에 보낸다 (멈춘 것처럼 보이지 않게).
/// 값이 바뀔 때만 보내고, 버리면(drop) 마지막 상태를 한 번 더 보낸 뒤 멈춘다.
pub(crate) struct ProgressTicker {
    stop: Arc<AtomicBool>,
}

impl ProgressTicker {
    const INTERVAL: std::time::Duration = std::time::Duration::from_millis(150);

    pub(crate) fn start(progress: Arc<Progress>, channel: Channel<ProgressSnapshot>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        std::thread::spawn(move || {
            let mut last: Option<ProgressSnapshot> = None;
            loop {
                let stopping = flag.load(Ordering::Relaxed);
                let snap = progress.snapshot();
                if last.as_ref() != Some(&snap) {
                    // 화면이 닫혀 받을 곳이 없으면 그만 보낸다
                    if channel.send(snap.clone()).is_err() {
                        break;
                    }
                    last = Some(snap);
                }
                if stopping {
                    break;
                }
                std::thread::sleep(Self::INTERVAL);
            }
        });
        Self { stop }
    }
}

impl Drop for ProgressTicker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// 폴더 바로 아래 항목의 크기 (읽기 전용). `id`가 없으면 시스템 드라이브.
#[tauri::command]
async fn analyze_folder(
    id: Option<String>,
    on_progress: Channel<ProgressSnapshot>,
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

    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    let (entries, cancelled) = {
        let path = path.clone();
        let cancel = cancel.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let entries = robin_storage::analyze(&path, &cancel, &progress);
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LargeFileItem {
    id: String,
    #[serde(flatten)]
    file: robin_storage::LargeFile,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LargeFiles {
    root: String,
    min_bytes: u64,
    items: Vec<LargeFileItem>,
    cancelled: bool,
}

/// 시스템 드라이브에서 큰 파일을 찾는다 (읽기 전용). Windows 폴더는 건너뛴다.
/// 화면은 경로 대신 ID만 받아 [`reveal_folder_item`]으로 탐색기에서 연다.
#[tauri::command]
async fn find_large_files(
    on_progress: Channel<ProgressSnapshot>,
    cache: State<'_, AppCache>,
) -> Result<LargeFiles, String> {
    const MIN_BYTES: u64 = 200 * 1024 * 1024;
    const LIMIT: usize = 40;
    let root = PathBuf::from(system_drive());
    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut slot) = cache.large_files_cancel.lock()
        && let Some(previous) = slot.replace(cancel.clone())
    {
        previous.store(true, Ordering::Relaxed);
    }
    let progress = Arc::new(Progress::new());
    let _ticker = ProgressTicker::start(progress.clone(), on_progress);
    let (files, cancelled) = {
        let root = root.clone();
        let cancel = cancel.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let files = robin_storage::largest_files(&root, MIN_BYTES, LIMIT, &cancel, &progress);
            (files, cancel.load(Ordering::Relaxed))
        })
        .await
        .map_err(|e| e.to_string())?
    };
    let items = files
        .into_iter()
        .map(|file| LargeFileItem {
            id: cache.folder_id(Path::new(&file.path)),
            file,
        })
        .collect();
    Ok(LargeFiles {
        root: root.display().to_string(),
        min_bytes: MIN_BYTES,
        items,
        cancelled,
    })
}

/// 진행 중인 큰 파일 찾기를 멈춘다.
#[tauri::command]
fn cancel_large_files(cache: State<'_, AppCache>) {
    if let Ok(slot) = cache.large_files_cancel.lock()
        && let Some(cancel) = slot.as_ref()
    {
        cancel.store(true, Ordering::Relaxed);
    }
}

/// 분석 결과의 항목을 탐색기로 연다.
// 경로가 연결 안 된 네트워크 드라이브면 확인이 오래 걸려 창이 멈추지 않게 메인 스레드 밖에서 돈다
#[tauri::command(async)]
fn reveal_folder_item(id: String, cache: State<'_, AppCache>) -> Result<(), String> {
    open_in_explorer(&cache.folder_path(&id).ok_or("알 수 없는 항목이에요")?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    // 앱은 하나만 띄운다: 두 번째로 켜면 떠 있는 창을 앞으로 가져온다. 두 창에서 같은 백업을 되돌리거나 지우는 일을 막는다.
    // 설치·제거 화면은 앱이 떠 있어도 열려야 하므로 앱 모드에서만. 개발 빌드는 설치된 RobinX와 식별자가 같아
    // 개발 앱이 설치된 앱 창만 띄우고 끝나므로 배포 빌드에서만 켠다.
    #[cfg(not(debug_assertions))]
    let builder = if setup::mode() == setup::Mode::App {
        builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
    } else {
        builder
    };
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppCache::new())
        // 설치·제거 화면을 Alt+F4 등으로 닫을 때도 진행 중이면 막고, 제거를 마쳤으면 임시 복사본을 정리한다
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && setup::mode() != setup::Mode::App
                && !setup::on_close_requested()
            {
                api.prevent_close();
            }
        })
        .setup(|app| {
            // 설치·제거 화면: 작은 고정 크기 창으로 띄우고, 앱의 다른 시작 작업은 하지 않는다
            #[cfg(windows)]
            if setup::mode() != setup::Mode::App {
                if let Some(window) = app.get_webview_window("main") {
                    let title = if setup::mode() == setup::Mode::Setup {
                        "RobinX 설치"
                    } else {
                        "RobinX 제거"
                    };
                    let _ = window.set_title(title);
                    let _ = window.set_size(tauri::LogicalSize::new(640.0, 480.0));
                    let _ = window.set_min_size(Some(tauri::LogicalSize::new(640.0, 480.0)));
                    let _ = window.set_resizable(false);
                    let _ = window.set_maximizable(false);
                    let _ = window.center();
                }
                return Ok(());
            }
            // 기간이 지난 백업을 정리한다 (처음 설정은 30일, 사용자가 "계속 보관"을 고르면 아무것도 하지 않는다)
            std::thread::spawn(|| {
                let _ = settings::prune_expired();
            });
            // 창 배경은 불투명하게 그린다 (예전에는 Windows 11 Mica로 배경화면이 비쳤지만,
            // 반투명한 헤더 뒤로 목록이 비쳐 겹쳐 보여서 2026-10-20에 뺐다)
            // 기본 창(1200×780)이 작업 영역보다 크면(1366×768 @125%, 1920×1080 @150% 등) 창 버튼이 화면 밖으로 나간다.
            // 보이기 전에 작업 영역 안으로 줄이고 가운데에 둔다.
            if let Some(window) = app.get_webview_window("main") {
                fit_to_work_area(&window);
            }
            // 창은 index.html이 시작 화면을 그리면서 바로 보여 준다. 이 안전장치는 화면을 끝내 불러오지 못했을 때만
            // 창이 숨은 채 남지 않게 하려는 것이라 넉넉히 기다린다. 너무 빨리 보이면 내용 없는 빈 창이 먼저 뜬다.
            if let Some(window) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(15));
                    if !window.is_visible().unwrap_or(true) {
                        let _ = window.show();
                    }
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
            uninstall_command_preview,
            uninstall_may_be_shared,
            open_task_scheduler,
            open_project_page,
            stop_waiting_uninstall,
            recently_removed,
            scan_leftovers,
            reveal_leftover,
            clean_leftovers,
            clean_leftovers_elevated,
            force_uninstall_check,
            list_backups,
            unreadable_backups,
            open_backup_folder,
            open_backup_operation,
            restore_backup,
            restore_backup_elevated,
            delete_backup,
            list_startup_items,
            list_hidden_startup_tasks,
            get_startup_icon,
            set_startup_enabled,
            clean_missing_startup,
            settings::get_settings,
            settings::set_backup_retention,
            settings::add_exclusion,
            settings::remove_exclusion,
            settings::backup_usage,
            settings::prune_backups,
            settings::old_backups,
            settings::delete_old_backups,
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
            find_large_files,
            cancel_large_files,
            #[cfg(windows)]
            setup::app_mode,
            #[cfg(windows)]
            setup::setup_info,
            #[cfg(windows)]
            setup::run_install,
            #[cfg(windows)]
            setup::run_uninstall,
            #[cfg(windows)]
            setup::launch_installed,
            #[cfg(windows)]
            setup::finish_setup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
