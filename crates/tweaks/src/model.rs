//! 최적화 항목의 모양. 항목은 코드가 아니라 데이터(`catalog.rs`)로 정의한다.

use serde::{Deserialize, Serialize};

/// 쓸 값. 지금은 현재 사용자(HKCU) 아래의 DWORD·문자열만 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Dword(u32),
    Str(&'static str),
}

impl Value {
    pub fn raw(&self) -> RawValue {
        match self {
            Value::Dword(v) => RawValue {
                vtype: RawType::Dword,
                bytes: v.to_le_bytes().to_vec(),
            },
            Value::Str(s) => RawValue {
                vtype: RawType::Sz,
                bytes: s
                    .encode_utf16()
                    .chain([0])
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            },
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Dword(v) => format!("{v} (DWORD)"),
            Value::Str(s) => format!("\"{s}\" (문자열)"),
        }
    }
}

/// 레지스트리 값 하나를 바꾸는 일. `key`는 `HKEY_CURRENT_USER` 아래 경로.
#[derive(Debug, Clone, Copy)]
pub struct RegChange {
    pub key: &'static str,
    pub name: &'static str,
    pub value: Value,
}

/// 여러 값 중 하나를 고르는 항목의 선택지 (예: 작업 표시줄 왼쪽/가운데)
#[derive(Debug, Clone, Copy)]
pub struct Choice {
    pub id: &'static str,
    pub label: &'static str,
    pub changes: &'static [RegChange],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Group {
    /// 개인정보: 광고 ID, 진단 데이터 기반 맞춤 환경 등
    Privacy,
    /// 작업 표시줄
    Taskbar,
    /// 추천·팁·광고성 제안
    Suggestions,
    /// 편의: 파일 확장자, 고정 키 팝업 등
    Convenience,
    /// 게임
    Gaming,
    /// 개발용 테스트 항목 (디버그 빌드에만 보인다)
    Test,
}

/// 바꾼 뒤 화면에 반영되려면 필요한 것
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Restart {
    /// 바로 반영
    None,
    /// 탐색기·작업 표시줄에 설정 변경 알림을 보내 반영한다. 열려 있는 창은 새로 고쳐야(F5) 보일 수 있다
    Explorer,
    /// 로그아웃 후 다시 로그인해야 반영
    SignOut,
}

#[derive(Debug, Clone, Copy)]
pub struct Tweak {
    /// 화면과 기록에서 쓰는 고유 ID (소문자·숫자·점·하이픈)
    pub id: &'static str,
    pub group: Group,
    pub title: &'static str,
    /// 무엇이 달라지는지
    pub description: &'static str,
    /// 왜 바꾸면 좋은지, 무엇을 잃는지
    pub why: &'static str,
    /// Windows 설정 앱에서 같은 설정을 찾는 위치 (직접 되돌리고 싶을 때)
    pub settings_path: &'static str,
    pub restart: Restart,
    /// 이 빌드 이상에서만 보인다 (Windows 11 = 22000)
    pub min_build: u32,
    /// 켜기/끄기 항목에서 켤 때 쓰는 값. 선택지 항목이면 비어 있다.
    pub changes: &'static [RegChange],
    /// 여러 값 중 고르는 항목의 선택지. 켜기/끄기 항목이면 비어 있다.
    pub options: &'static [Choice],
    /// 값이 하나도 없을 때 Windows가 쓰는 선택지 (예: 작업 표시줄 가운데)
    pub default_option: Option<&'static str>,
}

impl Tweak {
    pub fn is_choice(&self) -> bool {
        !self.options.is_empty()
    }

    /// 이 항목이 건드릴 수 있는 모든 값 (선택지 항목이면 모든 선택지의 값을 합친 것, 중복 없이)
    pub fn all_changes(&self) -> Vec<&'static RegChange> {
        let mut out: Vec<&'static RegChange> = Vec::new();
        let all = self
            .changes
            .iter()
            .chain(self.options.iter().flat_map(|o| o.changes.iter()));
        for c in all {
            if !out.iter().any(|x| x.key == c.key && x.name == c.name) {
                out.push(c);
            }
        }
        out
    }

    /// 쓸 값: 켜기/끄기 항목은 `option` 없이, 선택지 항목은 선택지 ID로.
    pub fn targets(&self, option: Option<&str>) -> Option<&'static [RegChange]> {
        match (self.is_choice(), option) {
            (false, None) => Some(self.changes),
            (true, Some(id)) => self.options.iter().find(|o| o.id == id).map(|o| o.changes),
            _ => None,
        }
    }
}

/// 레지스트리 원본 값 (종류 + 바이트). 되돌릴 때 그대로 쓴다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawValue {
    pub vtype: RawType,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RawType {
    None,
    Sz,
    ExpandSz,
    Binary,
    Dword,
    MultiSz,
    Qword,
}

impl RawValue {
    /// 같은 값인지. 문자열은 끝의 null 문자 차이를 무시한다.
    pub fn same_as(&self, other: &RawValue) -> bool {
        let text = |t: RawType| matches!(t, RawType::Sz | RawType::ExpandSz);
        if text(self.vtype) && text(other.vtype) {
            let trim = |b: &[u8]| {
                let mut v = b.to_vec();
                while v.len() >= 2 && v[v.len() - 2..] == [0, 0] {
                    v.truncate(v.len() - 2);
                }
                v
            };
            return self.vtype == other.vtype && trim(&self.bytes) == trim(&other.bytes);
        }
        self == other
    }
}

/// 항목의 현재 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    /// 모든 값이 바꿀 값과 같음
    Applied,
    /// 하나도 같지 않음 (기본값 또는 다른 값)
    NotApplied,
    /// 일부만 같음
    Partial,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_values_ignore_trailing_nulls() {
        let a = Value::Str("506").raw();
        let mut b = a.clone();
        b.bytes.extend([0, 0]);
        assert!(a.same_as(&b));
        assert!(!a.same_as(&Value::Str("510").raw()));
        assert!(!a.same_as(&Value::Dword(506).raw()));
        assert_eq!(Value::Dword(1).raw().bytes, vec![1, 0, 0, 0]);
    }
}
