//! 설치 폴더 크기 재기를 테스트 앱으로 검증한다 (읽기 전용). 기대한 결과가 아니면 실패로 끝난다.
//!
//! 순서: `robin-fixture install` → 이 예제 → `robin-fixture remove`
//! 테스트 앱은 크기를 기록하므로, 메모리에서만 기록을 지운 복사본으로 잰다 (레지스트리는 그대로).
//! 실제 프로그램 목록에서 잴 대상과 크기도 출력만 한다.

#[cfg(windows)]
fn main() {
    use robin_scanner::install_size::{install_sizes, size_targets};
    use robin_scanner::protect::KnownFolders;

    const FIXTURE_ID: &str = "hkcu:robinfixtureapp";
    let known = KnownFolders::from_env();
    let mut apps = robin_discovery::list_installed();

    let fixture = apps
        .iter_mut()
        .find(|a| a.id == FIXTURE_ID)
        .expect("테스트 앱이 설치돼 있어야 한다 (robin-fixture install)");
    assert!(
        fixture.estimated_size_kb.is_some(),
        "테스트 앱은 크기를 기록한다"
    );
    let sizes = install_sizes(std::slice::from_ref(fixture), &known);
    assert!(sizes.is_empty(), "크기 기록이 있으면 재지 않는다");

    fixture.estimated_size_kb = None;
    let location = fixture.install_location.clone().expect("설치 폴더");
    let sizes = install_sizes(std::slice::from_ref(fixture), &known);
    let kb = *sizes.get(FIXTURE_ID).expect("테스트 앱 폴더를 재야 한다");
    let bytes = robin_scanner::install_size::measure_dir(std::path::Path::new(&location)).unwrap();
    assert_eq!(kb, bytes.div_ceil(1024));
    println!("테스트 앱: {location} → {kb} KB");

    // 같은 폴더를 쓰는 다른 항목이 있으면 재지 않는다
    let mut twin = fixture.clone();
    twin.id = "hkcu:robinfixturetwin".into();
    let pair = [fixture.clone(), twin];
    assert!(
        install_sizes(&pair, &known).is_empty(),
        "같이 쓰는 폴더는 재지 않는다"
    );

    // 실제 목록 (출력만)
    let targets = size_targets(&apps, &known);
    let real = install_sizes(&apps, &known);
    println!(
        "크기 기록 없는 프로그램 중 잴 대상 {}개, 잰 것 {}개",
        targets.len(),
        real.len()
    );
    for (id, loc) in &targets {
        println!("  {id}  {loc}  {:?} KB", real.get(id));
    }
    println!("OK");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용 예제입니다");
}
