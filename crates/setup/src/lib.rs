//! RobinX 자체 설치 프로그램의 엔진. 설계는 `docs/INSTALLER.md`.
//!
//! - 설치하는 것은 robinx.exe(자기 자신) 하나와 등록 정보, 바로 가기다. 관리자 권한 없이 현재 사용자에게만 설치한다.
//! - 지우는 것은 설치 기록(`install.json`)에 적힌 파일과 바로 가기, 등록 키뿐이다. 폴더를 통째로 지우지 않는다.
//! - 0.1.0(NSIS)은 데이터 폴더(`%LOCALAPPDATA%\RobinX`)에 설치했으므로, 거기서는 실행 파일 두 개만 지운다.
//! - 모든 경로와 레지스트리 위치는 [`Env`]에서 나온다. 테스트는 임시 폴더와 시험용 레지스트리 위치를 쓴다.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(windows)]
mod process;
#[cfg(windows)]
mod shortcut;
#[cfg(windows)]
pub use process::{close_running, running_copies};

pub const PRODUCT: &str = "RobinX";
pub const EXE: &str = "robinx.exe";
pub const PUBLISHER: &str = "yoonrobin";
pub const HOMEPAGE: &str = "https://yoon-robin.github.io/RobinX/";
/// NSIS(0.1.0)가 만든 제거 프로그램 이름
const LEGACY_UNINSTALLER: &str = "uninstall.exe";
const MANIFEST: &str = "install.json";
const LINK_NAME: &str = "RobinX.lnk";

/// 설치에 쓰는 위치들. 실제로는 [`Env::from_system`], 테스트에서는 임시 폴더.
#[derive(Debug, Clone)]
pub struct Env {
    /// `%LOCALAPPDATA%`
    pub local_appdata: PathBuf,
    /// 시작 메뉴 `Programs` 폴더
    pub start_menu: PathBuf,
    /// 바탕 화면
    pub desktop: PathBuf,
    /// HKCU 아래에서 `Software` 역할을 하는 키 (실제: `Software`)
    pub reg_base: String,
}

impl Env {
    #[cfg(windows)]
    pub fn from_system() -> Option<Env> {
        let local_appdata = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
        let roaming = PathBuf::from(std::env::var_os("APPDATA")?);
        Some(Env {
            local_appdata,
            start_menu: roaming.join(r"Microsoft\Windows\Start Menu\Programs"),
            desktop: shortcut::desktop_dir()?,
            reg_base: "Software".into(),
        })
    }

    /// 새 설치 폴더: `%LOCALAPPDATA%\Programs\RobinX`
    pub fn install_dir(&self) -> PathBuf {
        self.local_appdata.join("Programs").join(PRODUCT)
    }

    /// 0.1.0(NSIS)의 설치 폴더이자 앱 데이터 폴더: `%LOCALAPPDATA%\RobinX`
    pub fn data_dir(&self) -> PathBuf {
        self.local_appdata.join(PRODUCT)
    }

    /// WebView2가 쓰는 앱 데이터 (`%LOCALAPPDATA%\<식별자>`)
    pub fn webview_dir(&self) -> PathBuf {
        self.local_appdata.join("com.yoonrobin.robinx")
    }

    pub fn uninstall_key(&self) -> String {
        format!(
            r"{}\Microsoft\Windows\CurrentVersion\Uninstall\{PRODUCT}",
            self.reg_base
        )
    }

    pub fn product_key(&self) -> String {
        format!(r"{}\{PUBLISHER}\{PRODUCT}", self.reg_base)
    }

    fn publisher_key(&self) -> String {
        format!(r"{}\{PUBLISHER}", self.reg_base)
    }
}

/// 이미 설치된 RobinX (등록 정보 기준)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Existing {
    pub dir: PathBuf,
    pub version: Option<String>,
    /// NSIS(0.1.0)로 설치됨 (설치 기록이 없음)
    pub legacy: bool,
}

/// 설치 폴더에 남기는 기록. 제거는 여기 적힌 것만 지운다.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    /// 설치 폴더 안의 파일 이름 (하위 경로 없음)
    pub files: Vec<String>,
    /// 만든 바로 가기 (전체 경로)
    pub shortcuts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// 바탕 화면 바로 가기를 만든다
    pub desktop_shortcut: bool,
    /// 자동 업데이트(`/UPDATE`): 바로 가기를 새로 만들지 않고, 있는 것만 새 위치로 고친다
    pub update: bool,
    /// 바로 가기를 만들지 않는다 (`/NS`)
    pub no_shortcuts: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    pub dir: PathBuf,
    pub exe: PathBuf,
    /// 예전 위치에서 지운 파일
    pub migrated: Vec<PathBuf>,
    /// 지우지 못했지만 설치는 끝난 것 (다음 설치·로그인 때 정리)
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UninstallOptions {
    /// 백업·설정·추적 기록(`%LOCALAPPDATA%\RobinX`)과 WebView2 데이터도 지운다. 되돌릴 수 없다.
    pub delete_data: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallReport {
    pub removed: Vec<PathBuf>,
    pub warnings: Vec<String>,
    /// 지우지 않고 남긴 데이터 폴더 (사용자가 고르지 않음)
    pub kept_data: Option<PathBuf>,
}

/// 설치 기록의 파일 이름은 설치 폴더 바로 아래의 평범한 이름만 받는다 (`..`, 경로 구분자 금지).
fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['\\', '/', ':'])
        && !name.contains("..")
}

/// 등록 정보의 경로에서 따옴표를 벗긴다 (NSIS는 `"C:\..."`처럼 저장한다).
fn unquote(s: &str) -> &str {
    s.trim().trim_matches('"')
}

fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        p.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .replace('/', "\\")
            .to_lowercase()
    };
    norm(a) == norm(b)
}

pub fn read_manifest(dir: &Path) -> Option<Manifest> {
    let text = std::fs::read_to_string(dir.join(MANIFEST)).ok()?;
    serde_json::from_str(&text).ok()
}

/// 임시 파일에 쓰고 바꿔치기한다. 쓰다 꺼지면 예전 기록이 그대로 남는다 (깨진 기록으로 제거가 엉뚱하게 돌지 않게).
fn write_manifest(dir: &Path, manifest: &Manifest) -> std::io::Result<()> {
    use std::io::Write;
    let text = serde_json::to_string_pretty(manifest).map_err(std::io::Error::other)?;
    let tmp = dir.join(format!("{MANIFEST}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, dir.join(MANIFEST)).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// 바꿔치기: `name.new`에 쓰고, 있던 파일은 `name.old`로 비켜 둔 뒤 이름을 바꾼다.
/// 실행 중인 exe도 이름은 바꿀 수 있어서, 앱이 아직 떠 있어도 설치가 된다 (옛 파일은 나중에 지운다).
fn replace_file(src: &Path, dest: &Path, warnings: &mut Vec<String>) -> std::io::Result<()> {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let new = dest.with_file_name(format!("{name}.new"));
    let old = dest.with_file_name(format!("{name}.old"));
    let _ = std::fs::remove_file(&new);
    std::fs::copy(src, &new)?;
    // 복사는 "인터넷에서 받은 파일" 표시(Zone.Identifier 스트림)까지 옮긴다. 그대로 두면 설치한 RobinX를 열 때마다
    // 보안 경고가 뜰 수 있다. 받은 설치 파일 자체는 이미 실행해서 확인을 거쳤다.
    let mut stream = new.clone().into_os_string();
    stream.push(":Zone.Identifier");
    let _ = std::fs::remove_file(stream);
    // 지난번에 지우지 못한 옛 파일
    let _ = std::fs::remove_file(&old);
    if dest.exists() {
        std::fs::rename(dest, &old)?;
    }
    if let Err(e) = std::fs::rename(&new, dest) {
        // 새 파일을 못 놓으면 원래 파일을 돌려놓는다
        let _ = std::fs::rename(&old, dest);
        return Err(e);
    }
    if old.exists() && std::fs::remove_file(&old).is_err() {
        warnings.push(format!("{}는 실행 중이라 다음에 지워요", old.display()));
    }
    Ok(())
}

#[cfg(windows)]
mod reg {
    use super::*;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    pub fn existing(env: &Env) -> Option<Existing> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(env.uninstall_key(), KEY_READ)
            .ok()?;
        let location: String = key.get_value("InstallLocation").ok()?;
        let dir = PathBuf::from(unquote(&location));
        if dir.as_os_str().is_empty() {
            return None;
        }
        let version = key.get_value::<String, _>("DisplayVersion").ok();
        let legacy = read_manifest(&dir).is_none();
        Some(Existing {
            dir,
            version,
            legacy,
        })
    }

    pub fn write(env: &Env, dir: &Path, exe: &Path, version: &str) -> std::io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(env.uninstall_key())?;
        let exe_q = format!("\"{}\"", exe.display());
        let size_kb = std::fs::metadata(exe).map(|m| m.len() / 1024).unwrap_or(0) as u32;
        key.set_value("DisplayName", &PRODUCT)?;
        key.set_value("DisplayIcon", &exe_q)?;
        key.set_value("DisplayVersion", &version)?;
        key.set_value("Publisher", &PUBLISHER)?;
        key.set_value("InstallLocation", &format!("\"{}\"", dir.display()))?;
        key.set_value("UninstallString", &format!("{exe_q} --uninstall"))?;
        key.set_value("QuietUninstallString", &format!("{exe_q} --uninstall /S"))?;
        key.set_value("MainBinaryName", &EXE)?;
        key.set_value("URLInfoAbout", &HOMEPAGE)?;
        key.set_value("NoModify", &1u32)?;
        key.set_value("NoRepair", &1u32)?;
        key.set_value("EstimatedSize", &size_kb)?;
        let (product, _) = hkcu.create_subkey(env.product_key())?;
        product.set_value("", &dir.display().to_string())?;
        Ok(())
    }

    pub fn remove(env: &Env) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(env.uninstall_key());
        let _ = hkcu.delete_subkey_all(env.product_key());
        // 다른 것이 없으면 게시자 키도 지운다 (비어 있지 않으면 실패하고 남는다)
        let _ = hkcu.delete_subkey(env.publisher_key());
    }
}

/// 등록 정보에 있는 설치. 없으면 None.
#[cfg(windows)]
pub fn existing(env: &Env) -> Option<Existing> {
    reg::existing(env)
}

/// `source`(지금 실행 중인 setup.exe = robinx.exe)를 설치한다.
#[cfg(windows)]
pub fn install(
    env: &Env,
    source: &Path,
    version: &str,
    opts: &InstallOptions,
) -> Result<InstallReport, String> {
    let dir = env.install_dir();
    let exe = dir.join(EXE);
    if same_path(source, &exe) {
        return Err("이미 설치된 RobinX에서 실행했어요. 받은 설치 파일을 실행해 주세요.".into());
    }
    let previous = existing(env);
    let old_manifest = read_manifest(&dir);
    let mut report = InstallReport {
        dir: dir.clone(),
        exe: exe.clone(),
        ..Default::default()
    };

    std::fs::create_dir_all(&dir).map_err(|e| format!("설치 폴더를 만들지 못했어요: {e}"))?;
    replace_file(source, &exe, &mut report.warnings)
        .map_err(|e| format!("RobinX를 설치 폴더에 복사하지 못했어요: {e}"))?;

    // 다른 위치(0.1.0 NSIS 등)에 있던 설치: 실행 파일만 지운다. 폴더와 그 안의 데이터는 그대로 둔다.
    let mut old_exes = Vec::new();
    if let Some(prev) = &previous
        && !same_path(&prev.dir, &dir)
    {
        for name in [EXE, LEGACY_UNINSTALLER] {
            let path = prev.dir.join(name);
            if path.is_file() {
                old_exes.push(path.clone());
                match std::fs::remove_file(&path) {
                    Ok(()) => report.migrated.push(path),
                    Err(e) => report.warnings.push(format!(
                        "예전 파일 {}를 지우지 못했어요: {e}",
                        path.display()
                    )),
                }
            }
        }
        // 데이터 폴더가 아니고 비었으면 지운다 (비어 있지 않으면 실패하고 남는다)
        if !same_path(&prev.dir, &env.data_dir()) {
            let _ = std::fs::remove_dir(&prev.dir);
        }
    }

    reg::write(env, &dir, &exe, version)
        .map_err(|e| format!("Windows에 설치 정보를 등록하지 못했어요: {e}"))?;

    // 바로 가기
    let mut shortcuts: Vec<PathBuf> = old_manifest.map(|m| m.shortcuts).unwrap_or_default();
    let start = env.start_menu.join(LINK_NAME);
    let desktop = env.desktop.join(LINK_NAME);
    let mut make = |link: &Path, wanted: bool, report: &mut InstallReport| {
        // 업데이트: 있던 바로 가기(예전 위치를 가리킬 수 있음)만 새 위치로 고친다
        let refresh =
            link.exists() && (opts.update || shortcut::points_to_any(link, &old_exes, &exe));
        if refresh || (wanted && !opts.no_shortcuts) {
            match shortcut::create(link, &exe) {
                Ok(()) => {
                    if !shortcuts.iter().any(|s| same_path(s, link)) {
                        shortcuts.push(link.to_path_buf());
                    }
                }
                Err(e) => report.warnings.push(format!(
                    "바로 가기 {}를 만들지 못했어요: {e}",
                    link.display()
                )),
            }
        }
    };
    make(&start, !opts.update, &mut report);
    make(&desktop, opts.desktop_shortcut && !opts.update, &mut report);
    shortcuts.retain(|s| s.exists());

    let manifest = Manifest {
        version: version.into(),
        files: vec![EXE.into()],
        shortcuts,
    };
    write_manifest(&dir, &manifest).map_err(|e| format!("설치 기록을 남기지 못했어요: {e}"))?;
    Ok(report)
}

/// 등록된 RobinX를 제거한다. 설치 기록에 있는 것만 지우고, 데이터는 `delete_data`일 때만 지운다.
/// 실행 중인 자기 자신은 지울 수 없으므로 호출하는 쪽이 임시 복사본에서 부른다.
#[cfg(windows)]
pub fn uninstall(env: &Env, opts: &UninstallOptions) -> Result<UninstallReport, String> {
    let dir = existing(env)
        .map(|e| e.dir)
        .unwrap_or_else(|| env.install_dir());
    let mut report = UninstallReport::default();

    let manifest = read_manifest(&dir).unwrap_or_else(|| Manifest {
        // NSIS(0.1.0) 설치: 기록이 없으므로 NSIS가 만든 것만 지운다
        version: String::new(),
        files: vec![EXE.into(), LEGACY_UNINSTALLER.into()],
        shortcuts: vec![env.start_menu.join(LINK_NAME), env.desktop.join(LINK_NAME)],
    });
    let exe = dir.join(EXE);

    for link in &manifest.shortcuts {
        // 다른 프로그램의 같은 이름 바로 가기는 건드리지 않는다
        if link.is_file() && shortcut::points_to_any(link, std::slice::from_ref(&exe), &exe) {
            match std::fs::remove_file(link) {
                Ok(()) => report.removed.push(link.clone()),
                Err(e) => report.warnings.push(format!("{}: {e}", link.display())),
            }
        }
    }
    for name in manifest.files.iter().filter(|n| is_plain_name(n)) {
        let path = dir.join(name);
        if path.is_file() {
            match std::fs::remove_file(&path) {
                Ok(()) => report.removed.push(path),
                Err(e) => report.warnings.push(format!("{}: {e}", path.display())),
            }
        }
    }
    for leftover in [
        format!("{EXE}.old"),
        format!("{EXE}.new"),
        MANIFEST.to_string(),
    ] {
        let _ = std::fs::remove_file(dir.join(leftover));
    }
    // 비었을 때만 지운다 (사용자가 넣은 파일이 있으면 폴더째 남는다)
    if !same_path(&dir, &env.data_dir()) {
        let _ = std::fs::remove_dir(&dir);
    }

    // 실행 파일을 지우지 못했으면(관리자 권한으로 실행 중인 RobinX 등) 등록 정보를 남긴다. 지우면 파일만 남고
    // 프로그램 목록에서 사라져 다시 제거할 방법이 없어진다.
    if exe.is_file() {
        report.warnings.push(format!(
            "{}를 지우지 못해 설치 정보를 남겨 두었어요. RobinX를 모두 닫고 다시 제거해 주세요",
            exe.display()
        ));
        return Ok(report);
    }
    reg::remove(env);

    if opts.delete_data {
        for data in [env.data_dir(), env.webview_dir()] {
            if data.exists() {
                match std::fs::remove_dir_all(&data) {
                    Ok(()) => report.removed.push(data),
                    Err(e) => report.warnings.push(format!("{}: {e}", data.display())),
                }
            }
        }
    } else if env.data_dir().exists() {
        report.kept_data = Some(env.data_dir());
    }
    Ok(report)
}

#[cfg(all(test, windows))]
mod tests;
