//! Windows 구현: Run 키와 시작 폴더를 읽고, `StartupApproved` 값을 바꾼다.

use std::path::{Path, PathBuf};

use robin_core::InstalledApplication;
use robin_discovery::command::{executable_path, expand_env};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE,
    KEY_WOW64_32KEY, KEY_WOW64_64KEY, RegType,
};
use winreg::{HKEY, RegKey, RegValue};

use crate::{SetError, StartupItem, StartupSource, approved, make_id, owning_app, shell};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";

/// 시작 폴더 안에서 무시할 파일
const IGNORED_FILES: [&str; 1] = ["desktop.ini"];

impl StartupSource {
    /// Run 키: (하이브, 뷰, 화면에 보여 줄 경로)
    fn run_key(self) -> Option<(HKEY, u32, &'static str)> {
        Some(match self {
            StartupSource::RunUser => (
                HKEY_CURRENT_USER,
                0,
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            ),
            StartupSource::RunMachine => (
                HKEY_LOCAL_MACHINE,
                KEY_WOW64_64KEY,
                r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            ),
            StartupSource::RunMachine32 => (
                HKEY_LOCAL_MACHINE,
                KEY_WOW64_32KEY,
                r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
            ),
            _ => return None,
        })
    }

    fn folder(self) -> Option<PathBuf> {
        let (var, name) = match self {
            StartupSource::FolderUser => ("APPDATA", "Startup"),
            StartupSource::FolderCommon => ("ProgramData", "StartUp"),
            _ => return None,
        };
        let base = std::env::var_os(var)?;
        Some(
            PathBuf::from(base)
                .join(r"Microsoft\Windows\Start Menu\Programs")
                .join(name),
        )
    }

    /// 켜짐·꺼짐 표시가 저장되는 곳 (작업 관리자와 같은 위치)
    fn approved_key(self) -> (HKEY, String) {
        let (hive, sub) = match self {
            StartupSource::RunUser => (HKEY_CURRENT_USER, "Run"),
            StartupSource::RunMachine => (HKEY_LOCAL_MACHINE, "Run"),
            StartupSource::RunMachine32 => (HKEY_LOCAL_MACHINE, "Run32"),
            StartupSource::FolderUser => (HKEY_CURRENT_USER, "StartupFolder"),
            StartupSource::FolderCommon => (HKEY_LOCAL_MACHINE, "StartupFolder"),
        };
        (hive, format!(r"{APPROVED}\{sub}"))
    }
}

/// 항목 하나를 만드는 데 필요한 원본 정보
struct Raw {
    source: StartupSource,
    name: String,
    location: String,
    command: String,
    /// 환경 변수를 펼친 실행 파일 경로
    file: Option<String>,
}

pub fn list(installed: &[InstalledApplication]) -> Vec<StartupItem> {
    let mut items: Vec<StartupItem> = read_all()
        .into_iter()
        .map(|raw| to_item(raw, installed))
        .collect();
    items.sort_by_key(|i| i.name.to_lowercase());
    items
}

fn read_all() -> Vec<Raw> {
    StartupSource::ALL
        .into_iter()
        .flat_map(|source| {
            if source.run_key().is_some() {
                read_run(source)
            } else {
                read_folder(source)
            }
        })
        .collect()
}

fn read_run(source: StartupSource) -> Vec<Raw> {
    let Some((hive, view, location)) = source.run_key() else {
        return Vec::new();
    };
    let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(RUN, KEY_READ | view) else {
        return Vec::new();
    };
    key.enum_values()
        .filter_map(Result::ok)
        .filter(|(name, _)| !name.is_empty())
        .filter_map(|(name, value)| {
            // REG_SZ, REG_EXPAND_SZ만 명령으로 본다
            if !matches!(value.vtype, RegType::REG_SZ | RegType::REG_EXPAND_SZ) {
                return None;
            }
            let command = reg_string(&value);
            let expanded = expand_env(&command, |v| std::env::var(v).ok());
            Some(Raw {
                source,
                name,
                location: location.to_string(),
                file: executable_path(&expanded),
                command,
            })
        })
        .collect()
}

fn reg_string(value: &RegValue) -> String {
    let wide: Vec<u16> = value
        .bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&wide)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

fn read_folder(source: StartupSource) -> Vec<Raw> {
    let Some(dir) = source.folder() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if IGNORED_FILES.iter().any(|f| f.eq_ignore_ascii_case(&name)) {
                return None;
            }
            let path = e.path().to_string_lossy().into_owned();
            let is_link = name.to_lowercase().ends_with(".lnk");
            let (command, file) = match is_link.then(|| shell::resolve_link(&path)).flatten() {
                Some(link) if link.args.is_empty() => (link.target.clone(), Some(link.target)),
                Some(link) => (
                    format!("\"{}\" {}", link.target, link.args),
                    Some(link.target),
                ),
                // 바로 가기 대상을 알 수 없으면(MSI 광고 바로 가기 등) 파일 자체를 보여 준다
                None if is_link => (path.clone(), None),
                None => (path.clone(), Some(path.clone())),
            };
            Some(Raw {
                source,
                name,
                location: dir.to_string_lossy().into_owned(),
                command,
                file,
            })
        })
        .collect()
}

fn is_absolute(path: &str) -> bool {
    let b = path.as_bytes();
    path.starts_with(r"\\") || (b.len() >= 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'))
}

fn to_item(raw: Raw, installed: &[InstalledApplication]) -> StartupItem {
    let file = raw.file.as_deref().filter(|f| is_absolute(f));
    let exists = file.is_some_and(|f| Path::new(f).is_file());
    // 확장자가 없으면 인자까지 경로로 잘못 자른 것일 수 있어 "없음"으로 단정하지 않는다
    let file_missing = file.is_some_and(|f| !exists && Path::new(f).extension().is_some());
    let publisher = file.filter(|_| exists).and_then(shell::company_name);
    let app_name = file
        .and_then(|f| owning_app(f, installed))
        .map(|a| a.display_name.clone());

    let (hive, approved_path) = raw.source.approved_key();
    let state = RegKey::predef(hive)
        .open_subkey_with_flags(&approved_path, KEY_QUERY_VALUE | KEY_WOW64_64KEY)
        .and_then(|k| k.get_raw_value(&raw.name))
        .ok();

    StartupItem {
        id: make_id(raw.source, &raw.name),
        enabled: approved::is_enabled(state.as_ref().map(|v| v.bytes.as_slice())),
        needs_admin: raw.source.needs_admin(),
        file_path: raw.file.clone(),
        file_missing,
        publisher,
        app_name,
        name: raw.name,
        source: raw.source,
        location: raw.location,
        command: raw.command,
    }
}

pub fn set_enabled(id: &str, enabled: bool) -> Result<(), SetError> {
    if !crate::is_valid_id(id) {
        return Err(SetError::NotFound);
    }
    // 화면에서 받은 ID를 디스크에서 다시 읽은 목록과 맞춰 본다
    let raw = read_all()
        .into_iter()
        .find(|r| make_id(r.source, &r.name) == id)
        .ok_or(SetError::NotFound)?;

    let (hive, path) = raw.source.approved_key();
    let key = match RegKey::predef(hive)
        .create_subkey_with_flags(&path, KEY_QUERY_VALUE | KEY_SET_VALUE | KEY_WOW64_64KEY)
    {
        Ok((k, _)) => k,
        Err(e) if e.raw_os_error() == Some(5) => return Err(SetError::AccessDenied),
        Err(e) => return Err(SetError::Other(e.to_string())),
    };
    let previous = key.get_raw_value(&raw.name).ok();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let bytes = approved::encode(
        previous.as_ref().map(|v| v.bytes.as_slice()),
        enabled,
        approved::filetime_from_unix(now),
    );
    key.set_raw_value(
        &raw.name,
        &RegValue {
            bytes: bytes.to_vec(),
            vtype: RegType::REG_BINARY,
        },
    )
    .map_err(|e| match e.raw_os_error() {
        Some(5) => SetError::AccessDenied,
        _ => SetError::Other(e.to_string()),
    })
}
