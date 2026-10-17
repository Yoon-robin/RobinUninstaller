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

use crate::protect::is_within;

/// 후보를 정리한다: 다른 후보 폴더 안에 든 항목은 빼고, 낮음은 숨기고, 확실 → 검토, 점수 순으로.
pub fn finalize(
    app_name: String,
    mut items: Vec<Leftover>,
    protected_skipped: usize,
    duration_ms: u64,
) -> ScanReport {
    let folders: Vec<String> = items
        .iter()
        .filter(|i| i.kind == LeftoverKind::Folder)
        .map(|i| i.location.clone())
        .collect();
    items.retain(|i| {
        !folders.iter().any(|f| {
            !f.eq_ignore_ascii_case(&i.location)
                && is_within(&i.location, f)
                && i.kind != LeftoverKind::RegistryKey
                && i.kind != LeftoverKind::RegistryValue
        })
    });
    items.sort_by_key(|i| i.id.to_lowercase());
    items.dedup_by(|a, b| a.id.eq_ignore_ascii_case(&b.id));
    items.retain(|i| i.confidence != Confidence::Low);
    items.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then(b.score.cmp(&a.score))
            .then(a.location.cmp(&b.location))
    });
    ScanReport {
        app_name,
        items,
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
    startup::scan(&ctx, &mut items, &mut protected);
    tracked::scan(&ctx, traces, &mut items, &mut protected);
    finalize(
        app.display_name.clone(),
        items,
        protected,
        started.elapsed().as_millis() as u64,
    )
}
