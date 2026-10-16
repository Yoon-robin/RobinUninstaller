//! 관리자 권한 작업자 모드.
//!
//! 일반 권한으로 지우지 못한 항목(HKLM, Program Files 등)만, 사용자가 UAC를 허용한 뒤 이 모드로 처리한다.
//! 앱 화면에서 넘어온 값은 믿지 않는다: 작업 기록을 디스크에서 다시 읽고, 보호 규칙 검사를 처음부터 다시 한다.
//!
//! ```text
//! robin-uninstaller.exe --elevated-clean   <작업 ID> --store <백업 저장소>
//! robin-uninstaller.exe --elevated-restore <작업 ID> --store <백업 저장소>
//! ```

use std::path::PathBuf;

use robin_recovery::{Guard, ItemStatus, Store};

pub const CLEAN: &str = "--elevated-clean";
pub const RESTORE: &str = "--elevated-restore";

/// 작업자 모드가 아니면 None. 맞으면 작업을 마친 뒤 종료 코드.
pub fn main() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first()?.as_str();
    if mode != CLEAN && mode != RESTORE {
        return None;
    }
    Some(run(mode, &args[1..]).unwrap_or(2))
}

fn run(mode: &str, rest: &[String]) -> Option<i32> {
    let op_id = rest.first()?;
    let store_root = match rest.get(1).map(String::as_str) {
        Some("--store") => PathBuf::from(rest.get(2)?),
        _ => return None,
    };
    // 저장소 위치는 항상 ...\RobinUninstaller\backups 여야 한다
    if !store_root
        .to_string_lossy()
        .to_lowercase()
        .ends_with(r"\robinuninstaller\backups")
    {
        return None;
    }
    let store = Store::new(store_root);
    let mut op = store.load(op_id).ok()?;

    if mode == CLEAN {
        let guard = Guard {
            known: robin_scanner::protect::KnownFolders::from_env(),
            store_root: store.root().to_path_buf(),
        };
        // 관리자 권한이 필요했던 항목만 다시 시도한다
        for item in &mut op.items {
            if item.status != ItemStatus::NeedsAdmin {
                item.status = match item.status {
                    ItemStatus::Pending => ItemStatus::Failed,
                    s => s,
                };
            } else {
                item.status = ItemStatus::Pending;
            }
        }
        robin_recovery::clean(&store, &mut op, &guard, |_| {});
    } else {
        robin_recovery::restore(&store, &mut op, None, |_| {});
    }
    Some(0)
}
