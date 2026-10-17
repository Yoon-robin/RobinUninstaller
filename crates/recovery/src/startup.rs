//! 실행 파일이 없는 시작 항목 정리.
//!
//! 프로그램은 지워졌는데 Run 값이나 시작 폴더 바로 가기만 남은 경우, 그 항목과 짝인 `StartupApproved` 값을
//! 잔여물과 같은 방식(백업·격리)으로 정리한다. 복구 기록에서 되돌릴 수 있다.
//! 실행 파일이 있는 항목과 예약 작업은 정리하지 않는다 (시작프로그램 화면에서 끄기만 한다).

use robin_scanner::{Leftover, LeftoverKind};
use robin_startup::CleanupTarget;
#[cfg(windows)]
use robin_startup::{CleanupRefusal, StartupEntry};

/// 복구 기록에 보일 작업 이름
pub const OPERATION_NAME: &str = "시작프로그램 정리";

/// 정리 대상을 백업·정리 흐름이 받는 항목으로 바꾼다. 순수 함수.
pub fn to_leftovers(targets: Vec<CleanupTarget>) -> Vec<Leftover> {
    targets
        .into_iter()
        .map(|t| match t {
            CleanupTarget::File(path) => Leftover::new(LeftoverKind::File, path, None, Vec::new()),
            CleanupTarget::RegistryValue { key, name } => {
                Leftover::new(LeftoverKind::RegistryValue, key, Some(name), Vec::new())
            }
        })
        .collect()
}

/// 시작 항목 하나에서 정리할 항목. 짝인 `StartupApproved` 값이 있는지는 레지스트리에서 다시 읽는다.
#[cfg(windows)]
pub fn leftovers_for(entry: &StartupEntry) -> Result<Vec<Leftover>, CleanupRefusal> {
    let approved = robin_startup::approved_location(entry.source);
    let has_approved = robin_startup::has_approved(entry.source, &entry.approved_name);
    robin_startup::cleanup_targets(entry, &approved, has_approved).map(to_leftovers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_targets() {
        let items = to_leftovers(vec![
            CleanupTarget::File(r"C:\Startup\a.lnk".into()),
            CleanupTarget::RegistryValue {
                key: r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(),
                name: "A".into(),
            },
        ]);
        assert_eq!(items[0].kind, LeftoverKind::File);
        assert_eq!(items[1].kind, LeftoverKind::RegistryValue);
        assert_eq!(items[1].value_name.as_deref(), Some("A"));
        assert_ne!(items[0].id, items[1].id);
    }
}
