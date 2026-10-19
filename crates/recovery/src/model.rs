//! 정리 작업 기록(매니페스트). 작업마다 백업 폴더 하나와 `manifest.json` 하나가 생긴다.
//!
//! ```text
//! %LOCALAPPDATA%\RobinX\backups\<작업 ID>\
//!   manifest.json
//!   files\<번호>\<원래 이름>      격리한 파일·폴더
//!   registry\<번호>.json          레지스트리 키·값 백업
//! ```

use robin_scanner::{Leftover, LeftoverKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemStatus {
    /// 아직 처리 전
    Pending,
    /// 백업·격리하고 원래 자리에서 없앰
    Removed,
    /// 관리자 권한이 있어야 지울 수 있음
    NeedsAdmin,
    /// 지우는 시점에 이미 없었음
    AlreadyGone,
    /// 안전 검사에 걸려 건드리지 않음
    Blocked,
    /// 실패 (이유는 `error`)
    Failed,
    /// 원래 자리로 되돌림
    Restored,
    /// 일부만 정리됨: 백업은 온전하지만 원래 자리에 일부가 남음 (다시 정리하거나 되돌릴 수 있다)
    Partial,
    /// 되돌렸지만 원래 자리에 내용이 다른 파일이 있어 일부는 덮어쓰지 않았음.
    /// 그 파일들은 **백업에만** 남아 있으므로, 이 상태가 있는 백업은 자동으로 지우지 않는다.
    Conflicted,
}

/// 백업이 어디에 있는지 (작업 폴더 기준 상대 경로).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Backup {
    /// 격리 폴더로 옮긴 파일·폴더
    Quarantine { path: String },
    /// 레지스트리 키·값을 기록한 JSON
    Registry { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationItem {
    pub id: String,
    pub kind: LeftoverKind,
    pub location: String,
    pub value_name: Option<String>,
    pub size_bytes: Option<u64>,
    pub status: ItemStatus,
    pub backup: Option<Backup>,
    pub error: Option<String>,
}

impl OperationItem {
    pub fn from_leftover(l: &Leftover) -> Self {
        OperationItem {
            id: l.id.clone(),
            kind: l.kind,
            location: l.location.clone(),
            value_name: l.value_name.clone(),
            size_bytes: l.size_bytes,
            status: ItemStatus::Pending,
            backup: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub app_name: String,
    /// 유닉스 시간(초)
    pub created_at: u64,
    pub items: Vec<OperationItem>,
}

impl OperationItem {
    /// 이 항목에 쓸 수 있는 백업 상대 경로인지. 매니페스트는 일반 사용자도 고칠 수 있는 곳에 있으므로,
    /// 관리자 작업자가 엉뚱한 파일을 읽거나 옮기지 않게 정해진 모양만 받는다.
    pub fn backup_path_is_valid(&self, index: usize) -> bool {
        match &self.backup {
            None => true,
            Some(Backup::Registry { path }) => *path == format!("registry\\{index}.json"),
            Some(Backup::Quarantine { path }) => path
                .strip_prefix(&format!("files\\{index}\\"))
                .is_some_and(|name| {
                    !name.is_empty()
                        && name == safe_file_name(name)
                        && name != "."
                        && name != ".."
                        && !name.contains('\\')
                }),
        }
    }
}

impl Operation {
    pub fn count(&self, status: ItemStatus) -> usize {
        self.items.iter().filter(|i| i.status == status).count()
    }
}

/// 작업 ID: 시간 순으로 정렬되고 겹치지 않게 `밀리초-프로세스번호`.
pub fn new_operation_id(now_millis: u128, pid: u32) -> String {
    format!("{now_millis:013}-{pid:x}")
}

/// 파일 이름으로 쓸 수 없는 글자를 바꾼다 (격리 폴더 안 이름용).
pub fn safe_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if r#"<>:"/\|?*"#.contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '.' || c == ' ');
    if trimmed.is_empty() {
        "item".into()
    } else {
        trimmed.chars().take(120).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_sort_by_time() {
        let a = new_operation_id(1_700_000_000_000, 10);
        let b = new_operation_id(1_700_000_000_001, 9);
        assert!(a < b);
    }

    #[test]
    fn file_names_are_sanitized() {
        assert_eq!(safe_file_name("Fixture Notes"), "Fixture Notes");
        assert_eq!(safe_file_name(r"a:b\c"), "a_b_c");
        assert_eq!(safe_file_name(" .. "), "item");
    }
}
