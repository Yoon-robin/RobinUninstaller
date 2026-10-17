//! Windows Installer(MSI) 조회. 읽기 전용.

use windows::Win32::System::ApplicationInstallationAndServicing::{
    INSTALLSTATE_ADVERTISED, INSTALLSTATE_DEFAULT, INSTALLSTATE_LOCAL, INSTALLSTATE_SOURCE,
    MSIINSTALLCONTEXT, MSIINSTALLCONTEXT_MACHINE, MSIINSTALLCONTEXT_USERUNMANAGED,
    MsiGetProductInfoExW, MsiQueryProductStateW,
};
use windows::core::{HSTRING, PCWSTR, PWSTR};

/// 제품이 Windows Installer에 등록돼 있는지 (광고만 된 제품도 제거할 수 있으므로 포함).
pub fn is_installed(product_code: &str) -> bool {
    let state = unsafe { MsiQueryProductStateW(&HSTRING::from(product_code)) };
    [
        INSTALLSTATE_DEFAULT,
        INSTALLSTATE_LOCAL,
        INSTALLSTATE_SOURCE,
        INSTALLSTATE_ADVERTISED,
    ]
    .contains(&state)
}

/// MSI가 기록한 설치 경로. 레지스트리 Uninstall 키에 InstallLocation이 없을 때 보강용.
pub fn install_location(product_code: &str, per_user: bool) -> Option<String> {
    let context = if per_user {
        MSIINSTALLCONTEXT_USERUNMANAGED
    } else {
        MSIINSTALLCONTEXT_MACHINE
    };
    product_info(product_code, context, "InstallLocation")
}

/// MSI가 등록한 제품 아이콘 파일 경로 (보통 `C:\Windows\Installer\{GUID}\*.ico`).
pub fn product_icon(product_code: &str, per_user: bool) -> Option<String> {
    let context = if per_user {
        MSIINSTALLCONTEXT_USERUNMANAGED
    } else {
        MSIINSTALLCONTEXT_MACHINE
    };
    product_info(product_code, context, "ProductIcon")
}

fn product_info(product_code: &str, context: MSIINSTALLCONTEXT, property: &str) -> Option<String> {
    let code = HSTRING::from(product_code);
    let prop = HSTRING::from(property);
    let mut len: u32 = 0;

    // 1차 호출로 길이를 얻고, 2차 호출로 값을 읽는다.
    let rc = unsafe {
        MsiGetProductInfoExW(&code, PCWSTR::null(), context, &prop, None, Some(&mut len))
    };
    if rc != 0 || len == 0 {
        return None;
    }

    let mut buf = vec![0u16; len as usize + 1];
    len += 1;
    let rc = unsafe {
        MsiGetProductInfoExW(
            &code,
            PCWSTR::null(),
            context,
            &prop,
            Some(PWSTR(buf.as_mut_ptr())),
            Some(&mut len),
        )
    };
    if rc != 0 {
        return None;
    }

    let value = String::from_utf16_lossy(&buf[..len as usize]);
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}
