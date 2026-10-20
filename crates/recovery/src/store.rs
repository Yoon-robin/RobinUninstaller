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

    /// 기본 위치: `%LOCALAPPDATA%\RobinX\backups`
    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(Store::new(
            PathBuf::from(local)
                .join(robin_core::APP_DIR)
                .join("backups"),
        ))
    }

    /// 관리자 작업자가 받은 저장소 경로가 이 앱의 저장소 모양인지:
    /// `<사용자 폴더 루트>\<계정>\AppData\Local\RobinX\<sub>` 정확히 그대로(대소문자 무시), `.`/`..` 없음.
    /// 문자열 끝만 보는 검사는 `D:\아무데나\RobinX\backups` 같은 곳도 통과시킨다.
    pub fn is_app_store_path(path: &Path, users_root: &str, sub: &str) -> bool {
        let norm = |s: &str| s.trim().trim_end_matches('\\').to_lowercase();
        let (p, users) = (norm(&path.to_string_lossy()), norm(users_root));
        if users.len() < 4 {
            return false;
        }
        let Some(rest) = p.strip_prefix(&format!("{users}\\")) else {
            return false;
        };
        let parts: Vec<&str> = rest.split('\\').collect();
        let app = robin_core::APP_DIR.to_lowercase();
        matches!(
            parts.as_slice(),
            [account, "appdata", "local", a, s]
                if !account.is_empty()
                    && *account != "."
                    && *account != ".."
                    && !account.contains(':')
                    && *a == app
                    && *s == sub.to_lowercase()
        )
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

    /// 백업 저장소 사용량: (작업 수, 바이트). 링크는 따라가지 않는다.
    pub fn usage(&self) -> (usize, u64) {
        let ops = self.list();
        let bytes = ops.iter().map(|op| dir_size(&self.dir(&op.id))).sum();
        (ops.len(), bytes)
    }

    /// 작업 하나의 백업이 차지하는 바이트. 잘못된 ID면 0.
    pub fn bytes_of(&self, op_id: &str) -> u64 {
        if is_valid_id(op_id) {
            dir_size(&self.dir(op_id))
        } else {
            0
        }
    }

    /// `days`일보다 오래된 작업 ID (최근 작업부터). `now`는 유닉스 초.
    pub fn expired(&self, days: u32, now: u64) -> Vec<String> {
        expired_ids(&self.list(), days, now)
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
        // 항목 하나씩 지우며 진행 상황을 알린다 (파일이 많으면 오래 걸린다)
        if let Ok(entries) = std::fs::read_dir(dir.join("files")) {
            for entry in entries.flatten() {
                let path = entry.path();
                let bytes = dir_size(&path);
                crate::progress::report(|p| p.set_current(path.display()));
                let removed = match entry.file_type() {
                    Ok(t) if t.is_dir() => std::fs::remove_dir_all(&path),
                    // 링크는 따라가지 않고 링크 자신만 지운다 (폴더 링크는 remove_dir로 지워진다)
                    Ok(t) if t.is_symlink() => {
                        std::fs::remove_dir(&path).or_else(|_| std::fs::remove_file(&path))
                    }
                    _ => std::fs::remove_file(&path),
                };
                match removed {
                    Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
                    _ => crate::progress::report(|p| p.add_files(1, bytes)),
                }
            }
        }
        for sub in ["files", "registry"] {
            match std::fs::remove_dir_all(dir.join(sub)) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
                _ => {}
            }
        }
        std::fs::remove_dir_all(dir)
    }
}

/// 폴더 크기. 링크는 따라가지 않는다.
fn dir_size(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.file_type().is_symlink() {
        0
    } else if meta.is_dir() {
        std::fs::read_dir(path)
            .map(|entries| entries.flatten().map(|e| dir_size(&e.path())).sum())
            .unwrap_or(0)
    } else {
        meta.len()
    }
}

/// 보관 기간이 지난 작업. 순수 함수.
/// 복원하다 충돌이 난 항목이 있으면(그 파일은 백업에만 있다) 기간이 지나도 자동으로 지우지 않는다.
pub fn expired_ids(ops: &[Operation], days: u32, now: u64) -> Vec<String> {
    let limit = u64::from(days) * 24 * 60 * 60;
    ops.iter()
        .filter(|op| now.saturating_sub(op.created_at) > limit)
        .filter(|op| {
            !op.items
                .iter()
                .any(|i| i.status == crate::model::ItemStatus::Conflicted)
        })
        .map(|op| op.id.clone())
        .collect()
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
    #[test]
    fn store_path_must_be_the_app_folder_under_a_user_profile() {
        use super::Store;
        use std::path::Path;
        let ok = |p: &str| Store::is_app_store_path(Path::new(p), r"C:\Users", "backups");
        assert!(ok(r"C:\Users\me\AppData\Local\RobinX\backups"));
        assert!(ok(r"c:\users\Me\appdata\local\robinx\BACKUPS\"));
        assert!(!ok(r"D:\evil\RobinX\backups"));
        assert!(!ok(r"C:\Users\me\Documents\AppData\Local\RobinX\backups"));
        assert!(!ok(
            r"C:\Users\me\AppData\Local\RobinX\backups\..\..\..\Desktop"
        ));
        assert!(!ok(r"C:\Users\..\AppData\Local\RobinX\backups"));
        assert!(!ok(r"C:\Users\me\AppData\Local\RobinX\startup"));
        assert!(!ok(r"C:\Users\me\AppData\Local\RobinUninstaller\backups"));
        assert!(!Store::is_app_store_path(
            Path::new(r"C:\me\AppData\Local\RobinX\backups"),
            "C:",
            "backups"
        ));
    }

    use super::*;

    #[test]
    fn finds_expired_operations() {
        let op = |id: &str, created_at: u64| Operation {
            id: id.into(),
            app_name: "x".into(),
            created_at,
            items: vec![],
        };
        let day = 24 * 60 * 60;
        let now = 100 * day;
        let ops = vec![
            op("a", now - 10 * day),
            op("b", now - 31 * day),
            op("c", now + day),
        ];
        assert_eq!(expired_ids(&ops, 30, now), vec!["b".to_string()]);
        assert!(expired_ids(&ops, 90, now).is_empty());
    }

    #[test]
    fn rejects_path_like_ids() {
        assert!(is_valid_id("1700000000000-1a2b"));
        assert!(!is_valid_id(r"..\..\Windows"));
        assert!(!is_valid_id(""));
    }
}
