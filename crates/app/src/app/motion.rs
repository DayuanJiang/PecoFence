//! Frame-clock plumbing and window lifetime around fades / GPU recovery.

use super::*;

/// Debug-only fault injection (`--test-script device-lost <n>`): the next `n` device rebuilds
/// fail as a driver reset in progress would make them.
#[cfg(debug_assertions)]
pub(super) static TEST_FAILING_RECOVERIES: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0);

/// GPU memory (dedicated plus shared) this process may gain over its baseline before an idle
/// point rebuilds the render stack to hand it back (`hand_back_gpu_memory`).
const GPU_REBUILD_GROWTH: u64 = 64 << 20;
/// The shortest time between two of those rebuilds.
const GPU_REBUILD_MIN_GAP: Duration = Duration::from_secs(300);

impl App {
    /// A fence window that is no longer needed (deleted, merged into another window as a tab)
    /// fades out before it is destroyed; until then it lives in `dying`.
    pub(super) fn retire_window(&mut self, w: FenceWindow) {
        self.forget_selected_window(w.hwnd());
        if w.retire() {
            self.dying.push(w);
        }
        // Its icons may be needed nowhere now: the GPU trim prunes them (see `prune_icons`).
        window::set_timer(self.control.hwnd(), TIMER_GPU_TRIM, GPU_TRIM_IDLE_MS);
    }

    /// `DXGI_ERROR_DEVICE_REMOVED`: rebuild the device, drop GPU bitmaps, recreate every surface.
    /// A failure (the driver is still resetting) is retried on a timer: a window that lost its
    /// device reports it only once, so nothing else would ever try again. When WARP had to
    /// stand in for the GPU, the retries ask for the hardware device again until they have
    /// backed off to their longest wait (about half a minute).
    pub(super) fn recover_device(&mut self) {
        let back_to_gpu =
            self.ctx.stack.on_fallback_warp() && self.device_retry_ms < DEVICE_RETRY_MAX_MS;
        if !self.fences.values().any(|w| w.device_lost()) && !back_to_gpu {
            self.device_retry_ms = DEVICE_RETRY_FIRST_MS;
            return;
        }
        if let Err(e) = self.rebuild_render_stack() {
            tracing::error!(error = %e, "render stack recovery failed");
            self.schedule_device_retry();
            return;
        }
        if self.fences.values().any(|w| w.device_lost()) || self.ctx.stack.on_fallback_warp() {
            // A window got no new surfaces (logged there), or WARP draws: rebuild everything
            // again later.
            self.schedule_device_retry();
        } else {
            self.device_retry_ms = DEVICE_RETRY_FIRST_MS;
        }
    }

    /// Arms the recovery retry unless one is pending; each armed retry waits twice as long as
    /// the previous one, up to `DEVICE_RETRY_MAX_MS`. The windows queue one request each, so a
    /// burst of failures within one batch arms a single retry.
    fn schedule_device_retry(&mut self) {
        if self.device_retry_pending {
            return;
        }
        self.device_retry_pending = true;
        tracing::info!(ms = self.device_retry_ms, "GPU recovery retry scheduled");
        window::set_timer(
            self.control.hwnd(),
            TIMER_DEVICE_RETRY,
            self.device_retry_ms,
        );
        self.device_retry_ms = (self.device_retry_ms * 2).min(DEVICE_RETRY_MAX_MS);
    }

    /// TIMER_DEVICE_RETRY fired.
    pub(super) fn retry_device_recovery(&mut self) {
        self.device_retry_pending = false;
        self.recover_device();
    }

    /// A new GPU device, then new surfaces for every window, redrawn at once.
    fn rebuild_render_stack(&mut self) -> windows_core::Result<()> {
        self.rebuild_stack()?;
        self.ctx.bitmaps.borrow_mut().clear();
        for w in self.fences.values() {
            w.recreate_surfaces(&self.ctx.stack);
        }
        self.redraw_notices();
        Ok(())
    }

    fn rebuild_stack(&self) -> windows_core::Result<()> {
        #[cfg(debug_assertions)]
        {
            use std::sync::atomic::Ordering;
            if TEST_FAILING_RECOVERIES
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
                .is_ok()
            {
                return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
                    0x887A_0005_u32 as i32, // DXGI_ERROR_DEVICE_REMOVED
                )));
            }
        }
        // Windows that lost their device draw at once, with WARP while the driver still
        // resets. A rebuild nothing waits for (back to the GPU, handing back GPU memory) keeps
        // the current device unless the hardware one can be created.
        let lost = self.fences.values().any(|w| w.device_lost());
        self.ctx.stack.recover(lost)
    }

    /// Drawing has been quiet for `GPU_TRIM_IDLE_MS`. An animation still running keeps its
    /// caches and the check is retried: composition-only runs (fades) never draw to re-arm it.
    /// A run whose last frame is over a second old has stalled and does not block the trim.
    pub(super) fn trim_gpu_if_idle(&mut self) {
        if self
            .frame_prev
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
        {
            window::set_timer(self.control.hwnd(), TIMER_GPU_TRIM, GPU_TRIM_IDLE_MS);
            return;
        }
        // Before the trim, which then frees the GPU memory of the bitmaps dropped here.
        self.prune_icons();
        self.ctx.bitmaps.borrow_mut().prune_wallpapers();
        // Diagnostic: keep every GPU cache (compare glitches with and without the trim).
        if pecofence_core::brand::var_os("PECOFENCE_NO_GPU_TRIM").is_none() {
            self.ctx.chrome.trim();
            self.ctx.stack.trim();
        }
    }

    /// Drops every icon no fence window needs (see `needed_icons`) from the icon cache and its
    /// GPU uploads. Runs when drawing has gone quiet: whatever made icons unneeded (an icon
    /// size, layout or DPI change, a portal navigating, a fence closed or deleted, files
    /// removed) redrew a fence or retired its window, and by then the new icons have replaced
    /// the old ones on screen. Returns whether GPU bitmaps were released.
    pub(super) fn prune_icons(&mut self) -> bool {
        let needed = self.needed_icons();
        let icons = self.ctx.icons.borrow_mut().retain_needed(&needed);
        let mut bitmaps = self.ctx.bitmaps.borrow_mut();
        let before = bitmaps.len();
        bitmaps.retain(|key| !IconCache::is_draw_key(key) || needed.contains(key));
        let uploads = before - bitmaps.len();
        if icons + uploads > 0 {
            tracing::debug!(
                icons,
                uploads,
                kept = needed.len(),
                "unneeded icons dropped"
            );
        }
        uploads > 0
    }

    /// Drops the icon extractions no fence window waits for any more, keeping the cached
    /// icons: when a fence's icon size changes or a portal navigates, the new icons would
    /// otherwise queue behind the old requests until `prune_icons` runs, and that waits for
    /// the drawing to go quiet, which every icon arriving postpones.
    pub(super) fn drop_unneeded_icon_requests(&self) {
        let needed = self.needed_icons();
        self.ctx.icons.borrow_mut().drop_unneeded_requests(&needed);
    }

    /// `IconCache::draw_key`s of what the fence windows show or can switch to at once: their
    /// items at the current icon size and DPI, and the items of their other tabs at those
    /// tabs' own size, so a tab switch stays instant.
    fn needed_icons(&self) -> HashSet<String> {
        let icons = self.ctx.icons.borrow();
        let mut needed = HashSet::new();
        for (&host, w) in &self.fences {
            w.needed_icons(&icons, &mut needed);
            for (key, px) in self.state.tab_icons(host, w.active_fence(), w.dpi()) {
                needed.insert(icons.draw_key(key, px));
            }
        }
        needed
    }

    /// `TIMER_IDLE_TRIM`: retried while a frame was drawn in the last second, so an animation
    /// never pays for the pages it touches again, and while Settings, Peek or an inline rename
    /// is open or a drag, move or resize holds the mouse button (they are in use without
    /// drawing a fence).
    pub(super) fn trim_if_idle(&mut self) {
        if self
            .frame_prev
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
        {
            window::set_timer(self.control.hwnd(), TIMER_IDLE_TRIM, GPU_TRIM_IDLE_MS);
            return;
        }
        if self.settings.is_some()
            || self.peek.is_some()
            || crate::rename::active_target().is_some()
            || self.ctx.behavior.size_move.borrow().is_some()
            || window::key_down(msg::VK_LBUTTON)
        {
            window::set_timer(self.control.hwnd(), TIMER_IDLE_TRIM, IDLE_TRIM_MS);
            return;
        }
        self.hand_back_gpu_memory();
        let before = pecofence_platform::memstats::MemoryStats::current().ok();
        let started = Instant::now();
        pecofence_platform::memstats::trim_working_set();
        if let Some(m) = before {
            tracing::debug!(
                private_ws_mb = m.private_working_set / (1 << 20),
                ms = started.elapsed().as_secs_f32() * 1000.0,
                "working set handed back"
            );
        }
    }

    /// At an idle point: once this process's GPU memory grew by `GPU_REBUILD_GROWTH` over its
    /// baseline (read at the first idle point after startup and after each rebuild), rebuilds
    /// the render stack the way device loss does. DirectComposition and the GPU driver keep
    /// some memory no trim gives back (each new surface size, Liquid Glass resources sized to
    /// every glide frame); a new device starts without it. On integrated GPUs this is system
    /// memory. WARP and the basic display adapter report 0, so nothing is rebuilt there.
    fn hand_back_gpu_memory(&mut self) {
        let Some((local, shared)) = self.ctx.stack.gpu_memory() else {
            return;
        };
        let used = local + shared;
        let mb = |b: u64| (b as f64 / (1u64 << 20) as f64 * 10.0).round() / 10.0;
        let Some(baseline) = self.gpu_baseline else {
            tracing::debug!(gpu_mb = mb(used), "GPU memory baseline");
            self.gpu_baseline = Some(used);
            return;
        };
        if used < baseline.saturating_add(GPU_REBUILD_GROWTH)
            || self
                .gpu_rebuilt
                .is_some_and(|t| t.elapsed() < GPU_REBUILD_MIN_GAP)
        {
            return;
        }
        let started = Instant::now();
        self.gpu_rebuilt = Some(started);
        if let Err(e) = self.rebuild_render_stack() {
            // Nothing was replaced: the old device keeps drawing.
            tracing::warn!(error = %e, "render stack rebuild for GPU memory failed");
            return;
        }
        if self.fences.values().any(|w| w.device_lost()) {
            // A window got no new surfaces (logged there): the device-loss retry rebuilds again.
            self.schedule_device_retry();
        }
        self.gpu_baseline = None;
        let after = self.ctx.stack.gpu_memory().map_or(0, |(l, s)| l + s);
        tracing::info!(
            before_mb = mb(used),
            after_mb = mb(after),
            ms = (started.elapsed().as_secs_f32() * 1000.0).round(),
            "render stack rebuilt to hand back GPU memory"
        );
    }

    pub(super) fn on_icons_ready(&mut self) {
        let (n, next_retry, replaced) = {
            let mut icons = self.ctx.icons.borrow_mut();
            (
                icons.drain_results(),
                icons.flush_retries(),
                icons.take_replaced(),
            )
        };
        if !replaced.is_empty() {
            // A re-check after a start at sign-in brought a different picture: the old one's
            // GPU copy goes, and the windows showing it look it up again.
            self.ctx
                .bitmaps
                .borrow_mut()
                .retain(|key| !replaced.contains(key));
            for w in self.fences.values() {
                w.reload_icons(&replaced);
            }
        }
        if n > 0 {
            for w in self.fences.values() {
                w.redraw_if_pending_icons();
            }
        }
        if let Some(d) = next_retry {
            // Failed extractions waiting for another attempt, or the next re-check pass: come
            // back when the first is due.
            let ms = d.as_millis().clamp(50, 600_000) as u32;
            window::set_timer(self.control.hwnd(), TIMER_ICONS, ms);
        }
    }

    /// One compositor frame: advance every fence's client tweens; ask for another frame while
    /// anything still moves.
    pub(super) fn on_frame(&mut self) {
        let now = Instant::now();
        // Frame cadence diagnostics (debug level): a gap since the previous frame of more than
        // ~1.5 compositor periods, or a slow frame, is what the user perceives as a hitch.
        if let Some(prev) = self.frame_prev.replace(now) {
            let gap = now.duration_since(prev);
            self.frame_run.frames += 1;
            self.frame_run.max_gap = self.frame_run.max_gap.max(gap);
        } else {
            self.frame_run = FrameRun {
                started: Some(now),
                ..FrameRun::default()
            };
        }
        let mut more = false;
        // Dying windows are ticked too: their shadow fades on the frame clock until the exit
        // fade's completion drops them.
        for w in self.fences.values().chain(self.dying.iter()) {
            more |= w.on_frame(now);
        }
        self.reflow_pushed();
        // Peek dimmer: a layered window, so its alpha is a client tween ticked here. The last
        // frame writes the clear alpha before the windows are torn down.
        let mut peek_done = false;
        if let Some(p) = self.peek.as_mut() {
            more |= p.on_frame(now);
            peek_done = p.close_finished(now);
        }
        if peek_done {
            self.end_peek_now();
        }
        let spent = now.elapsed();
        self.frame_run.max_frame = self.frame_run.max_frame.max(spent);
        if more {
            self.ctx.frames.request();
        } else {
            // End of an animation run: one summary line, at info when it stuttered visibly (a
            // gap of two 60 Hz frames, or a frame that took longer than a 60 Hz period to
            // produce). Smaller hitches stay at debug: at 60 Hz a normal frame's jitter
            // already passes 17 ms, and those lines filled most of the feedback logs.
            let r = &self.frame_run;
            let total = now.duration_since(r.started.unwrap_or(now));
            let ms = |d: Duration| (d.as_secs_f32() * 1000.0 * 10.0).round() / 10.0;
            if r.max_gap > Duration::from_millis(33) || r.max_frame > Duration::from_millis(16) {
                tracing::info!(
                    frames = r.frames + 1,
                    total_ms = ms(total),
                    max_gap_ms = ms(r.max_gap),
                    max_frame_ms = ms(r.max_frame),
                    "animation run stuttered"
                );
            } else {
                tracing::debug!(
                    frames = r.frames + 1,
                    total_ms = ms(total),
                    max_gap_ms = ms(r.max_gap),
                    max_frame_ms = ms(r.max_frame),
                    "animation run"
                );
            }
            self.frame_prev = None;
        }
    }
}
