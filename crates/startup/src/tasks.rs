//! 작업 스케줄러에서 로그인·부팅 때 실행되는 작업 읽기와 사용/사용 안 함 바꾸기.
//!
//! `\Microsoft\` 아래(Windows 자체 작업)는 읽지 않는다. 바꾸는 것은 작업의 `Enabled` 값 하나뿐이다.

use windows::Win32::Foundation::{E_ACCESSDENIED, VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::TaskScheduler::{
    IExecAction, IRegisteredTask, ITaskFolder, ITaskService, TASK_ACTION_EXEC, TASK_ACTION_TYPE,
    TASK_ENUM_HIDDEN, TASK_LOGON_TYPE, TASK_RUNLEVEL_HIGHEST, TASK_RUNLEVEL_TYPE,
    TASK_TRIGGER_BOOT, TASK_TRIGGER_LOGON, TASK_TRIGGER_TYPE2, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, Interface};

use crate::com::Com;

/// 로그인·부팅 때 실행되는 작업 하나 (읽은 그대로).
pub struct Task {
    pub name: String,
    /// `\Vendor\Updater` 같은 전체 경로
    pub path: String,
    /// 실행 명령 (첫 번째 프로그램 실행 동작). 프로그램 실행 동작이 없으면 `None`.
    pub command: Option<String>,
    pub author: Option<String>,
    pub enabled: bool,
    pub needs_admin: bool,
}

/// 같은 컴퓨터의 작업 스케줄러에 현재 사용자로 연결한다.
fn connect() -> windows::core::Result<ITaskService> {
    unsafe {
        let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        let none = VARIANT::default();
        service.Connect(&none, &none, &none, &none)?;
        Ok(service)
    }
}

/// 로그인·부팅 트리거가 있는 작업 전체. 읽을 권한이 없는 폴더·작업은 건너뛴다.
pub fn list() -> Vec<Task> {
    let _com = Com::init();
    let Ok(service) = connect() else {
        return Vec::new();
    };
    let Ok(root) = (unsafe { service.GetFolder(&BSTR::from(r"\")) }) else {
        return Vec::new();
    };
    let user = crate::model::CurrentUser::from_env();
    let mut tasks = Vec::new();
    walk(&root, &user, &mut tasks);
    tasks
}

fn walk(folder: &ITaskFolder, user: &crate::model::CurrentUser, out: &mut Vec<Task>) {
    unsafe {
        if let Ok(collection) = folder.GetTasks(TASK_ENUM_HIDDEN.0) {
            for i in 1..=collection.Count().unwrap_or(0) {
                if let Ok(task) = collection.get_Item(&VARIANT::from(i))
                    && let Some(task) = read(&task, user)
                {
                    out.push(task);
                }
            }
        }
        let Ok(subfolders) = folder.GetFolders(0) else {
            return;
        };
        for i in 1..=subfolders.Count().unwrap_or(0) {
            let Ok(sub) = subfolders.get_Item(&VARIANT::from(i)) else {
                continue;
            };
            let path = sub.Path().map(|p| p.to_string()).unwrap_or_default();
            if crate::model::is_windows_task_folder(&path) {
                continue;
            }
            walk(&sub, user, out);
        }
    }
}

fn read(task: &IRegisteredTask, user: &crate::model::CurrentUser) -> Option<Task> {
    unsafe {
        let def = task.Definition().ok()?;

        let triggers = def.Triggers().ok()?;
        let mut count = 0;
        triggers.Count(&mut count).ok()?;
        let at_start = (1..=count).any(|i| {
            let mut kind = TASK_TRIGGER_TYPE2::default();
            triggers.get_Item(i).and_then(|t| t.Type(&mut kind)).is_ok()
                && (kind == TASK_TRIGGER_LOGON || kind == TASK_TRIGGER_BOOT)
        });
        if !at_start {
            return None;
        }

        let mut command = None;
        if let Ok(actions) = def.Actions() {
            let mut count = 0;
            let _ = actions.Count(&mut count);
            for i in 1..=count {
                let Ok(action) = actions.get_Item(i) else {
                    continue;
                };
                let mut kind = TASK_ACTION_TYPE::default();
                if action.Type(&mut kind).is_err() || kind != TASK_ACTION_EXEC {
                    continue;
                }
                let Ok(exec) = action.cast::<IExecAction>() else {
                    continue;
                };
                let (mut path, mut args) = (BSTR::new(), BSTR::new());
                let _ = exec.Path(&mut path);
                let _ = exec.Arguments(&mut args);
                command = Some(crate::model::task_command(
                    &path.to_string(),
                    &args.to_string(),
                ));
                break;
            }
        }

        let mut author = BSTR::new();
        if let Ok(info) = def.RegistrationInfo() {
            let _ = info.Author(&mut author);
        }

        let (mut user_id, mut run_level, mut logon) = (
            BSTR::new(),
            TASK_RUNLEVEL_TYPE::default(),
            TASK_LOGON_TYPE::default(),
        );
        if let Ok(principal) = def.Principal() {
            let _ = principal.UserId(&mut user_id);
            let _ = principal.RunLevel(&mut run_level);
            let _ = principal.LogonType(&mut logon);
        }

        Some(Task {
            name: task.Name().ok()?.to_string(),
            path: task.Path().ok()?.to_string(),
            command,
            author: Some(author.to_string()).filter(|a| !a.trim().is_empty()),
            enabled: task.Enabled().ok()? == VARIANT_TRUE,
            needs_admin: !user.owns_task(&user_id.to_string())
                || run_level == TASK_RUNLEVEL_HIGHEST,
        })
    }
}

/// 작업의 `Enabled` 값을 바꾼다. 권한이 없으면 `PermissionDenied`.
pub fn set_enabled(path: &str, enabled: bool) -> std::io::Result<()> {
    let _com = Com::init();
    let result = (|| unsafe {
        let task = connect()?
            .GetFolder(&BSTR::from(r"\"))?
            .GetTask(&BSTR::from(path))?;
        task.SetEnabled(if enabled { VARIANT_TRUE } else { VARIANT_FALSE })
    })();
    result.map_err(|e| {
        if e.code() == E_ACCESSDENIED {
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, e.message())
        } else {
            std::io::Error::other(e.message())
        }
    })
}
