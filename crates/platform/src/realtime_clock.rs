//! A private high-resolution frame wake-up clock, without changing system timer resolution.
//!
//! One owned thread waits on an unnamed stop event and an unnamed, one-shot kernel timer.
//! At most one wake-up is queued for this process: the receiving WM_TIMER branch must call
//! [`RealtimeClock::acknowledge`] before it performs frame work. Missed deadlines are dropped.

use crate::{
    HWND,
    bindings::{self as b, HANDLE},
};
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_core::{BOOL, PCWSTR};

windows_core::link!("kernel32.dll" "system" fn CreateWaitableTimerExW(attributes: *const core::ffi::c_void, name: PCWSTR, flags: u32, access: u32) -> HANDLE);
windows_core::link!("kernel32.dll" "system" fn SetWaitableTimer(timer: HANDLE, due: *const i64, period: i32, completion: *const core::ffi::c_void, argument: *const core::ffi::c_void, resume: BOOL) -> BOOL);
windows_core::link!("kernel32.dll" "system" fn CancelWaitableTimer(timer: HANDLE) -> BOOL);

const CREATE_WAITABLE_TIMER_HIGH_RESOLUTION: u32 = 0x2;
const TIMER_MODIFY_STATE: u32 = 0x2;
const SYNCHRONIZE: u32 = 0x0010_0000;
const WM_TIMER: u32 = 0x0113;

// There is one realtime-glass clock per process. The UI can be blocked by a modal loop or
// slow frame, so never allow its posted messages to accumulate in an unbounded queue.
static PENDING: AtomicBool = AtomicBool::new(false);

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: this wrapper owns one non-null kernel handle and closes it exactly once.
        unsafe {
            let _ = b::CloseHandle(self.0);
        }
    }
}

fn arm(timer: HANDLE, deadline: Instant) -> io::Result<()> {
    // Negative 100 ns ticks specify a relative deadline. Round up so that conversion cannot
    // turn a small positive wait into the special absolute timestamp zero.
    let ticks = deadline
        .saturating_duration_since(Instant::now())
        .as_nanos()
        .div_ceil(100)
        .clamp(1, i64::MAX as u128) as i64;
    let due = -ticks;
    // SAFETY: timer is owned and live; due remains valid for this synchronous call. No APC,
    // periodic wake-up, system timer-resolution change, or resume-from-sleep is requested.
    let result = unsafe {
        SetWaitableTimer(
            timer,
            &due,
            0,
            core::ptr::null(),
            core::ptr::null(),
            false.into(),
        )
    };
    if result.as_bool() {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Precisely paced WM_TIMER notifications for the realtime-glass renderer.
///
/// Keep only one instance alive in a process. Stop/drop the existing instance before
/// creating another rate, and call acknowledge at the start of its WM_TIMER handler.
/// All handles are private and unnamed; Drop wakes and joins the worker before closing them.
pub struct RealtimeClock {
    fps: u32,
    stop: OwnedHandle,
    timer: OwnedHandle,
    worker: Option<JoinHandle<()>>,
}

impl RealtimeClock {
    pub fn start(hwnd: HWND, timer_id: usize, fps: u32) -> io::Result<Self> {
        if hwnd.0.is_null() || timer_id == 0 || !(5..=60).contains(&fps) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "realtime clock requires a window, nonzero timer id and 5..60 fps",
            ));
        }
        // SAFETY: create an unnamed, manually reset event owned only by this clock.
        let stop = unsafe { b::CreateEventW(None, true, false, PCWSTR::null()) };
        if stop.0.is_null() {
            return Err(io::Error::last_os_error());
        }
        let stop = OwnedHandle(stop);
        // SAFETY: null attributes/name create an unnamed process-owned timer, with only the
        // access rights needed to arm it and wait. Unsupported systems return an error, so
        // the caller can keep its ordinary SetTimer fallback.
        let timer = unsafe {
            CreateWaitableTimerExW(
                core::ptr::null(),
                PCWSTR::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_MODIFY_STATE | SYNCHRONIZE,
            )
        };
        if timer.0.is_null() {
            return Err(io::Error::last_os_error());
        }
        let timer = OwnedHandle(timer);
        let period = Duration::from_nanos(1_000_000_000 / u64::from(fps));
        let first = Instant::now() + period;
        // Arm before spawning, so a setup failure is returned synchronously to the caller.
        arm(timer.0, first)?;
        PENDING.store(false, Ordering::Release);
        let stop_value = stop.0.0 as usize;
        let timer_value = timer.0.0 as usize;
        let window_value = hwnd.0 as usize;
        let worker = thread::Builder::new()
            .name("realtime-glass-clock".into())
            .stack_size(64 * 1024)
            .spawn(move || {
                // These handles stay owned by RealtimeClock until Drop has joined us.
                let stop = HANDLE(stop_value as *mut core::ffi::c_void);
                let timer = HANDLE(timer_value as *mut core::ffi::c_void);
                let hwnd = HWND(window_value as *mut core::ffi::c_void);
                let handles = [stop, timer];
                let mut deadline = first;
                loop {
                    // SAFETY: both handles remain live until after this worker exits. Stop
                    // is first, so simultaneous stop/timer signals choose shutdown.
                    let waited = unsafe { b::WaitForMultipleObjects(&handles, false, b::INFINITE) };
                    if waited == b::WAIT_OBJECT_0 as u32 { break; }
                    if waited != b::WAIT_OBJECT_0 as u32 + 1 {
                        tracing::warn!(error = %io::Error::last_os_error(), "realtime clock wait failed");
                        break;
                    }
                    if PENDING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                        // SAFETY: posting is asynchronous and performs no window/GPU work on
                        // this worker. WPARAM is the caller's ordinary WM_TIMER timer id.
                        let posted = unsafe {
                            b::PostMessageW(Some(hwnd), WM_TIMER, b::WPARAM(timer_id), b::LPARAM(0))
                        };
                        if !posted.as_bool() {
                            PENDING.store(false, Ordering::Release);
                            tracing::warn!(error = %io::Error::last_os_error(), "realtime clock post failed");
                            break;
                        }
                    }
                    deadline += period;
                    let now = Instant::now();
                    if deadline <= now {
                        // Drop missed frames. Never burst or spin to catch up after a
                        // blocked UI, suspension or a scheduler delay.
                        deadline = now + period;
                    }
                    if let Err(error) = arm(timer, deadline) {
                        tracing::warn!(%error, "realtime clock rearm failed");
                        break;
                    }
                }
            })?;
        Ok(Self {
            fps,
            stop,
            timer,
            worker: Some(worker),
        })
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn is_running(&self) -> bool {
        self.worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
    }

    /// Called at the beginning of the dedicated WM_TIMER handler, before frame work.
    pub fn acknowledge() {
        PENDING.store(false, Ordering::Release);
    }
}

impl Drop for RealtimeClock {
    fn drop(&mut self) {
        // SAFETY: both private handles stay live through the join. The stop event wakes an
        // infinite wait immediately; cancelling the timer stops any subsequent timer signal.
        unsafe {
            let _ = b::SetEvent(self.stop.0);
            let _ = CancelWaitableTimer(self.timer.0);
        }
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                tracing::warn!("realtime clock worker panicked");
            }
        }
        PENDING.store(false, Ordering::Release);
    }
}
