//! 최적화 적용·되돌리기를 테스트 항목(`test.fixture`, `HKCU\Software\Robin Test Labs\Tweaks`)으로만 시험한다.
//! 실제 Windows 설정은 건드리지 않는다. 디버그 빌드에서 실행한다:
//!
//!   cargo run -p robin-tweaks --example tweaks_fixture

#[cfg(windows)]
fn main() {
    use robin_tweaks::{History, State, apply, catalog, revert, selected, state};
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, RegType};

    const KEY: &str = r"Software\Robin Test Labs\Tweaks";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let t = catalog::find("test.fixture").expect("디버그 빌드에서 실행하세요");
    let dir = std::env::temp_dir().join(format!("robin-tweaks-fixture-{}", std::process::id()));
    let history = History::new(&dir);

    let mut ok = true;
    let mut check = |name: &str, pass: bool| {
        println!("{} {name}", if pass { "✔" } else { "✘" });
        ok &= pass;
    };
    let clean = || {
        let _ = hkcu.delete_subkey_all(KEY);
        // 테스트 앱이 설치돼 있지 않아 비어 있으면 상위 키도 지운다 (하위 키가 있으면 실패하고 그대로 남는다)
        let _ = hkcu.delete_subkey(r"Software\Robin Test Labs");
    };
    let turbo = || {
        hkcu.open_subkey(KEY)
            .and_then(|k| k.get_value::<u32, _>("Turbo"))
            .ok()
    };
    let mode = || {
        hkcu.open_subkey(KEY)
            .and_then(|k| k.get_raw_value("Mode"))
            .ok()
            .map(|v| {
                (
                    v.vtype,
                    String::from_utf16_lossy(
                        &v.bytes
                            .chunks(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect::<Vec<_>>(),
                    )
                    .trim_end_matches('\0')
                    .to_string(),
                )
            })
    };

    // 1) 값이 없던 상태에서 적용 → 되돌리면 값과 만든 키까지 사라진다
    clean();
    check("처음엔 적용 안 됨", state(t) == State::NotApplied);
    check("적용", apply(&history, "test.fixture", None).is_ok());
    check(
        "적용됨으로 보임",
        state(t) == State::Applied && turbo() == Some(1),
    );
    let first = history.get("test.fixture");
    check(
        "원래 값(없음)이 기록됨",
        first.as_ref().is_some_and(|r| {
            r.originals
                .iter()
                .all(|o| o.previous.is_none() && !o.key_existed)
        }),
    );
    check(
        "다시 적용해도 처음 기록 유지",
        apply(&history, "test.fixture", None).is_ok() && history.get("test.fixture") == first,
    );
    check(
        "되돌리기",
        revert(&history, "test.fixture").is_ok_and(|o| o.left_as_is.is_empty()),
    );
    check("값과 만든 키가 사라짐", hkcu.open_subkey(KEY).is_err());
    check(
        "만든 상위 키도 사라짐",
        hkcu.open_subkey(r"Software\Robin Test Labs").is_err(),
    );
    check("기록도 정리됨", history.get("test.fixture").is_none());

    // 2) 원래 값이 있던 경우: 종류(REG_EXPAND_SZ)까지 그대로 돌아온다
    let (k, _) = hkcu.create_subkey(KEY).unwrap();
    k.set_value("Turbo", &5u32).unwrap();
    k.set_raw_value(
        "Mode",
        &winreg::RegValue {
            bytes: "loud\0".encode_utf16().flat_map(u16::to_le_bytes).collect(),
            vtype: RegType::REG_EXPAND_SZ,
        },
    )
    .unwrap();
    check(
        "원래 값 있는 상태에서 적용",
        apply(&history, "test.fixture", None).is_ok() && turbo() == Some(1),
    );
    check("되돌리기", revert(&history, "test.fixture").is_ok());
    check(
        "원래 값과 종류 그대로",
        turbo() == Some(5) && mode() == Some((RegType::REG_EXPAND_SZ, "loud".to_string())),
    );

    // 3) 적용한 뒤 다른 곳에서 바꾼 값은 덮어쓰지 않는다
    check("적용", apply(&history, "test.fixture", None).is_ok());
    hkcu.open_subkey_with_flags(KEY, winreg::enums::KEY_SET_VALUE)
        .unwrap()
        .set_value("Turbo", &9u32)
        .unwrap();
    check("일부만 적용됨으로 보임", state(t) == State::Partial);
    let outcome = revert(&history, "test.fixture");
    check(
        "다른 곳에서 바꾼 값은 그대로, 나머지는 되돌림",
        outcome.is_ok_and(|o| o.left_as_is.len() == 1 && o.left_as_is[0].ends_with("Turbo"))
            && turbo() == Some(9)
            && mode() == Some((RegType::REG_EXPAND_SZ, "loud".to_string())),
    );

    // 4) 선택지 항목: 값이 없으면 기본 선택지, 고르기 → 바꾸기 → 되돌리면 원래(값 없음)로
    clean();
    let c = catalog::find("test.choice").expect("디버그 빌드에서 실행하세요");
    let level = || {
        hkcu.open_subkey(KEY)
            .and_then(|k| k.get_value::<u32, _>("Level"))
            .ok()
    };
    check("값이 없으면 기본 선택지(낮음)", selected(c) == Some("low"));
    check(
        "높음 고르기",
        apply(&history, "test.choice", Some("high")).is_ok() && level() == Some(2),
    );
    check("높음으로 보임", selected(c) == Some("high"));
    let rec = history.get("test.choice");
    check(
        "원래 값(없음)과 고른 선택지 기록",
        rec.as_ref().is_some_and(|r| {
            r.option.as_deref() == Some("high") && r.originals[0].previous.is_none()
        }),
    );
    check(
        "낮음으로 바꾸기",
        apply(&history, "test.choice", Some("low")).is_ok() && level() == Some(1),
    );
    check(
        "바꿔도 원래 값은 처음 것, 선택지만 갱신",
        history.get("test.choice").is_some_and(|r| {
            r.option.as_deref() == Some("low") && r.originals[0].previous.is_none()
        }),
    );
    check(
        "모르는 선택지는 거부",
        apply(&history, "test.choice", Some("max")).is_err(),
    );
    check(
        "선택지 없이 적용은 거부",
        apply(&history, "test.choice", None).is_err(),
    );
    check(
        "켜기/끄기 항목에 선택지는 거부",
        apply(&history, "test.fixture", Some("low")).is_err(),
    );
    check(
        "되돌리기",
        revert(&history, "test.choice").is_ok_and(|o| o.left_as_is.is_empty()),
    );
    check(
        "원래대로(값과 만든 키 없음)",
        hkcu.open_subkey(KEY).is_err(),
    );

    // 5) 거부
    check(
        "모르는 항목은 거부",
        apply(&history, "privacy.unknown", None).is_err(),
    );
    check(
        "기록 없는 되돌리기는 거부",
        revert(&history, "test.fixture").is_err(),
    );

    clean();
    let _ = std::fs::remove_dir_all(&dir);
    println!("{}", if ok { "통과" } else { "실패" });
    if !ok {
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows 전용입니다");
}
