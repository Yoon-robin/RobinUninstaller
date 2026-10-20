//! 프로그램·게시자 이름을 비교하기 좋게 정규화한다. 순수 로직.
//!
//! "Adobe Photoshop 2026 (64-bit)" → 핵심 단어 ["adobe", "photoshop"].
//! 버전·연도·아키텍처·에디션 같은 꼬리표와 흔한 일반 단어는 근거로 쓰지 않는다.

/// 혼자서는 소유권 근거가 되지 않는 흔한 단어. (자체 목록)
const STOPWORDS: &[&str] = &[
    // 일반 소프트웨어 용어
    "software",
    "program",
    "programs",
    "application",
    "applications",
    "app",
    "apps",
    "setup",
    "install",
    "installer",
    "uninstall",
    "uninstaller",
    "update",
    "updater",
    "updates",
    "tool",
    "tools",
    "utility",
    "utilities",
    "data",
    "files",
    "file",
    "common",
    "shared",
    "system",
    "service",
    "services",
    "helper",
    "runtime",
    "runtimes",
    "redistributable",
    "redist",
    "driver",
    "drivers",
    "plugin",
    "plugins",
    "addin",
    "addon",
    "addons",
    "extension",
    "extensions",
    "library",
    "libraries",
    "framework",
    "package",
    "packages",
    "component",
    "components",
    "module",
    "modules",
    "desktop",
    "client",
    "server",
    "launcher",
    "manager",
    "suite",
    "studio",
    "platform",
    "engine",
    "core",
    "user",
    "users",
    "local",
    "global",
    "cache",
    "temp",
    "config",
    "settings",
    "support",
    "help",
    "documentation",
    "docs",
    "sample",
    "samples",
    "default",
    // 에디션·꼬리표
    "edition",
    "version",
    "free",
    "pro",
    "professional",
    "premium",
    "home",
    "lite",
    "light",
    "standard",
    "ultimate",
    "enterprise",
    "personal",
    "basic",
    "plus",
    "trial",
    "portable",
    "beta",
    "preview",
    "release",
    "stable",
    "insider",
    "build",
    "full",
    "new",
    "classic",
    // 회사 형태
    "inc",
    "corp",
    "corporation",
    "llc",
    "ltd",
    "limited",
    "gmbh",
    "company",
    "co",
    "technologies",
    "technology",
    "labs",
    "lab",
    "group",
    "foundation",
    "project",
    "team",
    "studios",
    "games",
    "entertainment",
    // 아키텍처
    "x64",
    "x86",
    "amd64",
    "arm64",
    "win32",
    "win64",
    "bit",
    "windows",
    "microsoft",
    // 연결어
    "the",
    "for",
    "and",
    "of",
    "by",
    "with",
    "to",
    "in",
    "on",
];

/// 이름을 소문자 단어 목록으로 나눈다. 숫자만으로 된 단어(버전·연도)와 2글자 이하 단어는 뺀다.
pub fn tokens(name: &str) -> Vec<String> {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .filter(|w| !w.chars().all(|c| c.is_ascii_digit()))
        .filter(|w| !is_version_like(w))
        .filter(|w| w.chars().count() >= 3 || !w.is_ascii())
        .map(str::to_string)
        .collect()
}

/// 근거로 쓸 수 있는 단어만 (stoplist 제외).
pub fn significant_tokens(name: &str) -> Vec<String> {
    tokens(name)
        .into_iter()
        .filter(|w| !STOPWORDS.contains(&w.as_str()))
        .collect()
}

/// "v2", "2026r2", "x64" 같은 꼬리표.
fn is_version_like(w: &str) -> bool {
    let digits = w.chars().filter(|c| c.is_ascii_digit()).count();
    (w.starts_with('v') && digits == w.len() - 1 && digits > 0)
        || (digits >= 2 && digits * 2 >= w.len())
}

/// 비교용 키: 단어를 공백 없이 이어 붙인다. "Fixture Notes" == "FixtureNotes" == "fixture-notes".
pub fn compact(name: &str) -> String {
    tokens(name).concat()
}

/// 폴더·키 이름이 프로그램 이름과 어떻게 맞는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameMatch {
    /// 꼬리표를 뺀 전체 이름이 같음 ("Fixture Notes" vs "fixture-notes")
    Exact,
    /// 근거 단어를 모두 포함 ("Photoshop" 폴더 vs "Adobe Photoshop 2026", 게시자 단어 제외)
    AllSignificant,
    None,
}

/// `candidate`(폴더/키 이름)가 프로그램 이름과 맞는지. `publisher`는 앞에 붙은 게시자 단어를 떼는 데 쓴다.
pub fn match_name(candidate: &str, product: &str, publisher: Option<&str>) -> NameMatch {
    let m = match_name_ignoring_versions(candidate, product, publisher);
    // "Photoshop 2021" 폴더는 "Photoshop 2026"과 이름은 같지만 다른 버전의 것일 수 있다 → 정확 일치로 보지 않는다.
    if m == NameMatch::Exact && !versions_compatible(candidate, product) {
        return NameMatch::AllSignificant;
    }
    m
}

/// 이름 속 숫자 묶음. "Adobe Photoshop 2026 (27.1)" → ["2026", "27", "1"]
fn version_numbers(name: &str) -> Vec<String> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter(|d| !d.is_empty())
        .map(str::to_string)
        .collect()
}

/// 후보 이름에 숫자가 없으면 호환. 있으면 그 숫자가 모두 프로그램 이름에도 있어야 한다.
fn versions_compatible(candidate: &str, product: &str) -> bool {
    let cand = version_numbers(candidate);
    let prod = version_numbers(product);
    cand.iter().all(|n| prod.contains(n))
}

fn match_name_ignoring_versions(
    candidate: &str,
    product: &str,
    publisher: Option<&str>,
) -> NameMatch {
    let cand = compact(candidate);
    if cand.is_empty() {
        return NameMatch::None;
    }

    let product_sig = significant_tokens(product);
    // 근거 단어가 하나도 없는 이름("Setup Tools")은 이름만으로 매칭하지 않는다.
    if product_sig.is_empty() {
        return NameMatch::None;
    }

    if cand == compact(product) {
        return NameMatch::Exact;
    }

    // 게시자 단어를 뗀 제품 이름: "Adobe Photoshop" → "photoshop"
    let publisher_words = publisher.map(tokens).unwrap_or_default();
    let product_core: Vec<&String> = product_sig
        .iter()
        .filter(|w| !publisher_words.contains(w))
        .collect();
    if !product_core.is_empty() {
        let core_compact: String = product_core.iter().map(|s| s.as_str()).collect();
        if cand == core_compact {
            return NameMatch::Exact;
        }
        // 단어 단위로만 비교한다: "Timeline"·"Outline" 폴더가 "LINE"에 걸리지 않게.
        // 붙여 쓴 이름("FixtureNotesCache")은 대소문자 경계에서 나누고, 소문자로만 붙여 쓴 이름
        // ("fixturenotes-cache")은 두 단어 이상인 제품 이름 전체가 이어서 들어 있을 때만 받는다.
        let cand_tokens = tokens(&split_camel(candidate));
        let words_match = product_core.iter().all(|w| cand_tokens.contains(w));
        let joined_match =
            product_core.len() > 1 && cand_tokens.concat().contains(core_compact.as_str());
        if (words_match || joined_match) && product_core.iter().map(|w| w.len()).sum::<usize>() >= 4
        {
            return NameMatch::AllSignificant;
        }
    }
    NameMatch::None
}

/// 대소문자 경계에서 띄어 쓴다: "FixtureNotesCache" → "Fixture Notes Cache". 대문자 묶음("SteamVR")은 그대로 둔다.
fn split_camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 8);
    let mut prev_lower = false;
    for c in name.chars() {
        if c.is_uppercase() && prev_lower {
            out.push(' ');
        }
        prev_lower = c.is_lowercase() || c.is_ascii_digit();
        out.push(c);
    }
    out
}

/// 폴더·키 이름이 게시자 이름과 같은지 ("Adobe Inc." vs "Adobe").
pub fn matches_publisher(candidate: &str, publisher: &str) -> bool {
    let cand = compact(candidate);
    let full = compact(publisher);
    let core: String = significant_tokens(publisher).concat();
    !cand.is_empty() && (cand == full || (!core.is_empty() && cand == core))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_versions_arch_and_editions() {
        assert_eq!(
            significant_tokens("Adobe Photoshop 2026 (64-bit)"),
            ["adobe", "photoshop"]
        );
        assert_eq!(significant_tokens("Blender 5.0 x64 Edition"), ["blender"]);
        assert_eq!(significant_tokens("Topaz Video AI v7"), ["topaz", "video"]);
        assert!(
            significant_tokens("Microsoft Visual C++ Redistributable")
                .iter()
                .all(|w| w == "visual")
        );
    }

    #[test]
    fn exact_and_core_name_matches() {
        assert_eq!(
            match_name("Fixture Notes", "Fixture Notes", None),
            NameMatch::Exact
        );
        assert_eq!(
            match_name("fixture-notes", "Fixture Notes 1.0", None),
            NameMatch::Exact
        );
        assert_eq!(
            match_name("Photoshop", "Adobe Photoshop 2026", Some("Adobe Inc.")),
            NameMatch::Exact
        );
        assert_eq!(
            match_name(
                "Adobe Photoshop 2026 Settings",
                "Adobe Photoshop 2026",
                Some("Adobe Inc.")
            ),
            NameMatch::AllSignificant
        );
    }

    #[test]
    fn different_versions_are_not_exact() {
        assert_eq!(
            match_name(
                "Adobe Photoshop 2021",
                "Adobe Photoshop 2026",
                Some("Adobe")
            ),
            NameMatch::AllSignificant
        );
        assert_eq!(
            match_name("Blender 4.2", "Blender", None),
            NameMatch::AllSignificant
        );
        assert_eq!(
            match_name(
                "Adobe Photoshop 2026",
                "Adobe Photoshop 2026",
                Some("Adobe")
            ),
            NameMatch::Exact
        );
        // 폴더 이름에 숫자가 없으면 버전과 상관없이 같은 프로그램으로 본다
        assert_eq!(match_name("Blender", "Blender 5.0", None), NameMatch::Exact);
    }

    #[test]
    fn generic_or_unrelated_names_do_not_match() {
        assert_eq!(match_name("Common", "Common Tools", None), NameMatch::None);
        assert_eq!(
            match_name("Adobe", "Adobe Photoshop", Some("Adobe")),
            NameMatch::None
        );
        assert_eq!(
            match_name("Illustrator", "Adobe Photoshop", Some("Adobe")),
            NameMatch::None
        );
        assert_eq!(
            match_name("Setup", "Setup Tools 2.0", None),
            NameMatch::None
        );
    }

    #[test]
    fn short_names_do_not_match_inside_other_words() {
        // "LINE"의 핵심 단어가 다른 단어 속에 들어 있을 뿐인 폴더
        for folder in ["Timeline", "Outline", "Online Services"] {
            assert_eq!(
                match_name(folder, "LINE", Some("LY Corporation")),
                NameMatch::None,
                "{folder}"
            );
        }
        // 띄어쓰기 없이 붙여 쓴 이름은 여전히 맞는다
        for folder in [
            "FixtureNotesCache",
            "fixturenotes-cache",
            "Fixture Notes Logs",
        ] {
            assert_eq!(
                match_name(folder, "Fixture Notes", None),
                NameMatch::AllSignificant,
                "{folder}"
            );
        }
        assert_eq!(
            match_name("LINE Data", "LINE", Some("LY Corporation")),
            NameMatch::AllSignificant
        );
    }

    #[test]
    fn publisher_matching() {
        assert!(matches_publisher("Adobe", "Adobe Inc."));
        assert!(matches_publisher("Robin Test Labs", "Robin Test Labs"));
        assert!(!matches_publisher("Adobe Systems", "Autodesk, Inc."));
    }
}
