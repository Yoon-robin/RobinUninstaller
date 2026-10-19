//! 저장공간의 관리자 권한 정리를 디버그 빌드의 시험용 항목("테스트 앱 관리자 캐시")으로만 확인한다.
//! 실제 캐시는 건드리지 않는다.
//!
//! ```text
//! cargo run -p robin-fixture -- install --machine      (관리자 터미널에서: 가짜 캐시를 관리자 소유로 만든다)
//! cargo run -p robin-storage --example storage_admin_fixture
//!     일반 터미널: 스캔에 보이고, 미리 고르지 않고, 일반 권한 정리는 "관리자 권한 필요"로 돌려주고 파일은 그대로인지
//!     관리자 터미널: 위에 더해 실제로 정리되고 폴더 자체는 남는지
//! cargo run -p robin-fixture -- remove                  (관리자 터미널에서)
//! ```
//!
//! 앱에서 UAC 창을 거쳐 관리자 작업자가 정리하는 흐름은 사람이 "예"를 눌러야 해서 따로 시험한다 (LOOP E3 🔒).

#[cfg(windows)]
fn main() {
    const ID: &str = "fixture-admin-cache";
    let cache = std::path::PathBuf::from(std::env::var("ProgramData").unwrap())
        .join(r"Robin Test Labs\Fixture Notes\AdminCache");
    assert!(
        cache.is_dir(),
        "먼저 관리자 터미널에서 robin-fixture install --machine 을 실행하세요"
    );

    let found = robin_storage::scan()
        .into_iter()
        .find(|c| c.id == ID)
        .expect("스캔 결과에 관리자 테스트 항목이 있어야 함");
    assert!(found.admin, "관리자 항목으로 보여야 함");
    assert!(
        !found.default_selected,
        "관리자 항목은 미리 고르지 않아야 함"
    );
    assert!(found.bytes >= 12288 && found.files >= 2, "{found:?}");
    println!(
        "스캔: {} 바이트, 파일 {}개, 관리자 항목",
        found.bytes, found.files
    );

    let elevated = robin_storage::system::is_elevated();
    let result = robin_storage::clean(&[ID.into()], false);
    let r = &result[0];
    if !elevated {
        assert!(
            r.needs_admin,
            "일반 권한이면 관리자 권한이 필요하다고 돌려줘야 함: {r:?}"
        );
        assert_eq!(r.freed_bytes, 0);
        assert!(
            cache.join("a.bin").exists(),
            "일반 권한 정리는 파일을 건드리지 않아야 함"
        );
        println!("확인: 일반 권한 정리는 \"관리자 권한 필요\"로 돌려주고 파일은 그대로");
        println!("통과 (관리자 터미널에서 다시 실행하면 실제 정리까지 확인해요)");
        return;
    }

    assert!(r.skipped.is_none() && !r.needs_admin, "{r:?}");
    assert_eq!(r.freed_bytes, found.bytes);
    assert_eq!(r.failed_files, 0);
    println!(
        "정리(관리자): {} 바이트, 파일 {}개",
        r.freed_bytes, r.deleted_files
    );
    assert!(cache.is_dir(), "규칙이 가리키는 폴더 자체는 남아야 함");
    assert_eq!(
        std::fs::read_dir(&cache).unwrap().count(),
        0,
        "빈 하위 폴더까지 정리돼야 함"
    );
    println!("통과");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용 예제예요");
}
