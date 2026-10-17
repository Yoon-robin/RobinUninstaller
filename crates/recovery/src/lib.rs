//! 잔여물 정리: 백업·격리 후 제거, 작업 기록, 복원.
//!
//! 영구 삭제는 하지 않는다. 파일·폴더는 백업 폴더로 옮기고, 레지스트리는 원본 그대로 기록한 뒤 지운다.
//! 사용자가 복구 기록에서 직접 "백업 삭제"를 고를 때만 실제로 사라진다.

pub mod model;
pub mod startup;
pub mod store;

#[cfg(windows)]
pub mod clean;
#[cfg(windows)]
mod files;
#[cfg(windows)]
pub mod registry;

pub use model::{Backup, ItemStatus, Operation, OperationItem};
pub use store::Store;

#[cfg(windows)]
pub use clean::{Guard, clean, clean_only, restore};

/// 새 작업을 만든다 (아직 아무것도 지우지 않음).
pub fn new_operation(app_name: &str, items: &[robin_scanner::Leftover]) -> Operation {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Operation {
        id: model::new_operation_id(now.as_millis(), std::process::id()),
        app_name: app_name.to_string(),
        created_at: now.as_secs(),
        items: items.iter().map(OperationItem::from_leftover).collect(),
    }
}
