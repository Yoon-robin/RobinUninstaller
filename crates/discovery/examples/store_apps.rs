//! Store·MSIX 앱 목록을 출력한다 (읽기 전용): `cargo run -p robin-discovery --example store_apps`
fn main() {
    let apps = robin_discovery::list_store_apps();
    for app in &apps {
        println!(
            "{:<40} {:<28} {:<16} {:?} {}",
            app.display_name,
            app.publisher.as_deref().unwrap_or("-"),
            app.version.as_deref().unwrap_or("-"),
            app.source,
            app.install_date.as_deref().unwrap_or("-"),
        );
    }
    let icons = apps
        .iter()
        .filter(|a| robin_discovery::store_app_icon(a).is_some())
        .count();
    eprintln!("{} apps, icons: {icons}", apps.len());
}
