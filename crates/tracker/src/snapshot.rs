//! 스냅샷 찍기 (읽기 전용). 프로그램이 설치되는 곳만, 정해진 깊이까지 본다.
//!
//! - 폴더: Program Files(64/32), ProgramData, AppData(Roaming·Local·Local\Programs) 4단계,
//!   시작 메뉴(사용자·공용) 3단계, 바탕 화면(사용자·공용) 1단계
//! - 레지스트리: `Software`(HKCU, HKLM 64/32비트) 3단계, `Software\Classes` 1단계,
//!   Uninstall·App Paths 하위 키, Run 값, 서비스 키
//! - 정션·심볼릭 링크는 따라가지 않는다.

use std::os::windows::fs::MetadataExt;
use std::path::Path;

use winreg::RegKey;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};

use crate::model::Snapshot;

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
/// 한 스냅샷에서 볼 항목 수 상한 (아주 큰 폴더 때문에 멈추지 않게). 파일과 레지스트리는 따로 센다.
/// 넘으면 스냅샷에 `incomplete`를 표시하고, 그런 스냅샷으로는 비교 결과를 저장하지 않는다.
const MAX_FILE_ENTRIES: usize = 2_000_000;
const MAX_KEY_ENTRIES: usize = 1_000_000;

fn env(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|v| !v.is_empty())
}

/// 바탕 화면에서는 설치 프로그램이 만드는 바로 가기만 기록한다 (사용자 파일은 추적 기록에 넣지 않는다)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    ShortcutsOnly,
}

/// (폴더, 깊이, 거르는 방법) 목록
fn file_roots() -> Vec<(String, usize, Filter)> {
    let mut roots: Vec<(String, usize, Filter)> = Vec::new();
    let mut push = |p: Option<String>, depth: usize, filter: Filter| {
        if let Some(p) = p
            && !roots.iter().any(|(r, _, _)| r.eq_ignore_ascii_case(&p))
        {
            roots.push((p, depth, filter));
        }
    };
    push(env("ProgramFiles"), 4, Filter::All);
    push(env("ProgramFiles(x86)"), 4, Filter::All);
    push(env("ProgramW6432"), 4, Filter::All);
    push(env("ProgramData"), 4, Filter::All);
    push(env("APPDATA"), 4, Filter::All);
    push(env("LOCALAPPDATA"), 4, Filter::All);
    push(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\Programs")),
        4,
        Filter::All,
    );
    push(
        env("APPDATA").map(|a| format!(r"{a}\Microsoft\Windows\Start Menu\Programs")),
        3,
        Filter::All,
    );
    push(
        env("ProgramData").map(|a| format!(r"{a}\Microsoft\Windows\Start Menu\Programs")),
        3,
        Filter::All,
    );
    push(
        env("USERPROFILE").map(|u| format!(r"{u}\Desktop")),
        1,
        Filter::ShortcutsOnly,
    );
    push(
        env("PUBLIC").map(|u| format!(r"{u}\Desktop")),
        1,
        Filter::ShortcutsOnly,
    );
    roots
}

fn is_shortcut(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk") || e.eq_ignore_ascii_case("url"))
}

fn walk_files(dir: &Path, depth: usize, filter: Filter, snap: &mut Snapshot, budget: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *budget == 0 {
            snap.incomplete = true;
            return;
        }
        *budget -= 1;
        let Ok(meta) = entry.metadata() else { continue };
        // 정션·심볼릭 링크는 따라가지도, 기록하지도 않는다
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            continue;
        }
        let path = entry.path();
        let text = path.to_string_lossy();
        if filter == Filter::ShortcutsOnly {
            if !meta.is_dir() && is_shortcut(&path) {
                snap.add_file(&text);
            }
        } else if meta.is_dir() {
            snap.add_folder(&text);
            if depth > 1 {
                walk_files(&path, depth - 1, filter, snap, budget);
            }
        } else {
            snap.add_file(&text);
        }
    }
}

#[derive(Clone, Copy)]
struct Hive {
    root: winreg::HKEY,
    view: u32,
    label: &'static str,
    /// 32비트 뷰: 표시 경로의 `SOFTWARE\` 뒤에 `WOW6432Node\`를 넣는다
    wow: bool,
}

const HKCU: Hive = Hive {
    root: HKEY_CURRENT_USER,
    view: 0,
    label: "HKCU",
    wow: false,
};
const HKLM64: Hive = Hive {
    root: HKEY_LOCAL_MACHINE,
    view: KEY_WOW64_64KEY,
    label: "HKLM",
    wow: false,
};
const HKLM32: Hive = Hive {
    root: HKEY_LOCAL_MACHINE,
    view: KEY_WOW64_32KEY,
    label: "HKLM",
    wow: true,
};

impl Hive {
    fn open(&self, path: &str) -> Option<RegKey> {
        RegKey::predef(self.root)
            .open_subkey_with_flags(path, KEY_READ | self.view)
            .ok()
    }

    /// 잔여물 스캐너·정리와 같은 표기 (`HKLM\SOFTWARE\WOW6432Node\...`)
    fn display(&self, path: &str) -> String {
        let shown = if self.wow && path.len() >= 9 && path[..9].eq_ignore_ascii_case("software\\") {
            format!("SOFTWARE\\WOW6432Node\\{}", &path[9..])
        } else {
            path.to_string()
        };
        format!("{}\\{}", self.label, shown)
    }
}

/// `Software` 아래에서 따로 보거나(Classes) 보지 않는(Microsoft·Policies·WOW6432Node) 키
const SKIP_UNDER_SOFTWARE: [&str; 4] = ["microsoft", "classes", "policies", "wow6432node"];

fn walk_keys(hive: Hive, path: &str, depth: usize, snap: &mut Snapshot, budget: &mut usize) {
    let Some(key) = hive.open(path) else { return };
    for name in key.enum_keys().flatten() {
        if *budget == 0 {
            snap.incomplete = true;
            return;
        }
        *budget -= 1;
        let child = format!(r"{path}\{name}");
        snap.add_key(&hive.display(&child));
        if depth > 1 {
            walk_keys(hive, &child, depth - 1, snap, budget);
        }
    }
}

fn record_values(hive: Hive, path: &str, snap: &mut Snapshot) {
    if let Some(key) = hive.open(path) {
        let shown = hive.display(path);
        for (name, _) in key.enum_values().flatten() {
            if !name.is_empty() {
                snap.add_value(&shown, &name);
            }
        }
    }
}

/// 지금 상태를 찍는다. 몇 초 걸릴 수 있다.
pub fn take() -> Snapshot {
    let mut snap = Snapshot {
        taken_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        ..Default::default()
    };
    let mut budget = MAX_FILE_ENTRIES;
    for (root, depth, filter) in file_roots() {
        walk_files(Path::new(&root), depth, filter, &mut snap, &mut budget);
    }
    let mut budget = MAX_KEY_ENTRIES;

    for hive in [HKCU, HKLM64, HKLM32] {
        // Software\X\Y\Z (Microsoft 등은 아래에서 필요한 곳만)
        if let Some(software) = hive.open("Software") {
            for name in software.enum_keys().flatten() {
                if SKIP_UNDER_SOFTWARE.contains(&name.to_lowercase().as_str()) {
                    continue;
                }
                let path = format!(r"Software\{name}");
                snap.add_key(&hive.display(&path));
                walk_keys(hive, &path, 2, &mut snap, &mut budget);
            }
        }
        for path in [
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            r"Software\Microsoft\Windows\CurrentVersion\App Paths",
        ] {
            walk_keys(hive, path, 1, &mut snap, &mut budget);
        }
        record_values(
            hive,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            &mut snap,
        );
    }
    // 확장자·ProgID 등록 (64비트 뷰와 HKCU만, 1단계)
    for hive in [HKCU, HKLM64] {
        walk_keys(hive, r"Software\Classes", 1, &mut snap, &mut budget);
    }
    walk_keys(
        HKLM64,
        r"SYSTEM\CurrentControlSet\Services",
        1,
        &mut snap,
        &mut budget,
    );

    snap.apps = robin_discovery::list_installed()
        .into_iter()
        .map(|a| a.id)
        .collect();
    snap
}
