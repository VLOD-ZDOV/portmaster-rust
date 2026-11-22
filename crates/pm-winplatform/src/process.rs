//! Windows process information retrieval using Win32 API.
//!
//! Provides executable paths, process names, creation timestamps, and parent PIDs
//! for attribution on Windows platforms.

#[cfg(windows)]
use std::path::Path;

/// Details of a running Windows process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub created_at: i64,
    pub exe: String,
    pub name: String,
}

#[cfg(windows)]
#[allow(unsafe_code)]
pub fn get_process_info(pid: u32) -> Option<WinProcessInfo> {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    if pid == 0 {
        return Some(WinProcessInfo {
            pid: 0,
            ppid: 0,
            created_at: 0,
            exe: "System Idle Process".to_string(),
            name: "System Idle Process".to_string(),
        });
    }

    if pid == 4 {
        return Some(WinProcessInfo {
            pid: 4,
            ppid: 0,
            created_at: 0,
            exe: r"C:\Windows\System32\ntoskrnl.exe".to_string(),
            name: "System".to_string(),
        });
    }

    let mut exe_path = String::new();
    let mut created_at = 0i64;
    let mut ppid = 0u32;
    let mut name_fallback = String::new();

    unsafe {
        // First try to open process handle with limited query rights.
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 && handle != INVALID_HANDLE_VALUE {
            let mut buf = [0u16; 1024];
            let mut size = buf.len() as u32;
            let success = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
            if success != 0 && size > 0 {
                exe_path = String::from_utf16_lossy(&buf[..size as usize]);
            }

            let mut creation = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut exit = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut kernel = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut user = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            if GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
                let intervals =
                    ((creation.dwHighDateTime as u64) << 32) | (creation.dwLowDateTime as u64);
                // 100ns intervals from 1601 to 1970 = 11644473600 seconds = 116444736000000000 intervals
                if intervals > 116_444_736_000_000_000 {
                    created_at = ((intervals - 116_444_736_000_000_000) / 10_000) as i64;
                }
            }

            CloseHandle(handle);
        }

        // Use ToolHelp snapshot to find parent PID and fallback name
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot != 0 && snapshot != INVALID_HANDLE_VALUE {
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32ProcessID == pid {
                        ppid = entry.th32ParentProcessID;
                        let len = entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(entry.szExeFile.len());
                        name_fallback = String::from_utf16_lossy(&entry.szExeFile[..len]);
                        break;
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
        }
    }

    if exe_path.is_empty() {
        if name_fallback.is_empty() {
            return None;
        }
        exe_path = name_fallback.clone();
    }

    let name = Path::new(&exe_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            if !name_fallback.is_empty() {
                name_fallback
            } else {
                exe_path.clone()
            }
        });

    Some(WinProcessInfo {
        pid,
        ppid,
        created_at,
        exe: exe_path,
        name,
    })
}

#[cfg(not(windows))]
pub fn get_process_info(_pid: u32) -> Option<WinProcessInfo> {
    None
}
