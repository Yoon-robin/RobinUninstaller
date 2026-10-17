//! 시작프로그램 모델과 순수 함수 (시스템 접근 없음).
//!
//! 켜짐·꺼짐 상태는 작업 관리자와 같은 곳에 둔다:
//! `...\Explorer\StartupApproved\{Run|Run32|StartupFolder}`의 바이너리 값(12바이트).
//! 첫 바이트가 홀수(03)면 꺼짐, 짝수(02·04·06)거나 값이 없으면 켜짐. 4~11바이트는 끈 시각(FILETIME).
//! Run 값이나 바로 가기 파일 자체는 건드리지 않으므로 끄기는 언제든 되돌릴 수 있다.

use serde::Serialize;

/// 시작 항목이 등록된 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupSource {
    /// `HKCU\...\Run`
    RunUser,
    /// `HKLM\...\Run` (64비트 뷰)
    RunMachine,
    /// `HKLM\SOFTWARE\WOW6432Node\...\Run`
    RunMachine32,
    /// `%APPDATA%\...\Start Menu\Programs\Startup`
    FolderUser,
    /// `%ProgramData%\...\Start Menu\Programs\StartUp`
    FolderCommon,
}

pub const ALL_SOURCES: [StartupSource; 5] = [
    StartupSource::RunUser,
    StartupSource::RunMachine,
    StartupSource::RunMachine32,
    StartupSource::FolderUser,
    StartupSource::FolderCommon,
];

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";

impl StartupSource {
    /// ID와 기록에 쓰는 짧은 이름
    pub fn key(self) -> &'static str {
        match self {
            StartupSource::RunUser => "run-user",
            StartupSource::RunMachine => "run-machine",
            StartupSource::RunMachine32 => "run-machine32",
            StartupSource::FolderUser => "folder-user",
            StartupSource::FolderCommon => "folder-common",
        }
    }

    /// HKLM이나 공용 폴더라 바꾸려면 관리자 권한이 필요한지
    pub fn is_machine(self) -> bool {
        !matches!(self, StartupSource::RunUser | StartupSource::FolderUser)
    }

    /// `StartupApproved` 아래 하위 키 이름
    pub fn approved_subkey(self) -> &'static str {
        match self {
            StartupSource::RunUser | StartupSource::RunMachine => "Run",
            StartupSource::RunMachine32 => "Run32",
            StartupSource::FolderUser | StartupSource::FolderCommon => "StartupFolder",
        }
    }
}

/// 시작 항목 하나.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    /// 등록 위치와 이름으로 만든 16자리 16진수 ID
    pub id: String,
    /// 화면에 보여 줄 이름 (Run 값 이름, 또는 확장자를 뺀 파일 이름)
    pub name: String,
    pub source: StartupSource,
    /// 레지스트리 키 경로 또는 시작 폴더 경로
    pub location: String,
    /// `StartupApproved`에 쓰이는 이름 (Run 값 이름, 또는 확장자까지 포함한 파일 이름)
    pub approved_name: String,
    /// 실행되는 명령 (바로 가기면 대상 + 인자)
    pub command: String,
    /// 실행 파일 경로 (해석할 수 있을 때)
    pub target_path: Option<String>,
    /// 실행 파일 경로를 알지만 그 파일이 없음
    pub target_missing: bool,
    /// 파일 버전 정보의 회사 이름
    pub publisher: Option<String>,
    /// 파일 버전 정보의 설명
    pub description: Option<String>,
    pub enabled: bool,
    /// 끈 시각 (유닉스 초). 기록돼 있을 때만.
    pub disabled_at: Option<u64>,
    /// RobinUninstaller가 끈 항목이라 원래 값으로 되돌릴 수 있음
    pub changed_by_us: bool,
}

/// 등록 위치와 이름으로 ID를 만든다. 레지스트리 값 이름과 파일 이름은 대소문자를 구분하지 않는다.
pub fn make_id(source: StartupSource, approved_name: &str) -> String {
    // FNV-1a 64비트: 실행할 때마다 같은 값이 나와야 해서 표준 해시(무작위 시드)를 쓰지 않는다.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let text = format!("{}\0{}", source.key(), approved_name.to_lowercase());
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// ID 형식 검사 (관리자 권한 작업자에 넘기기 전에).
pub fn is_valid_id(id: &str) -> bool {
    id.len() == 16 && id.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

/// 1601-01-01부터 1970-01-01까지의 100ns 단위 수
const FILETIME_UNIX_EPOCH: u64 = 116_444_736_000_000_000;

pub fn filetime_to_unix(filetime: u64) -> Option<u64> {
    filetime
        .checked_sub(FILETIME_UNIX_EPOCH)
        .filter(|_| filetime != 0)
        .map(|t| t / 10_000_000)
}

pub fn unix_to_filetime(seconds: u64) -> u64 {
    seconds * 10_000_000 + FILETIME_UNIX_EPOCH
}

/// `StartupApproved` 값 해석: (켜짐 여부, 끈 시각)
pub fn parse_approved(bytes: Option<&[u8]>) -> (bool, Option<u64>) {
    match bytes {
        Some([first, ..]) if first & 1 == 1 => {
            let at = bytes
                .and_then(|b| b.get(4..12))
                .and_then(|b| b.try_into().ok())
                .and_then(|b: [u8; 8]| filetime_to_unix(u64::from_le_bytes(b)));
            (false, at)
        }
        _ => (true, None),
    }
}

/// 작업 관리자가 쓰는 "꺼짐" 값: 03 00 00 00 + 끈 시각
pub fn disabled_value(now_unix: u64) -> Vec<u8> {
    let mut v = vec![3, 0, 0, 0];
    v.extend_from_slice(&unix_to_filetime(now_unix).to_le_bytes());
    v
}

/// 작업 관리자가 쓰는 "켜짐" 값: 02 + 0 열한 개
pub fn enabled_value() -> Vec<u8> {
    let mut v = vec![0; 12];
    v[0] = 2;
    v
}

/// 명령에서 실행 파일 경로를 찾는다. `rundll32.exe`처럼 경로 없이 이름만 있으면 Windows 폴더에서 찾는다.
pub fn resolve_target(
    command: &str,
    env: impl Fn(&str) -> Option<String>,
    exists: impl Fn(&str) -> bool,
) -> Option<String> {
    let expanded = robin_discovery::command::expand_env(command, &env);
    let path = robin_discovery::command::executable_path(&expanded)?;
    if path.contains('\\') || path.contains('/') {
        return Some(path);
    }
    let windir = env("SystemRoot").or_else(|| env("windir"))?;
    [
        format!(r"{windir}\System32\{path}"),
        format!(r"{windir}\{path}"),
    ]
    .into_iter()
    .find(|p| exists(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_values_from_task_manager() {
        // 이 PC에서 실제로 읽은 값들
        let off = [3, 0, 0, 0, 0x59, 0x84, 0xe8, 0x72, 0xea, 0xe5, 0xdc, 0x01];
        let (enabled, at) = parse_approved(Some(&off));
        assert!(!enabled);
        assert!(at.is_some_and(|t| t > 1_600_000_000));

        assert_eq!(parse_approved(Some(&enabled_value())), (true, None));
        assert_eq!(
            parse_approved(Some(&[4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
            (true, None)
        );
        assert_eq!(parse_approved(None), (true, None));
        assert_eq!(parse_approved(Some(&[])), (true, None));
        assert_eq!(parse_approved(Some(&[3])), (false, None));
    }

    #[test]
    fn disabled_value_round_trips() {
        let v = disabled_value(1_760_000_000);
        assert_eq!(v.len(), 12);
        assert_eq!(parse_approved(Some(&v)), (false, Some(1_760_000_000)));
    }

    #[test]
    fn ids_are_stable_and_case_insensitive() {
        let a = make_id(StartupSource::RunUser, "Steam");
        assert_eq!(a, make_id(StartupSource::RunUser, "STEAM"));
        assert_ne!(a, make_id(StartupSource::RunMachine, "Steam"));
        assert!(is_valid_id(&a));
        assert!(!is_valid_id(r"..\..\x"));
        assert!(!is_valid_id("ABCDEF0123456789"));
    }

    #[test]
    fn resolves_targets() {
        let env = |name: &str| match name {
            "SystemRoot" => Some(r"C:\WINDOWS".to_string()),
            "ProgramFiles" => Some(r"C:\Program Files".to_string()),
            _ => None,
        };
        let exists = |p: &str| p == r"C:\WINDOWS\System32\rundll32.exe";
        assert_eq!(
            resolve_target(r#""%ProgramFiles%\App\app.exe" --tray"#, env, exists).as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
        assert_eq!(
            resolve_target("rundll32.exe shell32.dll,Foo", env, exists).as_deref(),
            Some(r"C:\WINDOWS\System32\rundll32.exe")
        );
        assert_eq!(resolve_target("nothere.exe", env, exists), None);
    }
}
