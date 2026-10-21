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
        .find(&item_id)
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

/// 제외 목록에서 `index`번째를 뺀다. `location`은 화면이 본 그 항목의 위치로, 그사이 목록이 바뀌었으면(두 번 누름 등)
/// 다른 항목을 빼지 않도록 확인만 한다.
#[tauri::command]
pub async fn remove_exclusion(
    index: usize,
    location: String,
    value_name: Option<String>,
) -> Result<(), String> {
    blocking(move || {
        robin_settings::update(&settings_store()?, |s| {
            // 위치만 보면 같은 키의 값 두 개 중 엉뚱한 것을 뺄 수 있어 값 이름도 맞춰 본다
            if s.exclusions
                .get(index)
                .is_none_or(|e| e.location != location || e.value_name != value_name)
            {
                return Err("목록이 바뀌었어요. 다시 확인해 주세요".into());
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PruneResult {
    deleted: usize,
    /// 권한 등으로 지우지 못한 백업 (복구 기록에서 하나씩 지울 수 있다)
    failed: usize,
}

impl PruneResult {
    pub fn is_empty(&self) -> bool {
        self.deleted == 0 && self.failed == 0
    }
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
    let ids = store.expired(days, now());
    robin_recovery::progress::report_total(ids.len() as u64);
    let (mut deleted, mut failed) = (0, 0);
    for id in ids {
        // 사용자가 지금 되돌리거나 지우는 백업은 건너뛴다 (다음에 켤 때 다시 본다)
        let Ok(_lock) = crate::BackupLock::acquire(&id) else {
            robin_recovery::progress::report_unit_done();
            continue;
        };
        match store.delete(&id) {
            Ok(()) => deleted += 1,
            Err(_) => failed += 1,
        }
        robin_recovery::progress::report_unit_done();
    }
    Ok(PruneResult { deleted, failed })
}

/// 설정의 "지금 정리". 앱을 켤 때 도는 정리와 같고, 화면에 진행 상황을 보낸다.
#[tauri::command]
pub async fn prune_backups(
    on_progress: tauri::ipc::Channel<robin_core::progress::ProgressSnapshot>,
) -> Result<PruneResult, String> {
    let progress = std::sync::Arc::new(robin_core::progress::Progress::new());
    let _ticker = crate::ProgressTicker::start(progress.clone(), on_progress);
    blocking(move || robin_recovery::progress::with(progress, prune_expired)).await
}

/// 복구 기록에서 고르는 기간은 보관 기간 선택지와 같은 값만 받는다.
fn retention_choice(days: u32) -> Result<u32, String> {
    if RETENTION_CHOICES.contains(&days) {
        Ok(days)
    } else {
        Err("고를 수 없는 기간이에요".into())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OldBackups {
    count: usize,
    bytes: u64,
}

/// `days`일보다 오래된 백업이 몇 개, 얼마나 되는지 (읽기 전용). 복원 중 충돌이 난 백업은 빠진다.
#[tauri::command]
pub async fn old_backups(days: u32) -> Result<OldBackups, String> {
    blocking(move || {
        let days = retention_choice(days)?;
        let store = crate::store()?;
        let ids = store.expired(days, now());
        let bytes = ids.iter().map(|id| store.bytes_of(id)).sum();
        Ok(OldBackups {
            count: ids.len(),
            bytes,
        })
    })
    .await
}

/// `days`일보다 오래된 백업을 지운다. 사용자가 기간을 고르고 한 번 더 확인한 뒤에만 부른다.
/// 관리자 권한으로 격리한 백업처럼 일반 권한으로 못 지운 것은 `failed`로 남고, 복구 기록에서 하나씩 지울 수 있다.
#[tauri::command]
pub async fn delete_old_backups(
    days: u32,
    on_progress: tauri::ipc::Channel<robin_core::progress::ProgressSnapshot>,
) -> Result<PruneResult, String> {
    // 백업이 크면 지우는 데 오래 걸려서, 몇 번째 백업을 지우는지와 지운 크기를 보낸다
    let progress = std::sync::Arc::new(robin_core::progress::Progress::new());
    let _ticker = crate::ProgressTicker::start(progress.clone(), on_progress);
    blocking(move || {
        let days = retention_choice(days)?;
        let store = crate::store()?;
        let ids = store.expired(days, now());
        progress.set_total(ids.len() as u64);
        let (mut deleted, mut failed) = (0, 0);
        for id in ids {
            // 지금 되돌리거나 지우는 중인 백업은 건드리지 않고 "지우지 못함"으로 센다
            let Ok(_lock) = crate::BackupLock::acquire(&id) else {
                failed += 1;
                progress.unit_done();
                continue;
            };
            match robin_recovery::progress::with(progress.clone(), || store.delete(&id)) {
                Ok(()) => deleted += 1,
                Err(_) => failed += 1,
            }
            progress.unit_done();
        }
        Ok(PruneResult { deleted, failed })
    })
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_retention_choices_are_accepted() {
        for d in robin_settings::RETENTION_CHOICES {
            assert_eq!(super::retention_choice(d), Ok(d));
        }
        for d in [0, 1, 7, 29, 31, 365, u32::MAX] {
            assert!(super::retention_choice(d).is_err());
        }
    }
}
