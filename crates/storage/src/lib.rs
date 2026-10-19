//! 저장공간 정리.
//!
//! - **스캔**은 읽기 전용이다. 규칙마다 크기·파일 수·실행 중이라 막힌 이유를 계산한다.
//! - **정리**는 다시 만들어지는 것(캐시·임시 파일)만 기본으로 한다. 되돌릴 수 없는 항목은 호출자가
//!   `allow_irreversible`로 따로 허락해야 하고, 사용자 파일은 규칙에 아예 없다 (`rules` 테스트 참고).
//! - 정리할 때는 화면에서 넘어온 값을 믿지 않는다. 규칙 ID만 받아 경로와 조건을 처음부터 다시 계산한다.
//! - **분석**은 폴더별 크기를 보여 주기만 한다. 아무것도 지우지 않는다.

pub mod rules;
#[cfg(windows)]
pub mod system;
pub mod walk;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use rules::{Kind, Risk, Rule};
use walk::Tally;

/// 스캔 결과의 항목 하나 (규칙 하나).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
    pub why: String,
    pub risk: Risk,
    pub kind: Kind,
    pub admin: bool,
    pub default_selected: bool,
    pub bytes: u64,
    pub files: u64,
    /// 권한이 없어 크기를 다 세지 못한 폴더 수
    pub inaccessible: u64,
    pub locations: Vec<String>,
    /// 실행 중이라 지금은 정리할 수 없게 만드는 프로그램
    pub running: Vec<String>,
    pub guidance: Option<String>,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn min_age(rule: &Rule) -> Duration {
    Duration::from_secs(rule.min_age_hours * 3600)
}

fn running_blockers(rule: &Rule, running: &std::collections::HashSet<String>) -> Vec<String> {
    rule.blocked_by
        .iter()
        .filter(|exe| running.contains(**exe))
        .map(|exe| exe.to_string())
        .collect()
}

/// 정리할 수 있는 항목을 찾는다 (읽기 전용). 크기가 0인 항목은 뺀다.
#[cfg(windows)]
pub fn scan() -> Vec<Category> {
    let running = robin_execution::running_exe_names();
    let mut out: Vec<Category> = rules::all()
        .collect::<Vec<_>>()
        .par_iter()
        .filter_map(|rule| {
            let paths = walk::rule_paths(rule, &env);
            let mut tally = Tally::default();
            match rule.kind {
                Kind::RecycleBin => {
                    let (bytes, items) = system::recycle_bin_size();
                    tally.bytes = bytes;
                    tally.files = items;
                }
                Kind::Files | Kind::Info => {
                    for p in &paths {
                        tally.add(walk::measure(p, min_age(rule)));
                    }
                }
            }
            // 관리자 항목은 크기를 못 세도(권한 없음) 보여 준다
            let visible = tally.bytes > 0 || (rule.admin && tally.inaccessible > 0);
            visible.then(|| Category {
                id: rule.id.into(),
                name: rule.name.into(),
                description: rule.description.into(),
                why: rule.why.into(),
                risk: rule.risk,
                kind: rule.kind,
                admin: rule.admin,
                default_selected: rule.default_selected(),
                bytes: tally.bytes,
                files: tally.files,
                inaccessible: tally.inaccessible,
                locations: paths.iter().map(|p| p.display().to_string()).collect(),
                running: running_blockers(rule, &running),
                guidance: rule.guidance.map(Into::into),
            })
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.bytes));
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub id: String,
    pub freed_bytes: u64,
    pub deleted_files: u64,
    /// 잠겨 있거나 권한이 없어 지우지 못한 파일
    pub failed_files: u64,
    /// 손대지 않은 이유
    pub skipped: Option<String>,
    /// 관리자 권한으로 다시 시도해야 함
    pub needs_admin: bool,
}

impl CleanResult {
    fn skipped(id: &str, reason: impl Into<String>) -> Self {
        CleanResult {
            id: id.into(),
            freed_bytes: 0,
            deleted_files: 0,
            failed_files: 0,
            skipped: Some(reason.into()),
            needs_admin: false,
        }
    }
}

/// 정리하는 동안 화면에 보내는 소식. 오래 걸려도 멈춘 것처럼 보이지 않게 한다.
#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CleanEvent {
    /// 항목 하나를 시작한다. `index`는 0부터.
    Started {
        id: String,
        index: usize,
        total: usize,
    },
    /// 지금까지의 진행 (자주 보내지 않는다: 약 0.1초마다)
    Progress {
        id: String,
        freed_bytes: u64,
        deleted_files: u64,
        failed_files: u64,
        /// 방금 지운(또는 지우려던) 파일
        current: String,
    },
    /// 항목 하나가 끝났다
    Finished { result: CleanResult },
}

/// 진행 소식을 보내는 간격
const PROGRESS_EVERY: Duration = Duration::from_millis(100);

/// 고른 항목을 정리한다. 되돌릴 수 없는 항목은 `allow_irreversible`이 켜져 있을 때만 지운다.
#[cfg(windows)]
pub fn clean(ids: &[String], allow_irreversible: bool) -> Vec<CleanResult> {
    clean_with_progress(ids, allow_irreversible, &mut |_| {})
}

/// [`clean`]과 같고, 항목을 시작·진행·끝낼 때마다 `on_event`를 부른다.
#[cfg(windows)]
pub fn clean_with_progress(
    ids: &[String],
    allow_irreversible: bool,
    on_event: &mut dyn FnMut(CleanEvent),
) -> Vec<CleanResult> {
    let running = robin_execution::running_exe_names();
    let elevated = system::is_elevated();
    let total = ids.len();

    let mut results = Vec::with_capacity(total);
    for (index, id) in ids.iter().enumerate() {
        on_event(CleanEvent::Started {
            id: id.clone(),
            index,
            total,
        });
        let result = clean_one(id, allow_irreversible, &running, elevated, on_event);
        on_event(CleanEvent::Finished {
            result: result.clone(),
        });
        results.push(result);
    }
    results
}

#[cfg(windows)]
fn clean_one(
    id: &str,
    allow_irreversible: bool,
    running: &std::collections::HashSet<String>,
    elevated: bool,
    on_event: &mut dyn FnMut(CleanEvent),
) -> CleanResult {
    let Some(rule) = rules::find(id) else {
        return CleanResult::skipped(id, "알 수 없는 항목이에요");
    };
    if rule.kind == Kind::Info {
        return CleanResult::skipped(id, "직접 정리하는 항목이에요");
    }
    if rule.risk == Risk::Irreversible && !allow_irreversible {
        return CleanResult::skipped(id, "되돌릴 수 없는 항목은 따로 확인해야 지워요");
    }
    let blockers = running_blockers(rule, running);
    if !blockers.is_empty() {
        return CleanResult::skipped(
            id,
            format!("{} 실행 중이라 건너뛰었어요", blockers.join(", ")),
        );
    }
    if rule.admin && !elevated {
        return CleanResult {
            needs_admin: true,
            ..CleanResult::skipped(id, "관리자 권한이 필요해요")
        };
    }

    if rule.kind == Kind::RecycleBin {
        let (bytes, items) = system::recycle_bin_size();
        // 이미 비어 있으면 Windows가 오류를 돌려주므로 비울 것이 없으면 그대로 끝낸다
        if items == 0 {
            return CleanResult {
                skipped: None,
                ..CleanResult::skipped(id, "")
            };
        }
        // Windows가 한 번에 비우므로 중간 진행은 알 수 없다. 화면은 걸린 시간을 보여 준다.
        return match system::empty_recycle_bin() {
            Ok(()) => CleanResult {
                freed_bytes: bytes,
                deleted_files: items,
                skipped: None,
                ..CleanResult::skipped(id, "")
            },
            Err(e) => CleanResult::skipped(id, format!("휴지통을 비우지 못했어요: {e}")),
        };
    }

    let mut result = CleanResult {
        skipped: None,
        ..CleanResult::skipped(id, "")
    };
    let mut last_sent = std::time::Instant::now();
    for path in walk::rule_paths(rule, &env) {
        // 지우는 파일의 실제 위치가 이 폴더 안인지 핸들로 확인한다 (정션 바꿔치기 방지)
        let Ok(root) = std::fs::canonicalize(&path) else {
            continue;
        };
        walk::visit(
            &path,
            min_age(rule),
            &mut |file, len| {
                match system::delete_file_within(file, &root) {
                    Ok(()) => {
                        result.freed_bytes += len;
                        result.deleted_files += 1;
                    }
                    // 실행 중인 프로그램이 쥐고 있는 파일 등은 건너뛴다
                    Err(_) => result.failed_files += 1,
                }
                if last_sent.elapsed() >= PROGRESS_EVERY {
                    last_sent = std::time::Instant::now();
                    on_event(CleanEvent::Progress {
                        id: id.to_string(),
                        freed_bytes: result.freed_bytes,
                        deleted_files: result.deleted_files,
                        failed_files: result.failed_files,
                        current: file.to_string_lossy().into_owned(),
                    });
                }
            },
            // 비게 된 하위 폴더만 지운다 (비어 있지 않으면 실패하고 그대로 남는다)
            &mut |dir| {
                let _ = std::fs::remove_dir(dir);
            },
        );
    }
    result
}

/// 큰 파일 하나 (읽기 전용 결과)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFile {
    pub name: String,
    pub path: String,
    pub bytes: u64,
    /// 마지막으로 고친 때 (유닉스 초)
    pub modified: Option<u64>,
}

/// `root` 아래에서 `min_bytes` 이상인 파일을 큰 것부터 `limit`개 찾는다. 읽기 전용, 아무것도 지우지 않는다.
/// Windows 폴더(손댈 수 없는 시스템 파일)와 숨김 시스템 파일·링크는 건너뛴다. `cancel`이 켜지면 중간에 멈춘다.
pub fn largest_files(
    root: &Path,
    min_bytes: u64,
    limit: usize,
    cancel: &AtomicBool,
) -> Vec<LargeFile> {
    let mut out = collect_large(root, min_bytes, cancel);
    out.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    out.truncate(limit);
    out
}

fn collect_large(dir: &Path, min_bytes: u64, cancel: &AtomicBool) -> Vec<LargeFile> {
    if cancel.load(Ordering::Relaxed) || in_windows_dir(dir) {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let Ok(m) = e.metadata() else { continue };
        #[cfg(windows)]
        let attrs = std::os::windows::fs::MetadataExt::file_attributes(&m);
        #[cfg(not(windows))]
        let attrs = 0;
        if walk::is_untouchable(attrs) {
            continue;
        }
        if m.is_dir() {
            dirs.push(e.path());
        } else if m.len() >= min_bytes {
            found.push(LargeFile {
                name: e.file_name().to_string_lossy().into_owned(),
                path: e.path().to_string_lossy().into_owned(),
                bytes: m.len(),
                modified: m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
            });
        }
    }
    let sub: Vec<LargeFile> = dirs
        .par_iter()
        .flat_map_iter(|d| collect_large(d, min_bytes, cancel))
        .collect();
    found.extend(sub);
    found
}

/// 폴더 분석 결과 한 줄
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub bytes: u64,
    pub files: u64,
    pub inaccessible: u64,
    /// Windows 폴더 안: 하드 링크(WinSxS 등)를 여러 번 세어 실제 차지하는 공간보다 크게 보일 수 있다.
    /// 파일마다 열어 링크를 확인하면 분석이 매우 느려져서, 정확히 세는 대신 화면에 알린다.
    pub approximate: bool,
}

/// `path`가 Windows 폴더이거나 그 안인지 (하드 링크가 많은 곳)
fn in_windows_dir(path: &Path) -> bool {
    std::env::var("SystemRoot").is_ok_and(|w| {
        let p = path.to_string_lossy().to_lowercase();
        let w = w.trim_end_matches('\\').to_lowercase();
        !w.is_empty() && (p == w || p.starts_with(&format!("{w}\\")))
    })
}

fn dir_size(path: &Path, cancel: &AtomicBool) -> Tally {
    if cancel.load(Ordering::Relaxed) {
        return Tally::default();
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return Tally {
            inaccessible: 1,
            ..Tally::default()
        };
    };
    let mut tally = Tally::default();
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let Ok(m) = e.metadata() else { continue };
        #[cfg(windows)]
        let attrs = std::os::windows::fs::MetadataExt::file_attributes(&m);
        #[cfg(not(windows))]
        let attrs = 0;
        if walk::is_untouchable(attrs) {
            continue;
        }
        if m.is_dir() {
            dirs.push(e.path());
        } else {
            tally.bytes += m.len();
            tally.files += 1;
        }
    }
    let sub = dirs
        .par_iter()
        .map(|d| dir_size(d, cancel))
        .reduce(Tally::default, |mut a, b| {
            a.add(b);
            a
        });
    tally.add(sub);
    tally
}

/// 폴더 바로 아래 항목들의 크기 (큰 것부터). 읽기 전용. `cancel`이 켜지면 중간에 멈춘다.
pub fn analyze(dir: &Path, cancel: &AtomicBool) -> Vec<FolderEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let children: Vec<(std::path::PathBuf, std::fs::Metadata)> = entries
        .flatten()
        .filter_map(|e| Some((e.path(), e.metadata().ok()?)))
        .collect();

    let mut out: Vec<FolderEntry> = children
        .par_iter()
        .filter_map(|(path, m)| {
            #[cfg(windows)]
            let attrs = std::os::windows::fs::MetadataExt::file_attributes(m);
            #[cfg(not(windows))]
            let attrs = 0;
            if walk::is_untouchable(attrs) {
                return None;
            }
            let tally = if m.is_dir() {
                dir_size(path, cancel)
            } else {
                Tally {
                    bytes: m.len(),
                    files: 1,
                    inaccessible: 0,
                }
            };
            Some(FolderEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path: path.display().to_string(),
                is_dir: m.is_dir(),
                bytes: tally.bytes,
                files: tally.files,
                inaccessible: tally.inaccessible,
                approximate: in_windows_dir(path),
            })
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.bytes));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_files_sorts_filters_and_limits() {
        let root = std::env::temp_dir().join(format!("robin-large-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("a").join("b")).unwrap();
        let write = |rel: &str, len: usize| std::fs::write(root.join(rel), vec![0u8; len]).unwrap();
        write("small.txt", 10);
        write(r"a\mid.bin", 3000);
        write(r"a\b\big.iso", 9000);
        write(r"a\b\bigger.mkv", 12000);
        write("edge.zip", 2000);

        let cancel = AtomicBool::new(false);
        let found = largest_files(&root, 2000, 3, &cancel);
        let names: Vec<_> = found.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["bigger.mkv", "big.iso", "mid.bin"]);
        assert!(found.iter().all(|f| f.modified.is_some()));

        // 기준보다 작은 파일은 나오지 않는다
        let all = largest_files(&root, 2000, 100, &cancel);
        assert_eq!(all.len(), 4);
        assert!(all.iter().all(|f| f.bytes >= 2000));

        // 멈추면 아무것도 찾지 않는다
        cancel.store(true, Ordering::Relaxed);
        assert!(largest_files(&root, 1, 100, &cancel).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
