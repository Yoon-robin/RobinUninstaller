//! 시작프로그램 흔적 찾기 (읽기 전용).
//!
//! - 시작 폴더(사용자·공용)의 바로 가기 중 대상이 설치 폴더 안인 것
//! - 잔여물로 찾은 Run 값·바로 가기와 이름이 같은 `StartupApproved` 값 (켜짐·꺼짐 상태)
//! - 짝(Run 값·바로 가기)이 이미 없어진 `StartupApproved` 값 중 이름이 프로그램과 맞는 것 (검토)

use std::collections::HashSet;
use std::path::Path;

use robin_startup::StartupSource;
use robin_startup::model::ALL_SOURCES;

use crate::context::ScanContext;
use crate::model::{Evidence, Leftover, LeftoverKind};
use crate::names::{NameMatch, match_name};
use crate::protect::{is_within, protected_path_reason};

const FOLDER_SOURCES: [StartupSource; 2] = [StartupSource::FolderUser, StartupSource::FolderCommon];

pub fn scan(ctx: &ScanContext, out: &mut Vec<Leftover>, protected: &mut usize) {
    // (출처, StartupApproved 값 이름) — 잔여물로 찾은 시작 항목
    let mut startup_items: Vec<(StartupSource, String)> = out
        .iter()
        .filter(|i| i.kind == LeftoverKind::RegistryValue)
        .filter_map(|i| Some((run_source(&i.location)?, i.value_name.clone()?)))
        .collect();

    if let Some(dir) = &ctx.install_location {
        for source in FOLDER_SOURCES {
            for path in robin_startup::startup_folder_files(source) {
                if let Some(item) = shortcut(ctx, &path, dir, protected) {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    startup_items.push((source, file_name));
                    out.push(item);
                }
            }
        }
    }

    let mut taken: HashSet<(StartupSource, String)> = HashSet::new();
    for (source, name) in startup_items {
        if robin_startup::has_approved(source, &name) {
            let mut item = approved(source, &name, vec![Evidence::PointsIntoInstallLocation]);
            item.note = Some(format!(
                "{}의 시작프로그램 켜짐·꺼짐 상태 값이에요.",
                ctx.app_name
            ));
            out.push(item);
        }
        taken.insert((source, name.to_lowercase()));
    }

    orphans(ctx, &taken, out);
}

/// 대상이 설치 폴더 안인 시작 폴더 바로 가기
fn shortcut(ctx: &ScanContext, path: &Path, dir: &str, protected: &mut usize) -> Option<Leftover> {
    let is_link = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));
    if !is_link {
        return None;
    }
    let (target, _) = robin_startup::shortcut_target(path)?;
    if target.is_empty() || !is_within(&target, dir) {
        return None;
    }
    let location = path.to_string_lossy().into_owned();
    if protected_path_reason(&location, &ctx.known).is_some() || ctx.is_self(&location) {
        *protected += 1;
        return None;
    }
    let mut item = Leftover::new(
        LeftoverKind::File,
        location,
        None,
        vec![Evidence::PointsIntoInstallLocation],
    );
    item.size_bytes = Some(std::fs::metadata(path).map(|m| m.len()).unwrap_or(0));
    item.note = Some(format!(
        "{} 시작 시 자동 실행되는 바로 가기예요.",
        ctx.app_name
    ));
    Some(item)
}

/// 짝이 없어진 `StartupApproved` 값. 경로로는 확인할 수 없으니 이름이 맞을 때만, 검토 수준으로.
fn orphans(ctx: &ScanContext, taken: &HashSet<(StartupSource, String)>, out: &mut Vec<Leftover>) {
    for source in ALL_SOURCES {
        let names = robin_startup::approved_names(source);
        if names.is_empty() {
            continue;
        }
        let folder = robin_startup::startup_folder(source);
        let run: Vec<String> = robin_startup::run_values(source)
            .into_iter()
            .map(|(n, _)| n.to_lowercase())
            .collect();
        for name in names {
            let lower = name.to_lowercase();
            if taken.contains(&(source, lower.clone())) {
                continue;
            }
            let still_there = match &folder {
                Some(dir) => dir.join(&name).exists(),
                None => run.contains(&lower),
            };
            if still_there {
                continue;
            }
            let stem = match &folder {
                Some(_) => Path::new(&name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| name.clone()),
                None => name.clone(),
            };
            let mut evidence = match match_name(&stem, &ctx.app_name, ctx.publisher.as_deref()) {
                NameMatch::Exact => vec![Evidence::ExactName],
                NameMatch::AllSignificant => vec![Evidence::PartialName],
                NameMatch::None => continue,
            };
            if let Some(other) = ctx.other_app_named(&stem) {
                evidence.push(Evidence::SharedWithOtherApp { app_name: other });
            }
            let mut item = approved(source, &name, evidence);
            item.note = Some(
                "짝이 되는 시작 항목이 이미 없어진 켜짐·꺼짐 상태 값이에요. 이름만 맞아서 확인이 필요해요."
                    .into(),
            );
            out.push(item);
        }
    }
}

fn approved(source: StartupSource, name: &str, evidence: Vec<Evidence>) -> Leftover {
    Leftover::new(
        LeftoverKind::RegistryValue,
        robin_startup::approved_location(source),
        Some(name.to_string()),
        evidence,
    )
}

/// 레지스트리 스캔이 찾은 Run 값의 위치 → 시작 항목 출처 (RunOnce 등은 `StartupApproved`가 없다)
fn run_source(location: &str) -> Option<StartupSource> {
    const RUN: &str = r"microsoft\windows\currentversion\run";
    match location.to_lowercase().as_str() {
        l if l == format!(r"hkcu\software\{RUN}") => Some(StartupSource::RunUser),
        l if l == format!(r"hklm\software\{RUN}") => Some(StartupSource::RunMachine),
        l if l == format!(r"hklm\software\wow6432node\{RUN}") => Some(StartupSource::RunMachine32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_locations_map_to_sources() {
        assert_eq!(
            run_source(r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
            Some(StartupSource::RunUser)
        );
        assert_eq!(
            run_source(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
            Some(StartupSource::RunMachine)
        );
        assert_eq!(
            run_source(r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run"),
            Some(StartupSource::RunMachine32)
        );
        assert_eq!(
            run_source(r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce"),
            None
        );
    }

    #[test]
    fn approved_locations_are_hive_specific() {
        assert_eq!(
            robin_startup::approved_location(StartupSource::RunMachine32),
            r"HKLM\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32"
        );
        assert_eq!(
            robin_startup::approved_location(StartupSource::FolderUser),
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder"
        );
    }
}
