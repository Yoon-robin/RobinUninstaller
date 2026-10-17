//! Run 키 읽기와 `StartupApproved` 값 읽기·쓰기.

use std::io;

use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE,
    KEY_WOW64_32KEY, KEY_WOW64_64KEY, RegType,
};
use winreg::{RegKey, RegValue};

use crate::model::{APPROVED_KEY, RUN_KEY, StartupSource};

fn root(source: StartupSource) -> RegKey {
    if source.is_machine() {
        RegKey::predef(HKEY_LOCAL_MACHINE)
    } else {
        RegKey::predef(HKEY_CURRENT_USER)
    }
}

/// Run 키 위치 (화면 표시용)
pub fn run_location(source: StartupSource) -> String {
    match source {
        StartupSource::RunMachine => format!(r"HKLM\{RUN_KEY}"),
        StartupSource::RunMachine32 => {
            format!(
                r"HKLM\Software\WOW6432Node\{}",
                &RUN_KEY["Software\\".len()..]
            )
        }
        _ => format!(r"HKCU\{RUN_KEY}"),
    }
}

/// Run 키의 (값 이름, 명령) 목록. 이름 없는 기본값은 뺀다.
pub fn run_values(source: StartupSource) -> Vec<(String, String)> {
    let view = match source {
        StartupSource::RunMachine32 => KEY_WOW64_32KEY,
        _ => KEY_WOW64_64KEY,
    };
    let Ok(key) = root(source).open_subkey_with_flags(RUN_KEY, KEY_READ | view) else {
        return Vec::new();
    };
    key.enum_values()
        .flatten()
        .filter(|(name, _)| !name.is_empty())
        .filter_map(|(name, _)| {
            // REG_SZ와 REG_EXPAND_SZ 모두 문자열로 읽힌다 (환경 변수는 펼치지 않은 채로)
            let command: String = key.get_value(&name).ok()?;
            Some((name, command))
        })
        .collect()
}

fn approved_path(source: StartupSource) -> String {
    format!(r"{APPROVED_KEY}\{}", source.approved_subkey())
}

/// `StartupApproved` 값. 없으면 `None`.
pub fn read_approved(source: StartupSource, name: &str) -> Option<Vec<u8>> {
    root(source)
        .open_subkey_with_flags(approved_path(source), KEY_QUERY_VALUE | KEY_WOW64_64KEY)
        .ok()?
        .get_raw_value(name)
        .ok()
        .map(|v| v.bytes)
}

/// `StartupApproved` 키의 값 이름 목록.
pub fn approved_names(source: StartupSource) -> Vec<String> {
    let Ok(key) =
        root(source).open_subkey_with_flags(approved_path(source), KEY_READ | KEY_WOW64_64KEY)
    else {
        return Vec::new();
    };
    key.enum_values()
        .flatten()
        .map(|(name, _)| name)
        .filter(|name| !name.is_empty())
        .collect()
}

/// `StartupApproved` 값을 쓴다. `None`이면 값을 지운다 (원래 값이 없었던 경우).
pub fn write_approved(source: StartupSource, name: &str, value: Option<&[u8]>) -> io::Result<()> {
    let (key, _) = root(source).create_subkey_with_flags(
        approved_path(source),
        KEY_QUERY_VALUE | KEY_SET_VALUE | KEY_WOW64_64KEY,
    )?;
    match value {
        Some(bytes) => key.set_raw_value(
            name,
            &RegValue {
                bytes: bytes.to_vec(),
                vtype: RegType::REG_BINARY,
            },
        ),
        None => match key.delete_value(name) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        },
    }
}
