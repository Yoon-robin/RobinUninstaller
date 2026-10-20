//! RobinX가 끈 항목의 원래 `StartupApproved` 값(작업이면 원래 `Enabled` 값) 기록.
//!
//! 다시 켤 때 이 값을 그대로 되돌려 놓는다 (원래 값이 없었다면 값을 지운다).
//! 작업 관리자 등 다른 곳에서 끈 항목은 기록이 없으므로 작업 관리자와 같은 "켜짐" 값을 쓴다.
//!
//! ```text
//! %LOCALAPPDATA%\RobinX\startup\changes.json
//! ```

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    /// 끄기 전의 `StartupApproved` 값. `None`이면 값이 없었음 (작업은 항상 `None`).
    pub previous: Option<Vec<u8>>,
    /// 작업 스케줄러 작업의 끄기 전 `Enabled` 값. 작업이 아니면 `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_enabled: Option<bool>,
    /// 사람이 기록을 읽을 때 알아보기 위한 이름
    pub name: String,
    pub changed_at: u64,
}

pub struct History {
    root: PathBuf,
}

impl History {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        History { root: root.into() }
    }

    /// 기본 위치: `%LOCALAPPDATA%\RobinX\startup`
    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(History::new(
            PathBuf::from(local)
                .join(robin_core::APP_DIR)
                .join("startup"),
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn file(&self) -> PathBuf {
        self.root.join("changes.json")
    }

    /// 항목 ID → 기록. 파일이 없거나 깨졌으면 빈 기록.
    pub fn load(&self) -> BTreeMap<String, Change> {
        std::fs::read_to_string(self.file())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<Change> {
        self.load().remove(id)
    }

    /// 기록을 바꿔 저장할 때 읽는다. 파일이 없으면 빈 기록이지만, 있는데 읽지 못하면 오류다:
    /// 빈 기록으로 보고 저장하면 다른 항목들의 원래 값이 모두 사라진다.
    fn load_for_update(&self) -> std::io::Result<BTreeMap<String, Change>> {
        match std::fs::read_to_string(self.file()) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(e),
            Ok(text) => serde_json::from_str(&text).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("시작프로그램 변경 기록이 손상됐어요: {e}"),
                )
            }),
        }
    }

    /// 읽고 바꾸고 저장하는 동안 다른 변경이 끼어들지 않게 한다.
    fn update<T>(
        &self,
        f: impl FnOnce(&mut BTreeMap<String, Change>) -> Option<T>,
    ) -> std::io::Result<Option<T>> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut all = self.load_for_update()?;
        let out = f(&mut all);
        if out.is_some() {
            self.save(&all)?;
        }
        Ok(out)
    }

    /// 이미 기록이 있으면 덮어쓰지 않는다 (처음 끄기 전의 값이 진짜 원래 값).
    pub fn remember(&self, id: &str, change: Change) -> std::io::Result<()> {
        self.update(|all| {
            (!all.contains_key(id)).then(|| {
                all.insert(id.to_string(), change);
            })
        })
        .map(|_| ())
    }

    pub fn forget(&self, id: &str) -> std::io::Result<()> {
        self.update(|all| all.remove(id).map(|_| ())).map(|_| ())
    }

    /// 임시 파일에 쓰고 바꿔치기해서, 쓰다 꺼져도 깨진 파일이 남지 않게 한다.
    fn save(&self, all: &BTreeMap<String, Change>) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let tmp = self
            .root
            .join(format!("changes.json.{}.tmp", std::process::id()));
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(serde_json::to_string_pretty(all)?.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp, self.file()).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_first_original_value() {
        let dir = std::env::temp_dir().join(format!("robin-startup-test-{}", std::process::id()));
        let history = History::new(&dir);
        let first = Change {
            previous: None,
            previous_enabled: None,
            name: "A".into(),
            changed_at: 1,
        };
        history.remember("0123456789abcdef", first.clone()).unwrap();
        history
            .remember(
                "0123456789abcdef",
                Change {
                    previous: Some(vec![3]),
                    previous_enabled: None,
                    name: "A".into(),
                    changed_at: 2,
                },
            )
            .unwrap();
        assert_eq!(history.get("0123456789abcdef"), Some(first));

        history.forget("0123456789abcdef").unwrap();
        assert_eq!(history.get("0123456789abcdef"), None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn never_saves_over_an_unreadable_history() {
        let dir = std::env::temp_dir().join(format!("robin-startup-broken-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let history = History::new(&dir);
        std::fs::write(dir.join("changes.json"), "{ not json").unwrap();
        let change = Change {
            previous: Some(vec![2]),
            previous_enabled: None,
            name: "B".into(),
            changed_at: 1,
        };
        assert!(history.remember("fedcba9876543210", change).is_err());
        assert!(history.forget("fedcba9876543210").is_err());
        assert_eq!(
            std::fs::read_to_string(dir.join("changes.json")).unwrap(),
            "{ not json"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
