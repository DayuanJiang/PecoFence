//! Everything (voidtools' file search) through its IPC window: whether it runs, its version,
//! queries sent with `WM_COPYDATA` and the replies it sends back the same way. The byte layout
//! of queries and replies lives in `pecofence_core::everything`.

use crate::bindings::*;
use crate::wide::to_wide;
use std::path::PathBuf;
use windows_core::PCWSTR;

pub const WM_COPYDATA_MSG: u32 = WM_COPYDATA as u32;

/// Window classes of Everything's IPC window: 1.4, and 1.5 running as its default "1.5a"
/// instance.
const IPC_CLASSES: [&str; 2] = [
    // 1.5 first: when both run, the one that knows pinyin answers.
    "EVERYTHING_TASKBAR_NOTIFICATION_(1.5a)",
    "EVERYTHING_TASKBAR_NOTIFICATION",
];
/// `EVERYTHING_WM_IPC` requests (wParam of `WM_USER`).
const GET_MAJOR_VERSION: usize = 0;
const GET_MINOR_VERSION: usize = 1;

/// A running Everything (1.5 first when both run).
#[derive(Clone, Copy, Debug)]
pub struct Instance {
    pub hwnd: HWND,
    pub major: u32,
    pub minor: u32,
}

impl Instance {
    /// Everything 1.5 matches Chinese names by pinyin (`pinyin:` / `pinyin-jian:`).
    pub fn supports_pinyin(&self) -> bool {
        (self.major, self.minor) >= (1, 5)
    }

    /// The program file of the running Everything.
    pub fn exe(&self) -> Option<PathBuf> {
        crate::process::image_path(crate::desktop::window_pid(self.hwnd))
    }
}

fn ask(hwnd: HWND, request: usize) -> Option<u32> {
    let mut result = 0usize;
    // SAFETY: a plain message with no pointers; a hung Everything times out.
    let ok = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_USER as u32,
            WPARAM(request),
            LPARAM(0),
            SMTO_ABORTIFHUNG as u32,
            500,
            Some(&mut result),
        )
    };
    (ok.0 != 0).then_some(result as u32)
}

/// The running Everything, if any.
pub fn find() -> Option<Instance> {
    IPC_CLASSES.iter().find_map(|class| {
        let class = to_wide(class);
        // SAFETY: plain FFI call with a NUL-terminated class name.
        let hwnd = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) };
        if hwnd.0.is_null() {
            return None;
        }
        Some(Instance {
            hwnd,
            major: ask(hwnd, GET_MAJOR_VERSION)?,
            minor: ask(hwnd, GET_MINOR_VERSION).unwrap_or(0),
        })
    })
}

/// Sends a query (`pecofence_core::everything::query2w`) to Everything; the results come back
/// to `reply_to` as `WM_COPYDATA`. False when Everything refused it or did not answer.
pub fn send_query(target: HWND, reply_to: HWND, kind: usize, payload: &[u8]) -> bool {
    let data = COPYDATASTRUCT {
        dwData: kind,
        cbData: payload.len() as u32,
        lpData: payload.as_ptr() as *mut core::ffi::c_void,
    };
    let mut result = 0usize;
    // SAFETY: WM_COPYDATA with a structure and buffer that outlive the call; the system copies
    // the bytes into Everything's process.
    let ok = unsafe {
        SendMessageTimeoutW(
            target,
            WM_COPYDATA as u32,
            WPARAM(reply_to.0 as usize),
            LPARAM(&data as *const COPYDATASTRUCT as isize),
            SMTO_ABORTIFHUNG as u32,
            1000,
            Some(&mut result),
        )
    };
    ok.0 != 0 && result != 0
}

/// The `dwData` and the bytes of a `WM_COPYDATA` this window received (`lparam` of the message,
/// valid only while it is handled).
pub fn copydata(lparam: isize) -> Option<(usize, Vec<u8>)> {
    if lparam == 0 {
        return None;
    }
    // SAFETY: the system passes a COPYDATASTRUCT valid for the duration of WM_COPYDATA.
    unsafe {
        let data = &*(lparam as *const COPYDATASTRUCT);
        if data.lpData.is_null() || data.cbData == 0 {
            return Some((data.dwData, Vec::new()));
        }
        let bytes = std::slice::from_raw_parts(data.lpData as *const u8, data.cbData as usize);
        Some((data.dwData, bytes.to_vec()))
    }
}

/// Everything installed but not running: its program file in the usual install folders.
pub fn installed_exe() -> Option<PathBuf> {
    let mut roots = Vec::new();
    for var in [
        "ProgramFiles",
        "ProgramW6432",
        "ProgramFiles(x86)",
        "LOCALAPPDATA",
    ] {
        if let Some(v) = std::env::var_os(var) {
            roots.push(PathBuf::from(v));
        }
    }
    for root in roots {
        for dir in ["Everything 1.5a", "Everything", "Programs\\Everything"] {
            for exe in ["Everything64.exe", "Everything.exe"] {
                let p = root.join(dir).join(exe);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    None
}
