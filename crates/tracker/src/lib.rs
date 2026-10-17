//! 설치 추적 (Phase 6, 1차: 스냅샷 비교).
//!
//! 설치 전에 스냅샷을 찍고, 사용자가 설치를 마치면 다시 찍어 새로 생긴 파일·폴더·레지스트리를 찾는다.
//! 사용자가 그 프로그램 것이 아닌 항목을 뺀 뒤 기록으로 저장하면, 나중에 그 프로그램을 지울 때
//! 잔여물 스캐너가 "설치 때 생긴 것"이라는 가장 강한 근거로 쓴다.
//! 이 크레이트는 읽기 전용이다. 실제 정리는 항상 `crates/recovery`의 백업·격리를 거친다.
//! 설치 중 다른 프로그램이 바꾼 것이 섞일 수 있어서, 늘 바뀌는 위치는 빼고(`model::is_noise`) 사용자가 확인한다.

pub mod model;
#[cfg(windows)]
pub mod snapshot;
pub mod store;

pub use model::{Change, ChangeKind, Snapshot, Trace, TraceDraft, diff};
pub use store::TraceStore;
