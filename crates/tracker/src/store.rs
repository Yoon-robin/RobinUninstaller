//! 추적 기록 저장소: `%LOCALAPPDATA%\RobinUninstaller\traces`.
//!
//! - `pending.json`: 추적을 시작할 때 찍은 스냅샷 (앱을 껐다 켜도 이어서 끝낼 수 있게)
//! - `<ID>.json`: 저장한 추적 기록

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::model::{Snapshot, Trace, is_valid_id};

pub struct TraceStore {
    root: PathBuf,
}

const PENDING: &str = "pending.json";

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(tmp, path)
}

fn invalid() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, "잘못된 기록 ID")
}

impl TraceStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        TraceStore { root: root.into() }
    }

    pub fn default_location() -> Option<Self> {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(TraceStore::new(
            PathBuf::from(local).join("RobinUninstaller").join("traces"),
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn save_pending(&self, snap: &Snapshot) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        write_atomic(&self.root.join(PENDING), &serde_json::to_string(snap)?)
    }

    pub fn load_pending(&self) -> Option<Snapshot> {
        let text = std::fs::read_to_string(self.root.join(PENDING)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// 추적을 시작한 시각 (유닉스 초). 큰 스냅샷 파일을 읽지 않고 파일 시각으로 알아낸다.
    pub fn pending_started_at(&self) -> Option<u64> {
        let modified = std::fs::metadata(self.root.join(PENDING))
            .ok()?
            .modified()
            .ok()?;
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    }

    pub fn clear_pending(&self) -> std::io::Result<()> {
        match std::fs::remove_file(self.root.join(PENDING)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    pub fn save(&self, trace: &Trace) -> std::io::Result<()> {
        if !is_valid_id(&trace.id) {
            return Err(invalid());
        }
        std::fs::create_dir_all(&self.root)?;
        write_atomic(
            &self.root.join(format!("{}.json", trace.id)),
            &serde_json::to_string_pretty(trace)?,
        )
    }

    /// 최근 기록부터.
    pub fn list(&self) -> Vec<Trace> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut traces: Vec<Trace> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let id = name.strip_suffix(".json")?;
                if !is_valid_id(id) {
                    return None;
                }
                serde_json::from_str(&std::fs::read_to_string(e.path()).ok()?).ok()
            })
            .collect();
        traces.sort_by(|a, b| b.id.cmp(&a.id));
        traces
    }

    /// 이 프로그램과 연결된 기록.
    pub fn for_app(&self, app_id: &str) -> Vec<Trace> {
        self.list()
            .into_iter()
            .filter(|t| t.app_ids.iter().any(|a| a == app_id))
            .collect()
    }

    /// 기록만 지운다 (설치된 파일은 건드리지 않는다).
    pub fn delete(&self, id: &str) -> std::io::Result<()> {
        if !is_valid_id(id) {
            return Err(invalid());
        }
        std::fs::remove_file(self.root.join(format!("{id}.json")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Change, ChangeKind};

    #[test]
    fn saves_lists_and_filters_by_app() {
        let dir = std::env::temp_dir().join(format!("robin-trace-test-{}", std::process::id()));
        let store = TraceStore::new(&dir);
        let trace = |id: &str, app: &str| Trace {
            id: id.into(),
            name: app.into(),
            started_at: 1,
            finished_at: 2,
            app_ids: vec![app.into()],
            changes: vec![Change {
                kind: ChangeKind::Folder,
                location: r"C:\Program Files\Acme".into(),
                value_name: None,
            }],
        };
        store.save(&trace("0000000000001-a", "hkcu:acme")).unwrap();
        store.save(&trace("0000000000002-b", "hkcu:other")).unwrap();
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.list()[0].id, "0000000000002-b");
        assert_eq!(store.for_app("hkcu:acme").len(), 1);
        assert!(store.save(&trace(r"..\x", "a")).is_err());

        store.save_pending(&Snapshot::default()).unwrap();
        assert!(store.load_pending().is_some());
        assert!(store.pending_started_at().is_some());
        // pending.json은 기록 목록에 섞이지 않는다
        assert_eq!(store.list().len(), 2);
        store.clear_pending().unwrap();
        assert!(store.load_pending().is_none());

        store.delete("0000000000001-a").unwrap();
        assert_eq!(store.list().len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
