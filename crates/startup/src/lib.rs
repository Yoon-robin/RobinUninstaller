//! 시작프로그램 목록과 끄기·원상복구.
//!
//! 대상: Run 키(HKCU, HKLM 64·32비트), 시작 폴더(사용자, 공용), 로그인·부팅 때 실행되는 예약 작업.
//! 끄기는 작업 관리자와 같은 방식(`StartupApproved` 값)이라 Run 값이나 바로 가기 파일은 그대로 남는다.
//! 예약 작업은 작업의 "사용" 여부(`Enabled`)만 바꾸고 작업 자체는 그대로 둔다.
//! RobinX가 끈 항목은 원래 값을 기록해 두었다가, 다시 켤 때 그대로 되돌린다.

pub mod history;
pub mod model;

#[cfg(windows)]
mod com;
#[cfg(windows)]
mod fileinfo;
#[cfg(windows)]
mod folder;
#[cfg(windows)]
mod registry;
#[cfg(windows)]
mod tasks;

pub use history::{Change, History};
pub use model::{
    CleanupRefusal, CleanupTarget, StartupEntry, StartupSource, cleanup_targets, is_valid_id,
};

#[derive(Debug)]
pub enum ToggleError {
    /// 목록에 없는 항목 (그 사이 지워졌거나 잘못된 ID)
    NotFound,
    /// HKLM·공용 폴더 항목이나 다른 계정의 작업이라 관리자 권한이 필요함
    NeedsAdmin,
    Failed(String),
}

impl std::fmt::Display for ToggleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleError::NotFound => write!(f, "시작프로그램 목록에 없는 항목이에요"),
            ToggleError::NeedsAdmin => write!(f, "관리자 권한이 필요해요"),
            ToggleError::Failed(m) => write!(f, "{m}"),
        }
    }
}

/// 시작 폴더 경로. Run 키 출처면 `None`. (잔여물 스캐너도 쓴다)
#[cfg(windows)]
pub fn startup_folder(source: StartupSource) -> Option<std::path::PathBuf> {
    folder::folder(source)
}

/// 시작 폴더 안의 파일 (하위 폴더와 desktop.ini는 뺀다). Run 키 출처면 빈 목록.
#[cfg(windows)]
pub fn startup_folder_files(source: StartupSource) -> Vec<std::path::PathBuf> {
    folder::files(source)
}

/// 바로 가기(.lnk)의 (대상 경로, 인자). 대상이 파일이 아닌 바로 가기는 대상이 빈 문자열.
#[cfg(windows)]
pub fn shortcut_target(path: &std::path::Path) -> Option<(String, String)> {
    folder::resolve_shortcut(path)
}

/// Run 키의 (값 이름, 명령) 목록. 시작 폴더 출처면 빈 목록.
#[cfg(windows)]
pub fn run_values(source: StartupSource) -> Vec<(String, String)> {
    if folder::folder(source).is_some() {
        return Vec::new();
    }
    registry::run_values(source)
}

/// `StartupApproved` 키 위치 (화면 표시·정리용, 예: `HKCU\Software\...\StartupApproved\Run`).
pub fn approved_location(source: StartupSource) -> String {
    let hive = if source.is_machine() { "HKLM" } else { "HKCU" };
    format!(
        r"{hive}\{}\{}",
        model::APPROVED_KEY,
        source.approved_subkey()
    )
}

/// `StartupApproved` 키에 있는 값 이름들. 읽기 전용.
#[cfg(windows)]
pub fn approved_names(source: StartupSource) -> Vec<String> {
    registry::approved_names(source)
}

/// `StartupApproved` 값이 있는지. 읽기 전용.
#[cfg(windows)]
pub fn has_approved(source: StartupSource, name: &str) -> bool {
    registry::read_approved(source, name).is_some()
}

/// 모든 출처 (Run 키·시작 폴더, 그다음 예약 작업)
#[cfg(windows)]
const EVERY_SOURCE: [StartupSource; 6] = [
    StartupSource::RunUser,
    StartupSource::RunMachine,
    StartupSource::RunMachine32,
    StartupSource::FolderUser,
    StartupSource::FolderCommon,
    StartupSource::Task,
];

/// 출처 하나의 시작 항목. 읽기 전용.
#[cfg(windows)]
fn entries_for(
    source: StartupSource,
    changed: &std::collections::BTreeMap<String, Change>,
) -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    if source == StartupSource::Task {
        for task in tasks::list() {
            entries.push(build_task(task, changed));
        }
    } else if let Some(dir) = folder::folder(source) {
        for path in folder::files(source) {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let full = path.to_string_lossy().into_owned();
            let is_link = path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));
            let command = match is_link.then(|| folder::resolve_shortcut(&path)).flatten() {
                Some((target, args)) if !target.is_empty() => {
                    format!("\"{target}\" {args}").trim_end().to_string()
                }
                // 대상을 알 수 없는 바로 가기, 또는 스크립트·실행 파일 자체
                _ => format!("\"{full}\""),
            };
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| file_name.clone());
            entries.push(build(
                source,
                name,
                file_name,
                dir.to_string_lossy().into_owned(),
                command,
                changed,
            ));
        }
    } else {
        for (name, command) in registry::run_values(source) {
            entries.push(build(
                source,
                name.clone(),
                name,
                registry::run_location(source),
                command,
                changed,
            ));
        }
    }
    entries
}

/// ID로 항목 하나를 찾는다. 출처를 알면 그 출처만 읽고, 모르면 찾을 때까지 출처를 하나씩 읽는다
/// (예약 작업·파일 버전 정보까지 전체 목록을 매번 읽지 않도록).
#[cfg(windows)]
fn find_entry(history: &History, id: &str, source: Option<StartupSource>) -> Option<StartupEntry> {
    let changed = history.load();
    let sources: Vec<StartupSource> = match source {
        Some(s) => vec![s],
        None => EVERY_SOURCE.to_vec(),
    };
    sources
        .into_iter()
        .find_map(|s| entries_for(s, &changed).into_iter().find(|e| e.id == id))
}

/// 시작 항목 전체 (이름순). 읽기 전용.
#[cfg(windows)]
pub fn list(history: &History) -> Vec<StartupEntry> {
    let changed = history.load();
    let mut entries: Vec<StartupEntry> = EVERY_SOURCE
        .into_iter()
        .flat_map(|source| entries_for(source, &changed))
        .collect();

    entries.sort_by_key(|e| e.name.to_lowercase());
    entries
}

#[cfg(windows)]
fn build_task(
    task: tasks::Task,
    changed: &std::collections::BTreeMap<String, Change>,
) -> StartupEntry {
    // 폴더를 가리키는 바로 가기도 있으므로 파일이 아니라 "있는지"로 본다
    let exists = |p: &str| std::path::Path::new(p).exists();
    let target = task
        .command
        .as_deref()
        .and_then(|c| model::resolve_target(c, |v| std::env::var(v).ok(), exists));
    let target_missing = target.as_deref().is_some_and(|p| !exists(p));
    let info = match &target {
        Some(p) if !target_missing => fileinfo::read(p),
        _ => fileinfo::FileInfo::default(),
    };
    let id = model::make_id(StartupSource::Task, &task.path);
    // 작업 스케줄러는 끈 시각을 남기지 않는다. 우리가 끈 것이면 기록의 시각을 쓴다.
    let change = changed.get(&id).filter(|_| !task.enabled);

    StartupEntry {
        changed_by_us: change.is_some(),
        disabled_at: change.map(|c| c.changed_at),
        id,
        name: task.name,
        source: StartupSource::Task,
        location: "작업 스케줄러".to_string(),
        approved_name: task.path,
        // 프로그램 실행이 아닌 작업(COM 처리기 등)
        command: task
            .command
            .unwrap_or_else(|| "(프로그램 실행이 아닌 작업)".to_string()),
        target_path: target,
        target_missing,
        publisher: info.company,
        description: info.description,
        author: task.author,
        needs_admin: task.needs_admin,
        enabled: task.enabled,
    }
}

#[cfg(windows)]
fn build(
    source: StartupSource,
    name: String,
    approved_name: String,
    location: String,
    command: String,
    changed: &std::collections::BTreeMap<String, Change>,
) -> StartupEntry {
    // 폴더를 가리키는 바로 가기도 있으므로 파일이 아니라 "있는지"로 본다
    let exists = |p: &str| std::path::Path::new(p).exists();
    let target = model::resolve_target(&command, |v| std::env::var(v).ok(), exists);
    let target_missing = target.as_deref().is_some_and(|p| !exists(p));
    let info = match &target {
        Some(p) if !target_missing => fileinfo::read(p),
        _ => fileinfo::FileInfo::default(),
    };
    let (enabled, disabled_at) =
        model::parse_approved(registry::read_approved(source, &approved_name).as_deref());
    let id = model::make_id(source, &approved_name);

    StartupEntry {
        changed_by_us: !enabled && changed.contains_key(&id),
        id,
        name,
        source,
        location,
        approved_name,
        command,
        target_path: target,
        target_missing,
        publisher: info.company,
        description: info.description,
        author: None,
        needs_admin: source.is_machine(),
        enabled,
        disabled_at,
    }
}

/// 항목을 끄거나 다시 켠다. 끌 때는 원래 값을 기록하고, 켤 때는 그 값으로 되돌린다.
/// 바뀐 뒤의 항목을 돌려준다.
#[cfg(windows)]
pub fn set_enabled(
    history: &History,
    id: &str,
    enabled: bool,
) -> Result<StartupEntry, ToggleError> {
    if !is_valid_id(id) {
        return Err(ToggleError::NotFound);
    }
    let entry = find_entry(history, id, None).ok_or(ToggleError::NotFound)?;
    // 바꾼 뒤에는 그 출처만 다시 읽는다
    let source = entry.source;
    let find = || find_entry(history, id, Some(source));
    let io_err = |e: std::io::Error| match e.kind() {
        std::io::ErrorKind::PermissionDenied => ToggleError::NeedsAdmin,
        _ => ToggleError::Failed(e.to_string()),
    };
    let save_err =
        |e: std::io::Error| ToggleError::Failed(format!("변경 기록을 저장하지 못했어요: {e}"));

    if entry.source == StartupSource::Task {
        return set_task_enabled(history, entry, enabled, find);
    }

    if entry.enabled == enabled {
        if enabled {
            // 다른 곳에서 이미 다시 켰다면 남은 기록은 더 이상 맞지 않는다
            history.forget(id).map_err(save_err)?;
        }
        return Ok(entry);
    }

    if enabled {
        let restore = match history.get(id) {
            // 원래 값이 "켜짐"이었으면 그대로, 원래 값이 없었으면 값을 지운다
            Some(change) => match change.previous {
                Some(prev) if model::parse_approved(Some(&prev)).0 => Some(prev),
                Some(_) => Some(model::enabled_value()),
                None => None,
            },
            // 다른 곳(작업 관리자 등)에서 끈 항목은 작업 관리자와 같은 값으로 켠다
            None => Some(model::enabled_value()),
        };
        registry::write_approved(entry.source, &entry.approved_name, restore.as_deref())
            .map_err(io_err)?;
        history.forget(id).map_err(save_err)?;
    } else {
        // 켜진 상태에서 남아 있는 기록은 낡은 것이다. 지금 값을 원래 값으로 새로 기록한다.
        history.forget(id).map_err(save_err)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // 쓰는 도중 꺼져도 원래 값을 잃지 않도록 기록을 먼저 남긴다
        history
            .remember(
                id,
                Change {
                    previous: registry::read_approved(entry.source, &entry.approved_name),
                    previous_enabled: None,
                    name: entry.name.clone(),
                    changed_at: now,
                },
            )
            .map_err(save_err)?;
        if let Err(e) = registry::write_approved(
            entry.source,
            &entry.approved_name,
            Some(&model::disabled_value(now)),
        ) {
            let _ = history.forget(id);
            return Err(io_err(e));
        }
    }

    find().ok_or(ToggleError::NotFound)
}

/// 예약 작업 끄기·켜기: 작업의 `Enabled` 값만 바꾸고, 끄기 전 값을 기록해 두었다가 켤 때 되돌린다.
#[cfg(windows)]
fn set_task_enabled(
    history: &History,
    entry: StartupEntry,
    enabled: bool,
    find: impl Fn() -> Option<StartupEntry>,
) -> Result<StartupEntry, ToggleError> {
    let id = entry.id.clone();
    let io_err = |e: std::io::Error| match e.kind() {
        std::io::ErrorKind::PermissionDenied => ToggleError::NeedsAdmin,
        _ => ToggleError::Failed(e.to_string()),
    };
    let save_err =
        |e: std::io::Error| ToggleError::Failed(format!("변경 기록을 저장하지 못했어요: {e}"));

    if entry.enabled == enabled {
        if enabled {
            history.forget(&id).map_err(save_err)?;
        }
        return Ok(entry);
    }

    if enabled {
        // 끄기 전 값이 기록돼 있으면 그 값으로 (끌 때는 항상 켜져 있었으므로 보통 "사용")
        let restore = history
            .get(&id)
            .and_then(|c| c.previous_enabled)
            .unwrap_or(true);
        tasks::set_enabled(&entry.approved_name, restore).map_err(io_err)?;
        history.forget(&id).map_err(save_err)?;
    } else {
        history.forget(&id).map_err(save_err)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // 바꾸는 도중 꺼져도 원래 값을 잃지 않도록 기록을 먼저 남긴다
        history
            .remember(
                &id,
                Change {
                    previous: None,
                    previous_enabled: Some(entry.enabled),
                    name: entry.name.clone(),
                    changed_at: now,
                },
            )
            .map_err(save_err)?;
        if let Err(e) = tasks::set_enabled(&entry.approved_name, false) {
            let _ = history.forget(&id);
            return Err(io_err(e));
        }
    }

    find().ok_or(ToggleError::NotFound)
}
