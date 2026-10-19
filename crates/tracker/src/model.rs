//! 스냅샷·변경·추적 기록 모델과 비교. 순수 로직 (시스템 접근 없음).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Folder,
    File,
    RegistryKey,
    RegistryValue,
}

/// 설치하는 동안 새로 생긴 것 하나. 위치 표기는 잔여물 스캐너·정리와 같다
/// (`C:\...`, `HKCU\Software\...`, `HKLM\SOFTWARE\WOW6432Node\...`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub kind: ChangeKind,
    pub location: String,
    pub value_name: Option<String>,
}

/// 한 시점의 파일·레지스트리 목록. 키는 비교용 소문자, 값은 원래 표기.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub taken_at: u64,
    pub folders: BTreeMap<String, String>,
    pub files: BTreeMap<String, String>,
    pub keys: BTreeMap<String, String>,
    /// `키\0값 이름`(소문자) → (키, 값 이름)
    pub values: BTreeMap<String, (String, String)>,
    /// 설치된 프로그램 ID (새로 설치된 프로그램을 알아내는 데 쓴다)
    pub apps: Vec<String>,
    /// 항목이 너무 많아 끝까지 보지 못했음. 이런 스냅샷으로 비교하면 원래 있던 것이 "새로 생김"으로 보일 수 있다.
    #[serde(default)]
    pub incomplete: bool,
}

impl Snapshot {
    pub fn add_folder(&mut self, path: &str) {
        self.folders.insert(path.to_lowercase(), path.to_string());
    }
    pub fn add_file(&mut self, path: &str) {
        self.files.insert(path.to_lowercase(), path.to_string());
    }
    pub fn add_key(&mut self, key: &str) {
        self.keys.insert(key.to_lowercase(), key.to_string());
    }
    pub fn add_value(&mut self, key: &str, name: &str) {
        self.values.insert(
            format!("{}\0{}", key.to_lowercase(), name.to_lowercase()),
            (key.to_string(), name.to_string()),
        );
    }
}

/// 설치 전후를 비교한 결과 (사용자가 고르기 전).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDraft {
    /// 스냅샷 하나라도 끝까지 보지 못해 결과를 믿을 수 없음
    #[serde(default)]
    pub incomplete: bool,
    pub started_at: u64,
    pub finished_at: u64,
    pub changes: Vec<Change>,
    /// 새로 생긴 설치 항목 (프로그램 ID)
    pub new_app_ids: Vec<String>,
}

/// 저장한 추적 기록.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub id: String,
    pub name: String,
    pub started_at: u64,
    pub finished_at: u64,
    /// 이 기록과 연결된 프로그램 ID. 잔여물 스캔은 ID가 같은 프로그램에만 이 기록을 쓴다.
    pub app_ids: Vec<String>,
    pub changes: Vec<Change>,
}

/// `path`가 `parent`의 하위인지 (소문자 경로끼리, 구분자 경계).
fn is_under(path: &str, parent: &str) -> bool {
    path.len() > parent.len() && path.starts_with(parent) && path.as_bytes()[parent.len()] == b'\\'
}

/// 설치하는 동안 계속 바뀌는 곳 (브라우저·Windows·임시 파일). 설치와 관계없는 변화가 섞이지 않게 뺀다.
const NOISE: &[&str] = &[
    r"\appdata\local\temp\",
    r"\windows\temp\",
    r"\appdata\local\packages\",
    r"\appdata\local\microsoft\",
    r"\appdata\roaming\microsoft\",
    r"\programdata\microsoft\",
    r"\appdata\local\crashdumps\",
    r"\appdata\local\d3dscache\",
    r"\appdata\local\nvidia\",
    r"\inetcache\",
    r"\webcache\",
    r"\google\chrome\user data\",
    r"\bravesoftware\brave-browser\user data\",
    r"\vivaldi\user data\",
    r"\mozilla\firefox\profiles\",
    r"\robinx\",
    r"\robinuninstaller\", // 예전 이름
    r"\microsoft\windows\wer\",
    r"\software\microsoft\",
    r"\software\wow6432node\microsoft\",
    r"\software\classes\local settings\",
];

/// 설치와 관계없이 늘 바뀌는 위치인지. 위치 끝에 `\`를 붙여 비교한다.
pub fn is_noise(location: &str) -> bool {
    let l = format!("{}\\", location.to_lowercase());
    // Uninstall·Run·App Paths는 Microsoft 아래지만 설치 흔적의 핵심이라 남긴다
    const KEEP: &[&str] = &[
        r"\currentversion\uninstall\",
        r"\currentversion\run\",
        r"\currentversion\app paths\",
        r"\microsoft\windows\start menu\",
    ];
    if KEEP.iter().any(|k| l.contains(k)) {
        return false;
    }
    NOISE.iter().any(|n| l.contains(n))
}

/// 설치 전후 비교: 새로 생긴 것만. 새 폴더·키 아래 항목은 그 폴더·키 하나로 묶는다.
pub fn diff(before: &Snapshot, after: &Snapshot) -> TraceDraft {
    let mut changes = Vec::new();

    let added_folders: Vec<&String> = after
        .folders
        .keys()
        .filter(|k| !before.folders.contains_key(*k))
        .collect();
    // 새 폴더 중 가장 바깥 것만 (BTreeMap이라 부모가 자식보다 먼저 나온다)
    let mut top_folders: Vec<&String> = Vec::new();
    for f in &added_folders {
        if !top_folders.iter().any(|t| is_under(f, t)) {
            top_folders.push(f);
        }
    }
    for f in &top_folders {
        changes.push(Change {
            kind: ChangeKind::Folder,
            location: after.folders[*f].clone(),
            value_name: None,
        });
    }
    for (k, path) in &after.files {
        if !before.files.contains_key(k) && !top_folders.iter().any(|t| is_under(k, t)) {
            changes.push(Change {
                kind: ChangeKind::File,
                location: path.clone(),
                value_name: None,
            });
        }
    }

    let mut top_keys: Vec<&String> = Vec::new();
    for k in after.keys.keys().filter(|k| !before.keys.contains_key(*k)) {
        if !top_keys.iter().any(|t| is_under(k, t)) {
            top_keys.push(k);
        }
    }
    for k in &top_keys {
        changes.push(Change {
            kind: ChangeKind::RegistryKey,
            location: after.keys[*k].clone(),
            value_name: None,
        });
    }
    for (k, (key, name)) in &after.values {
        let key_lower = key.to_lowercase();
        let in_new_key = top_keys
            .iter()
            .any(|t| key_lower == **t || is_under(&key_lower, t));
        if !before.values.contains_key(k) && !in_new_key {
            changes.push(Change {
                kind: ChangeKind::RegistryValue,
                location: key.clone(),
                value_name: Some(name.clone()),
            });
        }
    }

    changes.retain(|c| !is_noise(&c.location));
    let incomplete = before.incomplete || after.incomplete;

    TraceDraft {
        incomplete,
        started_at: before.taken_at,
        finished_at: after.taken_at,
        changes,
        new_app_ids: after
            .apps
            .iter()
            .filter(|a| !before.apps.contains(a))
            .cloned()
            .collect(),
    }
}

/// 기록 ID는 숫자·소문자 16진수·하이픈만 (경로 조작 방지).
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f' | '-'))
}

pub fn new_trace_id(now_millis: u128, pid: u32) -> String {
    format!("{now_millis:013}-{pid:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(folders: &[&str], files: &[&str], keys: &[&str], values: &[(&str, &str)]) -> Snapshot {
        let mut s = Snapshot::default();
        folders.iter().for_each(|f| s.add_folder(f));
        files.iter().for_each(|f| s.add_file(f));
        keys.iter().for_each(|k| s.add_key(k));
        values.iter().for_each(|(k, v)| s.add_value(k, v));
        s
    }

    const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    #[test]
    fn reports_only_outermost_new_items() {
        let before = snap(
            &[r"C:\Program Files", r"C:\Program Files\Old"],
            &[r"C:\Program Files\Old\a.exe"],
            &[r"HKCU\Software\Old"],
            &[(RUN, "Old")],
        );
        let after = snap(
            &[
                r"C:\Program Files",
                r"C:\Program Files\Old",
                r"C:\Program Files\Acme",
                r"C:\Program Files\Acme\bin",
            ],
            &[
                r"C:\Program Files\Old\a.exe",
                r"C:\Program Files\Old\plugin.dll",
                r"C:\Program Files\Acme\bin\acme.exe",
            ],
            &[
                r"HKCU\Software\Old",
                r"HKCU\Software\Acme",
                r"HKCU\Software\Acme\Settings",
            ],
            &[
                (RUN, "Old"),
                (RUN, "Acme"),
                (r"HKCU\Software\Acme", "Theme"),
            ],
        );
        let d = diff(&before, &after);
        let locs: Vec<String> = d
            .changes
            .iter()
            .map(|c| format!("{:?} {} {:?}", c.kind, c.location, c.value_name))
            .collect();
        assert_eq!(
            locs,
            vec![
                r#"Folder C:\Program Files\Acme None"#,
                r#"File C:\Program Files\Old\plugin.dll None"#,
                r#"RegistryKey HKCU\Software\Acme None"#,
                r#"RegistryValue HKCU\Software\Microsoft\Windows\CurrentVersion\Run Some("Acme")"#,
            ]
        );
    }

    #[test]
    fn prefix_named_folder_is_not_a_child() {
        let before = snap(&[r"C:\Apps\Foo"], &[], &[], &[]);
        let after = snap(&[r"C:\Apps\Foo", r"C:\Apps\Foo2"], &[], &[], &[]);
        assert_eq!(diff(&before, &after).changes.len(), 1);
    }

    #[test]
    fn noise_is_dropped_but_install_records_are_kept() {
        assert!(is_noise(r"C:\Users\me\AppData\Local\Temp\setup123"));
        assert!(is_noise(
            r"C:\Users\me\AppData\Local\Microsoft\OneDrive\logs"
        ));
        assert!(is_noise(r"C:\ProgramData\Microsoft\Windows Defender\Scans"));
        // 시작 메뉴는 Microsoft 아래지만 설치 흔적이라 남긴다
        assert!(!is_noise(
            r"C:\Users\me\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Acme"
        ));
        // 프로그램 폴더 안의 temp는 잡음이 아니다
        assert!(!is_noise(r"C:\Program Files\Acme\temp"));
        assert!(is_noise(
            r"C:\Users\me\AppData\Local\Google\Chrome\User Data\Default"
        ));
        assert!(is_noise(
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs"
        ));
        assert!(!is_noise(
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Acme"
        ));
        assert!(!is_noise(
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
        ));
        assert!(!is_noise(r"C:\Program Files\Acme"));
    }

    #[test]
    fn finds_new_apps() {
        let before = Snapshot {
            apps: vec!["hkcu:a".into()],
            ..Default::default()
        };
        let mut after = before.clone();
        after.apps.push("hkcu:b".into());
        assert_eq!(diff(&before, &after).new_app_ids, vec!["hkcu:b"]);
    }

    #[test]
    fn ids_are_safe() {
        assert!(is_valid_id(&new_trace_id(1_700_000_000_000, 0x1a2b)));
        assert!(!is_valid_id(r"..\x"));
    }
}
