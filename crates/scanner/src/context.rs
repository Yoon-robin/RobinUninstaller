//! 한 번의 스캔에 필요한 정보: 대상 프로그램, 다른 설치 프로그램, 기본 폴더 위치.

use robin_core::InstalledApplication;

use crate::names::{NameMatch, match_name};
use crate::protect::{KnownFolders, is_within};

#[derive(Debug, Clone)]
pub struct OtherApp {
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
            .filter(|p| !p.is_empty());

        let others = installed
            .iter()
            .filter(|o| o.id != app.id)
            .map(|o| OtherApp {
                name: o.display_name.clone(),
                publisher: o.publisher.clone(),
                install_location: o.install_location.clone().filter(|p| !p.trim().is_empty()),
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

    /// RobinUninstaller 자신의 파일·설정은 절대 후보로 올리지 않는다.
    pub fn is_self(&self, location: &str) -> bool {
        let l = location.to_lowercase();
        l.contains("robinuninstaller") && !l.contains("robinfixture")
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
            let by_name = match_name(last, &o.name, o.publisher.as_deref()) == NameMatch::Exact;
            (by_location || by_name).then(|| o.name.clone())
        })
    }

    /// 다른 설치 프로그램이 이 이름(키·폴더 이름)을 자기 이름으로 쓰는지.
    pub fn other_app_named(&self, name: &str) -> Option<String> {
        self.others
            .iter()
            .find(|o| match_name(name, &o.name, o.publisher.as_deref()) == NameMatch::Exact)
            .map(|o| o.name.clone())
    }
}
