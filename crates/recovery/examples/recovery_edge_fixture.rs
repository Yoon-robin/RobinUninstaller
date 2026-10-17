//! 백업·복원의 경계 상황을 테스트 전용 위치(`Robin Test Labs\Edge`)로만 검증한다. 다른 것은 건드리지 않는다.
//!
//! - 복원할 자리에 새 파일이 생겼으면 덮어쓰지 않는다
//! - 다시 정리할 때 이미 있는 레지스트리 백업을 덮어쓰지 않는다 (지난번에 지운 부분이 백업에서 사라지지 않게)
//! - 상위 키와 하위 키를 함께 정리해도 둘 다 온전히 복원된다
//! - 조작된 매니페스트(백업 경로·위치)는 거부한다
//! - `ROBIN_CROSS_DRIVE_DIR`(백업과 다른 드라이브의 폴더)가 있으면 드라이브를 넘는 이동과 복원도 확인한다
//!
//! ```text
//! cargo run -p robin-recovery --example recovery_edge_fixture
//! ```

#[cfg(windows)]
fn main() {
    use std::path::{Path, PathBuf};

    use robin_recovery::{Backup, Guard, ItemStatus, Store, registry};
    use robin_scanner::{Leftover, LeftoverKind};
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    const KEY: &str = r"Software\Robin Test Labs\Edge";

    let local = PathBuf::from(std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA"));
    let base = local.join("Robin Test Labs").join("Edge");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let cleanup = || {
        let _ = std::fs::remove_dir_all(&base);
        let _ = hkcu.delete_subkey_all(KEY);
        let _ = hkcu.delete_subkey(r"Software\Robin Test Labs");
        let _ = std::fs::remove_dir(local.join("Robin Test Labs"));
    };
    cleanup();

    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };

    let dir = std::env::temp_dir().join(format!("robin-edge-{}", std::process::id()));
    let store = Store::new(dir.join("backups"));
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        ..Default::default()
    };
    let leftover = |kind, location: String, value: Option<&str>| {
        Leftover::new(kind, location, value.map(str::to_string), vec![])
    };

    // 1) 복원할 자리에 새 파일이 생기면 덮어쓰지 않는다
    std::fs::create_dir_all(&base).unwrap();
    let file = base.join("settings.json");
    std::fs::write(&file, "original").unwrap();
    let mut op = robin_recovery::new_operation(
        "edge",
        &[leftover(
            LeftoverKind::File,
            file.display().to_string(),
            None,
        )],
    );
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    check(
        "파일 정리",
        op.items[0].status == ItemStatus::Removed && !file.exists(),
    );
    std::fs::write(&file, "new").unwrap();
    robin_recovery::restore(&store, &mut op, &guard, None, |_| {});
    check(
        "새로 생긴 파일을 덮어쓰지 않음",
        op.items[0].status == ItemStatus::Removed
            && std::fs::read_to_string(&file).unwrap() == "new",
    );
    std::fs::remove_file(&file).unwrap();
    robin_recovery::restore(&store, &mut op, &guard, None, |_| {});
    check(
        "자리가 비면 원본으로 복원",
        op.items[0].status == ItemStatus::Restored
            && std::fs::read_to_string(&file).unwrap() == "original",
    );

    // 2) 상위·하위 키를 함께 정리하고 복원 (하위 키가 먼저 정리되도록 앞에 둔다)
    let (k, _) = hkcu.create_subkey(format!(r"{KEY}\Parent\Child")).unwrap();
    k.set_value("Level", &7u32).unwrap();
    let (p, _) = hkcu.create_subkey(format!(r"{KEY}\Parent")).unwrap();
    p.set_value("Theme", &"dark").unwrap();
    let parent = format!(r"HKCU\{KEY}\Parent");
    let child = format!(r"{parent}\Child");
    let mut op = robin_recovery::new_operation(
        "edge",
        &[
            leftover(LeftoverKind::RegistryKey, child.clone(), None),
            leftover(LeftoverKind::RegistryKey, parent.clone(), None),
        ],
    );
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    check(
        "상위·하위 키 정리",
        op.items.iter().all(|i| i.status == ItemStatus::Removed)
            && hkcu.open_subkey(format!(r"{KEY}\Parent")).is_err(),
    );
    robin_recovery::restore(&store, &mut op, &guard, None, |i| {
        if let Some(e) = &i.error {
            println!("  복원 {}: {e}", i.location);
        }
    });
    let level: Option<u32> = hkcu
        .open_subkey(format!(r"{KEY}\Parent\Child"))
        .and_then(|k| k.get_value("Level"))
        .ok();
    let theme: Option<String> = hkcu
        .open_subkey(format!(r"{KEY}\Parent"))
        .and_then(|k| k.get_value("Theme"))
        .ok();
    check(
        "상위·하위 키 모두 복원",
        op.items.iter().all(|i| i.status == ItemStatus::Restored)
            && level == Some(7)
            && theme.as_deref() == Some("dark"),
    );

    // 3) 다시 정리할 때 기존 레지스트리 백업을 덮어쓰지 않는다
    let (k, _) = hkcu.create_subkey(format!(r"{KEY}\Retry\A")).unwrap();
    k.set_value("x", &1u32).unwrap();
    hkcu.create_subkey(format!(r"{KEY}\Retry\B")).unwrap();
    let retry = format!(r"HKCU\{KEY}\Retry");
    let mut op = robin_recovery::new_operation(
        "edge",
        &[leftover(LeftoverKind::RegistryKey, retry.clone(), None)],
    );
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    // 지난번에 일부만 지워진 상황을 흉내: 키를 일부만 되살리고 "실패"로 되돌린다
    hkcu.create_subkey(format!(r"{KEY}\Retry\B")).unwrap();
    op.items[0].status = ItemStatus::Failed;
    store.save(&op).unwrap();
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    let backup_file = store.dir(&op.id).join(r"registry\0.json");
    let saved: registry::RegistryBackup =
        serde_json::from_slice(&std::fs::read(&backup_file).unwrap()).unwrap();
    let has_a = matches!(&saved, registry::RegistryBackup::Key { dump, .. }
        if dump.subkeys.iter().any(|s| s.name == "A"));
    check("다시 정리해도 처음 백업(A 포함)을 유지", has_a);
    robin_recovery::restore(&store, &mut op, &guard, None, |_| {});
    check(
        "처음 상태(A의 값 포함)로 복원",
        hkcu.open_subkey(format!(r"{KEY}\Retry\A"))
            .and_then(|k| k.get_value::<u32, _>("x"))
            .ok()
            == Some(1),
    );

    // 4) 조작된 매니페스트는 거부한다
    std::fs::write(&file, "tamper").unwrap();
    let mut op = robin_recovery::new_operation(
        "edge",
        &[leftover(
            LeftoverKind::File,
            file.display().to_string(),
            None,
        )],
    );
    op.items[0].backup = Some(Backup::Quarantine {
        path: r"files\0\..\..\escape".into(),
    });
    store.save(&op).unwrap();
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    check(
        "이상한 백업 경로는 거부",
        op.items[0].status == ItemStatus::Blocked && file.exists(),
    );
    let mut op = robin_recovery::new_operation(
        "edge",
        &[leftover(
            LeftoverKind::File,
            file.display().to_string(),
            None,
        )],
    );
    robin_recovery::clean(&store, &mut op, &guard, |_| {});
    op.items[0].location = r"C:\Windows\System32\edge.txt".into();
    robin_recovery::restore(&store, &mut op, &guard, None, |_| {});
    check(
        "기록의 위치를 Windows 폴더로 바꾸면 복원 거부",
        op.items[0].status == ItemStatus::Removed
            && !Path::new(r"C:\Windows\System32\edge.txt").exists(),
    );
    let mut copied = store.load(&op.id).unwrap();
    copied.id = "0000000000001-1".into();
    check(
        "폴더와 ID가 다른 기록은 읽지 않음",
        std::fs::create_dir_all(store.dir(&copied.id)).is_ok()
            && std::fs::write(
                store.dir(&copied.id).join("manifest.json"),
                serde_json::to_vec(&{
                    let mut c = copied.clone();
                    c.id = op.id.clone();
                    c
                })
                .unwrap(),
            )
            .is_ok()
            && store.load(&copied.id).is_err(),
    );

    // 5) 다른 드라이브로 옮겼다가 되돌리기
    if let Some(other) = std::env::var_os("ROBIN_CROSS_DRIVE_DIR") {
        let src = PathBuf::from(other).join("Robin Test Labs Cross");
        let _ = std::fs::remove_dir_all(&src);
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("sub").join("a.txt"), "cross").unwrap();
        let mut op = robin_recovery::new_operation(
            "edge",
            &[leftover(
                LeftoverKind::Folder,
                src.display().to_string(),
                None,
            )],
        );
        robin_recovery::clean(&store, &mut op, &guard, |i| {
            if let Some(e) = &i.error {
                println!("  정리 {}: {e}", i.location);
            }
        });
        check(
            "다른 드라이브 폴더 정리",
            op.items[0].status == ItemStatus::Removed && !src.exists(),
        );
        robin_recovery::restore(&store, &mut op, &guard, None, |_| {});
        check(
            "다른 드라이브로 복원",
            op.items[0].status == ItemStatus::Restored
                && std::fs::read_to_string(src.join("sub").join("a.txt"))
                    .ok()
                    .as_deref()
                    == Some("cross"),
        );
        let _ = std::fs::remove_dir_all(&src);
    } else {
        println!("건너뜀: 다른 드라이브 (ROBIN_CROSS_DRIVE_DIR 없음)");
    }

    cleanup();
    let _ = std::fs::remove_dir_all(&dir);
    std::process::exit(if ok { 0 } else { 1 });
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}
