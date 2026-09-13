//! Peek: hotkey registration with fallbacks, toggle / end, hotkey label helpers.

use super::*;

/// `RegisterHotKey` arguments for a Peek hotkey choice.
pub(super) fn peek_hotkey_args(h: PeekHotkey) -> (u32, u32) {
    match h {
        PeekHotkey::WinSpace => (hotkey::mods::WIN | hotkey::mods::NOREPEAT, hotkey::VK_SPACE),
        PeekHotkey::CtrlAltSpace => (
            hotkey::mods::CONTROL | hotkey::mods::ALT | hotkey::mods::NOREPEAT,
            hotkey::VK_SPACE,
        ),
        PeekHotkey::WinShiftSpace => (
            hotkey::mods::WIN | hotkey::mods::SHIFT | hotkey::mods::NOREPEAT,
            hotkey::VK_SPACE,
        ),
    }
}

pub(super) fn peek_hotkey_label(h: PeekHotkey) -> &'static str {
    match h {
        PeekHotkey::WinSpace => pecofence_core::i18n::text("Win+空格"),
        PeekHotkey::CtrlAltSpace => pecofence_core::i18n::text("Ctrl+Alt+空格"),
        PeekHotkey::WinShiftSpace => pecofence_core::i18n::text("Win+Shift+空格"),
    }
}

impl App {
    /// Registers / re-registers / drops the Peek hotkey to match the settings.
    pub(super) fn sync_peek_hotkey(&mut self) {
        let want = self
            .state
            .config
            .settings
            .peek
            .enabled
            .then_some(self.state.config.settings.peek.hotkey);
        // While a fallback is registered `want != peek_hotkey`, so every sync (startup, preset,
        // settings apply) retries the preferred combination first and switches back once free.
        if want == self.peek_hotkey {
            return;
        }
        let prev = self.peek_hotkey;
        let choice_changed = want != self.peek_hotkey_wanted;
        self.peek_hotkey_wanted = want;
        let hwnd = self.control.hwnd();
        if self.peek_hotkey.is_some() {
            hotkey::unregister(hwnd, HOTKEY_PEEK);
            self.peek_hotkey = None;
        }
        if let Some(h) = want {
            // The chosen combination first, then the others: Win+Space family combinations are
            // reserved by Windows on multi-layout systems, so a fallback keeps Peek usable.
            let candidates = [
                h,
                PeekHotkey::CtrlAltSpace,
                PeekHotkey::WinShiftSpace,
                PeekHotkey::WinSpace,
            ];
            let mut registered = None;
            for c in candidates {
                let (mods, vk) = peek_hotkey_args(c);
                if hotkey::register(hwnd, HOTKEY_PEEK, mods, vk) {
                    registered = Some(c);
                    break;
                }
            }
            match registered {
                Some(c) => {
                    self.peek_hotkey = Some(c);
                    tracing::info!(hotkey = peek_hotkey_label(c), "peek hotkey registered");
                    if c != h {
                        tracing::warn!(
                            wanted = peek_hotkey_label(h),
                            "peek hotkey taken; using fallback"
                        );
                        // The user's choice stays in settings (it is retried on every sync);
                        // toast only when the effective combination actually changed.
                        if (Some(c) != prev || choice_changed)
                            && let Some(t) = &self.tray
                        {
                            t.show_info(
                                "PecoFence",
                                &pecofence_core::i18n::format(
                                    "快捷键 {0} 已被系统或其他程序占用，「浮现栅栏」改用 {1}。",
                                    &[
                                        peek_hotkey_label(h).to_string(),
                                        peek_hotkey_label(c).to_string(),
                                    ],
                                ),
                                false,
                            );
                        }
                    }
                }
                None => {
                    tracing::warn!("no peek hotkey combination is free");
                    if (prev.is_some() || choice_changed)
                        && let Some(t) = &self.tray
                    {
                        t.show_info(
                            "PecoFence",
                            pecofence_core::i18n::text(
                                "所有「浮现栅栏」快捷键组合都已被占用，可从托盘菜单使用该功能。",
                            ),
                            true,
                        );
                    }
                }
            }
        }
    }

    /// Peek: fences float above everything (over a dimmer) until ended.
    pub(super) fn toggle_peek(&mut self) {
        if let Some(p) = self.peek.as_mut() {
            let now = Instant::now();
            if p.is_closing() {
                // Pressed again during the fade-out: dim again, nothing to rebuild.
                p.reopen(now);
                self.ctx.frames.request();
            } else {
                self.end_peek();
            }
            return;
        }
        if self.fences.is_empty() {
            return;
        }
        let dim = self.state.config.settings.peek.dim;
        let overlay = match PeekOverlay::show(
            &self.peek_class,
            self.queue.clone(),
            self.control.hwnd(),
            dim,
            &self.ctx.motion,
        ) {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!(error = %e, "peek dimmer failed");
                return;
            }
        };
        let over = overlay.hwnds();
        let animating = overlay.animating(Instant::now());
        self.peek = Some(overlay);
        self.ctx.behavior.floating.set(true);
        for window in self.fences.values() {
            window.redraw();
        }
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.set_peek(true, &over);
        }
        if animating {
            self.ctx.frames.request();
        }
    }

    /// The asynchronous focus handoff may have raised the topmost dimmer above the fences.
    /// Reapply the Peek band only if this completion belongs to the current overlay.
    pub(super) fn on_peek_focused(&mut self, dimmer: HWND) {
        if !self.peek.as_ref().is_some_and(|p| p.contains(dimmer)) {
            return;
        }
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.reanchor("peek focused");
        }
    }

    /// Ends the peek the way the user sees it: the dimmer fades out (83 ms) while the fences
    /// stay in the topmost band, then `end_peek_now` runs from the frame tick. Repeated calls
    /// during the fade are no-ops.
    pub(super) fn end_peek(&mut self) {
        let now = Instant::now();
        let Some(p) = self.peek.as_mut() else {
            return;
        };
        if p.is_closing() {
            return;
        }
        if !self.ctx.motion.enabled() {
            self.end_peek_now();
            return;
        }
        p.begin_close(now);
        if p.close_finished(now) {
            self.end_peek_now();
        } else {
            self.ctx.frames.request();
        }
    }

    /// Ends the peek at once (layout rebuilds, config swaps, shutdown).
    pub(super) fn end_peek_now(&mut self) {
        if self.peek.is_none() {
            return;
        }
        self.ctx.behavior.floating.set(false);
        for window in self.fences.values() {
            window.redraw();
        }
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.set_peek(false, &[]);
        }
        // Dropping the windows destroys them (after the fences left the topmost band).
        self.peek = None;
    }
}
