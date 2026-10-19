//! 시작프로그램 모델과 순수 함수 (시스템 접근 없음).
//!
//! 켜짐·꺼짐 상태는 작업 관리자와 같은 곳에 둔다:
//! `...\Explorer\StartupApproved\{Run|Run32|StartupFolder}`의 바이너리 값(12바이트).
//! 첫 바이트가 홀수(03)면 꺼짐, 짝수(02·04·06)거나 값이 없으면 켜짐. 4~11바이트는 끈 시각(FILETIME).
//! Run 값이나 바로 가기 파일 자체는 건드리지 않으므로 끄기는 언제든 되돌릴 수 있다.
//!
//! 작업 스케줄러의 작업(로그인·부팅 트리거)은 `StartupApproved`가 아니라 작업 자체의 `Enabled` 값만 바꾼다.

use serde::{Deserialize, Serialize};

/// 시작 항목이 등록된 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// 작업 스케줄러에서 로그인·부팅 때 실행되는 작업 (`\Microsoft\` 아래는 뺀다)
    Task,
}

/// `StartupApproved` 값으로 켜고 끄는 곳 (Run 키와 시작 폴더). 작업은 따로 읽는다.
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
            StartupSource::Task => "task",
        }
    }

    /// HKLM이나 공용 폴더라 바꾸려면 관리자 권한이 필요한지.
    /// 작업은 작업마다 달라서 `false`로 두고 항목의 `needs_admin`으로 판단한다.
    pub fn is_machine(self) -> bool {
        matches!(
            self,
            StartupSource::RunMachine | StartupSource::RunMachine32 | StartupSource::FolderCommon
        )
    }

    /// `StartupApproved` 아래 하위 키 이름
    pub fn approved_subkey(self) -> &'static str {
        match self {
            StartupSource::RunUser | StartupSource::RunMachine => "Run",
            StartupSource::RunMachine32 => "Run32",
            StartupSource::FolderUser | StartupSource::FolderCommon => "StartupFolder",
            // 작업은 StartupApproved를 쓰지 않는다
            StartupSource::Task => "",
        }
    }
}

/// 시작 항목 하나.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    /// 등록 위치와 이름으로 만든 16자리 16진수 ID
    pub id: String,
    /// 화면에 보여 줄 이름 (Run 값 이름, 또는 확장자를 뺀 파일 이름)
    pub name: String,
    pub source: StartupSource,
    /// 레지스트리 키 경로, 시작 폴더 경로, 또는 "작업 스케줄러"
    pub location: String,
    /// `StartupApproved`에 쓰이는 이름 (Run 값 이름, 또는 확장자까지 포함한 파일 이름).
    /// 작업은 `\Vendor\Updater` 같은 작업 경로.
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
    /// 작업을 만든 이 (작업 스케줄러의 "만든 이"). 작업에만 있다.
    pub author: Option<String>,
    /// 바꾸려면 관리자 권한(UAC)이 필요해 보이는지
    pub needs_admin: bool,
    pub enabled: bool,
    /// 끈 시각 (유닉스 초). 기록돼 있을 때만.
    pub disabled_at: Option<u64>,
    /// RobinX가 끈 항목이라 원래 값으로 되돌릴 수 있음
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

/// Windows 자체 작업 폴더(`\Microsoft\...`)인지. 이 폴더는 목록에 넣지 않는다.
pub fn is_windows_task_folder(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower == r"\microsoft" || lower.starts_with(r"\microsoft\")
}

/// 작업 동작의 실행 파일과 인자를 한 줄 명령으로 합친다 (Run 값과 같은 모양으로).
pub fn task_command(path: &str, args: &str) -> String {
    let path = path.trim().trim_matches('"');
    format!("\"{path}\" {}", args.trim()).trim_end().to_string()
}

/// 지금 로그인한 사용자. 작업이 이 사용자로 실행되는지 볼 때 쓴다.
pub struct CurrentUser {
    pub name: String,
    pub domain: String,
}

impl CurrentUser {
    pub fn from_env() -> Self {
        CurrentUser {
            name: std::env::var("USERNAME").unwrap_or_default(),
            domain: std::env::var("USERDOMAIN").unwrap_or_default(),
        }
    }

    /// 작업의 실행 사용자(`UserId`)가 이 사용자인지. SID나 SYSTEM 같은 다른 계정이면 `false`.
    /// 이 사용자로 실행되는 작업은 보통 이 사용자가 만든 것이라 관리자 권한 없이 바꿀 수 있다.
    pub fn owns_task(&self, user_id: &str) -> bool {
        if self.name.is_empty() {
            return false;
        }
        match user_id.rsplit_once('\\') {
            Some((domain, name)) => {
                name.eq_ignore_ascii_case(&self.name)
                    && (domain.eq_ignore_ascii_case(&self.domain) || domain == ".")
            }
            None => user_id.eq_ignore_ascii_case(&self.name),
        }
    }
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

/// 실행 파일이 없는 시작 항목을 정리할 때 백업·격리할 대상.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupTarget {
    /// 시작 폴더의 바로 가기(또는 파일)
    File(String),
    /// Run 값 또는 `StartupApproved` 값 (`key`는 `HKCU\...` 형식)
    RegistryValue { key: String, name: String },
}

/// 정리하지 않는 이유
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupRefusal {
    /// 실행 파일이 있거나 알 수 없음 → 끄기만 한다
    TargetPresent,
    /// 예약 작업은 정리하지 않는다 (끄기만 한다)
    Task,
}

/// 실행 파일이 없는 시작 항목에서 지울 것: Run 값이나 시작 폴더 파일, 그리고 짝인 `StartupApproved` 값(있으면).
/// 실제로 지우는 일은 `crates/recovery`가 백업한 뒤에 한다. 순수 함수.
pub fn cleanup_targets(
    entry: &StartupEntry,
    approved_location: &str,
    has_approved: bool,
) -> Result<Vec<CleanupTarget>, CleanupRefusal> {
    if entry.source == StartupSource::Task {
        return Err(CleanupRefusal::Task);
    }
    if !entry.target_missing {
        return Err(CleanupRefusal::TargetPresent);
    }
    let mut targets = vec![match entry.source {
        StartupSource::FolderUser | StartupSource::FolderCommon => CleanupTarget::File(format!(
            r"{}\{}",
            entry.location.trim_end_matches('\\'),
            entry.approved_name
        )),
        _ => CleanupTarget::RegistryValue {
            key: entry.location.clone(),
            name: entry.approved_name.clone(),
        },
    }];
    if has_approved {
        targets.push(CleanupTarget::RegistryValue {
            key: approved_location.to_string(),
            name: entry.approved_name.clone(),
        });
    }
    Ok(targets)
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
    fn task_helpers() {
        assert!(is_windows_task_folder(r"\Microsoft"));
        assert!(is_windows_task_folder(r"\Microsoft\Windows\Defrag"));
        assert!(!is_windows_task_folder(r"\MicrosoftEdgeUpdate"));
        assert!(!is_windows_task_folder(r"\Vendor"));

        assert_eq!(
            task_command(r#""C:\App\up.exe""#, " /silent "),
            r#""C:\App\up.exe" /silent"#
        );
        assert_eq!(task_command(r"C:\App\up.exe", ""), r#""C:\App\up.exe""#);

        let me = CurrentUser {
            name: "Blitz".into(),
            domain: "DESKTOP-1".into(),
        };
        assert!(me.owns_task(r"DESKTOP-1\blitz"));
        assert!(me.owns_task("Blitz"));
        assert!(!me.owns_task(r"OTHER\Blitz"));
        assert!(!me.owns_task("SYSTEM"));
        assert!(!me.owns_task("S-1-5-18"));
        assert!(!me.owns_task(""));
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

    fn entry(source: StartupSource, missing: bool) -> StartupEntry {
        let (location, approved_name) = match source {
            StartupSource::FolderUser => (
                r"C:\Users\me\Startup\".to_string(),
                "Old Tool.lnk".to_string(),
            ),
            _ => (
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".to_string(),
                "OldTool".to_string(),
            ),
        };
        StartupEntry {
            id: make_id(source, &approved_name),
            name: "Old Tool".into(),
            source,
            location,
            approved_name,
            command: String::new(),
            target_path: Some(r"C:\Gone\tool.exe".into()),
            target_missing: missing,
            publisher: None,
            description: None,
            author: None,
            needs_admin: false,
            enabled: true,
            disabled_at: None,
            changed_by_us: false,
        }
    }

    #[test]
    fn cleanup_targets_for_missing_entries() {
        let approved =
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
        let run = entry(StartupSource::RunUser, true);
        assert_eq!(
            cleanup_targets(&run, approved, true).unwrap(),
            vec![
                CleanupTarget::RegistryValue {
                    key: run.location.clone(),
                    name: "OldTool".into()
                },
                CleanupTarget::RegistryValue {
                    key: approved.into(),
                    name: "OldTool".into()
                },
            ]
        );
        let link = entry(StartupSource::FolderUser, true);
        assert_eq!(
            cleanup_targets(&link, approved, false).unwrap(),
            vec![CleanupTarget::File(
                r"C:\Users\me\Startup\Old Tool.lnk".into()
            )]
        );
    }

    #[test]
    fn cleanup_refuses_present_targets_and_tasks() {
        assert_eq!(
            cleanup_targets(&entry(StartupSource::RunUser, false), "", true),
            Err(CleanupRefusal::TargetPresent)
        );
        assert_eq!(
            cleanup_targets(&entry(StartupSource::Task, true), "", true),
            Err(CleanupRefusal::Task)
        );
    }
}
