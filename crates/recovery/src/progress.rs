//! 정리·되돌리기·백업 지우기의 진행 상황.
//!
//! 이 작업들은 한 스레드에서 차례로 돈다. 함수마다 인자를 늘리는 대신, 부르는 쪽이 [`with`]로 진행 상황을
//! 이 스레드에 걸어 두면 안쪽 코드(복사·삭제)가 [`report`]로 숫자를 올린다. 걸어 둔 것이 없으면 아무 일도 하지 않는다.
//! 다른 드라이브로 옮길 때는 복사라 몇 분 걸릴 수 있어서, 화면이 멈춘 것처럼 보이지 않게 하려는 것이다.

use std::cell::RefCell;
use std::sync::Arc;

pub use robin_core::progress::{Progress, ProgressSnapshot};

thread_local! {
    static CURRENT: RefCell<Option<Arc<Progress>>> = const { RefCell::new(None) };
}

/// `f`가 도는 동안 이 스레드의 진행 상황을 `progress`로 둔다.
pub fn with<R>(progress: Arc<Progress>, f: impl FnOnce() -> R) -> R {
    let previous = CURRENT.with(|c| c.replace(Some(progress)));
    let result = f();
    CURRENT.with(|c| *c.borrow_mut() = previous);
    result
}

/// 걸어 둔 진행 상황이 있으면 올린다.
pub(crate) fn report(f: impl FnOnce(&Progress)) {
    CURRENT.with(|c| {
        if let Some(p) = c.borrow().as_ref() {
            f(p)
        }
    });
}

/// 바깥(앱 쪽 반복문)에서 전체 수를 알릴 때
pub fn report_total(total: u64) {
    report(|p| p.set_total(total));
}

/// 바깥(앱 쪽 반복문)에서 하나를 끝냈다고 알릴 때
pub fn report_unit_done() {
    report(|p| p.unit_done());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_only_inside_with() {
        let p = Arc::new(Progress::new());
        report(|p| p.add_file(1)); // 걸어 둔 게 없으면 무시
        with(p.clone(), || {
            report(|p| p.add_file(10));
            report(|p| p.unit_done());
        });
        report(|p| p.add_file(100)); // 끝난 뒤에도 무시
        let s = p.snapshot();
        assert_eq!((s.files, s.bytes, s.done), (1, 10, 1));
    }
}
