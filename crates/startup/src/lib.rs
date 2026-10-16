//! 시작프로그램 목록과 켜기·끄기.
//!
//! 찾는 곳: Run 키(HKCU, HKLM 64/32비트)와 시작 폴더(사용자, 모든 사용자).
//! 끄기는 작업 관리자와 같은 방식으로 `Explorer\StartupApproved`에 "꺼짐" 표시만 남긴다.
//! Run 값이나 바로 가기 파일은 그대로 두므로, 다시 켜면 원래대로 돌아온다. 아무것도 지우지 않는다.

use robin_core::InstalledApplication;
use serde::Serialize;

pub mod approved;
#[cfg(windows)]
mod imp;
#[cfg(windows)]
mod shell;

/// 시작 항목이 등록된 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupSource {
    /// `HKCU\...\Run`
    RunUser,
    /// `HKLM\...\Run` (64비트 뷰)
    RunMachine,
    /// `HKLM\SOFTWARE\WOW6432Node\...\Run`
    RunMachine32,
    /// `%APPDATA%\...\Startup`
    FolderUser,
    /// `%ProgramData%\...\StartUp`
    FolderCommon,
}

impl StartupSource {
    pub const ALL: [StartupSource; 5] = [
        StartupSource::RunUser,
        StartupSource::RunMachine,
        StartupSource::RunMachine32,
        StartupSource::FolderUser,
        StartupSource::FolderCommon,
    ];

    fn id_prefix(self) -> &'static str {
        match self {
            StartupSource::RunUser => "run-user",
            StartupSource::RunMachine => "run-machine",
            StartupSource::RunMachine32 => "run-machine32",
            StartupSource::FolderUser => "folder-user",
            StartupSource::FolderCommon => "folder-common",
        }
    }

    /// 모든 사용자에게 적용되는 항목은 켜고 끄는 데 관리자 권한이 필요하다.
    pub fn needs_admin(self) -> bool {
        !matches!(self, StartupSource::RunUser | StartupSource::FolderUser)
    }
}

/// 시작 항목 하나.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    /// 출처 + 이름으로 만든 ID. 명령줄로 넘겨도 안전한 글자(소문자·숫자·`-`·`:`)만 쓴다.
    pub id: String,
    /// Run 값 이름 또는 시작 폴더의 파일 이름
    pub name: String,
    pub source: StartupSource,
    /// 레지스트리 키 또는 시작 폴더 경로
    pub location: String,
    /// Run 값에 적힌 명령. 바로 가기는 대상과 인자.
    pub command: String,
    /// 실행 파일 (환경 변수를 펼친 경로). 알아낼 수 없으면 None.
    pub file_path: Option<String>,
    /// 절대 경로인데 파일이 없음 → 이미 지워진 프로그램이 남긴 항목일 가능성이 크다
    pub file_missing: bool,
    /// 실행 파일의 버전 정보에 적힌 회사 이름
    pub publisher: Option<String>,
    /// 실행 파일이 설치 폴더 안에 있는 프로그램
    pub app_name: Option<String>,
    pub enabled: bool,
    pub needs_admin: bool,
}

#[derive(Debug)]
pub enum SetError {
    /// 목록에 없는 ID (그사이 지워졌거나 잘못된 요청)
    NotFound,
    AccessDenied,
    Other(String),
}

/// 시작 항목 목록. `installed`는 실행 파일이 어느 프로그램 것인지 알아내는 데 쓴다.
pub fn list(installed: &[InstalledApplication]) -> Vec<StartupItem> {
    #[cfg(windows)]
    return imp::list(installed);
    #[cfg(not(windows))]
    {
        let _ = installed;
        Vec::new()
    }
}

/// 항목을 켜거나 끈다. ID는 디스크에서 다시 읽은 목록에서 찾으므로, 화면에서 넘어온 경로는 쓰지 않는다.
pub fn set_enabled(id: &str, enabled: bool) -> Result<(), SetError> {
    #[cfg(windows)]
    return imp::set_enabled(id, enabled);
    #[cfg(not(windows))]
    {
        let _ = (id, enabled);
        Err(SetError::Other("Windows 전용입니다".into()))
    }
}

/// 출처와 이름으로 ID를 만든다. 이름은 대소문자를 구분하지 않으므로 소문자로 바꿔 16진수로 적는다.
pub fn make_id(source: StartupSource, name: &str) -> String {
    let hex: String = name
        .to_lowercase()
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("{}:{hex}", source.id_prefix())
}

/// 작업자 명령줄로 넘겨도 되는 ID인지 (경로·따옴표·공백 차단).
pub fn is_valid_id(id: &str) -> bool {
    let Some((prefix, hex)) = id.split_once(':') else {
        return false;
    };
    StartupSource::ALL.iter().any(|s| s.id_prefix() == prefix)
        && !hex.is_empty()
        && hex.len() <= 2048
        && hex.len() % 2 == 0
        && hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

/// 설치 폴더로 쓰기엔 너무 넓은 폴더. 여기를 설치 폴더로 적어 둔 프로그램은 짝짓지 않는다.
const GENERIC_DIRS: [&str; 9] = [
    "program files",
    "program files (x86)",
    "programdata",
    "windows",
    "users",
    "appdata\\local",
    "appdata\\roaming",
    "appdata\\local\\programs",
    "system32",
];

fn normalize_dir(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// 실행 파일이 설치 폴더 안에 있는 프로그램. 여러 개면 가장 깊은 설치 폴더를 고른다.
pub fn owning_app<'a>(
    file: &str,
    apps: &'a [InstalledApplication],
) -> Option<&'a InstalledApplication> {
    let file = normalize_dir(file);
    apps.iter()
        .filter_map(|app| {
            let dir = normalize_dir(app.install_location.as_deref()?);
            // 드라이브 루트(`c:`)나 공용 폴더 자체는 제외
            if dir.len() <= 3
                || GENERIC_DIRS
                    .iter()
                    .any(|g| dir.ends_with(&format!("\\{g}")))
            {
                return None;
            }
            let rest = file.strip_prefix(&dir)?;
            rest.starts_with('\\').then_some((dir.len(), app))
        })
        .max_by_key(|(len, _)| *len)
        .map(|(_, app)| app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::{InstallScope, InstallTechnology, UninstallerStatus};

    fn app(name: &str, location: Option<&str>) -> InstalledApplication {
        InstalledApplication {
            id: name.to_string(),
            display_name: name.to_string(),
            publisher: None,
            version: None,
            install_location: location.map(str::to_string),
            install_date: None,
            estimated_size_kb: None,
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Exe,
            scope: InstallScope::User,
            uninstaller: UninstallerStatus::Available,
        }
    }

    #[test]
    fn ids_are_safe_and_case_insensitive() {
        let id = make_id(StartupSource::RunUser, "My \"App\" ..\\x");
        assert!(is_valid_id(&id));
        assert!(
            id.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':')
        );
        assert_eq!(id, make_id(StartupSource::RunUser, "MY \"APP\" ..\\X"));
        assert_ne!(id, make_id(StartupSource::RunMachine, "My \"App\" ..\\x"));
    }

    #[test]
    fn rejects_bad_ids() {
        assert!(!is_valid_id("run-user:"));
        assert!(!is_valid_id("run-user:abc"));
        assert!(!is_valid_id("other:6162"));
        assert!(!is_valid_id("run-user:61 62"));
        assert!(!is_valid_id(r"run-user:..\x"));
    }

    #[test]
    fn matches_deepest_install_folder() {
        let apps = vec![
            app("Suite", Some(r"C:\Program Files\Acme")),
            app("Acme Sync", Some(r"C:\Program Files\Acme\Sync\")),
            app("Other", Some(r"C:\Program Files\Acme2")),
        ];
        let hit = owning_app(r"C:\Program Files\Acme\Sync\sync.exe", &apps).unwrap();
        assert_eq!(hit.display_name, "Acme Sync");
        let hit = owning_app(r"c:/program files/acme/tool.exe", &apps).unwrap();
        assert_eq!(hit.display_name, "Suite");
        // 이름이 앞부분만 같은 폴더는 다른 프로그램
        assert!(owning_app(r"C:\Program Files\Acme22\x.exe", &apps).is_none());
    }

    #[test]
    fn ignores_generic_install_folders() {
        let apps = vec![
            app("Bad", Some(r"C:\Program Files")),
            app("Root", Some(r"C:\")),
            app("Local", Some(r"C:\Users\me\AppData\Local")),
            app("None", None),
        ];
        assert!(owning_app(r"C:\Program Files\Foo\foo.exe", &apps).is_none());
        assert!(owning_app(r"C:\Users\me\AppData\Local\Foo\foo.exe", &apps).is_none());
    }
}
