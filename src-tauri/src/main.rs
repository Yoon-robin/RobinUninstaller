// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 관리자 권한 작업자로 실행된 경우(--elevated-*): 창 없이 작업만 하고 끝낸다.
    if let Some(code) = robinx_lib::worker::main() {
        std::process::exit(code);
    }
    robinx_lib::run()
}
