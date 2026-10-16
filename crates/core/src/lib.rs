//! RobinUninstaller 공통 모델.
//!
//! UI(Tauri)와 엔진 크레이트가 함께 쓰는 타입만 둔다. 시스템을 건드리는 코드는 여기에 넣지 않는다.

use serde::Serialize;

/// 프로그램이 등록된 레지스트리 위치.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallScope {
    /// HKLM, 64비트 뷰
    Machine64,
    /// HKLM, 32비트 뷰 (WOW6432Node)
    Machine32,
    /// HKCU (현재 사용자)
    User,
}

/// 설치 방식.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallTechnology {
    Exe,
    Msi,
}

/// 기본 언인스톨러를 실행할 수 있는 상태인지.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UninstallerStatus {
    /// 언인스톨러 파일이 있음 (MSI는 Windows Installer에 제품이 등록돼 있음)
    Available,
    /// 등록 정보는 있지만 언인스톨러 파일(또는 MSI 제품)이 없음 → 강제 제거 후보
    Missing,
    /// 제거 명령이 등록돼 있지 않음
    NotRegistered,
    /// 명령 형식을 해석하지 못해 확인할 수 없음
    Unknown,
}

/// 설치된 프로그램 하나.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApplication {
    /// 범위 + 레지스트리 키 이름으로 만든 고유 ID. 표시 이름은 겹칠 수 있어서 쓰지 않는다.
    pub id: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_location: Option<String>,
    pub install_date: Option<String>,
    /// 레지스트리에 기록된 예상 크기 (KB)
    pub estimated_size_kb: Option<u64>,
    pub display_icon: Option<String>,
    pub uninstall_command: Option<String>,
    pub quiet_uninstall_command: Option<String>,
    /// MSI 제품 코드 (`{GUID}`)
    pub product_code: Option<String>,
    pub technology: InstallTechnology,
    pub scope: InstallScope,
    pub uninstaller: UninstallerStatus,
}

impl InstallScope {
    pub fn key_prefix(self) -> &'static str {
        match self {
            InstallScope::Machine64 => "hklm64",
            InstallScope::Machine32 => "hklm32",
            InstallScope::User => "hkcu",
        }
    }
}

/// `scope`와 레지스트리 하위 키 이름으로 ID를 만든다.
pub fn make_app_id(scope: InstallScope, key_name: &str) -> String {
    format!("{}:{}", scope.key_prefix(), key_name.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_id_is_scope_qualified_and_case_insensitive() {
        assert_eq!(make_app_id(InstallScope::User, "{ABC}"), "hkcu:{abc}");
        assert_ne!(
            make_app_id(InstallScope::Machine64, "Foo"),
            make_app_id(InstallScope::Machine32, "Foo")
        );
    }
}
