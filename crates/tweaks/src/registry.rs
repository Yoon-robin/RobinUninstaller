//! 현재 사용자(HKCU) 레지스트리 읽기·쓰기. 이 모듈 밖에서는 레지스트리를 직접 만지지 않는다.

use std::io;

use winreg::enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE, RegType};
use winreg::{RegKey, RegValue};

use crate::model::{RawType, RawValue};

fn hkcu() -> RegKey {
    RegKey::predef(HKEY_CURRENT_USER)
}

fn to_raw_type(t: RegType) -> Option<RawType> {
    Some(match t {
        RegType::REG_NONE => RawType::None,
        RegType::REG_SZ => RawType::Sz,
        RegType::REG_EXPAND_SZ => RawType::ExpandSz,
        RegType::REG_BINARY => RawType::Binary,
        RegType::REG_DWORD => RawType::Dword,
        RegType::REG_MULTI_SZ => RawType::MultiSz,
        RegType::REG_QWORD => RawType::Qword,
        _ => return None,
    })
}

fn from_raw_type(t: RawType) -> RegType {
    match t {
        RawType::None => RegType::REG_NONE,
        RawType::Sz => RegType::REG_SZ,
        RawType::ExpandSz => RegType::REG_EXPAND_SZ,
        RawType::Binary => RegType::REG_BINARY,
        RawType::Dword => RegType::REG_DWORD,
        RawType::MultiSz => RegType::REG_MULTI_SZ,
        RawType::Qword => RegType::REG_QWORD,
    }
}

pub fn key_exists(key: &str) -> bool {
    hkcu().open_subkey_with_flags(key, KEY_QUERY_VALUE).is_ok()
}

/// 값을 읽는다. 키나 값이 없으면 `Ok(None)`. 다루지 않는 종류(REG_LINK 등)면 오류.
pub fn read(key: &str, name: &str) -> io::Result<Option<RawValue>> {
    let k = match hkcu().open_subkey_with_flags(key, KEY_QUERY_VALUE) {
        Ok(k) => k,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    match k.get_raw_value(name) {
        Ok(v) => {
            let vtype =
                to_raw_type(v.vtype).ok_or_else(|| io::Error::other("다루지 않는 값 종류"))?;
            Ok(Some(RawValue {
                vtype,
                bytes: v.bytes.to_vec(),
            }))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn write(key: &str, name: &str, value: &RawValue) -> io::Result<()> {
    let (k, _) = hkcu().create_subkey_with_flags(key, KEY_QUERY_VALUE | KEY_SET_VALUE)?;
    k.set_raw_value(
        name,
        &RegValue {
            bytes: value.bytes.clone(),
            vtype: from_raw_type(value.vtype),
        },
    )
}

/// 값을 지운다. 이미 없으면 그대로 성공.
pub fn delete(key: &str, name: &str) -> io::Result<()> {
    let k = match hkcu().open_subkey_with_flags(key, KEY_SET_VALUE) {
        Ok(k) => k,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    match k.delete_value(name) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// 키가 비어 있으면(값·하위 키 없음) 지운다. 우리가 만든 키를 되돌릴 때만 쓴다.
pub fn delete_key_if_empty(key: &str) -> io::Result<()> {
    let Ok(k) = hkcu().open_subkey_with_flags(key, KEY_READ) else {
        return Ok(());
    };
    let empty = k.enum_values().next().is_none() && k.enum_keys().next().is_none();
    drop(k);
    if !empty {
        return Ok(());
    }
    let (parent, leaf) = key.rsplit_once('\\').unwrap_or(("", key));
    let parent = if parent.is_empty() {
        hkcu()
    } else {
        hkcu().open_subkey_with_flags(parent, KEY_READ | KEY_SET_VALUE)?
    };
    match parent.delete_subkey(leaf) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}
