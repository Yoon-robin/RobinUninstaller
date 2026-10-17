//! COM 초기화 도우미 (바로 가기 해석과 작업 스케줄러가 함께 쓴다).

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

/// 이 스레드에서 COM을 쓰는 동안만 초기화해 둔다.
pub(crate) struct Com(bool);

impl Com {
    pub(crate) fn init() -> Self {
        // S_OK, S_FALSE 둘 다 짝을 맞춰 해제해야 한다. 다른 방식으로 이미 초기화된 스레드면 그대로 쓴다.
        Com(unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok())
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}
