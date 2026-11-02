//! 「临时显示 Windows 桌面」 (feedback #84): a shortcut, the tray menu or `pecofence-cli
//! windows-desktop` hides every fence and shows the Windows desktop icons PecoFence hid, and
//! the next press brings both back. Nothing of it is saved: quitting or signing out meanwhile
//! leaves the icons shown, as always, and the next start shows the fences.

use super::peek::{hotkey_json, hotkey_problem_text, peek_hotkey_label};
use super::*;

impl App {
    /// Switches 「临时显示 Windows 桌面」 on or off; `reason` (a fixed text) goes to the log.
    pub(super) fn set_windows_desktop(&mut self, on: bool, reason: &'static str) {
        if on == self.windows_desktop {
            return;
        }
        self.end_peek_now();
        let mut guard = self.anchor.borrow_mut();
        let Some(a) = guard.as_mut() else {
            return;
        };
        self.windows_desktop = on;
        a.set_fences_hidden(on, true, "windows desktop");
        if on {
            // Only the icons PecoFence hid: icons hidden in Explorer itself stay hidden.
            a.restore_desktop_icons();
        } else if self.state.config.settings.hide_real_icons && !self.no_hide_icons {
            a.request_hide_desktop_icons();
        }
        tracing::info!(on, reason, "windows desktop");
    }

    /// Registers / drops the shortcut to match the settings. While a chosen one is taken,
    /// every sync tries it again; the user hears of it once.
    pub(super) fn sync_windows_desktop_hotkey(&mut self) {
        // The settings page is recording a shortcut: `resume_hotkeys` syncs once it ends.
        if self.hotkeys_paused {
            return;
        }
        let want = self.state.config.settings.windows_desktop_hotkey();
        if want == self.windows_desktop_hotkey {
            return;
        }
        let hwnd = self.control.hwnd();
        if self.windows_desktop_hotkey.take().is_some() {
            hotkey::unregister(hwnd, HOTKEY_WINDOWS_DESKTOP);
        }
        let Some(h) = want else {
            self.windows_desktop_hotkey_taken = None;
            return;
        };
        if hotkey::register(
            hwnd,
            HOTKEY_WINDOWS_DESKTOP,
            h.mods | hotkey::mods::NOREPEAT,
            h.vk,
        ) {
            self.windows_desktop_hotkey = Some(h);
            self.windows_desktop_hotkey_taken = None;
            tracing::info!(hotkey = %h, "windows desktop hotkey registered");
            return;
        }
        tracing::warn!(hotkey = %h, "windows desktop hotkey taken");
        if self.windows_desktop_hotkey_taken.replace(h) != Some(h)
            && let Some(t) = &self.tray
        {
            t.show_info(
                "PecoFence",
                &pecofence_core::i18n::format(
                    "快捷键 {0} 已被系统或其他程序占用，“临时显示 Windows 桌面”暂时没有快捷键。",
                    &[peek_hotkey_label(h)],
                ),
                false,
            );
        }
    }

    /// Why `h` cannot be the shortcut, in the interface language (None = it can).
    fn windows_desktop_hotkey_problem(&self, h: Hotkey) -> Option<String> {
        if let Err(problem) = h.check() {
            return Some(hotkey_problem_text(problem, h));
        }
        if self.is_peek_hotkey(h) {
            return Some(pecofence_core::i18n::format(
                "{0} 已经是“{1}”的快捷键，请换一个",
                &[
                    peek_hotkey_label(h),
                    pecofence_core::i18n::text("速览栅栏").to_string(),
                ],
            ));
        }
        if self.windows_desktop_hotkey != Some(h) && !self.hotkey_free(h) {
            tracing::info!(hotkey = %h, "windows desktop hotkey refused: taken");
            return Some(pecofence_core::i18n::format(
                "{0} 已被 Windows 或其他程序占用，请换一个",
                &[peek_hotkey_label(h)],
            ));
        }
        None
    }

    /// The same check in English for `pecofence-cli settings set windowsDesktopHotkey` (the
    /// text itself was checked already).
    pub(super) fn windows_desktop_hotkey_cli_problem(&self, h: Hotkey) -> Option<String> {
        if self.is_peek_hotkey(h) {
            Some(format!("{h} is the Peek shortcut"))
        } else if self.windows_desktop_hotkey != Some(h) && !self.hotkey_free(h) {
            Some(format!("{h} is already used by Windows or another program"))
        } else {
            None
        }
    }

    /// `h` is Peek's shortcut, the chosen one or the fallback in use.
    fn is_peek_hotkey(&self, h: Hotkey) -> bool {
        let peek = &self.state.config.settings.peek;
        self.peek_hotkey == Some(h) || (peek.enabled && peek.chosen_hotkey() == h)
    }

    /// 「临时显示 Windows 桌面」 › 快捷键 on the settings page: a recorded key press, or `None`
    /// for 清除. A combination that is refused or taken leaves the settings as they are and
    /// the page keeps recording.
    pub(super) fn set_windows_desktop_hotkey_from_page(&mut self, h: Option<Hotkey>) {
        let recording = self.hotkeys_paused;
        self.resume_hotkeys();
        let refused = h.and_then(|h| self.windows_desktop_hotkey_problem(h));
        match &refused {
            Some(_) if recording => self.pause_hotkeys(),
            Some(_) => {}
            None => {
                let mut settings = self.state.config.settings.clone();
                settings.windows_desktop_hotkey = h.map(|h| h.to_string());
                self.apply_settings(settings, "settings");
            }
        }
        if let Some(host) = &self.settings {
            host.post_json(
                &serde_json::json!({ "type": "windowsDesktopHotkeyResult", "ok": refused.is_none(), "text": refused })
                    .to_string(),
            );
        }
    }

    /// `windowsDesktopHotkey` of the settings state: the chosen shortcut (`null` = none) and
    /// whether it is registered.
    pub(super) fn windows_desktop_hotkey_json(&self) -> serde_json::Value {
        let chosen = self.state.config.settings.windows_desktop_hotkey();
        serde_json::json!({
            "chosen": chosen.map(hotkey_json),
            "registered": chosen.is_some() && chosen == self.windows_desktop_hotkey,
        })
    }

    /// The tray menu entry: what a click does, with the shortcut after a tab.
    pub(super) fn windows_desktop_menu_text(&self) -> String {
        let text = if self.windows_desktop {
            pecofence_core::i18n::text("回到栅栏")
        } else {
            pecofence_core::i18n::text("临时显示 Windows 桌面")
        };
        match self.windows_desktop_hotkey {
            Some(h) => format!("{text}\t{}", peek_hotkey_label(h)),
            None => text.to_string(),
        }
    }
}
