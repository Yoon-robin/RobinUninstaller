//! RobinUninstaller가 끈 항목의 원래 `StartupApproved` 값 기록.
//!
//! 다시 켤 때 이 값을 그대로 되돌려 놓는다 (원래 값이 없었다면 값을 지운다).
//! 작업 관리자 등 다른 곳에서 끈 항목은 기록이 없으므로 작업 관리자와 같은 "켜짐" 값을 쓴다.
//!
//! ```text
//! %LOCALAPPDATA%\RobinUninstaller\startup\changes.json
//! ```

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    /// 끄기 전의 값. `None`이면 값이 없었음.
    pub previous: Option<Vec<u8>>,
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

    /// 기본 위치: `%LOCALAPPDATA%\RobinUninstaller\startup`
    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(History::new(
            PathBuf::from(local)
                .join("RobinUninstaller")
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

    /// 이미 기록이 있으면 덮어쓰지 않는다 (처음 끄기 전의 값이 진짜 원래 값).
    pub fn remember(&self, id: &str, change: Change) -> std::io::Result<()> {
        let mut all = self.load();
        if all.contains_key(id) {
            return Ok(());
        }
        all.insert(id.to_string(), change);
        self.save(&all)
    }

    pub fn forget(&self, id: &str) -> std::io::Result<()> {
        let mut all = self.load();
        if all.remove(id).is_some() {
            self.save(&all)?;
        }
        Ok(())
    }

    /// 임시 파일에 쓰고 바꿔치기해서, 쓰다 꺼져도 깨진 파일이 남지 않게 한다.
    fn save(&self, all: &BTreeMap<String, Change>) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let tmp = self.root.join("changes.json.tmp");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(serde_json::to_string_pretty(all)?.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(tmp, self.file())
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
            name: "A".into(),
            changed_at: 1,
        };
        history.remember("0123456789abcdef", first.clone()).unwrap();
        history
            .remember(
                "0123456789abcdef",
                Change {
                    previous: Some(vec![3]),
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
}
