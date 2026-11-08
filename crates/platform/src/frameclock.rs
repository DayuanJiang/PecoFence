//! A frame tick aligned with the compositor (`DCompositionWaitForCompositorClock`, or
//! `DwmFlush` on Windows 10, which does not have the compositor clock).
//!
//! Compositor-side animations (opacity, offset, clip…) never need this. It exists for the few
//! values that are rasterised *inside* a Direct2D surface each frame — a hover highlight
//! fading, the scroll offset gliding — where the app itself has to redraw. `SetTimer` ticks
//! at 15.6 ms granularity and drifts against vsync; the compositor clock does not.
//!
//! One thread waits on the clock and calls `on_frame` (typically a `PostMessage`) once per
//! vsync **while armed**. Consumers call [`FrameClock::request`] whenever they still have
//! work for the next frame; the clock disarms itself after each delivered frame, so an idle
//! app costs nothing.

use crate::bindings::*;
use crate::wide::to_wide;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use windows_core::{PCSTR, PCWSTR};

/// A missing compositor tick must not hold a short client animation past its endpoint.
/// The wait's own timeout is rounded up to the system timer (15.6 ms unless some app raised
/// its resolution), which made 16 ms into 31 ms: on a PC without a GPU, whose compositor
/// skips ticks, every animation alternated 16 and 31 ms frames. A high-resolution waitable
/// timer among the waited handles ends the wait on time instead.
const COMPOSITOR_TIMEOUT: Duration = Duration::from_millis(16);
/// The plain timeout: without the timer (Windows 10 before 1803 has no high-resolution
/// timers, but it also has no compositor clock), and as a backstop when the timer is used.
const COMPOSITOR_TIMEOUT_MS: u32 = 16;
const BACKSTOP_TIMEOUT_MS: u32 = 50;
/// Some Windows builds occasionally return from the zero-handle wait without waiting at all.
const IMMEDIATE_WAIT: Duration = Duration::from_millis(1);
/// Pace that fast-return path without limiting a real high-refresh compositor signal.
const FALLBACK_FRAME: Duration = Duration::from_micros(8_333);

/// `DCompositionWaitForCompositorClock(count, handles, timeout_ms)`.
type WaitForCompositorClock = unsafe extern "system" fn(u32, *const HANDLE, u32) -> u32;

/// The compositor clock exists from Windows 11 on. It is looked up at run time so the
/// executable still loads on Windows 10.
fn compositor_clock() -> Option<WaitForCompositorClock> {
    let name = to_wide("dcomp.dll");
    // SAFETY: loading a system DLL by name from System32 and looking up an export; the
    // transmute matches the documented signature of that export.
    unsafe {
        let dll = LoadLibraryExW(
            PCWSTR(name.as_ptr()),
            None,
            LOAD_LIBRARY_SEARCH_SYSTEM32 as u32,
        );
        if dll.0.is_null() {
            return None;
        }
        let proc = GetProcAddress(
            dll,
            PCSTR(c"DCompositionWaitForCompositorClock".as_ptr().cast()),
        )?;
        Some(std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            WaitForCompositorClock,
        >(proc))
    }
}

/// A high-resolution waitable timer for the wait timeout, or `None` where Windows has none.
fn high_resolution_timer() -> Option<HANDLE> {
    // SAFETY: plain FFI call; the handle is owned by the clock thread for its lifetime.
    let timer = unsafe {
        CreateWaitableTimerExW(
            None,
            PCWSTR::null(),
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION as u32,
            TIMER_ALL_ACCESS as u32,
        )
    };
    (!timer.0.is_null()).then_some(timer)
}

/// Sets `timer` to fire once, `after` from now. False when Windows refused.
fn arm_timer(timer: HANDLE, after: Duration) -> bool {
    // Relative due times are negative, in 100 ns units.
    let due = -((after.as_nanos() / 100) as i64);
    // SAFETY: `timer` is a live waitable timer handle; no completion routine.
    unsafe { SetWaitableTimer(timer, &due, 0, None, None, false) }.as_bool()
}

struct Shared {
    armed: AtomicBool,
    quit: AtomicBool,
    wake: Mutex<()>,
    cv: Condvar,
    /// Frames the timeout started instead of a compositor tick, since the last `take_timeouts`.
    timeouts: AtomicU32,
}

pub struct FrameClock {
    shared: Arc<Shared>,
}

impl FrameClock {
    /// Starts the clock thread. `on_frame` runs on that thread once per requested frame and
    /// must be cheap and thread-safe (post a message; never touch UI state).
    pub fn start(on_frame: impl Fn() + Send + 'static) -> Self {
        let shared = Arc::new(Shared {
            armed: AtomicBool::new(false),
            quit: AtomicBool::new(false),
            wake: Mutex::new(()),
            cv: Condvar::new(),
            timeouts: AtomicU32::new(0),
        });
        let worker = shared.clone();
        std::thread::Builder::new()
            .name("frame-clock".into())
            .spawn(move || {
                let clock = compositor_clock();
                let timer = clock.and_then(|_| high_resolution_timer());
                if clock.is_none() {
                    tracing::info!("compositor clock unavailable; pacing frames with DwmFlush");
                } else if timer.is_none() {
                    tracing::info!("no high-resolution timer; compositor clock timeout is coarse");
                }
                loop {
                    {
                        let mut guard = worker.wake.lock().unwrap();
                        while !worker.armed.load(Ordering::Acquire)
                            && !worker.quit.load(Ordering::Acquire)
                        {
                            guard = worker.cv.wait(guard).unwrap();
                        }
                    }
                    if worker.quit.load(Ordering::Acquire) {
                        if let Some(timer) = timer {
                            // SAFETY: the timer handle is owned by this thread.
                            unsafe {
                                let _ = CloseHandle(timer);
                            }
                        }
                        return;
                    }
                    let wait_started = Instant::now();
                    let waited = match (clock, timer) {
                        // A real compositor signal keeps the display's native cadence; the
                        // timer bounds a missing signal to 60 Hz. With one handle the wait
                        // returns WAIT_OBJECT_0 + 1 for a tick and WAIT_OBJECT_0 for the timer.
                        (Some(wait), Some(timer)) if arm_timer(timer, COMPOSITOR_TIMEOUT) => {
                            // SAFETY: one live timer handle; the backstop bounds the wait.
                            let r = unsafe { wait(1, &timer, BACKSTOP_TIMEOUT_MS) };
                            if r == WAIT_OBJECT_0 as u32 || r == WAIT_TIMEOUT as u32 {
                                worker.timeouts.fetch_add(1, Ordering::Relaxed);
                            }
                            r
                        }
                        // SAFETY: no handles, plain timeout wait.
                        (Some(wait), _) => unsafe {
                            wait(0, std::ptr::null(), COMPOSITOR_TIMEOUT_MS)
                        },
                        // Windows 10: block until the next desktop composition pass.
                        // SAFETY: plain FFI call without arguments.
                        (None, _) => {
                            if unsafe { DwmFlush() }.is_ok() {
                                0
                            } else {
                                WAIT_FAILED
                            }
                        }
                    };
                    let waited_for = wait_started.elapsed();
                    if waited == WAIT_FAILED || waited_for < IMMEDIATE_WAIT {
                        // On unsupported or fast-returning implementations, prevent a busy loop
                        // while still giving short client tweens several samples.
                        if let Some(left) = FALLBACK_FRAME.checked_sub(waited_for) {
                            std::thread::sleep(left);
                        }
                    }
                    if worker.armed.swap(false, Ordering::AcqRel) {
                        on_frame();
                    }
                }
            })
            .expect("frame-clock thread");
        Self { shared }
    }

    /// How many frames the timeout started instead of a compositor tick since the last call
    /// (for the animation run diagnostics).
    pub fn take_timeouts(&self) -> u32 {
        self.shared.timeouts.swap(0, Ordering::Relaxed)
    }

    /// Asks for one more frame callback. Idempotent within a frame.
    pub fn request(&self) {
        if !self.shared.armed.swap(true, Ordering::AcqRel) {
            let _guard = self.shared.wake.lock().unwrap();
            self.shared.cv.notify_one();
        }
    }
}

impl Drop for FrameClock {
    fn drop(&mut self) {
        self.shared.quit.store(true, Ordering::Release);
        let _guard = self.shared.wake.lock().unwrap();
        self.shared.cv.notify_one();
    }
}
