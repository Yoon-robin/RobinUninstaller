//! 바로 가기(.lnk) 대상과 실행 파일의 회사 이름 읽기. 읽기 전용.

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize, IPersistFile, STGM_READ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::core::{HSTRING, Interface};

pub struct Link {
    pub target: String,
    pub args: String,
}

fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end]).trim().to_string()
}

/// 바로 가기의 대상 경로와 인자. 대상이 파일 경로가 아니면(MSI 광고 바로 가기 등) None.
pub fn resolve_link(path: &str) -> Option<Link> {
    unsafe {
        // 이미 다른 방식으로 초기화된 스레드면 실패하지만, 그래도 COM은 쓸 수 있다
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| {
            let link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
            link.cast::<IPersistFile>()
                .ok()?
                .Load(&HSTRING::from(path), STGM_READ)
                .ok()?;
            let mut target = [0u16; 1024];
            link.GetPath(&mut target, std::ptr::null_mut(), 0).ok()?;
            let mut args = [0u16; 2048];
            let _ = link.GetArguments(&mut args);
            let target = from_wide(&target);
            (!target.is_empty()).then(|| Link {
                target,
                args: from_wide(&args),
            })
        })();
        if initialized {
            CoUninitialize();
        }
        result
    }
}

/// 실행 파일 버전 정보의 `CompanyName`.
pub fn company_name(path: &str) -> Option<String> {
    let file = HSTRING::from(path);
    unsafe {
        let size = GetFileVersionInfoSizeW(&file, None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(&file, None, size, data.as_mut_ptr().cast()).ok()?;

        let query = |sub: &str| -> Option<(*const u16, usize)> {
            let mut ptr = std::ptr::null_mut();
            let mut len = 0u32;
            VerQueryValueW(
                data.as_ptr().cast(),
                &HSTRING::from(sub),
                &mut ptr,
                &mut len,
            )
            .as_bool()
            .then_some((ptr as *const u16, len as usize))
            .filter(|(p, l)| !p.is_null() && *l > 0)
        };

        // 파일에 적힌 언어·코드 페이지부터, 없으면 흔한 영어 조합
        let mut langs = Vec::new();
        if let Some((ptr, len)) = query(r"\VarFileInfo\Translation") {
            // len은 바이트 수. (언어, 코드 페이지) u16 쌍의 목록이다
            let pairs = std::slice::from_raw_parts(ptr, len / 2);
            langs.extend(
                pairs
                    .chunks_exact(2)
                    .map(|p| format!("{:04x}{:04x}", p[0], p[1])),
            );
        }
        langs.extend(["040904b0", "040904e4", "04090000"].map(String::from));

        langs.iter().find_map(|lang| {
            let (ptr, len) = query(&format!(r"\StringFileInfo\{lang}\CompanyName"))?;
            // 문자열 값의 len은 글자 수(끝의 NUL 포함)
            let name = from_wide(std::slice::from_raw_parts(ptr, len));
            (!name.is_empty()).then_some(name)
        })
    }
}
