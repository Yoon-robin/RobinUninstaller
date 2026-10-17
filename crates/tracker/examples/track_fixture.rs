//! 테스트 앱 설치를 추적해 기록이 맞는지, 잔여물 스캔이 그 기록을 근거로 쓰는지 검증한다.
//!
//! ```text
//! cargo build -p robin-fixture
//! cargo run -p robin-tracker --example track_fixture
//! ```
//! 테스트 앱을 지웠다가(remove) 설치(install)하고, 끝나면 다시 지운다. 기록은 임시 폴더에 두고 지운다.
//! 실제 PC에서는 다른 프로그램이 같은 시간에 바꾼 것이 섞일 수 있어, 예상 밖 항목은 실패가 아니라 경고로 보여 준다.

#[cfg(windows)]
fn main() {
    use std::process::Command;

    use robin_scanner::{Confidence, Evidence};
    use robin_tracker::{ChangeKind, Trace, TraceStore, diff, snapshot};

    const FIXTURE_ID: &str = "hkcu:robinfixtureapp";

    let exe = std::env::current_exe().expect("현재 실행 파일");
    let fixture = exe
        .parent()
        .and_then(|examples| examples.parent())
        .map(|d| d.join("robin-fixture.exe"))
        .filter(|p| p.exists())
        .expect("먼저 cargo build -p robin-fixture 를 실행하세요");
    let run = |arg: &str| {
        let ok = Command::new(&fixture)
            .arg(arg)
            .status()
            .is_ok_and(|s| s.success());
        assert!(ok, "robin-fixture {arg}");
    };

    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };

    run("remove");
    let before = snapshot::take();
    run("install");
    let after = snapshot::take();
    let draft = diff(&before, &after);
    println!(
        "스냅샷: 폴더 {} · 파일 {} · 키 {} · 값 {}",
        after.folders.len(),
        after.files.len(),
        after.keys.len(),
        after.values.len()
    );
    for c in &draft.changes {
        println!(
            "  + {:?} {}{}",
            c.kind,
            c.location,
            c.value_name
                .as_deref()
                .map(|v| format!(" → {v}"))
                .unwrap_or_default()
        );
    }

    let has = |kind: ChangeKind, suffix: &str, value: Option<&str>| {
        draft.changes.iter().any(|c| {
            c.kind == kind
                && c.location.to_lowercase().ends_with(&suffix.to_lowercase())
                && c.value_name.as_deref() == value
        })
    };
    check("스냅샷을 끝까지 봄", !draft.incomplete);
    check(
        "새 프로그램으로 Fixture Notes를 알아냄",
        draft.new_app_ids == [FIXTURE_ID],
    );
    check(
        r"설치 폴더 (Local\Robin Test Labs)",
        has(ChangeKind::Folder, r"\Local\Robin Test Labs", None),
    );
    check(
        r"설정 폴더 (Roaming\Robin Test Labs)",
        has(ChangeKind::Folder, r"\Roaming\Robin Test Labs", None),
    );
    check(
        r"캐시 폴더 (Local\Fixture Notes)",
        has(ChangeKind::Folder, r"\Local\Fixture Notes", None),
    );
    check(
        "시작 폴더 바로 가기",
        has(ChangeKind::File, r"\Startup\Fixture Notes.lnk", None),
    );
    check(
        r"레지스트리 키 (Software\Robin Test Labs)",
        has(
            ChangeKind::RegistryKey,
            r"HKCU\Software\Robin Test Labs",
            None,
        ),
    );
    check(
        "등록 정보 (Uninstall 키)",
        has(ChangeKind::RegistryKey, r"\Uninstall\RobinFixtureApp", None),
    );
    check(
        "Run 값",
        has(
            ChangeKind::RegistryValue,
            r"\CurrentVersion\Run",
            Some("FixtureNotes"),
        ),
    );
    // 다른 프로그램이 같은 시간에 바꾼 것 (실제 PC에서는 있을 수 있음)
    let markers = ["robin", "fixture"];
    let unexpected: Vec<_> = draft
        .changes
        .iter()
        .filter(|c| {
            let text = format!(
                "{} {}",
                c.location,
                c.value_name.as_deref().unwrap_or_default()
            )
            .to_lowercase();
            !markers.iter().any(|m| text.contains(m))
        })
        .collect();
    if !unexpected.is_empty() {
        println!(
            "⚠ 테스트 앱과 관계없어 보이는 변경 {}개 (경고만)",
            unexpected.len()
        );
    }

    // 기록을 저장했다가 다시 읽어 잔여물 스캔에 넘긴다
    let dir = std::env::temp_dir().join(format!("robin-track-test-{}", std::process::id()));
    let store = TraceStore::new(&dir);
    store
        .save(&Trace {
            id: "0000000000001-f".into(),
            name: "Fixture Notes".into(),
            started_at: draft.started_at,
            finished_at: draft.finished_at,
            app_ids: draft.new_app_ids.clone(),
            changes: draft.changes.clone(),
        })
        .expect("기록 저장");
    let traces = store.for_app(FIXTURE_ID);
    check("기록을 프로그램 ID로 찾음", traces.len() == 1);

    let installed = robin_discovery::list_installed();
    let app = installed
        .iter()
        .find(|a| a.id == FIXTURE_ID)
        .expect("테스트 앱")
        .clone();
    let report = robin_scanner::scan_with_traces(&app, &installed, &traces);
    let tracked: Vec<_> = report
        .items
        .iter()
        .filter(|i| i.evidence.contains(&Evidence::TrackedInstall))
        .collect();
    for i in &tracked {
        println!(
            "  추적 근거 [{:?} {:>3}] {}",
            i.confidence, i.score, i.location
        );
    }
    check(
        "설치 폴더에 추적 근거가 붙고 확실",
        tracked.iter().any(|i| {
            i.location.ends_with(r"\Local\Robin Test Labs") && i.confidence == Confidence::Strong
        }),
    );
    check(
        "Uninstall 키에도 추적 근거",
        tracked
            .iter()
            .any(|i| i.location.ends_with(r"\Uninstall\RobinFixtureApp")),
    );
    check(
        "추적 근거 항목은 모두 테스트 앱 것",
        tracked.iter().all(|i| {
            let text = format!("{} {:?}", i.location, i.value_name).to_lowercase();
            markers.iter().any(|m| text.contains(m))
        }),
    );

    let _ = std::fs::remove_dir_all(&dir);
    run("remove");
    std::process::exit(if ok { 0 } else { 1 });
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}
