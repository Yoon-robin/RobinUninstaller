fn main() {
    // exe에 들어가는 아이콘(icon.ico)이 바뀌면 빌드 스크립트를 다시 돌려 새 아이콘을 넣는다.
    // 이 줄이 없으면 아이콘만 바꿨을 때 예전 아이콘이 그대로 남는다.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build()
}
