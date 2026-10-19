//! 최적화: Windows 공식 설정을 원래 값을 기록한 뒤 바꾸고, 그대로 되돌린다.
//!
//! - 항목은 `catalog`에 데이터로 정의한다. 현재 사용자(HKCU) 설정만, 정책·보안 키는 쓰지 않는다.
//! - 적용: 지금 값을 읽어 **먼저 기록**하고 → 새 값을 쓴다. 하나라도 실패하면 이미 쓴 값을 원래대로 돌린다.
//! - 되돌리기: 기록한 원래 값으로 돌린다. 그 사이 다른 곳(설정 앱 등)에서 바꾼 값은 덮어쓰지 않고 알려 준다.
//! - 상태: 지금 값을 읽어 적용됨/일부/아님을 보여 준다 (읽기 전용).

pub mod catalog;
pub mod history;
pub mod model;
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
                changes: t
                    .changes
                    .iter()
                    .map(|c| ChangeView {
                        location: format!(r"HKCU\{}", c.key),
                        name: c.name.into(),
                        value: c.value.display(),
                    })
                    .collect(),
            }
        })
        .collect()
}

/// 지금 값이 바꿀 값과 같은 것이 몇 개인지로 상태를 정한다.
#[cfg(windows)]
pub fn state(t: &Tweak) -> State {
    let matches = t
        .changes
        .iter()
        .filter(|c| {
            registry::read(c.key, c.name)
                .ok()
                .flatten()
                .is_some_and(|v| v.same_as(&c.value.raw()))
        })
        .count();
    match matches {
        0 => State::NotApplied,
        n if n == t.changes.len() => State::Applied,
        _ => State::Partial,
    }
}

/// 적용한다. 원래 값을 먼저 기록하고(이미 기록이 있으면 그대로), 새 값을 쓴다.
#[cfg(windows)]
pub fn apply(history: &History, id: &str) -> Result<(), String> {
    let t = catalog::find(id).ok_or("알 수 없는 항목이에요")?;
    // 지금 값 읽기 (원래 값 후보)
    let mut originals = Vec::with_capacity(t.changes.len());
    for c in t.changes {
        let previous = registry::read(c.key, c.name).map_err(|e| {
            format!(
                "지금 값을 읽지 못해 바꾸지 않았어요 ({}): {e}",
                location(c.key, c.name)
            )
        })?;
        originals.push(history::Original {
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
                originals: originals.clone(),
                applied_at: now(),
            },
        )
        .map_err(|e| format!("원래 값을 기록하지 못해 바꾸지 않았어요: {e}"))?;

    for (i, c) in t.changes.iter().enumerate() {
        if let Err(e) = registry::write(c.key, c.name, &c.value.raw()) {
            // 이번에 쓴 값만 방금 읽은 값으로 되돌린다
            for (o, done) in originals.iter().zip(t.changes).take(i) {
                let _ = restore_one(o, &done.value.raw());
            }
            if newly {
                let _ = history.forget(id);
            }
            return Err(format!(
                "바꾸지 못했어요 ({}): {e}",
                location(c.key, c.name)
            ));
        }
    }
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

/// 원래 값 하나를 되돌린다. 지금 값이 우리가 쓴 값과 다르면(다른 곳에서 바꿈) 건드리지 않고 `Ok(false)`.
#[cfg(windows)]
fn restore_one(o: &history::Original, ours: &model::RawValue) -> std::io::Result<bool> {
    let current = registry::read(&o.key, &o.name)?;
    if !current.as_ref().is_some_and(|v| v.same_as(ours)) {
        // 이미 원래 값이면 할 일 없음, 다른 값이면 덮어쓰지 않는다
        return Ok(current == o.previous
            || current
                .zip(o.previous.as_ref())
                .is_some_and(|(c, p)| c.same_as(p)));
    }
    match &o.previous {
        Some(v) => registry::write(&o.key, &o.name, v)?,
        None => {
            registry::delete(&o.key, &o.name)?;
            if !o.key_existed {
                // 우리가 만든 키만, 아래에서 위로 비어 있으면 지운다
                let top = o.created_from.as_deref().unwrap_or(&o.key);
                let mut key = o.key.as_str();
                loop {
                    registry::delete_key_if_empty(key)?;
                    if key.eq_ignore_ascii_case(top) {
                        break;
                    }
                    match key.rsplit_once('\\') {
                        Some((parent, _)) => key = parent,
                        None => break,
                    }
                }
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
    let mut outcome = RevertOutcome::default();
    for o in &record.originals {
        // 기록이 지금 목록과 다르면(항목 정의가 바뀐 경우) 목록에 있는 값만 다룬다
        let Some(c) = t
            .changes
            .iter()
            .find(|c| c.key == o.key && c.name == o.name)
        else {
            continue;
        };
        match restore_one(o, &c.value.raw()) {
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
    Ok(outcome)
}
