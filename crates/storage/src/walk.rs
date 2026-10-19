//! 경로 해석과 파일 탐색.
//!
//! - 정션·심볼릭 링크(리파스 포인트)는 따라가지도, 지우지도 않는다.
//! - OneDrive 같은 클라우드 파일(오프라인·필요할 때 받기)은 열거나 지우지 않는다. 열면 다운로드가 시작된다.
//! - 지우는 건 파일뿐이고, 비게 된 하위 폴더만 정리한다. 규칙이 가리키는 폴더 자체는 남긴다.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::rules::{Base, Rule};

const FILE_ATTRIBUTE_OFFLINE: u32 = 0x1000;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x4_0000;
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;

/// 따라가거나 건드리면 안 되는 항목인지 (리파스 포인트, 클라우드 파일)
pub fn is_untouchable(attributes: u32) -> bool {
    attributes
        & (FILE_ATTRIBUTE_REPARSE_POINT
            | FILE_ATTRIBUTE_OFFLINE
            | FILE_ATTRIBUTE_RECALL_ON_OPEN
            | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
        != 0
}

#[cfg(windows)]
fn attributes(meta: &std::fs::Metadata) -> u32 {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes()
}

#[cfg(not(windows))]
fn attributes(_: &std::fs::Metadata) -> u32 {
    0
}

/// TEMP가 가리키는 폴더를 정리 기준으로 써도 되는지.
///
/// TEMP는 사용자나 다른 프로그램이 바꿀 수 있는 값이다. 문서·바탕 화면처럼 엉뚱한 곳을 가리키면
/// 그 아래의 오래된 파일을 "임시 파일"로 보고 지우게 되므로, 금지 목록이 아니라 **허용 목록**으로 판단한다.
/// Windows가 쓰는 임시 폴더(`%LOCALAPPDATA%\Temp`와 그 하위(원격 세션의 `Temp\2` 등), `%SystemRoot%\Temp`)만 허용한다.
fn is_safe_temp_root(root: &Path, env: &impl Fn(&str) -> Option<String>) -> bool {
    let norm = |p: &str| p.trim().trim_end_matches('\\').to_lowercase();
    let r = norm(&root.to_string_lossy());
    // `..`·`.` 조각이나 드라이브 상대 경로는 비교를 속일 수 있어 받지 않는다
    if r.split('\\').any(|seg| seg == ".." || seg == ".") || r.len() <= 3 {
        return false;
    }
    [("LOCALAPPDATA", "temp"), ("SystemRoot", "temp")]
        .iter()
        .filter_map(|(var, sub)| env(var).map(|base| format!("{}\\{sub}", norm(&base))))
        .filter(|allowed| allowed.len() > 8)
        .any(|allowed| r == allowed || r.starts_with(&format!("{allowed}\\")))
}

/// 규칙의 경로 하나를 실제 경로들로 바꾼다. `*` 조각은 그 자리의 하위 폴더 전부.
/// 존재하는 경로만, 그리고 기준 폴더 안에 있는 경로만 돌려준다.
pub fn resolve(base: Base, rel: &str, env: &impl Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let Some(root) = env(base.env_var()).filter(|r| !r.trim().is_empty()) else {
        return Vec::new();
    };
    let root = PathBuf::from(format!("{}\\", root.trim_end_matches('\\')));
    // TEMP는 사용자가 바꿀 수 있는 값이라, Windows 임시 폴더가 아니면 이 규칙을 건너뛴다
    if base == Base::Temp && !is_safe_temp_root(&root, env) {
        return Vec::new();
    }
    let parts: Vec<&str> = rel.split('\\').filter(|p| !p.is_empty()).collect();
    // `..`은 starts_with 검사를 통과해 버리므로 아예 받지 않는다
    if parts
        .iter()
        .any(|p| *p == "." || *p == ".." || p.contains(':'))
    {
        return Vec::new();
    }
    let mut current = vec![root.clone()];
    for part in parts {
        let mut next = Vec::new();
        for dir in current {
            if part == "*" {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        // 하위 폴더만, 리파스 포인트는 빼고
                        if e.metadata()
                            .is_ok_and(|m| m.is_dir() && !is_untouchable(attributes(&m)))
                        {
                            next.push(e.path());
                        }
                    }
                }
            } else {
                let p = dir.join(part);
                // 중간 조각이 정션이면 그 너머로 가지 않는다 (규칙 밖의 다른 드라이브·폴더일 수 있다)
                if std::fs::symlink_metadata(&p).is_ok_and(|m| is_untouchable(attributes(&m))) {
                    continue;
                }
                next.push(p);
            }
        }
        current = next;
    }
    current
        .into_iter()
        .filter(|p| {
            // 존재하고, 그 자체가 리파스 포인트가 아니며, 기준 폴더 안에 있어야 한다.
            // 일반 권한으로는 열어 볼 수도 없는 곳(배달 최적화 캐시 등)은 "권한 없음"으로 남겨,
            // 크기는 모르지만 관리자 권한 항목으로 보여 준다. 관리자 권한 작업자는 같은 함수로 다시 확인한다.
            let ok = match std::fs::symlink_metadata(p) {
                Ok(m) => !is_untouchable(attributes(&m)),
                Err(e) => e.kind() == std::io::ErrorKind::PermissionDenied,
            };
            ok && p.starts_with(&root)
        })
        .collect()
}

/// 규칙이 가리키는 실제 경로 전부
pub fn rule_paths(rule: &Rule, env: &impl Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = rule
        .paths
        .iter()
        .flat_map(|(base, rel)| resolve(*base, rel, env))
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Tally {
    pub bytes: u64,
    pub files: u64,
    /// 권한이 없어 읽지 못한 폴더 수
    pub inaccessible: u64,
}

impl Tally {
    pub fn add(&mut self, other: Tally) {
        self.bytes += other.bytes;
        self.files += other.files;
        self.inaccessible += other.inaccessible;
    }
}

/// 만든 시각과 마지막 수정 시각이 모두 `min_age`보다 오래됐는지 (시간을 알 수 없으면 오래된 것으로 보지 않는다).
/// 압축을 푼 파일은 수정 시각이 원본 그대로라 오래돼 보이므로 만든 시각도 본다.
fn old_enough(meta: &std::fs::Metadata, min_age: Duration, now: SystemTime) -> bool {
    if min_age.is_zero() {
        return true;
    }
    let age = |t: std::io::Result<SystemTime>| t.ok().and_then(|t| now.duration_since(t).ok());
    let newest = match (age(meta.modified()), age(meta.created())) {
        (Some(m), Some(c)) => Some(m.min(c)),
        (m, c) => m.or(c),
    };
    newest.is_some_and(|a| a >= min_age)
}

/// `path`(폴더 또는 파일) 아래에서 조건에 맞는 파일마다 `on_file`을 부른다.
/// 폴더는 깊은 것부터 `on_dir_done`을 부른다 (빈 폴더 정리용). `path` 자신은 부르지 않는다.
pub fn visit(
    path: &Path,
    min_age: Duration,
    on_file: &mut dyn FnMut(&Path, u64),
    on_dir_done: &mut dyn FnMut(&Path),
) -> u64 {
    let now = SystemTime::now();
    let mut inaccessible = 0;
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        // 열어 볼 권한이 없으면 "읽지 못한 폴더"로 센다 (관리자 권한 항목의 크기를 모른다고 알리기 위해)
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return 1,
        Err(_) => return 0,
    };
    if is_untouchable(attributes(&meta)) {
        return 0;
    }
    if meta.is_file() {
        if old_enough(&meta, min_age, now) {
            on_file(path, meta.len());
        }
        return 0;
    }

    // (폴더, 하위 탐색을 마쳤는지)
    let mut stack: Vec<(PathBuf, bool)> = vec![(path.to_path_buf(), false)];
    while let Some((dir, done)) = stack.pop() {
        if done {
            // 방금 만든 빈 폴더(실행 중인 프로그램이 곧 쓸 수 있음)는 남긴다
            let dir_old =
                std::fs::symlink_metadata(&dir).is_ok_and(|m| old_enough(&m, min_age, now));
            if dir != path && dir_old {
                on_dir_done(&dir);
            }
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            inaccessible += 1;
            continue;
        };
        stack.push((dir, true));
        for e in entries.flatten() {
            // Windows에서 DirEntry::metadata는 찾기 결과를 그대로 써서 추가 호출이 없다
            let Ok(m) = e.metadata() else { continue };
            if is_untouchable(attributes(&m)) {
                continue;
            }
            if m.is_dir() {
                stack.push((e.path(), false));
            } else if old_enough(&m, min_age, now) {
                on_file(&e.path(), m.len());
            }
        }
    }
    inaccessible
}

/// 크기만 센다 (읽기 전용)
pub fn measure(path: &Path, min_age: Duration) -> Tally {
    measure_with(path, min_age, None)
}

/// [`measure`]와 같고, 센 파일을 `progress`에도 올린다
pub fn measure_with(
    path: &Path,
    min_age: Duration,
    progress: Option<&robin_core::progress::Progress>,
) -> Tally {
    let mut tally = Tally::default();
    tally.inaccessible = visit(
        path,
        min_age,
        &mut |_, len| {
            tally.bytes += len;
            tally.files += 1;
            if let Some(p) = progress {
                p.add_file(len);
            }
        },
        &mut |_| {},
    );
    tally
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_and_links_are_untouchable() {
        assert!(is_untouchable(FILE_ATTRIBUTE_REPARSE_POINT));
        assert!(is_untouchable(FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS | 0x20));
        assert!(is_untouchable(FILE_ATTRIBUTE_OFFLINE));
        assert!(!is_untouchable(0x20)); // ARCHIVE
    }

    #[test]
    // Windows 경로(드라이브 문자, 역슬래시) 동작을 확인하는 테스트라 Windows·Wine에서만 돌린다
    #[cfg_attr(not(windows), ignore)]
    fn resolves_wildcards_inside_base_only() {
        let root =
            std::env::temp_dir().join(format!("robin-storage-resolve-{}", std::process::id()));
        for profile in ["Default", "Profile 1"] {
            std::fs::create_dir_all(root.join("Browser").join(profile).join("Cache")).unwrap();
        }
        std::fs::create_dir_all(root.join("Browser").join("Empty")).unwrap();
        let env = |k: &str| (k == "LOCALAPPDATA").then(|| root.to_string_lossy().into_owned());

        let found = resolve(Base::LocalAppData, r"Browser\*\Cache", &env);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(resolve(Base::LocalAppData, r"..\..\Windows", &env).is_empty());
        assert!(
            resolve(Base::AppData, "Browser", &env).is_empty(),
            "기준이 없으면 빈 목록"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn measures_only_old_files() {
        let root =
            std::env::temp_dir().join(format!("robin-storage-measure-{}", std::process::id()));
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("a.bin"), [0u8; 10]).unwrap();
        std::fs::write(root.join("sub").join("b.bin"), [0u8; 5]).unwrap();

        let all = measure(&root, Duration::ZERO);
        assert_eq!((all.bytes, all.files), (15, 2));
        // 방금 만든 파일은 하루가 안 됐으므로 0
        let old = measure(&root, Duration::from_secs(24 * 3600));
        assert_eq!(old.files, 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn temp_root_must_be_a_windows_temp_folder() {
        let env = |v: &str| match v {
            "USERPROFILE" => Some(r"C:\Users\me".to_string()),
            "LOCALAPPDATA" => Some(r"C:\Users\me\AppData\Local".to_string()),
            "SystemRoot" => Some(r"C:\Windows".to_string()),
            _ => None,
        };
        // 허용: Windows가 쓰는 임시 폴더
        assert!(is_safe_temp_root(
            Path::new(r"C:\Users\me\AppData\Local\Temp\"),
            &env
        ));
        assert!(is_safe_temp_root(
            Path::new(r"c:\users\ME\appdata\local\temp\2"),
            &env
        ));
        assert!(is_safe_temp_root(Path::new(r"C:\Windows\Temp"), &env));
        // 거부: 사용자 파일이 있을 수 있는 곳, 루트, 비슷한 이름, 경로 조작
        assert!(!is_safe_temp_root(Path::new(r"C:\"), &env));
        assert!(!is_safe_temp_root(Path::new(r"C:\Users\me\"), &env));
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\Documents"),
            &env
        ));
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\Desktop\Temp"),
            &env
        ));
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\OneDrive\Temp"),
            &env
        ));
        assert!(!is_safe_temp_root(Path::new(r"C:\Windows"), &env));
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\AppData\Local\Temporary"),
            &env
        ));
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\AppData\Local\Temp\..\..\..\Documents"),
            &env
        ));
        // 기준 변수가 없으면 아무것도 허용하지 않는다
        assert!(!is_safe_temp_root(
            Path::new(r"C:\Users\me\AppData\Local\Temp"),
            &|_: &str| None
        ));
    }

    #[test]
    fn temp_rule_resolves_nothing_when_temp_points_to_documents() {
        let docs = std::env::temp_dir().join(format!("robin-docs-{}", std::process::id()));
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("report.docx"), b"x").unwrap();
        let docs_s = docs.to_string_lossy().into_owned();
        let env = move |v: &str| match v {
            "TEMP" => Some(docs_s.clone()),
            "LOCALAPPDATA" => Some(r"C:\Users\me\AppData\Local".to_string()),
            "SystemRoot" => Some(r"C:\Windows".to_string()),
            _ => None,
        };
        assert!(resolve(Base::Temp, "", &env).is_empty());
        assert!(resolve(Base::Temp, "*", &env).is_empty());
        let _ = std::fs::remove_dir_all(docs);
    }
}
