//! 언인스톨러 프로세스를 띄운다.
//!
//! ShellExecuteEx를 쓰는 이유: 언인스톨러의 매니페스트가 관리자 권한을 요구하면 Windows가 UAC 창을 띄워 준다.
//! 셸(cmd.exe)을 거치지 않으므로 명령 문자열이 해석되거나 다른 명령이 끼어들 수 없다.

use windows::Win32::Foundation::{
    CloseHandle, ERROR_CANCELLED, ERROR_ELEVATION_REQUIRED, HANDLE, WAIT_OBJECT_0,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessId, INFINITE, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{
    SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};

#[derive(Debug)]
pub enum LaunchError {
    /// 사용자가 UAC 창에서 "아니요"를 눌렀음
    Cancelled,
    Failed(String),
}

/// 실행 중인 언인스톨러 프로세스. Drop 시 핸들을 닫는다.
pub struct Launched {
    handle: HANDLE,
    pub pid: u32,
}

impl Launched {
    /// 최상위 프로세스가 끝날 때까지 기다리고 종료 코드를 돌려준다.
    pub fn wait(&self) -> Option<u32> {
        unsafe {
            if WaitForSingleObject(self.handle, INFINITE) != WAIT_OBJECT_0 {
                return None;
            }
            let mut code = 0u32;
            GetExitCodeProcess(self.handle, &mut code).ok()?;
            Some(code)
        }
    }
}

impl Drop for Launched {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// 실행한다. 권한 상승이 필요하다고 거절되면 `runas`로 한 번 더 시도한다.
pub fn launch(file: &str, args: &str, working_dir: Option<&str>) -> Result<Launched, LaunchError> {
    match shell_execute(file, args, working_dir, None) {
        Err(e) if e.code() == ERROR_ELEVATION_REQUIRED.to_hresult() => {
            shell_execute(file, args, working_dir, Some(w!("runas")))
        }
        other => other,
    }
    .map_err(|e| {
        if e.code() == ERROR_CANCELLED.to_hresult() {
            LaunchError::Cancelled
        } else {
            LaunchError::Failed(e.message())
        }
    })
}

fn shell_execute(
    file: &str,
    args: &str,
    working_dir: Option<&str>,
    verb: Option<PCWSTR>,
) -> windows::core::Result<Launched> {
    let file = HSTRING::from(file);
    let args = HSTRING::from(args);
    let dir = working_dir.map(HSTRING::from);

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: verb.unwrap_or(PCWSTR::null()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(args.as_ptr()),
        lpDirectory: dir.as_ref().map_or(PCWSTR::null(), |d| PCWSTR(d.as_ptr())),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut info)?;
        if info.hProcess.is_invalid() {
            // 프로세스 핸들 없이 끝난 경우(다른 프로세스에 넘겨짐 등)는 추적할 수 없다.
            return Err(windows::core::Error::from_thread());
        }
        Ok(Launched {
            handle: info.hProcess,
            pid: GetProcessId(info.hProcess),
        })
    }
}
