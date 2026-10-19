//! 사용자 설정: `%LOCALAPPDATA%\RobinX\settings.json`.
//!
//! - 백업 보관 기간: 새로 쓰기 시작하면 30일 (사용자와 정함, 2026-10-19). 정리한 흔적의 백업도 공간을 차지하므로,
//!   기간이 지난 백업은 앱을 켤 때 지운다. 사용자가 저장해 둔 값("계속 보관" 포함)은 그대로 따른다.
//!   설정 파일이 있는데 읽지 못하면 백업을 지우지 않는 쪽("계속 보관")으로 둔다.
//! - 잔여물 스캔 제외 목록: 사용자가 "다음부터 제외"를 고른 위치. 스캔 결과에서 빼기만 하므로 무언가를 지우게 만들 수는 없다.
//!   항목은 화면이 경로를 직접 넘기지 않고, 스캔 결과의 항목 ID로만 추가된다.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 고를 수 있는 백업 보관 기간 (일)
pub const RETENTION_CHOICES: [u32; 3] = [30, 90, 180];
/// 설정 파일이 없거나 값이 빠져 있을 때의 보관 기간 (일)
pub const DEFAULT_RETENTION_DAYS: u32 = 30;
/// 제외 목록 상한
pub const MAX_EXCLUSIONS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExclusionKind {
    /// 파일·폴더 (그 아래 전부)
    Path,
    /// 레지스트리 키 (그 아래 전부) 또는 값 하나
    Registry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exclusion {
    pub kind: ExclusionKind,
    pub location: String,
    /// 레지스트리 값 하나만 뺄 때
    pub value_name: Option<String>,
    /// 추가한 시각 (유닉스 초)
    pub added_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// 이 기간(일)보다 오래된 백업을 지운다. None이면 계속 보관.
    /// 파일에 값이 없으면 기본값(30일), `null`로 저장돼 있으면 사용자가 고른 "계속 보관"이다.
    pub backup_retention_days: Option<u32>,
    pub exclusions: Vec<Exclusion>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backup_retention_days: Some(DEFAULT_RETENTION_DAYS),
            exclusions: Vec::new(),
        }
    }
}

impl Settings {
    /// 백업을 지우지 않는 설정 (설정을 읽지 못했을 때 쓴다)
    fn keep_all() -> Self {
        Self {
            backup_retention_days: None,
            exclusions: Vec::new(),
        }
    }

    /// 보관 기간은 정해진 값만 받는다.
    pub fn set_retention(&mut self, days: Option<u32>) -> Result<(), String> {
        if days.is_some_and(|d| !RETENTION_CHOICES.contains(&d)) {
            return Err("고를 수 없는 보관 기간이에요".into());
        }
        self.backup_retention_days = days;
        Ok(())
    }

    /// 제외 항목을 더한다. 이미 있거나 그 위치를 덮는 항목이 있으면 그대로 둔다.
    pub fn add_exclusion(&mut self, exclusion: Exclusion) -> Result<bool, String> {
        if exclusion.location.trim().is_empty() {
            return Err("빈 위치는 제외할 수 없어요".into());
        }
        let covered = is_excluded(
            &self.exclusions,
            exclusion.kind,
            &exclusion.location,
            exclusion.value_name.as_deref(),
        );
        if covered {
            return Ok(false);
        }
        if self.exclusions.len() >= MAX_EXCLUSIONS {
            return Err(format!("제외 목록은 {MAX_EXCLUSIONS}개까지예요"));
        }
        self.exclusions.push(exclusion);
        Ok(true)
    }
}

fn normalize(location: &str) -> String {
    location
        .trim()
        .trim_matches('"')
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn within(child: &str, parent: &str) -> bool {
    child == parent
        || (child.len() > parent.len()
            && child.starts_with(parent)
            && child.as_bytes()[parent.len()] == b'\\')
}

/// 이 위치가 제외 목록에 걸리는지. 폴더·키를 제외하면 그 아래 전부, 값을 제외하면 그 값 하나만.
pub fn is_excluded(
    exclusions: &[Exclusion],
    kind: ExclusionKind,
    location: &str,
    value_name: Option<&str>,
) -> bool {
    let loc = normalize(location);
    exclusions.iter().filter(|e| e.kind == kind).any(|e| {
        let ex = normalize(&e.location);
        match (&e.value_name, value_name) {
            // 값 하나를 제외: 같은 키의 같은 값만
            (Some(ev), Some(v)) => ex == loc && ev.eq_ignore_ascii_case(v),
            (Some(_), None) => false,
            // 폴더·키를 제외: 그 아래 전부(값 포함)
            (None, _) => within(&loc, &ex),
        }
    })
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        SettingsStore { path: path.into() }
    }

    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(SettingsStore::new(
            PathBuf::from(local)
                .join(robin_core::APP_DIR)
                .join("settings.json"),
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 파일이 없으면 기본값(30일 보관, 제외 없음). 파일이 있는데 읽지 못하거나 깨져 있으면
    /// 사용자가 고른 값을 알 수 없으므로 백업을 지우지 않는 "계속 보관"으로 둔다.
    pub fn load(&self) -> Settings {
        match std::fs::read_to_string(&self.path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| Settings::keep_all()),
            Err(_) => Settings::keep_all(),
        }
    }

    /// 임시 파일에 쓰고 바꿔치기한다.
    pub fn save(&self, settings: &Settings) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(serde_json::to_string_pretty(settings)?.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(tmp, &self.path)
    }
}

/// 설정을 읽어 바꾸고 저장한다.
pub fn update<T>(
    store: &SettingsStore,
    f: impl FnOnce(&mut Settings) -> Result<T, String>,
) -> Result<T, String> {
    let mut settings = store.load();
    let out = f(&mut settings)?;
    store
        .save(&settings)
        .map_err(|e| format!("설정을 저장하지 못했어요: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(kind: ExclusionKind, location: &str, value: Option<&str>) -> Exclusion {
        Exclusion {
            kind,
            location: location.into(),
            value_name: value.map(Into::into),
            added_at: 0,
        }
    }

    #[test]
    fn folder_and_key_exclusions_cover_children() {
        let list = vec![
            ex(
                ExclusionKind::Path,
                r"C:\Users\me\AppData\Roaming\Acme\",
                None,
            ),
            ex(ExclusionKind::Registry, r"HKCU\Software\Acme", None),
        ];
        assert!(is_excluded(
            &list,
            ExclusionKind::Path,
            r"c:/users/me/appdata/roaming/acme/sub",
            None
        ));
        assert!(!is_excluded(
            &list,
            ExclusionKind::Path,
            r"C:\Users\me\AppData\Roaming\Acme2",
            None
        ));
        assert!(is_excluded(
            &list,
            ExclusionKind::Registry,
            r"HKCU\SOFTWARE\Acme\X",
            Some("v")
        ));
        // 종류가 다르면 걸리지 않는다
        assert!(!is_excluded(
            &list,
            ExclusionKind::Registry,
            r"C:\Users\me\AppData\Roaming\Acme",
            None
        ));
    }

    #[test]
    fn value_exclusion_is_only_that_value() {
        let run = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        let list = vec![ex(ExclusionKind::Registry, run, Some("Acme"))];
        assert!(is_excluded(
            &list,
            ExclusionKind::Registry,
            run,
            Some("acme")
        ));
        assert!(!is_excluded(
            &list,
            ExclusionKind::Registry,
            run,
            Some("Other")
        ));
        assert!(!is_excluded(&list, ExclusionKind::Registry, run, None));
    }

    #[test]
    fn settings_validate_and_round_trip() {
        let mut s = Settings::default();
        assert!(s.set_retention(Some(7)).is_err());
        s.set_retention(Some(90)).unwrap();
        assert!(
            s.add_exclusion(ex(ExclusionKind::Path, r"C:\A", None))
                .unwrap()
        );
        // 이미 덮는 항목이 있으면 더하지 않는다
        assert!(
            !s.add_exclusion(ex(ExclusionKind::Path, r"C:\A\B", None))
                .unwrap()
        );
        assert!(s.add_exclusion(ex(ExclusionKind::Path, " ", None)).is_err());

        let dir = std::env::temp_dir().join(format!("robin-settings-{}", std::process::id()));
        let store = SettingsStore::new(dir.join("settings.json"));
        assert_eq!(store.load(), Settings::default());
        store.save(&s).unwrap();
        assert_eq!(store.load(), s);
        update(&store, |s| s.set_retention(None)).unwrap();
        assert_eq!(store.load().backup_retention_days, None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn retention_defaults_to_30_days_only_when_nothing_was_saved() {
        let dir =
            std::env::temp_dir().join(format!("robin-settings-default-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let store = SettingsStore::new(path.clone());

        // 파일이 없으면 30일
        assert_eq!(
            store.load().backup_retention_days,
            Some(DEFAULT_RETENTION_DAYS)
        );
        // 값이 빠진 예전 파일도 30일 (제외 목록은 그대로)
        std::fs::write(&path, r#"{"exclusions":[]}"#).unwrap();
        assert_eq!(store.load().backup_retention_days, Some(30));
        // 사용자가 "계속 보관"을 저장했으면 그대로
        std::fs::write(&path, r#"{"backupRetentionDays":null,"exclusions":[]}"#).unwrap();
        assert_eq!(store.load().backup_retention_days, None);
        // 고른 기간도 그대로
        std::fs::write(&path, r#"{"backupRetentionDays":180}"#).unwrap();
        assert_eq!(store.load().backup_retention_days, Some(180));
        // 깨진 파일이면 지우지 않는 쪽
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(store.load().backup_retention_days, None);
        // 저장하면 기본값도 파일에 남아, 다음에 읽어도 같다
        store.save(&Settings::default()).unwrap();
        assert_eq!(store.load(), Settings::default());

        let _ = std::fs::remove_dir_all(dir);
    }
}
