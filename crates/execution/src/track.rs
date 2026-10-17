//! 언인스톨러가 띄운 자식 프로세스까지 추적한다.
//!
//! NSIS·Inno Setup 언인스톨러는 자기 자신을 임시 폴더에 복사해 다시 실행하고 원본은 바로 종료한다.
//! 그래서 최상위 프로세스만 기다리면 실제 제거가 끝나기 전에 "완료"로 판단하게 된다.
//! 프로세스 스냅샷을 주기적으로 찍어, 추적 중인 프로세스를 부모로 둔 새 프로세스를 계속 편입한다.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use windows::Win32::Foundation::CloseHandle;

use crate::ignore::is_ignored;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

const POLL: Duration = Duration::from_millis(250);

/// 프로세스가 만들어진 시각 (FILETIME, 100ns 단위). 열 수 없으면 None.
fn creation_time(pid: u32) -> Option<u64> {
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let (mut created, mut exited, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        let ok = GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user);
        let _ = CloseHandle(handle);
        ok.ok()?;
        Some((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
    }
}

/// 지금 시각 (FILETIME, 100ns 단위)
fn now_filetime() -> u64 {
    let unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    unix.as_nanos() as u64 / 100 + 116_444_736_000_000_000
}

struct Proc {
    ppid: u32,
    exe: String,
}

/// pid → (부모 pid, 실행 파일 이름)
pub(crate) fn snapshot_pids() -> Vec<u32> {
    snapshot().into_keys().collect()
}

fn snapshot() -> HashMap<u32, Proc> {
    let mut map = HashMap::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return map;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                map.insert(
                    entry.th32ProcessID,
                    Proc {
                        ppid: entry.th32ParentProcessID,
                        exe: String::from_utf16_lossy(&entry.szExeFile[..len]),
                    },
                );
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    map
}

/// 지금 실행 중인 프로세스의 실행 파일 이름 (소문자). 정리 전에 "실행 중이면 건너뜀" 확인용.
pub fn running_exe_names() -> std::collections::HashSet<String> {
    snapshot()
        .into_values()
        .map(|p| p.exe.to_lowercase())
        .collect()
}

/// `root_pid`와 그 자손이 모두 끝날 때까지 기다린다. 살아 있는 개수가 바뀔 때마다 `on_count`를 부른다.
///
/// - pid는 재사용될 수 있으므로 (pid, 부모 pid) 쌍으로 추적한다. 같은 pid라도 부모가 다르면 다른 프로세스다.
///   최상위 프로세스는 호출자가 핸들을 쥐고 있어 pid가 재사용되지 않는다.
/// - 콘솔 호스트·설문 브라우저처럼 기다릴 필요 없는 프로세스는 세지 않는다(그 자식도 따라가지 않음).
/// - `stop`이 켜지면(사용자가 "기다리지 않고 확인") 바로 돌아간다.
pub fn wait_for_tree(
    root_pid: u32,
    target_name: &str,
    stop: &AtomicBool,
    mut on_count: impl FnMut(usize),
) {
    // 살아 있는 추적 대상: pid → 처음 봤을 때의 부모 pid (최상위는 None)
    let mut alive: HashMap<u32, Option<u32>> = HashMap::from([(root_pid, None)]);
    let mut ever_seen: Vec<u32> = vec![root_pid];
    // 추적한 프로세스가 만들어진 시각과, 끝난 것을 처음 본 시각.
    // 끝난 프로세스의 pid를 다른 프로그램이 다시 받아 자식을 만들면, 그 자식은 우리 것이 아니다.
    let mut created: HashMap<u32, u64> = HashMap::new();
    if let Some(t) = creation_time(root_pid) {
        created.insert(root_pid, t);
    }
    let mut gone_at: HashMap<u32, u64> = HashMap::new();
    let mut last_count = 0;

    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        let procs = snapshot();
        let now = now_filetime();

        // 끝났거나, 같은 pid를 다른 프로세스가 다시 받은(만든 시각이 다름) 추적 대상을 기록한다
        for &pid in &ever_seen {
            if gone_at.contains_key(&pid) {
                continue;
            }
            let reused = || {
                created
                    .get(&pid)
                    .is_some_and(|&t| creation_time(pid).is_some_and(|now_t| now_t != t))
            };
            if !procs.contains_key(&pid) || reused() {
                gone_at.insert(pid, now);
            }
        }

        // 추적 중이었던 프로세스의 자식을 편입한다. 부모가 이미 끝났어도 자식의 ppid 값은 남아 있다.
        for (&pid, proc) in &procs {
            if ever_seen.contains(&proc.ppid) && !ever_seen.contains(&pid) && pid != proc.ppid {
                let child_created = creation_time(pid);
                // 부모가 끝난 뒤에 만들어진 프로세스는 pid를 물려받은 남의 자식이다
                if let (Some(c), Some(&gone)) = (child_created, gone_at.get(&proc.ppid))
                    && c > gone
                {
                    continue;
                }
                if let Some(c) = child_created {
                    created.insert(pid, c);
                }
                ever_seen.push(pid);
                if !is_ignored(&proc.exe, target_name) {
                    alive.insert(pid, Some(proc.ppid));
                }
            }
        }

        alive.retain(|pid, expected_ppid| match procs.get(pid) {
            None => false,
            Some(actual) => expected_ppid.is_none_or(|e| e == actual.ppid),
        });

        if alive.len() != last_count {
            last_count = alive.len();
            on_count(last_count);
        }
        if alive.is_empty() {
            return;
        }
        std::thread::sleep(POLL);
    }
}
