//! 작업 기록 저장소. 매니페스트는 매 항목 처리 뒤 곧바로 저장해, 중간에 꺼져도 무엇을 했는지 남는다.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::model::Operation;

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Store { root: root.into() }
    }

    /// 기본 위치: `%LOCALAPPDATA%\RobinUninstaller\backups`
    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(Store::new(
            PathBuf::from(local)
                .join("RobinUninstaller")
                .join("backups"),
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn dir(&self, op_id: &str) -> PathBuf {
        self.root.join(op_id)
    }

    /// 매니페스트를 임시 파일에 쓰고 바꿔치기해서, 쓰다 꺼져도 깨진 파일이 남지 않게 한다.
    pub fn save(&self, op: &Operation) -> std::io::Result<()> {
        let dir = self.dir(&op.id);
        std::fs::create_dir_all(&dir)?;
        let tmp = dir.join("manifest.json.tmp");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(serde_json::to_string_pretty(op)?.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(tmp, dir.join("manifest.json"))
    }

    pub fn load(&self, op_id: &str) -> std::io::Result<Operation> {
        if !is_valid_id(op_id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "잘못된 작업 ID",
            ));
        }
        let text = std::fs::read_to_string(self.dir(op_id).join("manifest.json"))?;
        let op: Operation = serde_json::from_str(&text).map_err(std::io::Error::other)?;
        // 다른 작업의 매니페스트를 복사해 넣은 경우 등: 폴더 이름과 기록의 ID가 같아야 한다
        if op.id != op_id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "작업 기록의 ID가 폴더와 달라요",
            ));
        }
        Ok(op)
    }

    /// 최근 작업부터.
    pub fn list(&self) -> Vec<Operation> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut ops: Vec<Operation> = entries
            .flatten()
            .filter_map(|e| self.load(&e.file_name().to_string_lossy()).ok())
            .collect();
        ops.sort_by(|a, b| b.id.cmp(&a.id));
        ops
    }

    /// 작업 기록과 백업을 완전히 지운다 (사용자가 요청했을 때만).
    pub fn delete(&self, op_id: &str) -> std::io::Result<()> {
        if !is_valid_id(op_id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "잘못된 작업 ID",
            ));
        }
        // 보관한 파일을 먼저 지우고 기록은 마지막에 지운다. 중간에 실패해도(권한 등) 기록이 남아 목록에 보인다.
        let dir = self.dir(op_id);
        for sub in ["files", "registry"] {
            match std::fs::remove_dir_all(dir.join(sub)) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
                _ => {}
            }
        }
        std::fs::remove_dir_all(dir)
    }
}

/// 작업 ID는 숫자·소문자 16진수·하이픈만 (경로 조작 방지).
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_like_ids() {
        assert!(is_valid_id("1700000000000-1a2b"));
        assert!(!is_valid_id(r"..\..\Windows"));
        assert!(!is_valid_id(""));
    }
}
