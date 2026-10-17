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

/// 고른 항목을 정리한다. 되돌릴 수 없는 항목은 `allow_irreversible`이 켜져 있을 때만 지운다.
#[cfg(windows)]
pub fn clean(ids: &[String], allow_irreversible: bool) -> Vec<CleanResult> {
    let running = robin_execution::running_exe_names();
    let elevated = system::is_elevated();

    ids.iter()
        .map(|id| {
            let Some(rule) = rules::find(id) else {
                return CleanResult::skipped(id, "알 수 없는 항목이에요");
            };
            if rule.kind == Kind::Info {
                return CleanResult::skipped(id, "직접 정리하는 항목이에요");
            }
            if rule.risk == Risk::Irreversible && !allow_irreversible {
                return CleanResult::skipped(id, "되돌릴 수 없는 항목은 따로 확인해야 지워요");
            }
            let blockers = running_blockers(rule, &running);
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
            for path in walk::rule_paths(rule, &env) {
                // 지우는 파일의 실제 위치가 이 폴더 안인지 핸들로 확인한다 (정션 바꿔치기 방지)
                let Ok(root) = std::fs::canonicalize(&path) else {
                    continue;
                };
                walk::visit(
                    &path,
                    min_age(rule),
                    &mut |file, len| match system::delete_file_within(file, &root) {
                        Ok(()) => {
                            result.freed_bytes += len;
                            result.deleted_files += 1;
                        }
                        // 실행 중인 프로그램이 쥐고 있는 파일 등은 건너뛴다
                        Err(_) => result.failed_files += 1,
                    },
                    // 비게 된 하위 폴더만 지운다 (비어 있지 않으면 실패하고 그대로 남는다)
                    &mut |dir| {
                        let _ = std::fs::remove_dir(dir);
                    },
                );
            }
            result
        })
        .collect()
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
            })
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.bytes));
    out
}
