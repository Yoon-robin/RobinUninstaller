//! 기본 언인스톨러 실행.
//!
//! 흐름: 실행 계획 → 실행(필요하면 UAC) → 자식 프로세스까지 끝날 때까지 대기 → 설치 정보를 다시 읽어 결과 판정.
//! 잔여물 스캔·삭제는 여기서 하지 않는다 (Phase 3).

mod ignore;
#[cfg(windows)]
mod launch;
pub mod plan;
#[cfg(windows)]
mod track;

use robin_core::InstalledApplication;
use serde::Serialize;

pub use plan::{LaunchPlan, PlanError, plan_for};
#[cfg(windows)]
pub use track::running_exe_names;

/// 진행 상황. 프론트엔드로 그대로 전달된다.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UninstallEvent {
    /// 언인스톨러를 띄우는 중 (UAC 창이 뜰 수 있음)
    Launching { file: String },
    /// 언인스톨러가 실행 중. 관련 프로세스 수가 바뀔 때마다 보낸다.
    #[serde(rename_all = "camelCase")]
    Running { process_count: usize },
    /// 프로세스가 모두 끝나고 설치 정보를 확인하는 중
    Verifying,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UninstallOutcome {
    /// 설치 정보가 사라짐 → 제거 완료
    Removed,
    /// 제거는 됐지만 재부팅해야 마무리됨 (MSI 3010/1641)
    RemovedRebootRequired,
    /// 언인스톨러는 끝났지만 아직 설치돼 있음 (사용자가 취소했거나 실패)
    #[serde(rename_all = "camelCase")]
    StillInstalled { exit_code: Option<u32> },
    /// UAC 창에서 거절함
    ElevationDenied,
    /// 실행할 수 없음
    Failed { message: String },
}

/// 관리자 권한(UAC)으로 실행하고 끝날 때까지 기다린다. 사용자가 거절하면 `Err(None)`.
#[cfg(windows)]
pub fn run_elevated_and_wait(file: &str, args: &str) -> Result<Option<u32>, Option<String>> {
    match launch::launch_elevated(file, args) {
        Ok(launched) => Ok(launched.wait()),
        Err(launch::LaunchError::Cancelled) => Err(None),
        Err(launch::LaunchError::Failed(m)) => Err(Some(m)),
    }
}

/// MSI 종료 코드 중 의미가 분명한 것.
const MSI_REBOOT_REQUIRED: u32 = 3010;
const MSI_REBOOT_INITIATED: u32 = 1641;

/// 종료 코드와 "아직 설치돼 있는지"를 합쳐 결과를 정한다. 순수 함수.
pub fn classify(exit_code: Option<u32>, still_installed: bool) -> UninstallOutcome {
    match (still_installed, exit_code) {
        (false, Some(MSI_REBOOT_REQUIRED | MSI_REBOOT_INITIATED)) => {
            UninstallOutcome::RemovedRebootRequired
        }
        (false, _) => UninstallOutcome::Removed,
        (true, _) => UninstallOutcome::StillInstalled { exit_code },
    }
}

/// 기본 언인스톨러를 실행하고 끝날 때까지 기다린다 (블로킹).
///
/// `is_still_installed`는 끝난 뒤 설치 정보를 다시 읽어 확인하는 함수다.
/// `stop_waiting`이 켜지면 프로세스를 더 기다리지 않고 바로 확인 단계로 넘어간다.
#[cfg(windows)]
pub fn run_uninstaller(
    app: &InstalledApplication,
    stop_waiting: &std::sync::atomic::AtomicBool,
    mut on_event: impl FnMut(UninstallEvent),
    is_still_installed: impl Fn() -> bool,
) -> UninstallOutcome {
    let plan = match plan_for(app, |name| std::env::var(name).ok()) {
        Ok(plan) => plan,
        Err(e) => {
            return UninstallOutcome::Failed {
                message: e.to_string(),
            };
        }
    };
    let (file, args) = plan.file_and_args();

    on_event(UninstallEvent::Launching { file: file.clone() });
    let working_dir = match &plan {
        LaunchPlan::Executable { path, .. } => std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned()),
        LaunchPlan::Msi { .. } => None,
    };

    let launched = match launch::launch(&file, &args, working_dir.as_deref()) {
        Ok(l) => l,
        Err(launch::LaunchError::Cancelled) => return UninstallOutcome::ElevationDenied,
        Err(launch::LaunchError::Failed(message)) => return UninstallOutcome::Failed { message },
    };

    track::wait_for_tree(launched.pid, &app.display_name, stop_waiting, |count| {
        on_event(UninstallEvent::Running {
            process_count: count,
        })
    });
    let exit_code = launched.exit_code();
    drop(launched);

    on_event(UninstallEvent::Verifying);
    // 일부 언인스톨러는 프로세스가 끝난 직후에 등록 정보를 지운다. 잠시 여유를 두고 확인한다.
    let mut still = is_still_installed();
    for _ in 0..6 {
        if !still {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        still = is_still_installed();
    }

    classify(exit_code, still)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification() {
        assert_eq!(classify(Some(0), false), UninstallOutcome::Removed);
        assert_eq!(
            classify(Some(3010), false),
            UninstallOutcome::RemovedRebootRequired
        );
        assert_eq!(
            classify(Some(1602), true),
            UninstallOutcome::StillInstalled {
                exit_code: Some(1602)
            }
        );
        // 종료 코드가 성공이어도 아직 등록돼 있으면 제거된 게 아니다
        assert_eq!(
            classify(Some(0), true),
            UninstallOutcome::StillInstalled { exit_code: Some(0) }
        );
    }
}
