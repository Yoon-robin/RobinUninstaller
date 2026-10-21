//! 정리 규칙. 우리가 직접 정한 목록이다 (다른 프로그램의 규칙 파일을 가져오지 않는다).
//!
//! 원칙 (CLAUDE.md "삭제 원칙"):
//! - **다시 만들어지는 것**(캐시·임시 파일)만 확인 뒤 바로 지운다.
//! - **되돌릴 수 없는 것**(덤프, 휴지통)은 따로 경고하고 기본 선택하지 않는다.
//! - **사용자 파일**(문서·다운로드·바탕 화면 등)은 규칙에 절대 넣지 않는다. 테스트가 이를 검사한다.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Risk {
    /// 지워도 프로그램이 다시 만든다 (캐시·임시 파일)
    Regenerable,
    /// 지우면 되돌릴 수 없다 (덤프, 휴지통)
    Irreversible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// 폴더 안의 파일을 지운다 (폴더 자체는 남긴다)
    Files,
    /// 휴지통 비우기 (Windows API)
    RecycleBin,
    /// 크기와 안내만 보여 준다 (우리가 지우지 않음)
    Info,
}

/// 경로의 기준 위치 (환경 변수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Temp,
    LocalAppData,
    AppData,
    UserProfile,
    SystemRoot,
    ProgramData,
    SystemDrive,
}

impl Base {
    pub fn env_var(self) -> &'static str {
        match self {
            Base::Temp => "TEMP",
            Base::LocalAppData => "LOCALAPPDATA",
            Base::AppData => "APPDATA",
            Base::UserProfile => "USERPROFILE",
            Base::SystemRoot => "SystemRoot",
            Base::ProgramData => "ProgramData",
            Base::SystemDrive => "SystemDrive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    /// 영문 소문자·숫자·하이픈 (관리자 작업자 인자로도 쓴다)
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// 왜 지워도 되는지 (화면의 "왜 안전한가")
    pub why: &'static str,
    pub risk: Risk,
    pub kind: Kind,
    pub admin: bool,
    /// (기준, 상대 경로). 경로 조각이 `*` 하나면 그 자리의 모든 하위 폴더 (브라우저 프로필 등).
    pub paths: &'static [(Base, &'static str)],
    /// 마지막 수정 뒤 이만큼(시간) 지난 파일만 지운다
    pub min_age_hours: u64,
    /// 이 프로그램이 실행 중이면 건너뛴다 (실행 파일 이름, 소문자)
    pub blocked_by: &'static [&'static str],
    /// Info 항목의 안내 문구
    pub guidance: Option<&'static str>,
}

impl Rule {
    /// 기본 선택: 다시 만들어지고, 관리자 권한이 필요 없는 정리 항목만
    pub fn default_selected(&self) -> bool {
        self.kind == Kind::Files && self.risk == Risk::Regenerable && !self.admin
    }
}

const fn regen(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    why: &'static str,
    paths: &'static [(Base, &'static str)],
    blocked_by: &'static [&'static str],
) -> Rule {
    Rule {
        id,
        name,
        description,
        why,
        risk: Risk::Regenerable,
        kind: Kind::Files,
        admin: false,
        paths,
        min_age_hours: 0,
        blocked_by,
        guidance: None,
    }
}

const CHROMIUM_CACHE: &str =
    "다시 방문하면 브라우저가 새로 받아 와요. 로그인·방문 기록·북마크는 건드리지 않아요.";

static RULES: &[Rule] = &[
    Rule {
        min_age_hours: 24,
        ..regen(
            "user-temp",
            "사용자 임시 파일",
            "프로그램이 설치·실행 중에 잠깐 쓰고 남긴 파일",
            "하루 넘게 손대지 않은 파일만 지워요. 지금 쓰는 파일은 잠겨 있어 건너뛰어요.",
            &[(Base::Temp, "")],
            &[],
        )
    },
    Rule {
        admin: true,
        min_age_hours: 24,
        ..regen(
            "windows-temp",
            "Windows 임시 파일",
            "Windows와 서비스가 남긴 임시 파일",
            "하루 넘게 손대지 않은 파일만 지워요. 관리자 권한이 필요해요.",
            &[(Base::SystemRoot, "Temp")],
            &[],
        )
    },
    Rule {
        admin: true,
        min_age_hours: 24,
        ..regen(
            "windows-update-cache",
            "Windows 업데이트 다운로드 파일",
            "Windows 업데이트가 설치하려고 받아 둔 파일",
            "업데이트를 설치하려고 받아 둔 원본이에요. 하루 넘게 손대지 않은 파일만 지우고, 지금 쓰는 파일은 잠겨 있어 건너뛰어요. 아직 설치하지 않은 업데이트가 있었다면 Windows가 다시 받아요. 관리자 권한이 필요해요.",
            &[(Base::SystemRoot, r"SoftwareDistribution\Download")],
            &[],
        )
    },
    Rule {
        admin: true,
        ..regen(
            "delivery-optimization",
            "배달 최적화 파일",
            "업데이트를 다른 PC와 나눠 받으려고 Windows가 모아 둔 파일",
            "이 PC의 업데이트에는 영향이 없고, 다른 PC에 나눠 줄 파일만 없어져요. 필요하면 Windows가 다시 받아요. 관리자 권한이 필요해요.",
            &[(
                Base::SystemRoot,
                r"ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache",
            )],
            &[],
        )
    },
    regen(
        "shader-cache",
        "그래픽 셰이더 캐시",
        "NVIDIA·AMD·Intel·DirectX가 게임용으로 만들어 둔 캐시",
        "게임을 처음 실행할 때 다시 만들어져요. 그때 잠깐 끊길 수 있어요.",
        &[
            (Base::LocalAppData, r"NVIDIA\DXCache"),
            (Base::LocalAppData, r"NVIDIA\GLCache"),
            (Base::LocalAppData, r"AMD\DxCache"),
            (Base::LocalAppData, r"AMD\DxcCache"),
            (Base::LocalAppData, r"AMD\GLCache"),
            (Base::LocalAppData, r"Intel\ShaderCache"),
            (Base::LocalAppData, "D3DSCache"),
        ],
        &[],
    ),
    regen(
        "chrome-cache",
        "Chrome 캐시",
        "Chrome이 웹페이지를 빨리 열려고 저장한 파일",
        CHROMIUM_CACHE,
        &[
            (Base::LocalAppData, r"Google\Chrome\User Data\*\Cache"),
            (Base::LocalAppData, r"Google\Chrome\User Data\*\Code Cache"),
            (Base::LocalAppData, r"Google\Chrome\User Data\*\GPUCache"),
        ],
        &["chrome.exe"],
    ),
    regen(
        "edge-cache",
        "Edge 캐시",
        "Edge가 웹페이지를 빨리 열려고 저장한 파일",
        CHROMIUM_CACHE,
        &[
            (Base::LocalAppData, r"Microsoft\Edge\User Data\*\Cache"),
            (Base::LocalAppData, r"Microsoft\Edge\User Data\*\Code Cache"),
            (Base::LocalAppData, r"Microsoft\Edge\User Data\*\GPUCache"),
        ],
        &["msedge.exe"],
    ),
    regen(
        "brave-cache",
        "Brave 캐시",
        "Brave가 웹페이지를 빨리 열려고 저장한 파일",
        CHROMIUM_CACHE,
        &[
            (
                Base::LocalAppData,
                r"BraveSoftware\Brave-Browser\User Data\*\Cache",
            ),
            (
                Base::LocalAppData,
                r"BraveSoftware\Brave-Browser\User Data\*\Code Cache",
            ),
            (
                Base::LocalAppData,
                r"BraveSoftware\Brave-Browser\User Data\*\GPUCache",
            ),
        ],
        &["brave.exe"],
    ),
    regen(
        "whale-cache",
        "웨일 캐시",
        "네이버 웨일이 웹페이지를 빨리 열려고 저장한 파일",
        CHROMIUM_CACHE,
        &[
            (Base::LocalAppData, r"Naver\Naver Whale\User Data\*\Cache"),
            (
                Base::LocalAppData,
                r"Naver\Naver Whale\User Data\*\Code Cache",
            ),
            (
                Base::LocalAppData,
                r"Naver\Naver Whale\User Data\*\GPUCache",
            ),
        ],
        &["whale.exe"],
    ),
    regen(
        "discord-cache",
        "Discord 캐시",
        "Discord가 이미지·동영상을 빨리 보여 주려고 저장한 파일",
        "Discord를 다시 열면 필요한 것만 새로 받아 와요. 대화 내용은 서버에 있어요.",
        &[
            (Base::AppData, r"discord\Cache"),
            (Base::AppData, r"discord\Code Cache"),
            (Base::AppData, r"discord\GPUCache"),
        ],
        &["discord.exe"],
    ),
    regen(
        "firefox-cache",
        "Firefox 캐시",
        "Firefox가 웹페이지를 빨리 열려고 저장한 파일",
        "다시 방문하면 새로 받아 와요. 로그인·방문 기록·북마크는 다른 곳에 있어 그대로예요.",
        &[(Base::LocalAppData, r"Mozilla\Firefox\Profiles\*\cache2")],
        &["firefox.exe"],
    ),
    regen(
        "vscode-cache",
        "VS Code 캐시",
        "VS Code가 받아 둔 확장 설치 파일과 화면 캐시",
        "확장 설치 파일은 필요할 때 다시 받아요. 설치된 확장·설정·작업 파일은 건드리지 않아요.",
        &[
            (Base::AppData, r"Code\CachedExtensionVSIXs"),
            (Base::AppData, r"Code\CachedData"),
            (Base::AppData, r"Code\Cache"),
            (Base::AppData, r"Code\Code Cache"),
            (Base::AppData, r"Code\GPUCache"),
        ],
        &["code.exe"],
    ),
    regen(
        "slack-cache",
        "Slack 캐시",
        "Slack이 이미지·파일 미리 보기를 빨리 보여 주려고 저장한 파일",
        "Slack을 다시 열면 필요한 것만 새로 받아 와요. 대화 내용은 서버에 있어요.",
        &[
            (Base::AppData, r"Slack\Cache"),
            (Base::AppData, r"Slack\Code Cache"),
            (Base::AppData, r"Slack\GPUCache"),
        ],
        &["slack.exe"],
    ),
    regen(
        "spotify-cache",
        "Spotify 캐시",
        "Spotify가 들었던 곡을 다시 빨리 틀려고 저장한 파일",
        "Spotify 설정의 '캐시 지우기'와 같아요. 다시 들으면 새로 받아 와요. 오프라인 저장한 곡(Storage 폴더)은 건드리지 않아요.",
        &[(Base::LocalAppData, r"Spotify\Data")],
        &["spotify.exe"],
    ),
    regen(
        "steam-web-cache",
        "Steam 웹 캐시",
        "Steam 앱 안의 상점·커뮤니티 화면이 저장한 파일",
        "Steam을 다시 열면 새로 받아 와요. 게임과 저장 파일은 건드리지 않아요.",
        &[(Base::LocalAppData, r"Steam\htmlcache")],
        &["steam.exe", "steamwebhelper.exe"],
    ),
    regen(
        "npm-cache",
        "npm 캐시",
        "Node.js 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 프로젝트는 그대로예요. 파일마다 내용 검사가 있어서, 쓰는 중에 지워져도 빠진 것만 다시 받아요. 지금 쓰는 파일은 잠겨 있어 건너뛰어요.",
        &[(Base::LocalAppData, "npm-cache")],
        // node.exe는 npm과 상관없는 도구(VS Code, 개발 서버 등)도 써서, 이것으로 막으면 거의 정리할 수 없었다
        &[],
    ),
    regen(
        "pip-cache",
        "pip 캐시",
        "Python 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 패키지는 그대로예요. 파일마다 내용 검사가 있어서, 쓰는 중에 지워져도 빠진 것만 다시 받아요. 지금 쓰는 파일은 잠겨 있어 건너뛰어요.",
        &[(Base::LocalAppData, r"pip\Cache")],
        // python.exe는 pip과 상관없는 프로그램도 많이 써서 막는 기준으로 쓰지 않는다
        &["pip.exe"],
    ),
    regen(
        "yarn-cache",
        "Yarn 캐시",
        "Yarn 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 프로젝트는 그대로예요. 파일마다 내용 검사가 있어서, 쓰는 중에 지워져도 빠진 것만 다시 받아요. 지금 쓰는 파일은 잠겨 있어 건너뛰어요.",
        &[(Base::LocalAppData, r"Yarn\Cache")],
        &[],
    ),
    regen(
        "cargo-cache",
        "Cargo 다운로드 캐시",
        "Rust 크레이트를 내려받아 둔 압축 파일과 풀어 둔 소스",
        "다음 빌드 때 인터넷에서 다시 받아 와요. 설치한 도구(cargo install)는 건드리지 않아요.",
        &[
            (Base::UserProfile, r".cargo\registry\cache"),
            (Base::UserProfile, r".cargo\registry\src"),
        ],
        // 실제로 빌드할 때만 막는다 (rust-analyzer는 편집기가 늘 띄워 둔다)
        &["cargo.exe", "rustc.exe"],
    ),
    regen(
        "gradle-cache",
        "Gradle 캐시",
        "Android·Java 빌드에 쓰는 라이브러리 사본",
        "다음 빌드 때 다시 받아 와요. 그래서 첫 빌드가 오래 걸려요. Gradle·Android Studio가 실행 중이면 건너뛰어요.",
        &[(Base::UserProfile, r".gradle\caches")],
        &["java.exe", "studio64.exe"],
    ),
    regen(
        "nuget-cache",
        "NuGet HTTP 캐시",
        ".NET 패키지 서버에서 받아 둔 응답",
        "다음 복원 때 다시 받아 와요. 설치된 패키지 폴더는 건드리지 않아요.",
        &[(Base::LocalAppData, r"NuGet\v3-cache")],
        &["dotnet.exe", "devenv.exe"],
    ),
    Rule {
        id: "crash-dumps",
        name: "프로그램 오류 덤프",
        description: "프로그램이 비정상 종료될 때 남긴 메모리 사본과 오류 보고",
        why: "오류 원인을 조사할 때만 쓰여요. 지우면 되돌릴 수 없지만 프로그램 동작에는 영향이 없어요.",
        risk: Risk::Irreversible,
        kind: Kind::Files,
        admin: false,
        paths: &[
            (Base::LocalAppData, "CrashDumps"),
            (Base::LocalAppData, r"Microsoft\Windows\WER\ReportArchive"),
            (Base::LocalAppData, r"Microsoft\Windows\WER\ReportQueue"),
        ],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: None,
    },
    Rule {
        id: "memory-dump",
        name: "Windows 메모리 덤프",
        description: "Windows가 블루스크린 때 남긴 메모리 사본",
        why: "블루스크린 원인을 조사할 때만 쓰여요. 지우면 되돌릴 수 없어요. 관리자 권한이 필요해요.",
        risk: Risk::Irreversible,
        kind: Kind::Files,
        admin: true,
        paths: &[
            (Base::SystemRoot, "MEMORY.DMP"),
            (Base::SystemRoot, "Minidump"),
            (Base::ProgramData, r"Microsoft\Windows\WER\ReportArchive"),
            (Base::ProgramData, r"Microsoft\Windows\WER\ReportQueue"),
        ],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: None,
    },
    Rule {
        id: "recycle-bin",
        name: "휴지통",
        description: "이미 지운 파일이 모여 있는 곳 (모든 드라이브)",
        why: "비우면 되돌릴 수 없어요. 휴지통 안에 필요한 파일이 없는지 먼저 확인하세요.",
        risk: Risk::Irreversible,
        kind: Kind::RecycleBin,
        admin: false,
        paths: &[],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: None,
    },
    Rule {
        id: "hibernation",
        name: "최대 절전 파일 (hiberfil.sys)",
        description: "최대 절전 모드와 빠른 시작을 위해 Windows가 메모리 크기만큼 잡아 둔 파일",
        why: "끄면 이 파일이 사라지지만 최대 절전 모드와 빠른 시작도 함께 꺼져요.",
        risk: Risk::Irreversible,
        kind: Kind::Info,
        admin: true,
        paths: &[(Base::SystemDrive, r"\hiberfil.sys")],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: Some(
            "최대 절전 모드를 쓰지 않는다면 관리자 권한 명령 프롬프트에서 `powercfg /hibernate off`를 실행하면 이 파일이 사라져요. \
             다시 켜려면 `powercfg /hibernate on`. 빠른 시작이 꺼져서 PC를 켜는 데 몇 초 더 걸릴 수 있어요.",
        ),
    },
    Rule {
        id: "windows-old",
        name: "이전 Windows 설치 (Windows.old)",
        description: "Windows 업데이트 전 버전의 사본",
        why: "지우면 이전 버전으로 되돌릴 수 없어요.",
        risk: Risk::Irreversible,
        kind: Kind::Info,
        admin: true,
        paths: &[(Base::SystemDrive, r"\Windows.old")],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: Some(
            "Windows 설정 → 시스템 → 저장소 → 임시 파일에서 \"이전 Windows 설치\"를 골라 지울 수 있어요.",
        ),
    },
    Rule {
        id: "wsl-disks",
        name: "WSL 가상 디스크",
        description: "WSL(리눅스) 배포판의 가상 디스크 파일",
        why: "리눅스 안에서 파일을 지워도 이 파일은 자동으로 줄지 않아요.",
        risk: Risk::Irreversible,
        kind: Kind::Info,
        admin: false,
        paths: &[
            (Base::LocalAppData, "wsl"),
            (Base::LocalAppData, r"Docker\wsl"),
        ],
        min_age_hours: 0,
        blocked_by: &[],
        guidance: Some(
            "PowerShell에서 `wsl --manage <배포판 이름> --set-sparse true`를 실행하면 빈 공간을 돌려받아요. 배포판을 지우면 안의 데이터도 사라지니 주의하세요.",
        ),
    },
];

/// 디버그 빌드에만 있는 시험용 규칙. 실제 캐시 대신 이것으로 정리 기능을 시험한다.
/// `fixture-admin-cache`는 관리자 권한 정리(UAC → 관리자 작업자) 흐름을 시험하는 항목이다.
/// 파일은 `robin-fixture install --machine`(관리자로 실행)이 만들고, 만든 사람이 관리자라 일반 권한으로는 지울 수 없다.
#[cfg(debug_assertions)]
static DEBUG_RULES: &[Rule] = &[
    regen(
        "fixture-cache",
        "테스트 앱 캐시 (개발용)",
        "테스트 앱 Fixture Notes가 만든 가짜 캐시. 디버그 빌드에만 보여요.",
        "robin-fixture install이 만든 시험용 파일이에요.",
        &[(Base::LocalAppData, r"Fixture Notes\cache")],
        &[],
    ),
    Rule {
        admin: true,
        ..regen(
            "fixture-admin-cache",
            "테스트 앱 관리자 캐시 (개발용)",
            "관리자 권한으로 만든 테스트 앱 Fixture Notes의 가짜 캐시. 디버그 빌드에만 보여요.",
            "robin-fixture install --machine이 만든 시험용 파일이에요. 관리자 권한이 필요해요.",
            &[(
                Base::ProgramData,
                r"Robin Test Labs\Fixture Notes\AdminCache",
            )],
            &[],
        )
    },
];
#[cfg(not(debug_assertions))]
static DEBUG_RULES: &[Rule] = &[];

/// 모든 규칙 (디버그 빌드면 시험용 규칙 포함)
pub fn all() -> impl Iterator<Item = &'static Rule> {
    RULES.iter().chain(DEBUG_RULES.iter())
}

pub fn find(id: &str) -> Option<&'static Rule> {
    all().find(|r| r.id == id)
}

/// 관리자 작업자 인자로 쓸 수 있는 ID인지
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && find(id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 사용자 파일이 들어 있는 폴더는 어떤 규칙에도 들어가면 안 된다.
    #[test]
    fn rules_never_touch_user_files() {
        let user_folders = [
            "desktop",
            "documents",
            "downloads",
            "pictures",
            "music",
            "videos",
            "onedrive",
            "saved games",
            "source",
            "projects",
        ];
        for rule in all() {
            for (base, rel) in rule.paths {
                let first = rel
                    .trim_start_matches('\\')
                    .split('\\')
                    .next()
                    .unwrap_or_default()
                    .to_lowercase();
                if *base == Base::UserProfile {
                    assert!(
                        !user_folders.contains(&first.as_str()),
                        "{}: 사용자 폴더 {rel}",
                        rule.id
                    );
                    assert!(
                        first.starts_with('.'),
                        "{}: 숨김 도구 폴더만 허용 {rel}",
                        rule.id
                    );
                }
                // 지우는 규칙은 기준 폴더 자체를 가리키면 안 된다 (TEMP만 예외: 안의 파일만 지운다)
                if rule.kind == Kind::Files && *base != Base::Temp {
                    assert!(!rel.is_empty(), "{}: 기준 폴더 전체", rule.id);
                }
            }
        }
    }

    #[test]
    fn ids_are_unique_and_valid() {
        let mut seen = std::collections::HashSet::new();
        for rule in all() {
            assert!(seen.insert(rule.id), "중복 ID {}", rule.id);
            assert!(is_valid_id(rule.id), "{}", rule.id);
        }
        assert!(!is_valid_id(r"..\x"));
        assert!(!is_valid_id("no-such-rule"));
    }

    /// 관리자 권한 정리를 시험하는 항목은 관리자 항목이고, 미리 고르지 않으며, 테스트 앱 폴더만 가리킨다
    #[cfg(debug_assertions)]
    #[test]
    fn admin_fixture_rule_is_admin_only() {
        let rule = find("fixture-admin-cache").expect("디버그 빌드에 있어야 함");
        assert!(rule.admin);
        assert!(!rule.default_selected());
        assert!(
            rule.paths
                .iter()
                .all(|(base, rel)| *base == Base::ProgramData
                    && rel.starts_with(r"Robin Test Labs\Fixture Notes\"))
        );
    }

    #[test]
    fn only_safe_items_are_selected_by_default() {
        for rule in all() {
            if rule.default_selected() {
                assert_eq!(rule.risk, Risk::Regenerable);
                assert!(!rule.admin);
            }
            if rule.risk == Risk::Irreversible {
                assert!(!rule.default_selected(), "{}", rule.id);
            }
        }
    }
}
