//! Startup transfers icon ownership after a frame with its wallpaper material reaches DWM.

use super::*;
use pecofence_render::stack::PendingPresentation;

const COMMIT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_COMMIT_ATTEMPTS: u32 = 3;
const COMMIT_RETRY_BACKOFF: Duration = Duration::from_secs(10);
const MATERIAL_WARNING_AFTER: Duration = Duration::from_secs(10);

fn material_ready(has_wallpaper: bool, diagnostic_solid: bool) -> bool {
    has_wallpaper || diagnostic_solid
}

#[derive(Debug, PartialEq, Eq)]
enum HandoffDecision {
    Wait,
    Retry,
    BackOff,
    TakeOver,
}

fn handoff_decision(
    presented: Option<bool>,
    elapsed: Duration,
    windows_ready: bool,
    attempts: u32,
) -> HandoffDecision {
    match presented {
        None if elapsed < COMMIT_TIMEOUT => HandoffDecision::Wait,
        Some(true) if windows_ready => HandoffDecision::TakeOver,
        _ if attempts < MAX_COMMIT_ATTEMPTS => HandoffDecision::Retry,
        _ => HandoffDecision::BackOff,
    }
}

pub(super) struct StartupHandoff {
    started: Instant,
    stale_marker: bool,
    attempts: u32,
    presentation: Option<PendingPresentation>,
    retry_after: Option<Instant>,
    pub(super) windows_prepared: bool,
    material_warning_shown: bool,
    revealed: bool,
}

impl StartupHandoff {
    pub(super) fn new(started: Instant, stale_marker: bool) -> Self {
        Self {
            started,
            stale_marker,
            attempts: 0,
            presentation: None,
            retry_after: None,
            windows_prepared: false,
            material_warning_shown: false,
            revealed: false,
        }
    }

    fn back_off(&mut self, now: Instant) {
        self.presentation = None;
        self.attempts = 0;
        self.retry_after = Some(now + COMMIT_RETRY_BACKOFF);
    }

    fn retry_due(&self, now: Instant) -> bool {
        self.retry_after.is_none_or(|deadline| now >= deadline)
    }
}

impl App {
    fn startup_material_ready(&self) -> bool {
        material_ready(
            self.wallpaper_sig.is_some(),
            pecofence_core::brand::var_os("PECOFENCE_SOLID").is_some(),
        )
    }

    pub(super) fn request_startup_presentation(&mut self) {
        let material_ready = self.startup_material_ready();
        let Some(startup) = self.startup_handoff.as_mut() else {
            return;
        };
        if !startup.windows_prepared || !material_ready || !startup.retry_due(Instant::now()) {
            window::set_timer(self.control.hwnd(), TIMER_STARTUP_PRESENTED, 1_000);
            return;
        }
        startup.retry_after = None;
        if startup.revealed {
            if let Some(anchor) = self.anchor.borrow_mut().as_mut() {
                anchor.hold_startup_visibility(false);
            }
            tracing::info!(
                count = self.fences.len(),
                "startup phase: showing prepared fences without fade"
            );
            // A display/layout change may have created windows since the previous attempt.
            for fence in self.fences.values() {
                fence.show_immediate();
            }
            if let Some(anchor) = self.anchor.borrow_mut().as_mut() {
                anchor.reanchor("prepared startup frame");
                tracing::debug!("z-order after show:\n{}", anchor.zorder_report(false));
            }
        } else {
            for fence in self.fences.values() {
                fence.prepare_startup_frame();
            }
            tracing::info!("startup phase: committing wallpaper material before showing fences");
        }
        startup.attempts += 1;
        let control = self.control.hwnd().0 as isize;
        startup.presentation = match self.ctx.stack.present_first_frame(move || {
            window::post_message(
                HWND(control as *mut core::ffi::c_void),
                WM_APP_STARTUP_PRESENTED,
                0,
                0,
            );
        }) {
            Ok(presentation) => Some(presentation),
            Err(error) => {
                tracing::warn!(%error, "startup visual-tree commit could not be requested");
                None
            }
        };
        window::set_timer(self.control.hwnd(), TIMER_STARTUP_PRESENTED, 1_000);
    }

    pub(super) fn invalidate_startup_presentation(&mut self) {
        if let Some(startup) = self.startup_handoff.as_mut()
            && startup.presentation.is_some()
        {
            // An old callback can still arrive; it only polls the new presentation's result.
            startup.attempts = 0;
            tracing::debug!("startup layout changed; requesting a fresh presentation");
            self.request_startup_presentation();
        }
    }

    pub(super) fn check_startup_presentation(&mut self) {
        let material_ready = self.startup_material_ready();
        let Some(startup) = self.startup_handoff.as_mut() else {
            window::kill_timer(self.control.hwnd(), TIMER_STARTUP_PRESENTED);
            return;
        };
        if !startup.windows_prepared || !material_ready {
            // Keep Explorer usable on an unavailable/invalid wallpaper. Retrying runs on a
            // worker; the tray and Settings remain responsive and no black fence is exposed.
            if startup.started.elapsed() >= MATERIAL_WARNING_AFTER
                && !startup.material_warning_shown
            {
                startup.material_warning_shown = true;
                tracing::warn!(
                    "startup material unavailable; retaining Explorer desktop while retrying"
                );
            }
            return;
        }
        if !startup.retry_due(Instant::now()) {
            return;
        }
        if startup.presentation.is_none() && startup.attempts == 0 {
            self.request_startup_presentation();
            return;
        }
        let (presented, elapsed) = if let Some(presentation) = &startup.presentation {
            let elapsed = presentation.started.elapsed();
            let presented = match presentation.take_result() {
                Some(Ok(())) => Some(true),
                Some(Err(error)) => {
                    tracing::warn!(%error, "startup frame presentation failed; retaining Explorer icons");
                    Some(false)
                }
                None if elapsed < COMMIT_TIMEOUT => None,
                None => {
                    tracing::warn!(
                        attempt = startup.attempts,
                        "startup commit timed out; retaining Explorer icons"
                    );
                    None
                }
            };
            (presented, elapsed)
        } else {
            (Some(false), Duration::ZERO)
        };
        // A failed window creation must not turn the desktop empty just because an empty
        // visual-tree commit succeeded. An intentionally hidden empty inbox is not required.
        let hidden = self.state.hidden_inbox();
        let revealed = startup.revealed;
        let windows_ready = self
            .state
            .host_fences()
            .iter()
            .filter(|f| Some(f.id) != hidden)
            .all(|f| {
                self.fences.get(&f.id).is_some_and(|w| {
                    if revealed {
                        w.ready_for_desktop_takeover()
                    } else {
                        w.ready_for_startup_material()
                    }
                })
            });
        match handoff_decision(presented, elapsed, windows_ready, startup.attempts) {
            HandoffDecision::Wait => return,
            HandoffDecision::Retry => {
                self.request_startup_presentation();
                return;
            }
            HandoffDecision::BackOff => {
                tracing::warn!(
                    windows_ready,
                    "startup presentation unavailable; retaining Explorer icons and retrying after backoff"
                );
                // Keep ownership of the visibility hold: dropping this state strands hidden
                // windows forever even after a transient compositor/device failure recovers.
                startup.back_off(Instant::now());
                return;
            }
            HandoffDecision::TakeOver => {}
        }
        if !startup.revealed {
            // The material was committed while hidden. Reveal it without a fade, then wait
            // for that visible frame before taking ownership of Explorer's desktop icons.
            startup.revealed = true;
            startup.attempts = 0;
            tracing::info!("startup phase: hidden material commit completed");
            self.request_startup_presentation();
            return;
        }
        let startup = self.startup_handoff.take().unwrap();
        self.desktop_takeover_ready = true;
        window::kill_timer(self.control.hwnd(), TIMER_STARTUP_PRESENTED);
        tracing::info!(
            count = self.fences.len(),
            elapsed_ms = startup.started.elapsed().as_millis(),
            "startup phase: first fence frame presented"
        );
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            if self.state.config.settings.hide_real_icons && !self.no_hide_icons {
                tracing::info!("startup phase: requesting desktop icon hide after first frame");
                // Explorer may still be settling; the anchor retains its existing retry ladder,
                // recovery marker and watchdog ownership.
                a.request_hide_desktop_icons();
            }
            a.reanchor("startup handoff");
        }
        if startup.stale_marker {
            tracing::warn!("stale icons-hidden marker found: previous run did not restore icons");
            if let Some(t) = &self.tray {
                t.show_info(
                    "PecoFence",
                    pecofence_core::i18n::text(
                        "上次 PecoFence 没有正常退出，桌面图标没能恢复显示。现在已重新接管。",
                    ),
                    true,
                );
            }
        }
        self.finish_startup_work();
    }

    fn finish_startup_work(&mut self) {
        self.remove_old_update_downloads();
        self.offer_crash_feedback();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placeholder_is_not_a_ready_material() {
        assert!(!material_ready(false, false));
        assert!(material_ready(true, false));
        assert!(material_ready(false, true));
    }

    #[test]
    fn only_a_successful_presentation_with_ready_windows_allows_takeover() {
        assert_eq!(
            handoff_decision(Some(true), Duration::ZERO, true, 1),
            HandoffDecision::TakeOver
        );
        for presented in [Some(false), None] {
            assert_eq!(
                handoff_decision(presented, COMMIT_TIMEOUT, true, 1),
                HandoffDecision::Retry
            );
        }
        assert_eq!(
            handoff_decision(Some(true), Duration::ZERO, false, 1),
            HandoffDecision::Retry
        );
    }

    #[test]
    fn an_incomplete_commit_waits_without_hiding_or_consuming_retries() {
        for attempts in [1, MAX_COMMIT_ATTEMPTS] {
            assert_eq!(
                handoff_decision(
                    None,
                    COMMIT_TIMEOUT - Duration::from_millis(1),
                    true,
                    attempts
                ),
                HandoffDecision::Wait
            );
        }
    }

    #[test]
    fn exhausted_presentation_failures_never_authorize_hiding() {
        for (presented, ready) in [(None, true), (Some(false), true), (Some(true), false)] {
            assert_eq!(
                handoff_decision(presented, COMMIT_TIMEOUT, ready, MAX_COMMIT_ATTEMPTS),
                HandoffDecision::BackOff
            );
        }
    }

    #[test]
    fn exhausted_attempts_preserve_handoff_and_resume_after_backoff() {
        let now = Instant::now();
        for revealed in [false, true] {
            let mut startup = StartupHandoff::new(now, true);
            startup.windows_prepared = true;
            startup.revealed = revealed;
            startup.attempts = MAX_COMMIT_ATTEMPTS;
            startup.back_off(now);
            assert!(startup.windows_prepared && startup.stale_marker);
            assert_eq!(startup.revealed, revealed);
            assert_eq!(startup.attempts, 0);
            assert!(!startup.retry_due(now + COMMIT_RETRY_BACKOFF - Duration::from_millis(1)));
            assert!(startup.retry_due(now + COMMIT_RETRY_BACKOFF));
        }
    }
}
