//! 정리할 수 있는 항목을 보여 준다 (읽기 전용, 아무것도 지우지 않음).
//!
//! ```text
//! cargo run -p robin-storage --example storage_scan
//! ```

#[cfg(windows)]
fn main() {
    let start = std::time::Instant::now();
    for c in robin_storage::scan() {
        println!(
            "{:>8.2} GB  {:<28} {:?}{}{}{}",
            c.bytes as f64 / 1e9,
            c.name,
            c.risk,
            if c.admin { " · 관리자" } else { "" },
            if c.default_selected {
                " · 기본 선택"
            } else {
                ""
            },
            if c.running.is_empty() {
                String::new()
            } else {
                format!(" · 실행 중: {}", c.running.join(", "))
            },
        );
    }
    println!("({} ms)", start.elapsed().as_millis());
}

#[cfg(not(windows))]
fn main() {}
