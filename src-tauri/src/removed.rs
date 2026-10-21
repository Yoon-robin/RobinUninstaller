//! "제거한 프로그램" 목록을 데이터 폴더(`%LOCALAPPDATA%\RobinX\removed.json`)에 남긴다.
//!
//! 제거 뒤 "다시 시작 필요"를 보고 PC를 다시 시작해도 남은 흔적을 찾을 수 있게 한다.
//! 프로그램 정보(등록 정보 사본)만 담고, 파일·레지스트리는 건드리지 않는다.

use std::path::{Path, PathBuf};

use robin_core::InstalledApplication;
use serde::{Deserialize, Serialize};

/// 너무 오래 쌓이지 않게 최근 것만 남긴다.
const MAX_ENTRIES: usize = 50;
/// 보관 기간을 "계속 보관"으로 골랐을 때 목록에 남기는 기간
const KEEP_ALL_DAYS: u64 = 365;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovedApp {
    #[serde(flatten)]
    pub app: InstalledApplication,
    /// 제거한 때 (Unix 초)
    pub removed_at: u64,
    /// 남은 흔적을 한 번이라도 찾아봤는지 (사이드바 표시용)
    #[serde(default)]
    pub scanned: bool,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn default_path() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(
        PathBuf::from(local)
            .join(robin_core::APP_DIR)
            .join("removed.json"),
    )
}

/// 목록에 남길 기간(일): 백업 보관 기간과 같게 한다. 백업이 지워진 뒤에도 흔적은 다시 찾을 수 있지만,
/// 오래된 항목이 계속 쌓이지 않게 한다.
fn keep_days() -> u64 {
    robin_settings::SettingsStore::default_location()
        .and_then(|s| s.load().backup_retention_days)
        .map_or(KEEP_ALL_DAYS, u64::from)
}

/// 파일에서 읽는다. 없거나 깨졌으면 빈 목록. 기간이 지난 항목은 뺀다.
pub fn load_from(path: &Path, keep_days: u64, now: u64) -> Vec<RemovedApp> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut list: Vec<RemovedApp> = serde_json::from_str(&text).unwrap_or_default();
    let limit = keep_days.saturating_mul(86_400);
    list.retain(|r| now.saturating_sub(r.removed_at) <= limit);
    list.truncate(MAX_ENTRIES);
    list
}

pub fn save_to(path: &Path, list: &[RemovedApp]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(list)?)?;
    std::fs::rename(&tmp, path)
}

pub fn load() -> Vec<RemovedApp> {
    default_path()
        .map(|p| load_from(&p, keep_days(), now()))
        .unwrap_or_default()
}

pub fn save(list: &[RemovedApp]) {
    if let Some(p) = default_path() {
        let _ = save_to(&p, list);
    }
}

/// 맨 앞에 넣는다 (같은 프로그램이 있으면 옮긴다).
pub fn push(list: &mut Vec<RemovedApp>, app: InstalledApplication) {
    list.retain(|r| r.app.id != app.id);
    list.insert(
        0,
        RemovedApp {
            app,
            removed_at: now(),
            scanned: false,
        },
    );
    list.truncate(MAX_ENTRIES);
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::{InstallScope, InstallTechnology, UninstallerStatus};

    fn app(id: &str) -> InstalledApplication {
        InstalledApplication {
            id: id.into(),
            display_name: "Fixture Notes".into(),
            publisher: Some("Robin Test Labs".into()),
            version: None,
            install_location: Some(r"C:\Users\me\AppData\Local\Programs\Fixture Notes".into()),
            install_date: None,
            estimated_size_kb: Some(10),
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Exe,
            scope: InstallScope::User,
            uninstaller: UninstallerStatus::Available,
        }
    }

    #[test]
    fn saves_and_loads_and_drops_expired() {
        let dir = std::env::temp_dir().join(format!("robinx-removed-test-{}", std::process::id()));
        let path = dir.join("removed.json");
        let mut list = Vec::new();
        push(&mut list, app("hkcu:a"));
        push(&mut list, app("hkcu:b"));
        push(&mut list, app("hkcu:a"));
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].app.id, "hkcu:a");
        list[1].removed_at = 1_000;
        list[0].removed_at = 1_000 + 40 * 86_400;
        list[0].scanned = true;
        save_to(&path, &list).unwrap();

        let back = load_from(&path, 30, 1_000 + 40 * 86_400);
        assert_eq!(back.len(), 1, "30일 지난 항목은 빠진다");
        assert_eq!(back[0].app.id, "hkcu:a");
        assert!(back[0].scanned);
        assert_eq!(
            back[0].app.install_location.as_deref(),
            Some(r"C:\Users\me\AppData\Local\Programs\Fixture Notes")
        );
        assert_eq!(load_from(&path, 365, 1_000 + 40 * 86_400).len(), 2);

        std::fs::write(&path, "{broken").unwrap();
        assert!(load_from(&path, 30, 0).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
