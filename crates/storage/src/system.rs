//! Windows API: 관리자 여부, 휴지통, 드라이브 용량.

use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::UI::Shell::{
    IsUserAnAdmin, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHEmptyRecycleBinW,
    SHQUERYRBINFO, SHQueryRecycleBinW,
};
use windows::core::{HSTRING, PCWSTR};

/// 지금 프로세스가 관리자 권한으로 실행 중인지
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
