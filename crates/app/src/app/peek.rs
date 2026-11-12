//! Peek: hotkey registration with fallbacks, the shortcut field on the settings page, toggle /
//! end, and the search across all fences (`peek_search.rs` is its box).

use super::*;
use pecofence_core::hotkey::{FALLBACKS, PRESETS, Problem};

/// The reasons `end_peek` gets when the keyboard or the foreground left the Peek's windows.
const PEEK_FOCUS_LOST: [&str; 3] = [
    "another window came to the front",
    "dimmer lost the focus",
    "search box lost the focus",
];
/// How long after the Peek began the desktop coming to the front is taken for the late answer
/// to a click before the shortcut (seen 0.65 s after it on Windows 10), not for a move away.
const PEEK_LATE_DESKTOP: Duration = Duration::from_millis(1500);
/// Frames between making a covered fence transparent and raising it over the window that hides
/// it: the opacity reaches the compositor only when the message loop has run again, and the
/// fence must not come over the window while it is still opaque on screen.
pub(super) const PEEK_RAISE_FRAMES: u8 = 1;
/// The longest that wait may take (a window timer, so about 63 ms in effect). A compositor
/// clock (Windows 11) gives its two beats 25 to 40 ms after the fences were made transparent,
/// before this, so nothing changes there: the timer finds the fences raised. Windows 10 has
/// no compositor clock: the frame clock waits in `DwmFlush`, and on a PC without a GPU its
/// first two beats after a rest took 127 to 174 ms together, so the covered fences were first
/// seen 232 to 279 ms after the shortcut (60 to 112 ms in 0.3.3, which did not fade). With
/// the limit they are raised 63 to 67 ms after they were made transparent and seen after 149
/// to 186 ms; no opaque fence showed before the fade in the Peeks of the guest's checks.
pub(super) const PEEK_RAISE_LIMIT_MS: u32 = 50;
/// Frames a faded fence stays transparent after the Peek put it back under the windows. A
/// change of the z-order reaches the screen a frame or two after a change of opacity does:
/// made opaque in the same turn, the fence showed for one frame over the window that hides it
/// again (4 of 6 closes on a test box). Nobody sees the wait, the fence is under that window.
const PEEK_REST_FRAMES: u8 = 4;

/// A shortcut as the tray menu and messages show it: `Ctrl+Alt+空格`.
pub(super) fn peek_hotkey_label(h: Hotkey) -> String {
    hotkey_keys(h).join("+")
}

/// The key names of `h` in the interface language, one per key cap.
pub(super) fn hotkey_keys(h: Hotkey) -> Vec<String> {
    h.parts()
        .into_iter()
        .map(|part| {
            match part.as_str() {
                "Ctrl" => pecofence_core::i18n::text("键名\x04Ctrl"),
                "Shift" => pecofence_core::i18n::text("键名\x04Shift"),
                "Space" => pecofence_core::i18n::text("空格"),
                "Enter" => pecofence_core::i18n::text("键名\x04Enter"),
                "Esc" => pecofence_core::i18n::text("键名\x04Esc"),
                "Backspace" => pecofence_core::i18n::text("键名\x04Backspace"),
                "Insert" => pecofence_core::i18n::text("键名\x04Insert"),
                "Delete" => pecofence_core::i18n::text("键名\x04Delete"),
                "Home" => pecofence_core::i18n::text("键名\x04Home"),
                "End" => pecofence_core::i18n::text("键名\x04End"),
                "PageUp" => pecofence_core::i18n::text("键名\x04PageUp"),
                "PageDown" => pecofence_core::i18n::text("键名\x04PageDown"),
                "Left" => "←",
                "Up" => "↑",
                "Right" => "→",
                "Down" => "↓",
                // Win, Alt, Tab, letters, digits, F1 to F24, Num0 to Num9.
                _ => return part,
            }
            .to_string()
        })
        .collect()
}

/// Why the settings page cannot use `h`, in the interface language.
pub(super) fn hotkey_problem_text(problem: Problem, h: Hotkey) -> String {
    match problem {
        Problem::Malformed | Problem::UnsupportedKey => {
            pecofence_core::i18n::text("这个键不能用在快捷键里，请换一个").to_string()
        }
        Problem::NoModifier => {
            pecofence_core::i18n::text("快捷键需要包含 Ctrl、Alt 或 Win").to_string()
        }
        Problem::WinAlone => pecofence_core::i18n::text(
            "Windows 保留了 Win 加单个键的组合，请再加上 Ctrl、Alt 或 Shift",
        )
        .to_string(),
        Problem::Reserved => pecofence_core::i18n::format(
            "{0} 是 Windows 常用的快捷键，请换一个",
            &[peek_hotkey_label(h)],
        ),
    }
}

/// A shortcut as the settings page shows it: its codes, key caps and label.
pub(super) fn hotkey_json(h: Hotkey) -> serde_json::Value {
    serde_json::json!({
        "mods": h.mods,
        "vk": h.vk,
        "keys": hotkey_keys(h),
        "label": peek_hotkey_label(h),
    })
}

/// Peek search: how well `query` (lower case) matches the shown name or the file name (with the
/// extension the shown name may hide): the better score, and the highlights in the shown name.
/// Shell namespace items (Recycle Bin, ...) match by their shown name only.
pub(super) fn search_match(
    name: &str,
    path: Option<&str>,
    query: &str,
) -> Option<(u32, Vec<std::ops::Range<usize>>)> {
    let shown = pecofence_core::matcher::match_name(name, query);
    let file = path
        .filter(|p| !shell::is_namespace_path(Path::new(p)))
        .and_then(|p| Path::new(p).file_name())
        .and_then(|n| pecofence_core::matcher::match_name(&n.to_string_lossy(), query));
    let score = shown
        .as_ref()
        .map(|m| m.score)
        .max(file.as_ref().map(|m| m.score))?;
    Some((score, shown.map(|m| m.ranges).unwrap_or_default()))
}

impl App {
    /// Registers / re-registers / drops the Peek hotkey to match the settings.
    pub(super) fn sync_peek_hotkey(&mut self) {
        self.sync_double_ctrl();
        // The settings page is recording a shortcut: `resume_hotkeys` syncs once it ends.
        if self.hotkeys_paused {
            return;
        }
        let peek = &self.state.config.settings.peek;
        // 「双击 Ctrl」 chosen: the combination stays in the settings but is not registered.
        let want = (peek.enabled && !peek.double_ctrl).then(|| peek.chosen_hotkey());
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
            // The chosen combination first, then the fallbacks: Windows keeps Win+Space and
            // Win+Shift+Space (older files may name them), so a fallback keeps Peek usable.
            let registered = std::iter::once(h).chain(FALLBACKS).find(|c| {
                hotkey::register(hwnd, HOTKEY_PEEK, c.mods | hotkey::mods::NOREPEAT, c.vk)
            });
            match registered {
                Some(c) => {
                    self.peek_hotkey = Some(c);
                    tracing::info!(hotkey = %c, "peek hotkey registered");
                    if c != h {
                        tracing::warn!(wanted = %h, "peek hotkey taken; using fallback");
                        // The user's choice stays in settings (it is retried on every sync);
                        // toast only when the effective combination actually changed.
                        if (Some(c) != prev || choice_changed)
                            && let Some(t) = &self.tray
                        {
                            t.show_info(
                                "PecoFence",
                                &pecofence_core::i18n::format(
                                    "快捷键 {0} 已被系统或其他程序占用，“速览栅栏”改用 {1}。",
                                    &[peek_hotkey_label(h), peek_hotkey_label(c)],
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
                                "“速览栅栏”的快捷键都被占用了，请从托盘菜单使用。",
                            ),
                            true,
                        );
                    }
                }
            }
        }
    }

    /// 「双击 Ctrl」 as Peek's shortcut: listened to while Peek is on and chosen, and not while
    /// the settings page records a shortcut (the taps would start a Peek).
    fn sync_double_ctrl(&self) {
        let peek = &self.state.config.settings.peek;
        crate::double_ctrl::set_enabled(
            peek.enabled && peek.double_ctrl && !self.hotkeys_paused,
            self.control.hwnd(),
        );
    }

    /// Whether `h` can be the Peek hotkey right now: it is ours already, or `RegisterHotKey`
    /// takes it (and it is released again at once).
    pub(super) fn peek_hotkey_free(&self, h: Hotkey) -> bool {
        self.peek_hotkey == Some(h) || self.hotkey_free(h)
    }

    /// Whether `RegisterHotKey` takes `h` (it is released again at once).
    pub(super) fn hotkey_free(&self, h: Hotkey) -> bool {
        let hwnd = self.control.hwnd();
        let free = hotkey::register(hwnd, HOTKEY_PROBE, h.mods | hotkey::mods::NOREPEAT, h.vk);
        if free {
            hotkey::unregister(hwnd, HOTKEY_PROBE);
        }
        free
    }

    /// The settings page started recording a shortcut: release ours so that pressing one
    /// reaches the page instead of starting a Peek or 「临时显示 Windows 桌面」.
    pub(super) fn pause_hotkeys(&mut self) {
        if self.hotkeys_paused {
            return;
        }
        let hwnd = self.control.hwnd();
        if self.peek_hotkey.is_some() {
            hotkey::unregister(hwnd, HOTKEY_PEEK);
        }
        if self.windows_desktop_hotkey.is_some() {
            hotkey::unregister(hwnd, HOTKEY_WINDOWS_DESKTOP);
        }
        self.hotkeys_paused = true;
        self.sync_double_ctrl();
    }

    /// Recording ended (or the settings window went away): register the shortcuts again.
    pub(super) fn resume_hotkeys(&mut self) {
        if !std::mem::take(&mut self.hotkeys_paused) {
            return;
        }
        let before = (self.peek_hotkey, self.windows_desktop_hotkey);
        let hwnd = self.control.hwnd();
        let again =
            |id: i32, h: Hotkey| hotkey::register(hwnd, id, h.mods | hotkey::mods::NOREPEAT, h.vk);
        // Another program took one meanwhile: the syncs below try again (Peek through its
        // fallbacks).
        if let Some(h) = self.peek_hotkey
            && !again(HOTKEY_PEEK, h)
        {
            self.peek_hotkey = None;
        }
        if let Some(h) = self.windows_desktop_hotkey
            && !again(HOTKEY_WINDOWS_DESKTOP, h)
        {
            self.windows_desktop_hotkey = None;
        }
        // Settings applied during the recording (pecofence-cli, say) take effect now.
        self.sync_peek_hotkey();
        self.sync_windows_desktop_hotkey();
        if (self.peek_hotkey, self.windows_desktop_hotkey) != before {
            self.push_settings_state();
        }
    }

    /// 「快捷键」 on the settings page: a recorded key press or a combination from the list
    /// (it replaces 「双击 Ctrl」). A combination that is refused or taken leaves the settings
    /// as they are and the page keeps recording.
    pub(super) fn set_peek_hotkey_from_page(&mut self, mods: u32, vk: u32) {
        let recording = self.hotkeys_paused;
        self.resume_hotkeys();
        let h = Hotkey::new(mods, vk);
        let refused = match h.check() {
            Err(problem) => Some(hotkey_problem_text(problem, h)),
            Ok(_) if self.state.config.settings.windows_desktop_hotkey() == Some(h) => {
                Some(pecofence_core::i18n::format(
                    "{0} 已经是“{1}”的快捷键，请换一个",
                    &[
                        peek_hotkey_label(h),
                        pecofence_core::i18n::text("临时显示 Windows 桌面").to_string(),
                    ],
                ))
            }
            Ok(_) if !self.peek_hotkey_free(h) => {
                tracing::info!(hotkey = %h, "peek hotkey refused: taken");
                Some(pecofence_core::i18n::format(
                    "{0} 已被 Windows 或其他程序占用，请换一个",
                    &[peek_hotkey_label(h)],
                ))
            }
            Ok(_) => None,
        };
        match &refused {
            Some(_) if recording => self.pause_hotkeys(),
            Some(_) => {}
            None => {
                let mut settings = self.state.config.settings.clone();
                settings.peek.set_hotkey(h);
                settings.peek.double_ctrl = false;
                self.apply_settings(settings, "settings");
                // The choice already in the settings changes nothing above, but while a
                // fallback stands in for it, it is free again now: switch back.
                let before = self.peek_hotkey;
                self.sync_peek_hotkey();
                if self.peek_hotkey != before {
                    self.push_settings_state();
                }
            }
        }
        if let Some(host) = &self.settings {
            host.post_json(
                &serde_json::json!({ "type": "peekHotkeyResult", "ok": refused.is_none(), "text": refused })
                    .to_string(),
            );
        }
    }

    /// `peekHotkey` of the settings state: the chosen shortcut, the one registered (`null`
    /// when Peek is off, 「双击 Ctrl」 is chosen or every combination is taken) and the quick
    /// choices.
    pub(super) fn peek_hotkey_json(&self) -> serde_json::Value {
        serde_json::json!({
            "chosen": hotkey_json(self.state.config.settings.peek.chosen_hotkey()),
            "active": self.peek_hotkey.map(hotkey_json),
            "presets": PRESETS.map(hotkey_json),
        })
    }

    /// Peek: fences float above everything (over a dimmer) until ended.
    pub(super) fn toggle_peek(&mut self) {
        if let Some(p) = self.peek.as_mut() {
            let now = Instant::now();
            if p.is_closing() {
                // Pressed again during the fade-out: dim again, nothing to rebuild. The fences
                // that were fading out fade back in on the next frame (any still held first
                // come over their window then).
                p.reopen(now);
                self.peek_raise_in = Some(0);
                window::set_timer(self.control.hwnd(), TIMER_PEEK_RAISE, PEEK_RAISE_LIMIT_MS);
                self.ctx.frames.request();
            } else {
                self.end_peek("hotkey or tray menu");
            }
            return;
        }
        if self.fences.is_empty() {
            return;
        }
        self.note_search_dialog();
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
        self.prefetch_lnk_targets();
        self.ctx.behavior.floating.set(true);
        self.redraw_for_floating();
        // A Peek that ended a moment ago may still wait to make its fences opaque.
        self.rest_peek_fades();
        let held = self.prepare_peek_fade();
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.hold_for_peek(&held);
            a.set_peek(true, &over);
        }
        if !held.is_empty() {
            self.peek_raise_in = Some(PEEK_RAISE_FRAMES);
            window::set_timer(self.control.hwnd(), TIMER_PEEK_RAISE, PEEK_RAISE_LIMIT_MS);
        }
        if animating || !held.is_empty() {
            self.ctx.frames.request();
        }
    }

    /// The fences another program's window hides completely do not pop up over it: each is made
    /// transparent here, raised `PEEK_RAISE_FRAMES` frames later (`raise_peek_held`), when
    /// that has reached the screen, and then fades in with the dimmer. A fence on visible
    /// desktop is left alone (it would blink). Returns the fences to hold back; none with
    /// animations off.
    fn prepare_peek_fade(&mut self) -> Vec<HWND> {
        if !self.ctx.motion.enabled() {
            return Vec::new();
        }
        let covered = self
            .anchor
            .borrow()
            .as_ref()
            .map(|a| a.covered_fences())
            .unwrap_or_default();
        let held: Vec<HWND> = self
            .fences
            .values()
            .filter(|w| covered.contains(&w.hwnd()) && w.peek_fade_prepare())
            .map(|w| w.hwnd())
            .collect();
        tracing::debug!(
            covered = covered.len(),
            held = held.len(),
            fences = self.fences.len(),
            "peek: fences hidden by a window fade in with the dimmer"
        );
        held
    }

    /// `PEEK_RAISE_LIMIT_MS` after the Peek opened: fences that still wait for the frame
    /// clock's beats are raised now and fade in. Nothing to do when the beats came first, when
    /// the Peek is closing or over.
    pub(super) fn peek_raise_overdue(&mut self) {
        let Some(beats_left) = self.peek_raise_in.take() else {
            return;
        };
        tracing::debug!(
            beats_left,
            limit_ms = PEEK_RAISE_LIMIT_MS,
            "peek: the frame clock is slow to beat; covered fences raised at the limit"
        );
        self.raise_peek_held();
    }

    /// Frame tick of the Peek's fence fade: counts down to the raise of the held fences, then
    /// raises them and starts the fade-in. True while it still waits.
    pub(super) fn tick_peek_raise(&mut self) -> bool {
        match self.peek_raise_in {
            None => false,
            Some(0) => {
                self.peek_raise_in = None;
                self.raise_peek_held();
                false
            }
            Some(n) => {
                self.peek_raise_in = Some(n - 1);
                true
            }
        }
    }

    /// Frame tick after a Peek: counts down to `rest_peek_fades`. True while it still waits.
    pub(super) fn tick_peek_rest(&mut self) -> bool {
        match self.peek_rest_in {
            None => false,
            Some(0) => {
                self.rest_peek_fades();
                false
            }
            Some(n) => {
                self.peek_rest_in = Some(n - 1);
                true
            }
        }
    }

    /// Every fence the Peek faded is opaque again, with its shadow.
    fn rest_peek_fades(&mut self) {
        if self.peek_rest_in.take().is_some() {
            tracing::debug!("peek: faded fences opaque again");
        }
        for w in self.fences.values() {
            w.peek_fade_rest();
        }
    }

    /// The held fences come over the windows (transparent by now) and every fence the Peek
    /// took fades in.
    fn raise_peek_held(&mut self) {
        if self.peek.as_ref().is_none_or(|p| p.is_closing()) {
            return;
        }
        let raised = self
            .anchor
            .borrow_mut()
            .as_mut()
            .is_some_and(|a| a.release_peek_held());
        if raised {
            // As after the dimmer's focus handoff: the search box stays over the fences.
            crate::peek_search::raise();
        }
        for w in self.fences.values() {
            w.peek_fade_in();
        }
    }

    /// The Peek starts to end: the fences that go back under a window fade out with the dimmer,
    /// and so do the fences it showed although they are hidden (隐藏所有栅栏): they faded only
    /// after the dimmer was gone, a second motion. Fences still held (never raised) stay as
    /// they are. Returns whether any fades.
    fn begin_peek_fade_out(&mut self) -> bool {
        self.peek_raise_in = None;
        // The anchor is asked first and let go: the fades below touch windows.
        let (covered, hidden_again): (Vec<HWND>, Vec<HWND>) = match self.anchor.borrow().as_ref() {
            Some(a) => (
                a.covered_fences()
                    .into_iter()
                    .filter(|f| !a.is_peek_held(*f))
                    .collect(),
                a.peek_shown_fences(),
            ),
            None => return false,
        };
        let fading = self
            .fences
            .values()
            .filter(|w| {
                (covered.contains(&w.hwnd()) || hidden_again.contains(&w.hwnd()))
                    && w.peek_fade_out()
            })
            .count();
        tracing::debug!(
            covered = covered.len(),
            hidden_again = hidden_again.len(),
            fading,
            "peek ending: fences that go back under a window or are hidden again fade out"
        );
        fading > 0
    }

    /// A file drag out of a fence started (`true`) or came back to a fence, the desktop or Esc
    /// (`false`) during the Peek: the dimmer fades out while it runs, so the window the file
    /// goes to shows as it is, and fades in again after it (the pointer passes through it
    /// meanwhile, see `peek::set_drag_click_through`).
    pub(super) fn peek_drag(&mut self, dragging: bool) {
        if let Some(p) = self.peek.as_mut() {
            p.set_dragging(dragging, Instant::now());
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
        crate::peek_search::raise();
        // The dimmer's focus handoff runs on another thread and can land after the search box
        // took the keyboard (double Ctrl opens both at once): give it back to the box, or
        // Enter, Backspace and the arrows would go to the dimmer.
        crate::peek_search::focus();
    }

    /// `pecofence-cli peek start --search`: Peek with `text` in its search box (started when it
    /// is not running, the text replaced when the box is open). False without fence windows.
    pub(super) fn peek_search_text(&mut self, text: &str) -> bool {
        if self.peek.as_ref().is_none_or(|p| p.is_closing()) {
            self.toggle_peek();
        }
        if self.peek.as_ref().is_none_or(|p| p.is_closing()) {
            return false;
        }
        if crate::peek_search::is_open() {
            crate::peek_search::set_text(text);
        } else {
            self.open_peek_search(text);
        }
        self.update_peek_search();
        true
    }

    /// Characters typed on the dimmer: open the search box with them, or add them to it.
    pub(super) fn peek_search_type(&mut self, text: &str) {
        if self.peek.as_ref().is_none_or(|p| p.is_closing()) {
            return;
        }
        if crate::peek_search::is_open() {
            crate::peek_search::append(text);
        } else {
            self.open_peek_search(text);
        }
        self.update_peek_search();
    }

    /// Opens the search box with `text`: in the upper middle of the screen, or beside the
    /// fences in sight when it would cover one there.
    pub(super) fn open_peek_search(&mut self, text: &str) {
        let fences = self.fence_plates(|_| true);
        let dark = self.theme_mode == ThemeMode::Dark;
        crate::peek_search::open(text, self.queue.clone(), dark, &fences);
    }

    /// Screen rectangles of the fences in sight that `pick` takes, for the search box to keep
    /// clear of. A 「桌面」 arranged like the desktop is left out: it has no plate, and its
    /// window spans all its loose icons, wherever they are.
    fn fence_plates(&self, pick: impl Fn(&FenceId) -> bool) -> Vec<RECT> {
        self.fences
            .iter()
            .filter(|(id, w)| desktop::is_visible(w.hwnd()) && !w.is_desktop_arranged() && pick(id))
            .map(|(_, w)| w.rect())
            .collect()
    }

    /// Re-runs the search for the text in the box over every fence, its hidden tabs and rolled-up
    /// fences included. The highlighted match stays if it still matches, else the first one
    /// (fences in reading order: left to right, then top down; tabs in strip order).
    pub(super) fn update_peek_search(&mut self) {
        let typed = crate::peek_search::query().unwrap_or_default();
        if typed.is_empty() {
            self.show_search_cue();
        }
        let query = typed.trim().to_lowercase();
        self.rank_fence_matches(&query);
        // The first matches of this search: a box that covers a fence holding some moves
        // aside, this once (typing on never moves it).
        if !self.search.placed && !self.peek_hits.is_empty() {
            self.search.placed = true;
            let holding =
                self.fence_plates(|id| self.peek_hits.iter().any(|(host, _, _)| host == id));
            crate::peek_search::keep_clear(&holding);
        }
        self.schedule_everything(typed.trim());
        self.refresh_search_rows();
        crate::peek_search::rows_shown_for_change();
    }

    /// Shows the matches: each fence switches to the tab holding its highlighted match, else
    /// stays on its tab if that has matches, else moves to the first tab that has some (the tab
    /// it showed before comes back once it has none, and when the search ends). Then the marks
    /// go to every fence and the match count to the box.
    pub(super) fn show_peek_hits(&mut self, searching: bool) {
        let started = std::time::Instant::now();
        self.show_peek_hits_now(searching);
        crate::peek_search::slow_stage("fence_marks", started);
    }

    fn show_peek_hits_now(&mut self, searching: bool) {
        let current = self.peek_hits.get(self.peek_hit).copied();
        let hosts: Vec<FenceId> = self.fences.keys().copied().collect();
        for host in hosts {
            let shown = self.state.active_tab_of(host);
            let before = self.peek_tabs.get(&host).copied().unwrap_or(shown);
            let tabs_with_hits: Vec<FenceId> = self
                .peek_hits
                .iter()
                .filter(|(h, _, _)| *h == host)
                .map(|&(_, tab, _)| tab)
                .collect();
            let want = match current {
                Some((h, tab, _)) if h == host => tab,
                _ if tabs_with_hits.is_empty() || tabs_with_hits.contains(&before) => before,
                _ => tabs_with_hits[0],
            };
            if want != shown {
                self.peek_tabs.entry(host).or_insert(before);
                self.switch_tab(host, want);
            }
            if want == before {
                self.peek_tabs.remove(&host);
            }
        }
        for (id, w) in &self.fences {
            let shown = self.state.active_tab_of(*id);
            let marks = searching.then(|| {
                let mut tab_hits: HashMap<FenceId, usize> = HashMap::new();
                for &(_, tab, _) in self.peek_hits.iter().filter(|(host, _, _)| host == id) {
                    *tab_hits.entry(tab).or_default() += 1;
                }
                SearchMarks {
                    matches: self
                        .peek_hits
                        .iter()
                        .filter(|(host, tab, _)| host == id && *tab == shown)
                        .map(|&(_, _, item)| item)
                        .collect(),
                    current: current
                        .filter(|(host, _, _)| host == id)
                        .map(|(_, _, item)| item),
                    tab_hits,
                }
            });
            w.set_search_marks(marks);
        }
        let rows = self
            .search
            .rows
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    super::search::RowKind::ShowAll | super::search::RowKind::ToDesktop
                )
            })
            .count();
        // The bottom bar says how many there are; the box says it only when there are none.
        let status = if searching && rows == 0 {
            pecofence_core::i18n::text("没有匹配的文件").to_string()
        } else {
            String::new()
        };
        crate::peek_search::set_status(&status);
    }

    /// Closes the search box, puts back the tabs it switched and draws every fence as usual
    /// again (fences it expanded roll back up).
    fn end_peek_search(&mut self) {
        crate::peek_search::close();
        self.peek_hits.clear();
        self.peek_hit = 0;
        let (dialog, before) = (self.search.dialog, self.search.before);
        let waited = self.search.asked_at.is_some();
        self.search = Default::default();
        // Smart fences that waited for an answer the closed search box will not get.
        if waited {
            self.ask_next_smart();
        }
        // What was in front stays noted until the next Peek notes it again.
        self.search.dialog = dialog;
        self.search.before = before;
        window::kill_timer(self.control.hwnd(), TIMER_EVERYTHING);
        for (host, tab) in std::mem::take(&mut self.peek_tabs) {
            if self.state.active_tab_of(host) != tab {
                self.switch_tab(host, tab);
            }
        }
        for w in self.fences.values() {
            w.set_search_marks(None);
        }
    }

    /// Ends the peek the way the user sees it: the dimmer fades out (83 ms) while the fences
    /// stay in the topmost band, then `end_peek_now` runs from the frame tick. Repeated calls
    /// during the fade are no-ops. `reason` (a fixed text) goes to the log.
    pub(super) fn end_peek(&mut self, reason: &'static str) {
        let now = Instant::now();
        let Some(p) = self.peek.as_mut() else {
            return;
        };
        if p.is_closing() {
            return;
        }
        // Windows 10 can make the desktop the foreground window about a second after a click
        // on it that came just before the Peek's shortcut. That click was no move away from the
        // Peek (the dimmer covers the desktop, so a click during the Peek never reaches it):
        // keep the Peek and take the keyboard back.
        let age = p.age(now);
        if PEEK_FOCUS_LOST.contains(&reason) && age < PEEK_LATE_DESKTOP {
            let front = desktop::foreground_window();
            if self
                .anchor
                .borrow()
                .as_ref()
                .is_some_and(|a| a.is_desktop_window(front))
            {
                tracing::info!(
                    reason,
                    ms = age.as_millis() as u64,
                    "desktop came to the front just after the peek began; peek kept"
                );
                p.take_focus(self.control.hwnd());
                return;
            }
        }
        tracing::info!(reason, "peek ending");
        if !self.ctx.motion.enabled() {
            self.end_peek_now();
            return;
        }
        let fences_fade = self.begin_peek_fade_out();
        let Some(p) = self.peek.as_mut() else {
            return;
        };
        p.begin_close(now, fences_fade);
        let finished = p.close_finished(now);
        self.end_peek_search();
        if finished {
            self.end_peek_now();
        } else {
            self.ctx.frames.request();
        }
    }

    /// Ends the peek at once (layout rebuilds, config swaps, shutdown).
    /// After `floating` changed: only the fences that float differently are redrawn. Redrawing
    /// every fence took 15 to 20 ms in the last frame of the Peek's fade out on a 4K screen.
    fn redraw_for_floating(&self) {
        let started = Instant::now();
        let redrawn = self
            .fences
            .values()
            .filter(|w| w.redraw_for_floating())
            .count();
        tracing::debug!(
            floating = self.ctx.behavior.floating.get(),
            redrawn,
            fences = self.fences.len(),
            ms = crate::share_log::millis(started.elapsed()),
            "fences redrawn for the Peek"
        );
    }

    pub(super) fn end_peek_now(&mut self) {
        if self.peek.is_none() {
            return;
        }
        let started = Instant::now();
        self.end_peek_search();
        let searched = Instant::now();
        self.ctx.behavior.floating.set(false);
        self.redraw_for_floating();
        let redrawn = Instant::now();
        if let Some(a) = self.anchor.borrow_mut().as_mut() {
            a.set_peek(false, &[]);
        }
        // The fences the Peek faded are back under the windows: they become opaque again a few
        // frames later (`PEEK_REST_FRAMES`), at once when nothing animates.
        self.peek_raise_in = None;
        if self.ctx.motion.enabled() && self.fences.values().any(|w| w.peek_faded()) {
            self.peek_rest_in = Some(PEEK_REST_FRAMES);
            self.ctx.frames.request();
        } else {
            self.rest_peek_fades();
        }
        let anchored = Instant::now();
        // Dropping the windows destroys them (after the fences left the topmost band).
        self.peek = None;
        // The dimmer had the keyboard and passed the keys on to a fence (Tab, the arrows), so
        // that fence gets no WM_KILLFOCUS now: its focus ring goes with the Peek. It stayed on
        // an icon while the keys went to the window that came to the front.
        let front = desktop::foreground_window();
        let rings = self
            .fences
            .values()
            .filter(|w| w.hwnd() != front && w.keyboard_left())
            .count();
        let ms = crate::share_log::millis;
        tracing::debug!(
            search_ms = ms(searched - started),
            redraw_ms = ms(redrawn - searched),
            zorder_ms = ms(anchored - redrawn),
            dimmer_ms = ms(anchored.elapsed()),
            rings,
            "peek ended"
        );
    }
}
