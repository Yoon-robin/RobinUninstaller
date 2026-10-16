//! Tauri 앱 진입점. 여기는 얇은 command 계층만 두고, 실제 로직은 `crates/`의 엔진이 담당한다.

use robin_core::InstalledApplication;

/// 설치된 프로그램 목록 (읽기 전용).
#[tauri::command]
async fn list_installed_apps() -> Vec<InstalledApplication> {
    tauri::async_runtime::spawn_blocking(robin_discovery::list_installed)
        .await
        .unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_installed_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
