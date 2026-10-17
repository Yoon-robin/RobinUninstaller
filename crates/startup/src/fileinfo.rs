//! 실행 파일의 버전 정보에서 회사 이름과 설명을 읽는다 (읽기 전용).

use std::ffi::c_void;

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::core::HSTRING;

#[derive(Debug, Default)]
pub struct FileInfo {
    pub company: Option<String>,
    pub description: Option<String>,
}

pub fn read(path: &str) -> FileInfo {
    let wide = HSTRING::from(path);
    let size = unsafe { GetFileVersionInfoSizeW(&wide, None) };
    if size == 0 {
        return FileInfo::default();
    }
    let mut buf = vec![0u8; size as usize];
    if unsafe { GetFileVersionInfoW(&wide, None, size, buf.as_mut_ptr().cast()) }.is_err() {
        return FileInfo::default();
    }

    // 파일에 기록된 언어·코드 페이지 순서대로 찾고, 없으면 영어(미국)·유니코드로 찾는다.
    let mut langs: Vec<String> = translations(&buf)
        .into_iter()
        .map(|(lang, cp)| format!("{lang:04x}{cp:04x}"))
        .collect();
    langs.push("040904b0".into());
    langs.push("040904e4".into());

    let find = |field: &str| {
        langs
            .iter()
            .find_map(|lang| query_string(&buf, &format!(r"\StringFileInfo\{lang}\{field}")))
    };
    FileInfo {
        company: find("CompanyName"),
        description: find("FileDescription"),
    }
}

fn query(buf: &[u8], sub_block: &str) -> Option<(*const c_void, u32)> {
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let mut len = 0u32;
    let ok = unsafe {
        VerQueryValueW(
            buf.as_ptr().cast(),
            &HSTRING::from(sub_block),
            &mut ptr,
            &mut len,
        )
    }
    .as_bool();
    (ok && !ptr.is_null() && len > 0).then_some((ptr.cast_const(), len))
}

fn translations(buf: &[u8]) -> Vec<(u16, u16)> {
    let Some((ptr, len)) = query(buf, r"\VarFileInfo\Translation") else {
        return Vec::new();
    };
    // (언어, 코드 페이지) u16 쌍의 배열. 포인터는 buf 안을 가리킨다.
    let words = unsafe { std::slice::from_raw_parts(ptr.cast::<u16>(), len as usize / 2) };
    words
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[lang, cp]| (*lang, *cp))
        .collect()
}

fn query_string(buf: &[u8], sub_block: &str) -> Option<String> {
    let (ptr, len) = query(buf, sub_block)?;
    // 문자열 값의 len은 문자 수 (끝의 0 포함)
    let chars = unsafe { std::slice::from_raw_parts(ptr.cast::<u16>(), len as usize) };
    let end = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
    let text = String::from_utf16_lossy(&chars[..end]).trim().to_string();
    (!text.is_empty()).then_some(text)
}
