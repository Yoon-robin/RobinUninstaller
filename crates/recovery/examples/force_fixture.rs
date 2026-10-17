//! 언인스톨러가 없는 테스트 앱(Fixture Notes)을 강제 제거 → 목록에서 사라짐 → 복원 → 다시 보임까지 검증한다.
//! 순서: `robin-fixture install --broken` → 이 예제 → `robin-fixture remove`
//! 백업은 임시 폴더에 두고 끝나면 지운다. 테스트 앱 흔적 외에는 건드리지 않는다.

use robin_core::UninstallerStatus;
use robin_recovery::{Guard, ItemStatus, Store};
use robin_scanner::{Confidence, Leftover, LeftoverKind};

const FIXTURE_ID: &str = "hkcu:robinfixtureapp";

fn find_fixture() -> Option<robin_core::InstalledApplication> {
    robin_discovery::list_installed()
        .into_iter()
        .find(|a| a.id == FIXTURE_ID)
}

fn main() {
    let app = find_fixture().expect("테스트 앱을 먼저 등록하세요 (robin-fixture install --broken)");
    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };

    check(
        "언인스톨러 없음으로 보임",
        app.uninstaller == UninstallerStatus::Missing,
    );
    let gate = robin_execution::force::check(&app);
    check("강제 제거 대상에서 빠지지 않음", gate.blocked.is_none());
    check("실행 중인 프로세스 없음", gate.running.is_empty());

    let report = robin_scanner::scan(&app, &robin_discovery::list_installed());
    let strong: Vec<Leftover> = report
        .items
        .iter()
        .filter(|i| i.confidence == Confidence::Strong)
        .cloned()
        .collect();
    for i in &strong {
        println!("  확실 {:?} {}", i.kind, i.location);
    }
    check(
        "등록 정보(Uninstall 키)가 확실 목록에 있음",
        strong.iter().any(|i| {
            i.kind == LeftoverKind::RegistryKey
                && i.location.ends_with(r"\Uninstall\RobinFixtureApp")
        }),
    );
    check(
        "설치 폴더가 확실 목록에 있음",
        strong.iter().any(|i| {
            i.kind == LeftoverKind::Folder && i.location.ends_with(r"Robin Test Labs\Fixture Notes")
        }),
    );
    check(
        "테스트 앱 흔적만 확실 목록에 있음",
        strong.iter().all(|i| {
            i.location.contains("Robin Test Labs")
                || i.location.contains("RobinFixtureApp")
                || i.location.contains("Fixture Notes")
                || matches!(i.value_name.as_deref(), Some(v) if v.contains("Fixture"))
        }),
    );

    let store = Store::new(std::env::temp_dir().join("robin-force-test"));
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    };
    let mut op = robin_recovery::new_operation(&app.display_name, &strong);
    robin_recovery::clean(&store, &mut op, &guard, |i| {
        println!(
            "  정리 {:?} {} {}",
            i.status,
            i.location,
            i.error.clone().unwrap_or_default()
        )
    });
    check(
        "확실 항목 모두 정리됨",
        op.items.iter().all(|i| i.status == ItemStatus::Removed),
    );
    check("프로그램 목록에서 사라짐", find_fixture().is_none());

    let mut reloaded = store.load(&op.id).expect("매니페스트");
    robin_recovery::restore(&store, &mut reloaded, &guard, None, |_| {});
    check(
        "모두 복원됨",
        reloaded
            .items
            .iter()
            .all(|i| i.status == ItemStatus::Restored),
    );
    check(
        "복원 뒤 다시 목록에 보임",
        find_fixture().is_some_and(|a| a.uninstaller == UninstallerStatus::Missing),
    );

    store.delete(&op.id).expect("테스트 백업 정리");
    println!("정리: cargo run -p robin-fixture -- remove");
    std::process::exit(if ok { 0 } else { 1 });
}
