//! 시작 항목을 읽어 보여 준다 (읽기 전용).
//!
//! ```text
//! cargo run -p robin-startup --example list
//! ```

#[cfg(windows)]
fn main() {
    let history = robin_startup::History::default_location().expect("LOCALAPPDATA");
    for e in robin_startup::list(&history) {
        println!(
            "{} {:<28} {:<14} {}{}",
            if e.enabled { "켜짐" } else { "꺼짐" },
            e.name,
            e.source.key(),
            e.target_path.as_deref().unwrap_or("(대상 모름)"),
            if e.target_missing {
                "  ← 파일 없음"
            } else {
                ""
            },
        );
        println!(
            "     게시자: {}  설명: {}",
            e.publisher.as_deref().unwrap_or("-"),
            e.description.as_deref().unwrap_or("-")
        );
    }
}

#[cfg(not(windows))]
fn main() {}
