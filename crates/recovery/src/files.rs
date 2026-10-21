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

use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, MOVE_FILE_FLAGS, MoveFileExW};
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
    /// `needs_admin`: 지우지 못한 까닭이 권한이라 관리자 권한으로 다시 시도할 수 있음.
    Partial {
        message: String,
        needs_admin: bool,
    },
    Other(String),
}

impl FsError {
    /// 화면에 보여 줄 문장 (디버그 출력 대신)
    pub fn message(&self) -> String {
        match self {
            FsError::NotFound => "찾을 수 없어요".into(),
            FsError::ReparsePoint => "다른 위치를 가리키는 링크(정션)라 건드리지 않았어요".into(),
            FsError::AccessDenied => "관리자 권한이 필요해요".into(),
            FsError::InUse => "다른 프로그램이 사용 중이에요".into(),
            FsError::Conflict => "같은 이름이 이미 있어요".into(),
            FsError::Partial { message, .. } | FsError::Other(message) => message.clone(),
        }
    }
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
/// 대상 자신뿐 아니라 **상위 폴더**가 링크여도 거부한다: 검사한 경로와 실제로 옮겨지는 위치가 달라질 수 있다.
pub fn check(path: &Path) -> Result<(), FsError> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 || reparse_in_parents(path) {
        return Err(FsError::ReparsePoint);
    }
    Ok(())
}

/// 상위 폴더(드라이브 루트 제외) 중 하나라도 정션·심볼릭 링크인지. 없는 폴더는 건너뛴다.
pub fn reparse_in_parents(path: &Path) -> bool {
    path.ancestors()
        .skip(1)
        .filter(|p| p.parent().is_some())
        .any(|p| {
            std::fs::symlink_metadata(p)
                .is_ok_and(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
        })
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
            // 복사할 자리가 모자라면 시작하지 않는다 (중간에 디스크가 가득 차지 않게)
            if let Some(free) = dest.parent().and_then(free_space)
                && tree_size(src) > free
            {
                return Err(FsError::Other(
                    "옮길 드라이브에 빈 공간이 모자라요".to_string(),
                ));
            }
            // 맨 위 폴더는 새로 만든다. 그 사이에 같은 이름의 폴더가 생겼으면(다시 설치 등) 그 폴더를 쓰지 않는다:
            // 복사하다 실패하면 아래에서 복사본을 통째로 지우는데, 남의 폴더면 그 안의 파일까지 지우게 된다.
            if std::fs::symlink_metadata(src)?.is_dir() {
                std::fs::create_dir(dest).map_err(|e| match e.kind() {
                    std::io::ErrorKind::AlreadyExists => FsError::Conflict,
                    _ => e.into(),
                })?;
            }
            if let Err(copy_err) = copy_tree(src, dest) {
                // 원본은 아직 그대로라 미완성 복사본만 치운다
                let _ = remove_tree(dest);
                return Err(copy_err);
            }
            if direction == Direction::Restore {
                // 데이터는 원래 자리에 다 돌아갔다. 백업 쪽은 지우지 못해도 괜찮다.
                let _ = remove_tree(src);
                return Ok(());
            }
            // 복사본과 같은 것만 지운다. 복사하는 동안 새로 생겼거나 바뀐 파일은 백업에 없으므로 남긴다.
            remove_copied(src, dest).map_err(|e| FsError::Partial {
                message: format!(
                    "다른 드라이브로 복사는 끝났지만 원래 자리의 일부를 지우지 못했어요: {}",
                    e.message()
                ),
                needs_admin: matches!(e, FsError::AccessDenied),
            })
        }
        Err(e) => Err(e.into()),
    }
}

/// 원본의 남은 부분만 지운다 (`Partial` 뒤 다시 시도할 때. 복사본은 이미 온전하다).
/// 백업과 같은 파일만 지우고, 그 뒤에 새로 생기거나 바뀐 파일은 남긴다 (백업에 없으므로).
pub fn remove_rest(src: &Path, backup: &Path) -> Result<(), FsError> {
    check(src)?;
    ensure_deletable(src)?;
    remove_copied(src, backup)
}

/// `src`에서 복사본 `copy`와 같은 파일(크기·수정 시각)만 지우고, 빈 폴더를 정리한다.
/// 지울 수 있는 것은 끝까지 지우고, 남긴 것이 있으면 첫 오류를 돌려준다.
fn remove_copied(src: &Path, copy: &Path) -> Result<(), FsError> {
    let meta = std::fs::symlink_metadata(src)?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(FsError::ReparsePoint);
    }
    if meta.is_dir() {
        let mut first_err = None;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            if let Err(e) = remove_copied(&entry.path(), &copy.join(entry.file_name())) {
                first_err.get_or_insert(e);
            }
        }
        if let Some(e) = first_err {
            return Err(e);
        }
        std::fs::remove_dir(src)?;
        return Ok(());
    }
    let same = std::fs::symlink_metadata(copy)
        .is_ok_and(|c| c.len() == meta.len() && c.modified().ok() == meta.modified().ok());
    if !same {
        return Err(FsError::Other(format!(
            "복사한 뒤 새로 생기거나 바뀐 파일이라 남겨 두었어요: {}",
            src.display()
        )));
    }
    std::fs::remove_file(src)?;
    Ok(())
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
        crate::progress::report(|p| p.set_current(src.display()));
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_tree(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        if dest.exists() {
            return Err(FsError::Conflict);
        }
        let copied = std::fs::copy(src, dest)?;
        crate::progress::report(|p| p.add_file(copied));
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
        // 크기가 같아도 내용이 다를 수 있다: 바이트까지 같아야 "이미 돌아와 있음"으로 본다
        if !same_contents(src, dest) {
            kept.push(dest.to_string_lossy().into_owned());
        }
    } else {
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

/// 두 파일의 내용이 바이트 단위로 같은지. 읽지 못하면 다르다고 본다(덮어쓰지 않는 쪽).
fn same_contents(a: &Path, b: &Path) -> bool {
    use std::io::Read;
    let (Ok(ma), Ok(mb)) = (std::fs::metadata(a), std::fs::metadata(b)) else {
        return false;
    };
    if !mb.is_file() || ma.len() != mb.len() {
        return false;
    }
    let (Ok(mut fa), Ok(mut fb)) = (std::fs::File::open(a), std::fs::File::open(b)) else {
        return false;
    };
    let (mut ba, mut bb) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    loop {
        let n = match fa.read(&mut ba) {
            Ok(n) => n,
            Err(_) => return false,
        };
        if n == 0 {
            return true;
        }
        if fb.read_exact(&mut bb[..n]).is_err() || ba[..n] != bb[..n] {
            return false;
        }
    }
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

/// 폴더(또는 파일)의 전체 크기. 링크는 따라가지 않는다.
fn tree_size(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        0
    } else if meta.is_dir() {
        std::fs::read_dir(path)
            .map(|entries| entries.flatten().map(|e| tree_size(&e.path())).sum())
            .unwrap_or(0)
    } else {
        meta.len()
    }
}

/// `dir`이 있는 드라이브의 빈 공간 (바이트)
fn free_space(dir: &Path) -> Option<u64> {
    // 아직 없는 폴더면 있는 상위 폴더로
    let existing = dir.ancestors().find(|p| p.exists())?;
    let w = wide(existing);
    let mut free = 0u64;
    unsafe { GetDiskFreeSpaceExW(PCWSTR(w.as_ptr()), Some(&mut free), None, None) }.ok()?;
    Some(free)
}
