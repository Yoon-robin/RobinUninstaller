//! Store·MSIX 패키지 정보를 다듬는 순수 함수. Windows API 없이 시험할 수 있게 따로 둔다.

/// WinRT `DateTime`(1601-01-01부터 100ns 단위, UTC)을 `YYYY-MM-DD`로 바꾼다.
pub fn ticks_to_date(ticks: i64) -> Option<String> {
    const TICKS_PER_DAY: i64 = 864_000_000_000;
    // 1601-01-01부터 1970-01-01까지의 날 수
    const DAYS_TO_UNIX: i64 = 134_774;
    if ticks <= 0 {
        return None;
    }
    let days = ticks / TICKS_PER_DAY - DAYS_TO_UNIX;
    // Howard Hinnant의 civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// `file:///C:/a%20b/c.png` 같은 파일 URI를 Windows 경로로 바꾼다. 파일 URI가 아니면 `None`.
pub fn file_uri_to_path(uri: &str) -> Option<String> {
    let rest = uri
        .get(..8)
        .filter(|p| p.eq_ignore_ascii_case("file:///"))
        .map(|_| &uri[8..])?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = rest.get(i + 1..i + 3)
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    let path = String::from_utf8(out).ok()?.replace('/', "\\");
    (!path.is_empty()).then_some(path)
}

/// 게시자 DN(`CN=Contoso, O=Contoso, C=US`)에서 CN 값만 꺼낸다.
pub fn publisher_from_dn(dn: &str) -> Option<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in dn.chars() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts.iter().find_map(|p| {
        let p = p.trim();
        let value = p.get(..3).filter(|k| k.eq_ignore_ascii_case("CN="))?;
        let value = p[value.len()..].trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

/// 매니페스트에 적힌 로고 파일 이름(`StoreLogo.png`)과 실제 폴더의 파일 이름들로 쓸 파일을 고른다.
///
/// 실제 파일은 보통 `StoreLogo.scale-200.png`처럼 배율이 붙어 있다. 선명한 것을 먼저 고르고, 고대비용은 마지막에 쓴다.
pub fn pick_logo(file_name: &str, names: &[String]) -> Option<String> {
    let lower = file_name.to_lowercase();
    if let Some(exact) = names.iter().find(|n| n.to_lowercase() == lower) {
        return Some(exact.clone());
    }
    let (stem, ext) = lower.rsplit_once('.')?;
    let prefix = format!("{stem}.");
    let suffix = format!(".{ext}");
    names
        .iter()
        .filter_map(|n| {
            let l = n.to_lowercase();
            let qualifiers = l.strip_prefix(&prefix)?.strip_suffix(&suffix)?.to_string();
            let mut score = if qualifiers.contains("scale-200") {
                0
            } else if qualifiers.contains("scale-150") {
                1
            } else if qualifiers.contains("scale-100") {
                2
            } else if qualifiers.contains("targetsize-") {
                3
            } else {
                4
            };
            if qualifiers.contains("contrast-") {
                score += 10;
            }
            Some((score, n))
        })
        .min()
        .map(|(_, n)| n.clone())
}

/// 표시 이름이 비었거나 리소스 문자열을 풀지 못했으면 `None`.
pub fn usable_display_name(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty() && !name.to_ascii_lowercase().starts_with("ms-resource:"))
        .then(|| name.to_string())
}

/// 앱이 아니라 다른 앱이 쓰는 기반 패키지(런타임·데이터·스토어 구성 요소)인지.
/// 프레임워크 표시가 없어 걸러지지 않지만, 사용자가 직접 지울 대상이 아니다.
pub fn is_infrastructure_package(package_name: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "MicrosoftCorporationII.WinAppRuntime.", // Windows 앱 SDK 런타임
        "Microsoft.WindowsAppRuntime.",
        "Microsoft.Winget.Source",    // winget 패키지 목록 데이터
        "Microsoft.StorePurchaseApp", // Microsoft Store 구매 구성 요소
    ];
    PREFIXES
        .iter()
        .any(|p| package_name.len() >= p.len() && package_name[..p.len()].eq_ignore_ascii_case(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infrastructure_packages_are_recognized() {
        assert!(is_infrastructure_package(
            "MicrosoftCorporationII.WinAppRuntime.Main.1.8"
        ));
        assert!(is_infrastructure_package(
            "MicrosoftCorporationII.WinAppRuntime.Singleton"
        ));
        assert!(is_infrastructure_package("Microsoft.Winget.Source"));
        assert!(is_infrastructure_package("Microsoft.StorePurchaseApp"));
        assert!(!is_infrastructure_package("Microsoft.DesktopAppInstaller"));
        assert!(!is_infrastructure_package("Microsoft.WindowsCalculator"));
    }

    #[test]
    fn converts_winrt_ticks_to_date() {
        // 1970-01-01 = 116444736000000000
        assert_eq!(
            ticks_to_date(116_444_736_000_000_000).as_deref(),
            Some("1970-01-01")
        );
        // 2024-02-29 12:00 UTC
        let unix = 1_709_208_000_i64;
        assert_eq!(
            ticks_to_date(unix * 10_000_000 + 116_444_736_000_000_000).as_deref(),
            Some("2024-02-29")
        );
        assert_eq!(ticks_to_date(0), None);
    }

    #[test]
    fn file_uri_becomes_windows_path() {
        assert_eq!(
            file_uri_to_path("file:///C:/Program%20Files/WindowsApps/A_1.0/Assets/Logo.png")
                .as_deref(),
            Some(r"C:\Program Files\WindowsApps\A_1.0\Assets\Logo.png")
        );
        assert_eq!(
            file_uri_to_path("FILE:///D:/%ED%95%9C%EA%B8%80/a.png").as_deref(),
            Some(r"D:\한글\a.png")
        );
        assert_eq!(file_uri_to_path("https://example.com/a.png"), None);
        assert_eq!(file_uri_to_path("ms-appx:///Assets/a.png"), None);
    }

    #[test]
    fn publisher_cn_is_extracted() {
        assert_eq!(
            publisher_from_dn("CN=Microsoft Corporation, O=Microsoft Corporation, C=US").as_deref(),
            Some("Microsoft Corporation")
        );
        assert_eq!(
            publisher_from_dn(r#"O=Foo, CN="Contoso, Inc.", C=US"#).as_deref(),
            Some("Contoso, Inc.")
        );
        assert_eq!(publisher_from_dn("O=NoCn"), None);
    }

    #[test]
    fn picks_sharpest_logo_and_avoids_high_contrast() {
        let names: Vec<String> = [
            "StoreLogo.scale-100.png",
            "StoreLogo.scale-200_contrast-black.png",
            "StoreLogo.scale-150.png",
            "StoreLogoWide.scale-200.png",
            "Square44x44Logo.scale-200.png",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            pick_logo("StoreLogo.png", &names).as_deref(),
            Some("StoreLogo.scale-150.png")
        );
        // 정확히 같은 파일이 있으면 그것
        let exact = vec![
            "storelogo.png".to_string(),
            "StoreLogo.scale-200.png".into(),
        ];
        assert_eq!(
            pick_logo("StoreLogo.png", &exact).as_deref(),
            Some("storelogo.png")
        );
        // 고대비용만 있으면 그거라도
        let contrast = vec!["Logo.contrast-white_scale-100.png".to_string()];
        assert_eq!(
            pick_logo("Logo.png", &contrast).as_deref(),
            Some("Logo.contrast-white_scale-100.png")
        );
        assert_eq!(pick_logo("Logo.png", &[]), None);
    }

    #[test]
    fn unresolved_resource_names_are_rejected() {
        assert_eq!(usable_display_name("  "), None);
        assert_eq!(
            usable_display_name("ms-resource:AppName"),
            None,
            "리소스 문자열 그대로는 이름으로 쓰지 않는다"
        );
        assert_eq!(usable_display_name("계산기").as_deref(), Some("계산기"));
    }
}
