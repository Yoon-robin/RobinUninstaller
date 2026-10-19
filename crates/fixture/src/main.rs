//! 제거·잔여물 기능을 실제 프로그램 없이 시험하기 위한 가짜 프로그램 "Fixture Notes" (게시자 "Robin Test Labs").
//!
//! 현재 사용자(HKCU)와 사용자 AppData에만 쓰므로 관리자 권한이 필요 없고 다른 프로그램에 영향을 주지 않는다.
//!
//! ```text
//! robin-fixture install [--cancel|--hang|--machine|--broken]   등록 + 일부러 흔적과 미끼를 만든다
//!                                         (--machine: 관리자 권한 흔적도 만든다. 관리자로 실행해야 함)
//!                                         (--broken: 언인스톨러 파일을 지워 "언인스톨러 없음" 상태로 만든다 → 강제 제거 시험용)
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
//!   - `HKCU\...\Run` 값 `FixtureNotes` (설치 폴더를 가리킴)
//!   - 시작 폴더의 `Fixture Notes.lnk` (설치 폴더의 notes.exe를 가리킴)
//!   - 예약 작업 `\Fixture Notes Updater` (현재 사용자 로그인 때 notes.exe 실행. 테스트 중에는 실행되지 않는다)
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
//!   - `HKLM\...\Run` 값 `FixtureNotesMachine` (없는 파일을 가리킴 → "파일 없음" 표시 확인용)
//!   - 예약 작업 `\Fixture Notes Machine Task` (부팅 때 SYSTEM으로 없는 파일 실행 → 실행돼도 아무 일 없음)
//!
//! 실행 파일이 없는 시작 항목(지워진 프로그램이 남긴 것 흉내, 테스트 앱과 이름이 달라 잔여물 스캔에는 걸리지 않는다):
//!   - `HKCU\...\Run` 값 `RobinGhostTool`, 시작 폴더의 `Robin Ghost Tool.lnk` (둘 다 없는 `ghost.exe`를 가리킴)
//!
//! 시작프로그램 화면에서 끈 상태(`Explorer\StartupApproved` 값)도 `remove`가 함께 지운다.

#[cfg(windows)]
fn main() {
    imp::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}

#[cfg(windows)]
mod winapi;

#[cfg(windows)]
mod imp {
    use std::path::{Path, PathBuf};
    use std::process::{Command, exit};

    use crate::winapi;
    use std::time::Duration;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, KEY_WOW64_64KEY};

    const UNINSTALL_KEY: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RobinFixtureApp";
    const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    const RUN_VALUE: &str = "FixtureNotes";
    const MACHINE_RUN_VALUE: &str = "FixtureNotesMachine";
    const APPROVED_KEY: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
    const SHORTCUT: &str = "Fixture Notes.lnk";
    const PUBLISHER: &str = "Robin Test Labs";
    const PRODUCT: &str = "Fixture Notes";
    const TASK_NAME: &str = "Fixture Notes Updater";
    const GHOST_RUN_VALUE: &str = "RobinGhostTool";
    const GHOST_SHORTCUT: &str = "Robin Ghost Tool.lnk";
    const MACHINE_TASK_NAME: &str = "Fixture Notes Machine Task";

    fn env_dir(var: &str) -> PathBuf {
        PathBuf::from(std::env::var(var).unwrap_or_else(|_| panic!("{var}")))
    }

    fn startup_folder() -> PathBuf {
        env_dir("APPDATA").join(r"Microsoft\Windows\Start Menu\Programs\Startup")
    }

    fn startup_shortcut() -> PathBuf {
        startup_folder().join(SHORTCUT)
    }

    /// 실행 파일이 없는 시작 항목이 가리키는 (일부러 만들지 않는) 파일
    fn ghost_exe() -> PathBuf {
        env_dir("LOCALAPPDATA")
            .join("Robin Ghost Tool")
            .join("ghost.exe")
    }

    fn create_shortcut(link: &Path, target: &Path) {
        winapi::create_shortcut(link, target, "--tray").expect("시작 폴더 바로 가기 만들기");
    }

    /// 현재 사용자가 로그인할 때만 실행되는 예약 작업 (관리자 권한 없이 만들 수 있다).
    /// notes.exe는 진짜 프로그램이 아니라서 혹시 실행돼도 아무 일도 일어나지 않는다.
    fn register_task(target: &Path) {
        let user = match (std::env::var("USERDOMAIN"), std::env::var("USERNAME")) {
            (Ok(d), Ok(u)) => format!("{d}\\{u}"),
            (_, Ok(u)) => u,
            _ => panic!("USERNAME"),
        };
        winapi::register_task(
            TASK_NAME,
            &winapi::Trigger::Logon { user },
            target,
            "--update",
            PUBLISHER,
            "RobinX test task",
        )
        .expect("예약 작업 만들기");
    }

    /// 관리자 예약 작업은 권한이 없으면 지워지지 않고 그대로 남는다 (조용히 건너뜀).
    fn unregister_task() {
        for name in [TASK_NAME, MACHINE_TASK_NAME] {
            let _ = winapi::delete_task(name);
        }
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
                eprintln!(
                    "사용법: robin-fixture install [--cancel|--hang|--machine|--broken] | remove"
                );
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
            .create_subkey_with_flags(RUN_KEY, KEY_SET_VALUE | KEY_WOW64_64KEY)
            .expect("HKLM Run 키 (관리자 권한 필요)");
        run.set_value(
            MACHINE_RUN_VALUE,
            &format!("\"{}\" --tray", dir.join("notes.exe").display()),
        )
        .unwrap();
        winapi::register_task(
            MACHINE_TASK_NAME,
            &winapi::Trigger::BootAsSystem,
            &dir.join("missing.exe"),
            "",
            PUBLISHER,
            "RobinX test task (machine)",
        )
        .expect("관리자 예약 작업 만들기 (관리자 권한 필요)");
        println!("관리자 권한 흔적 만듦: {}", dir.display());
    }

    fn install(mode: Option<&str>) {
        if mode == Some("--machine") {
            install_machine();
        }
        let broken = mode == Some("--broken");
        let mode = mode.filter(|m| *m != "--machine" && *m != "--broken");
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
        create_shortcut(&startup_shortcut(), &dir.join("notes.exe"));
        // 실행 파일이 없는 시작 항목 (ghost.exe는 만들지 않는다)
        run.set_value(
            GHOST_RUN_VALUE,
            &format!("\"{}\" --background", ghost_exe().display()),
        )
        .unwrap();
        create_shortcut(&startup_folder().join(GHOST_SHORTCUT), &ghost_exe());
        register_task(&dir.join("notes.exe"));

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
        if broken {
            // 등록 정보는 남기고 언인스톨러 파일만 없앤다 (언인스톨러가 손상된 프로그램 흉내)
            std::fs::remove_file(&uninstaller).expect("언인스톨러 삭제");
            println!("언인스톨러 없음 상태로 만듦");
        }
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

    /// 시작프로그램 화면에서 테스트 항목을 끈 기록(원래 값)도 지운다
    fn forget_startup_history() {
        use robin_startup::StartupSource::*;
        use robin_startup::model::make_id;
        let Some(history) = robin_startup::History::default_location() else {
            return;
        };
        for (source, name) in [
            (RunUser, RUN_VALUE.to_string()),
            (RunUser, GHOST_RUN_VALUE.to_string()),
            (RunMachine, MACHINE_RUN_VALUE.to_string()),
            (FolderUser, SHORTCUT.to_string()),
            (FolderUser, GHOST_SHORTCUT.to_string()),
            (Task, format!(r"\{TASK_NAME}")),
            (Task, format!(r"\{MACHINE_TASK_NAME}")),
        ] {
            let _ = history.forget(&make_id(source, &name));
        }
    }

    fn remove_all() {
        // 관리자 권한 흔적 (권한이 없으면 조용히 건너뛴다)
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let _ = hklm.delete_subkey_all(machine_key());
        let _ = hklm.delete_subkey(format!(r"SOFTWARE\{PUBLISHER}"));
        let _ = std::fs::remove_dir_all(machine_dir());
        if let Ok(run) = hklm.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE | KEY_WOW64_64KEY) {
            let _ = run.delete_value(MACHINE_RUN_VALUE);
        }
        if let Ok(k) = hklm.open_subkey_with_flags(
            format!(r"{APPROVED_KEY}\Run"),
            KEY_SET_VALUE | KEY_WOW64_64KEY,
        ) {
            let _ = k.delete_value(MACHINE_RUN_VALUE);
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(UNINSTALL_KEY);
        for key in extra_keys() {
            let _ = hkcu.delete_subkey_all(&key);
        }
        let _ = hkcu.delete_subkey(format!(r"Software\{PUBLISHER}"));
        if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
            let _ = run.delete_value(RUN_VALUE);
            let _ = run.delete_value(GHOST_RUN_VALUE);
        }
        for (sub, name) in [
            ("Run", RUN_VALUE),
            ("StartupFolder", SHORTCUT),
            ("Run", GHOST_RUN_VALUE),
            ("StartupFolder", GHOST_SHORTCUT),
        ] {
            if let Ok(k) =
                hkcu.open_subkey_with_flags(format!(r"{APPROVED_KEY}\{sub}"), KEY_SET_VALUE)
            {
                let _ = k.delete_value(name);
            }
        }
        let _ = std::fs::remove_file(startup_shortcut());
        let _ = std::fs::remove_file(startup_folder().join(GHOST_SHORTCUT));
        forget_startup_history();
        unregister_task();
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
