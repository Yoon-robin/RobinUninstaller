//! 파일·폴더 후보 찾기 (읽기 전용).
//!
//! 프로그램들이 흔히 쓰는 위치의 1~2단계 아래만 본다. C 드라이브 전체를 뒤지지 않는다.

use std::path::{Path, PathBuf};

use crate::context::ScanContext;
use crate::model::{Evidence, Leftover, LeftoverKind};
use crate::names::{NameMatch, match_name, matches_publisher};
use crate::protect::{is_within, protected_path_reason};

const WALK_LIMIT: usize = 50_000;

pub fn scan(ctx: &ScanContext, out: &mut Vec<Leftover>, protected: &mut usize) {
    let mut seen: Vec<String> = Vec::new();

    // 1) 등록된 설치 폴더가 아직 남아 있으면 가장 강한 후보
    if let Some(dir) = &ctx.install_location
        && Path::new(dir).is_dir()
    {
        push(
            ctx,
            out,
            protected,
            &mut seen,
            PathBuf::from(dir),
            vec![Evidence::InstallLocation],
        );
    }

    // 2) 흔한 위치에서 이름으로 찾기
    for root in &ctx.file_roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
            let name = display_stem(&path, is_dir);
            if name.is_empty() {
                continue;
            }

            // 게시자 폴더로 보고 그 안의 제품 폴더를 찾는다. 단, 게시자와 제품 이름이 같으면("Discord") 그 폴더가 곧 제품 폴더다.
            if is_dir
                && let Some(publisher) = &ctx.publisher
                && matches_publisher(&name, publisher)
                && match_name(&name, &ctx.app_name, Some(publisher)) != NameMatch::Exact
            {
                scan_publisher_folder(ctx, out, protected, &mut seen, &path);
                continue;
            }

            if let Some(ev) = name_evidence(&name, ctx) {
                push(ctx, out, protected, &mut seen, path, vec![ev]);
            }
        }
    }
}

/// 게시자 폴더 안에서 제품 폴더를 찾는다. 안에 이 제품 것만 있으면 게시자 폴더 자체도 후보.
fn scan_publisher_folder(
    ctx: &ScanContext,
    out: &mut Vec<Leftover>,
    protected: &mut usize,
    seen: &mut Vec<String>,
    dir: &Path,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut matched = Vec::new();
    let mut others = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
        let name = display_stem(&path, is_dir);
        match match_name(&name, &ctx.app_name, ctx.publisher.as_deref()) {
            m @ (NameMatch::Exact | NameMatch::AllSignificant) => matched.push((path, m)),
            NameMatch::None => others += 1,
        }
    }

    let all_exact = matched.iter().all(|(_, m)| *m == NameMatch::Exact);
    if !matched.is_empty() && others == 0 && all_exact {
        // 게시자 폴더 전체가 이 제품 것뿐 → 폴더째 후보 (하위는 따로 올리지 않음)
        push(
            ctx,
            out,
            protected,
            seen,
            dir.to_path_buf(),
            vec![
                Evidence::PublisherAndProduct,
                Evidence::EmptyPublisherFolder,
            ],
        );
    } else {
        for (path, m) in matched {
            push(ctx, out, protected, seen, path, vec![publisher_evidence(m)]);
        }
    }
}

/// 게시자 폴더 아래에서 이름이 정확히 맞으면 강한 근거, 핵심 단어만 맞으면(다른 버전 등) 검토 수준.
pub(crate) fn publisher_evidence(m: NameMatch) -> Evidence {
    if m == NameMatch::Exact {
        Evidence::PublisherAndProduct
    } else {
        Evidence::PublisherAndPartialName
    }
}

fn name_evidence(name: &str, ctx: &ScanContext) -> Option<Evidence> {
    match match_name(name, &ctx.app_name, ctx.publisher.as_deref()) {
        NameMatch::Exact => Some(Evidence::ExactName),
        NameMatch::AllSignificant => Some(Evidence::PartialName),
        NameMatch::None => None,
    }
}

/// 시작 메뉴 바로 가기는 `.lnk`를 떼고 비교한다.
fn display_stem(path: &Path, is_dir: bool) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !is_dir && name.to_lowercase().ends_with(".lnk") {
        name[..name.len() - 4].to_string()
    } else if is_dir || name.to_lowercase().ends_with(".url") {
        name
    } else {
        // 루트 바로 아래의 일반 파일은 후보로 보지 않는다 (설정 파일 하나 등은 이름만으로 판단하기 어려움)
        String::new()
    }
}

fn push(
    ctx: &ScanContext,
    out: &mut Vec<Leftover>,
    protected: &mut usize,
    seen: &mut Vec<String>,
    path: PathBuf,
    mut evidence: Vec<Evidence>,
) {
    let location = path.to_string_lossy().into_owned();
    let key = location.to_lowercase();
    if seen.iter().any(|s| is_within(&key, s)) {
        return; // 이미 상위 폴더가 후보
    }
    if protected_path_reason(&location, &ctx.known).is_some() || ctx.is_self(&location) {
        *protected += 1;
        return;
    }

    if let Some(other) = ctx.other_app_sharing_path(&location) {
        evidence.push(Evidence::SharedWithOtherApp { app_name: other });
    }

    let is_dir = path.is_dir();
    let (size, docs, complete) = if is_dir {
        measure(&path)
    } else {
        (
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
            usize::from(is_document(&path)),
            true,
        )
    };
    if docs > 0 {
        evidence.push(Evidence::ContainsUserDocuments { count: docs });
    }
    if !complete {
        evidence.push(Evidence::TooLargeToCheck);
    }

    seen.push(key);
    let mut item = Leftover::new(
        if is_dir {
            LeftoverKind::Folder
        } else {
            LeftoverKind::File
        },
        location,
        None,
        evidence,
    );
    item.size_bytes = Some(size);
    out.push(item);
}

/// 사용자가 만든 문서로 보이는지 (확장자 기준)
pub(crate) fn is_document(path: &Path) -> bool {
    path.file_name().is_some_and(crate::walk::is_document_name)
}

/// 폴더 크기와 문서 파일 수, 끝까지 셌는지 (최대 WALK_LIMIT개까지만 센다). 여러 스레드로 읽는다.
pub(crate) fn measure(dir: &Path) -> (u64, usize, bool) {
    let t = crate::walk::tally(dir, WALK_LIMIT);
    (t.size, t.docs, t.complete)
}
