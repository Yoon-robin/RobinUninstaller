//! 바로 가기(.lnk) 만들기·읽기와 바탕 화면 위치.

use std::path::{Path, PathBuf};

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoTaskMemFree, CoUninitialize, IPersistFile, STGM_READ,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, IShellLinkW, KF_FLAG_DEFAULT, SHGetKnownFolderPath, ShellLink,
};
use windows::core::{HSTRING, Interface, Result};

fn with_com<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();
    let result = f();
    if initialized {
        unsafe { CoUninitialize() };
    }
    result
}

/// `link`에 `target`을 실행하는 바로 가기를 만든다 (있으면 덮어쓴다).
pub fn create(link: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    with_com(|| unsafe {
        let shell: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell.SetPath(&HSTRING::from(target.as_os_str()))?;
        if let Some(dir) = target.parent() {
            shell.SetWorkingDirectory(&HSTRING::from(dir.as_os_str()))?;
        }
        shell.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        shell
            .cast::<IPersistFile>()?
            .Save(&HSTRING::from(link.as_os_str()), true)
    })
}

/// 바로 가기가 가리키는 실행 파일. 읽지 못하면 None.
pub fn target_of(link: &Path) -> Option<PathBuf> {
    with_com(|| unsafe {
        let shell: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell
            .cast::<IPersistFile>()?
            .Load(&HSTRING::from(link.as_os_str()), STGM_READ)?;
        let mut buf = [0u16; 1024];
        shell.GetPath(&mut buf, std::ptr::null_mut(), 0)?;
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Ok(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
    })
    .ok()
    .filter(|p| !p.as_os_str().is_empty())
}

/// 바로 가기가 `olds` 중 하나나 `new`를 가리키는지 (RobinX의 바로 가기인지)
pub fn points_to_any(link: &Path, olds: &[PathBuf], new: &Path) -> bool {
    let Some(target) = target_of(link) else {
        return false;
    };
    olds.iter()
        .chain(std::iter::once(&new.to_path_buf()))
        .any(|p| super::same_path(p, &target))
}

/// 바탕 화면 폴더 (OneDrive로 옮겨진 경우도 따른다)
pub fn desktop_dir() -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}
