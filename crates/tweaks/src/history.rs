//! 바꾸기 전 원래 값 기록.
//!
//! ```text
//! %LOCALAPPDATA%\RobinX\tweaks\changes.json
//! ```
//!
//! 레지스트리를 바꾸기 **전에** 먼저 기록한다. 기록을 저장하지 못하면 바꾸지 않는다.
//! 같은 항목을 다시 적용해도 처음 기록(진짜 원래 값)을 덮어쓰지 않는다.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::RawValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Original {
    /// `HKEY_CURRENT_USER` 아래 경로
    pub key: String,
    pub name: String,
    /// 바꾸기 전 값. `None`이면 값이 없었다 (되돌릴 때 지운다).
    pub previous: Option<RawValue>,
    /// 바꾸기 전에 키가 있었는지. 없었으면 되돌릴 때 빈 키도 지운다.
    pub key_existed: bool,
    /// 적용하면서 새로 만든 가장 위쪽 키 (`key`의 상위 중 없던 것). 되돌릴 때 여기까지 빈 키를 지운다.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub originals: Vec<Original>,
    pub applied_at: u64,
}

pub struct History {
    root: PathBuf,
}

impl History {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        History { root: root.into() }
    }

    /// 기본 위치: `%LOCALAPPDATA%\RobinX\tweaks`
    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(History::new(
            PathBuf::from(local)
                .join(robin_core::APP_DIR)
                .join("tweaks"),
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn file(&self) -> PathBuf {
        self.root.join("changes.json")
    }

    /// 항목 ID → 기록. 파일이 없거나 깨졌으면 빈 기록.
    pub fn load(&self) -> BTreeMap<String, Record> {
        std::fs::read_to_string(self.file())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<Record> {
        self.load().remove(id)
    }

    /// 이미 기록이 있으면 그대로 두고 `false`. 새로 남겼으면 `true`.
    pub fn remember(&self, id: &str, record: Record) -> std::io::Result<bool> {
        let mut all = self.load();
        if all.contains_key(id) {
            return Ok(false);
        }
        all.insert(id.to_string(), record);
        self.save(&all)?;
        Ok(true)
    }

    pub fn forget(&self, id: &str) -> std::io::Result<()> {
        let mut all = self.load();
        if all.remove(id).is_some() {
            self.save(&all)?;
        }
        Ok(())
    }

    /// 임시 파일에 쓰고 바꿔치기해서, 쓰다 꺼져도 깨진 파일이 남지 않게 한다.
    fn save(&self, all: &BTreeMap<String, Record>) -> std::io::Result<()> {
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
    use crate::model::Value;

    #[test]
    fn first_record_wins_and_forget_removes_it() {
        let dir = std::env::temp_dir().join(format!("robin-tweaks-test-{}", std::process::id()));
        let history = History::new(&dir);
        let record = |v| Record {
            originals: vec![Original {
                key: r"Software\X".into(),
                name: "A".into(),
                previous: Some(Value::Dword(v).raw()),
                key_existed: true,
                created_from: None,
            }],
            applied_at: 1,
        };
        assert!(history.remember("a.b", record(7)).unwrap());
        assert!(!history.remember("a.b", record(9)).unwrap());
        assert_eq!(history.get("a.b"), Some(record(7)));
        history.forget("a.b").unwrap();
        assert_eq!(history.get("a.b"), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
