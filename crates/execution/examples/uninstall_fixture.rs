//! 테스트 앱(robin-fixture)만 대상으로 제거 흐름을 끝까지 실행해 본다.
//! 먼저 `cargo run -p robin-fixture -- install`로 등록해 둬야 한다. 다른 프로그램은 절대 건드리지 않는다.

const FIXTURE_ID: &str = "hkcu:robinfixtureapp";

fn main() {
    let Some(app) = robin_discovery::list_installed()
        .into_iter()
        .find(|a| a.id == FIXTURE_ID)
    else {
        eprintln!("테스트 앱이 등록돼 있지 않아요");
        std::process::exit(1);
    };

    let started = std::time::Instant::now();
    let outcome = robin_execution::run_uninstaller(
        &app,
        |event| println!("[{:>5}ms] {event:?}", started.elapsed().as_millis()),
        || {
            robin_discovery::list_installed()
                .iter()
                .any(|a| a.id == FIXTURE_ID)
        },
    );
    println!("[{:>5}ms] 결과: {outcome:?}", started.elapsed().as_millis());
}
