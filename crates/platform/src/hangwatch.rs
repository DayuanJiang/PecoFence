//! Notices when the UI thread stops answering messages for as long as Windows takes before it
//! calls a program "not responding", and writes to the log which command it was running and
//! where it was stuck (`module+rva` return addresses, as for a crash), so a report of a freeze
//! says where.

use crate::bindings::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Windows marks a window "not responding" after 5 s without messages.
const HANG_MS: u32 = 5000;
const MAX_FRAMES: usize = 48;
/// How much of the stuck thread's stack is copied for the scan.
const STACK_COPY: usize = 64 * 1024;
/// `CONTEXT_FULL` on x64: control, integer and floating-point registers.
const CONTEXT_FULL: u32 = 0x0010_000B;

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
    // Marks the log as one of a version that also writes `process exiting` when it ends.
    tracing::info!("hang watch started");
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

/// The stuck thread's instruction pointer and the return addresses on the top of its stack,
/// as `module+rva`. The thread is suspended only to read its registers and copy that part of
/// its stack; nothing else runs meanwhile, because anything more (the system's unwinder, the
/// loader, the heap) may wait for a lock the stopped thread holds, and the watch would then
/// never resume it. The copy is scanned afterwards: a value counts as a return address when
/// it points into a loaded module right behind a call instruction. Every real frame shows, in
/// order, but older return addresses left on the stack show between them (in a 7 s test
/// stall, 24 of 41 frames).
fn thread_stack(thread: u32, (low, high): (u64, u64)) -> Vec<String> {
    #[repr(C, align(16))]
    struct Aligned(CONTEXT);
    let mut copy = vec![0u64; STACK_COPY / 8];
    let mut words = 0;
    let mut rip = 0;
    // SAFETY: the thread is resumed before the handle is closed; the copy reads the stopped
    // thread's committed stack from its stack pointer up, within its bounds and `copy`'s length.
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
            if GetThreadContext(handle, &mut ctx.0).as_bool() && (low..high).contains(&ctx.0.Rsp) {
                rip = ctx.0.Rip;
                words = (((high - ctx.0.Rsp) / 8) as usize).min(copy.len());
                std::ptr::copy_nonoverlapping(ctx.0.Rsp as *const u64, copy.as_mut_ptr(), words);
            }
            ResumeThread(handle);
        }
        let _ = CloseHandle(handle);
    }
    if rip == 0 {
        return Vec::new();
    }
    std::iter::once(rip)
        .chain(
            copy[..words]
                .iter()
                .copied()
                .filter(|&v| after_call(v))
                .take(MAX_FRAMES),
        )
        .map(|pc| format!("  {}", crate::crashlog::locate_unpinned(pc)))
        .collect()
}

/// Whether `addr` points into a loaded module right behind a call instruction: `call rel32`,
/// or `call` through a register or memory operand (`FF /2`, 2, 3, 6 or 7 bytes long).
fn after_call(addr: u64) -> bool {
    let Some(module) = crate::crashlog::module_holding(addr) else {
        return false;
    };
    if addr < module.0 as u64 + 8 {
        return false;
    }
    // The module may be unloaded meanwhile (it is not pinned): ReadProcessMemory fails then,
    // where a plain read would fault.
    let mut code = [0u8; 7];
    let mut read = 0usize;
    // SAFETY: the buffer is a local of the stated length; the source is checked by the call.
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            (addr - 7) as *const core::ffi::c_void,
            code.as_mut_ptr().cast(),
            code.len(),
            Some(&mut read),
        )
    };
    if !ok.as_bool() || read != code.len() {
        return false;
    }
    let b = |back: u64| code[code.len() - back as usize];
    let modrm = |back: u64, modbits: u8| {
        b(back) == 0xFF && b(back - 1) >> 6 == modbits && (b(back - 1) >> 3) & 7 == 2
    };
    b(5) == 0xE8
        || modrm(2, 0b11)
        || modrm(2, 0b00)
        || modrm(3, 0b01)
        || (b(6) == 0xFF && b(5) == 0x15)
        || modrm(6, 0b10)
        || (modrm(7, 0b10) && b(6) & 7 == 0b100)
}
