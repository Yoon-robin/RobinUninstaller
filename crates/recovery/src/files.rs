//! 파일·폴더 격리와 복원.
//!
//! 지우지 않고 백업 폴더로 **옮긴다**. 같은 드라이브면 이름만 바꾸는 이동이라 빠르고 원자적이다
//! (폴더 안에 사용 중인 파일이 하나라도 있으면 통째로 실패하고 아무것도 바뀌지 않는다).
//! 다른 드라이브면 먼저 모든 파일을 지울 수 있는지 확인하고, 복사한 뒤 원본을 지운다.
//! 원본을 지우다 멈추면 **복사본은 절대 지우지 않는다** (`Partial`): 데이터가 한쪽에는 반드시 온전히 남는다.
//! 옮길 자리에 이미 무언가 있으면 덮어쓰지 않는다.

use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use windows::Win32::Storage::FileSystem::{MOVE_FILE_FLAGS, MoveFileExW};
use windows::core::PCWSTR;

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
/// 표준 권한 DELETE
const DELETE: u32 = 0x0001_0000;
const FILE_SHARE_ALL: u32 = 0x1 | 0x2 | 0x4;

#[derive(Debug)]
pub enum FsError {
    NotFound,
    ReparsePoint,
    AccessDenied,
    InUse,
    Conflict,
    /// 다른 드라이브로 복사는 끝났지만 원본 일부를 지우지 못함. 복사본은 온전하다.
    Partial(String),
    Other(String),
}

impl From<std::io::Error> for FsError {
    fn from(e: std::io::Error) -> Self {
        match e.raw_os_error() {
            Some(2 | 3) => FsError::NotFound,
            Some(5) => FsError::AccessDenied,
            Some(32 | 33) => FsError::InUse,
            Some(80 | 183) => FsError::Conflict,
            _ => FsError::Other(e.to_string()),
        }
    }
}

/// 정션·심볼릭 링크는 다른 위치를 가리키므로 다루지 않는다.
pub fn check(path: &Path) -> Result<(), FsError> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(FsError::ReparsePoint);
    }
    Ok(())
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

/// 덮어쓰지 않는 이름 바꾸기 (`std::fs::rename`은 Windows에서 같은 이름의 파일을 덮어쓴다).
fn rename_no_replace(src: &Path, dest: &Path) -> std::io::Result<()> {
    let (s, d) = (wide(src), wide(dest));
    unsafe { MoveFileExW(PCWSTR(s.as_ptr()), PCWSTR(d.as_ptr()), MOVE_FILE_FLAGS(0)) }
        .map_err(|e| std::io::Error::from_raw_os_error(e.code().0 & 0xFFFF))
}

/// 어느 쪽으로 옮기는지. 정리할 때는 원본을 지우지 못하면 `Partial`로 알리고,
/// 복원할 때는 데이터가 이미 원래 자리에 다 돌아갔으므로 백업 쪽 찌꺼기는 남겨 둬도 된다.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Clean,
    Restore,
}

/// `src`를 `dest`로 옮긴다 (`dest`는 아직 없어야 함).
pub fn move_item(src: &Path, dest: &Path, direction: Direction) -> Result<(), FsError> {
    if dest.exists() {
        return Err(FsError::Conflict);
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match rename_no_replace(src, dest) {
        Ok(()) => Ok(()),
        // ERROR_NOT_SAME_DEVICE: 다른 드라이브 → 확인 후 복사, 원본 삭제
        Err(e) if e.raw_os_error() == Some(17) => {
            if direction == Direction::Clean {
                // 지울 수 없는 파일이 있으면 아무것도 바꾸지 않고 멈춘다
                ensure_deletable(src)?;
            }
            if let Err(copy_err) = copy_tree(src, dest) {
                // 원본은 아직 그대로라 미완성 복사본만 치운다
                let _ = remove_tree(dest);
                return Err(copy_err);
            }
            match remove_tree(src) {
                Ok(()) => Ok(()),
                Err(_) if direction == Direction::Restore => Ok(()),
                Err(e) => Err(FsError::Partial(format!(
                    "다른 드라이브로 복사는 끝났지만 원래 자리의 일부를 지우지 못했어요 ({e:?})"
                ))),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// 원본의 남은 부분만 지운다 (`Partial` 뒤 다시 시도할 때. 복사본은 이미 온전하다).
pub fn remove_rest(src: &Path) -> Result<(), FsError> {
    check(src)?;
    ensure_deletable(src)?;
    remove_tree(src)
}

/// 폴더 안 모든 파일을 지울 수 있는지(다른 프로그램이 지우기를 막고 열어 두지 않았는지) 미리 확인한다.
fn ensure_deletable(path: &Path) -> Result<(), FsError> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(FsError::ReparsePoint);
    }
    if meta.is_dir() {
        for entry in std::fs::read_dir(path)? {
            ensure_deletable(&entry?.path())?;
        }
        Ok(())
    } else {
        std::fs::OpenOptions::new()
            .access_mode(DELETE)
            .share_mode(FILE_SHARE_ALL)
            .open(path)
            .map(|_| ())
            .map_err(FsError::from)
    }
}

fn copy_tree(src: &Path, dest: &Path) -> Result<(), FsError> {
    check(src)?;
    let meta = std::fs::symlink_metadata(src)?;
    if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_tree(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        if dest.exists() {
            return Err(FsError::Conflict);
        }
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

/// 백업을 원래 자리에 **없는 것만** 채워 넣는다 (일부만 정리된 항목 복원). 있는 파일은 덮어쓰지 않는다.
/// 백업은 그대로 두며, 원래 자리에 이미 다른 내용의 같은 이름 파일이 있으면 그 목록을 돌려준다.
pub fn merge_back(backup: &Path, original: &Path) -> Result<Vec<String>, FsError> {
    check(backup)?;
    let mut kept = Vec::new();
    merge(backup, original, &mut kept)?;
    Ok(kept)
}

fn merge(src: &Path, dest: &Path, kept: &mut Vec<String>) -> Result<(), FsError> {
    check(src)?;
    let meta = std::fs::symlink_metadata(src)?;
    if meta.is_dir() {
        if dest.exists() {
            check(dest)?;
        } else {
            std::fs::create_dir_all(dest)?;
        }
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            merge(&entry.path(), &dest.join(entry.file_name()), kept)?;
        }
    } else if dest.exists() {
        let same = std::fs::metadata(dest).is_ok_and(|d| d.len() == meta.len());
        if !same {
            kept.push(dest.to_string_lossy().into_owned());
        }
    } else {
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

fn remove_tree(path: &Path) -> Result<(), FsError> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
