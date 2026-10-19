//! 설치·제거 전에 실행 중인 RobinX(설치된 위치의 robinx.exe)를 찾고 끝낸다.
//! 이 프로세스 자신(설치 프로그램)은 빼고, 경로가 정확히 같은 것만 다룬다.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
    QueryFullProcessImageNameW, TerminateProcess,
};
use windows::core::PWSTR;

fn pids() -> Vec<u32> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return out;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                out.push(entry.th32ProcessID);
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    out
}

fn image_path(pid: u32) -> Option<PathBuf> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        ok.ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(
            &buf[..len as usize],
        )))
    }
}

/// `exes` 중 하나로 실행 중인 프로세스 ID (자기 자신 제외)
pub fn running_copies(exes: &[PathBuf]) -> Vec<u32> {
    let me = std::process::id();
    pids()
        .into_iter()
        .filter(|&pid| pid != 0 && pid != me)
        .filter(|&pid| {
            image_path(pid).is_some_and(|p| exes.iter().any(|e| super::same_path(e, &p)))
        })
        .collect()
}

/// `exes`로 실행 중인 RobinX가 끝나기를 `wait`만큼 기다리고, 그래도 남아 있으면 끝낸다.
/// 남은 프로세스 수를 돌려준다 (0이면 모두 끝남).
pub fn close_running(exes: &[PathBuf], wait: Duration) -> usize {
    let start = Instant::now();
    while start.elapsed() < wait {
        if running_copies(exes).is_empty() {
            return 0;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    for pid in running_copies(exes) {
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(handle, 1);
                let _ = CloseHandle(handle);
            }
        }
    }
    std::thread::sleep(Duration::from_millis(500));
    running_copies(exes).len()
}
