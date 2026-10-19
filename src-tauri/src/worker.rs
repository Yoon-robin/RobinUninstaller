//! 관리자 권한 작업자 모드.
//!
//! 일반 권한으로 지우지 못한 항목(HKLM, Program Files 등)만, 사용자가 UAC를 허용한 뒤 이 모드로 처리한다.
//! 앱 화면에서 넘어온 값은 믿지 않는다: 작업 기록을 디스크에서 다시 읽고, 보호 규칙 검사를 처음부터 다시 한다.
//!
//! ```text
//! robinx.exe --elevated-clean   <작업 ID> --store <백업 저장소>
//! robinx.exe --elevated-restore <작업 ID> --store <백업 저장소> [--items 0,3]
//! robinx.exe --elevated-delete  <작업 ID> --store <백업 저장소>
//! robinx.exe --elevated-startup <on|off> <시작 항목 ID> --store <시작프로그램 기록 폴더>
//! robinx.exe --elevated-storage <0|1> <정리 규칙 ID,...> --store <저장공간 기록 폴더>
//! ```
//!
//! 시작 항목(Run 키·시작 폴더·예약 작업)도 마찬가지로 ID만 받고, 목록을 다시 읽어 그 ID의 항목을 찾는다.
//!
//! 종료 코드: 0 = 다 끝남, 1 = 일부 항목을 처리하지 못함(이유는 기록에), 2 = 인자 거부, 3 = 기록 저장 실패.

use std::path::PathBuf;

use robin_recovery::{Guard, ItemStatus, Store};

pub const CLEAN: &str = "--elevated-clean";
pub const RESTORE: &str = "--elevated-restore";
pub const STARTUP: &str = "--elevated-startup";
pub const STORAGE: &str = "--elevated-storage";
pub const DELETE: &str = "--elevated-delete";
/// 복원할 항목 번호 목록 (`--elevated-restore` 뒤에 선택으로 붙는다)
pub const ITEMS: &str = "--items";

/// 일부 항목을 처리하지 못함. 기록은 저장됐으므로 앱은 기록을 읽어 항목별 이유를 보여 준다.
pub const EXIT_SOME_FAILED: i32 = 1;
/// 작업 기록을 저장하지 못해 멈춤.
pub const EXIT_SAVE_FAILED: i32 = 3;

/// 작업자 모드가 아니면 None. 맞으면 작업을 마친 뒤 종료 코드.
pub fn main() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first()?.as_str();
    if mode == STARTUP {
        return Some(run_startup(&args[1..]).unwrap_or(2));
    }
    if mode == STORAGE {
        return Some(run_storage(&args[1..]).unwrap_or(2));
    }
    if mode == DELETE {
        return Some(run_delete(&args[1..]).unwrap_or(2));
    }
    if mode != CLEAN && mode != RESTORE {
        return None;
    }
    Some(run(mode, &args[1..]).unwrap_or(2))
}

/// 저장소 경로 인자(`--store <경로>`)가 정말 이 앱의 저장소인지 확인한다.
/// - 모양: `<사용자 폴더 루트>\<계정>\AppData\Local\RobinX\<sub>` 그대로 (다른 계정으로 UAC를 허용해도 같은 루트)
/// - 경로 어디에도 정션·심볼릭 링크가 없어야 한다 (모양은 맞지만 다른 곳으로 이어지는 경우를 막는다)
fn store_arg(rest: &[String], sub: &str) -> Option<PathBuf> {
    let root = match rest.first().map(String::as_str) {
        Some("--store") => PathBuf::from(rest.get(1)?),
        _ => return None,
    };
    let users_root = robin_scanner::protect::KnownFolders::from_env().users_root;
    (Store::is_app_store_path(&root, &users_root, sub) && !robin_recovery::path_has_link(&root))
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
    // 기록 위치는 항상 ...\RobinX\startup 이어야 한다
    let history = robin_startup::History::new(store_arg(&rest[2..], "startup")?);
    Some(match robin_startup::set_enabled(&history, id, enabled) {
        Ok(_) => 0,
        Err(_) => 1,
    })
}

/// 저장공간 정리 결과 파일 이름. 기록 폴더(`...\RobinX\storage`) 안에 쓴다.
pub const STORAGE_RESULT: &str = "elevated-result.json";

/// `--elevated-storage <irreversible:0|1> <규칙ID,규칙ID> --store <...\RobinX\storage>`
/// 규칙 ID만 받고, 경로와 조건은 엔진이 처음부터 다시 계산한다.
fn run_storage(rest: &[String]) -> Option<i32> {
    let allow_irreversible = match rest.first()?.as_str() {
        "1" => true,
        "0" => false,
        _ => return None,
    };
    let ids: Vec<String> = rest.get(1)?.split(',').map(str::to_string).collect();
    // 관리자 작업자는 관리자 권한이 필요한 규칙만 처리한다. 다른 관리자 계정으로 UAC를 허용하면
    // TEMP·LOCALAPPDATA가 그 계정 것을 가리키므로, 사용자 영역 규칙은 여기서 다루지 않는다.
    if ids.is_empty()
        || !ids
            .iter()
            .all(|id| robin_storage::rules::find(id).is_some_and(|r| r.admin))
    {
        return None;
    }
    let dir = store_arg(&rest[2..], "storage")?;
    let results = robin_storage::clean(&ids, allow_irreversible);
    std::fs::create_dir_all(&dir).ok()?;
    std::fs::write(dir.join(STORAGE_RESULT), serde_json::to_vec(&results).ok()?).ok()?;
    Some(0)
}

fn run(mode: &str, rest: &[String]) -> Option<i32> {
    let op_id = rest.first()?;
    if !robin_recovery::store::is_valid_id(op_id) {
        return None;
    }
    // 저장소 위치는 항상 ...\RobinX\backups 여야 한다
    let store_root = store_arg(&rest[1..], "backups")?;
    // 선택: --items 0,3,5 (복원할 항목 번호)
    let indices: Option<Vec<usize>> = match rest.get(3).map(String::as_str) {
        None => None,
        Some(ITEMS) => Some(
            rest.get(4)?
                .split(',')
                .map(|n| n.parse().ok())
                .collect::<Option<Vec<usize>>>()?,
        ),
        Some(_) => return None,
    };
    if rest.len() > 5 {
        return None;
    }
    // 매니페스트는 일반 사용자 권한으로 고칠 수 있다: store.load가 ID를 확인하고, clean/restore가
    // 백업 경로 모양·백업 내용·보호 규칙을 항목마다 다시 확인한다.
    let store = Store::new(&store_root);
    let mut op = store.load(op_id).ok()?;
    let guard = Guard {
        known: robin_scanner::protect::KnownFolders::from_env(),
        store_root: store.root().to_path_buf(),
        elevated: true,
        foreign_user: !same_user(&store_root),
    };

    if mode == CLEAN {
        // 관리자 권한이 필요했던 항목(일부만 정리된 항목 포함)만 다시 시도한다
        let targets: Vec<usize> = op
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| matches!(item.status, ItemStatus::NeedsAdmin | ItemStatus::Partial))
            .map(|(i, _)| i)
            .collect();
        if robin_recovery::clean_only(&store, &mut op, &guard, &targets, |_| {}).is_err() {
            return Some(EXIT_SAVE_FAILED);
        }
        let unfinished = targets.iter().any(|&i| {
            !matches!(
                op.items[i].status,
                ItemStatus::Removed | ItemStatus::AlreadyGone
            )
        });
        Some(if unfinished { EXIT_SOME_FAILED } else { 0 })
    } else {
        let ids: Option<Vec<String>> = indices.map(|idx| {
            idx.iter()
                .filter_map(|&i| op.items.get(i).map(|item| item.id.clone()))
                .collect()
        });
        if robin_recovery::restore(&store, &mut op, &guard, ids.as_deref(), |_| {}).is_err() {
            return Some(EXIT_SAVE_FAILED);
        }
        let failed = op.items.iter().any(|i| {
            i.error
                .as_deref()
                .is_some_and(|e| e.starts_with("복원 실패"))
        });
        Some(if failed { EXIT_SOME_FAILED } else { 0 })
    }
}

/// 작업자가 정리를 요청한 사용자와 같은 계정인지. 다른 관리자 계정으로 UAC를 허용하면 작업자의
/// `%LOCALAPPDATA%`가 달라져 저장소 경로가 그 아래에 있지 않다.
fn same_user(store_root: &std::path::Path) -> bool {
    std::env::var("LOCALAPPDATA")
        .is_ok_and(|local| robin_scanner::protect::is_within(&store_root.to_string_lossy(), &local))
}

/// 백업 하나를 지운다 (관리자 권한으로 격리한 폴더는 원래 권한이 그대로라 일반 권한으로 못 지울 수 있다).
/// 사용자가 화면에서 "영구 삭제"를 확인한 뒤에만 불린다. 작업 ID와 저장소 위치만 받는다.
fn run_delete(rest: &[String]) -> Option<i32> {
    let op_id = rest.first()?;
    if !robin_recovery::store::is_valid_id(op_id) || rest.len() != 3 {
        return None;
    }
    let store = Store::new(store_arg(&rest[1..], "backups")?);
    Some(if store.delete(op_id).is_ok() { 0 } else { 1 })
}
