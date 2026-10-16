//! Windows Installer가 레지스트리에 쓰는 "압축 GUID" 변환. 순수 로직.
//!
//! `{AC76BA86-1033-FFFF-7760-BC15014EA700}` → `68AB67CA3301FFFF7706CB5110E47A00`
//! 앞 세 묶음은 글자 순서를 뒤집고, 나머지 8바이트는 바이트마다 두 글자를 맞바꾼다.

use robin_discovery::command::is_product_code;

pub fn pack(product_code: &str) -> Option<String> {
    if !is_product_code(product_code) {
        return None;
    }
    let hex: String = product_code
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    let rev = |s: &str| s.chars().rev().collect::<String>();
    let mut out = String::with_capacity(32);
    out.push_str(&rev(&hex[0..8]));
    out.push_str(&rev(&hex[8..12]));
    out.push_str(&rev(&hex[12..16]));
    for pair in hex.as_bytes()[16..32].chunks(2) {
        out.push(pair[1] as char);
        out.push(pair[0] as char);
    }
    Some(out.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_known_codes() {
        assert_eq!(
            pack("{AC76BA86-1033-FFFF-7760-BC15014EA700}").as_deref(),
            Some("68AB67CA3301FFFF7706CB5110E47A00")
        );
        assert_eq!(pack("{not-a-guid}"), None);
    }
}
