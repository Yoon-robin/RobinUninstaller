//! 제거·잔여물 기능을 실제 프로그램 없이 시험하기 위한 가짜 프로그램 "Fixture Notes" (게시자 "Robin Test Labs").
//!
//! 현재 사용자(HKCU)와 사용자 AppData에만 쓰므로 관리자 권한이 필요 없고 다른 프로그램에 영향을 주지 않는다.
//!
//! ```text
//! robin-fixture install [--cancel|--hang|--machine]   등록 + 일부러 흔적과 미끼를 만든다
//!                                         (--machine: 관리자 권한 흔적도 만든다. 관리자로 실행해야 함)
//! robin-fixture remove                      만든 것을 전부 지운다 (정리용)
//! ```
//!
//! 등록되는 언인스톨러는 NSIS처럼 동작한다: 자기 자신을 임시 폴더에 복사해 다시 실행하고 원본은 바로 끝난다.
//! 복사본은 몇 초 뒤 **설치 폴더와 등록 정보만** 지운다. 아래 흔적은 일부러 남긴다:
//!
//! - 남는 흔적 (잔여물 스캔이 찾아야 함)
//!   - `%APPDATA%\Robin Test Labs\Fixture Notes\settings.json`
//!   - `%LOCALAPPDATA%\Fixture Notes\cache\a.bin`
//!   - `HKCU\Software\Robin Test Labs\Fixture Notes`
//!   - `HKCU\...\Run` 값 `FixtureNotes` (설치 폴더를 가리킴, 시작프로그램 화면 시험에도 쓴다)
//! - 미끼 (확실 후보가 되면 안 됨)
//!   - `%APPDATA%\Robin Test Labs\Other Product\data.txt`, `HKCU\Software\Robin Test Labs\Other Product`
//!   - `%APPDATA%\Fixture Notes 2019\old.txt` (다른 버전)
//!   - `HKCU\Software\Fixture Viewer` (비슷한 이름의 다른 프로그램)
//!
//! `--cancel`: 아무것도 지우지 않고 끝난다(취소 흉내). `--hang`: 2분 동안 끝나지 않는다.
//!
//! `--machine`(관리자 권한 필요)은 아래 흔적을 추가로 남긴다. 지우거나 되돌리려면 관리자 권한이 있어야 한다.
//!   - `HKLM\SOFTWARE\Robin Test Labs\Fixture Notes`
//!   - `C:\Program Files\Robin Test Labs\Fixture Notes\data.txt`
//!   - `HKLM\...\Run` 값 `FixtureNotesMachine` (없는 파일을 가리킴, 모든 사용자 시작 항목 끄기·켜기 시험용)

#[cfg(windows)]
fn main() {
    imp::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}

#[cfg(windows)]
mod imp {
    use std::path::{Path, PathBuf};
    use std::process::{Command, exit};
    use std::time::Duration;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const UNINSTALL_KEY: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RobinFixtureApp";
    const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    const RUN_VALUE: &str = "FixtureNotes";
    /// 시작프로그램 화면에서 끄고 켤 때 Windows(작업 관리자 방식)가 남기는 표시
    const STARTUP_APPROVED_KEY: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const MACHINE_RUN_VALUE: &str = "FixtureNotesMachine";
    const PUBLISHER: &str = "Robin Test Labs";
    const PRODUCT: &str = "Fixture Notes";

    fn env_dir(var: &str) -> PathBuf {
        PathBuf::from(std::env::var(var).unwrap_or_else(|_| panic!("{var}")))
    }

    fn install_dir() -> PathBuf {
        env_dir("LOCALAPPDATA").join(PUBLISHER).join(PRODUCT)
    }

    /// 일부러 남기는 흔적과 미끼 파일들 (설치 폴더 밖)
    fn extra_files() -> Vec<(PathBuf, &'static str)> {
        let roaming = env_dir("APPDATA");
        let local = env_dir("LOCALAPPDATA");
        vec![
            (
                roaming.join(PUBLISHER).join(PRODUCT).join("settings.json"),
                "{\"theme\":\"dark\"}",
            ),
            (local.join(PRODUCT).join("cache").join("a.bin"), "cache"),
            (
                roaming
                    .join(PUBLISHER)
                    .join("Other Product")
                    .join("data.txt"),
                "decoy",
            ),
            (roaming.join("Fixture Notes 2019").join("old.txt"), "decoy"),
        ]
    }

    fn extra_keys() -> [String; 3] {
        [
            format!(r"Software\{PUBLISHER}\{PRODUCT}"),
            format!(r"Software\{PUBLISHER}\Other Product"),
            r"Software\Fixture Viewer".to_string(),
        ]
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        match args.first().map(String::as_str) {
            Some("install") => install(args.get(1).map(String::as_str)),
            Some("remove") => remove_all(),
            Some("--uninstall") => stage1(args.get(1).map(String::as_str)),
            Some("--stage2") => stage2(Path::new(&args[1])),
            _ => {
                eprintln!("사용법: robin-fixture install [--cancel|--hang] | remove");
                exit(2);
            }
        }
    }

    fn machine_dir() -> PathBuf {
        env_dir("ProgramFiles").join(PUBLISHER)
    }

    fn machine_key() -> String {
        format!(r"SOFTWARE\{PUBLISHER}\{PRODUCT}")
    }

    /// 관리자 권한이 있어야 만들고 지울 수 있는 흔적
    fn install_machine() {
        let dir = machine_dir().join(PRODUCT);
        std::fs::create_dir_all(&dir).expect("Program Files 폴더 생성 (관리자 권한 필요)");
        std::fs::write(dir.join("data.txt"), "machine-wide data").unwrap();
        let (k, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .create_subkey(machine_key())
            .expect("HKLM 키 생성 (관리자 권한 필요)");
        k.set_value("Theme", &"dark").unwrap();
        k.set_value("Level", &7u32).unwrap();
        let (run, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .create_subkey(RUN_KEY)
            .expect("HKLM Run 키 열기 (관리자 권한 필요)");
        // 없는 파일을 가리킨다: 로그인할 때 아무것도 실행되지 않고, 화면에서는 "파일 없음"으로 보여야 한다
        run.set_value(
            MACHINE_RUN_VALUE,
            &format!("\"{}\" --tray", dir.join("notes-service.exe").display()),
        )
        .unwrap();
        println!("관리자 권한 흔적 만듦: {}", dir.display());
    }

    fn install(mode: Option<&str>) {
        if mode == Some("--machine") {
            install_machine();
        }
        let mode = mode.filter(|m| *m != "--machine");
        let dir = install_dir();
        std::fs::create_dir_all(&dir).expect("폴더 생성");
        let uninstaller = dir.join("uninstall.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &uninstaller).expect("언인스톨러 복사");
        std::fs::write(dir.join("notes.exe"), "not a real program").unwrap();

        for (path, content) in extra_files() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for key in extra_keys() {
            let (k, _) = hkcu.create_subkey(&key).unwrap();
            k.set_value("Theme", &"dark").unwrap();
        }
        let (run, _) = hkcu.create_subkey(RUN_KEY).unwrap();
        run.set_value(
            RUN_VALUE,
            &format!("\"{}\" --tray", dir.join("notes.exe").display()),
        )
        .unwrap();

        let (key, _) = hkcu.create_subkey(UNINSTALL_KEY).expect("레지스트리 등록");
        let mut cmd = format!("\"{}\" --uninstall", uninstaller.display());
        if let Some(mode) = mode {
            cmd.push(' ');
            cmd.push_str(mode);
        }
        key.set_value("DisplayName", &PRODUCT).unwrap();
        key.set_value("Publisher", &PUBLISHER).unwrap();
        key.set_value("DisplayVersion", &"1.0.0").unwrap();
        key.set_value("InstallLocation", &dir.display().to_string())
            .unwrap();
        key.set_value("UninstallString", &cmd).unwrap();
        key.set_value("DisplayIcon", &uninstaller.display().to_string())
            .unwrap();
        key.set_value("EstimatedSize", &1024u32).unwrap();
        println!("등록함: {}", dir.display());
    }

    /// NSIS 흉내: 임시 폴더로 자신을 복사해 실행하고 바로 끝난다.
    fn stage1(mode: Option<&str>) {
        match mode {
            Some("--cancel") => {
                std::thread::sleep(Duration::from_secs(1));
                exit(1602); // 사용자 취소
            }
            Some("--hang") => {
                std::thread::sleep(Duration::from_secs(120));
                exit(0);
            }
            _ => {}
        }
        let temp = std::env::temp_dir().join("robin_fixture_Au_.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &temp).expect("임시 복사");
        Command::new(&temp)
            .arg("--stage2")
            .arg(install_dir())
            .spawn()
            .expect("2단계 실행");
        exit(0);
    }

    /// 설치 폴더와 등록 정보만 지운다 (흔적은 일부러 남김).
    fn stage2(dir: &Path) {
        std::thread::sleep(Duration::from_secs(3));
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(UNINSTALL_KEY);
        let _ = std::fs::remove_dir_all(dir);
        exit(0);
    }

    fn remove_all() {
        // 관리자 권한 흔적 (권한이 없으면 조용히 건너뛴다)
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let _ = hklm.delete_subkey_all(machine_key());
        let _ = hklm.delete_subkey(format!(r"SOFTWARE\{PUBLISHER}"));
        let _ = std::fs::remove_dir_all(machine_dir());
        for (key, value) in [
            (RUN_KEY, MACHINE_RUN_VALUE),
            (STARTUP_APPROVED_KEY, MACHINE_RUN_VALUE),
        ] {
            if let Ok(k) = hklm.open_subkey_with_flags(key, winreg::enums::KEY_SET_VALUE) {
                let _ = k.delete_value(value);
            }
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(UNINSTALL_KEY);
        for key in extra_keys() {
            let _ = hkcu.delete_subkey_all(&key);
        }
        let _ = hkcu.delete_subkey(format!(r"Software\{PUBLISHER}"));
        if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_SET_VALUE) {
            let _ = run.delete_value(RUN_VALUE);
        }
        if let Ok(approved) =
            hkcu.open_subkey_with_flags(STARTUP_APPROVED_KEY, winreg::enums::KEY_SET_VALUE)
        {
            let _ = approved.delete_value(RUN_VALUE);
        }
        let _ = std::fs::remove_dir_all(install_dir());
        for (path, _) in extra_files() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        }
        let roaming = env_dir("APPDATA");
        let local = env_dir("LOCALAPPDATA");
        for dir in [
            roaming.join(PUBLISHER),
            local.join(PUBLISHER),
            local.join(PRODUCT),
            local.join("RobinFixtureApp"), // 예전 버전의 테스트 앱 폴더
        ] {
            let _ = std::fs::remove_dir_all(dir);
        }
        let _ = std::fs::remove_file(std::env::temp_dir().join("robin_fixture_Au_.exe"));
        println!("정리함");
    }
}
