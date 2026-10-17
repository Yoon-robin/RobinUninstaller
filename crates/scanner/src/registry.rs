//! 레지스트리 후보 찾기 (읽기 전용).
//!
//! - `Software\게시자\제품`, `Software\제품` 키 (HKCU, HKLM 64/32비트)
//! - 남아 있는 Uninstall 등록 정보, MSI 제품 등록(압축 GUID)
//! - 설치 폴더를 가리키는 Run 값, App Paths, 서비스

use winreg::RegKey;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};

use crate::context::ScanContext;
use crate::model::{Evidence, Leftover, LeftoverKind};
use crate::msi_guid;
use crate::names::{NameMatch, match_name, matches_publisher};
use crate::protect::{is_within, protected_key_reason};

const UNINSTALL: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

#[derive(Clone, Copy)]
struct Hive {
    root: winreg::HKEY,
    view: u32,
    /// 화면 표시용 접두어
    label: &'static str,
    /// 32비트 뷰는 실제 경로에 WOW6432Node가 들어간다 (표시용)
    wow: bool,
}

const HIVES: [Hive; 3] = [
    Hive {
        root: HKEY_CURRENT_USER,
        view: 0,
        label: "HKCU",
        wow: false,
    },
    Hive {
        root: HKEY_LOCAL_MACHINE,
        view: KEY_WOW64_64KEY,
        label: "HKLM",
        wow: false,
    },
    Hive {
        root: HKEY_LOCAL_MACHINE,
        view: KEY_WOW64_32KEY,
        label: "HKLM",
        wow: true,
    },
];

impl Hive {
    fn open(&self, path: &str) -> Option<RegKey> {
        RegKey::predef(self.root)
            .open_subkey_with_flags(path, KEY_READ | self.view)
            .ok()
    }

    /// `SOFTWARE\Vendor` → `HKLM\SOFTWARE\WOW6432Node\Vendor` (32비트 뷰)
    fn display(&self, path: &str) -> String {
        let shown = if self.wow && path.len() >= 9 && path[..9].eq_ignore_ascii_case("software\\") {
            format!("SOFTWARE\\WOW6432Node\\{}", &path[9..])
        } else {
            path.to_string()
        };
        format!("{}\\{}", self.label, shown)
    }
}

pub fn scan(ctx: &ScanContext, out: &mut Vec<Leftover>, protected: &mut usize) {
    uninstall_entry(ctx, out);
    msi_products(ctx, out);
    software_keys(ctx, out, protected);
    if let Some(dir) = ctx.install_location.clone() {
        run_values(ctx, &dir, out);
        app_paths(ctx, &dir, out);
        services(ctx, &dir, out);
    }
}

/// 제거된 프로그램의 Uninstall 키가 아직 있으면 (언인스톨러가 등록 정보를 안 지운 경우).
fn uninstall_entry(ctx: &ScanContext, out: &mut Vec<Leftover>) {
    let Some((scope, key_name)) = ctx.app_id.split_once(':') else {
        return;
    };
    let hive = match scope {
        "hkcu" => HIVES[0],
        "hklm64" => HIVES[1],
        "hklm32" => HIVES[2],
        _ => return,
    };
    let Some(root) = hive.open(UNINSTALL) else {
        return;
    };
    if let Some(actual) = root
        .enum_keys()
        .flatten()
        .find(|k| k.eq_ignore_ascii_case(key_name))
    {
        let path = format!("{UNINSTALL}\\{actual}");
        out.push(Leftover::new(
            LeftoverKind::RegistryKey,
            hive.display(&path),
            None,
            vec![Evidence::UninstallEntry],
        ));
    }
}

fn msi_products(ctx: &ScanContext, out: &mut Vec<Leftover>) {
    let Some(packed) = ctx.product_code.as_deref().and_then(msi_guid::pack) else {
        return;
    };
    let places = [
        (
            HIVES[1],
            format!(r"SOFTWARE\Classes\Installer\Products\{packed}"),
        ),
        (
            HIVES[1],
            format!(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\UserData\S-1-5-18\Products\{packed}"
            ),
        ),
        (
            HIVES[0],
            format!(r"Software\Microsoft\Installer\Products\{packed}"),
        ),
    ];
    for (hive, path) in places {
        if hive.open(&path).is_some() {
            let mut item = Leftover::new(
                LeftoverKind::RegistryKey,
                hive.display(&path),
                None,
                vec![Evidence::MsiProductCode],
            );
            item.note = Some(
                "Windows Installer 등록 정보예요. 제거가 실패한 MSI 제품을 정리할 때만 지우세요."
                    .into(),
            );
            out.push(item);
        }
    }
}

/// `Software` 아래에서 게시자·제품 이름으로 찾는다.
fn software_keys(ctx: &ScanContext, out: &mut Vec<Leftover>, protected: &mut usize) {
    for hive in HIVES {
        let Some(software) = hive.open("SOFTWARE") else {
            continue;
        };
        for name in software.enum_keys().flatten() {
            let path = format!("SOFTWARE\\{name}");
            if protected_key_reason(&path, true).is_some() {
                continue; // Microsoft·Classes 등은 이름으로 들어가지 않는다
            }

            if let Some(publisher) = &ctx.publisher
                && matches_publisher(&name, publisher)
            {
                publisher_key(ctx, hive, &software, &name, out, protected);
                continue;
            }

            let ev = match match_name(&name, &ctx.app_name, ctx.publisher.as_deref()) {
                NameMatch::Exact => Evidence::ExactName,
                NameMatch::AllSignificant => Evidence::PartialName,
                NameMatch::None => continue,
            };
            push_key(ctx, hive, &path, &name, vec![ev], out, protected);
        }
    }
}

fn publisher_key(
    ctx: &ScanContext,
    hive: Hive,
    software: &RegKey,
    publisher_name: &str,
    out: &mut Vec<Leftover>,
    protected: &mut usize,
) {
    let Ok(key) = software.open_subkey_with_flags(publisher_name, KEY_READ | hive.view) else {
        return;
    };
    let mut matched = Vec::new();
    let mut others = 0;
    for sub in key.enum_keys().flatten() {
        match match_name(&sub, &ctx.app_name, ctx.publisher.as_deref()) {
            NameMatch::None => others += 1,
            m => matched.push((sub, m)),
        }
    }
    let has_values = key.enum_values().flatten().next().is_some();

    let all_exact = matched.iter().all(|(_, m)| *m == NameMatch::Exact);
    if !matched.is_empty() && others == 0 && !has_values && all_exact {
        let path = format!("SOFTWARE\\{publisher_name}");
        push_key(
            ctx,
            hive,
            &path,
            publisher_name,
            vec![
                Evidence::PublisherAndProduct,
                Evidence::EmptyPublisherFolder,
            ],
            out,
            protected,
        );
    } else {
        for (sub, m) in matched {
            let path = format!("SOFTWARE\\{publisher_name}\\{sub}");
            push_key(
                ctx,
                hive,
                &path,
                &sub,
                vec![crate::files::publisher_evidence(m)],
                out,
                protected,
            );
        }
    }
}

fn push_key(
    ctx: &ScanContext,
    hive: Hive,
    path: &str,
    leaf: &str,
    mut evidence: Vec<Evidence>,
    out: &mut Vec<Leftover>,
    protected: &mut usize,
) {
    if protected_key_reason(path, true).is_some() || ctx.is_self(path) {
        *protected += 1;
        return;
    }
    if let Some(other) = ctx.other_app_named(leaf) {
        evidence.push(Evidence::SharedWithOtherApp { app_name: other });
    }
    out.push(Leftover::new(
        LeftoverKind::RegistryKey,
        hive.display(path),
        None,
        evidence,
    ));
}

/// 값 데이터 안의 실행 파일 경로
fn target_of(data: &str) -> String {
    let expanded = robin_discovery::command::expand_env(data, |n| std::env::var(n).ok());
    robin_discovery::command::executable_path(&expanded).unwrap_or(expanded)
}

/// 값 데이터 안의 실행 파일 경로가 설치 폴더 안인지.
fn points_into(data: &str, dir: &str) -> bool {
    is_within(&target_of(data), dir)
}

/// "설치 폴더를 가리킴" 근거. 실행 파일이 다른 프로그램의 설치 폴더 안이면(같은 폴더를 함께 쓰는 제품군 등) 감점한다.
fn pointing_evidence(ctx: &ScanContext, data: &str) -> Vec<Evidence> {
    let mut evidence = vec![Evidence::PointsIntoInstallLocation];
    if let Some(other) = ctx.other_app_owning(&target_of(data)) {
        evidence.push(Evidence::SharedWithOtherApp { app_name: other });
    }
    evidence
}

fn run_values(ctx: &ScanContext, dir: &str, out: &mut Vec<Leftover>) {
    for hive in HIVES {
        for sub in ["Run", "RunOnce"] {
            let path = format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\{sub}");
            let Some(key) = hive.open(&path) else {
                continue;
            };
            for (name, value) in key.enum_values().flatten() {
                let data = value.to_string();
                if points_into(&data, dir) {
                    let mut item = Leftover::new(
                        LeftoverKind::RegistryValue,
                        hive.display(&path),
                        Some(name),
                        pointing_evidence(ctx, &data),
                    );
                    item.note = Some(format!("{} 시작 시 자동 실행 항목이에요.", ctx.app_name));
                    out.push(item);
                }
            }
        }
    }
}

fn app_paths(ctx: &ScanContext, dir: &str, out: &mut Vec<Leftover>) {
    let path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";
    for hive in HIVES {
        let Some(key) = hive.open(path) else { continue };
        for exe in key.enum_keys().flatten() {
            let Ok(sub) = key.open_subkey_with_flags(&exe, KEY_READ | hive.view) else {
                continue;
            };
            let target: String = sub.get_value("").unwrap_or_default();
            if !target.is_empty() && points_into(&target, dir) {
                out.push(Leftover::new(
                    LeftoverKind::RegistryKey,
                    hive.display(&format!("{path}\\{exe}")),
                    None,
                    pointing_evidence(ctx, &target),
                ));
            }
        }
    }
}

fn services(ctx: &ScanContext, dir: &str, out: &mut Vec<Leftover>) {
    let path = r"SYSTEM\CurrentControlSet\Services";
    let Some(key) = HIVES[1].open(path) else {
        return;
    };
    for name in key.enum_keys().flatten() {
        let Ok(sub) = key.open_subkey_with_flags(&name, KEY_READ) else {
            continue;
        };
        let image: String = sub.get_value("ImagePath").unwrap_or_default();
        if !image.is_empty() && points_into(&image, dir) {
            let mut item = Leftover::new(
                LeftoverKind::RegistryKey,
                format!("HKLM\\{path}\\{name}"),
                None,
                pointing_evidence(ctx, &image),
            );
            item.note =
                Some("Windows 서비스예요. 지우기 전에 서비스를 중지하고 제거해야 해요.".into());
            out.push(item);
        }
    }
}
