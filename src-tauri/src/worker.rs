//! 관리자 권한 작업자 모드.
//!
//! 일반 권한으로 지우지 못한 항목(HKLM, Program Files 등)만, 사용자가 UAC를 허용한 뒤 이 모드로 처리한다.
//! 앱 화면에서 넘어온 값은 믿지 않는다: 작업 기록을 디스크에서 다시 읽고, 보호 규칙 검사를 처음부터 다시 한다.
//!
//! ```text
//! robin-uninstaller.exe --elevated-clean   <작업 ID> --store <백업 저장소>
//! robin-uninstaller.exe --elevated-restore <작업 ID> --store <백업 저장소>
//! robin-uninstaller.exe --elevated-startup <on|off> <시작 항목 ID> --store <시작프로그램 기록 폴더>
//! ```
//!
//! 시작 항목도 마찬가지로 ID만 받고, 목록을 다시 읽어 그 ID의 항목을 찾는다.

use std::path::PathBuf;

use robin_recovery::{Guard, ItemStatus, Store};

pub const CLEAN: &str = "--elevated-clean";
pub const RESTORE: &str = "--elevated-restore";
pub const STARTUP: &str = "--elevated-startup";

/// 작업자 모드가 아니면 None. 맞으면 작업을 마친 뒤 종료 코드.
pub fn main() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first()?.as_str();
    if mode == STARTUP {
        return Some(run_startup(&args[1..]).unwrap_or(2));
    }
    if mode != CLEAN && mode != RESTORE {
        return None;
    }
    Some(run(mode, &args[1..]).unwrap_or(2))
}

/// 저장소 경로 인자(`--store <경로>`)가 RobinUninstaller 폴더 아래 정해진 곳인지 확인한다.
fn store_arg(rest: &[String], expected_suffix: &str) -> Option<PathBuf> {
    let root = match rest.first().map(String::as_str) {
        Some("--store") => PathBuf::from(rest.get(1)?),
        _ => return None,
    };
    root.to_string_lossy()
        .to_lowercase()
        .ends_with(expected_suffix)
        .then_some(root)
}

fn run_startup(rest: &[String]) -> Option<i32> {
    let enabled = match rest.first()?.as_str() {
        "on" => true,
        "off" => false,
        _ => return None,
    };
    let id = rest.get(1)?;
    if !robin_startup::is_valid_id(id) {
        return None;
    }
    // 기록 위치는 항상 ...\RobinUninstaller\startup 이어야 한다
    let history = robin_startup::History::new(store_arg(&rest[2..], r"\robinuninstaller\startup")?);
    Some(match robin_startup::set_enabled(&history, id, enabled) {
        Ok(_) => 0,
        Err(_) => 1,
    })
}

fn run(mode: &str, rest: &[String]) -> Option<i32> {
    let op_id = rest.first()?;
    // 저장소 위치는 항상 ...\RobinUninstaller\backups 여야 한다
    let store = Store::new(store_arg(&rest[1..], r"\robinuninstaller\backups")?);
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
