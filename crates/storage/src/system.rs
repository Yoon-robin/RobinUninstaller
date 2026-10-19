//! Windows API: 관리자 여부, 휴지통, 드라이브 용량.

use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::UI::Shell::{
    IsUserAnAdmin, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHEmptyRecycleBinW,
    SHQUERYRBINFO, SHQueryRecycleBinW,
};
use windows::core::{HSTRING, PCWSTR};

/// 지금 프로세스가 관리자 권한으로 실행 중인지
/// 다운로드 폴더 (사용자가 옮겼어도 따른다)
pub fn downloads_dir() -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath};
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(std::path::PathBuf::from)
    }
}

pub fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin() }.as_bool()
}

/// 모든 드라이브 휴지통의 (바이트, 항목 수)
pub fn recycle_bin_size() -> (u64, u64) {
    let mut info = SHQUERYRBINFO {
        cbSize: size_of::<SHQUERYRBINFO>() as u32,
        ..Default::default()
    };
    match unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) } {
        Ok(()) => (info.i64Size.max(0) as u64, info.i64NumItems.max(0) as u64),
        Err(_) => (0, 0),
    }
}

/// 모든 드라이브 휴지통을 비운다 (확인 창·소리 없이 — 확인은 앱 화면에서 이미 받았다)
pub fn empty_recycle_bin() -> Result<(), String> {
    unsafe {
        SHEmptyRecycleBinW(
            None,
            PCWSTR::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        )
    }
    .map_err(|e| e.message())
}

/// 드라이브의 (전체, 남은) 바이트. `root`는 `C:\` 형식.
pub fn disk_space(root: &str) -> Option<(u64, u64)> {
    let mut free = 0u64;
    let mut total = 0u64;
    unsafe {
        GetDiskFreeSpaceExW(
            &HSTRING::from(root),
            Some(&mut free),
            Some(&mut total),
            None,
        )
    }
    .ok()?;
    Some((total, free))
}

/// 파일 하나를 지운다. 경로를 다시 해석하지 않도록 **핸들로** 연 뒤:
/// 리파스 포인트·폴더가 아닌지, 실제 위치가 `root` 안인지 확인하고, 같은 핸들로 지운다.
/// 정리하는 동안 누군가 상위 폴더를 정션으로 바꿔치기해도 `root` 밖의 파일은 지워지지 않는다.
/// `root`는 `std::fs::canonicalize`로 얻은 실제 경로(`\\?\C:\...`)여야 한다.
pub fn delete_file_within(path: &std::path::Path, root: &std::path::Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_NAME_NORMALIZED, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FileDispositionInfo, GetFileInformationByHandle,
        GetFinalPathNameByHandleW, OPEN_EXISTING, SetFileInformationByHandle,
    };

    const DELETE: u32 = 0x0001_0000;
    let denied = || std::io::Error::from_raw_os_error(5);
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();

    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            DELETE | FILE_READ_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT,
            None,
        )
    }
    .map_err(|e| std::io::Error::from_raw_os_error(e.code().0 & 0xFFFF))?;
    let handle = Handle(handle);

    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    unsafe { GetFileInformationByHandle(handle.0, &mut info) }
        .map_err(|e| std::io::Error::from_raw_os_error(e.code().0 & 0xFFFF))?;
    if info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT.0 | FILE_ATTRIBUTE_DIRECTORY.0) != 0 {
        return Err(denied());
    }

    let mut buf = vec![0u16; 1024];
    let len =
        unsafe { GetFinalPathNameByHandleW(handle.0, &mut buf, FILE_NAME_NORMALIZED) } as usize;
    if len == 0 || len >= buf.len() {
        return Err(denied());
    }
    let actual = String::from_utf16_lossy(&buf[..len]).to_lowercase();
    let root = root.to_string_lossy().trim_end_matches('\\').to_lowercase();
    if !actual.starts_with(&format!("{root}\\")) {
        return Err(denied());
    }

    let disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
    unsafe {
        SetFileInformationByHandle(
            handle.0,
            FileDispositionInfo,
            (&disposition as *const FILE_DISPOSITION_INFO).cast(),
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    }
    .map_err(|e| std::io::Error::from_raw_os_error(e.code().0 & 0xFFFF))
}
