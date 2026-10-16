//! 파일·폴더 격리와 복원.
//!
//! 지우지 않고 백업 폴더로 **옮긴다**. 같은 드라이브면 이름만 바꾸는 이동이라 빠르고 원자적이다
//! (폴더 안에 사용 중인 파일이 하나라도 있으면 통째로 실패하고 아무것도 바뀌지 않는다).
//! 다른 드라이브면 복사한 뒤 원본을 지운다.

use std::os::windows::fs::MetadataExt;
use std::path::Path;

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

#[derive(Debug)]
pub enum FsError {
    NotFound,
    ReparsePoint,
    AccessDenied,
    InUse,
    Conflict,
    Other(String),
}

impl From<std::io::Error> for FsError {
    fn from(e: std::io::Error) -> Self {
        match e.raw_os_error() {
            Some(2 | 3) => FsError::NotFound,
            Some(5) => FsError::AccessDenied,
            Some(32 | 33) => FsError::InUse,
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

/// `src`를 `dest`로 옮긴다 (`dest`는 아직 없어야 함).
pub fn move_item(src: &Path, dest: &Path) -> Result<(), FsError> {
    if dest.exists() {
        return Err(FsError::Conflict);
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::rename(src, dest) {
        Ok(()) => Ok(()),
        // ERROR_NOT_SAME_DEVICE: 다른 드라이브 → 복사 후 원본 삭제
        Err(e) if e.raw_os_error() == Some(17) => {
            if let Err(copy_err) = copy_tree(src, dest) {
                let _ = remove_tree(dest);
                return Err(copy_err);
            }
            if let Err(rm_err) = remove_tree(src) {
                // 원본을 못 지웠으면 복사본을 치워서 상태를 원래대로
                let _ = remove_tree(dest);
                return Err(rm_err);
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
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
