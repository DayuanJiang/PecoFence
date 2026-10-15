//! Process helpers: spawn a detached helper, wait for a process to exit.

use crate::bindings::*;
use crate::wide::to_wide;
use windows_core::{Error, PCWSTR, PWSTR, Result};

const SYNCHRONIZE: u32 = 0x0010_0000;

/// Blocks until the process with `pid` exits. Returns immediately if it cannot be opened.
pub fn wait_for_process_exit(pid: u32) {
    // SAFETY: plain FFI calls; the handle is closed afterwards.
    unsafe {
        let h = OpenProcess(SYNCHRONIZE, false, pid);
        if h.0.is_null() {
            return;
        }
        let _ = WaitForSingleObject(h, INFINITE);
        let _ = CloseHandle(h);
    }
}

pub fn current_pid() -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetCurrentProcessId() }
}

/// Lets any process take the foreground window from this one, e.g. the running instance that
/// a second launch asks to show its settings. Only works while this process may set it.
pub fn allow_foreground_handoff() {
    // SAFETY: plain FFI call; fails harmlessly without the foreground right.
    let _ =
        unsafe { crate::store_bindings::AllowSetForegroundWindow(crate::store_bindings::ASFW_ANY) };
}

/// How long ago the process at the server end of `pipe` (a connected client handle) started.
pub fn pipe_server_age(pipe: &std::fs::File) -> Option<std::time::Duration> {
    use std::os::windows::io::AsRawHandle;
    windows_core::link!("kernel32.dll" "system" fn GetNamedPipeServerProcessId(pipe: HANDLE, pid: *mut u32) -> windows_core::BOOL);
    windows_core::link!("kernel32.dll" "system" fn GetProcessTimes(process: HANDLE, created: *mut u64, exited: *mut u64, kernel: *mut u64, user: *mut u64) -> windows_core::BOOL);
    windows_core::link!("kernel32.dll" "system" fn GetSystemTimeAsFileTime(now: *mut u64));
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    let (mut pid, mut created, mut now, mut unused) = (0u32, 0u64, 0u64, 0u64);
    // SAFETY: plain FFI calls on a live pipe handle; FILETIMEs are written as u64 (same layout);
    // the process handle is closed before returning.
    unsafe {
        GetNamedPipeServerProcessId(HANDLE(pipe.as_raw_handle()), &mut pid)
            .ok()
            .ok()?;
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if process.0.is_null() {
            return None;
        }
        let ok = GetProcessTimes(process, &mut created, &mut unused, &mut unused, &mut unused);
        let _ = CloseHandle(process);
        ok.ok().ok()?;
        GetSystemTimeAsFileTime(&mut now);
    }
    // FILETIME counts 100 ns intervals.
    Some(std::time::Duration::from_nanos(
        now.saturating_sub(created) * 100,
    ))
}

/// Whether process `pid` runs elevated (as administrator). `None` when its token cannot be
/// read, which for another user's or an elevated process is the usual answer.
pub fn is_elevated(pid: u32) -> Option<bool> {
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    // TOKEN_INFORMATION_CLASS::TokenElevation; the answer is a TOKEN_ELEVATION (one u32).
    const TOKEN_ELEVATION_CLASS: TOKEN_INFORMATION_CLASS = 20;
    // SAFETY: plain FFI calls; the process (unless it is the pseudo handle) and token handles
    // are closed before returning.
    unsafe {
        let own = pid == GetCurrentProcessId();
        let process = if own {
            GetCurrentProcess()
        } else {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
        };
        if process.0.is_null() {
            return None;
        }
        let mut token = HANDLE::default();
        let opened = OpenProcessToken(process, TOKEN_QUERY as u32, &mut token).as_bool();
        if !own {
            let _ = CloseHandle(process);
        }
        if !opened {
            return None;
        }
        let (mut elevated, mut len) = (0u32, 0u32);
        let ok = GetTokenInformation(
            token,
            TOKEN_ELEVATION_CLASS,
            Some((&mut elevated as *mut u32).cast()),
            4,
            &mut len,
        )
        .as_bool();
        let _ = CloseHandle(token);
        ok.then_some(elevated != 0)
    }
}

/// Seconds from the start of process `pid` to the start of this one (negative when `pid`
/// started later), e.g. how long after Explorer a launch at sign-in came.
pub fn seconds_after_start_of(pid: u32) -> Option<f64> {
    windows_core::link!("kernel32.dll" "system" fn GetProcessTimes(process: HANDLE, created: *mut u64, exited: *mut u64, kernel: *mut u64, user: *mut u64) -> windows_core::BOOL);
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    // SAFETY: FILETIMEs are written as u64 (same layout) into locals.
    let created = |process: HANDLE| unsafe {
        let (mut created, mut unused) = (0u64, 0u64);
        GetProcessTimes(process, &mut created, &mut unused, &mut unused, &mut unused)
            .as_bool()
            .then_some(created)
    };
    // SAFETY: plain FFI calls; the opened process handle is closed before returning.
    let theirs = unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if process.0.is_null() {
            return None;
        }
        let at = created(process);
        let _ = CloseHandle(process);
        at?
    };
    // SAFETY: GetCurrentProcess is a pseudo handle that needs no closing.
    let ours = created(unsafe { GetCurrentProcess() })?;
    // FILETIME counts 100 ns intervals.
    Some((ours as f64 - theirs as f64) / 10_000_000.0)
}

/// True when running from an MSIX package (Microsoft Store install). Packaged processes
/// get virtualized HKCU writes, so the Run-key autostart is replaced by the manifest's
/// `windows.startupTask`, which users control under Settings > Apps > Startup.
pub fn is_packaged() -> bool {
    const APPMODEL_ERROR_NO_PACKAGE: i32 = 15700;
    windows_core::link!("kernel32.dll" "system" fn GetCurrentPackageFullName(length: *mut u32, name: windows_core::PWSTR) -> i32);
    let mut len = 0u32;
    // SAFETY: querying the required length only; a null buffer is documented as valid.
    let status = unsafe { GetCurrentPackageFullName(&mut len, PWSTR::null()) };
    status != APPMODEL_ERROR_NO_PACKAGE
}

/// Asks Windows to start this program again (without arguments) after it closed for an update:
/// a Store package update or an installer using Restart Manager. Not after a crash, a hang or
/// a reboot, where the watchdog and the startup entry take care of it.
pub fn register_restart_after_update() -> Result<()> {
    const RESTART_NO_CRASH: u32 = 1;
    const RESTART_NO_HANG: u32 = 2;
    const RESTART_NO_REBOOT: u32 = 8;
    windows_core::link!("kernel32.dll" "system" fn RegisterApplicationRestart(pwzcommandline: PCWSTR, dwflags: u32) -> windows_core::HRESULT);
    // SAFETY: a null command line is documented as valid (restart without arguments).
    unsafe {
        RegisterApplicationRestart(
            PCWSTR::null(),
            RESTART_NO_CRASH | RESTART_NO_HANG | RESTART_NO_REBOOT,
        )
    }
    .ok()
}

/// Starts `exe` with `args` as a detached process (no console window). Returns its pid.
pub fn spawn_detached(exe: &str, args: &[&str]) -> Result<u32> {
    let mut cmd = format!("\"{exe}\"");
    for a in args {
        cmd.push(' ');
        cmd.push('"');
        cmd.push_str(a);
        cmd.push('"');
    }
    let mut cmd_w = to_wide(&cmd);
    let exe_w = to_wide(exe);
    let si = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();
    // SAFETY: buffers outlive the call; handles are closed afterwards.
    unsafe {
        let ok = CreateProcessW(
            PCWSTR(exe_w.as_ptr()),
            Some(PWSTR(cmd_w.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NO_WINDOW as u32,
            None,
            PCWSTR::null(),
            &si,
            &mut pi,
        );
        if !ok.as_bool() {
            return Err(Error::from_thread());
        }
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
        Ok(pi.dwProcessId)
    }
}
