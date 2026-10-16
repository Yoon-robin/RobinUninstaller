//! `Explorer\StartupApproved` 값 해석. 순수 함수만 둔다.
//!
//! 작업 관리자가 쓰는 12바이트 REG_BINARY 값이다.
//! - 첫 바이트가 짝수(보통 `02`, `06`)면 켜짐, 홀수(`03`, `07`)면 꺼짐
//! - 꺼짐일 때 4~11바이트는 끈 시각(FILETIME)
//! - 값이 없으면 켜짐

/// 값이 없거나 비어 있으면 켜진 것으로 본다.
pub fn is_enabled(data: Option<&[u8]>) -> bool {
    match data.and_then(|d| d.first()) {
        Some(flag) => flag & 1 == 0,
        None => true,
    }
}

/// 새로 쓸 값. 기존 값의 첫 바이트가 있으면 꺼짐 비트만 바꿔서, Windows가 쓰던 다른 비트는 그대로 둔다.
pub fn encode(previous: Option<&[u8]>, enabled: bool, now_filetime: u64) -> [u8; 12] {
    let base = previous.and_then(|d| d.first()).copied().unwrap_or(0x02) & !1;
    let mut out = [0u8; 12];
    if enabled {
        out[0] = base;
    } else {
        out[0] = base | 1;
        out[4..].copy_from_slice(&now_filetime.to_le_bytes());
    }
    out
}

/// 1601-01-01부터의 100ns 단위 (FILETIME).
pub fn filetime_from_unix(unix_secs: u64) -> u64 {
    (unix_secs + 11_644_473_600) * 10_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_task_manager_values() {
        assert!(is_enabled(None));
        assert!(is_enabled(Some(&[])));
        assert!(is_enabled(Some(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])));
        assert!(is_enabled(Some(&[6, 0, 0, 0])));
        assert!(!is_enabled(Some(&[3, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8])));
        assert!(!is_enabled(Some(&[7])));
    }

    #[test]
    fn round_trips() {
        let off = encode(None, false, 0x0102_0304_0506_0708);
        assert_eq!(off[0], 3);
        assert_eq!(&off[4..], &0x0102_0304_0506_0708u64.to_le_bytes());
        assert!(!is_enabled(Some(&off)));

        let on = encode(Some(&off), true, 99);
        assert_eq!(on, [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(is_enabled(Some(&on)));

        // 06으로 켜져 있던 값은 07로 꺼지고 다시 06으로 켜진다
        let off = encode(Some(&[6, 0, 0, 0]), false, 1);
        assert_eq!(off[0], 7);
        assert_eq!(encode(Some(&off), true, 1)[0], 6);
    }

    #[test]
    fn filetime_epoch() {
        assert_eq!(filetime_from_unix(0), 116_444_736_000_000_000);
    }
}
