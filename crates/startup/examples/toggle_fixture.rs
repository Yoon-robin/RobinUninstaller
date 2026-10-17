//! 테스트 앱 "Fixture Notes"의 시작 항목만 껐다가 다시 켜서 원래 상태로 돌아오는지 확인한다.
//! 다른 시작 항목은 건드리지 않는다.
//!
//! ```text
//! cargo run -p robin-fixture -- install
//! cargo run -p robin-startup --example toggle_fixture
//! cargo run -p robin-fixture -- remove
//! ```

#[cfg(windows)]
fn main() {
    use robin_startup::{History, StartupSource, ToggleError};

    const FIXTURE_ITEMS: [(&str, StartupSource); 3] = [
        ("FixtureNotes", StartupSource::RunUser),
        ("Fixture Notes", StartupSource::FolderUser),
        ("Fixture Notes Updater", StartupSource::Task),
    ];

    // 실제 기록과 섞이지 않게 임시 기록 폴더를 쓴다
    let dir = std::env::temp_dir().join(format!("robin-startup-fixture-{}", std::process::id()));
    let history = History::new(&dir);
    let find = |name: &str, source: StartupSource| {
        robin_startup::list(&history)
            .into_iter()
            .find(|e| e.name == name && e.source == source)
    };

    for (name, source) in FIXTURE_ITEMS {
        let entry = find(name, source)
            .unwrap_or_else(|| panic!("{name} 없음. 먼저 robin-fixture install 을 실행하세요"));
        assert!(entry.enabled, "{name}: 처음엔 켜져 있어야 함");
        assert!(
            entry
                .target_path
                .as_deref()
                .is_some_and(|p| p.ends_with(r"Fixture Notes\notes.exe")),
            "{name}: 대상 해석 {:?}",
            entry.target_path
        );
        println!(
            "확인: {name} 켜짐, 대상 {}",
            entry.target_path.as_deref().unwrap()
        );

        assert!(
            !entry.needs_admin,
            "{name}: 관리자 권한 없이 바꿀 수 있어야 함"
        );
        if source == StartupSource::Task {
            assert_eq!(entry.approved_name, r"\Fixture Notes Updater");
            assert_eq!(entry.author.as_deref(), Some("Robin Test Labs"));
        }

        let off = robin_startup::set_enabled(&history, &entry.id, false).expect("끄기");
        assert!(!off.enabled && off.changed_by_us && off.disabled_at.is_some());
        let change = history.get(&entry.id).expect("원래 값 기록");
        if source == StartupSource::Task {
            assert_eq!(change.previous_enabled, Some(true));
        }
        // 다시 읽어도 꺼져 있어야 함 (작업 스케줄러·레지스트리에 실제로 반영됐는지)
        assert!(find(name, source).is_some_and(|e| !e.enabled));
        println!("  끔 (끈 시각 기록됨, 원래 값 기록됨)");

        let on = robin_startup::set_enabled(&history, &entry.id, true).expect("켜기");
        assert!(on.enabled && !on.changed_by_us);
        assert!(find(name, source).is_some_and(|e| e.enabled));
        assert!(history.get(&entry.id).is_none(), "기록이 남으면 안 됨");
        println!("  다시 켬 → 원래 상태");
    }

    // 관리자 권한 항목 (install --machine 으로 만든 경우에만)
    if let Some(entry) = find("FixtureNotesMachine", StartupSource::RunMachine) {
        assert!(entry.target_missing, "없는 파일을 가리켜야 함");
        match robin_startup::set_enabled(&history, &entry.id, false) {
            Err(ToggleError::NeedsAdmin) => println!("확인: HKLM 항목은 관리자 권한 필요"),
            Ok(_) => {
                // 관리자로 실행한 경우: 바로 되돌린다
                robin_startup::set_enabled(&history, &entry.id, true).expect("되돌리기");
                println!("확인: HKLM 항목 끄기·켜기 (관리자)");
            }
            Err(e) => panic!("{e}"),
        }
    }

    // 관리자 예약 작업 (install --machine 으로 만든 경우에만): SYSTEM으로 실행되는 부팅 작업
    if let Some(entry) = find("Fixture Notes Machine Task", StartupSource::Task) {
        assert!(
            entry.needs_admin,
            "다른 계정 작업은 관리자 권한 필요로 보여야 함"
        );
        match robin_startup::set_enabled(&history, &entry.id, false) {
            Err(ToggleError::NeedsAdmin) => println!("확인: SYSTEM 예약 작업은 관리자 권한 필요"),
            Ok(_) => {
                robin_startup::set_enabled(&history, &entry.id, true).expect("되돌리기");
                println!("확인: SYSTEM 예약 작업 끄기·켜기 (관리자)");
            }
            Err(e) => panic!("{e}"),
        }
    }

    let _ = std::fs::remove_dir_all(dir);
    println!("통과");
}

#[cfg(not(windows))]
fn main() {}
