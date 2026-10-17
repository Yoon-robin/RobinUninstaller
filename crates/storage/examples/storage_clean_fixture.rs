//! 디버그 빌드의 시험용 항목("테스트 앱 캐시")만 정리해서 정리 기능을 확인한다.
//! 실제 캐시는 건드리지 않는다.
//!
//! ```text
//! cargo run -p robin-fixture -- install
//! cargo run -p robin-storage --example storage_clean_fixture
//! cargo run -p robin-fixture -- remove
//! ```

#[cfg(windows)]
fn main() {
    let cache = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
        .join(r"Fixture Notes\cache");
    assert!(cache.is_dir(), "먼저 robin-fixture install 을 실행하세요");
    // 하위 폴더와 파일을 더 만들어 빈 폴더 정리까지 확인한다
    std::fs::create_dir_all(cache.join(r"nested\deeper")).unwrap();
    std::fs::write(cache.join(r"nested\deeper\b.bin"), [7u8; 4096]).unwrap();

    let found = robin_storage::scan()
        .into_iter()
        .find(|c| c.id == "fixture-cache")
        .expect("스캔 결과에 테스트 항목이 있어야 함");
    assert!(found.bytes >= 4096 && found.files >= 2, "{found:?}");
    println!("스캔: {} 바이트, 파일 {}개", found.bytes, found.files);

    // 되돌릴 수 없는 항목은 허락 없이 지우지 않아야 한다
    let refused = robin_storage::clean(&["recycle-bin".into()], false);
    assert!(refused[0].skipped.is_some() && refused[0].freed_bytes == 0);
    println!("확인: 휴지통은 따로 허락하지 않으면 건드리지 않음");

    let result = robin_storage::clean(&["fixture-cache".into()], false);
    let r = &result[0];
    assert!(r.skipped.is_none(), "{r:?}");
    assert_eq!(r.freed_bytes, found.bytes);
    assert_eq!(r.failed_files, 0);
    println!("정리: {} 바이트, 파일 {}개", r.freed_bytes, r.deleted_files);

    assert!(cache.is_dir(), "규칙이 가리키는 폴더 자체는 남아야 함");
    assert_eq!(
        std::fs::read_dir(&cache).unwrap().count(),
        0,
        "빈 하위 폴더까지 정리돼야 함"
    );
    assert!(
        robin_storage::scan()
            .iter()
            .all(|c| c.id != "fixture-cache"),
        "정리 뒤에는 0바이트라 목록에서 빠져야 함"
    );

    // 지우는 파일의 실제 위치가 정리 폴더 밖이면 거부해야 한다 (정션 바꿔치기 방지)
    let outside = cache.parent().unwrap().join("outside.txt");
    std::fs::write(&outside, "keep").unwrap();
    let root = std::fs::canonicalize(&cache).unwrap();
    assert!(robin_storage::system::delete_file_within(&outside, &root).is_err());
    assert!(outside.exists(), "정리 폴더 밖의 파일은 남아야 함");
    std::fs::remove_file(&outside).unwrap();
    println!("확인: 정리 폴더 밖의 파일은 지우지 않음");
    println!("통과");
}

#[cfg(not(windows))]
fn main() {}
