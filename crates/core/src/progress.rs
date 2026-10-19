//! 오래 걸리는 탐색의 진행 상황.
//!
//! 탐색하는 쪽(여러 스레드)은 숫자만 올리고, 화면에 보내는 쪽이 짧은 간격으로 [`Progress::snapshot`]을 읽어 보낸다.
//! 그래서 탐색 코드는 화면이나 채널을 몰라도 되고, 파일마다 메시지를 보내는 부담도 없다.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

#[derive(Debug, Default)]
pub struct Progress {
    files: AtomicU64,
    bytes: AtomicU64,
    done: AtomicU64,
    total: AtomicU64,
    current: Mutex<String>,
}

/// 화면에 보내는 한 순간의 진행 상황
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    /// 지금까지 살펴본 파일 수
    pub files: u64,
    /// 지금까지 센 크기
    pub bytes: u64,
    /// 끝낸 단위(항목·폴더) 수와 전체 수. 전체를 모르면 0
    pub done: u64,
    pub total: u64,
    /// 지금 살펴보는 곳 (항목 이름이나 폴더 경로)
    pub current: String,
}

impl Progress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&self, bytes: u64) {
        self.files.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_files(&self, files: u64, bytes: u64) {
        self.files.fetch_add(files, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn unit_done(&self) {
        self.done.fetch_add(1, Ordering::Relaxed);
    }

    /// 지금 살펴보는 곳. 다른 스레드가 쓰는 중이면 이번 것은 건너뛴다 (어차피 곧 다시 바뀐다).
    pub fn set_current(&self, current: impl std::fmt::Display) {
        if let Ok(mut c) = self.current.try_lock() {
            c.clear();
            use std::fmt::Write;
            let _ = write!(c, "{current}");
        }
    }

    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            files: self.files.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
            done: self.done.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            current: self.current.lock().map(|c| c.clone()).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_from_many_threads() {
        let p = Progress::new();
        p.set_total(4);
        std::thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| {
                    for _ in 0..1000 {
                        p.add_file(2);
                    }
                    p.unit_done();
                });
            }
        });
        p.set_current("C:\\Temp");
        let snap = p.snapshot();
        assert_eq!(
            (snap.files, snap.bytes, snap.done, snap.total),
            (4000, 8000, 4, 4)
        );
        assert_eq!(snap.current, "C:\\Temp");
    }
}
