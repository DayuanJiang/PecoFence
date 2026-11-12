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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use windows_core::{PCSTR, PCWSTR};

/// A missing compositor tick must not hold a short client animation past its endpoint.
/// The wait's own timeout is rounded up to the system timer (15.6 ms unless some app raised
/// its resolution), which made 16 ms into 31 ms: on a PC without a GPU, whose compositor
/// skips ticks, every animation alternated 16 and 31 ms frames. A high-resolution waitable
/// timer among the waited handles ends the wait on time instead.
///
/// The timer is due this long after the moment the next tick is expected ([`Pace`]), so a
/// tick that comes on time always ends the wait first. A fixed 16 ms from the start of the
/// wait fired 0.4 ms before every tick of a 60 Hz display (one every 16.67 ms) when a frame
/// took under 0.7 ms to draw: 1.1 frames per refresh, at uneven times.
const TICK_SLACK: Duration = Duration::from_millis(2);
/// How long the clock listens for the display's tick after a frame the timer started,
/// before it waits to be asked for the next frame. The tick of a display whose phase the
/// clock does not know yet (the compositor clock rests while nothing changes, so the first
/// frame of a run may be the timer's) can come while the frame is being drawn; unheard, it
/// left every frame of that run to the timer, each about a millisecond before its tick
/// (1 wheel glide in 72 on a PC with a GPU).
const TICK_LISTEN: Duration = Duration::from_millis(3);
/// The refresh interval until Windows reports one, and when it reports none: 60 Hz.
const DEFAULT_REFRESH: Duration = Duration::from_micros(16_667);
/// The longest interval the timer counts with (55.5 Hz; a 59 Hz display keeps its own).
const SLOWEST_PACE: Duration = Duration::from_millis(18);
/// Two ticks closer together than this (and than half a refresh interval) are a wait that
/// did not wait.
const DOUBLE_TICK: Duration = Duration::from_millis(4);
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

/// The interval the tick timer counts with: the refresh interval the compositor reports
/// (or [`DEFAULT_REFRESH`]), halved until it is no slower than [`SLOWEST_PACE`].
fn refresh_interval() -> Duration {
    let mut info = DWM_TIMING_INFO {
        cbSize: std::mem::size_of::<DWM_TIMING_INFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a live DWM_TIMING_INFO with its size set; a null window asks for the
    // desktop's timing.
    let ok = unsafe { DwmGetCompositionTimingInfo(HWND::default(), &mut info) }.is_ok();
    let rate = info.rateRefresh;
    let (num, den) = (rate.uiNumerator, rate.uiDenominator);
    let refresh = refresh_from_rate(ok, num, den);
    let timer = timer_interval(refresh);
    let answer = (u64::from(!ok) << 63)
        | ((refresh.as_micros() as u64 & 0x7FFF_FFFF) << 32)
        | (timer.as_micros() as u64 & 0xFFFF_FFFF);
    if refresh_is_news(&LOGGED_REFRESH, answer) {
        tracing::debug!(
            ok,
            num,
            den,
            refresh_us = refresh.as_micros() as u64,
            timer_us = timer.as_micros() as u64,
            "frame clock: refresh interval for the tick timer"
        );
    }
    timer
}

/// What [`refresh_interval`] logged last: its line is written when the answer changes. While
/// the display sleeps the compositor wait fails at once and every frame asks again, which
/// wrote one line per frame (2,100 lines, 360 KB, in three minutes of animations at debug
/// level).
static LOGGED_REFRESH: AtomicU64 = AtomicU64::new(0);

/// Whether `answer` is not what was logged last; it is noted as logged.
fn refresh_is_news(logged: &AtomicU64, answer: u64) -> bool {
    logged.swap(answer, Ordering::Relaxed) != answer
}

/// `num / den` Hz as an interval; [`DEFAULT_REFRESH`] for a failed call and for a rate outside
/// 22 to 500 Hz (the wait's 50 ms backstop must stay behind the timer).
fn refresh_from_rate(ok: bool, num: u32, den: u32) -> Duration {
    if ok && num > 0 && den > 0 {
        let refresh = Duration::from_secs_f64(den as f64 / num as f64);
        if refresh >= Duration::from_millis(2) && refresh <= Duration::from_millis(45) {
            return refresh;
        }
    }
    DEFAULT_REFRESH
}

/// A compositor slower than 60 Hz still gets a frame from the timer between two of its ticks,
/// as it did with the fixed 16 ms: its interval is halved until it is under 18 ms. That is
/// what the timer is there for. Without a GPU (virtual machines, remote desktop) Windows
/// reports 32 Hz and ticks every 31.25 ms; counted with that, the timer never fired and a
/// 167 ms roll-up drew 6 frames where it draws 11. A 50 or 30 Hz display gets two frames per
/// refresh, as before.
fn timer_interval(mut refresh: Duration) -> Duration {
    while refresh > SLOWEST_PACE {
        refresh /= 2;
    }
    refresh
}

/// When the timer that bounds a wait for the compositor is due, and which returns of the
/// wait start a frame.
///
/// The next tick is expected one interval ([`refresh_interval`]) after the last one. The
/// timer is due [`TICK_SLACK`] after that moment, so on a display of 60 Hz or more that
/// ticks it never fires, whatever the refresh rate and however short the frame's own work
/// was. When a tick is missing, the timer's frame keeps the cadence: the tick after it is
/// expected one interval after the missed one.
///
/// A wrong interval (a rate Windows did not report, a monitor or session that changed
/// during a run) lasts until the next run asks again, and does little harm meanwhile. Too
/// long: every tick still ends the wait first, only a missing one is noticed later. Too
/// short: the timer starts the frame ahead of its tick. A tick that follows less than half
/// an interval later starts no second frame and is taken as the new starting point: still
/// one frame per refresh. A later one starts a frame of its own, as every tick after a timer
/// frame did before: two frames per refresh, each refresh showing the newer.
struct Pace {
    refresh: Duration,
    /// The last tick: when it came, or when it was expected if the timer ended the wait.
    tick: Option<Instant>,
    /// When the last frame started, and whether the timer started it.
    frame: Option<(Instant, bool)>,
    /// When the wait in progress expects its tick.
    expected: Instant,
}

/// How a wait for the compositor ended.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Ended {
    Tick,
    Timer,
    /// Neither: the wait failed (it does at once while the display sleeps).
    Failed,
}

/// What the end of a wait starts.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Start {
    Frame,
    /// A frame after the rest of [`FALLBACK_FRAME`]: the wait did not wait.
    PacedFrame,
    Nothing,
}

impl Pace {
    fn new(now: Instant) -> Self {
        Self {
            refresh: DEFAULT_REFRESH,
            tick: None,
            frame: None,
            expected: now,
        }
    }

    /// How long from `now` the timer of a wait that starts now may run. `refresh` is asked
    /// for the display's interval when there is no recent tick to count from (a run starts,
    /// a frame took several intervals).
    fn timer_due(&mut self, now: Instant, refresh: impl FnOnce() -> Duration) -> Duration {
        let recent = self.tick.and_then(|tick| {
            let since = now.saturating_duration_since(tick);
            (since < self.refresh * 3).then_some((tick, since))
        });
        self.expected = match recent {
            // The first expected tick after `now`: a frame that took longer than one
            // interval waits for the next tick, not for one that has passed.
            Some((tick, since)) => {
                let n = (since.as_nanos() / self.refresh.as_nanos()) as u32 + 1;
                tick + self.refresh * n
            }
            None => {
                self.refresh = refresh();
                now + self.refresh
            }
        };
        self.expected.saturating_duration_since(now) + TICK_SLACK
    }

    /// A tick heard at `now` while no frame was asked for (right after a frame the timer
    /// started): the next one is expected one interval after it.
    fn tick_heard(&mut self, now: Instant) {
        self.tick = Some(now);
    }

    /// The wait ended at `now`. A tick less than half an interval after a frame the timer
    /// started is the one that frame stood in for: it starts nothing. A tick right after a
    /// frame another tick started is a wait that did not wait (some Windows builds return
    /// from it at once now and then); it is paced like a failed wait. A tick that comes as
    /// soon as the wait begins because the frame before took one interval to draw is neither.
    fn wait_ended(&mut self, now: Instant, ended: Ended) -> Start {
        let since_frame = self
            .frame
            .map(|(at, by_timer)| (now.saturating_duration_since(at), by_timer));
        let start = match ended {
            Ended::Timer => {
                self.tick = Some(self.expected);
                Start::Frame
            }
            Ended::Failed => {
                self.tick = None;
                Start::PacedFrame
            }
            Ended::Tick => {
                self.tick = Some(now);
                match since_frame {
                    Some((since, true)) if since < self.refresh / 2 => return Start::Nothing,
                    Some((since, false)) if since < (self.refresh / 2).min(DOUBLE_TICK) => {
                        Start::PacedFrame
                    }
                    _ => Start::Frame,
                }
            }
        };
        // A paced frame starts when the fallback sleep is over.
        let at = if start == Start::PacedFrame {
            now + FALLBACK_FRAME
        } else {
            now
        };
        self.frame = Some((at, ended == Ended::Timer));
        start
    }
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
                let mut pace = Pace::new(Instant::now());
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
                    // Whether the timer started this frame.
                    let mut by_timer = false;
                    // Whether the wait failed or did not wait.
                    let paced = match (clock, timer) {
                        // A real compositor signal keeps the display's native cadence; the
                        // timer bounds a missing signal to one refresh interval. With one
                        // handle the wait returns WAIT_OBJECT_0 + 1 for a tick and
                        // WAIT_OBJECT_0 for the timer.
                        (Some(wait), Some(timer))
                            if arm_timer(timer, pace.timer_due(wait_started, refresh_interval)) =>
                        {
                            // SAFETY: one live timer handle; the backstop bounds the wait.
                            let r = unsafe { wait(1, &timer, BACKSTOP_TIMEOUT_MS) };
                            let ended = if r == WAIT_OBJECT_0 as u32 + 1 {
                                Ended::Tick
                            } else if r == WAIT_OBJECT_0 as u32 || r == WAIT_TIMEOUT as u32 {
                                worker.timeouts.fetch_add(1, Ordering::Relaxed);
                                Ended::Timer
                            } else {
                                Ended::Failed
                            };
                            by_timer = ended == Ended::Timer;
                            match pace.wait_ended(Instant::now(), ended) {
                                Start::Frame => false,
                                Start::PacedFrame => true,
                                // The frame the timer started is already drawn for this tick.
                                Start::Nothing => continue,
                            }
                        }
                        (Some(wait), _) => {
                            // SAFETY: no handles, plain timeout wait.
                            let r = unsafe { wait(0, std::ptr::null(), COMPOSITOR_TIMEOUT_MS) };
                            r == WAIT_FAILED || wait_started.elapsed() < IMMEDIATE_WAIT
                        }
                        // Windows 10: block until the next desktop composition pass.
                        // SAFETY: plain FFI call without arguments.
                        (None, _) => {
                            unsafe { DwmFlush() }.is_err()
                                || wait_started.elapsed() < IMMEDIATE_WAIT
                        }
                    };
                    if paced {
                        // On unsupported or fast-returning implementations, prevent a busy loop
                        // while still giving short client tweens several samples.
                        if let Some(left) = FALLBACK_FRAME.checked_sub(wait_started.elapsed()) {
                            std::thread::sleep(left);
                        }
                    }
                    if worker.armed.swap(false, Ordering::AcqRel) {
                        on_frame();
                    }
                    // The frame is being drawn: listen for the tick it stood in for.
                    if by_timer
                        && let (Some(wait), Some(timer)) = (clock, timer)
                        && arm_timer(timer, TICK_LISTEN)
                    {
                        // SAFETY: one live timer handle; the backstop bounds the wait.
                        let r = unsafe { wait(1, &timer, BACKSTOP_TIMEOUT_MS) };
                        if r == WAIT_OBJECT_0 as u32 + 1 {
                            pace.tick_heard(Instant::now());
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hz(rate: f64) -> Duration {
        Duration::from_secs_f64(1.0 / rate)
    }

    /// The clock thread's loop against a display that ticks every `real` (never: `None`),
    /// with the interval Windows reported as `believed` and `work` of drawing per frame.
    /// Returns when each frame started (ms) and how many of them the timer started.
    fn run(
        believed: Duration,
        real: Option<Duration>,
        work: Duration,
        frames: usize,
    ) -> (Vec<f64>, usize) {
        let t0 = Instant::now();
        let mut now = t0 + Duration::from_micros(700);
        let mut pace = Pace::new(now);
        let (mut starts, mut by_timer_count) = (Vec::new(), 0);
        while starts.len() < frames {
            let timer_at = now + pace.timer_due(now, || believed);
            let tick_at = real.map(|r| {
                let n = (now.duration_since(t0).as_nanos() / r.as_nanos()) as u32 + 1;
                t0 + r * n
            });
            let by_timer = tick_at.is_none_or(|t| t > timer_at);
            now = if by_timer { timer_at } else { tick_at.unwrap() };
            let ended = if by_timer { Ended::Timer } else { Ended::Tick };
            match pace.wait_ended(now, ended) {
                Start::Nothing => {}
                start => {
                    assert_eq!(start, Start::Frame, "a healthy wait is never paced");
                    by_timer_count += by_timer as usize;
                    starts.push(now.duration_since(t0).as_secs_f64() * 1000.0);
                    now += work;
                }
            }
        }
        (starts, by_timer_count)
    }

    fn gaps(starts: &[f64]) -> Vec<f64> {
        starts.windows(2).map(|w| w[1] - w[0]).collect()
    }

    #[test]
    fn the_refresh_interval_is_logged_when_it_changes_not_for_every_frame() {
        let logged = AtomicU64::new(0);
        // A sleeping display: every frame gets the same answer.
        let lines = (0..2_000)
            .filter(|_| refresh_is_news(&logged, 16_667))
            .count();
        assert_eq!(lines, 1);
        // The display comes back at another rate, then sleeps again.
        assert!(refresh_is_news(&logged, 6_944));
        assert!(!refresh_is_news(&logged, 6_944));
        assert!(refresh_is_news(&logged, 16_667));
    }

    #[test]
    fn a_ticking_display_starts_every_frame_itself_at_60_120_and_144_hz() {
        for rate in [60.0, 120.0, 144.0] {
            // 0.1 and 0.3 ms: the frames a fixed 16 ms timer overtook at 60 Hz.
            for work_us in [100, 300, 1_500] {
                let (starts, by_timer) = run(
                    hz(rate),
                    Some(hz(rate)),
                    Duration::from_micros(work_us),
                    240,
                );
                assert_eq!(by_timer, 0, "{rate} Hz, {work_us} us of work");
                for gap in gaps(&starts) {
                    assert!(
                        (gap - 1000.0 / rate).abs() < 0.01,
                        "{rate} Hz, {work_us} us of work: a gap of {gap} ms"
                    );
                }
            }
        }
    }

    #[test]
    fn without_ticks_the_timer_keeps_one_frame_per_refresh_interval() {
        // A PC without a GPU whose compositor does not tick: the 167 ms roll-up still gets
        // its 11 frames, however long each one takes to draw.
        for work_ms in [1, 5, 12] {
            let (starts, by_timer) = run(hz(60.0), None, Duration::from_millis(work_ms), 60);
            assert_eq!(by_timer, 60);
            for gap in gaps(&starts) {
                assert!((gap - 1000.0 / 60.0).abs() < 0.01, "a gap of {gap} ms");
            }
        }
    }

    #[test]
    fn a_missing_tick_costs_two_ms_and_the_next_tick_is_on_time_again() {
        let mut pace = Pace::new(Instant::now());
        let t0 = Instant::now();
        let refresh = hz(60.0);
        assert_eq!(pace.timer_due(t0, || refresh), refresh + TICK_SLACK);
        assert_eq!(pace.wait_ended(t0 + refresh, Ended::Tick), Start::Frame);
        // The next tick does not come: the timer ends the wait 2 ms after it was due.
        let now = t0 + refresh + Duration::from_micros(300);
        let due = pace.timer_due(now, || unreachable!());
        assert_eq!(now + due, t0 + refresh * 2 + TICK_SLACK);
        assert_eq!(pace.wait_ended(now + due, Ended::Timer), Start::Frame);
        // The one after it is expected where the display would have had it.
        let now = now + due + Duration::from_millis(1);
        let due = pace.timer_due(now, || unreachable!());
        assert_eq!(now + due, t0 + refresh * 3 + TICK_SLACK);
    }

    #[test]
    fn a_slower_display_than_reported_still_gets_one_even_frame_per_refresh() {
        // Windows said 60 Hz (or nothing), the display ticks at 50 Hz: the timer starts each
        // frame 1.3 ms before the tick, and the tick right after it starts no second frame.
        let (starts, _) = run(hz(60.0), Some(hz(50.0)), Duration::from_micros(300), 120);
        for gap in &gaps(&starts)[1..] {
            assert!((gap - 20.0).abs() < 0.01, "a gap of {gap} ms");
        }
    }

    #[test]
    fn a_faster_display_than_reported_starts_every_frame_itself() {
        let (starts, by_timer) = run(hz(60.0), Some(hz(144.0)), Duration::from_micros(300), 240);
        assert_eq!(by_timer, 0);
        for gap in gaps(&starts) {
            assert!((gap - 1000.0 / 144.0).abs() < 0.01, "a gap of {gap} ms");
        }
    }

    #[test]
    fn a_frame_longer_than_one_interval_waits_for_the_tick_after_it() {
        let (starts, by_timer) = run(hz(60.0), Some(hz(60.0)), Duration::from_millis(24), 60);
        assert_eq!(by_timer, 0);
        for gap in gaps(&starts) {
            assert!((gap - 2000.0 / 60.0).abs() < 0.01, "a gap of {gap} ms");
        }
    }

    #[test]
    fn a_frame_that_ends_just_before_the_next_tick_starts_on_that_tick() {
        // The tick ends the wait in under 1 ms: it is a real one, the frame is not held back
        // for the fallback sleep (a roll's 6.6 ms frames at 144 Hz, 16.5 ms ones at 60 Hz).
        for (rate, work_us) in [(60.0, 16_500), (144.0, 6_600)] {
            let (starts, by_timer) = run(
                hz(rate),
                Some(hz(rate)),
                Duration::from_micros(work_us),
                120,
            );
            assert_eq!(by_timer, 0);
            for gap in gaps(&starts) {
                assert!((gap - 1000.0 / rate).abs() < 0.01, "a gap of {gap} ms");
            }
        }
    }

    #[test]
    fn a_wait_that_does_not_wait_is_paced_like_a_failed_one() {
        let t0 = Instant::now();
        let mut pace = Pace::new(t0);
        pace.timer_due(t0, || hz(60.0));
        assert_eq!(pace.wait_ended(t0, Ended::Tick), Start::Frame);
        // Every wait returns a "tick" as soon as the frame before is drawn.
        let mut now = t0 + Duration::from_micros(300);
        for _ in 0..5 {
            pace.timer_due(now, || unreachable!());
            assert_eq!(pace.wait_ended(now, Ended::Tick), Start::PacedFrame);
            now += FALLBACK_FRAME + Duration::from_micros(300);
        }
        // The display sleeps: the wait fails at once, and the interval is asked for again
        // when it is back.
        pace.timer_due(now, || unreachable!());
        assert_eq!(pace.wait_ended(now, Ended::Failed), Start::PacedFrame);
        let mut asked = false;
        pace.timer_due(now + FALLBACK_FRAME, || {
            asked = true;
            hz(60.0)
        });
        assert!(asked);
    }

    #[test]
    fn a_tick_heard_after_a_timer_frame_sets_the_phase() {
        // A run begins while the compositor clock rests: the timer starts the first frame
        // at a moment that has nothing to do with the display's ticks.
        let t0 = Instant::now();
        let refresh = hz(60.0);
        let mut pace = Pace::new(t0);
        let due = pace.timer_due(t0, || refresh);
        assert_eq!(pace.wait_ended(t0 + due, Ended::Timer), Start::Frame);
        // The display's tick comes 0.6 ms later, while that frame is drawn.
        let tick = t0 + due + Duration::from_micros(600);
        pace.tick_heard(tick);
        // The next wait expects the tick one interval after the one heard: it ends the wait.
        let now = tick + Duration::from_micros(400);
        let due = pace.timer_due(now, || unreachable!());
        assert_eq!(now + due, tick + refresh + TICK_SLACK);
        assert_eq!(pace.wait_ended(tick + refresh, Ended::Tick), Start::Frame);
    }

    #[test]
    fn a_compositor_without_a_gpu_keeps_its_frame_between_two_ticks() {
        // Windows reports 32 Hz there and ticks every 31.25 ms: the timer counts with half of
        // it and starts a frame between two ticks, 64 a second, whatever a frame costs.
        assert_eq!(timer_interval(hz(32.0)), hz(32.0) / 2);
        for work_ms in [0, 5, 11] {
            let (starts, by_timer) = run(
                timer_interval(hz(32.0)),
                Some(hz(32.0)),
                Duration::from_millis(work_ms),
                120,
            );
            assert!(
                (59..=61).contains(&by_timer),
                "{by_timer} of 120 by the timer"
            );
            let total = starts[starts.len() - 1] - starts[1];
            assert!(
                (total / 118.0 - 15.625).abs() < 0.05,
                "mean gap {}",
                total / 118.0
            );
            for gap in &gaps(&starts)[1..] {
                assert!((13.0..=18.0).contains(gap), "a gap of {gap} ms");
            }
        }
    }

    #[test]
    fn slow_displays_are_paced_at_twice_or_four_times_their_rate() {
        assert_eq!(timer_interval(hz(60.0)), hz(60.0));
        assert_eq!(timer_interval(hz(59.0)), hz(59.0));
        assert_eq!(timer_interval(hz(144.0)), hz(144.0));
        assert_eq!(timer_interval(hz(50.0)), hz(50.0) / 2);
        assert_eq!(timer_interval(hz(30.0)), hz(30.0) / 2);
        assert_eq!(timer_interval(hz(24.0)), hz(24.0) / 4);
        // A 50 Hz display: a frame from the timer, a frame from the tick, in every refresh.
        let (starts, by_timer) = run(
            timer_interval(hz(50.0)),
            Some(hz(50.0)),
            Duration::from_micros(300),
            100,
        );
        assert!((49..=51).contains(&by_timer));
        let total = starts[starts.len() - 1] - starts[1];
        assert!((total / 98.0 - 10.0).abs() < 0.05);
    }

    #[test]
    fn the_refresh_interval_falls_back_to_60_hz() {
        assert_eq!(refresh_from_rate(true, 144, 1), hz(144.0));
        assert_eq!(refresh_from_rate(true, 60_000, 1_001).as_micros(), 16_683);
        assert_eq!(refresh_from_rate(false, 144, 1), DEFAULT_REFRESH);
        assert_eq!(refresh_from_rate(true, 0, 0), DEFAULT_REFRESH);
        assert_eq!(refresh_from_rate(true, 60, 0), DEFAULT_REFRESH);
        assert_eq!(refresh_from_rate(true, 1, 1), DEFAULT_REFRESH);
        assert_eq!(refresh_from_rate(true, 5_000, 1), DEFAULT_REFRESH);
    }
}
