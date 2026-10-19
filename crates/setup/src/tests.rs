//! 임시 폴더와 시험용 레지스트리 위치(`HKCU\Software\RobinX-SetupTest-*`)에서만 설치·제거한다.

use super::*;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

struct Sandbox {
    root: PathBuf,
    env: Env,
    source: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let tag = format!("{name}-{}", std::process::id());
        let root = std::env::temp_dir().join(format!("robin-setup-test-{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let env = Env {
            local_appdata: root.join("Local"),
            start_menu: root.join("StartMenu"),
            desktop: root.join("Desktop"),
            reg_base: format!(r"Software\RobinX-SetupTest-{tag}"),
        };
        std::fs::create_dir_all(&env.local_appdata).unwrap();
        std::fs::create_dir_all(&env.desktop).unwrap();
        // 설치 파일 역할을 하는 가짜 exe (내용만 다르게)
        let source = root.join("Downloads").join("RobinX_0.2.0_x64-setup.exe");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"robinx 0.2.0").unwrap();
        Sandbox { root, env, source }
    }

    fn reg_value(&self, name: &str) -> Option<String> {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(self.env.uninstall_key())
            .ok()?
            .get_value(name)
            .ok()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.env.reg_base);
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn fresh_install_registers_copies_and_links() {
    let sb = Sandbox::new("fresh");
    let opts = InstallOptions {
        desktop_shortcut: true,
        ..Default::default()
    };
    let r = install(&sb.env, &sb.source, "0.2.0", &opts).unwrap();

    let exe = sb.env.install_dir().join(EXE);
    assert_eq!(r.exe, exe);
    assert_eq!(std::fs::read(&exe).unwrap(), b"robinx 0.2.0");
    assert!(!exe.with_file_name("robinx.exe.new").exists());

    assert_eq!(sb.reg_value("DisplayVersion").as_deref(), Some("0.2.0"));
    assert_eq!(sb.reg_value("DisplayName").as_deref(), Some("RobinX"));
    assert_eq!(
        sb.reg_value("UninstallString"),
        Some(format!("\"{}\" --uninstall", exe.display()))
    );
    let found = existing(&sb.env).unwrap();
    assert!(same_path(&found.dir, &sb.env.install_dir()));
    assert!(!found.legacy);

    let start = sb.env.start_menu.join(LINK_NAME);
    let desk = sb.env.desktop.join(LINK_NAME);
    assert!(same_path(&shortcut::target_of(&start).unwrap(), &exe));
    assert!(same_path(&shortcut::target_of(&desk).unwrap(), &exe));
    let m = read_manifest(&sb.env.install_dir()).unwrap();
    assert_eq!(m.files, vec![EXE.to_string()]);
    assert_eq!(m.shortcuts.len(), 2);
}

#[test]
fn installed_copy_refuses_to_install_over_itself() {
    let sb = Sandbox::new("self");
    install(&sb.env, &sb.source, "0.2.0", &InstallOptions::default()).unwrap();
    let exe = sb.env.install_dir().join(EXE);
    assert!(install(&sb.env, &exe, "0.2.0", &InstallOptions::default()).is_err());
}

/// 0.1.0(NSIS)은 데이터 폴더에 설치했다. 업데이트하면 실행 파일 두 개만 지우고 백업은 그대로 둬야 한다.
#[test]
fn update_from_nsis_keeps_backups_and_moves_links() {
    let sb = Sandbox::new("nsis");
    let legacy = sb.env.data_dir();
    std::fs::create_dir_all(legacy.join("backups").join("op-1")).unwrap();
    std::fs::write(
        legacy.join("backups").join("op-1").join("manifest.json"),
        b"{}",
    )
    .unwrap();
    std::fs::write(legacy.join("settings.json"), b"{}").unwrap();
    std::fs::write(legacy.join(EXE), b"robinx 0.1.0").unwrap();
    std::fs::write(legacy.join(LEGACY_UNINSTALLER), b"nsis").unwrap();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(sb.env.uninstall_key()).unwrap();
    key.set_value("InstallLocation", &format!("\"{}\"", legacy.display()))
        .unwrap();
    key.set_value("DisplayVersion", &"0.1.0").unwrap();
    let start = sb.env.start_menu.join(LINK_NAME);
    shortcut::create(&start, &legacy.join(EXE)).unwrap();

    let before = existing(&sb.env).unwrap();
    assert!(before.legacy);
    assert_eq!(before.version.as_deref(), Some("0.1.0"));

    let opts = InstallOptions {
        update: true,
        ..Default::default()
    };
    let r = install(&sb.env, &sb.source, "0.2.0", &opts).unwrap();

    assert!(!legacy.join(EXE).exists());
    assert!(!legacy.join(LEGACY_UNINSTALLER).exists());
    assert_eq!(r.migrated.len(), 2);
    // 데이터는 그대로
    assert!(legacy.join("settings.json").exists());
    assert!(
        legacy
            .join("backups")
            .join("op-1")
            .join("manifest.json")
            .exists()
    );
    // 바로 가기는 새 위치를 가리킨다. 업데이트라 바탕 화면 바로 가기는 새로 만들지 않는다.
    let exe = sb.env.install_dir().join(EXE);
    assert!(same_path(&shortcut::target_of(&start).unwrap(), &exe));
    assert!(!sb.env.desktop.join(LINK_NAME).exists());
    assert_eq!(sb.reg_value("DisplayVersion").as_deref(), Some("0.2.0"));
}

#[test]
fn reinstall_replaces_the_exe() {
    let sb = Sandbox::new("again");
    install(&sb.env, &sb.source, "0.2.0", &InstallOptions::default()).unwrap();
    std::fs::write(&sb.source, b"robinx 0.3.0").unwrap();
    install(
        &sb.env,
        &sb.source,
        "0.3.0",
        &InstallOptions {
            update: true,
            ..Default::default()
        },
    )
    .unwrap();
    let exe = sb.env.install_dir().join(EXE);
    assert_eq!(std::fs::read(&exe).unwrap(), b"robinx 0.3.0");
    assert!(!exe.with_file_name("robinx.exe.old").exists());
    assert_eq!(sb.reg_value("DisplayVersion").as_deref(), Some("0.3.0"));
}

#[test]
fn uninstall_keeps_data_and_files_it_did_not_install() {
    let sb = Sandbox::new("uninst");
    install(
        &sb.env,
        &sb.source,
        "0.2.0",
        &InstallOptions {
            desktop_shortcut: true,
            ..Default::default()
        },
    )
    .unwrap();
    let dir = sb.env.install_dir();
    std::fs::write(dir.join("내 메모.txt"), b"user file").unwrap();
    std::fs::create_dir_all(sb.env.data_dir().join("backups")).unwrap();
    // 다른 프로그램의 같은 이름 바로 가기는 남아야 한다
    let other = sb.root.join("other.exe");
    std::fs::write(&other, b"x").unwrap();
    let foreign = sb.env.desktop.join("RobinX.lnk");
    shortcut::create(&foreign, &other).unwrap();

    let r = uninstall(&sb.env, &UninstallOptions::default()).unwrap();

    assert!(!dir.join(EXE).exists());
    assert!(!dir.join(MANIFEST).exists());
    assert!(
        dir.join("내 메모.txt").exists(),
        "사용자 파일은 남아야 한다"
    );
    assert!(!sb.env.start_menu.join(LINK_NAME).exists());
    assert!(
        foreign.exists(),
        "RobinX를 가리키지 않는 바로 가기는 남아야 한다"
    );
    assert!(sb.env.data_dir().join("backups").exists());
    assert!(r.kept_data.is_some());
    assert!(existing(&sb.env).is_none());
    assert!(sb.reg_value("DisplayName").is_none());
}

#[test]
fn uninstall_removes_empty_dir_and_data_only_when_asked() {
    let sb = Sandbox::new("data");
    install(&sb.env, &sb.source, "0.2.0", &InstallOptions::default()).unwrap();
    std::fs::create_dir_all(sb.env.data_dir().join("backups")).unwrap();
    std::fs::create_dir_all(sb.env.webview_dir()).unwrap();
    uninstall(&sb.env, &UninstallOptions { delete_data: true }).unwrap();
    assert!(!sb.env.install_dir().exists());
    assert!(!sb.env.data_dir().exists());
    assert!(!sb.env.webview_dir().exists());
}

/// 조작된 설치 기록으로 설치 폴더 밖의 파일을 지우게 할 수 없다.
#[test]
fn tampered_manifest_cannot_reach_outside() {
    let sb = Sandbox::new("tamper");
    install(&sb.env, &sb.source, "0.2.0", &InstallOptions::default()).unwrap();
    let victim = sb.env.local_appdata.join("victim.txt");
    std::fs::write(&victim, b"keep").unwrap();
    let dir = sb.env.install_dir();
    let mut m = read_manifest(&dir).unwrap();
    m.files.push(r"..\..\victim.txt".into());
    m.files.push(victim.display().to_string());
    write_manifest(&dir, &m).unwrap();
    uninstall(&sb.env, &UninstallOptions::default()).unwrap();
    assert!(victim.exists());
}

#[test]
fn plain_names_only() {
    for ok in ["robinx.exe", "uninstall.exe"] {
        assert!(is_plain_name(ok));
    }
    for bad in ["", ".", "..", r"..\x", "a/b", r"C:\x", "x..y"] {
        assert!(!is_plain_name(bad), "{bad}");
    }
}
