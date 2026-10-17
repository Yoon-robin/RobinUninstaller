//! 절대 후보로 올리지 않는 경로와 레지스트리 키. 순수 로직.
//!
//! 점수가 아무리 높아도 여기 걸리면 제외한다. 잘못 지우면 Windows나 여러 프로그램이 망가지는 곳들이다.

/// 경로 비교용 정규화: 따옴표·끝 구분자 제거, `/` → `\`, 소문자.
pub fn normalize_path(path: &str) -> String {
    let p = path.trim().trim_matches('"').replace('/', "\\");
    let p = p.trim_end_matches('\\');
    // "C:" 만 남은 경우 루트로 본다
    let p = if p.len() == 2 && p.ends_with(':') {
        format!("{p}\\")
    } else {
        p.to_string()
    };
    p.to_lowercase()
}

/// `child`가 `parent`와 같거나 그 아래에 있는지 (정규화된 경로끼리).
pub fn is_within(child: &str, parent: &str) -> bool {
    let c = normalize_path(child);
    let p = normalize_path(parent);
    if p.is_empty() {
        return false;
    }
    c == p || c.starts_with(&format!("{}\\", p.trim_end_matches('\\')))
}

/// 이 PC의 기본 폴더 위치. 테스트에서 주입할 수 있도록 값으로 받는다.
#[derive(Debug, Clone, Default)]
pub struct KnownFolders {
    pub windows: String,
    pub program_files: Vec<String>,
    pub program_data: String,
    pub users_root: String,
    pub user_profile: String,
    pub app_data: String,
    pub local_app_data: String,
    /// 그 밖에 폴더 자체를 지우면 안 되는 곳 (바탕 화면, 문서, 시작 메뉴, Temp 등)
    pub others: Vec<String>,
}

impl KnownFolders {
    #[cfg(windows)]
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).unwrap_or_default();
        let user = env("USERPROFILE");
        let app_data = env("APPDATA");
        let program_data = env("ProgramData");
        let local = env("LOCALAPPDATA");
        let mut program_files = vec![
            env("ProgramFiles"),
            env("ProgramFiles(x86)"),
            env("ProgramW6432"),
        ];
        program_files.retain(|p| !p.is_empty());
        program_files.dedup();
        KnownFolders {
            windows: env("SystemRoot"),
            program_files,
            program_data: program_data.clone(),
            users_root: std::path::Path::new(&user)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            user_profile: user.clone(),
            app_data: app_data.clone(),
            local_app_data: local.clone(),
            others: vec![
                format!("{user}\\Desktop"),
                format!("{user}\\Documents"),
                format!("{user}\\Downloads"),
                format!("{user}\\Pictures"),
                format!("{user}\\Music"),
                format!("{user}\\Videos"),
                format!("{user}\\OneDrive"),
                format!("{local}\\Programs"),
                format!("{local}\\Temp"),
                format!("{local}\\Microsoft"),
                format!("{local}\\Packages"),
                format!("{app_data}\\Microsoft"),
                format!("{app_data}\\Microsoft\\Windows\\Start Menu\\Programs"),
                format!("{program_data}\\Microsoft"),
                format!("{program_data}\\Microsoft\\Windows\\Start Menu\\Programs"),
                format!("{program_data}\\Package Cache"),
                env("PUBLIC"),
                env("TEMP"),
            ],
        }
    }

    /// 사용자가 직접 만든 파일이 있는 폴더 (바탕 화면, 문서, 다운로드 등). 이 아래는 자동 선택하지 않는다.
    pub fn user_content_dirs(&self) -> impl Iterator<Item = &String> {
        const NAMES: [&str; 7] = [
            "desktop",
            "documents",
            "downloads",
            "pictures",
            "music",
            "videos",
            "onedrive",
        ];
        self.others.iter().filter(|p| {
            let last = normalize_path(p);
            let last = last.rsplit('\\').next().unwrap_or_default().to_string();
            NAMES.contains(&last.as_str())
        })
    }

    /// `path`가 사용자 파일 폴더(바탕 화면·문서 등) 안에 있는지.
    pub fn is_user_content(&self, path: &str) -> bool {
        self.user_content_dirs().any(|d| is_within(path, d))
    }

    fn exact_roots(&self) -> impl Iterator<Item = &String> {
        self.program_files
            .iter()
            .chain([
                &self.program_data,
                &self.users_root,
                &self.user_profile,
                &self.app_data,
                &self.local_app_data,
            ])
            .chain(self.others.iter())
            .filter(|p| !p.is_empty())
    }
}

/// 이름이 이 단어로 된 폴더(어디에 있든)는 여러 프로그램이 같이 쓰는 곳이라 통째로 다루지 않는다.
const SHARED_FOLDER_NAMES: &[&str] = &[
    "common files",
    "microsoft shared",
    "windows nt",
    "windowspowershell",
    "windowsapps",
    "windows defender",
    "windows mail",
    "windows media player",
    "windows photo viewer",
    "internet explorer",
    "microsoft.net",
    "dotnet",
    "reference assemblies",
    "msbuild",
    "installshield installation information",
    "package cache",
    "system volume information",
    "$recycle.bin",
    "crypto",
    "crypt",
    "temp",
    "tmp",
];

/// 정규화하면 다른 곳을 가리킬 수 있는 경로인지 (`..`, `.`, `\\?\`, 겹친 구분자, 끝 점·공백, 8.3 짧은 이름).
/// 이런 경로는 글자 비교로 보호 규칙을 피해 갈 수 있으므로 아예 다루지 않는다.
pub fn is_suspicious_path(path: &str) -> bool {
    // 끝 공백은 Windows가 조용히 잘라 다른 곳을 가리키므로 남겨 둔 채 검사한다
    let p = path.trim_start().trim_matches('"').replace('/', "\\");
    if p.starts_with(r"\\?\") || p.starts_with(r"\\.\") {
        return true;
    }
    // UNC(\\server\share)의 앞 두 글자 말고는 구분자가 겹치면 안 된다
    let body = p.strip_prefix(r"\\").unwrap_or(&p);
    if body.contains(r"\\") {
        return true;
    }
    body.trim_end_matches('\\')
        .split('\\')
        .enumerate()
        .any(|(i, seg)| {
            let drive = i == 0 && seg.len() == 2 && seg.ends_with(':');
            !drive
                && (seg == "."
                    || seg == ".."
                    || seg.ends_with('.')
                    || seg.ends_with(' ')
                    || seg
                        .find('~')
                        .is_some_and(|t| seg[t + 1..].starts_with(|c: char| c.is_ascii_digit())))
        })
}

/// 경로가 보호 대상이면 이유를 돌려준다.
pub fn protected_path_reason(path: &str, known: &KnownFolders) -> Option<&'static str> {
    if is_suspicious_path(path) {
        return Some("확인할 수 없는 형태의 경로");
    }
    let p = normalize_path(path);

    // 드라이브 루트 ("c:\")
    if p.len() <= 3 && p.chars().nth(1) == Some(':') {
        return Some("드라이브 루트");
    }
    if !known.windows.is_empty() && is_within(&p, &known.windows) {
        return Some("Windows 시스템 폴더");
    }
    if known.exact_roots().any(|root| normalize_path(root) == p) {
        return Some("Windows 기본 폴더");
    }
    let last = p.rsplit('\\').next().unwrap_or_default();
    if SHARED_FOLDER_NAMES.contains(&last) {
        return Some("여러 프로그램이 함께 쓰는 폴더");
    }
    // 공유 폴더의 하위도 다루지 않는다 (Common Files\Adobe 같은 곳은 다른 제품과 공유)
    if p.split('\\').any(|seg| SHARED_FOLDER_NAMES.contains(&seg)) {
        return Some("공유 폴더 아래");
    }
    None
}

/// 키 자체를 후보로 올리면 안 되는 레지스트리 경로 (하이브 이름 없이 `Software\...` 형태, 소문자 비교).
const PROTECTED_KEYS: &[&str] = &[
    "software",
    "software\\classes",
    "software\\microsoft",
    "software\\policies",
    "software\\wow6432node",
    "software\\wow6432node\\microsoft",
    "software\\wow6432node\\classes",
    "software\\microsoft\\windows",
    "software\\microsoft\\windows\\currentversion",
    "software\\microsoft\\windows\\currentversion\\uninstall",
    "software\\microsoft\\windows\\currentversion\\run",
    "software\\microsoft\\windows\\currentversion\\runonce",
    "software\\microsoft\\windows\\currentversion\\app paths",
    "software\\wow6432node\\microsoft\\windows",
    "software\\wow6432node\\microsoft\\windows\\currentversion",
    "software\\wow6432node\\microsoft\\windows\\currentversion\\uninstall",
    "software\\wow6432node\\microsoft\\windows\\currentversion\\run",
    "software\\wow6432node\\microsoft\\windows\\currentversion\\runonce",
    "software\\wow6432node\\microsoft\\windows\\currentversion\\app paths",
    "software\\wow6432node\\classes",
    "software\\wow6432node\\policies",
    "system",
    "system\\currentcontrolset",
    "system\\currentcontrolset\\services",
];

/// 그 아래 어떤 키도 이름만으로는 후보가 될 수 없는 영역 (Windows·Microsoft 자체 설정).
const PROTECTED_KEY_PREFIXES: &[&str] = &[
    "software\\microsoft\\",
    "software\\wow6432node\\microsoft\\",
    "software\\classes\\",
    "software\\policies\\",
    "software\\wow6432node\\classes\\",
    "software\\wow6432node\\policies\\",
];

/// `SYSTEM\CurrentControlSet\Services\X` 또는 `SYSTEM\ControlSet001\Services\X` 같은 서비스 키인지 (하이브 이름 제외).
pub fn is_service_key(key: &str) -> bool {
    let k = key.trim_matches('\\').to_lowercase();
    let mut parts = k.split('\\');
    parts.next() == Some("system")
        && parts.next().is_some_and(|cs| {
            cs == "currentcontrolset"
                || cs
                    .strip_prefix("controlset")
                    .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        })
        && parts.next() == Some("services")
        && parts.next().is_some()
}

/// 레지스트리 키(하이브 이름 제외, 예: `Software\Vendor\Product`)가 보호 대상이면 이유.
/// `by_name_only`: 이름만 보고 찾은 후보면 Microsoft·Classes 영역 전체를 막는다.
/// Uninstall·Run처럼 정확한 근거(경로·ID)로 찾은 항목은 false로 넘겨 그 아래 개별 키·값을 허용한다.
pub fn protected_key_reason(key: &str, by_name_only: bool) -> Option<&'static str> {
    let k = key.trim_matches('\\').to_lowercase();
    if k.contains("\\\\") || k.split('\\').any(|s| s.is_empty() || s == "." || s == "..") {
        return Some("확인할 수 없는 형태의 레지스트리 경로");
    }
    if PROTECTED_KEYS.contains(&k.as_str()) {
        return Some("Windows 기본 레지스트리 키");
    }
    if by_name_only && PROTECTED_KEY_PREFIXES.iter().any(|p| k.starts_with(p)) {
        return Some("Windows·Microsoft 설정 영역");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known() -> KnownFolders {
        KnownFolders {
            windows: r"C:\Windows".into(),
            program_files: vec![r"C:\Program Files".into(), r"C:\Program Files (x86)".into()],
            program_data: r"C:\ProgramData".into(),
            users_root: r"C:\Users".into(),
            user_profile: r"C:\Users\robin".into(),
            app_data: r"C:\Users\robin\AppData\Roaming".into(),
            local_app_data: r"C:\Users\robin\AppData\Local".into(),
            others: vec![r"C:\Users\robin\Documents".into()],
        }
    }

    #[test]
    fn blocks_roots_windows_and_known_folders() {
        let k = known();
        assert!(protected_path_reason(r"C:\", &k).is_some());
        assert!(protected_path_reason("D:", &k).is_some());
        assert!(protected_path_reason(r"C:\Windows\System32\drivers", &k).is_some());
        assert!(protected_path_reason(r"c:\program files\", &k).is_some());
        assert!(protected_path_reason(r"C:\Users\robin\AppData\Roaming", &k).is_some());
        assert!(protected_path_reason(r"C:\Users\robin\Documents", &k).is_some());
    }

    #[test]
    fn blocks_shared_folders_and_their_children() {
        let k = known();
        assert!(protected_path_reason(r"C:\Program Files\Common Files", &k).is_some());
        assert!(protected_path_reason(r"C:\Program Files (x86)\Common Files\Adobe", &k).is_some());
        assert!(protected_path_reason(r"C:\Program Files\WindowsApps\Foo", &k).is_some());
    }

    #[test]
    fn allows_normal_program_folders() {
        let k = known();
        assert_eq!(
            protected_path_reason(r"C:\Program Files\Fixture Notes", &k),
            None
        );
        assert_eq!(
            protected_path_reason(
                r"C:\Users\robin\AppData\Roaming\Robin Test Labs\Fixture Notes",
                &k
            ),
            None
        );
    }

    #[test]
    fn within_is_segment_aware() {
        assert!(is_within(r"C:\Apps\Foo\bin", r"C:\Apps\Foo"));
        assert!(is_within(r"C:\Apps\Foo", r"c:\apps\foo\"));
        assert!(!is_within(r"C:\Apps\FooBar", r"C:\Apps\Foo"));
    }

    #[test]
    fn registry_protection() {
        assert!(protected_key_reason(r"Software\Microsoft", false).is_some());
        assert!(
            protected_key_reason(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                false
            )
            .is_some()
        );
        assert!(protected_key_reason(r"Software\Microsoft\Office", true).is_some());
        assert_eq!(
            protected_key_reason(r"Software\Robin Test Labs\Fixture Notes", true),
            None
        );
    }

    #[test]
    fn suspicious_paths_are_protected() {
        let k = known();
        for p in [
            r"C:\\Program Files",
            r"C:\Program Files.",
            r"C:\PROGRA~1",
            r"\\?\C:\Program Files",
            r"C:\Program Files\.",
            r"C:\Program Files\..",
            r"C:\Program Files\X\..\..\Windows",
            r"C:\Windows.\System32",
            r"C:\Program Files\Acme ",
        ] {
            assert!(protected_path_reason(p, &k).is_some(), "{p}");
        }
        // 평범한 경로와 UNC는 그대로
        assert!(!is_suspicious_path(r"C:\Program Files\Acme\"));
        assert!(!is_suspicious_path(r"\\server\share\Acme"));
        assert!(!is_suspicious_path(r"C:\Apps\v1.2\a~b"));
    }

    #[test]
    fn user_content_and_services() {
        let k = known();
        assert!(k.is_user_content(r"C:\Users\robin\Documents\report.docx"));
        assert!(!k.is_user_content(r"C:\Users\robin\AppData\Roaming\Acme"));
        assert!(is_service_key(r"SYSTEM\ControlSet001\Services\Acme"));
        assert!(is_service_key(r"system\currentcontrolset\services\acme\"));
        assert!(!is_service_key(r"SYSTEM\ControlSet001\Services"));
        assert!(!is_service_key(r"SOFTWARE\Acme\Services\X"));
        assert!(
            protected_key_reason(
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
                false
            )
            .is_some()
        );
        assert!(protected_key_reason(r"Software\..\Microsoft", false).is_some());
    }
}
