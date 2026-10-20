//! 폴더 아래 전체를 여러 스레드로 훑어 크기와 문서 파일 수를 센다 (읽기 전용).
//!
//! 큰 폴더(설치 폴더 등)는 하위 폴더가 수만 개라 한 스레드로 차례로 읽으면 디스크를 기다리는 시간이 길다.
//! 하위 폴더를 여러 스레드가 나눠 읽는다. 심볼릭 링크·정션은 따라가지 않는다.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rayon::prelude::*;

/// 사용자가 만든 문서로 보는 확장자. 이런 파일이 있으면 감점하고 경고한다.
pub(crate) const DOCUMENT_EXTENSIONS: &[&str] = &[
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf", "hwp", "hwpx", "odt", "ods", "psd", "ai",
    "indd", "prproj", "aep", "blend", "flp", "als", "kra", "xcf", "cr2", "nef", "arw", "dng",
];

/// 확장자로 본 사용자 문서인지 (대소문자 무시)
pub(crate) fn is_document_name(name: &std::ffi::OsStr) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            DOCUMENT_EXTENSIONS
                .iter()
                .any(|d| d.eq_ignore_ascii_case(ext))
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tally {
    pub size: u64,
    pub docs: usize,
    /// 항목 수 한도 안에서 끝까지 셌는지
    pub complete: bool,
}

/// `dir` 아래 파일 크기 합과 문서 파일 수. 항목을 `limit`개 넘게 보면 거기서 멈추고 `complete: false`.
/// 하위 폴더는 여러 스레드가 나눠 읽는다 (저장공간 분석과 같은 rayon 방식).
pub(crate) fn tally(dir: &Path, limit: usize) -> Tally {
    let visited = AtomicUsize::new(0);
    let over = AtomicBool::new(false);
    let (size, docs) = walk(dir, limit, &visited, &over);
    Tally {
        size,
        docs,
        complete: !over.into_inner(),
    }
}

fn walk(dir: &Path, limit: usize, visited: &AtomicUsize, over: &AtomicBool) -> (u64, usize) {
    if over.load(Ordering::Relaxed) {
        return (0, 0);
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (0, 0);
    };
    let (mut size, mut docs, mut seen) = (0u64, 0usize, 0usize);
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        seen += 1;
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            dirs.push(entry.path());
        } else {
            // Windows는 폴더를 읽을 때 받은 정보로 크기를 주므로 파일을 따로 열지 않는다
            size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            if is_document_name(&entry.file_name()) {
                docs += 1;
            }
        }
    }
    if visited.fetch_add(seen, Ordering::Relaxed) + seen > limit {
        over.store(true, Ordering::Relaxed);
        return (size, docs);
    }
    let (sub_size, sub_docs) = dirs
        .par_iter()
        .map(|d| walk(d, limit, visited, over))
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    (size + sub_size, docs + sub_docs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_every_file_and_document_and_respects_the_limit() {
        let root = std::env::temp_dir().join(format!("robin-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut expected = 0u64;
        for i in 0..20 {
            let d = root.join(format!("d{i}")).join("sub");
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("a.bin"), vec![0u8; i + 1]).unwrap();
            expected += i as u64 + 1;
        }
        std::fs::write(root.join("report.DOCX"), b"xx").unwrap();
        expected += 2;

        let t = tally(&root, 10_000);
        assert_eq!(
            t,
            Tally {
                size: expected,
                docs: 1,
                complete: true
            }
        );
        // 항목이 한도보다 많으면 끝까지 셌다고 하지 않는다
        assert!(!tally(&root, 5).complete);
        // 없는 폴더는 0
        assert_eq!(tally(&root.join("none"), 10).size, 0);
        let _ = std::fs::remove_dir_all(&root);
    }
}
