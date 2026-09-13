//! Frame-clock plumbing and window lifetime around fades / GPU recovery.

use super::*;

impl App {
    /// A fence window that is no longer needed (deleted, merged into another window as a tab)
    /// fades out before it is destroyed; until then it lives in `dying`.
    pub(super) fn retire_window(&mut self, w: FenceWindow) {
        if w.retire() {
            self.dying.push(w);
        }
    }

    /// `DXGI_ERROR_DEVICE_REMOVED`: rebuild the device, drop GPU bitmaps, recreate every surface.
    pub(super) fn recover_device(&mut self) {
        if !self.fences.values().any(|w| w.device_lost()) {
            return;
        }
        if let Err(e) = self.ctx.stack.recover() {
            tracing::error!(error = %e, "render stack recovery failed");
            return;
        }
        self.ctx.bitmaps.borrow_mut().clear();
        for w in self.fences.values() {
            w.recreate_surfaces(&self.ctx.stack);
        }
    }

    pub(super) fn on_icons_ready(&mut self) {
        let (n, next_retry) = {
            let mut icons = self.ctx.icons.borrow_mut();
            (icons.drain_results(), icons.flush_retries())
        };
        if n > 0 {
            for w in self.fences.values() {
                w.redraw_if_pending_icons();
            }
        }
        if let Some(d) = next_retry {
            // Failed extractions waiting for another attempt: come back when the first is due.
            let ms = d.as_millis().clamp(50, 10_000) as u32;
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
            // End of an animation run: one summary line, at info when it stuttered (a gap of
            // more than ~2 frames at 143 Hz / 1 frame at 60 Hz, or a frame that took longer
            // than a 143 Hz period to produce).
            let r = &self.frame_run;
            let total = now.duration_since(r.started.unwrap_or(now));
            let ms = |d: Duration| (d.as_secs_f32() * 1000.0 * 10.0).round() / 10.0;
            if r.max_gap > Duration::from_millis(17) || r.max_frame > Duration::from_millis(7) {
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
