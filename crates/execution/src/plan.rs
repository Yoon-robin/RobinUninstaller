//! 제거 명령을 "무엇을 어떤 인자로 실행할지"로 바꾼다. 순수 로직 (시스템 접근 없음).
//!
//! 등록된 명령 문자열을 셸에 그대로 넘기지 않는다. MSI는 검증된 제품 코드로 명령을 직접 만든다.

use std::path::Path;

use robin_core::{InstallTechnology, InstalledApplication, UninstallerStatus};
use robin_discovery::command::{expand_env, find_product_code, is_product_code, split_command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchPlan {
    /// `msiexec.exe /x {GUID}` — 인자는 제품 코드로만 만든다.
    Msi { product_code: String },
    /// 등록된 언인스톨러 실행 파일 + 원본 인자
    Executable { path: String, args: String },
}

impl LaunchPlan {
    /// 실제로 실행할 파일과 인자.
    pub fn file_and_args(&self) -> (String, String) {
        match self {
            LaunchPlan::Msi { product_code } => {
                ("msiexec.exe".into(), format!("/x {product_code}"))
            }
            LaunchPlan::Executable { path, args } => (path.clone(), args.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// 언인스톨러 파일이 없음 → 강제 제거(이후 버전) 대상
    UninstallerMissing,
    /// 제거 명령이 없음
    NoCommand,
    /// 상대 경로나 알 수 없는 형식이라 안전하게 실행할 수 없음
    Unsupported(String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::UninstallerMissing => write!(f, "언인스톨러 파일이 없어서 실행할 수 없어요"),
            PlanError::NoCommand => write!(f, "등록된 제거 명령이 없어요"),
            PlanError::Unsupported(cmd) => write!(f, "지원하지 않는 제거 명령 형식이에요: {cmd}"),
        }
    }
}

/// `lookup_env`는 `%VAR%` 확장용 (테스트에서 고정값을 넣기 위해 주입).
pub fn plan_for(
    app: &InstalledApplication,
    lookup_env: impl Fn(&str) -> Option<String>,
) -> Result<LaunchPlan, PlanError> {
    if app.uninstaller == UninstallerStatus::Missing {
        return Err(PlanError::UninstallerMissing);
    }

    // MSI 제품은 레지스트리 명령(/I로 기록된 경우도 많음) 대신 항상 /x로 직접 만든다.
    if app.technology == InstallTechnology::Msi
        && let Some(code) = app.product_code.as_deref().filter(|c| is_product_code(c))
    {
        return Ok(LaunchPlan::Msi {
            product_code: code.to_string(),
        });
    }

    let command = app
        .uninstall_command
        .as_deref()
        .ok_or(PlanError::NoCommand)?;
    let command = expand_env(command, lookup_env);

    match split_command(&command) {
        None => match find_product_code(&command) {
            Some(code) => Ok(LaunchPlan::Msi {
                product_code: code.to_string(),
            }),
            None => Err(PlanError::Unsupported(command)),
        },
        Some((path, args)) => {
            if !Path::new(&path).is_absolute() {
                return Err(PlanError::Unsupported(command));
            }
            Ok(LaunchPlan::Executable { path, args })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::InstallScope;

    fn app(
        technology: InstallTechnology,
        product_code: Option<&str>,
        command: Option<&str>,
    ) -> InstalledApplication {
        InstalledApplication {
            id: "hkcu:test".into(),
            display_name: "Test".into(),
            publisher: None,
            version: None,
            install_location: None,
            install_date: None,
            estimated_size_kb: None,
            display_icon: None,
            uninstall_command: command.map(Into::into),
            quiet_uninstall_command: None,
            product_code: product_code.map(Into::into),
            technology,
            scope: InstallScope::User,
            uninstaller: UninstallerStatus::Available,
        }
    }

    const CODE: &str = "{6B725F8C-1FCE-49E4-A5A6-AE481A44E45C}";
    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn msi_always_uses_uninstall_switch() {
        let a = app(
            InstallTechnology::Msi,
            Some(CODE),
            Some(&format!("MsiExec.exe /I{CODE}")),
        );
        let plan = plan_for(&a, no_env).unwrap();
        assert_eq!(
            plan.file_and_args(),
            ("msiexec.exe".into(), format!("/x {CODE}"))
        );
    }

    #[test]
    fn msiexec_command_without_flag_is_still_msi() {
        let a = app(
            InstallTechnology::Exe,
            None,
            Some(&format!("msiexec.exe /x {CODE} AI_FLAG=1")),
        );
        assert_eq!(
            plan_for(&a, no_env).unwrap(),
            LaunchPlan::Msi {
                product_code: CODE.into()
            }
        );
    }

    #[test]
    fn exe_keeps_original_args_and_expands_env() {
        let a = app(
            InstallTechnology::Exe,
            None,
            Some(r#""%ProgramFiles%\App\unins000.exe" /SILENT"#),
        );
        let env = |n: &str| (n == "ProgramFiles").then(|| r"C:\Program Files".to_string());
        assert_eq!(
            plan_for(&a, env).unwrap(),
            LaunchPlan::Executable {
                path: r"C:\Program Files\App\unins000.exe".into(),
                args: "/SILENT".into()
            }
        );
    }

    #[test]
    fn rejects_relative_and_missing() {
        let rel = app(
            InstallTechnology::Exe,
            None,
            Some("rundll32.exe foo.dll,Uninstall"),
        );
        assert!(matches!(
            plan_for(&rel, no_env),
            Err(PlanError::Unsupported(_))
        ));

        let mut missing = app(InstallTechnology::Exe, None, Some(r"C:\a\u.exe"));
        missing.uninstaller = UninstallerStatus::Missing;
        assert_eq!(
            plan_for(&missing, no_env),
            Err(PlanError::UninstallerMissing)
        );

        let none = app(InstallTechnology::Exe, None, None);
        assert_eq!(plan_for(&none, no_env), Err(PlanError::NoCommand));
    }
}
