//! 제거 기능을 실제 프로그램 없이 시험하기 위한 가짜 프로그램.
//!
//! 현재 사용자(HKCU) 영역에만 등록하므로 관리자 권한이 필요 없고 다른 프로그램에 영향을 주지 않는다.
//!
//! ```text
//! robin-fixture install [--cancel]   가짜 앱 "RobinUninstaller 테스트 앱"을 등록
//! robin-fixture remove               등록과 폴더를 바로 지움 (정리용)
//! ```
//!
//! 등록되는 언인스톨러는 NSIS처럼 동작한다: 자기 자신을 임시 폴더에 복사해 다시 실행하고 원본은 바로 끝난다.
//! 복사본이 몇 초 뒤 등록 정보와 폴더를 지운다. `--cancel`로 설치하면 아무것도 지우지 않고 끝난다(취소 흉내).

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
    use winreg::enums::HKEY_CURRENT_USER;

    const KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RobinFixtureApp";

    fn install_dir() -> PathBuf {
        PathBuf::from(std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA")).join("RobinFixtureApp")
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        match args.first().map(String::as_str) {
            Some("install") => install(args.iter().any(|a| a == "--cancel")),
            Some("remove") => remove_all(),
            Some("--uninstall") => stage1(args.iter().any(|a| a == "--cancel")),
            Some("--stage2") => stage2(Path::new(&args[1])),
            _ => {
                eprintln!("사용법: robin-fixture install [--cancel] | remove");
                exit(2);
            }
        }
    }

    fn install(cancel: bool) {
        let dir = install_dir();
        std::fs::create_dir_all(&dir).expect("폴더 생성");
        let uninstaller = dir.join("uninstall.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &uninstaller).expect("언인스톨러 복사");
        std::fs::write(dir.join("data.txt"), "RobinUninstaller fixture").unwrap();

        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(KEY)
            .expect("레지스트리 등록");
        let mut cmd = format!("\"{}\" --uninstall", uninstaller.display());
        if cancel {
            cmd.push_str(" --cancel");
        }
        key.set_value("DisplayName", &"RobinUninstaller 테스트 앱")
            .unwrap();
        key.set_value("Publisher", &"RobinUninstaller").unwrap();
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
    fn stage1(cancel: bool) {
        if cancel {
            std::thread::sleep(Duration::from_secs(1));
            exit(1602); // 사용자 취소
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

    fn stage2(dir: &Path) {
        std::thread::sleep(Duration::from_secs(3));
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(KEY);
        let _ = std::fs::remove_dir_all(dir);
        exit(0);
    }

    fn remove_all() {
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(KEY);
        let _ = std::fs::remove_dir_all(install_dir());
        let _ = std::fs::remove_file(std::env::temp_dir().join("robin_fixture_Au_.exe"));
        println!("정리함");
    }
}
