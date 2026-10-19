//! 설정을 바꾼 뒤 탐색기·작업 표시줄에 알린다. 설정 앱이 값을 바꿀 때 보내는 것과 같은 알림이라,
//! 탐색기를 다시 시작하지 않아도 대부분 바로 반영된다 (열려 있는 탐색기 창은 F5가 필요할 수 있다).

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use windows::core::w;

use crate::model::Restart;

/// 응답하지 않는 창 때문에 오래 기다리지 않게 짧게 기다린다.
const TIMEOUT_MS: u32 = 500;

fn broadcast(area: Option<windows::core::PCWSTR>) {
    let lparam = LPARAM(area.map_or(0, |a| a.as_ptr() as isize));
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            lparam,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MS,
            None,
        );
    }
}

pub fn settings_changed(restart: Restart) {
    if restart == Restart::SignOut {
        // 다시 로그인해야 반영되는 설정이라 알려도 소용없다
        return;
    }
    broadcast(None);
    broadcast(Some(w!("TraySettings")));
    broadcast(Some(w!("ImmersiveColorSet")));
    if restart == Restart::Explorer {
        // 탐색기 보기(확장자·숨김 파일 등)를 새로 그리게 한다
        unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
    }
}
