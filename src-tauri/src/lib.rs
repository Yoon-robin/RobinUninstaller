//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.
//!
//! 프론트엔드는 경로가 아니라 앱 ID만 넘긴다. 경로는 항상 엔진이 읽어 둔 목록에서 찾는다.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use robin_core::InstalledApplication;
use tauri::State;

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
    let Some(icon_source) = cache.get(&id).and_then(|a| a.display_icon) else {
        return Ok(None);
    };

    let icon =
        tauri::async_runtime::spawn_blocking(move || robin_discovery::icon_data_url(&icon_source))
            .await
            .map_err(|e| e.to_string())?;

    if let Ok(mut map) = cache.icons.lock() {
        map.insert(id, icon.clone());
    }
    Ok(icon)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppCache::default())
        .invoke_handler(tauri::generate_handler![
            list_installed_apps,
            get_app_icon,
            open_install_location
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
