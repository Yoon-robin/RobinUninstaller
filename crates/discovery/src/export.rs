//! 프로그램 목록 내보내기(CSV)와 웹 검색 주소. 순수 함수라 어느 OS에서든 시험할 수 있다.

use std::collections::HashMap;

use robin_core::{InstallScope, InstallTechnology, InstalledApplication, PackageSource, StoreApp};

const HEADER: [&str; 9] = [
    "종류",
    "이름",
    "게시자",
    "버전",
    "크기(KB)",
    "크기 출처",
    "설치일",
    "형식",
    "설치 폴더",
];

/// 데스크톱 프로그램과 Store 앱을 CSV 한 장으로 만든다.
///
/// Excel에서 한글이 깨지지 않도록 BOM을 붙이고 줄 끝은 CRLF로 한다.
/// `measured`는 크기 기록이 없어 설치 폴더를 직접 잰 값(앱 ID → KB)이다.
pub fn programs_csv(
    apps: &[InstalledApplication],
    measured: &HashMap<String, u64>,
    store: &[StoreApp],
) -> String {
    let mut out = String::from("\u{feff}");
    push_row(&mut out, HEADER.iter().map(|s| s.to_string()));

    for a in apps {
        let recorded = a.estimated_size_kb.filter(|&kb| kb > 0);
        let (size, size_source) = match (recorded, measured.get(&a.id)) {
            (Some(kb), _) => (kb.to_string(), "기록"),
            (None, Some(kb)) => (kb.to_string(), "직접 잼"),
            (None, None) => (String::new(), ""),
        };
        let technology = match a.technology {
            InstallTechnology::Exe => "EXE",
            InstallTechnology::Msi => "MSI",
        };
        let scope = match a.scope {
            InstallScope::Machine64 => "모든 사용자",
            InstallScope::Machine32 => "모든 사용자 (32비트)",
            InstallScope::User => "현재 사용자",
        };
        push_row(
            &mut out,
            [
                "데스크톱".to_string(),
                a.display_name.clone(),
                a.publisher.clone().unwrap_or_default(),
                a.version.clone().unwrap_or_default(),
                size,
                size_source.to_string(),
                a.install_date
                    .as_deref()
                    .map(normalize_date)
                    .unwrap_or_default(),
                format!("{technology} · {scope}"),
                a.install_location
                    .as_deref()
                    .map(|l| l.trim().trim_matches('"').to_string())
                    .unwrap_or_default(),
            ],
        );
    }

    for s in store {
        let source = match s.source {
            PackageSource::Store => "Store",
            PackageSource::Sideloaded => "직접 배포",
            PackageSource::Developer => "개발자 모드",
        };
        push_row(
            &mut out,
            [
                "Store 앱".to_string(),
                s.display_name.clone(),
                s.publisher.clone().unwrap_or_default(),
                s.version.clone().unwrap_or_default(),
                String::new(),
                String::new(),
                s.install_date.clone().unwrap_or_default(),
                format!("MSIX · {source}"),
                s.install_location.clone().unwrap_or_default(),
            ],
        );
    }
    out
}

fn push_row(out: &mut String, cells: impl IntoIterator<Item = String>) {
    let line: Vec<String> = cells.into_iter().map(|c| csv_cell(&c)).collect();
    out.push_str(&line.join(","));
    out.push_str("\r\n");
}

/// CSV 칸 하나. 쉼표·따옴표·줄바꿈이 있으면 따옴표로 감싼다.
/// `=`·`+`·`-`·`@`로 시작하면 Excel이 수식으로 실행할 수 있으므로 앞에 `'`를 붙여 글자로 만든다.
fn csv_cell(value: &str) -> String {
    let value = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_string()
    };
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

/// 레지스트리 설치일(`20240131`)을 `2024-01-31`로. 다른 형식은 그대로.
fn normalize_date(raw: &str) -> String {
    let raw = raw.trim();
    if raw.len() == 8 && raw.bytes().all(|b| b.is_ascii_digit()) {
        format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..])
    } else {
        raw.to_string()
    }
}

/// 프로그램 이름(과 게시자)으로 웹 검색 주소를 만든다.
pub fn search_url(name: &str, publisher: Option<&str>) -> String {
    let mut query = name.trim().to_string();
    if let Some(p) = publisher.map(str::trim).filter(|p| !p.is_empty())
        && !query.to_lowercase().contains(&p.to_lowercase())
    {
        query.push(' ');
        query.push_str(p);
    }
    format!("https://www.google.com/search?q={}", url_encode(&query))
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::UninstallerStatus;

    fn app(id: &str, name: &str, size: Option<u64>) -> InstalledApplication {
        InstalledApplication {
            id: id.into(),
            display_name: name.into(),
            publisher: Some("Robin Test Labs".into()),
            version: Some("1.0".into()),
            install_location: Some(r#""C:\Apps\Fixture""#.into()),
            install_date: Some("20260101".into()),
            estimated_size_kb: size,
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Msi,
            scope: InstallScope::User,
            uninstaller: UninstallerStatus::Available,
        }
    }

    #[test]
    fn csv_has_bom_header_and_rows() {
        let apps = [
            app("a", "Fixture Notes", Some(1024)),
            app("b", "No Size", None),
        ];
        let measured = HashMap::from([("b".to_string(), 77u64)]);
        let store = [StoreApp {
            id: "msix:x".into(),
            display_name: "계산기".into(),
            publisher: Some("Microsoft Corporation".into()),
            version: Some("11.0.0.0".into()),
            install_date: Some("2026-03-04".into()),
            install_location: None,
            family_name: "X".into(),
            source: PackageSource::Store,
            logo: Some(r"C:\secret\logo.png".into()),
        }];
        let csv = programs_csv(&apps, &measured, &store);
        let lines: Vec<&str> = csv.split("\r\n").collect();
        assert!(lines[0].starts_with("\u{feff}종류,이름,"));
        assert_eq!(
            lines[1],
            r"데스크톱,Fixture Notes,Robin Test Labs,1.0,1024,기록,2026-01-01,MSI · 현재 사용자,C:\Apps\Fixture"
        );
        assert!(lines[2].contains(",77,직접 잼,"));
        assert_eq!(
            lines[3],
            "Store 앱,계산기,Microsoft Corporation,11.0.0.0,,,2026-03-04,MSIX · Store,"
        );
        assert!(!csv.contains("secret"), "로고 경로는 내보내지 않는다");
        assert_eq!(lines.len(), 5, "마지막 줄도 CRLF로 끝난다");
    }

    #[test]
    fn cells_are_quoted_and_formulas_neutralized() {
        assert_eq!(csv_cell("a,b"), "\"a,b\"");
        assert_eq!(csv_cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_cell("=HYPERLINK(\"x\")"), "\"'=HYPERLINK(\"\"x\"\")\"");
        assert_eq!(csv_cell("-1"), "'-1");
        assert_eq!(csv_cell("@cmd"), "'@cmd");
        assert_eq!(csv_cell("평범한 이름"), "평범한 이름");
    }

    #[test]
    fn dates_are_normalized() {
        assert_eq!(normalize_date("20240131"), "2024-01-31");
        assert_eq!(normalize_date("1/31/2024"), "1/31/2024");
    }

    #[test]
    fn search_url_encodes_name_and_publisher() {
        assert_eq!(
            search_url("Fixture Notes", Some("Robin Test Labs")),
            "https://www.google.com/search?q=Fixture+Notes+Robin+Test+Labs"
        );
        // 이름에 게시자가 이미 있으면 붙이지 않는다
        assert_eq!(
            search_url("Microsoft Edge", Some("Microsoft Corporation")),
            "https://www.google.com/search?q=Microsoft+Edge+Microsoft+Corporation"
        );
        assert_eq!(
            search_url("Contoso App", Some("Contoso")),
            "https://www.google.com/search?q=Contoso+App"
        );
        assert_eq!(
            search_url("한글&=#", None),
            "https://www.google.com/search?q=%ED%95%9C%EA%B8%80%26%3D%23"
        );
    }
}
