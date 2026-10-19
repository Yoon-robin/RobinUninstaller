//! 테스트 앱(Fixture Notes)을 지운 뒤 남은 흔적을 찾는 전 과정을 검증한다. 기대한 결과가 아니면 실패로 끝난다.
//!
//! 순서: `robin-fixture install` → 이 예제 실행(언인스톨러 실행 → 스캔 → 확인) → `robin-fixture remove`
//! 이 예제는 아무것도 지우지 않는다 (언인스톨러가 지우는 것 제외).
//! 테스트 앱의 시작 항목은 미리 꺼 둔다 (`StartupApproved` 값이 생겨야 그 값도 찾는지 볼 수 있다).

#[cfg(windows)]
mod imp {
    use robin_scanner::Confidence;

    const FIXTURE_ID: &str = "hkcu:robinfixtureapp";

    /// 테스트 앱의 시작 항목 두 개(Run 값, 시작 폴더 바로 가기)를 꺼서 `StartupApproved` 값을 만든다.
    /// 기록은 임시 폴더에 두고 바로 지운다. 다른 시작 항목은 건드리지 않는다.
    fn disable_fixture_startup() {
        use robin_startup::{History, StartupSource};
        let dir = std::env::temp_dir().join(format!("robin-scan-startup-{}", std::process::id()));
        let history = History::new(&dir);
        for (name, source) in [
            ("FixtureNotes", StartupSource::RunUser),
            ("Fixture Notes", StartupSource::FolderUser),
        ] {
            let entry = robin_startup::list(&history)
                .into_iter()
                .find(|e| e.name == name && e.source == source)
                .unwrap_or_else(|| panic!("시작 항목 {name} 없음"));
            robin_startup::set_enabled(&history, &entry.id, false)
                .expect("테스트 앱 시작 항목 끄기");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    pub fn main() {
        let installed = robin_discovery::list_installed();
        let app = installed
            .iter()
            .find(|a| a.id == FIXTURE_ID)
            .cloned()
            .expect("테스트 앱을 먼저 등록하세요");
        disable_fixture_startup();

        let outcome = robin_execution::run_uninstaller(
            &app,
            &std::sync::atomic::AtomicBool::new(false),
            |_| {},
            || {
                robin_discovery::list_installed()
                    .iter()
                    .any(|a| a.id == FIXTURE_ID)
            },
        );
        println!("제거 결과: {outcome:?}");

        let report = robin_scanner::scan(&app, &robin_discovery::list_installed());
        for item in &report.items {
            println!(
                "  [{:?} {:>3}] {}{}",
                item.confidence,
                item.score,
                item.location,
                item.value_name
                    .as_deref()
                    .map(|v| format!(" → {v}"))
                    .unwrap_or_default()
            );
        }

        let strong = |needle: &str| {
            report.items.iter().any(|i| {
                i.confidence == Confidence::Strong
                    && (i.location.contains(needle) || i.value_name.as_deref() == Some(needle))
            })
        };
        let listed = |needle: &str| report.items.iter().any(|i| i.location.contains(needle));
        let review = |needle: &str| {
            report
                .items
                .iter()
                .any(|i| i.confidence == Confidence::Review && i.location.contains(needle))
        };
        let strong_contains = |needle: &str| {
            report
                .items
                .iter()
                .any(|i| i.confidence == Confidence::Strong && i.location.contains(needle))
        };

        let strong_value = |key_end: &str, value: &str| {
            report.items.iter().any(|i| {
                i.confidence == Confidence::Strong
                    && i.location.ends_with(key_end)
                    && i.value_name.as_deref() == Some(value)
            })
        };

        let checks = [
            // 이름만 맞는 항목은 같은 게시자의 다른 제품일 수 있어 미리 선택하지 않는다 (목록에는 나온다)
            (
                "설정 폴더 (게시자/제품) = 목록에 있음, 확실 아님",
                review(r"Roaming\Robin Test Labs\Fixture Notes"),
            ),
            (
                "레지스트리 키 = 목록에 있음, 확실 아님",
                review(r"HKCU\SOFTWARE\Robin Test Labs\Fixture Notes"),
            ),
            ("Run 값 = 확실", strong("FixtureNotes")),
            (
                "시작 폴더 바로 가기 = 확실",
                report.items.iter().any(|i| {
                    i.confidence == Confidence::Strong
                        && i.location.ends_with(r"Programs\Startup\Fixture Notes.lnk")
                }),
            ),
            (
                "Run의 StartupApproved 값 = 확실",
                strong_value(r"StartupApproved\Run", "FixtureNotes"),
            ),
            (
                "바로 가기의 StartupApproved 값 = 확실",
                strong_value(r"StartupApproved\StartupFolder", "Fixture Notes.lnk"),
            ),
            (
                "캐시 폴더(이름 일치) = 목록에 있음",
                listed(r"AppData\Local\Fixture Notes"),
            ),
            ("미끼 Other Product = 없음", !listed("Other Product")),
            ("미끼 Fixture Viewer = 없음", !listed("Fixture Viewer")),
            (
                "다른 버전 폴더 = 확실 아님",
                !strong_contains("Fixture Notes 2019"),
            ),
            (
                "다른 제품이 있는 게시자 폴더 = 통째로 올리지 않음",
                !report
                    .items
                    .iter()
                    .any(|i| i.location.ends_with(r"Roaming\Robin Test Labs")),
            ),
        ];

        let mut ok = true;
        for (name, pass) in checks {
            println!("{} {name}", if pass { "✔" } else { "✘" });
            ok &= pass;
        }
        std::process::exit(if ok { 0 } else { 1 });
    }
}

#[cfg(windows)]
fn main() {
    imp::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}
