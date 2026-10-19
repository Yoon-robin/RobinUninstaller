//! 테스트 앱이 쓰는 COM 도우미: 바로 가기 만들기, 예약 작업 등록·삭제.
//!
//! 예전에는 PowerShell로 만들었지만, PowerShell이 없는 환경(Wine 등)에서는 아무것도 만들지 않고 성공으로 끝나서
//! 테스트가 엉뚱한 곳에서 실패했다. Windows API를 직접 부르면 실패가 바로 드러난다.

use std::path::Path;

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize, IPersistFile,
};
use windows::Win32::System::TaskScheduler::{
    ITaskService, TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN, TASK_LOGON_SERVICE_ACCOUNT,
    TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::core::{BSTR, HSTRING, Interface, Result};

/// 이 함수 안에서만 COM을 초기화한다.
fn with_com<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();
    let result = f();
    if initialized {
        unsafe { CoUninitialize() };
    }
    result
}

/// `link`(.lnk)에 `target args`를 실행하는 바로 가기를 만든다.
pub fn create_shortcut(link: &Path, target: &Path, args: &str) -> Result<()> {
    with_com(|| unsafe {
        let shell: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell.SetPath(&HSTRING::from(target.as_os_str()))?;
        shell.SetArguments(&HSTRING::from(args))?;
        shell
            .cast::<IPersistFile>()?
            .Save(&HSTRING::from(link.as_os_str()), true)
    })
}

/// 작업이 언제 실행되는지
pub enum Trigger {
    /// 이 사용자가 로그인할 때, 그 사용자 권한으로 (관리자 권한 필요 없음)
    Logon { user: String },
    /// 부팅할 때 SYSTEM으로 (관리자 권한 필요)
    BootAsSystem,
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn task_xml(
    trigger: &Trigger,
    command: &Path,
    args: &str,
    author: &str,
    description: &str,
) -> String {
    let (trigger_xml, principal) = match trigger {
        Trigger::Logon { user } => (
            format!(
                "<LogonTrigger><Enabled>true</Enabled><UserId>{0}</UserId></LogonTrigger>",
                xml_escape(user)
            ),
            format!(
                "<UserId>{}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel>",
                xml_escape(user)
            ),
        ),
        Trigger::BootAsSystem => (
            "<BootTrigger><Enabled>true</Enabled></BootTrigger>".to_string(),
            "<UserId>S-1-5-18</UserId><RunLevel>HighestAvailable</RunLevel>".to_string(),
        ),
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Author>{author}</Author><Description>{description}</Description></RegistrationInfo>
  <Triggers>{trigger_xml}</Triggers>
  <Principals><Principal id="Author">{principal}</Principal></Principals>
  <Settings><Enabled>true</Enabled><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries></Settings>
  <Actions Context="Author"><Exec><Command>{command}</Command><Arguments>{args}</Arguments></Exec></Actions>
</Task>"#,
        author = xml_escape(author),
        description = xml_escape(description),
        command = xml_escape(&command.display().to_string()),
        args = xml_escape(args),
    )
}

fn connect() -> Result<ITaskService> {
    unsafe {
        let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        let none = VARIANT::default();
        service.Connect(&none, &none, &none, &none)?;
        Ok(service)
    }
}

/// 루트 폴더(`\`)에 작업을 등록한다. 같은 이름이 있으면 덮어쓴다.
pub fn register_task(
    name: &str,
    trigger: &Trigger,
    command: &Path,
    args: &str,
    author: &str,
    description: &str,
) -> Result<()> {
    let xml = task_xml(trigger, command, args, author, description);
    with_com(|| unsafe {
        let folder = connect()?.GetFolder(&BSTR::from(r"\"))?;
        let none = VARIANT::default();
        let (user, logon, sddl) = match trigger {
            Trigger::Logon { user } => (
                VARIANT::from(BSTR::from(user.as_str())),
                TASK_LOGON_INTERACTIVE_TOKEN,
                VARIANT::default(),
            ),
            // 관리자가 기본 권한으로 등록한 SYSTEM 작업은 일반 사용자에게 보이지 않는다.
            // 실제 프로그램 다수처럼 로그인한 사용자가 읽을 수 있게(AU 읽기) 등록해, 일반 권한 앱의 목록에 나오게 한다.
            Trigger::BootAsSystem => (
                VARIANT::from(BSTR::from("SYSTEM")),
                TASK_LOGON_SERVICE_ACCOUNT,
                VARIANT::from(BSTR::from("D:(A;;FA;;;BA)(A;;FA;;;SY)(A;;FR;;;AU)")),
            ),
        };
        folder.RegisterTask(
            &BSTR::from(name),
            &BSTR::from(xml),
            TASK_CREATE_OR_UPDATE.0,
            &user,
            &none,
            logon,
            &sddl,
        )?;
        Ok(())
    })
}

/// 작업을 지운다. 없거나 권한이 없으면 오류.
pub fn delete_task(name: &str) -> Result<()> {
    with_com(|| unsafe {
        connect()?
            .GetFolder(&BSTR::from(r"\"))?
            .DeleteTask(&BSTR::from(name), 0)
    })
}

/// 이 프로세스의 표준 입출력 핸들을 자식 프로세스에 물려주지 않게 한다.
/// Windows는 물려줄 수 있는 핸들을 모두 자식에게 넘기므로(입출력을 끊어도), 오래 사는 자식이 부른 쪽의 파이프를
/// 잡고 있어 `robin-fixture run | ...` 같은 호출이 끝나지 않는다.
pub fn stop_inheriting_std_handles() {
    use windows::Win32::Foundation::{HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation};
    use windows::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        unsafe {
            if let Ok(handle) = GetStdHandle(which)
                && !handle.is_invalid()
            {
                let _ = SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
            }
        }
    }
}
