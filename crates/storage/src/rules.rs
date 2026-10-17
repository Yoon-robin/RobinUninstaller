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
        "npm-cache",
        "npm 캐시",
        "Node.js 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 프로젝트는 그대로예요.",
        &[(Base::LocalAppData, "npm-cache")],
        &["node.exe"],
    ),
    regen(
        "pip-cache",
        "pip 캐시",
        "Python 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 패키지는 그대로예요.",
        &[(Base::LocalAppData, r"pip\Cache")],
        &["pip.exe", "python.exe"],
    ),
    regen(
        "yarn-cache",
        "Yarn 캐시",
        "Yarn 패키지를 내려받아 둔 사본",
        "다음 설치 때 인터넷에서 다시 받아 와요. 이미 설치된 프로젝트는 그대로예요.",
        &[(Base::LocalAppData, r"Yarn\Cache")],
        &["node.exe"],
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
        &["cargo.exe", "rustc.exe", "rust-analyzer.exe"],
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
            "PowerShell에서 wsl --manage <배포판 이름> --set-sparse true 를 실행하면 빈 공간을 돌려받아요. 배포판을 지우면 안의 데이터도 사라지니 주의하세요.",
        ),
    },
];

/// 디버그 빌드에만 있는 시험용 규칙. 실제 캐시 대신 이것으로 정리 기능을 시험한다.
#[cfg(debug_assertions)]
static DEBUG_RULES: &[Rule] = &[regen(
    "fixture-cache",
    "테스트 앱 캐시 (개발용)",
    "테스트 앱 Fixture Notes가 만든 가짜 캐시. 디버그 빌드에만 보여요.",
    "robin-fixture install이 만든 시험용 파일이에요.",
    &[(Base::LocalAppData, r"Fixture Notes\cache")],
    &[],
)];
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
