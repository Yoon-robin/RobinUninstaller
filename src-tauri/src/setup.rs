//! 자체 설치·제거 모드 (설계: `docs/INSTALLER.md`, 엔진: `crates/setup`).
//!
//! 배포하는 setup.exe는 robinx.exe와 같은 파일이다. 앱을 띄우기 전에 [`main`]이 실행 방법을 보고 모드를 정한다.
//! - 자동 업데이트(`/P`, `/S`, `/UPDATE`)와 조용한 제거(`--uninstall /S`)는 창 없이 여기서 끝낸다.
//! - 설치·제거 화면은 평소 앱과 같은 창에서 React가 [`app_mode`]를 보고 그린다.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use robin_setup::{Env, InstallOptions, UninstallOptions};
use serde::Serialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// 제거 프로그램이 임시 복사본에서 실행 중임을 알리는 인수
const FROM_TEMP: &str = "--from-temp";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    App,
    Setup,
    Uninstall,
}

static MODE: OnceLock<Mode> = OnceLock::new();
/// 설치·제거 중. 이때 창을 닫으면 등록 정보만 남거나 데이터 폴더가 반쯤 지워진 채로 끝나므로 닫지 않는다.
static WORKING: AtomicBool = AtomicBool::new(false);

/// 설치·제거하는 동안 켜 두고, 끝나면(오류·패닉 포함) 끈다
struct Working;

impl Working {
    fn start() -> Result<Working, String> {
        if WORKING.swap(true, Ordering::SeqCst) {
            return Err("이미 진행 중이에요".into());
        }
        Ok(Working)
    }
}

impl Drop for Working {
    fn drop(&mut self) {
        WORKING.store(false, Ordering::SeqCst);
    }
}

/// 설치·제거 화면 창을 닫으려 할 때 (닫기 단추, Alt+F4). 진행 중이면 막고, 제거를 마쳤으면 임시 복사본을 지우게 해 둔다.
/// 닫아도 되면 true.
pub fn on_close_requested() -> bool {
    if WORKING.load(Ordering::SeqCst) {
        return false;
    }
    if mode() == Mode::Uninstall {
        schedule_self_delete();
    }
    true
}

pub fn mode() -> Mode {
    *MODE.get().unwrap_or(&Mode::App)
}

struct Args {
    all: Vec<String>,
}

impl Args {
    fn from_env() -> Args {
        Args {
            all: std::env::args().skip(1).collect(),
        }
    }
    fn has(&self, flag: &str) -> bool {
        self.all.iter().any(|a| a.eq_ignore_ascii_case(flag))
    }
    /// `/ARGS` 뒤의 인수 (앱을 다시 띄울 때 넘긴다)
    fn relaunch_args(&self) -> Vec<String> {
        match self
            .all
            .iter()
            .position(|a| a.eq_ignore_ascii_case("/ARGS"))
        {
            Some(i) => self.all[i + 1..].to_vec(),
            None => Vec::new(),
        }
    }
}

fn current_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

/// 실행 파일 이름에 setup·installer가 들어 있으면 설치 화면
fn named_like_installer(exe: &Path) -> bool {
    let name = exe
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    name.contains("setup") || name.contains("installer")
}

/// 설치 모드가 남기는 기록. 자동 업데이트는 화면이 없으므로 문제가 생기면 여기서 본다.
fn log(line: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("RobinX-setup.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "[{}] {line}", unix_now());
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 창 없이 다른 프로그램을 띄운다 (기다리지 않음)
fn spawn_detached(exe: &Path, args: &[String]) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    std::process::Command::new(exe)
        .args(args)
        // 설치 화면이 쓴 임시 WebView 위치를 앱에 물려주지 않는다 (물려주면 앱 설정이 초기화된 것처럼 보이고,
        // 그 뒤 업데이트로 다시 띄울 때마다 계속 이어진다)
        .env_remove("WEBVIEW2_USER_DATA_FOLDER")
        .creation_flags(DETACHED_PROCESS)
        .spawn()
        .map(|_| ())
}

/// 실행 중인 설치된 RobinX들 (새 위치와 예전 위치)
fn installed_exes(env: &Env) -> Vec<PathBuf> {
    let mut exes = vec![env.install_dir().join(robin_setup::EXE)];
    if let Some(e) = robin_setup::existing(env) {
        exes.push(e.dir.join(robin_setup::EXE));
    }
    exes
}

/// 제거가 끝난 뒤 임시 복사본을 지우게 해 둔다. 창(WebView2)이 다 닫히는 데 몇 초 걸리므로,
/// cmd가 1초 간격으로 최대 30번 지워 보고 지워지면 끝낸다.
fn schedule_self_delete() {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let Some(exe) = current_exe() else { return };
    // 임시 폴더의 복사본만 지운다 (설치된 실행 파일을 지우는 실수를 막는다)
    let in_temp = std::fs::canonicalize(&exe)
        .ok()
        .zip(std::fs::canonicalize(std::env::temp_dir()).ok())
        .is_some_and(|(e, t)| e.starts_with(t));
    let name_ok = exe
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("RobinX-uninstall-"));
    if !in_temp || !name_ok {
        log(&format!(
            "임시 복사본이 아니라 지우지 않아요: {}",
            exe.display()
        ));
        return;
    }
    let path = exe.display().to_string();
    // cmd는 따옴표 안의 %도 환경 변수로 펼친다. 이런 경로(드문 사용자 이름 등)는 지우지 않고 남긴다 (Temp라 나중에 정리된다)
    if path.contains('%') {
        log(&format!("경로에 %가 있어 임시 복사본을 남겨요: {path}"));
        return;
    }
    let cmd = format!(
        "for /l %i in (1,1,30) do @(ping -n 2 127.0.0.1 >nul & del /f /q \"{path}\" 2>nul & if not exist \"{path}\" exit /b 0)"
    );
    // 인수를 그대로 넘긴다: 보통 방식(args)은 안쪽 따옴표를 \"로 바꾸는데 cmd는 그걸 이해하지 못한다
    if let Err(e) = std::process::Command::new("cmd")
        .raw_arg(format!("/d /c {cmd}"))
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        log(&format!("임시 복사본 정리를 맡기지 못했어요: {e}"));
    }
}

/// WebView2 런타임이 있는지 (설치·제거 화면에 필요)
fn webview2_installed() -> bool {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    const GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    let candidates = [
        (
            HKEY_LOCAL_MACHINE,
            format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{GUID}"),
        ),
        (
            HKEY_LOCAL_MACHINE,
            format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{GUID}"),
        ),
        (
            HKEY_CURRENT_USER,
            format!(r"Software\Microsoft\EdgeUpdate\Clients\{GUID}"),
        ),
    ];
    candidates.iter().any(|(root, path)| {
        RegKey::predef(*root)
            .open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("pv"))
            .is_ok_and(|v| !v.is_empty() && v != "0.0.0.0")
    })
}

fn webview2_missing_message() {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
    use windows::core::HSTRING;
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(
                "RobinX 화면을 그리려면 Microsoft Edge WebView2 런타임이 필요해요.\n\n\
                 확인을 누르면 Microsoft 다운로드 페이지가 열려요. 설치한 뒤 이 파일을 다시 실행해 주세요.",
            ),
            &HSTRING::from("RobinX"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
    let _ = std::process::Command::new("explorer")
        .arg("https://developer.microsoft.com/microsoft-edge/webview2/")
        .spawn();
}

/// 설치·제거 모드면 처리한다. 창 없는 작업이면 끝낸 뒤 종료 코드, 화면이 필요하면 모드만 정하고 None.
pub fn main() -> Option<i32> {
    let args = Args::from_env();
    let exe = current_exe()?;

    if args.has("--uninstall") {
        // 설치된 실행 파일은 실행 중에 지울 수 없으므로 임시 복사본으로 넘긴다
        if !args.has(FROM_TEMP) {
            let temp =
                std::env::temp_dir().join(format!("RobinX-uninstall-{}.exe", std::process::id()));
            if std::fs::copy(&exe, &temp).is_ok() {
                let mut forwarded = args.all.clone();
                forwarded.push(FROM_TEMP.into());
                if spawn_detached(&temp, &forwarded).is_ok() {
                    return Some(0);
                }
            }
            log("임시 복사본을 만들지 못해 설치 위치에서 그대로 제거해요");
        }
        if args.has("/S") {
            let code = quiet_uninstall();
            schedule_self_delete();
            return Some(code);
        }
        return gui(Mode::Uninstall);
    }

    if args.has("/P") || args.has("/S") || args.has("/UPDATE") {
        return Some(quiet_install(&args, &exe));
    }

    if named_like_installer(&exe) {
        return gui(Mode::Setup);
    }
    None
}

fn gui(mode: Mode) -> Option<i32> {
    if !webview2_installed() {
        webview2_missing_message();
        return Some(1);
    }
    // 설치 화면은 앱의 WebView 데이터를 건드리지 않도록 임시 위치를 쓴다 (제거 때 그 폴더를 지울 수도 있으므로)
    let data = std::env::temp_dir().join("RobinX-setup-webview");
    // SAFETY: 아직 다른 스레드가 없다 (앱을 띄우기 전)
    unsafe { std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", data) };
    let _ = MODE.set(mode);
    None
}

/// 자동 업데이트·조용한 설치: 앱이 끝나기를 기다렸다가 설치하고, `/R`이면 다시 띄운다.
fn quiet_install(args: &Args, exe: &Path) -> i32 {
    let Some(env) = Env::from_system() else {
        log("사용자 폴더 위치를 찾지 못했어요");
        return 1;
    };
    let left = robin_setup::close_running(&installed_exes(&env), Duration::from_secs(15));
    if left > 0 {
        log(&format!(
            "실행 중인 RobinX {left}개를 끝내지 못했지만 설치를 이어가요"
        ));
    }
    let opts = InstallOptions {
        desktop_shortcut: false,
        update: args.has("/UPDATE"),
        no_shortcuts: args.has("/NS"),
    };
    match robin_setup::install(&env, exe, VERSION, &opts) {
        Ok(report) => {
            log(&format!("{VERSION} 설치함: {}", report.exe.display()));
            for w in &report.warnings {
                log(w);
            }
            if args.has("/R")
                && let Err(e) = spawn_detached(&report.exe, &args.relaunch_args())
            {
                log(&format!("다시 실행하지 못했어요: {e}"));
            }
            0
        }
        Err(e) => {
            log(&format!("설치 실패: {e}"));
            // 업데이트하느라 앱을 닫았으니, 설치가 안 됐어도 있는 RobinX를 다시 띄운다 (창 없이 사라지지 않게)
            if args.has("/R")
                && let Some(existing) = installed_exes(&env).into_iter().find(|p| p.is_file())
                && let Err(e) = spawn_detached(&existing, &args.relaunch_args())
            {
                log(&format!("다시 실행하지 못했어요: {e}"));
            }
            1
        }
    }
}

fn quiet_uninstall() -> i32 {
    let Some(env) = Env::from_system() else {
        return 1;
    };
    robin_setup::close_running(&installed_exes(&env), Duration::from_secs(5));
    match robin_setup::uninstall(&env, &UninstallOptions::default()) {
        Ok(_) => 0,
        Err(e) => {
            log(&format!("제거 실패: {e}"));
            1
        }
    }
}

// ── 화면에서 부르는 명령 ─────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupInfo {
    mode: Mode,
    version: &'static str,
    existing: Option<robin_setup::Existing>,
    install_dir: String,
    /// 설치된 RobinX가 실행 중
    running: bool,
    /// 백업·설정 폴더가 있음
    data_exists: bool,
}

#[tauri::command]
pub fn app_mode() -> Mode {
    mode()
}

#[tauri::command(async)]
pub fn setup_info() -> Result<SetupInfo, String> {
    let env = Env::from_system().ok_or("사용자 폴더 위치를 찾지 못했어요")?;
    Ok(SetupInfo {
        mode: mode(),
        version: VERSION,
        existing: robin_setup::existing(&env),
        install_dir: env.install_dir().display().to_string(),
        running: !robin_setup::running_copies(&installed_exes(&env)).is_empty(),
        data_exists: env.data_dir().exists(),
    })
}

#[tauri::command]
pub async fn run_install(desktop_shortcut: bool) -> Result<robin_setup::InstallReport, String> {
    if mode() != Mode::Setup {
        return Err("설치 모드가 아니에요".into());
    }
    let working = Working::start()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _working = working;
        let env = Env::from_system().ok_or("사용자 폴더 위치를 찾지 못했어요")?;
        let exe = current_exe().ok_or("설치 파일 위치를 찾지 못했어요")?;
        // 화면에서 이미 알렸으므로 실행 중인 RobinX는 잠깐 기다렸다가 끝낸다
        robin_setup::close_running(&installed_exes(&env), Duration::from_secs(3));
        let opts = InstallOptions {
            desktop_shortcut,
            ..Default::default()
        };
        let report = robin_setup::install(&env, &exe, VERSION, &opts)?;
        log(&format!(
            "{VERSION} 설치함 (화면): {}",
            report.exe.display()
        ));
        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn run_uninstall(delete_data: bool) -> Result<robin_setup::UninstallReport, String> {
    if mode() != Mode::Uninstall {
        return Err("제거 모드가 아니에요".into());
    }
    let working = Working::start()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _working = working;
        let env = Env::from_system().ok_or("사용자 폴더 위치를 찾지 못했어요")?;
        robin_setup::close_running(&installed_exes(&env), Duration::from_secs(3));
        robin_setup::uninstall(&env, &UninstallOptions { delete_data })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 설치를 마치고 RobinX를 연다
#[tauri::command]
pub fn launch_installed() -> Result<(), String> {
    let env = Env::from_system().ok_or("사용자 폴더 위치를 찾지 못했어요")?;
    spawn_detached(&env.install_dir().join(robin_setup::EXE), &[]).map_err(|e| e.to_string())
}

/// 설치·제거 화면을 닫는다 (제거였다면 임시 복사본을 지우게 해 둔다)
#[tauri::command]
pub fn finish_setup(app: tauri::AppHandle) {
    if on_close_requested() {
        app.exit(0);
    }
}
