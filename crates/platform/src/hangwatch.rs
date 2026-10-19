//! Notices when the UI thread stops answering messages for as long as Windows takes before it
//! calls a program "not responding", and writes to the log which command it was running and
//! where it was stuck (a `module+rva` stack, as for a crash), so a report of a freeze says where.

use crate::bindings::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Windows marks a window "not responding" after 5 s without messages.
const HANG_MS: u32 = 5000;
const MAX_FRAMES: usize = 48;
/// `CONTEXT_FULL` on x64: control, integer and floating-point registers.
const CONTEXT_FULL: u32 = 0x0010_000B;

windows_core::link!("kernel32.dll" "system" fn RtlLookupFunctionEntry(pc: u64, image_base: *mut u64, history: *mut core::ffi::c_void) -> *const core::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn RtlVirtualUnwind(handler_type: u32, image_base: u64, pc: u64, entry: *const core::ffi::c_void, context: *mut CONTEXT, handler_data: *mut *mut core::ffi::c_void, establisher_frame: *mut u64, pointers: *mut core::ffi::c_void) -> *mut core::ffi::c_void);

/// The command the UI thread is running, if any (see [`running`]).
static RUNNING: Mutex<Option<String>> = Mutex::new(None);

/// Records the command the UI thread starts now, or `None` when it is done.
pub fn running(command: Option<&str>) {
    *RUNNING.lock().unwrap_or_else(|e| e.into_inner()) = command.map(str::to_string);
}

/// Starts watching `hwnd`, a window of the calling (UI) thread.
pub fn start(hwnd: HWND) {
    let (mut low, mut high) = (0u64, 0u64);
    // SAFETY: plain FFI calls; the out-pointers are locals.
    let thread = unsafe {
        GetCurrentThreadStackLimits(&mut low, &mut high);
        GetCurrentThreadId()
    };
    let hwnd = hwnd.0 as usize;
    let _ = std::thread::Builder::new()
        .name("hang watch".into())
        .spawn(move || watch(HWND(hwnd as *mut _), thread, (low, high)));
}

/// Whether `hwnd` answered a WM_NULL within `ms`; `None` when the window is gone.
fn answers(hwnd: HWND, ms: u32) -> Option<bool> {
    let mut result = 0usize;
    // SAFETY: WM_NULL carries nothing; the result pointer is a local.
    let sent = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_NULL as u32,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL as u32,
            ms,
            Some(&mut result),
        )
    };
    if sent.0 != 0 {
        return Some(true);
    }
    // SAFETY: plain FFI call.
    (unsafe { GetLastError() } == ERROR_TIMEOUT as u32).then_some(false)
}

fn watch(hwnd: HWND, thread: u32, stack: (u64, u64)) {
    let elapsed_ms = |since: Instant| since.elapsed().as_millis() as u64;
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let started = Instant::now();
        match answers(hwnd, HANG_MS) {
            Some(true) => continue,
            None => return,
            Some(false) => {}
        }
        let command = RUNNING.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let name = command.as_deref().unwrap_or("none");
        tracing::warn!(
            ms = elapsed_ms(started),
            command = name,
            "UI thread not responding"
        );
        let frames = thread_stack(thread, stack);
        tracing::warn!(stack = %frames.join("\n"), "UI thread stack");
        while answers(hwnd, 1000) == Some(false) {}
        tracing::info!(
            ms = elapsed_ms(started),
            command = name,
            "UI thread responding again"
        );
    }
}

/// The frames of `thread` as `module+rva`, walked while it is suspended. Only the system's
/// unwinder runs meanwhile: no allocation and no lock the stopped thread could be holding.
/// `stack` bounds the walk to the thread's stack.
fn thread_stack(thread: u32, (low, high): (u64, u64)) -> Vec<String> {
    #[repr(C, align(16))]
    struct Aligned(CONTEXT);
    let mut pcs: Vec<u64> = Vec::with_capacity(MAX_FRAMES);
    // SAFETY: the thread is resumed before the handle is closed; the unwinder reads the stopped
    // thread's stack within its bounds, and `pcs` never grows past its capacity.
    unsafe {
        let handle = OpenThread(
            (THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT) as u32,
            false,
            thread,
        );
        if handle.0.is_null() {
            return Vec::new();
        }
        if SuspendThread(handle) != u32::MAX {
            let mut ctx = Aligned(CONTEXT {
                ContextFlags: CONTEXT_FULL,
                ..Default::default()
            });
            if GetThreadContext(handle, &mut ctx.0).as_bool() {
                let ctx = &mut ctx.0;
                while pcs.len() < MAX_FRAMES && ctx.Rip != 0 && (low..high).contains(&ctx.Rsp) {
                    pcs.push(ctx.Rip);
                    let mut base = 0u64;
                    let entry = RtlLookupFunctionEntry(ctx.Rip, &mut base, std::ptr::null_mut());
                    if entry.is_null() {
                        // A leaf function: the return address is on top of the stack.
                        if ctx.Rsp + 8 > high {
                            break;
                        }
                        ctx.Rip = *(ctx.Rsp as *const u64);
                        ctx.Rsp += 8;
                    } else {
                        let mut data = std::ptr::null_mut();
                        let mut frame = 0u64;
                        RtlVirtualUnwind(
                            0,
                            base,
                            ctx.Rip,
                            entry,
                            ctx,
                            &mut data,
                            &mut frame,
                            std::ptr::null_mut(),
                        );
                    }
                }
            }
            ResumeThread(handle);
        }
        let _ = CloseHandle(handle);
    }
    pcs.iter()
        .map(|&pc| format!("  {}", crate::crashlog::locate(pc)))
        .collect()
}
