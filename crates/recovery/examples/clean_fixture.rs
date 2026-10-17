//! 테스트 앱(Fixture Notes)의 흔적으로 정리 → 확인 → 복원 → 원본과 같은지까지 검증한다.
//! 순서: `robin-fixture install` → 이 예제 → `robin-fixture remove`
//! 테스트 앱의 시작 항목은 미리 꺼 둔다 (`StartupApproved` 값도 정리·복원되는지 보려고).
//! 백업은 임시 폴더에 둔다. 테스트 앱 흔적과, 일부러 넣은 위험 항목(차단돼야 함) 외에는 건드리지 않는다.

use std::path::Path;

use robin_recovery::{Guard, ItemStatus, Store, registry};
use robin_scanner::{Confidence, Leftover, LeftoverKind};

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
        robin_startup::set_enabled(&history, &entry.id, false).expect("테스트 앱 시작 항목 끄기");
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// 비교용 스냅샷: 레지스트리는 백업 덤프, 파일은 (상대 경로, 내용) 목록.
fn snapshot(item: &Leftover) -> String {
    match item.kind {
        LeftoverKind::RegistryKey => format!(
            "{:?}",
            registry::backup_key(&item.location)
                .ok()
                .map(|b| serde_json::to_string(&b).unwrap())
        ),
        LeftoverKind::RegistryValue => format!(
            "{:?}",
            registry::backup_value(
                &item.location,
                item.value_name.as_deref().unwrap_or_default()
            )
            .ok()
            .map(|b| serde_json::to_string(&b).unwrap())
        ),
        _ => {
            let mut files = Vec::new();
            walk(
                Path::new(&item.location),
                Path::new(&item.location),
                &mut files,
            );
            files.sort();
            format!("{files:?}")
        }
    }
}

fn walk(root: &Path, p: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    if p.is_dir() {
        out.push((
            p.strip_prefix(root).unwrap().display().to_string(),
            Vec::new(),
        ));
        for e in std::fs::read_dir(p).unwrap().flatten() {
            walk(root, &e.path(), out);
        }
    } else if p.exists() {
        out.push((
            p.strip_prefix(root).unwrap().display().to_string(),
            std::fs::read(p).unwrap(),
        ));
    }
}

fn exists(item: &Leftover) -> bool {
    match item.kind {
        LeftoverKind::RegistryKey => registry::backup_key(&item.location).is_ok(),
        LeftoverKind::RegistryValue => {
            registry::backup_value(&item.location, item.value_name.as_deref().unwrap()).is_ok()
        }
        _ => Path::new(&item.location).exists(),
    }
}

fn fake(kind: LeftoverKind, location: &str) -> Leftover {
    Leftover::new(kind, location.to_string(), None, vec![])
}

fn main() {
    let app = robin_discovery::list_installed()
        .into_iter()
        .find(|a| a.id == FIXTURE_ID)
        .expect("테스트 앱을 먼저 등록하세요");
    disable_fixture_startup();
    let outcome = robin_execution::run_uninstaller(
        &app,
        &Default::default(),
        |_| {},
        || {
            robin_discovery::list_installed()
                .iter()
                .any(|a| a.id == FIXTURE_ID)
        },
    );
    println!("제거: {outcome:?}");

    let report = robin_scanner::scan(&app, &robin_discovery::list_installed());
    let strong: Vec<Leftover> = report
        .items
        .iter()
        .filter(|i| i.confidence == Confidence::Strong)
        .cloned()
        .collect();
    assert!(
        strong.iter().all(|i| i.location.contains("Robin Test Labs")
            || i.location.ends_with(r"\Fixture Notes.lnk")
            || matches!(
                i.value_name.as_deref(),
                Some("FixtureNotes" | "Fixture Notes.lnk")
            )),
        "테스트 앱 흔적만 대상이어야 함"
    );
    for needle in [
        r"\Fixture Notes.lnk",
        r"StartupApproved\Run",
        r"StartupApproved\StartupFolder",
    ] {
        assert!(
            strong.iter().any(|i| i.location.ends_with(needle)),
            "{needle} 항목이 확실 목록에 있어야 함"
        );
    }
    let before: Vec<String> = strong.iter().map(snapshot).collect();

    let mut targets = strong.clone();
    targets.push(fake(LeftoverKind::Folder, r"C:\Windows\System32"));
    targets.push(fake(LeftoverKind::RegistryKey, r"HKLM\SOFTWARE\Microsoft"));

    let store = Store::new(std::env::temp_dir().join("robin-recovery-test"));
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    };
    let mut op = robin_recovery::new_operation(&app.display_name, &targets);
    robin_recovery::clean(&store, &mut op, &guard, |i| {
        println!(
            "  정리 {:?} {} {}",
            i.status,
            i.location,
            i.error.clone().unwrap_or_default()
        )
    });

    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };
    check(
        "확실 항목 모두 Removed",
        op.items
            .iter()
            .take(strong.len())
            .all(|i| i.status == ItemStatus::Removed),
    );
    check(
        "정리 후 원래 자리에 없음",
        strong.iter().all(|i| !exists(i)),
    );
    check(
        "System32 차단",
        op.items[strong.len()].status == ItemStatus::Blocked,
    );
    check(
        r"HKLM\SOFTWARE\Microsoft 차단",
        op.items[strong.len() + 1].status == ItemStatus::Blocked,
    );
    check(
        "System32 그대로",
        Path::new(r"C:\Windows\System32\kernel32.dll").exists(),
    );
    check("매니페스트 저장됨", store.load(&op.id).is_ok());

    let mut reloaded = store.load(&op.id).unwrap();
    robin_recovery::restore(&store, &mut reloaded, &guard, None, |i| {
        println!(
            "  복원 {:?} {} {}",
            i.status,
            i.location,
            i.error.clone().unwrap_or_default()
        )
    });
    let after: Vec<String> = strong.iter().map(snapshot).collect();
    check(
        "복원 후 모두 Restored",
        reloaded
            .items
            .iter()
            .take(strong.len())
            .all(|i| i.status == ItemStatus::Restored),
    );
    check("복원 결과가 원본과 바이트 단위로 같음", before == after);

    // 복원한 것을 다시 복원하려 하면 아무 일도 없어야 함 (상태가 Restored)
    let mut again = store.load(&op.id).unwrap();
    robin_recovery::restore(&store, &mut again, &guard, None, |_| {});
    check(
        "두 번 복원해도 안전",
        again
            .items
            .iter()
            .take(strong.len())
            .all(|i| i.status == ItemStatus::Restored),
    );

    store.delete(&op.id).unwrap();
    std::process::exit(if ok { 0 } else { 1 });
}
