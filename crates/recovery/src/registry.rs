//! 레지스트리 키·값 백업, 삭제, 복원.
//!
//! 값은 형식(REG_SZ, REG_DWORD, REG_BINARY …)과 원본 바이트를 그대로 base64로 저장해, 복원하면 똑같이 돌아온다.
//! HKLM은 64비트 뷰로 열어 `WOW6432Node`가 들어간 경로도 글자 그대로 다룬다(자동 리디렉션 없음).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde::{Deserialize, Serialize};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, KEY_ENUMERATE_SUB_KEYS, KEY_QUERY_VALUE,
    KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY, RegType,
};

/// 표준 권한 DELETE (winreg에 상수가 없어 직접 정의)
const DELETE: u32 = 0x0001_0000;
/// 키를 지우는 데 필요한 권한만. 권한 변경(WRITE_DAC)·소유권(WRITE_OWNER)까지 달라고 하지 않는다.
const DELETE_ACCESS: u32 = DELETE | KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE | KEY_SET_VALUE;
use winreg::{RegKey, RegValue};

/// 너무 큰 키를 통째로 지우는 실수를 막기 위한 상한.
const MAX_ENTRIES: usize = 20_000;
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValueDump {
    pub name: String,
    pub vtype: u32,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyDump {
    pub name: String,
    pub values: Vec<ValueDump>,
    pub subkeys: Vec<KeyDump>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RegistryBackup {
    Key { location: String, dump: KeyDump },
    Value { location: String, value: ValueDump },
}

#[derive(Debug)]
pub enum RegError {
    NotFound,
    /// 어느 단계에서 거부됐는지 (진단용, 비어 있을 수 있음)
    AccessDenied(String),
    TooLarge,
    Conflict(String),
    Other(String),
}

impl From<std::io::Error> for RegError {
    fn from(e: std::io::Error) -> Self {
        match e.raw_os_error() {
            Some(2) => RegError::NotFound,
            Some(5) => RegError::AccessDenied(String::new()),
            _ => RegError::Other(e.to_string()),
        }
    }
}

/// `HKCU\Software\X` → (HKCU 루트, `Software\X`). HKCU·HKLM만 허용.
pub fn split_location(location: &str) -> Option<(RegKey, &str)> {
    let (hive, path) = location.split_once('\\')?;
    if path.is_empty() {
        return None;
    }
    match hive.to_ascii_uppercase().as_str() {
        "HKCU" | "HKEY_CURRENT_USER" => Some((RegKey::predef(HKEY_CURRENT_USER), path)),
        "HKLM" | "HKEY_LOCAL_MACHINE" => Some((RegKey::predef(HKEY_LOCAL_MACHINE), path)),
        _ => None,
    }
}

fn open(root: &RegKey, path: &str, access: u32) -> Result<RegKey, RegError> {
    Ok(root.open_subkey_with_flags(path, access | KEY_WOW64_64KEY)?)
}

fn dump_value(name: String, v: &RegValue) -> ValueDump {
    ValueDump {
        name,
        vtype: v.vtype.clone() as u32,
        data: B64.encode(&v.bytes),
    }
}

fn to_reg_value(d: &ValueDump) -> Result<RegValue, RegError> {
    let bytes = B64
        .decode(&d.data)
        .map_err(|e| RegError::Other(e.to_string()))?;
    let vtype = reg_type(d.vtype)
        .ok_or_else(|| RegError::Other(format!("알 수 없는 값 형식 {}", d.vtype)))?;
    Ok(RegValue { bytes, vtype })
}

fn reg_type(n: u32) -> Option<RegType> {
    use RegType::*;
    Some(match n {
        0 => REG_NONE,
        1 => REG_SZ,
        2 => REG_EXPAND_SZ,
        3 => REG_BINARY,
        4 => REG_DWORD,
        5 => REG_DWORD_BIG_ENDIAN,
        6 => REG_LINK,
        7 => REG_MULTI_SZ,
        8 => REG_RESOURCE_LIST,
        9 => REG_FULL_RESOURCE_DESCRIPTOR,
        10 => REG_RESOURCE_REQUIREMENTS_LIST,
        11 => REG_QWORD,
        _ => return None,
    })
}

fn dump_key(
    key: &RegKey,
    name: String,
    depth: usize,
    budget: &mut usize,
) -> Result<KeyDump, RegError> {
    if depth > MAX_DEPTH {
        return Err(RegError::TooLarge);
    }
    let mut values = Vec::new();
    for v in key.enum_values() {
        let (n, val) = v?;
        *budget = budget.checked_sub(1).ok_or(RegError::TooLarge)?;
        values.push(dump_value(n, &val));
    }
    let mut subkeys = Vec::new();
    for sub in key.enum_keys() {
        let sub = sub?;
        *budget = budget.checked_sub(1).ok_or(RegError::TooLarge)?;
        let child = key.open_subkey_with_flags(&sub, KEY_READ | KEY_WOW64_64KEY)?;
        subkeys.push(dump_key(&child, sub, depth + 1, budget)?);
    }
    Ok(KeyDump {
        name,
        values,
        subkeys,
    })
}

/// 키 전체를 읽어 둔다. 지우기 전에 반드시 이걸 먼저 저장한다.
pub fn backup_key(location: &str) -> Result<RegistryBackup, RegError> {
    let (root, path) =
        split_location(location).ok_or(RegError::Other("지원하지 않는 레지스트리 위치".into()))?;
    let key = open(&root, path, KEY_READ)?;
    let mut budget = MAX_ENTRIES;
    let name = path.rsplit('\\').next().unwrap_or(path).to_string();
    Ok(RegistryBackup::Key {
        location: location.to_string(),
        dump: dump_key(&key, name, 0, &mut budget)?,
    })
}

pub fn backup_value(location: &str, value_name: &str) -> Result<RegistryBackup, RegError> {
    let (root, path) =
        split_location(location).ok_or(RegError::Other("지원하지 않는 레지스트리 위치".into()))?;
    let key = open(&root, path, KEY_READ)?;
    let v = key.get_raw_value(value_name)?;
    Ok(RegistryBackup::Value {
        location: location.to_string(),
        value: dump_value(value_name.to_string(), &v),
    })
}

/// 실패한 단계와 Windows 오류 코드를 붙인다 (예: "부모 키 열기, 오류 5").
fn at(stage: &str) -> impl Fn(std::io::Error) -> RegError + '_ {
    move |e| {
        let code = e.raw_os_error().unwrap_or(-1);
        match RegError::from(e) {
            RegError::AccessDenied(_) => RegError::AccessDenied(format!("{stage}, 오류 {code}")),
            RegError::Other(m) => RegError::Other(format!("{stage}: {m}")),
            other => other,
        }
    }
}

/// 하위 키부터 하나씩 지운다. RegDeleteTree가 거부될 때의 대체 방법.
fn delete_tree_manually(parent: &RegKey, leaf: &str) -> Result<(), RegError> {
    let key = parent
        .open_subkey_with_flags(leaf, DELETE_ACCESS | KEY_WOW64_64KEY)
        .map_err(at("하위 키 열기"))?;
    let children: Vec<String> = key
        .enum_keys()
        .collect::<Result<_, _>>()
        .map_err(at("하위 키 목록"))?;
    for child in children {
        delete_tree_manually(&key, &child)?;
    }
    drop(key);
    parent
        .delete_subkey_with_flags(leaf, KEY_WOW64_64KEY)
        .map_err(at("키 삭제"))
}

/// 백업을 저장한 **뒤에만** 부른다.
pub fn delete(backup: &RegistryBackup) -> Result<(), RegError> {
    match backup {
        RegistryBackup::Key { location, .. } => {
            let (root, path) = split_location(location).ok_or(RegError::NotFound)?;
            let (parent, leaf) = path
                .rsplit_once('\\')
                .ok_or(RegError::Other("최상위 키는 지울 수 없어요".into()))?;
            // 부모는 읽기 권한으로만 연다. HKLM\SOFTWARE 같은 하이브 루트는 관리자여도 DELETE 권한을 주지 않기 때문에,
            // DELETE는 실제로 지울 키에만 요청한다.
            let parent = root
                .open_subkey_with_flags(parent, KEY_READ | KEY_WOW64_64KEY)
                .map_err(at("부모 키 열기"))?;
            let target = match parent.open_subkey_with_flags(leaf, DELETE_ACCESS | KEY_WOW64_64KEY)
            {
                Ok(k) => k,
                Err(e) if e.raw_os_error() == Some(2) => return Err(RegError::NotFound),
                Err(e) => return Err(at("키 열기")(e)),
            };
            // 키 안의 하위 키·값을 먼저 비우고 (실패하면 하나씩), 마지막에 키 자체를 지운다
            if target.delete_subkey_all("").is_err() {
                let children: Vec<String> = target
                    .enum_keys()
                    .collect::<Result<_, _>>()
                    .map_err(at("하위 키 목록"))?;
                for child in children {
                    delete_tree_manually(&target, &child)?;
                }
            }
            drop(target);
            parent
                .delete_subkey_with_flags(leaf, KEY_WOW64_64KEY)
                .map_err(at("키 삭제"))
        }
        RegistryBackup::Value { location, value } => {
            let (root, path) = split_location(location).ok_or(RegError::NotFound)?;
            let key = root
                .open_subkey_with_flags(path, KEY_SET_VALUE | KEY_WOW64_64KEY)
                .map_err(at("키 열기"))?;
            key.delete_value(&value.name).map_err(at("값 삭제"))
        }
    }
}

/// 원래 자리에 되돌린다. 같은 키·값이 이미 있으면(다시 설치했거나 새로 생김) 덮어쓰지 않는다.
pub fn restore(backup: &RegistryBackup) -> Result<(), RegError> {
    match backup {
        RegistryBackup::Key { location, dump } => {
            let (root, path) = split_location(location).ok_or(RegError::NotFound)?;
            if open(&root, path, KEY_READ).is_ok() {
                return Err(RegError::Conflict(
                    "같은 레지스트리 키가 이미 있어요".into(),
                ));
            }
            write_key(&root, path, dump)
        }
        RegistryBackup::Value { location, value } => {
            let (root, path) = split_location(location).ok_or(RegError::NotFound)?;
            let (key, _) = root.create_subkey_with_flags(path, KEY_ALL_ACCESS | KEY_WOW64_64KEY)?;
            if key.get_raw_value(&value.name).is_ok() {
                return Err(RegError::Conflict("같은 이름의 값이 이미 있어요".into()));
            }
            key.set_raw_value(&value.name, &to_reg_value(value)?)?;
            Ok(())
        }
    }
}

fn write_key(root: &RegKey, path: &str, dump: &KeyDump) -> Result<(), RegError> {
    let (key, _) = root.create_subkey_with_flags(path, KEY_ALL_ACCESS | KEY_WOW64_64KEY)?;
    for v in &dump.values {
        key.set_raw_value(&v.name, &to_reg_value(v)?)?;
    }
    for sub in &dump.subkeys {
        write_key(root, &format!("{path}\\{}", sub.name), sub)?;
    }
    Ok(())
}
