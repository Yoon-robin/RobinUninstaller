//! 한 번의 스캔에 필요한 정보: 대상 프로그램, 다른 설치 프로그램, 기본 폴더 위치.

use robin_core::InstalledApplication;

use crate::names::{NameMatch, match_name, matches_publisher, significant_tokens};
use crate::protect::{KnownFolders, is_within, protected_path_reason};

#[derive(Debug, Clone)]
pub struct OtherApp {
    /// 설치 항목 ID (`범위:키 이름`)
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanContext {
    pub app_id: String,
    pub app_name: String,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    pub product_code: Option<String>,
    pub others: Vec<OtherApp>,
    pub known: KnownFolders,
    /// 이름으로 후보를 찾을 폴더들 (Program Files, AppData 등)
    pub file_roots: Vec<String>,
}

impl ScanContext {
    pub fn new(
        app: &InstalledApplication,
        installed: &[InstalledApplication],
        known: KnownFolders,
    ) -> Self {
        let install_location = app
            .install_location
            .as_deref()
            .map(|p| {
                p.trim()
                    .trim_matches('"')
                    .trim_end_matches('\\')
                    .to_string()
            })
            .filter(|p| !p.is_empty())
            // 드라이브 루트·Program Files·Windows 같은 곳을 설치 폴더로 적어 둔 프로그램도 있다.
            // 그런 곳을 기준으로 "설치 폴더를 가리킴"을 찾으면 다른 프로그램 항목까지 잡히므로 기준으로 쓰지 않는다.
            .filter(|p| protected_path_reason(p, &known).is_none())
            .filter(|p| {
                !is_shared_parent(
                    p,
                    &app.display_name,
                    app.publisher.as_deref(),
                    installed.iter().filter(|o| o.id != app.id),
                )
            });

        let others = installed
            .iter()
            .filter(|o| o.id != app.id)
            .map(|o| OtherApp {
                id: o.id.clone(),
                name: o.display_name.clone(),
                publisher: o.publisher.clone(),
                // 여러 제품을 담는 상위 폴더("C:\Program Files\Adobe")를 적어 둔 프로그램은 그 폴더로 공유를 판단하지 않는다.
                // 그러면 그 아래 다른 제품의 폴더가 모두 "함께 쓰는 것"이 되어 정작 지운 프로그램의 폴더를 못 찾는다.
                install_location: o
                    .install_location
                    .clone()
                    .filter(|p| !p.trim().is_empty())
                    .filter(|p| {
                        !is_shared_parent(
                            p,
                            &o.display_name,
                            o.publisher.as_deref(),
                            installed.iter().filter(|x| x.id != o.id),
                        )
                    }),
            })
            .collect();

        let mut file_roots: Vec<String> = known.program_files.clone();
        file_roots.extend(
            [
                known.program_data.clone(),
                known.app_data.clone(),
                known.local_app_data.clone(),
                format!("{}\\Programs", known.local_app_data),
                format!(
                    "{}\\Microsoft\\Windows\\Start Menu\\Programs",
                    known.app_data
                ),
                format!(
                    "{}\\Microsoft\\Windows\\Start Menu\\Programs",
                    known.program_data
                ),
            ]
            .into_iter()
            .filter(|p| !p.starts_with('\\')),
        );

        ScanContext {
            app_id: app.id.clone(),
            app_name: app.display_name.clone(),
            publisher: app.publisher.clone(),
            install_location,
            product_code: app.product_code.clone(),
            others,
            known,
            file_roots,
        }
    }

    /// RobinX 자신의 파일·설정은 절대 후보로 올리지 않는다.
    pub fn is_self(&self, location: &str) -> bool {
        robin_core::is_own_data(location)
    }

    /// 이 경로를 다른 설치 프로그램도 쓰는 것 같으면 그 프로그램 이름.
    pub fn other_app_sharing_path(&self, location: &str) -> Option<String> {
        let last = location
            .trim_end_matches('\\')
            .rsplit('\\')
            .next()
            .unwrap_or_default();
        self.others.iter().find_map(|o| {
            let by_location = o
                .install_location
                .as_deref()
                .is_some_and(|their| is_within(their, location) || is_within(location, their));
            (by_location || self.name_used_by(last, o)).then(|| o.name.clone())
        })
    }

    /// 다른 설치 프로그램이 이 이름(키·폴더 이름)을 쓰는지.
    pub fn other_app_named(&self, name: &str) -> Option<String> {
        self.others
            .iter()
            .find(|o| self.name_used_by(name, o))
            .map(|o| o.name.clone())
    }

    /// `name`(폴더·키 이름)이 다른 프로그램 `o`의 것으로도 볼 수 있는지.
    /// - 이름이 정확히 같음
    /// - 같은 게시자의 다른 제품이고 이름 단어가 겹침 ("Acrobat" 폴더 ↔ "Adobe Acrobat Reader")
    /// - 그 프로그램 게시자의 폴더·키 ("Adobe" ↔ 아직 설치된 "Adobe Photoshop")
    fn name_used_by(&self, name: &str, o: &OtherApp) -> bool {
        let m = match_name(name, &o.name, o.publisher.as_deref());
        if m == NameMatch::Exact {
            return true;
        }
        let same_publisher = match (self.publisher.as_deref(), o.publisher.as_deref()) {
            (Some(a), Some(b)) => matches_publisher(a, b) || matches_publisher(b, a),
            _ => false,
        };
        // 같은 게시자 제품끼리는 폴더 이름의 단어가 그 제품 이름에 다 들어 있어도 공유로 본다 ("Acrobat" ↔ "Acrobat Reader")
        let words = significant_tokens(name);
        let theirs = significant_tokens(&o.name);
        let subset = !words.is_empty() && words.iter().all(|w| theirs.contains(w));
        (same_publisher && (m == NameMatch::AllSignificant || subset))
            || o.publisher
                .as_deref()
                .is_some_and(|p| matches_publisher(name, p))
    }

    /// 경로가 다른 설치 프로그램의 설치 폴더 안이면 그 프로그램 이름 (Run 값·서비스 등이 가리키는 실행 파일용).
    pub fn other_app_owning(&self, path: &str) -> Option<String> {
        self.others.iter().find_map(|o| {
            let their = o.install_location.as_deref()?.trim().trim_matches('"');
            (protected_path_reason(their, &self.known).is_none() && is_within(path, their))
                .then(|| o.name.clone())
        })
    }

    /// 다른 설치 프로그램 중 이 ID가 있는지 (`범위:키 이름`, 소문자)
    pub fn other_app_by_id(&self, id: &str) -> Option<&OtherApp> {
        self.others.iter().find(|o| o.id.eq_ignore_ascii_case(id))
    }

    /// Uninstall 키 이름(`...\Uninstall\<이름>`)이 다른 설치 프로그램의 등록 정보인지.
    pub fn uninstall_key_of_other(&self, key_name: &str) -> Option<String> {
        self.others
            .iter()
            .find(|o| {
                o.id.split_once(':')
                    .is_some_and(|(_, k)| k.eq_ignore_ascii_case(key_name))
            })
            .map(|o| o.name.clone())
    }
}

/// 설치 폴더로 적힌 곳이 한 제품의 폴더가 아니라 여러 제품을 담는 상위 폴더인지.
/// - 폴더 이름이 게시자 이름이고 제품 이름과는 맞지 않음 (`C:\Program Files\Adobe`, `...\Epic Games`)
/// - 다른 설치 프로그램의 설치 폴더가 그 아래에 있음
///
/// 이런 곳을 기준으로 삼으면 같은 게시자의 다른 제품(시작 항목, 실행 파일 등록, 폴더)이 "설치 폴더를 가리킴"으로 잡혀
/// 미리 선택된다.
fn is_shared_parent<'a>(
    location: &str,
    name: &str,
    publisher: Option<&str>,
    others: impl Iterator<Item = &'a InstalledApplication>,
) -> bool {
    let loc = location.trim().trim_matches('"').trim_end_matches('\\');
    let last = loc.rsplit('\\').next().unwrap_or_default();
    let publisher_folder = publisher.is_some_and(|p| matches_publisher(last, p))
        && match_name(last, name, publisher) == NameMatch::None;
    publisher_folder
        || others
            .filter_map(|o| o.install_location.as_deref())
            .map(|p| p.trim().trim_matches('"').trim_end_matches('\\'))
            .any(|their| {
                !their.is_empty() && !their.eq_ignore_ascii_case(loc) && is_within(their, loc)
            })
}

#[cfg(test)]
mod tests {
    use super::*;
    use robin_core::{InstallScope, InstallTechnology, UninstallerStatus};

    fn app(id: &str, name: &str, publisher: &str, location: Option<&str>) -> InstalledApplication {
        InstalledApplication {
            id: id.into(),
            display_name: name.into(),
            publisher: Some(publisher.into()),
            version: None,
            install_location: location.map(Into::into),
            install_date: None,
            estimated_size_kb: None,
            display_icon: None,
            uninstall_command: None,
            quiet_uninstall_command: None,
            product_code: None,
            technology: InstallTechnology::Exe,
            scope: InstallScope::Machine64,
            uninstaller: UninstallerStatus::Available,
        }
    }

    fn known() -> KnownFolders {
        KnownFolders {
            windows: r"C:\Windows".into(),
            program_files: vec![r"C:\Program Files".into()],
            ..Default::default()
        }
    }

    #[test]
    fn sibling_products_of_the_same_publisher_share_folders() {
        let target = app("hklm64:acro", "Adobe Acrobat (64-bit)", "Adobe Inc.", None);
        let installed = vec![
            target.clone(),
            app("hklm64:reader", "Adobe Acrobat Reader", "Adobe Inc.", None),
            app("hklm64:other", "Notepad++", "Don Ho", None),
        ];
        let ctx = ScanContext::new(&target, &installed, known());
        assert_eq!(
            ctx.other_app_named("Acrobat").as_deref(),
            Some("Adobe Acrobat Reader")
        );
        // 아직 설치된 같은 게시자 제품이 있으면 게시자 폴더·키도 공유
        assert!(ctx.other_app_named("Adobe").is_some());
        assert!(ctx.other_app_named("Fixture Notes").is_none());
        assert_eq!(
            ctx.uninstall_key_of_other("READER").as_deref(),
            Some("Adobe Acrobat Reader")
        );
    }

    #[test]
    fn shared_parent_folder_is_not_an_anchor() {
        // 게시자 이름 폴더를 설치 폴더로 적어 둔 제품
        let ae = app(
            "hklm:ae",
            "Adobe After Effects 2026",
            "Adobe Inc.",
            Some(r"C:\Program Files\Adobe"),
        );
        let ctx = ScanContext::new(&ae, std::slice::from_ref(&ae), known());
        assert!(ctx.install_location.is_none());
        // 다른 프로그램의 설치 폴더를 품은 폴더
        let launcher = app("hklm:l", "Some Launcher", "Acme", Some(r"C:\Games"));
        let game = app("hklm:g", "Some Game", "Acme", Some(r"C:\Games\Some Game"));
        let ctx = ScanContext::new(&launcher, &[launcher.clone(), game.clone()], known());
        assert!(ctx.install_location.is_none());
        // 그런 폴더를 적어 둔 다른 프로그램 때문에 지운 프로그램의 폴더가 "공유"로 보이지 않는다
        let ps = app(
            "hklm:ps",
            "Adobe Photoshop 2026",
            "Adobe Inc.",
            Some(r"C:\Program Files\Adobe\Adobe Photoshop 2026"),
        );
        let ctx = ScanContext::new(&ps, &[ps.clone(), ae.clone()], known());
        assert_eq!(
            ctx.install_location.as_deref(),
            Some(r"C:\Program Files\Adobe\Adobe Photoshop 2026")
        );
        assert!(
            ctx.other_app_sharing_path(r"C:\Program Files\Adobe\Adobe Photoshop 2026")
                .is_none()
        );
        // 게시자와 제품 이름이 같은 폴더는 그 제품의 폴더다
        let spotify = app(
            "hkcu:s",
            "Spotify",
            "Spotify AB",
            Some(r"C:\Users\me\AppData\Roaming\Spotify"),
        );
        let ctx = ScanContext::new(&spotify, std::slice::from_ref(&spotify), known());
        assert!(ctx.install_location.is_some());
    }

    #[test]
    fn protected_install_location_is_not_an_anchor() {
        let target = app("hkcu:x", "X", "Y", Some(r"C:\"));
        let ctx = ScanContext::new(&target, std::slice::from_ref(&target), known());
        assert!(ctx.install_location.is_none());
        let target = app("hkcu:x", "X", "Y", Some(r"C:\Program Files\"));
        let ctx = ScanContext::new(&target, std::slice::from_ref(&target), known());
        assert!(ctx.install_location.is_none());
        let other = app("hkcu:o", "Other", "Z", Some(r"C:\Program Files\Other"));
        let ctx = ScanContext::new(&target, &[target.clone(), other], known());
        assert_eq!(
            ctx.other_app_owning(r"C:\Program Files\Other\bin\o.exe")
                .as_deref(),
            Some("Other")
        );
    }
}
