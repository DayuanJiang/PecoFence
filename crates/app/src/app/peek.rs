//! Peek: hotkey registration with fallbacks, the shortcut field on the settings page, toggle /
//! end, and the search across all fences (`peek_search.rs` is its box).

use super::*;
use pecofence_core::hotkey::{FALLBACKS, PRESETS, Problem};

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

/// Peek search: the shown name or the file name (with the extension the shown name may hide)
/// contains `query`, which is lower case. Shell namespace items (Recycle Bin, ...) match by
/// their shown name only.
pub(super) fn search_matches(name: &str, path: Option<&str>, query: &str) -> bool {
    name.to_lowercase().contains(query)
        || path
            .filter(|p| !shell::is_namespace_path(Path::new(p)))
            .and_then(|p| Path::new(p).file_name())
            .is_some_and(|n| n.to_string_lossy().to_lowercase().contains(query))
}

impl App {
    /// Registers / re-registers / drops the Peek hotkey to match the settings.
    pub(super) fn sync_peek_hotkey(&mut self) {
        // The settings page is recording a shortcut: `resume_hotkeys` syncs once it ends.
        if self.hotkeys_paused {
            return;
        }
        let peek = &self.state.config.settings.peek;
        let want = peek.enabled.then(|| peek.chosen_hotkey());
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

    /// 「快捷键」 on the settings page: a recorded key press or a quick choice. A combination
    /// that is refused or taken leaves the settings as they are and the page keeps recording.
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
                self.apply_settings(settings);
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
    /// when Peek is off or every combination is taken) and the quick choices.
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
                // Pressed again during the fade-out: dim again, nothing to rebuild.
                p.reopen(now);
                self.ctx.frames.request();
            } else {
                self.end_peek("hotkey or tray menu");
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
    }

    /// Characters typed on the dimmer: open the search box with them, or add them to it.
    pub(super) fn peek_search_type(&mut self, text: &str) {
        if self.peek.as_ref().is_none_or(|p| p.is_closing()) {
            return;
        }
        if crate::peek_search::is_open() {
            crate::peek_search::append(text);
        } else {
            crate::peek_search::open(text, self.queue.clone(), self.theme_mode == ThemeMode::Dark);
        }
        self.update_peek_search();
    }

    /// Re-runs the search for the text in the box over every fence, its hidden tabs and rolled-up
    /// fences included. The highlighted match stays if it still matches, else the first one
    /// (fences in reading order: left to right, then top down; tabs in strip order).
    pub(super) fn update_peek_search(&mut self) {
        let query = crate::peek_search::query()
            .map(|q| q.trim().to_lowercase())
            .unwrap_or_default();
        let previous = self.peek_hits.get(self.peek_hit).copied();
        let mut hits = Vec::new();
        if !query.is_empty() {
            let mut windows: Vec<(RECT, FenceId)> =
                self.fences.iter().map(|(id, w)| (w.rect(), *id)).collect();
            windows.sort_by_key(|(r, id)| (r.left, r.top, *id));
            for (_, host) in windows {
                for tab in self.state.tabs_of(host) {
                    let Some(fence) = self.state.fence(tab) else {
                        continue;
                    };
                    for it in self.state.items_of(fence) {
                        if search_matches(&it.display_name, it.key.as_path(), &query) {
                            hits.push((host, tab, it.id));
                        }
                    }
                }
            }
        }
        self.peek_hits = hits;
        self.peek_hit = previous
            .and_then(|p| self.peek_hits.iter().position(|h| *h == p))
            .unwrap_or(0);
        self.show_peek_hits(!query.is_empty());
    }

    /// Up / Down in the search box: the previous / next match, wrapping around.
    pub(super) fn step_peek_search(&mut self, step: i32) {
        let n = self.peek_hits.len() as i32;
        if n == 0 {
            return;
        }
        self.peek_hit = (self.peek_hit as i32 + step).rem_euclid(n) as usize;
        self.show_peek_hits(true);
    }

    /// Enter in the search box: open the highlighted match and end the Peek. A folder opens in
    /// Explorer: shown inside its fence, it would sit behind the windows the Peek uncovered.
    pub(super) fn open_peek_hit(&mut self) {
        let Some(&(_, tab, item)) = self.peek_hits.get(self.peek_hit) else {
            return;
        };
        self.end_peek("search match opened");
        self.launch(item, tab, false);
    }

    /// Shows the matches: each fence switches to the tab holding its highlighted match, else
    /// stays on its tab if that has matches, else moves to the first tab that has some (the tab
    /// it showed before comes back once it has none, and when the search ends). Then the marks
    /// go to every fence and the match count to the box.
    fn show_peek_hits(&mut self, searching: bool) {
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
        let status = if !searching {
            String::new()
        } else if self.peek_hits.is_empty() {
            pecofence_core::i18n::text("没有匹配的文件").to_string()
        } else {
            format!("{} / {}", self.peek_hit + 1, self.peek_hits.len())
        };
        crate::peek_search::set_status(&status);
    }

    /// Closes the search box, puts back the tabs it switched and draws every fence as usual
    /// again (fences it expanded roll back up).
    fn end_peek_search(&mut self) {
        crate::peek_search::close();
        self.peek_hits.clear();
        self.peek_hit = 0;
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
        tracing::info!(reason, "peek ending");
        if !self.ctx.motion.enabled() {
            self.end_peek_now();
            return;
        }
        p.begin_close(now);
        let finished = p.close_finished(now);
        self.end_peek_search();
        if finished {
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
        self.end_peek_search();
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
