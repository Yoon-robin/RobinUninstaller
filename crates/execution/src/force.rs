//! 강제 제거(언인스톨러가 없는 프로그램) 전에 확인할 것.
//!
//! 강제 제거는 잔여물 정리와 같은 백업·격리 흐름으로 등록 정보와 파일을 치운다. 그 전에 두 가지를 본다.
//! - 보안 프로그램·안티치트·VPN·드라이버처럼 서비스나 커널 구성 요소가 남는 프로그램은 대상에서 뺀다.
//!   이런 프로그램은 제조사의 제거 도구를 써야 한다.
//! - 설치 폴더 안의 프로그램이 실행 중이면 파일이 잠겨 있으므로, 사용자가 닫을 때까지 진행하지 않는다.
//!   프로세스를 대신 강제 종료하지는 않는다.

use robin_core::InstalledApplication;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcess {
    pub pid: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceCheck {
    /// 강제 제거하면 안 되는 이유. 있으면 진행하지 않는다.
    pub blocked: Option<String>,
    /// 설치 폴더 안에서 실행 중인 프로세스
    pub running: Vec<RunningProcess>,
}

/// 이름·게시자에 이 말이 들어 있으면 강제 제거하지 않는다 (소문자로 비교).
const SENSITIVE_NAMES: [&str; 22] = [
    "antivirus",
    "anti-virus",
    "anti-malware",
    "antimalware",
    "internet security",
    "total security",
    "endpoint",
    "defender",
    "firewall",
    "anti-cheat",
    "anticheat",
    "easyanticheat",
    "battleye",
    "vanguard",
    "vpn",
    "wireguard",
    "tap-windows",
    "driver",
    "chipset",
    "알약",
    "백신",
    "보안",
];

const SENSITIVE_PUBLISHERS: [&str; 22] = [
    "kaspersky",
    "eset",
    "avast",
    "avg technologies",
    "norton",
    "gen digital",
    "mcafee",
    "bitdefender",
    "malwarebytes",
    "sophos",
    "trend micro",
    "ahnlab",
    "crowdstrike",
    "sentinelone",
    "f-secure",
    "panda security",
    "webroot",
    "comodo",
    "inca internet",
    "raonsecure",
    "wizvera",
    "nprotect",
];

/// 강제 제거 대상에서 빼야 하는 프로그램이면 그 이유. 순수 함수.
pub fn sensitive_reason(app: &InstalledApplication) -> Option<String> {
    let name = app.display_name.to_lowercase();
    let publisher = app.publisher.as_deref().unwrap_or_default().to_lowercase();
    let hit = SENSITIVE_NAMES
        .iter()
        .find(|w| name.contains(*w))
        .or_else(|| {
            SENSITIVE_PUBLISHERS
                .iter()
                .find(|w| contains_word(&publisher, w))
        });
    hit.map(|_| {
        "보안 프로그램·안티치트·VPN·드라이버로 보여요. 서비스나 드라이버가 남을 수 있어 강제 제거하지 않아요. \
         제조사가 제공하는 제거 도구를 써 주세요."
            .to_string()
    })
}

/// `word`가 단어 경계로 들어 있는지 ("eset"이 "Preset"에 걸리지 않게).
fn contains_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

fn normalize(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// `file`이 `dir` 안에 있는지 (대소문자 무시, 이름 앞부분만 같은 폴더는 제외).
pub fn is_inside(file: &str, dir: &str) -> bool {
    let (file, dir) = (normalize(file), normalize(dir));
    dir.len() > 3
        && file
            .strip_prefix(&dir)
            .is_some_and(|rest| rest.starts_with('\\'))
}

/// MSIX(패키지) 앱 안에서 설치된 프로그램은 실제 파일이 `%LOCALAPPDATA%\Packages\<패키지>\LocalCache\Local\...`
/// (또는 `...\LocalCache\Roaming\...`)에 있지만, 등록 정보에는 `%LOCALAPPDATA%\...`(`%APPDATA%\...`)로 적힌다.
/// 실행 중인 프로세스의 실제 경로를 등록 정보와 같은 모양으로 바꾼다. 해당하지 않으면 `None`.
pub fn devirtualize(path: &str, local_appdata: &str, appdata: &str) -> Option<String> {
    let packages = format!("{}\\packages\\", normalize(local_appdata));
    let lower = normalize(path);
    let rest = lower.strip_prefix(&packages)?;
    let (_package, rest) = rest.split_once('\\')?;
    let rest = rest.strip_prefix("localcache\\")?;
    if let Some(rest) = rest.strip_prefix("local\\") {
        Some(format!("{}\\{rest}", normalize(local_appdata)))
    } else {
        rest.strip_prefix("roaming\\")
            .map(|rest| format!("{}\\{rest}", normalize(appdata)))
    }
}

/// 실행 파일 `path`가 설치 폴더 `dir` 안에 있는지. 패키지 앱의 가상화 경로도 같은 위치로 본다.
fn runs_from(path: &str, dir: &str) -> bool {
    is_inside(path, dir)
        || match (std::env::var("LOCALAPPDATA"), std::env::var("APPDATA")) {
            (Ok(local), Ok(roaming)) => {
                devirtualize(path, &local, &roaming).is_some_and(|p| is_inside(&p, dir))
            }
            _ => false,
        }
}

/// 강제 제거 전 확인.
pub fn check(app: &InstalledApplication) -> ForceCheck {
    let running = match app.install_location.as_deref().map(str::trim) {
        Some(dir) if !dir.is_empty() => running_in(dir),
        _ => Vec::new(),
    };
    ForceCheck {
        blocked: sensitive_reason(app),
        running,
    }
}

#[cfg(windows)]
fn running_in(dir: &str) -> Vec<RunningProcess> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::core::PWSTR;

    let me = std::process::id();
    let mut out: Vec<RunningProcess> = crate::track::snapshot_pids()
        .into_iter()
        .filter(|&pid| pid != 0 && pid != me)
        .filter_map(|pid| unsafe {
            // 권한이 부족한 프로세스(다른 사용자, 보호 프로세스)는 열리지 않는다 → 건너뜀
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(handle);
            ok.ok()?;
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            runs_from(&path, dir).then_some(RunningProcess { pid, path })
        })
        .collect();
    out.sort_by_key(|p| p.pid);
    out
}

#[cfg(not(windows))]
fn running_in(_dir: &str) -> Vec<RunningProcess> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::{InstallScope, InstallTechnology, UninstallerStatus};

    fn app(name: &str, publisher: Option<&str>) -> InstalledApplication {
        InstalledApplication {
            id: name.into(),
            display_name: name.into(),
            publisher: publisher.map(Into::into),
            version: None,
            install_location: None,
            install_date: None,
            estimated_size_kb: None,
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Exe,
            scope: InstallScope::User,
            uninstaller: UninstallerStatus::Missing,
        }
    }

    #[test]
    fn blocks_security_vpn_and_drivers() {
        assert!(sensitive_reason(&app("ESET Security", Some("ESET, spol. s r.o."))).is_some());
        assert!(sensitive_reason(&app("Riot Vanguard", Some("Riot Games, Inc."))).is_some());
        assert!(sensitive_reason(&app("WireGuard", None)).is_some());
        assert!(sensitive_reason(&app("Realtek Audio Driver", None)).is_some());
        assert!(sensitive_reason(&app("AhnLab Safe Transaction", Some("AhnLab, Inc."))).is_some());
        assert!(sensitive_reason(&app("알약", Some("ESTsoft"))).is_some());
    }

    #[test]
    fn allows_ordinary_apps() {
        assert!(sensitive_reason(&app("Fixture Notes", Some("Robin Test Labs"))).is_none());
        assert!(sensitive_reason(&app("League of Legends", Some("Riot Games, Inc."))).is_none());
        // 게시자 이름 일부가 겹쳐도 단어가 다르면 막지 않는다
        assert!(sensitive_reason(&app("Preset Studio", Some("Preset Labs"))).is_none());
        assert!(sensitive_reason(&app("Notepad++", Some("Don HO"))).is_none());
    }

    #[test]
    fn inside_checks_folder_boundary() {
        assert!(is_inside(r"C:\Apps\Foo\bin\foo.exe", r"c:\apps\foo\"));
        assert!(!is_inside(r"C:\Apps\Foo2\foo.exe", r"C:\Apps\Foo"));
        assert!(!is_inside(r"C:\foo.exe", r"C:\"));
    }

    #[test]
    fn devirtualizes_package_local_cache_paths() {
        let (local, roaming) = (r"C:\Users\me\AppData\Local", r"C:\Users\me\AppData\Roaming");
        let real =
            r"C:\Users\me\AppData\Local\Packages\Some.App_abc\LocalCache\Local\Acme\Tool\tool.exe";
        let shown = devirtualize(real, local, roaming).unwrap();
        assert!(is_inside(&shown, r"C:\Users\me\AppData\Local\Acme\Tool"));
        let roam = r"C:\Users\me\AppData\Local\Packages\Some.App_abc\LocalCache\Roaming\Acme\a.exe";
        assert!(is_inside(
            &devirtualize(roam, local, roaming).unwrap(),
            r"C:\Users\me\AppData\Roaming\Acme"
        ));
        // 패키지 자신의 파일이나 다른 위치는 바꾸지 않는다
        assert!(
            devirtualize(
                r"C:\Users\me\AppData\Local\Packages\Some.App_abc\AC\x.exe",
                local,
                roaming
            )
            .is_none()
        );
        assert!(devirtualize(r"C:\Program Files\Acme\a.exe", local, roaming).is_none());
    }
}
