//! UninstallString / DisplayIcon 문자열 해석. 순수 함수만 둔다 (시스템 접근 없음).

/// 실행할 수 있는 파일 확장자. 따옴표 없는 명령은 이 확장자로 끝나는 가장 짧은 앞부분을 경로로 본다
/// (`CreateProcess`가 공백에서 끊어 가며 찾는 순서와 같다).
const EXECUTABLE_EXTENSIONS: [&str; 6] = [".exe", ".com", ".bat", ".cmd", ".vbs", ".ps1"];

fn is_msiexec(path: &str) -> bool {
    let last = path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    last == "msiexec" || last == "msiexec.exe"
}

/// 따옴표 없는 명령에서 실행 파일 부분의 길이 (바이트)
fn unquoted_path_len(command: &str) -> usize {
    let first_word = command.find(char::is_whitespace).unwrap_or(command.len());
    // 첫 단어에 경로 구분자가 없으면 경로 없는 명령이다 (wscript C:\x.vbs → wscript)
    if !command[..first_word].contains(['\\', '/']) {
        return first_word;
    }
    let lower = command.to_ascii_lowercase();
    let boundaries = command
        .char_indices()
        .filter(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i)
        .chain([command.len()]);
    for end in boundaries {
        if EXECUTABLE_EXTENSIONS
            .iter()
            .any(|ext| lower[..end].ends_with(ext))
        {
            return end;
        }
    }
    // 확장자가 없으면 첫 단어
    first_word
}

/// 제거·시작 명령에서 실행 파일 경로를 뽑는다.
///
/// - `"C:\a b\unins000.exe" /SILENT` → `C:\a b\unins000.exe`
/// - `C:\a b\uninst.exe /S` → `C:\a b\uninst.exe` (따옴표 없이 공백이 있는 경로도 확장자까지 자름)
/// - `C:\Tools\run.cmd --tray` → `C:\Tools\run.cmd`
/// - `wscript C:\x.vbs` → `wscript.exe` (경로 없는 명령은 `.exe`를 붙인다)
/// - `MsiExec.exe /X{GUID}`, `"C:\Windows\System32\msiexec.exe" /x ...` → `None` (MSI는 Windows Installer가 처리)
pub fn executable_path(command: &str) -> Option<String> {
    let command = command.trim();
    if command.is_empty() {
        return None;
    }

    let path = if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        rest[..end].to_string()
    } else {
        let mut path = command[..unquoted_path_len(command)].to_string();
        // 확장자가 없으면 Windows처럼 .exe를 붙여 본다 (rundll32 → rundll32.exe)
        let last = path.rsplit(['\\', '/']).next().unwrap_or_default();
        if !last.contains('.') {
            path.push_str(".exe");
        }
        path
    };
    (!path.is_empty() && !is_msiexec(&path)).then_some(path)
}

/// 제거 명령을 실행 파일과 인자 문자열로 나눈다. MSI 명령이면 `None`.
///
/// 셸(cmd)을 거치지 않고 직접 실행하기 위해 쓴다. 인자는 원본 그대로 넘긴다.
pub fn split_command(command: &str) -> Option<(String, String)> {
    let command = command.trim();
    let exe = executable_path(command)?;
    let consumed = if let Some(rest) = command.strip_prefix('"') {
        // 따옴표 + 경로 + 따옴표
        rest.find('"')? + 2
    } else {
        unquoted_path_len(command)
    };
    Some((exe, command[consumed..].trim().to_string()))
}

/// `DisplayIcon` 값(`path,index` 형식)을 경로와 아이콘 인덱스로 나눈다.
pub fn icon_location(display_icon: &str) -> Option<(String, i32)> {
    let raw = display_icon.trim().trim_matches('"');
    if raw.is_empty() {
        return None;
    }

    if let Some((path, index)) = raw.rsplit_once(',')
        && let Ok(index) = index.trim().parse::<i32>()
    {
        return Some((path.trim().trim_matches('"').to_string(), index));
    }
    Some((raw.to_string(), 0))
}

/// `%VAR%` 형태의 환경 변수를 펼친다. 없는 변수는 그대로 둔다.
pub fn expand_env(s: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) if end > 0 => {
                let name = &after[..end];
                match lookup(name) {
                    Some(value) => out.push_str(&value),
                    None => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            _ => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// 명령 문자열 안에 들어 있는 첫 번째 MSI 제품 코드를 찾는다 (`msiexec /x {GUID}`).
pub fn find_product_code(command: &str) -> Option<&str> {
    command
        .match_indices('{')
        .filter_map(|(i, _)| command.get(i..i + 38))
        .find(|candidate| is_product_code(candidate))
}

/// MSI 제품 코드 형식(`{8-4-4-4-12}`)인지.
pub fn is_product_code(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 38
        && b[0] == b'{'
        && b[37] == b'}'
        && b[1..37].iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_path_with_args() {
        assert_eq!(
            executable_path(r#""C:\Program Files\App\unins000.exe" /SILENT"#).as_deref(),
            Some(r"C:\Program Files\App\unins000.exe")
        );
    }

    #[test]
    fn unquoted_path_with_spaces() {
        assert_eq!(
            executable_path(r"C:\Program Files\App\Uninstall.EXE /S").as_deref(),
            Some(r"C:\Program Files\App\Uninstall.EXE")
        );
    }

    #[test]
    fn msiexec_is_not_a_file_check() {
        assert_eq!(
            executable_path("MsiExec.exe /X{11111111-2222-3333-4444-555555555555}"),
            None
        );
        assert_eq!(
            executable_path(r"C:\Windows\System32\msiexec.exe /x {X}"),
            None
        );
    }

    #[test]
    fn splits_exe_and_args() {
        assert_eq!(
            split_command(r#""C:\Program Files\App\unins000.exe" /SILENT /X"#),
            Some((
                r"C:\Program Files\App\unins000.exe".into(),
                "/SILENT /X".into()
            ))
        );
        assert_eq!(
            split_command(r"C:\Program Files\App\Uninstall.exe --uninstall"),
            Some((
                r"C:\Program Files\App\Uninstall.exe".into(),
                "--uninstall".into()
            ))
        );
        assert_eq!(
            split_command(r#""C:\a\u.exe""#),
            Some((r"C:\a\u.exe".into(), String::new()))
        );
        assert_eq!(
            split_command("MsiExec.exe /X{11111111-2222-3333-4444-555555555555}"),
            None
        );
    }

    #[test]
    fn icon_with_index() {
        assert_eq!(
            icon_location(r#""C:\App\app.exe",-101"#),
            Some((r"C:\App\app.exe".to_string(), -101))
        );
        assert_eq!(
            icon_location(r"C:\App\app.ico"),
            Some((r"C:\App\app.ico".to_string(), 0))
        );
    }

    #[test]
    fn expands_known_env_vars_only() {
        let lookup = |name: &str| {
            name.eq_ignore_ascii_case("windir")
                .then(|| r"C:\WINDOWS".to_string())
        };
        assert_eq!(
            expand_env(r"%windir%\system32\x.exe %NOPE% 100%", lookup),
            r"C:\WINDOWS\system32\x.exe %NOPE% 100%"
        );
    }

    #[test]
    fn finds_product_code_in_command() {
        assert_eq!(
            find_product_code("msiexec.exe /x {6B725F8C-1FCE-49E4-A5A6-AE481A44E45C} AI=1"),
            Some("{6B725F8C-1FCE-49E4-A5A6-AE481A44E45C}")
        );
        assert_eq!(find_product_code("uninstall.exe /S"), None);
    }

    #[test]
    fn product_code_format() {
        assert!(is_product_code("{90160000-008C-0000-1000-0000000FF1CE}"));
        assert!(!is_product_code("{not-a-guid}"));
        assert!(!is_product_code("Steam App 730"));
    }

    #[test]
    fn commands_without_exe() {
        assert_eq!(
            executable_path(r"C:\Tools\run.cmd --tray").as_deref(),
            Some(r"C:\Tools\run.cmd")
        );
        assert_eq!(
            executable_path(r"C:\Program Files\X\uninst.bat /S").as_deref(),
            Some(r"C:\Program Files\X\uninst.bat")
        );
        assert_eq!(
            executable_path(r"wscript C:\Users\me\x.vbs").as_deref(),
            Some("wscript.exe")
        );
        assert_eq!(
            executable_path(r"powershell -WindowStyle Hidden -File C:\Tools\a.ps1").as_deref(),
            Some("powershell.exe")
        );
        assert_eq!(
            executable_path(r"rundll32.exe shell32.dll,Control_RunDLL").as_deref(),
            Some("rundll32.exe")
        );
        assert_eq!(
            split_command(r"C:\Program Files\X\uninst.bat /S"),
            Some((r"C:\Program Files\X\uninst.bat".into(), "/S".into()))
        );
        assert_eq!(
            split_command("rundll32 shell32.dll,X"),
            Some(("rundll32.exe".into(), "shell32.dll,X".into()))
        );
    }

    #[test]
    fn quoted_msiexec_is_msi() {
        assert_eq!(
            executable_path(
                r#""C:\Windows\System32\MsiExec.exe" /X{11111111-2222-3333-4444-555555555555}"#
            ),
            None
        );
    }
}
