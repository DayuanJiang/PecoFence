//! Everything (voidtools' file search) through its IPC window: whether it runs, its version,
//! queries sent with `WM_COPYDATA` and the replies it sends back the same way. The byte layout
//! of queries and replies lives in `pecofence_core::everything`.

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use std::path::{Path, PathBuf};
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
    installed_in(&["Everything 1.5a", "Everything", "Programs\\Everything"])
}

/// Everything 1.5 installed in its own folder ("Everything 1.5a", beside a 1.4 in
/// "Everything"): what 「改用 1.5」 starts.
pub fn installed_15_exe() -> Option<PathBuf> {
    installed_in(&["Everything 1.5a"])
}

fn installed_in(dirs: &[&str]) -> Option<PathBuf> {
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
        for dir in dirs {
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

/// The machine-wide Run keys (as `reg.exe` names them: 64-bit, then the 32-bit view) whose value
/// "Everything" (the 1.4 installer's) starts `exe` for all users at sign-in.
pub fn all_users_run_keys(exe: &Path) -> Vec<&'static str> {
    const RRF_RT_REG_SZ: u32 = 0x0000_0002;
    const RRF_SUBKEY_WOW6464KEY: u32 = 0x0001_0000;
    const RRF_SUBKEY_WOW6432KEY: u32 = 0x0002_0000;
    let key = to_wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let name = to_wide("Everything");
    [
        (
            RRF_SUBKEY_WOW6464KEY,
            r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run",
        ),
        (
            RRF_SUBKEY_WOW6432KEY,
            r"HKLM\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
        ),
    ]
    .into_iter()
    .filter(|(view, _)| {
        let mut buf = [0u16; 1024];
        let mut size = (buf.len() * 2) as u32;
        // SAFETY: out-pointers reference locals with the declared sizes.
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(key.as_ptr()),
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ | view,
                None,
                Some(buf.as_mut_ptr().cast()),
                Some(&mut size),
            )
        };
        status.0 == 0 && command_names(&from_wide(&buf[..(size as usize / 2).min(buf.len())]), exe)
    })
    .map(|(_, reg_key)| reg_key)
    .collect()
}

/// Whether a Run command line starts `exe` (the quoted part, or the text before " -").
pub fn command_names(command: &str, exe: &Path) -> bool {
    let command = command.trim_end_matches('\0').trim();
    let program = match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next().unwrap_or(rest),
        None => command.split(" -").next().unwrap_or(command),
    };
    program.trim().eq_ignore_ascii_case(&exe.to_string_lossy())
}

/// Runs `program` with `params` (in `dir`) and waits up to `wait_ms` for it to end; `elevated`
/// asks Windows for administrator rights first. False when it did not start (an elevation
/// refused too) or did not end in time.
pub fn run_program(
    program: &Path,
    params: &str,
    dir: Option<&Path>,
    elevated: bool,
    wait_ms: u32,
) -> bool {
    const SEE_MASK_NOCLOSEPROCESS: u32 = 0x40;
    const WAIT_OBJECT_0: u32 = 0;
    let file = to_wide(&program.to_string_lossy());
    let params = to_wide(params);
    let dir = dir.map(|d| to_wide(&d.to_string_lossy()));
    let verb = to_wide(if elevated { "runas" } else { "open" });
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        // No error window of the shell's: a refused or failed start returns false at once.
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC as u32 | SEE_MASK_FLAG_NO_UI as u32,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        lpDirectory: dir.as_ref().map_or(PCWSTR::null(), |d| PCWSTR(d.as_ptr())),
        nShow: SW_HIDE,
        ..Default::default()
    };
    // SAFETY: every referenced buffer outlives the call; the process handle is closed below.
    let started = unsafe { ShellExecuteExW(&mut info).as_bool() };
    if !started || info.hProcess.0.is_null() {
        return false;
    }
    // SAFETY: a process handle this call owns.
    unsafe {
        let ended = WaitForSingleObject(info.hProcess, wait_ms) == WAIT_OBJECT_0;
        let _ = CloseHandle(info.hProcess);
        ended
    }
}
