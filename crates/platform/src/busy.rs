//! Where the processor time goes, for the log line the App writes when PecoFence stays busy
//! (GitHub #241: a core kept a quarter busy on one machine, and the log could not tell why).
//! The UI thread tallies its window messages and WinEvent callbacks here; the process and
//! thread times come from Windows.

use crate::bindings::*;
use std::cell::RefCell;
use std::time::Duration;
use windows_core::PWSTR;

windows_core::link!("kernel32.dll" "system" fn GetThreadTimes(thread: HANDLE, created: *mut u64, exited: *mut u64, kernel: *mut u64, user: *mut u64) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn GetProcessTimes(process: HANDLE, created: *mut u64, exited: *mut u64, kernel: *mut u64, user: *mut u64) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn GetThreadDescription(thread: HANDLE, description: *mut PWSTR) -> windows_core::HRESULT);
windows_core::link!("kernel32.dll" "system" fn LocalFree(mem: *mut core::ffi::c_void) -> *mut core::ffi::c_void);
windows_core::link!("kernel32.dll" "system" fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> HANDLE);
windows_core::link!("kernel32.dll" "system" fn Thread32First(snapshot: HANDLE, entry: *mut ThreadEntry) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn Thread32Next(snapshot: HANDLE, entry: *mut ThreadEntry) -> windows_core::BOOL);
windows_core::link!("ntdll.dll" "system" fn NtQueryInformationThread(thread: HANDLE, class: u32, info: *mut core::ffi::c_void, len: u32, ret: *mut u32) -> i32);
windows_core::link!("kernel32.dll" "system" fn GetModuleHandleExW(flags: u32, address: *const core::ffi::c_void, module: *mut HMODULE) -> windows_core::BOOL);
windows_core::link!("kernel32.dll" "system" fn GetModuleFileNameW(module: HMODULE, name: *mut u16, size: u32) -> u32);

const TH32CS_SNAPTHREAD: u32 = 0x4;
/// QUERY_INFORMATION for the start address, LIMITED for the times and the name.
const THREAD_QUERY: u32 = 0x0040 | 0x0800;
const THREAD_QUERY_SET_WIN32_START_ADDRESS: u32 = 9;
/// FROM_ADDRESS | UNCHANGED_REFCOUNT
const MODULE_OF_ADDRESS: u32 = 0x4 | 0x2;

#[repr(C)]
#[derive(Default)]
struct ThreadEntry {
    size: u32,
    usage: u32,
    thread_id: u32,
    owner_pid: u32,
    base_priority: i32,
    delta_priority: i32,
    flags: u32,
}

/// What kept the UI thread: a window message (with the timer id for `WM_TIMER`) or a WinEvent.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Key {
    Message(u32),
    Timer(usize),
    Event(u32),
}

thread_local! {
    static TALLY: RefCell<Vec<(Key, u32, Duration)>> = const { RefCell::new(Vec::new()) };
}

fn note(key: Key, spent: Duration) {
    TALLY.with(|t| {
        let mut t = t.borrow_mut();
        if let Some((_, n, d)) = t.iter_mut().find(|(k, ..)| *k == key) {
            *n += 1;
            *d += spent;
        } else if t.len() < 64 {
            // Bounded: a storm is one key, not a growing list.
            t.push((key, 1, spent));
        }
    });
}

/// An outermost window message handled on this thread in `spent`.
pub fn note_message(msg: u32, wparam: usize, spent: Duration) {
    if msg == WM_TIMER as u32 {
        note(Key::Timer(wparam), spent);
    } else {
        note(Key::Message(msg), spent);
    }
}

/// A WinEvent callback (`event`) that ran on this thread in `spent`.
pub fn note_event(event: u32, spent: Duration) {
    note(Key::Event(event), spent);
}

/// The busiest entries since the last call, most time first: "WM_INPUT 52310x 4100 ms".
pub fn take_tally(top: usize) -> Vec<String> {
    let mut all = TALLY.with(|t| std::mem::take(&mut *t.borrow_mut()));
    all.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));
    all.into_iter()
        .take(top)
        .map(|(key, n, d)| {
            let what = match key {
                Key::Message(m) => match message_name(m) {
                    Some(name) => name.to_string(),
                    None => format!("message {m:#06x}"),
                },
                Key::Timer(id) => format!("timer {id}"),
                Key::Event(e) => format!("event {e:#06x}"),
            };
            format!("{what} {n}x {} ms", d.as_millis())
        })
        .collect()
}

fn message_name(m: u32) -> Option<&'static str> {
    Some(match m {
        0x000F => "WM_PAINT",
        0x0014 => "WM_ERASEBKGND",
        0x0046 => "WM_WINDOWPOSCHANGING",
        0x0047 => "WM_WINDOWPOSCHANGED",
        0x0084 => "WM_NCHITTEST",
        0x00FF => "WM_INPUT",
        0x0200 => "WM_MOUSEMOVE",
        0x02A3 => "WM_MOUSELEAVE",
        0x031E => "WM_DWMCOMPOSITIONCHANGED",
        _ => return None,
    })
}

/// The id of the calling thread (the UI thread, when the App asks).
pub fn current_thread_id() -> u32 {
    // SAFETY: plain FFI call.
    unsafe { GetCurrentThreadId() }
}

/// Processor time (kernel and user) of this process so far.
pub fn process_cpu() -> Option<Duration> {
    // SAFETY: GetCurrentProcess is a pseudo handle; FILETIMEs are written into local u64s.
    unsafe {
        let (mut c, mut e, mut k, mut u) = (0u64, 0u64, 0u64, 0u64);
        GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u)
            .as_bool()
            .then(|| Duration::from_nanos((k + u) * 100))
    }
}

/// Every thread of this process: id, name (empty when it has none) and processor time so far.
pub fn thread_cpu() -> Vec<(u32, String, Duration)> {
    let pid = std::process::id();
    let mut out = Vec::new();
    // SAFETY: plain FFI calls; the snapshot and every opened thread handle are closed below,
    // the description is freed with LocalFree as GetThreadDescription asks.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snap.0.is_null() || snap.0 as isize == -1 {
            return out;
        }
        let mut entry = ThreadEntry {
            size: size_of::<ThreadEntry>() as u32,
            ..Default::default()
        };
        let mut more = Thread32First(snap, &mut entry).as_bool();
        while more {
            if entry.owner_pid == pid {
                let thread = OpenThread(THREAD_QUERY, false, entry.thread_id);
                if !thread.0.is_null() {
                    let (mut c, mut e, mut k, mut u) = (0u64, 0u64, 0u64, 0u64);
                    let cpu = GetThreadTimes(thread, &mut c, &mut e, &mut k, &mut u)
                        .as_bool()
                        .then(|| Duration::from_nanos((k + u) * 100));
                    let mut desc = PWSTR::null();
                    let mut name =
                        if GetThreadDescription(thread, &mut desc).is_ok() && !desc.is_null() {
                            let s = desc.to_string().unwrap_or_default();
                            LocalFree(desc.0.cast());
                            s
                        } else {
                            String::new()
                        };
                    // No name (a thread of Windows or of a library): the module it started in,
                    // such as d3d10warp.dll for the software renderer.
                    if name.is_empty() {
                        name = start_module(thread).unwrap_or_default();
                    }
                    let _ = CloseHandle(thread);
                    if let Some(cpu) = cpu {
                        out.push((entry.thread_id, name, cpu));
                    }
                }
            }
            entry.size = size_of::<ThreadEntry>() as u32;
            more = Thread32Next(snap, &mut entry).as_bool();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// The file name of the module where `thread` started.
///
/// # Safety
/// `thread` is an open handle with THREAD_QUERY_INFORMATION.
unsafe fn start_module(thread: HANDLE) -> Option<String> {
    // SAFETY: out-pointers to locals of the sizes passed; the module handle is not counted
    // (UNCHANGED_REFCOUNT), so nothing is released.
    unsafe {
        let mut start: *mut core::ffi::c_void = core::ptr::null_mut();
        let status = NtQueryInformationThread(
            thread,
            THREAD_QUERY_SET_WIN32_START_ADDRESS,
            (&mut start as *mut *mut core::ffi::c_void).cast(),
            size_of::<usize>() as u32,
            core::ptr::null_mut(),
        );
        if status < 0 || start.is_null() {
            return None;
        }
        let mut module = HMODULE::default();
        if !GetModuleHandleExW(MODULE_OF_ADDRESS, start, &mut module).as_bool() {
            return None;
        }
        let mut buf = [0u16; 260];
        let len = GetModuleFileNameW(module, buf.as_mut_ptr(), buf.len() as u32) as usize;
        let path = String::from_utf16_lossy(&buf[..len.min(buf.len())]);
        path.rsplit('\\').next().map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tally_names_the_busiest_first_and_starts_again_after_it_is_read() {
        let ms = Duration::from_millis;
        note_message(0x00FF, 0, ms(3));
        note_message(0x00FF, 0, ms(4));
        note_message(WM_TIMER as u32, 42, ms(1));
        note_message(WM_TIMER as u32, 7, ms(20));
        note_event(0x8004, ms(2));
        assert_eq!(
            take_tally(3),
            [
                "timer 7 1x 20 ms",
                "WM_INPUT 2x 7 ms",
                "event 0x8004 1x 2 ms"
            ]
        );
        assert!(take_tally(3).is_empty());
    }

    #[test]
    fn this_process_and_its_threads_have_processor_times() {
        assert!(process_cpu().is_some());
        let me = current_thread_id();
        assert!(thread_cpu().iter().any(|(id, ..)| *id == me));
    }
}
