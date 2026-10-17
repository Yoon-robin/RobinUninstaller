//! 잔여물 후보와 근거, 점수. 점수 계산은 순수 로직.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeftoverKind {
    Folder,
    File,
    RegistryKey,
    RegistryValue,
}

/// 왜 이 항목을 찾았는지. 화면에 그대로 보여 준다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Evidence {
    /// 등록된 설치 폴더 그 자체
    InstallLocation,
    /// 값이나 경로가 설치 폴더 안을 가리킴 (Run 값, App Paths, 서비스 실행 파일)
    PointsIntoInstallLocation,
    /// 제거된 프로그램의 등록 정보(Uninstall 키)가 남아 있음
    UninstallEntry,
    /// MSI 제품 코드로 등록된 키
    MsiProductCode,
    /// `게시자\제품` 구조로 이름이 일치
    PublisherAndProduct,
    /// 게시자 폴더 아래, 이름 핵심 단어만 일치 (다른 버전·에디션일 수 있음)
    PublisherAndPartialName,
    /// 폴더·키 이름이 프로그램 이름과 정확히 같음
    ExactName,
    /// 폴더·키 이름에 프로그램 이름의 핵심 단어가 모두 들어 있음
    PartialName,
    /// 게시자 폴더인데 안에 다른 제품이 남아 있지 않음
    EmptyPublisherFolder,
    /// 설치 추적 기록에서 이 프로그램을 설치할 때 생긴 것으로 확인됨
    TrackedInstall,
    /// 다른 설치된 프로그램도 쓰는 것으로 보임 (감점)
    #[serde(rename_all = "camelCase")]
    SharedWithOtherApp { app_name: String },
    /// 사용자가 만든 문서가 들어 있음 (감점·경고)
    #[serde(rename_all = "camelCase")]
    ContainsUserDocuments { count: usize },
    /// 바탕 화면·문서·다운로드처럼 사용자가 직접 파일을 두는 폴더 안 (감점·경고, 자동 선택 안 함)
    InUserFolder,
}

impl Evidence {
    pub fn weight(&self) -> i32 {
        match self {
            Evidence::UninstallEntry => 95,
            Evidence::TrackedInstall => 90,
            Evidence::MsiProductCode => 90,
            Evidence::InstallLocation => 85,
            Evidence::PointsIntoInstallLocation => 80,
            Evidence::PublisherAndProduct => 80,
            Evidence::PublisherAndPartialName => 55,
            Evidence::ExactName => 55,
            Evidence::PartialName => 30,
            Evidence::EmptyPublisherFolder => 50,
            Evidence::SharedWithOtherApp { .. } => -90,
            Evidence::ContainsUserDocuments { .. } => -40,
            Evidence::InUserFolder => -60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Confidence {
    /// 화면에 보여 주지 않음
    Low,
    /// 근거는 있지만 사용자가 확인해야 함 (기본 선택 안 함)
    Review,
    /// 강한 근거 (기본 선택)
    Strong,
}

pub const STRONG_THRESHOLD: i32 = 80;
pub const REVIEW_THRESHOLD: i32 = 30;

/// 점수: 가장 강한 긍정 근거 하나 + 나머지 긍정 근거의 절반(최대 100) + 감점 전부.
/// 약한 근거를 여러 개 모아 "확실"이 되지 않도록 최강 근거 위주로 계산한다.
pub fn score(evidence: &[Evidence]) -> i32 {
    let mut positives: Vec<i32> = evidence
        .iter()
        .map(Evidence::weight)
        .filter(|w| *w > 0)
        .collect();
    positives.sort_unstable_by(|a, b| b.cmp(a));
    let best = positives.first().copied().unwrap_or(0);
    let rest: i32 = positives.iter().skip(1).map(|w| w / 2).sum();
    let penalties: i32 = evidence
        .iter()
        .map(Evidence::weight)
        .filter(|w| *w < 0)
        .sum();
    (best + rest).min(100) + penalties
}

/// 다른 프로그램과 공유 중이면 점수와 상관없이 "확실"로 올리지 않는다.
pub fn classify(evidence: &[Evidence]) -> Confidence {
    let s = score(evidence);
    let shared = evidence
        .iter()
        .any(|e| matches!(e, Evidence::SharedWithOtherApp { .. }));
    if s >= STRONG_THRESHOLD && !shared {
        Confidence::Strong
    } else if s >= REVIEW_THRESHOLD {
        Confidence::Review
    } else {
        Confidence::Low
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    /// 화면·삭제용 고유 키 (경로 또는 레지스트리 전체 경로, 값이면 `키\\값이름`)
    pub id: String,
    pub kind: LeftoverKind,
    /// 사람이 읽을 위치 (예: `HKCU\Software\Vendor\Product`, `C:\Users\...\AppData\Roaming\Vendor`)
    pub location: String,
    /// 레지스트리 값이면 값 이름
    pub value_name: Option<String>,
    /// 폴더·파일 크기 (바이트). 레지스트리는 None
    pub size_bytes: Option<u64>,
    pub evidence: Vec<Evidence>,
    pub score: i32,
    pub confidence: Confidence,
    /// 지우기 전에 특별히 확인할 점 (서비스라 먼저 중지해야 함 등)
    pub note: Option<String>,
}

impl Leftover {
    pub fn new(
        kind: LeftoverKind,
        location: String,
        value_name: Option<String>,
        evidence: Vec<Evidence>,
    ) -> Self {
        let id = match &value_name {
            Some(v) => format!("{location}\\\\{v}"),
            None => location.clone(),
        };
        let score = score(&evidence);
        let confidence = classify(&evidence);
        Leftover {
            id,
            kind,
            location,
            value_name,
            size_bytes: None,
            evidence,
            score,
            confidence,
            note: None,
        }
    }

    /// 근거를 더하고 점수·분류를 다시 계산한다 (같은 항목을 다른 방법으로도 찾았을 때).
    /// 같은 종류의 감점 근거(다른 프로그램 공유, 문서 포함)는 하나만 둔다.
    pub fn add_evidence(&mut self, evidence: Evidence) {
        let duplicate = self.evidence.iter().any(|e| {
            e == &evidence
                || matches!(
                    (e, &evidence),
                    (
                        Evidence::SharedWithOtherApp { .. },
                        Evidence::SharedWithOtherApp { .. }
                    ) | (
                        Evidence::ContainsUserDocuments { .. },
                        Evidence::ContainsUserDocuments { .. }
                    )
                )
        });
        if !duplicate {
            self.evidence.push(evidence);
            self.score = score(&self.evidence);
            self.confidence = classify(&self.evidence);
        }
    }

    /// 위험을 알리는 근거 (상위 폴더로 올려야 하는 것)
    pub fn risk_evidence(&self) -> impl Iterator<Item = &Evidence> {
        self.evidence.iter().filter(|e| {
            matches!(
                e,
                Evidence::SharedWithOtherApp { .. }
                    | Evidence::ContainsUserDocuments { .. }
                    | Evidence::InUserFolder
            )
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub app_name: String,
    /// 점수 순으로 정렬된 후보 (Low는 빠짐)
    pub items: Vec<Leftover>,
    /// 보호 규칙 때문에 후보에서 뺀 항목 수
    pub protected_skipped: usize,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_needs_a_strong_signal() {
        assert_eq!(classify(&[Evidence::InstallLocation]), Confidence::Strong);
        assert_eq!(
            classify(&[Evidence::PublisherAndProduct]),
            Confidence::Strong
        );
        assert_eq!(classify(&[Evidence::ExactName]), Confidence::Review);
        // 약한 근거를 여러 개 모아도 확실이 되지 않는다
        assert_eq!(
            classify(&[Evidence::ExactName, Evidence::PartialName]),
            Confidence::Review
        );
        assert_eq!(classify(&[Evidence::PartialName]), Confidence::Review);
    }

    #[test]
    fn shared_or_documents_pull_it_down() {
        let shared = Evidence::SharedWithOtherApp {
            app_name: "Other".into(),
        };
        assert_eq!(
            classify(&[Evidence::InstallLocation, shared.clone()]),
            Confidence::Low
        );
        assert_eq!(classify(&[Evidence::ExactName, shared]), Confidence::Low);
        let docs = Evidence::ContainsUserDocuments { count: 3 };
        assert_eq!(
            classify(&[Evidence::PublisherAndProduct, docs]),
            Confidence::Review
        );
    }

    #[test]
    fn score_is_capped() {
        assert_eq!(
            score(&[
                Evidence::UninstallEntry,
                Evidence::InstallLocation,
                Evidence::ExactName
            ]),
            100
        );
    }
}
