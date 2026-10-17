//! 설치 추적 command. 스냅샷·비교는 `crates/tracker`가 하고, 여기는 상태만 이어 준다.
//!
//! 화면은 비교 결과에서 남길 항목의 **번호**만 넘긴다. 경로는 엔진이 만든 결과에서만 가져온다.

use std::sync::Mutex;

use robin_tracker::{Trace, TraceDraft, TraceStore};
use serde::Serialize;
use tauri::State;

/// 마지막으로 끝낸 추적의 비교 결과 (저장하기 전)
#[derive(Default)]
pub struct TrackingState {
    draft: Mutex<Option<TraceDraft>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewApp {
    id: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftView {
    #[serde(flatten)]
    draft: TraceDraft,
    /// 새로 생긴 설치 항목의 이름
    new_apps: Vec<NewApp>,
}

fn store() -> Result<TraceStore, String> {
    TraceStore::default_location().ok_or_else(|| "추적 기록 위치를 찾을 수 없어요".to_string())
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// 추적 중이면 시작 시각 (유닉스 초).
#[tauri::command]
pub async fn tracking_status() -> Result<Option<u64>, String> {
    blocking(|| Ok(store()?.pending_started_at())).await
}

/// 설치 전 상태를 기록한다. 이미 추적 중이면 다시 찍는다.
#[tauri::command]
pub async fn start_tracking() -> Result<u64, String> {
    blocking(|| {
        let snap = robin_tracker::snapshot::take();
        store()?
            .save_pending(&snap)
            .map_err(|e| format!("설치 전 상태를 저장하지 못했어요: {e}"))?;
        Ok(snap.taken_at)
    })
    .await
}

#[tauri::command]
pub async fn cancel_tracking(state: State<'_, TrackingState>) -> Result<(), String> {
    if let Ok(mut d) = state.draft.lock() {
        *d = None;
    }
    blocking(|| store()?.clear_pending().map_err(|e| e.to_string())).await
}

/// 설치 후 상태를 찍어 비교한다. 결과는 저장하기 전까지 여기에 둔다.
#[tauri::command]
pub async fn finish_tracking(state: State<'_, TrackingState>) -> Result<DraftView, String> {
    let draft = blocking(|| {
        let before = store()?
            .load_pending()
            .ok_or("추적을 시작한 기록이 없어요. 다시 시작해 주세요")?;
        let after = robin_tracker::snapshot::take();
        let draft = robin_tracker::diff(&before, &after);
        if draft.incomplete {
            // 끝까지 보지 못한 스냅샷끼리 비교하면 원래 있던 항목이 "새로 생김"으로 보일 수 있다
            return Err(
                "검사할 파일·레지스트리가 너무 많아 비교 결과를 믿을 수 없어요. 추적을 그만두고 다시 시도해 주세요"
                    .to_string(),
            );
        }
        Ok(draft)
    })
    .await?;

    let installed = blocking(|| Ok(robin_discovery::list_installed())).await?;
    let new_apps = draft
        .new_app_ids
        .iter()
        .map(|id| NewApp {
            id: id.clone(),
            name: installed
                .iter()
                .find(|a| &a.id == id)
                .map(|a| a.display_name.clone())
                .unwrap_or_else(|| id.clone()),
        })
        .collect();
    if let Ok(mut d) = state.draft.lock() {
        *d = Some(draft.clone());
    }
    Ok(DraftView { draft, new_apps })
}

/// 비교 결과에서 고른 항목(번호)과 프로그램만 기록으로 저장하고 추적을 끝낸다.
#[tauri::command]
pub async fn save_trace(
    name: String,
    keep: Vec<usize>,
    app_ids: Vec<String>,
    state: State<'_, TrackingState>,
) -> Result<Trace, String> {
    let draft = state
        .draft
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("저장할 추적 결과가 없어요")?;
    // 프로그램은 이번 추적에서 새로 생긴 것만 연결할 수 있다
    if app_ids.iter().any(|a| !draft.new_app_ids.contains(a)) {
        return Err("이번 설치에서 찾은 프로그램만 연결할 수 있어요".into());
    }
    let changes = draft
        .changes
        .iter()
        .enumerate()
        .filter(|(i, _)| keep.contains(i))
        .map(|(_, c)| c.clone())
        .collect();
    let name = name.trim();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let trace = Trace {
        id: robin_tracker::model::new_trace_id(now.as_millis(), std::process::id()),
        name: if name.is_empty() {
            "이름 없는 설치".into()
        } else {
            name.chars().take(120).collect()
        },
        started_at: draft.started_at,
        finished_at: draft.finished_at,
        app_ids,
        changes,
    };

    let saved = trace.clone();
    blocking(move || {
        let store = store()?;
        store
            .save(&saved)
            .map_err(|e| format!("기록을 저장하지 못했어요: {e}"))?;
        store.clear_pending().map_err(|e| e.to_string())
    })
    .await?;
    if let Ok(mut d) = state.draft.lock() {
        *d = None;
    }
    Ok(trace)
}

#[tauri::command]
pub async fn list_traces() -> Result<Vec<Trace>, String> {
    blocking(|| Ok(store()?.list())).await
}

/// 기록만 지운다. 설치된 프로그램과 파일은 그대로다.
#[tauri::command]
pub async fn delete_trace(id: String) -> Result<(), String> {
    blocking(move || store()?.delete(&id).map_err(|e| e.to_string())).await
}
