//! 다른 드라이브의 폴더를 정리·되돌리기·백업 삭제할 때 진행 상황이 실제로 올라가는지 본다.
//! 백업 저장소(임시 폴더, 보통 C:)와 다른 드라이브에 **시험용 폴더를 직접 만들어** 쓰고, 끝나면 지운다.
//! 실제 프로그램이나 사용자 파일은 건드리지 않는다.
//!
//! ```text
//! cargo run -p robin-recovery --example progress_cross_drive -- D:\
//! ```

#[cfg(windows)]
fn main() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use robin_recovery::progress::{Progress, with};
    use robin_recovery::{Guard, ItemStatus, Store};
    use robin_scanner::{Leftover, LeftoverKind};

    let drive = std::env::args()
        .nth(1)
        .expect("다른 드라이브를 주세요 (예: D:\\)");
    let src = std::path::Path::new(&drive).join("RobinX-progress-test");
    let store_root = std::env::temp_dir().join("robin-progress-test-store");
    assert!(!src.exists(), "{} 가 이미 있어요", src.display());
    let _ = std::fs::remove_dir_all(&store_root);

    // 시험용 파일: 하위 폴더 20개 × 파일 50개 × 64 KB ≈ 64 MB
    for d in 0..20 {
        let dir = src.join(format!("sub{d:02}"));
        std::fs::create_dir_all(&dir).unwrap();
        for f in 0..50 {
            std::fs::write(
                dir.join(format!("f{f:02}.bin")),
                vec![(d + f) as u8; 64 * 1024],
            )
            .unwrap();
        }
    }
    println!("시험 폴더 만듦: {}", src.display());

    let store = Store::new(store_root.clone());
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    };

    // 진행 상황을 0.1초마다 찍어 본다 (앱은 같은 방식으로 화면에 보낸다)
    let watch = |label: &'static str, progress: Arc<Progress>, stop: Arc<AtomicBool>| {
        std::thread::spawn(move || {
            let mut seen = 0;
            while !stop.load(Ordering::Relaxed) {
                let s = progress.snapshot();
                if s.files > 0 && seen < 3 {
                    println!(
                        "  {label} 중: {}/{} · 파일 {} · {} 바이트 · {}",
                        s.done, s.total, s.files, s.bytes, s.current
                    );
                    seen += 1;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        })
    };
    let run = |label: &'static str, f: &mut dyn FnMut()| {
        let progress = Arc::new(Progress::new());
        let stop = Arc::new(AtomicBool::new(false));
        let h = watch(label, progress.clone(), stop.clone());
        with(progress.clone(), f);
        stop.store(true, Ordering::Relaxed);
        let _ = h.join();
        let s = progress.snapshot();
        println!(
            "  {label} 끝: {}/{} · 파일 {} · {} 바이트",
            s.done, s.total, s.files, s.bytes
        );
        s
    };

    let item = Leftover::new(
        LeftoverKind::Folder,
        src.display().to_string(),
        None,
        vec![],
    );
    let mut op = robin_recovery::new_operation("진행 상황 시험", &[item]);
    store.save(&op).unwrap();

    let s = run("정리", &mut || {
        robin_recovery::clean(&store, &mut op, &guard, |_| {}).unwrap()
    });
    assert_eq!(
        op.items[0].status,
        ItemStatus::Removed,
        "{:?}",
        op.items[0].error
    );
    assert_eq!(
        (s.done, s.total, s.files),
        (1, 1, 1000),
        "다른 드라이브라 파일마다 복사해야 함"
    );
    assert!(!src.exists());

    let s = run("되돌리기", &mut || {
        robin_recovery::restore(&store, &mut op, &guard, None, |_| {}).unwrap()
    });
    assert_eq!(
        op.items[0].status,
        ItemStatus::Restored,
        "{:?}",
        op.items[0].error
    );
    assert_eq!((s.done, s.total, s.files), (1, 1, 1000));
    assert!(src.join("sub19").join("f49.bin").exists());

    // 백업을 하나 더 만들어 지우기를 본다
    let item = Leftover::new(
        LeftoverKind::Folder,
        src.display().to_string(),
        None,
        vec![],
    );
    let mut op2 = robin_recovery::new_operation("진행 상황 시험 2", &[item]);
    store.save(&op2).unwrap();
    robin_recovery::clean(&store, &mut op2, &guard, |_| {}).unwrap();
    let s = run("백업 삭제", &mut || store.delete(&op2.id).unwrap());
    assert!(
        s.bytes >= 64 * 1024 * 1000,
        "지운 크기가 올라가야 함: {s:?}"
    );
    assert!(!store.dir(&op2.id).exists());

    let _ = std::fs::remove_dir_all(&store_root);
    let _ = std::fs::remove_dir_all(&src);
    println!("통과");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용 예제예요");
}
