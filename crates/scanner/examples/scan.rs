//! 설치된 프로그램 하나를 골라 잔여물 후보를 미리 본다 (읽기 전용):
//! `cargo run -p robin-scanner --example scan -- "Blender"`
fn main() {
    let query = std::env::args()
        .nth(1)
        .expect("프로그램 이름 일부를 넘겨 주세요")
        .to_lowercase();
    let installed = robin_discovery::list_installed();
    let Some(app) = installed
        .iter()
        .find(|a| a.display_name.to_lowercase().contains(&query))
    else {
        eprintln!("찾지 못했어요");
        std::process::exit(1);
    };
    let report = robin_scanner::scan(app, &installed);
    println!(
        "{} — {}개 (보호로 제외 {}개, {}ms)",
        report.app_name,
        report.items.len(),
        report.protected_skipped,
        report.duration_ms
    );
    for item in &report.items {
        let kinds: Vec<String> = item.evidence.iter().map(|e| format!("{e:?}")).collect();
        println!(
            "  [{:?} {:>3}] {:?} {}{} {}",
            item.confidence,
            item.score,
            item.kind,
            item.location,
            item.value_name
                .as_deref()
                .map(|v| format!(" → {v}"))
                .unwrap_or_default(),
            kinds.join(", ")
        );
    }
}
