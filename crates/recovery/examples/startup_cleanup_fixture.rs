//! 실행 파일이 없는 테스트 시작 항목(`RobinGhostTool` Run 값, `Robin Ghost Tool.lnk`)을 백업·정리 → 사라짐 확인 →
//! 복원 → 원래대로(꺼짐 상태 포함) 돌아오는지 검증한다. 실행 파일이 있는 항목은 정리를 거부해야 한다.
//! 순서: `robin-fixture install` → 이 예제 → `robin-fixture remove`
//! 기록·백업은 임시 폴더에 두고 끝나면 지운다.

#[cfg(windows)]
fn main() {
    use robin_recovery::{Guard, ItemStatus, Store};
    use robin_startup::{CleanupRefusal, History, StartupEntry, StartupSource};

    let dir = std::env::temp_dir().join(format!("robin-startup-cleanup-{}", std::process::id()));
    let history = History::new(dir.join("history"));
    let find = |name: &str, source: StartupSource| -> Option<StartupEntry> {
        robin_startup::list(&history)
            .into_iter()
            .find(|e| e.name == name && e.source == source)
    };

    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };

    let run = find("RobinGhostTool", StartupSource::RunUser)
        .expect("테스트 앱을 먼저 등록하세요 (robin-fixture install)");
    let link = find("Robin Ghost Tool", StartupSource::FolderUser).expect("바로 가기 시작 항목");
    check("Run 값이 파일 없음으로 보임", run.target_missing);
    check("바로 가기가 파일 없음으로 보임", link.target_missing);

    // 꺼진 상태 값(StartupApproved)도 함께 정리·복원되는지 보려고 Run 값은 먼저 끈다
    robin_startup::set_enabled(&history, &run.id, false).expect("끄기");
    let run = find("RobinGhostTool", StartupSource::RunUser).unwrap();

    let present = find("FixtureNotes", StartupSource::RunUser).expect("FixtureNotes");
    check(
        "실행 파일이 있는 항목은 정리 거부",
        matches!(
            robin_recovery::startup::leftovers_for(&present),
            Err(CleanupRefusal::TargetPresent)
        ),
    );

    let mut items = robin_recovery::startup::leftovers_for(&run).expect("Run 값 정리 대상");
    check("Run 값 + 꺼짐 상태 값 2개", items.len() == 2);
    let link_items = robin_recovery::startup::leftovers_for(&link).expect("바로 가기 정리 대상");
    check("바로 가기 파일 1개", link_items.len() == 1);
    items.extend(link_items);
    for i in &items {
        println!(
            "  대상 {:?} {}{}",
            i.kind,
            i.location,
            i.value_name
                .as_deref()
                .map(|v| format!(" → {v}"))
                .unwrap_or_default()
        );
    }

    let store = Store::new(dir.join("backups"));
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    };
    let mut op = robin_recovery::new_operation(robin_recovery::startup::OPERATION_NAME, &items);
    robin_recovery::clean(&store, &mut op, &guard, |i| {
        println!(
            "  정리 {:?} {} {}",
            i.status,
            i.location,
            i.error.clone().unwrap_or_default()
        )
    })
    .expect("작업 기록 저장");
    check(
        "모두 정리됨",
        op.items.iter().all(|i| i.status == ItemStatus::Removed),
    );
    check(
        "목록에서 둘 다 사라짐",
        find("RobinGhostTool", StartupSource::RunUser).is_none()
            && find("Robin Ghost Tool", StartupSource::FolderUser).is_none(),
    );
    check(
        "꺼짐 상태 값도 사라짐",
        !robin_startup::has_approved(StartupSource::RunUser, "RobinGhostTool"),
    );
    check(
        "다른 시작 항목은 그대로",
        find("FixtureNotes", StartupSource::RunUser).is_some(),
    );

    let mut reloaded = store.load(&op.id).expect("매니페스트");
    robin_recovery::restore(&store, &mut reloaded, &guard, None, |_| {}).expect("작업 기록 저장");
    check(
        "모두 복원됨",
        reloaded
            .items
            .iter()
            .all(|i| i.status == ItemStatus::Restored),
    );
    let run_back = find("RobinGhostTool", StartupSource::RunUser);
    check(
        "Run 값이 꺼진 상태로 돌아옴",
        run_back
            .as_ref()
            .is_some_and(|e| !e.enabled && e.command == run.command),
    );
    check(
        "바로 가기가 돌아옴",
        find("Robin Ghost Tool", StartupSource::FolderUser).is_some(),
    );

    let _ = std::fs::remove_dir_all(&dir);
    println!("정리: cargo run -p robin-fixture -- remove");
    std::process::exit(if ok { 0 } else { 1 });
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}
