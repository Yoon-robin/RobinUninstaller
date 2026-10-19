//! 최적화 화면 command. 화면은 항목 ID만 넘기고, 무엇을 바꿀지는 엔진의 항목 목록에서 찾는다.
//! 현재 사용자(HKCU) 설정만 다루므로 관리자 권한 작업자가 필요 없다.

use robin_tweaks::{History, RevertOutcome, TweakView};

fn history() -> Result<History, String> {
    History::default_location().ok_or_else(|| "최적화 기록 위치를 찾을 수 없어요".to_string())
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// 항목과 현재 상태 (읽기 전용)
#[tauri::command]
pub async fn list_tweaks() -> Result<Vec<TweakView>, String> {
    blocking(|| Ok(robin_tweaks::list(&history()?))).await
}

/// 원래 값을 기록한 뒤 적용한다.
#[tauri::command]
pub async fn apply_tweak(id: String) -> Result<(), String> {
    blocking(move || robin_tweaks::apply(&history()?, &id)).await
}

/// RobinX가 바꾼 항목을 원래 값으로 되돌린다.
#[tauri::command]
pub async fn revert_tweak(id: String) -> Result<RevertOutcome, String> {
    blocking(move || robin_tweaks::revert(&history()?, &id)).await
}
