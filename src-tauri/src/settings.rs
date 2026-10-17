//! 설정 화면 command: 백업 보관 기간과 정리, 잔여물 스캔 제외 목록.
//!
//! 제외 항목은 화면에서 경로를 받지 않고 마지막 스캔 결과의 항목 ID로만 더한다.

use robin_settings::{Exclusion, ExclusionKind, RETENTION_CHOICES, SettingsStore};
use serde::Serialize;
use tauri::State;

use crate::AppCache;

fn settings_store() -> Result<SettingsStore, String> {
    SettingsStore::default_location().ok_or_else(|| "설정 위치를 찾을 수 없어요".to_string())
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    backup_retention_days: Option<u32>,
    retention_choices: Vec<u32>,
    exclusions: Vec<Exclusion>,
}

#[tauri::command]
pub async fn get_settings() -> Result<SettingsView, String> {
    blocking(|| {
        let s = settings_store()?.load();
        Ok(SettingsView {
            backup_retention_days: s.backup_retention_days,
            retention_choices: RETENTION_CHOICES.to_vec(),
            exclusions: s.exclusions,
        })
    })
    .await
}

/// 백업 보관 기간. None이면 계속 보관.
#[tauri::command]
pub async fn set_backup_retention(days: Option<u32>) -> Result<(), String> {
    blocking(move || robin_settings::update(&settings_store()?, |s| s.set_retention(days))).await
}

/// 마지막 스캔 결과의 항목을 다음 스캔부터 뺀다. 이미 덮여 있으면 false.
#[tauri::command]
pub async fn add_exclusion(item_id: String, cache: State<'_, AppCache>) -> Result<bool, String> {
    let item = cache
        .last_scan
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|i| i.id == item_id)
        .cloned()
        .ok_or("스캔 결과에 없는 항목이에요")?;
    let kind = match item.kind {
        robin_scanner::LeftoverKind::Folder | robin_scanner::LeftoverKind::File => {
            ExclusionKind::Path
        }
        _ => ExclusionKind::Registry,
    };
    blocking(move || {
        robin_settings::update(&settings_store()?, |s| {
            s.add_exclusion(Exclusion {
                kind,
                location: item.location,
                value_name: item.value_name,
                added_at: now(),
            })
        })
    })
    .await
}

#[tauri::command]
pub async fn remove_exclusion(index: usize) -> Result<(), String> {
    blocking(move || {
        robin_settings::update(&settings_store()?, |s| {
            if index >= s.exclusions.len() {
                return Err("목록에 없는 항목이에요".into());
            }
            s.exclusions.remove(index);
            Ok(())
        })
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupUsage {
    count: usize,
    bytes: u64,
    /// 지금 보관 기간 기준으로 지울 수 있는 백업 수 (계속 보관이면 0)
    expired: usize,
}

#[tauri::command]
pub async fn backup_usage() -> Result<BackupUsage, String> {
    blocking(|| {
        let store = crate::store()?;
        let (count, bytes) = store.usage();
        let expired = settings_store()?
            .load()
            .backup_retention_days
            .map(|days| store.expired(days, now()).len())
            .unwrap_or(0);
        Ok(BackupUsage {
            count,
            bytes,
            expired,
        })
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneResult {
    deleted: usize,
    /// 권한 등으로 지우지 못한 백업 (복구 기록에서 하나씩 지울 수 있다)
    failed: usize,
}

/// 보관 기간이 지난 백업을 지운다. 보관 기간을 고르지 않았으면 아무것도 지우지 않는다.
pub fn prune_expired() -> Result<PruneResult, String> {
    let Some(days) = settings_store()?.load().backup_retention_days else {
        return Ok(PruneResult {
            deleted: 0,
            failed: 0,
        });
    };
    let store = crate::store()?;
    let (mut deleted, mut failed) = (0, 0);
    for id in store.expired(days, now()) {
        match store.delete(&id) {
            Ok(()) => deleted += 1,
            Err(_) => failed += 1,
        }
    }
    Ok(PruneResult { deleted, failed })
}

#[tauri::command]
pub async fn prune_backups() -> Result<PruneResult, String> {
    blocking(prune_expired).await
}
