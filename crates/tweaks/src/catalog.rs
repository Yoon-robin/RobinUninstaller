//! 최적화 항목 목록.
//!
//! 원칙 (CLAUDE.md "최적화 원칙"):
//! - Windows 설정 앱에 있는 **공식 설정**을 현재 사용자(HKCU) 범위에서만 바꾼다. 관리자 권한이 필요 없다.
//! - 정책 키(`Software\Policies`)는 쓰지 않는다: "조직에서 관리" 표시가 생기고 설정 앱에서 못 바꾸게 된다.
//! - 보안(Defender, SmartScreen, UAC, 업데이트, 방화벽)은 건드리지 않는다. `tests`가 이를 확인한다.
//! - 효과를 설명할 수 있는 것만. "성능 n% 향상" 같은 말은 쓰지 않는다.

use crate::model::{Choice, Group, RegChange, Restart, Tweak, Value};

const ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const CDM: &str = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
const SEARCH: &str = r"Software\Microsoft\Windows\CurrentVersion\Search";

const fn dword(key: &'static str, name: &'static str, v: u32) -> RegChange {
    RegChange {
        key,
        name,
        value: Value::Dword(v),
    }
}

/// 항목마다 같은 기본값 (`..BASE`로 채운다)
const BASE: Tweak = Tweak {
    id: "",
    group: Group::Privacy,
    title: "",
    description: "",
    why: "",
    settings_path: "",
    restart: Restart::None,
    min_build: 0,
    changes: &[],
    options: &[],
    default_option: None,
};

const fn choice(id: &'static str, label: &'static str, changes: &'static [RegChange]) -> Choice {
    Choice { id, label, changes }
}

pub const TWEAKS: &[Tweak] = &[
    // ── 개인정보 ──
    Tweak {
        id: "privacy.advertising-id",
        group: Group::Privacy,
        title: "앱 맞춤 광고 끄기",
        description: "앱이 광고 ID로 나에게 맞춘 광고를 보여 주지 않게 해요.",
        why: "광고 ID는 앱 사이에서 같은 사람으로 알아보는 데 쓰여요. 꺼도 광고 수는 같고, 맞춤이 아닐 뿐이에요.",
        settings_path: "설정 > 개인 정보 및 보안 > 일반",
        restart: Restart::None,
        min_build: 0,
        changes: &[dword(
            r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
            "Enabled",
            0,
        )],
        ..BASE
    },
    Tweak {
        id: "privacy.tailored-experiences",
        group: Group::Privacy,
        title: "진단 데이터로 맞춤 팁·광고 받지 않기",
        description: "Microsoft가 진단 데이터를 바탕으로 팁과 광고, 추천을 맞추지 않게 해요.",
        why: "진단 데이터 자체를 보내는 설정은 그대로이고, 그 데이터를 맞춤 제안에 쓰는 것만 꺼요.",
        settings_path: "설정 > 개인 정보 및 보안 > 진단 및 피드백 > 맞춤형 환경",
        restart: Restart::None,
        min_build: 0,
        changes: &[dword(
            r"Software\Microsoft\Windows\CurrentVersion\Privacy",
            "TailoredExperiencesWithDiagnosticDataEnabled",
            0,
        )],
        ..BASE
    },
    Tweak {
        id: "privacy.language-list",
        group: Group::Privacy,
        title: "웹사이트에 내 언어 목록 알리지 않기",
        description: "웹사이트가 내 PC의 언어 목록을 보고 맞춤 콘텐츠를 보여 주지 않게 해요.",
        why: "언어 목록은 사람을 구별하는 단서가 될 수 있어요. 끄면 일부 사이트가 언어를 직접 고르게 할 수 있어요.",
        settings_path: "설정 > 개인 정보 및 보안 > 일반",
        restart: Restart::None,
        min_build: 0,
        changes: &[dword(
            r"Control Panel\International\User Profile",
            "HttpAcceptLanguageOptOut",
            1,
        )],
        ..BASE
    },
    Tweak {
        id: "privacy.app-launch-tracking",
        group: Group::Privacy,
        title: "앱 실행 기록으로 시작 메뉴 꾸미지 않기",
        description: "자주 쓰는 앱을 시작 메뉴와 검색에 띄우려고 앱 실행을 기록하지 않게 해요.",
        why: "끄면 시작 메뉴의 '자주 사용하는 앱'이 보이지 않을 수 있어요.",
        settings_path: "설정 > 개인 정보 및 보안 > 일반",
        restart: Restart::Explorer,
        min_build: 0,
        changes: &[dword(ADVANCED, "Start_TrackProgs", 0)],
        ..BASE
    },
    Tweak {
        id: "privacy.feedback-requests",
        group: Group::Privacy,
        title: "피드백 요청 받지 않기",
        description: "Windows가 사용 경험에 대한 의견을 묻는 알림을 띄우지 않게 해요.",
        why: "피드백 허브에서 직접 의견을 보내는 것은 그대로 할 수 있어요.",
        settings_path: "설정 > 개인 정보 및 보안 > 진단 및 피드백 > 피드백 빈도",
        restart: Restart::None,
        min_build: 0,
        changes: &[dword(
            r"Software\Microsoft\Siuf\Rules",
            "NumberOfSIUFInPeriod",
            0,
        )],
        ..BASE
    },
    // ── 추천·제안 ──
    Tweak {
        id: "suggestions.settings-app",
        group: Group::Suggestions,
        title: "설정 앱의 추천 콘텐츠 끄기",
        description: "설정 앱에 뜨는 추천 콘텐츠와 제안을 숨겨요.",
        why: "Microsoft 서비스나 기능을 권하는 내용이에요. 꺼도 설정 기능은 그대로예요.",
        settings_path: "설정 > 개인 정보 및 보안 > 일반",
        restart: Restart::None,
        min_build: 0,
        changes: &[
            dword(CDM, "SubscribedContent-338393Enabled", 0),
            dword(CDM, "SubscribedContent-353694Enabled", 0),
            dword(CDM, "SubscribedContent-353696Enabled", 0),
        ],
        ..BASE
    },
    Tweak {
        id: "suggestions.tips",
        group: Group::Suggestions,
        title: "Windows 팁과 제안 알림 끄기",
        description: "Windows를 쓰는 동안 팁·요령·제안 알림을 띄우지 않게 해요.",
        why: "새 기능 소개나 앱 권유 알림이 줄어요.",
        settings_path: "설정 > 시스템 > 알림 > 추가 설정",
        restart: Restart::None,
        min_build: 0,
        changes: &[
            dword(CDM, "SubscribedContent-338389Enabled", 0),
            dword(CDM, "SoftLandingEnabled", 0),
        ],
        ..BASE
    },
    Tweak {
        id: "suggestions.start-menu",
        group: Group::Suggestions,
        title: "시작 메뉴의 추천 앱·팁 끄기",
        description: "시작 메뉴에 설치하지 않은 앱 추천과 팁을 띄우지 않게 해요.",
        why: "시작 메뉴가 내가 설치한 앱과 파일 위주로 정리돼요.",
        settings_path: "설정 > 개인 설정 > 시작",
        restart: Restart::Explorer,
        min_build: 0,
        changes: &[
            dword(CDM, "SystemPaneSuggestionsEnabled", 0),
            dword(CDM, "SubscribedContent-338388Enabled", 0),
            dword(ADVANCED, "Start_IrisRecommendations", 0),
        ],
        ..BASE
    },
    Tweak {
        id: "suggestions.lock-screen",
        group: Group::Suggestions,
        title: "잠금 화면의 재미있는 정보·팁 끄기",
        description: "잠금 화면에 뜨는 팁, 재미있는 정보, 광고성 문구를 숨겨요.",
        why: "잠금 화면 배경은 그대로이고 그 위의 글만 사라져요.",
        settings_path: "설정 > 개인 설정 > 잠금 화면",
        restart: Restart::None,
        min_build: 0,
        changes: &[
            dword(CDM, "RotatingLockScreenOverlayEnabled", 0),
            dword(CDM, "SubscribedContent-338387Enabled", 0),
        ],
        ..BASE
    },
    Tweak {
        id: "suggestions.copilot-button",
        group: Group::Suggestions,
        title: "작업 표시줄의 Copilot 버튼 숨기기",
        description: "작업 표시줄에서 Copilot 버튼을 숨겨요.",
        why: "버튼만 숨기고 Copilot 앱은 지우지 않아요. 시작 메뉴에서 계속 열 수 있어요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄",
        restart: Restart::Explorer,
        min_build: 22000,
        changes: &[dword(ADVANCED, "ShowCopilotButton", 0)],
        ..BASE
    },
    // ── 편의 ──
    Tweak {
        id: "convenience.file-extensions",
        group: Group::Convenience,
        title: "파일 확장자 보이기",
        description: "탐색기에서 '사진.jpg', '설치.exe'처럼 파일 이름 끝의 확장자를 보여 줘요.",
        why: "문서처럼 보이는 실행 파일(예: 보고서.pdf.exe)을 알아보기 쉬워져요.",
        settings_path: "탐색기 > 보기 > 표시 > 파일 확장명",
        restart: Restart::Explorer,
        min_build: 0,
        changes: &[dword(ADVANCED, "HideFileExt", 0)],
        ..BASE
    },
    Tweak {
        id: "convenience.sticky-keys-prompt",
        group: Group::Convenience,
        title: "Shift 다섯 번 눌러도 고정 키 창 띄우지 않기",
        description: "게임이나 작업 중 Shift를 연달아 눌렀을 때 고정 키를 켤지 묻는 창이 뜨지 않게 해요.",
        why: "고정 키 기능 자체는 설정에서 그대로 켤 수 있어요.",
        settings_path: "설정 > 접근성 > 키보드 > 고정 키",
        restart: Restart::SignOut,
        min_build: 0,
        changes: &[RegChange {
            key: r"Control Panel\Accessibility\StickyKeys",
            name: "Flags",
            value: Value::Str("506"),
        }],
        ..BASE
    },
    // ── 게임 ──
    Tweak {
        id: "gaming.background-recording",
        group: Group::Gaming,
        title: "게임 백그라운드 녹화 끄기",
        description: "게임하는 동안 Xbox Game Bar가 화면을 계속 녹화해 두지 않게 해요.",
        why: "방금 플레이를 저장하는 기능을 안 쓴다면 끄는 게 좋아요. Game Bar의 다른 기능은 그대로예요.",
        settings_path: "설정 > 게임 > 캡처",
        restart: Restart::None,
        min_build: 0,
        changes: &[
            dword(r"System\GameConfigStore", "GameDVR_Enabled", 0),
            dword(
                r"Software\Microsoft\Windows\CurrentVersion\GameDVR",
                "AppCaptureEnabled",
                0,
            ),
        ],
        ..BASE
    },
    // ── 작업 표시줄 ──
    Tweak {
        id: "taskbar.alignment",
        group: Group::Taskbar,
        title: "작업 표시줄 아이콘 위치",
        description: "작업 표시줄의 시작 단추와 앱 아이콘을 가운데 또는 왼쪽에 둬요.",
        why: "Windows 10처럼 왼쪽이 익숙하면 왼쪽으로 옮길 수 있어요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄 > 작업 표시줄 동작",
        restart: Restart::Explorer,
        min_build: 22000,
        options: &[
            choice("center", "가운데", &[dword(ADVANCED, "TaskbarAl", 1)]),
            choice("left", "왼쪽", &[dword(ADVANCED, "TaskbarAl", 0)]),
        ],
        default_option: Some("center"),
        ..BASE
    },
    Tweak {
        id: "taskbar.combine",
        group: Group::Taskbar,
        title: "작업 표시줄 단추 합치기",
        description: "같은 앱의 창을 한 단추로 합칠지, 이름표를 보여 줄지 정해요.",
        why: "창을 여러 개 띄워 두고 오가는 일이 많으면 '합치지 않기'가 편해요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄 > 작업 표시줄 동작",
        restart: Restart::Explorer,
        min_build: 22631,
        options: &[
            choice(
                "always",
                "항상 합치기",
                &[
                    dword(ADVANCED, "TaskbarGlomLevel", 0),
                    dword(ADVANCED, "MMTaskbarGlomLevel", 0),
                ],
            ),
            choice(
                "when-full",
                "가득 차면 합치기",
                &[
                    dword(ADVANCED, "TaskbarGlomLevel", 1),
                    dword(ADVANCED, "MMTaskbarGlomLevel", 1),
                ],
            ),
            choice(
                "never",
                "합치지 않기",
                &[
                    dword(ADVANCED, "TaskbarGlomLevel", 2),
                    dword(ADVANCED, "MMTaskbarGlomLevel", 2),
                ],
            ),
        ],
        default_option: Some("always"),
        ..BASE
    },
    Tweak {
        id: "taskbar.search",
        group: Group::Taskbar,
        title: "작업 표시줄 검색",
        description: "작업 표시줄의 검색을 숨기거나, 아이콘이나 검색 상자로 보여 줘요.",
        why: "숨겨도 시작 메뉴를 열고 바로 입력하면 똑같이 검색돼요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄 > 작업 표시줄 항목",
        restart: Restart::Explorer,
        min_build: 0,
        options: &[
            choice(
                "hidden",
                "숨기기",
                &[dword(SEARCH, "SearchboxTaskbarMode", 0)],
            ),
            choice(
                "icon",
                "아이콘만",
                &[dword(SEARCH, "SearchboxTaskbarMode", 1)],
            ),
            choice(
                "box",
                "검색 상자",
                &[dword(SEARCH, "SearchboxTaskbarMode", 2)],
            ),
        ],
        ..BASE
    },
    Tweak {
        id: "taskbar.task-view-button",
        group: Group::Taskbar,
        title: "작업 보기 단추 숨기기",
        description: "작업 표시줄에서 작업 보기(가상 데스크톱) 단추를 숨겨요.",
        why: "Win+Tab으로 똑같이 열 수 있어요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄 > 작업 표시줄 항목",
        restart: Restart::Explorer,
        changes: &[dword(ADVANCED, "ShowTaskViewButton", 0)],
        ..BASE
    },
    Tweak {
        id: "taskbar.clock-seconds",
        group: Group::Taskbar,
        title: "시계에 초 보이기",
        description: "작업 표시줄 시계에 초까지 보여 줘요.",
        why: "시간을 정확히 봐야 할 때 편해요. 전력을 아주 조금 더 쓸 수 있어요.",
        settings_path: "설정 > 개인 설정 > 작업 표시줄 > 작업 표시줄 동작",
        restart: Restart::Explorer,
        min_build: 22621,
        changes: &[dword(ADVANCED, "ShowSecondsInSystemClock", 1)],
        ..BASE
    },
    Tweak {
        id: "taskbar.end-task",
        group: Group::Taskbar,
        title: "작업 표시줄에서 작업 끝내기",
        description: "작업 표시줄 앱을 오른쪽 클릭하면 '작업 끝내기'가 나와요.",
        why: "응답하지 않는 앱을 작업 관리자 없이 바로 닫을 수 있어요.",
        settings_path: "설정 > 시스템 > 개발자용 > 작업 끝내기",
        restart: Restart::Explorer,
        min_build: 22631,
        changes: &[dword(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings",
            "TaskbarEndTask",
            1,
        )],
        ..BASE
    },
    // ── 편의 (탐색기·기타) ──
    Tweak {
        id: "convenience.hidden-files",
        group: Group::Convenience,
        title: "숨김 파일 보이기",
        description: "탐색기에서 숨김 파일과 폴더(예: AppData)를 보여 줘요.",
        why: "남은 흔적이나 설정 파일을 직접 찾을 때 편해요. Windows 보호 파일은 여전히 숨겨져요.",
        settings_path: "탐색기 > 보기 > 표시 > 숨긴 항목",
        restart: Restart::Explorer,
        changes: &[dword(ADVANCED, "Hidden", 1)],
        ..BASE
    },
    Tweak {
        id: "convenience.compact-explorer",
        group: Group::Convenience,
        title: "탐색기 간격 줄이기",
        description: "탐색기 목록의 줄 간격을 줄여 한 화면에 더 많이 보여 줘요.",
        why: "터치보다 마우스를 주로 쓰면 더 많은 파일을 한눈에 볼 수 있어요.",
        settings_path: "탐색기 > 보기 > 간격 줄이기",
        restart: Restart::Explorer,
        min_build: 22000,
        changes: &[dword(ADVANCED, "UseCompactMode", 1)],
        ..BASE
    },
    Tweak {
        id: "convenience.explorer-start",
        group: Group::Convenience,
        title: "탐색기를 열면 먼저 보일 곳",
        description: "탐색기를 열었을 때 '홈'과 '내 PC' 중 어디를 먼저 보여 줄지 정해요.",
        why: "드라이브와 폴더로 바로 가고 싶다면 '내 PC'가 편해요.",
        settings_path: "탐색기 > ... > 옵션 > 일반 > 파일 탐색기 열기",
        restart: Restart::Explorer,
        options: &[
            choice("home", "홈", &[dword(ADVANCED, "LaunchTo", 2)]),
            choice("this-pc", "내 PC", &[dword(ADVANCED, "LaunchTo", 1)]),
        ],
        default_option: Some("home"),
        ..BASE
    },
    Tweak {
        id: "convenience.clipboard-history",
        group: Group::Convenience,
        title: "클립보드 기록 켜기",
        description: "복사한 내용을 여러 개 기억해 두고 Win+V로 골라 붙여 넣을 수 있게 해요.",
        why: "기록은 이 PC에만 남고 다시 시작하면 지워져요(고정한 항목 제외). 비밀번호를 자주 복사한다면 신중히 켜세요.",
        settings_path: "설정 > 시스템 > 클립보드",
        changes: &[dword(
            r"Software\Microsoft\Clipboard",
            "EnableClipboardHistory",
            1,
        )],
        ..BASE
    },
    Tweak {
        id: "suggestions.explorer-ads",
        group: Group::Suggestions,
        title: "탐색기의 동기화 서비스 알림 끄기",
        description: "탐색기 위쪽에 뜨는 OneDrive·Microsoft 365 같은 서비스 안내를 숨겨요.",
        why: "탐색기 기능은 그대로이고 권유 문구만 사라져요.",
        settings_path: "탐색기 > ... > 옵션 > 보기 > 동기화 공급자 알림 표시",
        restart: Restart::Explorer,
        changes: &[dword(ADVANCED, "ShowSyncProviderNotifications", 0)],
        ..BASE
    },
];

/// 개발용 테스트 항목. 테스트 앱의 키만 건드린다 (디버그 빌드에서만 목록에 나온다).
pub const TEST_TWEAK: Tweak = Tweak {
    id: "test.fixture",
    group: Group::Test,
    title: "테스트 앱 설정 (개발용)",
    description: "테스트 앱 Fixture Notes의 설정 값 두 개를 바꿔요. 실제 Windows 설정은 건드리지 않아요.",
    why: "적용·되돌리기를 실제 설정 없이 시험하려고 있어요.",
    settings_path: "",
    restart: Restart::None,
    min_build: 0,
    changes: &[
        dword(r"Software\Robin Test Labs\Tweaks", "Turbo", 1),
        RegChange {
            key: r"Software\Robin Test Labs\Tweaks",
            name: "Mode",
            value: Value::Str("quiet"),
        },
    ],
    ..BASE
};

/// 개발용 선택지 테스트 항목. 테스트 앱의 키만 건드린다.
pub const TEST_CHOICE: Tweak = Tweak {
    id: "test.choice",
    group: Group::Test,
    title: "테스트 앱 단계 (개발용)",
    description: "테스트 앱 Fixture Notes의 단계 값을 고르는 선택지 항목이에요.",
    why: "선택지 적용·바꾸기·되돌리기를 실제 설정 없이 시험하려고 있어요.",
    options: &[
        choice(
            "low",
            "낮음",
            &[dword(r"Software\Robin Test Labs\Tweaks", "Level", 1)],
        ),
        choice(
            "high",
            "높음",
            &[dword(r"Software\Robin Test Labs\Tweaks", "Level", 2)],
        ),
    ],
    default_option: Some("low"),
    ..BASE
};

/// 이 빌드에서 보여 줄 항목 (디버그 빌드에는 테스트 항목이 붙는다).
pub fn all() -> Vec<&'static Tweak> {
    let mut list: Vec<&'static Tweak> = TWEAKS.iter().collect();
    if cfg!(debug_assertions) {
        list.push(&TEST_TWEAK);
        list.push(&TEST_CHOICE);
    }
    list
}

pub fn find(id: &str) -> Option<&'static Tweak> {
    all().into_iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 어떤 항목도 이런 곳을 건드리면 안 된다 (대소문자 무시, 경로 일부만 맞아도)
    const FORBIDDEN: &[&str] = &[
        r"\policies",
        "windows defender",
        "smartscreen",
        "windowsupdate",
        "firewall",
        "enablelua",
        "consentprompt",
        "currentcontrolset",
        "lsa",
        r"\run",
        "winlogon",
    ];

    #[test]
    fn ids_are_unique_and_well_formed() {
        let mut ids: Vec<&str> = TWEAKS
            .iter()
            .map(|t| t.id)
            .chain([TEST_TWEAK.id, TEST_CHOICE.id])
            .collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "중복 ID");
        for id in ids {
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-'),
                "{id}"
            );
        }
    }

    #[test]
    fn each_tweak_is_either_a_toggle_or_a_choice() {
        for t in TWEAKS.iter().chain([&TEST_TWEAK, &TEST_CHOICE]) {
            assert!(
                t.changes.is_empty() != t.options.is_empty(),
                "{}: 켜기/끄기 값과 선택지 중 하나만 있어야 한다",
                t.id
            );
            if t.is_choice() {
                assert!(t.options.len() >= 2, "{}", t.id);
                let mut ids: Vec<&str> = t.options.iter().map(|o| o.id).collect();
                ids.sort_unstable();
                ids.dedup();
                assert_eq!(ids.len(), t.options.len(), "{}: 선택지 ID 중복", t.id);
                for o in t.options {
                    assert!(!o.changes.is_empty(), "{}/{}", t.id, o.id);
                }
                // 선택지마다 같은 값들을 다뤄야 되돌리기가 정확하다
                let keys = |o: &Choice| {
                    let mut k: Vec<(&str, &str)> =
                        o.changes.iter().map(|c| (c.key, c.name)).collect();
                    k.sort_unstable();
                    k
                };
                let first = keys(&t.options[0]);
                for o in t.options {
                    assert_eq!(keys(o), first, "{}/{}", t.id, o.id);
                }
                if let Some(d) = t.default_option {
                    assert!(
                        t.options.iter().any(|o| o.id == d),
                        "{}: 기본 선택지 {d}",
                        t.id
                    );
                }
            } else {
                assert!(t.default_option.is_none(), "{}", t.id);
            }
        }
    }

    #[test]
    fn no_tweak_touches_policies_or_security() {
        for t in TWEAKS {
            assert!(
                !t.settings_path.is_empty(),
                "{}: 설정 앱 위치를 적어야 한다",
                t.id
            );
            for c in t.all_changes() {
                let key = c.key.to_lowercase();
                assert!(
                    !key.starts_with('\\') && !key.contains(".."),
                    "{}: {}",
                    t.id,
                    c.key
                );
                for bad in FORBIDDEN {
                    assert!(!key.contains(bad), "{}: 금지된 경로 {}", t.id, c.key);
                }
                // 테스트 앱 키는 테스트 항목 전용
                assert!(!key.contains("robin test labs"), "{}", t.id);
            }
        }
    }

    #[test]
    fn test_items_only_touch_the_fixture_key() {
        for t in [&TEST_TWEAK, &TEST_CHOICE] {
            for c in t.all_changes() {
                assert!(c.key.starts_with(r"Software\Robin Test Labs\"), "{}", c.key);
            }
        }
        assert_eq!(find("test.fixture").is_some(), cfg!(debug_assertions));
        assert_eq!(find("test.choice").is_some(), cfg!(debug_assertions));
    }
}
