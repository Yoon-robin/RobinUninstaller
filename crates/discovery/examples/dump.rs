//! 탐색 결과를 JSON으로 출력한다: `cargo run -p robin-discovery --example dump`
fn main() {
    let apps = robin_discovery::list_installed();
    println!("{}", serde_json::to_string_pretty(&apps).unwrap());
    let icons = apps
        .iter()
        .filter(|a| robin_discovery::app_icon(a).is_some())
        .count();
    eprintln!("icons extracted: {icons}");
}
