//! 프로그램 아이콘을 PNG data URL로 추출한다. 읽기 전용.

use base64::Engine;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits, HGDIOBJ,
    ReleaseDC,
};
use windows::Win32::UI::Shell::SHDefExtractIconW;
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use windows::core::HSTRING;

use crate::command::{expand_env, icon_location};

// 카드 보기와 고해상도 화면에서도 선명하도록 64px로 뽑는다.
const SIZE: u32 = 64;

/// `DisplayIcon` 형식(`path,index`) 값에서 아이콘을 뽑아 `data:image/png;base64,...`로 돌려준다.
pub fn extract_data_url(display_icon: &str) -> Option<String> {
    let expanded = expand_env(display_icon, |name| std::env::var(name).ok());
    let (path, index) = icon_location(&expanded)?;
    if !std::path::Path::new(&path).is_file() {
        return None;
    }

    let rgba = unsafe { extract_rgba(&path, index)? };
    let png = encode_png(&rgba)?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

unsafe fn extract_rgba(path: &str, index: i32) -> Option<Vec<u8>> {
    let mut large = HICON::default();
    unsafe {
        SHDefExtractIconW(&HSTRING::from(path), index, 0, Some(&mut large), None, SIZE)
            .ok()
            .ok()?;
    }
    if large.is_invalid() {
        return None;
    }

    let result = unsafe { icon_to_rgba(large) };
    unsafe {
        let _ = DestroyIcon(large);
    }
    result
}

unsafe fn icon_to_rgba(icon: HICON) -> Option<Vec<u8>> {
    let mut info = ICONINFO::default();
    unsafe { GetIconInfo(icon, &mut info).ok()? };

    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SIZE as i32,
            biHeight: -(SIZE as i32), // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; (SIZE * SIZE * 4) as usize];

    let lines = unsafe {
        let dc = GetDC(None);
        let lines = GetDIBits(
            dc,
            info.hbmColor,
            0,
            SIZE,
            Some(bgra.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, dc);
        let _ = DeleteObject(HGDIOBJ(info.hbmColor.0));
        let _ = DeleteObject(HGDIOBJ(info.hbmMask.0));
        lines
    };
    if lines == 0 {
        return None;
    }

    // 알파 채널이 없는 옛날 아이콘은 전부 0이라 불투명으로 바꿔 준다.
    let has_alpha = bgra.as_chunks::<4>().0.iter().any(|p| p[3] != 0);
    for px in bgra.as_chunks_mut::<4>().0 {
        px.swap(0, 2); // BGRA → RGBA
        if !has_alpha {
            px[3] = 255;
        }
    }
    Some(bgra)
}

fn encode_png(rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, SIZE, SIZE);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    writer.finish().ok()?;
    Some(out)
}
