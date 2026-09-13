//! A frame tick aligned with the compositor (`DCompositionWaitForCompositorClock`).
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// A missing compositor tick must not hold a short client animation past its endpoint.
const COMPOSITOR_TIMEOUT_MS: u32 = 16;
/// Some Windows builds occasionally return from the zero-handle wait without waiting at all.
const IMMEDIATE_WAIT: Duration = Duration::from_millis(1);
/// Pace that fast-return path without limiting a real high-refresh compositor signal.
const FALLBACK_FRAME: Duration = Duration::from_micros(8_333);

struct Shared {
    armed: AtomicBool,
    quit: AtomicBool,
    wake: Mutex<()>,
    cv: Condvar,
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
        });
        let worker = shared.clone();
        std::thread::Builder::new()
            .name("frame-clock".into())
            .spawn(move || {
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
                        return;
                    }
                    let wait_started = Instant::now();
                    // SAFETY: no handles, plain timeout wait. A real compositor signal keeps
                    // the display's native cadence; timeout bounds a missing signal to 60 Hz.
                    let waited =
                        unsafe { DCompositionWaitForCompositorClock(None, COMPOSITOR_TIMEOUT_MS) };
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
