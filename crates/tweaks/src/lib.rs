//! 최적화: Windows 공식 설정을 원래 값을 기록한 뒤 바꾸고, 그대로 되돌린다.
//!
//! - 항목은 `catalog`에 데이터로 정의한다. 현재 사용자(HKCU) 설정만, 정책·보안 키는 쓰지 않는다.
//!   켜기/끄기 항목과, 여러 값 중 하나를 고르는 선택지 항목(작업 표시줄 정렬 등)이 있다.
//! - 적용: 지금 값을 읽어 **먼저 기록**하고 → 새 값을 쓰고 → **다시 읽어 확인**한다.
//!   하나라도 실패하면 이번에 바꾼 값을 바꾸기 전 값으로 돌린다.
//! - 되돌리기: 기록한 원래 값으로 돌린다. 그 사이 다른 곳(설정 앱 등)에서 바꾼 값은 덮어쓰지 않고 알려 준다.
//! - 적용·되돌리기 뒤 탐색기·작업 표시줄에 설정 변경 알림을 보내, 탐색기를 다시 시작하지 않아도 반영되게 한다.
//! - 상태: 지금 값을 읽어 보여 준다 (읽기 전용).

pub mod catalog;
pub mod history;
pub mod model;
#[cfg(windows)]
mod notify;
#[cfg(windows)]
mod registry;

pub use history::History;
pub use model::{Group, Restart, State, Tweak};

use serde::Serialize;

/// 화면에 보여 줄 레지스트리 변경 하나
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeView {
    pub location: String,
    pub name: String,
    pub value: String,
}

/// 선택지 하나
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
    pub id: String,
    pub label: String,
}

/// 화면에 보여 줄 항목 하나
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakView {
    pub id: String,
    pub group: Group,
    pub title: String,
    pub description: String,
    pub why: String,
    pub settings_path: String,
    pub restart: Restart,
    pub state: State,
    /// RobinX가 바꿨고 원래 값 기록이 있어 되돌릴 수 있음
    pub changed_by_us: bool,
    pub applied_at: Option<u64>,
    pub changes: Vec<ChangeView>,
    /// 선택지 항목이면 선택지 목록 (켜기/끄기 항목이면 비어 있다)
    pub options: Vec<OptionView>,
    /// 지금 값에 맞는 선택지. 어느 것에도 맞지 않으면 `None` (다른 값)
    pub selected: Option<String>,
}

/// 되돌리기 결과
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevertOutcome {
    /// 다른 곳에서 바뀌어 그대로 둔 값 (`HKCU\경로 → 이름`)
    pub left_as_is: Vec<String>,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn location(key: &str, name: &str) -> String {
    format!(r"HKCU\{key} → {name}")
}

/// Windows 빌드 번호 (모르면 0)
#[cfg(windows)]
fn windows_build() -> u32 {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|k| k.get_value::<String, _>("CurrentBuildNumber"))
        .ok()
        .and_then(|b| b.parse().ok())
        .unwrap_or(0)
}

#[cfg(windows)]
fn matches(changes: &[model::RegChange]) -> usize {
    changes
        .iter()
        .filter(|c| {
            registry::read(c.key, c.name)
                .ok()
                .flatten()
                .is_some_and(|v| v.same_as(&c.value.raw()))
        })
        .count()
}

/// 선택지 항목에서 지금 값에 맞는 선택지. 값이 하나도 없으면 Windows 기본 선택지.
#[cfg(windows)]
pub fn selected(t: &Tweak) -> Option<&'static str> {
    if let Some(o) = t
        .options
        .iter()
        .find(|o| matches(o.changes) == o.changes.len())
    {
        return Some(o.id);
    }
    let all_missing = t
        .all_changes()
        .iter()
        .all(|c| matches!(registry::read(c.key, c.name), Ok(None)));
    if all_missing { t.default_option } else { None }
}

/// 켜기/끄기 항목: 바꿀 값과 같은 것이 몇 개인지. 선택지 항목: 어떤 선택지에든 맞으면 적용됨.
#[cfg(windows)]
pub fn state(t: &Tweak) -> State {
    if t.is_choice() {
        return if selected(t).is_some() {
            State::Applied
        } else {
            State::NotApplied
        };
    }
    match matches(t.changes) {
        0 => State::NotApplied,
        n if n == t.changes.len() => State::Applied,
        _ => State::Partial,
    }
}

/// 이 PC에서 보여 줄 항목과 현재 상태 (읽기 전용).
#[cfg(windows)]
pub fn list(history: &History) -> Vec<TweakView> {
    let build = windows_build();
    let records = history.load();
    catalog::all()
        .into_iter()
        .filter(|t| build == 0 || build >= t.min_build)
        .map(|t| {
            let record = records.get(t.id);
            let view = |c: &model::RegChange, prefix: &str| ChangeView {
                location: format!(r"HKCU\{}", c.key),
                name: c.name.into(),
                value: format!("{prefix}{}", c.value.display()),
            };
            let changes = if t.is_choice() {
                t.options
                    .iter()
                    .flat_map(|o| o.changes.iter().map(move |c| (o.label, c)))
                    .map(|(label, c)| view(c, &format!("{label}: ")))
                    .collect()
            } else {
                t.changes.iter().map(|c| view(c, "")).collect()
            };
            TweakView {
                id: t.id.into(),
                group: t.group,
                title: t.title.into(),
                description: t.description.into(),
                why: t.why.into(),
                settings_path: t.settings_path.into(),
                restart: t.restart,
                state: state(t),
                changed_by_us: record.is_some(),
                applied_at: record.map(|r| r.applied_at),
                changes,
                options: t
                    .options
                    .iter()
                    .map(|o| OptionView {
                        id: o.id.into(),
                        label: o.label.into(),
                    })
                    .collect(),
                selected: if t.is_choice() {
                    selected(t).map(str::to_string)
                } else {
                    None
                },
            }
        })
        .collect()
}

/// 값 하나를 그대로 놓는다 (`None`이면 지운다).
#[cfg(windows)]
fn put(key: &str, name: &str, value: Option<&model::RawValue>) -> std::io::Result<()> {
    match value {
        Some(v) => registry::write(key, name, v),
        None => registry::delete(key, name),
    }
}

/// 다시 읽어서 기대한 값인지 확인한다.
#[cfg(windows)]
fn verify(key: &str, name: &str, expected: Option<&model::RawValue>) -> Result<(), String> {
    let now = registry::read(key, name).map_err(|e| e.to_string())?;
    let ok = match (now.as_ref(), expected) {
        (Some(a), Some(b)) => a.same_as(b),
        (None, None) => true,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "썼지만 다시 읽어 보니 다른 값이에요 ({}). 정책이나 다른 프로그램이 막고 있을 수 있어요",
            location(key, name)
        ))
    }
}

/// 적용한다. 켜기/끄기 항목은 `option` 없이, 선택지 항목은 선택지 ID로.
/// 처음 바꿀 때 원래 값을 먼저 기록하고(이미 기록이 있으면 그대로), 새 값을 쓰고, 다시 읽어 확인한다.
#[cfg(windows)]
pub fn apply(history: &History, id: &str, option: Option<&str>) -> Result<(), String> {
    let t = catalog::find(id).ok_or("알 수 없는 항목이에요")?;
    let targets = t.targets(option).ok_or("고를 수 없는 값이에요")?;

    // 바꾸기 전 값 (원래 값 후보이자, 실패했을 때 되돌릴 값)
    let mut before = Vec::new();
    for c in t.all_changes() {
        let previous = registry::read(c.key, c.name).map_err(|e| {
            format!(
                "지금 값을 읽지 못해 바꾸지 않았어요 ({}): {e}",
                location(c.key, c.name)
            )
        })?;
        before.push(history::Original {
            key: c.key.into(),
            name: c.name.into(),
            previous,
            key_existed: registry::key_exists(c.key),
            created_from: topmost_missing(c.key),
        });
    }
    let newly = history
        .remember(
            id,
            history::Record {
                originals: before.clone(),
                applied_at: now(),
                option: option.map(str::to_string),
            },
        )
        .map_err(|e| format!("원래 값을 기록하지 못해 바꾸지 않았어요: {e}"))?;

    let mut done: Vec<&model::RegChange> = Vec::new();
    let result = (|| {
        for c in targets {
            let raw = c.value.raw();
            registry::write(c.key, c.name, &raw)
                .map_err(|e| format!("바꾸지 못했어요 ({}): {e}", location(c.key, c.name)))?;
            done.push(c);
            verify(c.key, c.name, Some(&raw))?;
        }
        Ok::<(), String>(())
    })();

    if let Err(e) = result {
        // 이번에 쓴 값만 바꾸기 전 값으로 돌린다
        for c in done {
            if let Some(b) = before.iter().find(|b| b.key == c.key && b.name == c.name) {
                let _ = put(c.key, c.name, b.previous.as_ref());
            }
        }
        if newly {
            let _ = history.forget(id);
        }
        return Err(e);
    }
    if !newly {
        // 선택지를 바꿨으면 "지금 쓴 값"을 기록에 맞춘다 (원래 값은 처음 것 그대로)
        history
            .set_option(id, option)
            .map_err(|e| format!("바꿨지만 기록을 고치지 못했어요: {e}"))?;
    }
    notify::settings_changed(t.restart);
    Ok(())
}

/// `key`와 그 상위 중 지금 없는 가장 위쪽 키. 모두 있으면 `None`.
#[cfg(windows)]
fn topmost_missing(key: &str) -> Option<String> {
    let mut top = None;
    let mut k = key;
    while !registry::key_exists(k) {
        top = Some(k.to_string());
        match k.rsplit_once('\\') {
            Some((parent, _)) => k = parent,
            None => break,
        }
    }
    top
}

/// 원래 값 하나를 되돌린다. 지금 값이 우리가 쓴 값(`ours`)과 다르면(다른 곳에서 바꿈) 건드리지 않고 `Ok(false)`.
/// 되돌린 뒤 다시 읽어 확인한다.
#[cfg(windows)]
fn restore_one(o: &history::Original, ours: Option<&model::RawValue>) -> Result<bool, String> {
    let current = registry::read(&o.key, &o.name).map_err(|e| e.to_string())?;
    let same = |a: Option<&model::RawValue>, b: Option<&model::RawValue>| match (a, b) {
        (Some(a), Some(b)) => a.same_as(b),
        (None, None) => true,
        _ => false,
    };
    if !ours.is_some_and(|v| same(current.as_ref(), Some(v))) {
        // 이미 원래 값이면 할 일 없음, 다른 값이면 덮어쓰지 않는다
        return Ok(same(current.as_ref(), o.previous.as_ref()));
    }
    put(&o.key, &o.name, o.previous.as_ref()).map_err(|e| e.to_string())?;
    verify(&o.key, &o.name, o.previous.as_ref())?;
    if o.previous.is_none() && !o.key_existed {
        // 우리가 만든 키만, 아래에서 위로 비어 있으면 지운다
        let top = o.created_from.as_deref().unwrap_or(&o.key);
        let mut key = o.key.as_str();
        loop {
            registry::delete_key_if_empty(key).map_err(|e| e.to_string())?;
            if key.eq_ignore_ascii_case(top) {
                break;
            }
            match key.rsplit_once('\\') {
                Some((parent, _)) => key = parent,
                None => break,
            }
        }
    }
    Ok(true)
}

/// RobinX가 바꾼 항목을 원래 값으로 되돌린다.
#[cfg(windows)]
pub fn revert(history: &History, id: &str) -> Result<RevertOutcome, String> {
    let t = catalog::find(id).ok_or("알 수 없는 항목이에요")?;
    let record = history
        .get(id)
        .ok_or("RobinX가 바꾼 기록이 없어요. Windows 설정에서 직접 바꿔 주세요")?;
    // 지금 적용돼 있어야 할 값 (선택지 항목이면 마지막으로 고른 선택지)
    let ours = t.targets(record.option.as_deref()).unwrap_or(&[]);
    let mut outcome = RevertOutcome::default();
    for o in &record.originals {
        // 기록이 지금 목록과 다르면(항목 정의가 바뀐 경우) 목록에 있는 값만 다룬다
        if !t
            .all_changes()
            .iter()
            .any(|c| c.key == o.key && c.name == o.name)
        {
            continue;
        }
        let our = ours
            .iter()
            .find(|c| c.key == o.key && c.name == o.name)
            .map(|c| c.value.raw());
        match restore_one(o, our.as_ref()) {
            Ok(true) => {}
            Ok(false) => outcome.left_as_is.push(location(&o.key, &o.name)),
            Err(e) => {
                return Err(format!(
                    "되돌리지 못했어요 ({}): {e}. 기록은 남겨 두었어요",
                    location(&o.key, &o.name)
                ));
            }
        }
    }
    history
        .forget(id)
        .map_err(|e| format!("되돌렸지만 기록을 정리하지 못했어요: {e}"))?;
    notify::settings_changed(t.restart);
    Ok(outcome)
}
