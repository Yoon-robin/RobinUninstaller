//! 시작 폴더 읽기와 바로 가기(.lnk) 해석.

use std::path::{Path, PathBuf};

use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, IPersistFile, STGM_READ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::core::{HSTRING, Interface};

use crate::com::Com;
use crate::model::StartupSource;

/// 시작 폴더 경로
pub fn folder(source: StartupSource) -> Option<PathBuf> {
    let (var, last) = match source {
        StartupSource::FolderUser => ("APPDATA", "Startup"),
        StartupSource::FolderCommon => ("ProgramData", "StartUp"),
        _ => return None,
    };
    let base = std::env::var_os(var)?;
    Some(
        PathBuf::from(base)
            .join(r"Microsoft\Windows\Start Menu\Programs")
            .join(last),
    )
}

/// 시작 폴더 안의 파일 (하위 폴더와 desktop.ini는 뺀다).
pub fn files(source: StartupSource) -> Vec<PathBuf> {
    let Some(dir) = folder(source) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| {
            !p.file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("desktop.ini"))
        })
        .collect();
    files.sort();
    files
}

/// 바로 가기의 (대상 경로, 인자). 대상이 파일이 아닌 바로 가기(제어판 등)는 대상이 빈 문자열이다.
pub fn resolve_shortcut(path: &Path) -> Option<(String, String)> {
    let _com = Com::init();
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        link.cast::<IPersistFile>()
            .ok()?
            .Load(&HSTRING::from(path.as_os_str()), STGM_READ)
            .ok()?;

        let mut target = vec![0u16; 1024];
        let mut find = WIN32_FIND_DATAW::default();
        link.GetPath(&mut target, &mut find, 0).ok()?;
        let mut args = vec![0u16; 2048];
        let _ = link.GetArguments(&mut args);
        Some((from_wide(&target), from_wide(&args)))
    }
}

pub(crate) fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}
