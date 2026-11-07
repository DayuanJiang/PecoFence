//! Optional GPU background producer. Never injects into or changes WE/Explorer windows.
use super::*;
use pecofence_platform::realtime_clock::RealtimeClock;
use pecofence_render::live_capture::LiveCapture;

pub(super) struct RealtimeGlassRuntime {
    clock: Option<RealtimeClock>,
    normal_timer_fallback: bool,
    capture: Option<LiveCapture>,
    source: isize,
    generation: u32,
    next_retry: Instant,
    failures: u32,
    pub state: &'static str,
    pub error: Option<String>,
    frames: u64,
    draws: u64,
    started: Option<Instant>,
    last_frame: Option<Instant>,
    max_tick_ms: Rc<Cell<f64>>,
    last_visibility: Option<Instant>,
    visible: Vec<FenceId>,
    last_maintenance: Instant,
}

impl Default for RealtimeGlassRuntime {
    fn default() -> Self {
        Self {
            clock: None,
            normal_timer_fallback: false,
            capture: None,
            source: 0,
            generation: 0,
            next_retry: Instant::now(),
            failures: 0,
            state: "disabled",
            error: None,
            frames: 0,
            draws: 0,
            started: None,
            last_frame: None,
            max_tick_ms: Rc::new(Cell::new(0.0)),
            last_visibility: None,
            visible: Vec::new(),
            last_maintenance: Instant::now(),
        }
    }
}

impl RealtimeGlassRuntime {
    pub fn dto(&self) -> serde_json::Value {
        let seconds = self.started.map_or(0.0, |t| t.elapsed().as_secs_f64());
        serde_json::json!({
            "state":self.state,"backend":"windowsGraphicsCaptureDesktopHost",
            "frames":self.frames,"paneDraws":self.draws,
            "averageFps":if seconds>0.0 { self.frames as f64/seconds } else {0.0},
            "lastFrameAgeMs":self.last_frame.map(|t|t.elapsed().as_millis() as u64),
            "maxTickMs":self.max_tick_ms.get(),"error":self.error,
            "cpuReadback":false,
        })
    }
}

struct TickTiming {
    began: Instant,
    maximum: Rc<Cell<f64>>,
}
impl Drop for TickTiming {
    fn drop(&mut self) {
        self.maximum.set(
            self.maximum
                .get()
                .max(self.began.elapsed().as_secs_f64() * 1000.0),
        );
    }
}

impl App {
    pub(super) fn reset_realtime_glass(&mut self) {
        window::kill_timer(self.control.hwnd(), TIMER_REALTIME_GLASS);
        window::kill_timer(self.control.hwnd(), TIMER_REALTIME_GLASS_WATCHDOG);
        self.realtime_glass = Default::default();
        let had_live = self.ctx.bitmaps.borrow_mut().clear_realtime_glass();
        if had_live {
            for fence in self.fences.values() {
                fence.redraw_realtime_glass();
            }
        }
        if self.state.config.settings.realtime_glass.enabled {
            self.realtime_glass.state = "starting";
            window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS, 1);
            window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS_WATCHDOG, 1000);
        }
    }

    fn realtime_glass_failed(&mut self, error: String) {
        if self.realtime_glass.error.as_ref() != Some(&error) {
            tracing::warn!(%error,"realtime glass unavailable; keeping standard backdrop");
        }
        self.realtime_glass.capture = None;
        self.realtime_glass.clock = None;
        self.realtime_glass.state = "unavailable";
        self.realtime_glass.error = Some(error);
        self.realtime_glass.failures = self.realtime_glass.failures.saturating_add(1);
        let seconds = (1u64 << self.realtime_glass.failures.min(5)).min(30);
        self.realtime_glass.next_retry = Instant::now() + Duration::from_secs(seconds);
        let had_live = self.ctx.bitmaps.borrow_mut().clear_realtime_glass();
        if had_live {
            for fence in self.fences.values() {
                fence.redraw_realtime_glass();
            }
        }
        window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS, 500);
    }

    fn realtime_glass_gpu_error(&mut self, error: windows_core::Error) {
        let lost = pecofence_render::is_device_lost(error.code());
        self.realtime_glass_failed(error.to_string());
        if lost {
            for fence in self.fences.values() {
                fence.report_capture_device_lost();
            }
        }
    }

    pub(super) fn watch_realtime_glass_clock(&mut self) {
        if self
            .realtime_glass
            .clock
            .as_ref()
            .is_some_and(|clock| !clock.is_running())
        {
            self.realtime_glass.clock = None;
            self.realtime_glass.normal_timer_fallback = true;
            tracing::warn!("realtime frame clock stopped; returning to normal timer until rebuilt");
            window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS, 100);
        }
    }

    fn arm_realtime_glass_clock(&mut self, fps: u32) {
        if self.realtime_glass.normal_timer_fallback {
            window::set_timer(
                self.control.hwnd(),
                TIMER_REALTIME_GLASS,
                1000u32.div_ceil(fps),
            );
            return;
        }
        if self
            .realtime_glass
            .clock
            .as_ref()
            .is_some_and(|clock| clock.fps() == fps && clock.is_running())
        {
            return;
        }
        self.realtime_glass.clock = None;
        window::kill_timer(self.control.hwnd(), TIMER_REALTIME_GLASS);
        match RealtimeClock::start(self.control.hwnd(), TIMER_REALTIME_GLASS, fps) {
            Ok(clock) => self.realtime_glass.clock = Some(clock),
            Err(error) => {
                self.realtime_glass.normal_timer_fallback = true;
                tracing::warn!(%error,"high-resolution frame clock unavailable; using normal timer");
                window::set_timer(
                    self.control.hwnd(),
                    TIMER_REALTIME_GLASS,
                    1000u32.div_ceil(fps),
                );
            }
        }
    }

    pub(super) fn tick_realtime_glass(&mut self) {
        let began = Instant::now();
        let _timing = TickTiming {
            began,
            maximum: self.realtime_glass.max_tick_ms.clone(),
        };
        let options = self.state.config.settings.realtime_glass.clone();
        if !options.enabled {
            self.reset_realtime_glass();
            return;
        }
        if !self.ctx.theme.borrow().liquid_glass
            || self.ctx.stack.on_fallback_warp()
            || self.ctx.stack.is_warp
        {
            self.realtime_glass.clock = None;
            self.realtime_glass.capture = None;
            self.ctx.bitmaps.borrow_mut().clear_realtime_glass();
            self.realtime_glass.state = "inactive";
            window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS, 500);
            return;
        }
        // Z-order can change independently of the foreground window (Peek and settings).
        // Cache this more expensive discovery; animation sampling remains at the chosen FPS.
        if self
            .realtime_glass
            .last_visibility
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(250))
        {
            self.realtime_glass.visible = self
                .fences
                .iter()
                .filter_map(|(id, fence)| {
                    let hwnd = fence.hwnd();
                    if !desktop::is_visible(hwnd) || desktop::is_cloaked(hwnd) {
                        return None;
                    }
                    if options.pause_when_covered && desktop::has_covering_window_above(hwnd) {
                        return None;
                    }
                    Some(*id)
                })
                .collect();
            self.realtime_glass.last_visibility = Some(began);
        }
        let visible = self.realtime_glass.visible.clone();
        if visible.is_empty() {
            // Closing the session also stops frame production, unlike merely not drawing.
            self.realtime_glass.capture = None;
            self.realtime_glass.clock = None;
            self.realtime_glass.state = "paused";
            self.realtime_glass.started = None;
            let had_live = self.ctx.bitmaps.borrow_mut().clear_realtime_glass();
            if had_live {
                for fence in self.fences.values() {
                    fence.redraw_realtime_glass();
                }
            }
            window::set_timer(self.control.hwnd(), TIMER_REALTIME_GLASS, 250);
            return;
        }
        if began < self.realtime_glass.next_retry {
            return;
        }
        let host = desktop::shell_window();
        if host.0.is_null() || desktop::class_name(host) != "Progman" {
            self.realtime_glass_failed(
                "The Windows desktop host is temporarily unavailable".into(),
            );
            return;
        }
        let rect = window::window_rect(host);
        let bounds = [
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        ];
        let generation = self.ctx.stack.device_generation();
        if self.realtime_glass.source != host.0 as isize
            || self.realtime_glass.generation != generation
        {
            self.realtime_glass.capture = None;
            self.ctx.bitmaps.borrow_mut().clear_realtime_glass();
        }
        if self.realtime_glass.capture.is_none() {
            match LiveCapture::for_window(&self.ctx.stack.capture_device(), host.0 as isize, bounds)
            {
                Ok(capture) => {
                    if let Err(error) = capture.set_max_fps(options.fps) {
                        tracing::debug!(%error,"capture rate property unavailable; using presentation limit");
                    }
                    self.realtime_glass.capture = Some(capture);
                    self.realtime_glass.source = host.0 as isize;
                    self.realtime_glass.generation = generation;
                    self.realtime_glass.state = "waitingForFrame";
                    self.realtime_glass.started = Some(began);
                    self.realtime_glass.frames = 0;
                    self.realtime_glass.draws = 0;
                    self.realtime_glass.last_frame = None;
                    self.realtime_glass.max_tick_ms.set(0.0);
                    self.realtime_glass.error = None;
                    tracing::info!(
                        ?bounds,
                        fps = options.fps,
                        "realtime desktop-host capture started"
                    );
                }
                Err(error) => {
                    self.realtime_glass_gpu_error(error);
                    return;
                }
            }
        }
        self.arm_realtime_glass_clock(options.fps);
        let frame = (|| -> Result<_> {
            let capture = self.realtime_glass.capture.as_mut().unwrap();
            capture.set_bounds(bounds);
            if !capture.poll_latest()? {
                return Ok(None);
            }
            Ok(capture
                .surface()
                .map(|surface| (surface, capture.content_size())))
        })();
        let (surface, size) = match frame {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                if self.realtime_glass.frames == 0
                    && self
                        .realtime_glass
                        .started
                        .is_some_and(|t| t.elapsed() > Duration::from_secs(8))
                {
                    self.realtime_glass_failed(
                        "The desktop capture produced no initial frame within eight seconds".into(),
                    );
                }
                return;
            }
            Err(error) => {
                self.realtime_glass_gpu_error(error);
                return;
            }
        };
        if size.0 <= 0 || size.1 <= 0 {
            return;
        }
        let updated = self.ctx.bitmaps.borrow_mut().update_realtime_glass(
            &self.ctx.stack,
            &surface,
            (size.0 as u32, size.1 as u32),
            bounds,
            &options,
        );
        if let Err(error) = updated {
            self.realtime_glass_gpu_error(error);
            return;
        }
        for id in visible {
            if let Some(fence) = self.fences.get(&id) {
                fence.redraw_realtime_glass();
                self.realtime_glass.draws += 1;
            }
        }
        self.realtime_glass.frames += 1;
        self.realtime_glass.last_frame = Some(began);
        self.realtime_glass.state = "active";
        self.realtime_glass.failures = 0;
        if self.realtime_glass.last_maintenance.elapsed() >= Duration::from_secs(5)
            && self
                .frame_prev
                .is_none_or(|t| t.elapsed() >= Duration::from_secs(1))
            && self.dying.is_empty()
        {
            self.prune_icons();
            self.ctx.bitmaps.borrow_mut().prune_wallpapers();
            self.realtime_glass.last_maintenance = began;
        }
    }
}
