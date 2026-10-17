//! 잔여물 탐지 (읽기 전용 — 아무것도 지우지 않는다).
//!
//! 1. 후보 찾기: 설치 폴더, Program Files·AppData 등의 1~2단계, `Software` 레지스트리, Run·App Paths·서비스·MSI 등록,
//!    시작 폴더 바로 가기와 `StartupApproved` 값
//! 2. 근거 붙이기: 설치 폴더 일치, `게시자\제품` 구조, 이름 일치, 다른 프로그램과 공유(감점), 사용자 문서(감점)
//! 3. 보호 규칙: Windows·기본 폴더·공유 폴더·상위 레지스트리 키는 점수와 상관없이 제외
//! 4. 분류: 확실(기본 선택) / 검토 / 낮음(숨김)

mod context;
pub mod model;
pub mod msi_guid;
pub mod names;
pub mod protect;

#[cfg(windows)]
mod files;
#[cfg(windows)]
mod registry;
#[cfg(windows)]
mod startup;
#[cfg(windows)]
mod tracked;

pub use context::ScanContext;
pub use model::{Confidence, Evidence, Leftover, LeftoverKind, ScanReport};

use crate::protect::{is_within, normalize_path};

fn is_fs(kind: LeftoverKind) -> bool {
    matches!(kind, LeftoverKind::Folder | LeftoverKind::File)
}

/// 같은 항목인지: 파일·레지스트리 종류가 같고, 정규화한 위치와 값 이름이 같다.
fn same_item(a: &Leftover, b: &Leftover) -> bool {
    is_fs(a.kind) == is_fs(b.kind)
        && normalize_path(&a.location) == normalize_path(&b.location)
        && a.value_name.as_deref().map(str::to_lowercase)
            == b.value_name.as_deref().map(str::to_lowercase)
}

/// 후보를 정리한다.
/// 1. 같은 항목을 여러 방법으로 찾았으면 하나로 합치고 근거를 모은다.
/// 2. 폴더 안에 든 파일·폴더 후보는 빼되, 그 위험 신호(다른 프로그램 공유, 문서, 사용자 폴더)는 상위 폴더에 올린다.
///    "낮음"으로 숨겨질 폴더는 하위 후보를 삼키지 않는다.
/// 3. 낮음은 숨기고, 확실 → 검토, 점수 순으로 정렬한다.
pub fn finalize(
    app_name: String,
    items: Vec<Leftover>,
    protected_skipped: usize,
    duration_ms: u64,
) -> ScanReport {
    let mut merged: Vec<Leftover> = Vec::new();
    for item in items {
        if let Some(existing) = merged.iter_mut().find(|m| same_item(m, &item)) {
            for e in item.evidence {
                existing.add_evidence(e);
            }
            existing.size_bytes = existing.size_bytes.or(item.size_bytes);
            if existing.note.is_none() {
                existing.note = item.note;
            }
        } else {
            merged.push(item);
        }
    }

    // 폴더마다 하위 후보의 위험 신호를 모은다
    let folder_risks: Vec<(usize, Vec<Evidence>)> = merged
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == LeftoverKind::Folder)
        .map(|(fi, f)| {
            let risks = merged
                .iter()
                .enumerate()
                .filter(|(ci, c)| *ci != fi && is_fs(c.kind) && is_within(&c.location, &f.location))
                .flat_map(|(_, c)| c.risk_evidence().cloned().collect::<Vec<_>>())
                .collect();
            (fi, risks)
        })
        .collect();
    for (fi, risks) in folder_risks {
        for e in risks {
            merged[fi].add_evidence(e);
        }
    }

    let parents: Vec<String> = merged
        .iter()
        .filter(|i| i.kind == LeftoverKind::Folder && i.confidence != Confidence::Low)
        .map(|i| normalize_path(&i.location))
        .collect();
    merged.retain(|i| {
        !is_fs(i.kind)
            || !parents
                .iter()
                .any(|p| *p != normalize_path(&i.location) && is_within(&i.location, p))
    });
    merged.retain(|i| i.confidence != Confidence::Low);
    merged.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then(b.score.cmp(&a.score))
            .then(a.location.cmp(&b.location))
    });
    ScanReport {
        app_name,
        items: merged,
        protected_skipped,
        duration_ms,
    }
}

/// 스캔한다. `installed`는 지금 설치된 전체 목록(공유 여부 판단용), `app`은 대상(이미 제거됐을 수 있음).
/// 이 프로그램과 연결된 설치 추적 기록이 있으면 함께 쓴다.
#[cfg(windows)]
pub fn scan(
    app: &robin_core::InstalledApplication,
    installed: &[robin_core::InstalledApplication],
) -> ScanReport {
    let traces = robin_tracker::TraceStore::default_location()
        .map(|s| s.for_app(&app.id))
        .unwrap_or_default();
    scan_with_traces(app, installed, &traces)
}

/// `scan`과 같지만 설치 추적 기록을 직접 넘긴다 (예제·시험용).
#[cfg(windows)]
pub fn scan_with_traces(
    app: &robin_core::InstalledApplication,
    installed: &[robin_core::InstalledApplication],
    traces: &[robin_tracker::Trace],
) -> ScanReport {
    let started = std::time::Instant::now();
    let ctx = ScanContext::new(app, installed, protect::KnownFolders::from_env());
    let mut items = Vec::new();
    let mut protected = 0;
    files::scan(&ctx, &mut items, &mut protected);
    registry::scan(&ctx, &mut items, &mut protected);
    // 추적 기록의 Run 값도 켜짐·꺼짐 상태 값과 짝지어지도록 시작 항목보다 먼저 본다
    tracked::scan(&ctx, traces, &mut items, &mut protected);
    startup::scan(&ctx, &mut items, &mut protected);
    // 바탕 화면·문서 등 사용자 폴더 안의 항목은 근거가 강해도 자동 선택하지 않는다
    for item in &mut items {
        if is_fs(item.kind) && ctx.known.is_user_content(&item.location) {
            item.add_evidence(Evidence::InUserFolder);
        }
    }
    finalize(
        app.display_name.clone(),
        items,
        protected,
        started.elapsed().as_millis() as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: LeftoverKind, location: &str, evidence: Vec<Evidence>) -> Leftover {
        Leftover::new(kind, location.into(), None, evidence)
    }

    #[test]
    fn merges_same_location_written_differently() {
        let report = finalize(
            "x".into(),
            vec![
                item(
                    LeftoverKind::Folder,
                    "C:/Program Files/Acme",
                    vec![Evidence::InstallLocation],
                ),
                item(
                    LeftoverKind::Folder,
                    r"C:\Program Files\Acme\",
                    vec![Evidence::TrackedInstall],
                ),
            ],
            0,
            0,
        );
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].evidence.len(), 2);
    }

    #[test]
    fn child_risk_is_carried_to_parent() {
        let report = finalize(
            "x".into(),
            vec![
                item(
                    LeftoverKind::Folder,
                    r"C:\Data\Acme",
                    vec![Evidence::PublisherAndProduct],
                ),
                item(
                    LeftoverKind::Folder,
                    r"C:\Data\Acme\Shared",
                    vec![
                        Evidence::ExactName,
                        Evidence::SharedWithOtherApp {
                            app_name: "Other".into(),
                        },
                    ],
                ),
            ],
            0,
            0,
        );
        // 상위 폴더가 공유 감점을 받아 확실이 아니게 되고, 하위 후보는 상위에 합쳐진다
        assert!(
            report
                .items
                .iter()
                .all(|i| i.confidence != Confidence::Strong)
        );
    }

    #[test]
    fn hidden_parent_does_not_swallow_strong_child() {
        let report = finalize(
            "x".into(),
            vec![
                item(
                    LeftoverKind::Folder,
                    r"C:\Data\Vendor",
                    vec![
                        Evidence::PartialName,
                        Evidence::SharedWithOtherApp {
                            app_name: "O".into(),
                        },
                    ],
                ),
                item(
                    LeftoverKind::Folder,
                    r"C:\Data\Vendor\Acme",
                    vec![Evidence::InstallLocation],
                ),
            ],
            0,
            0,
        );
        assert_eq!(report.items.len(), 1);
        assert!(report.items[0].location.ends_with("Acme"));
    }
}
