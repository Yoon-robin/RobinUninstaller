//! 제거 완료를 기다릴 때 무시할 프로세스. 순수 로직.
//!
//! 언인스톨러가 콘솔 호스트나 설문 페이지(브라우저)를 띄우면, 그 프로세스가 끝날 때까지 기다릴 이유가 없다.
//! 단, 지우려는 프로그램이 그 브라우저 자체라면 무시하지 않는다.

const ALWAYS: &[&str] = &["conhost.exe", "cmd.exe", "taskkill.exe", "werfault.exe"];
const BROWSERS: &[(&str, &[&str])] = &[
    ("chrome.exe", &["chrome"]),
    ("msedge.exe", &["edge"]),
    ("firefox.exe", &["firefox"]),
    ("opera.exe", &["opera"]),
    ("brave.exe", &["brave"]),
    ("whale.exe", &["whale"]),
    ("iexplore.exe", &["internet explorer"]),
];

/// `exe_name`(파일 이름만)을 기다리지 않아도 되는지. `target_name`은 지우려는 프로그램 이름.
pub fn is_ignored(exe_name: &str, target_name: &str) -> bool {
    let exe = exe_name.to_ascii_lowercase();
    if ALWAYS.contains(&exe.as_str()) {
        return true;
    }
    let target = target_name.to_lowercase();
    BROWSERS
        .iter()
        .any(|(name, words)| *name == exe && !words.iter().any(|w| target.contains(w)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_console_and_survey_browsers() {
        assert!(is_ignored("conhost.exe", "Some App"));
        assert!(is_ignored("CMD.EXE", "Some App"));
        assert!(is_ignored("chrome.exe", "Some App"));
        assert!(!is_ignored("unins000.exe", "Some App"));
    }

    #[test]
    fn does_not_ignore_the_browser_being_removed() {
        assert!(!is_ignored("chrome.exe", "Google Chrome"));
        assert!(!is_ignored("firefox.exe", "Mozilla Firefox (x64 ko)"));
        assert!(is_ignored("chrome.exe", "Mozilla Firefox"));
    }
}
