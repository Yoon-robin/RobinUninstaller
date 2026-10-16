//! 테스트 앱(Fixture Notes)의 시작 항목을 끄고 다시 켜서, 원래 명령이 그대로 남는지 확인한다.
//! 기대한 결과가 아니면 실패로 끝난다.
//!
//! 순서: `robin-fixture install` → 이 예제 실행 → `robin-fixture remove`
//! 이 예제는 `HKCU\...\Run`의 `FixtureNotes` 값과 그 `StartupApproved` 표시만 건드린다.

use robin_startup::{StartupItem, StartupSource};

const NAME: &str = "FixtureNotes";

fn find() -> StartupItem {
    robin_startup::list(&robin_discovery::list_installed())
        .into_iter()
        .find(|i| i.source == StartupSource::RunUser && i.name == NAME)
        .expect("테스트 앱을 먼저 등록하세요 (robin-fixture install)")
}

fn main() {
    let before = find();
    println!("{before:#?}");
    let mut failures = Vec::new();
    let mut check = |ok: bool, what: &str| {
        println!("  {} {what}", if ok { "OK  " } else { "FAIL" });
        if !ok {
            failures.push(what.to_string());
        }
    };

    check(before.enabled, "처음에는 켜져 있음");
    check(!before.needs_admin, "사용자 항목은 관리자 권한이 필요 없음");
    check(
        before.app_name.as_deref() == Some("Fixture Notes"),
        "설치 폴더로 Fixture Notes 것임을 알아냄",
    );

    robin_startup::set_enabled(&before.id, false).expect("끄기");
    let off = find();
    check(!off.enabled, "끄면 꺼짐으로 보임");
    check(off.command == before.command, "끈 뒤에도 Run 값은 그대로");

    robin_startup::set_enabled(&before.id, true).expect("켜기");
    let on = find();
    check(on.enabled, "다시 켜면 켜짐으로 보임");
    check(
        on.command == before.command,
        "다시 켠 뒤에도 Run 값은 그대로",
    );

    check(
        matches!(
            robin_startup::set_enabled("run-user:00", true),
            Err(robin_startup::SetError::NotFound)
        ),
        "없는 ID는 거부",
    );
    check(
        robin_startup::set_enabled(r"run-user:..\x", true).is_err(),
        "잘못된 ID는 거부",
    );

    if failures.is_empty() {
        println!("모두 통과. 정리: cargo run -p robin-fixture -- remove");
    } else {
        eprintln!("실패 {}건: {failures:?}", failures.len());
        std::process::exit(1);
    }
}
