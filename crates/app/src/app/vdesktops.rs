//! 在哪个虚拟桌面显示: a fence shows on every Windows virtual desktop (the default) or only on
//! chosen ones. PecoFence follows the system's switches (Win+Ctrl+←/→, Task View) and never
//! switches desktops itself: on a switch the fences of the old desktop fade out and those of
//! the new one fade in, with quick hide's fade. A window with tabs follows its host fence, and
//! a window bound to some desktops shows the Task View glyph before its title, the desktops'
//! names in its infotip. 「每个虚拟桌面各用一套栅栏」 (`perDesktopFences`) makes new fences
//! belong to the desktop they are made on.
//!
//! The current desktop, the desktops' order and their names come from Explorer's
//! `VirtualDesktops` registry key (`wallpaper::desktop_id`, `wallpaper::virtual_desktops`), an
//! undocumented detail: a switch is noticed from the registry watch, a foreground change, or a
//! 100 ms poll that runs while some fence is bound to a desktop or the switch is on (the Store
//! edition gets no registry notifications, and Windows 10 keeps the current desktop in a key
//! the watch does not cover). The desktops seen last are saved (`Config.virtual_desktops`), so
//! fences follow a desktop that was closed or a whole new set (`pecofence_core::vdesk`).
//!
//! 「桌面」 (the inbox) always shows on every desktop: new desktop items land there.

use super::*;
use pecofence_core::vdesk::{self, DesktopRef};

/// The fence window poll for the current virtual desktop (see the module docs).
pub(super) const TIMER_VD_POLL: usize = 74;
const VD_POLL_MS: u32 = 100;

impl App {
    /// The current virtual desktop's key, `None` while Windows names none (one desktop).
    pub(super) fn read_current_desktop() -> Option<String> {
        wallpaper::desktop_id().map(|id| wallpaper::desktop_key(&id))
    }

    fn read_desktops() -> Vec<DesktopRef> {
        wallpaper::virtual_desktops()
            .into_iter()
            .map(|(id, name)| DesktopRef { id, name })
            .collect()
    }

    /// The current desktop for the menu, the page and the CLI: the one followed while the poll
    /// runs, else read now (no switch is followed then, so the one kept may be old).
    fn current_desktop_now(&self) -> Option<String> {
        if self.desktop_poll {
            self.state.current_desktop.clone()
        } else {
            let now = Self::read_current_desktop();
            if now != self.state.current_desktop {
                tracing::debug!(
                    kept = ?self.state.current_desktop,
                    now = ?now,
                    "current virtual desktop read afresh (no switch is followed)"
                );
            }
            now
        }
    }

    /// The desktops in Task View order as the menu, the page and the CLI show them: key, label
    /// (its Task View name, or 桌面 N), whether it is the current one.
    pub(super) fn desktop_labels(&self) -> Vec<(String, String, bool)> {
        let current = self.current_desktop_now();
        Self::read_desktops()
            .into_iter()
            .enumerate()
            .map(|(i, d)| {
                let label = d.name.unwrap_or_else(|| {
                    pecofence_core::i18n::format("桌面 {0}", &[(i + 1).to_string()])
                });
                let current = current
                    .as_deref()
                    .is_some_and(|c| c.eq_ignore_ascii_case(&d.id));
                (d.id, label, current)
            })
            .collect()
    }

    /// A desktop signal (registry watch, foreground change, poll): when the current virtual
    /// desktop changed, the fences fade in and out for it; when the desktops changed (one
    /// closed, renamed, a new set), the fences follow and the glyphs' infotips are renewed.
    pub(super) fn check_virtual_desktop(&mut self, source: &'static str) {
        if !self.desktop_poll || (self.vd_poll_only && source != "poll") {
            return;
        }
        let now = Self::read_current_desktop();
        // The list is read whole (names too) only when its ids changed, or the registry says
        // something in the key changed (a rename).
        let keys = wallpaper::virtual_desktop_keys();
        let same_ids = keys.len() == self.state.config.virtual_desktops.len()
            && keys
                .iter()
                .zip(&self.state.config.virtual_desktops)
                .all(|(k, d)| k.eq_ignore_ascii_case(&d.id));
        let mut changed =
            (!same_ids || source == "registry") && self.follow_desktops(now.as_deref());
        if now != self.state.current_desktop {
            tracing::info!(
                from = ?self.state.current_desktop,
                to = ?now,
                source,
                "virtual desktop switched"
            );
            self.state.current_desktop = now;
            changed = true;
        }
        if changed {
            self.apply_desktop_visibility("virtual desktops changed");
            // The page marks the current desktop and lists the names.
            self.push_settings_state();
        }
    }

    /// Whether the window of `fence` (its host's, for a tab) belongs on the current desktop.
    pub(super) fn on_current_desktop(&self, fence: FenceId) -> bool {
        self.state.on_current_desktop(fence)
    }

    /// Whether the windows of `a` and `b` are ever on screen together: one shows on every
    /// desktop, or they share one.
    pub(super) fn share_a_desktop(&self, a: FenceId, b: FenceId) -> bool {
        let desktops = |id| {
            self.state
                .fence(self.state.host_of(id))
                .map(|f| f.virtual_desktops.clone())
                .unwrap_or_default()
        };
        let (a, b) = (desktops(a), desktops(b));
        a.is_empty()
            || b.is_empty()
            || a.iter()
                .any(|x| b.iter().any(|y| x.eq_ignore_ascii_case(y)))
    }

    /// The infotip of the Task View glyph of `host`'s window: the names of its desktops, `None`
    /// for a window on every desktop (or one whose desktops Windows does not list).
    fn desktop_tip(&self, host: FenceId, labels: &[(String, String, bool)]) -> Option<String> {
        let f = self.state.fence(host)?;
        let names: Vec<&str> = labels
            .iter()
            .filter(|(key, _, _)| {
                f.virtual_desktops
                    .iter()
                    .any(|d| d.eq_ignore_ascii_case(key))
            })
            .map(|(_, label, _)| label.as_str())
            .collect();
        (!names.is_empty()).then(|| {
            pecofence_core::i18n::format(
                "只在“{0}”上显示",
                &[names.join(pecofence_core::i18n::text("、"))],
            )
        })
    }

    /// Hides the windows of fences that belong to other desktops and shows the ones that came
    /// back, unless quick hide (or 「临时显示 Windows 桌面」) holds them; puts the Task View
    /// glyph on the windows bound to some desktops. Starts or stops the poll as fences get bound
    /// or unbound.
    pub(super) fn apply_desktop_visibility(&mut self, reason: &'static str) {
        self.sync_desktop_poll();
        let labels = if self.desktop_poll {
            self.desktop_labels()
        } else {
            Vec::new()
        };
        let mut hidden = Vec::new();
        let mut shown = Vec::new();
        // Back on this desktop, but quick hide keeps them out of sight for now.
        let mut held = Vec::new();
        for (id, w) in &self.fences {
            w.set_desktop_mark(self.desktop_tip(*id, &labels));
            let off = !self.on_current_desktop(*id);
            if !w.set_off_desktop(off) {
                continue;
            }
            let title = self
                .state
                .fence(*id)
                .map(|f| f.title.clone())
                .unwrap_or_default();
            if off {
                hidden.push(title);
                continue;
            }
            let quick_hidden = self
                .anchor
                .borrow()
                .as_ref()
                .is_some_and(|a| a.quick_hides(w.hwnd(), self.windows_desktop));
            if quick_hidden {
                held.push(title);
            } else {
                w.show(false);
                shown.push(title);
            }
        }
        if hidden.is_empty() && shown.is_empty() && held.is_empty() {
            return;
        }
        tracing::info!(
            desktop = ?self.state.current_desktop,
            ?hidden,
            ?shown,
            ?held,
            reason,
            "fences faded for the virtual desktop"
        );
        if !shown.is_empty()
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.reanchor("fences of this virtual desktop shown");
        }
        // The loose icons arranged like the desktop flow around the fences that show here.
        self.arrange_desktop_fence();
    }

    /// The poll runs while some fence is bound to a desktop or 「每个虚拟桌面各用一套栅栏」 is
    /// on; when it starts, the current desktop is read afresh (no signal was followed until
    /// then) and the fences follow what changed since the desktops were seen last.
    fn sync_desktop_poll(&mut self) {
        let bound = self
            .state
            .fences()
            .iter()
            .any(|f| !f.virtual_desktops.is_empty());
        let want = bound || self.state.config.settings.per_desktop_fences;
        // Also at startup: a list left from earlier use goes once nothing needs it. A snapshot's
        // bound fence still needs it to follow when it is restored.
        if !want && !self.state.config.uses_virtual_desktops() {
            self.forget_desktop_list("no fence is bound and the switch is off");
        }
        if want == self.desktop_poll {
            return;
        }
        self.desktop_poll = want;
        if want {
            let current = Self::read_current_desktop();
            self.follow_desktops(current.as_deref());
            self.state.current_desktop = current;
            window::set_coalescable_timer(self.control.hwnd(), TIMER_VD_POLL, VD_POLL_MS, 25);
        } else {
            window::kill_timer(self.control.hwnd(), TIMER_VD_POLL);
        }
        tracing::info!(
            on = want,
            bound,
            switch = self.state.config.settings.per_desktop_fences,
            desktop = ?self.state.current_desktop,
            desktops = self.state.config.virtual_desktops.len(),
            "virtual desktop poll"
        );
    }

    /// Takes in the desktops Windows lists now (saved for the next start). Fences bound to a
    /// desktop that is gone, in every monitor layout, follow it ([`vdesk::follow`]: the left
    /// neighbour of a closed one, the same name or position in a new set) or show on every
    /// desktop when nothing tells; that also catches bindings brought back by a snapshot or an
    /// import while the list stayed the same. A list that does not hold the current desktop is
    /// taken for one Explorer is still writing, and changes nothing. Returns whether anything
    /// changed.
    fn follow_desktops(&mut self, current: Option<&str>) -> bool {
        // Nothing bound anywhere and the switch off: the file keeps no desktop list (and no Task
        // View names) for someone who does not use the feature.
        if !self.state.config.uses_virtual_desktops() {
            self.forget_desktop_list("the feature is not in use");
            return false;
        }
        let now = Self::read_desktops();
        let consistent = current.is_some_and(|c| now.iter().any(|d| d.id.eq_ignore_ascii_case(c)));
        if now.is_empty() || !consistent {
            return false;
        }
        let old = self.state.config.virtual_desktops.clone();
        let listed_changed = now != old;
        if listed_changed {
            self.state.config.virtual_desktops = now.clone();
            self.state.mark_dirty();
            tracing::info!(
                before = old.len(),
                now = now.len(),
                ids = ?now.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
                "virtual desktops listed"
            );
        }
        let mut moved = 0;
        for layout in &mut self.state.config.layouts {
            for f in &mut layout.fences {
                let Some((keep, gone)) = vdesk::follow(&f.virtual_desktops, &old, &now) else {
                    continue;
                };
                for g in &gone {
                    tracing::info!(
                        fence = %f.id,
                        gone = %g.id,
                        to = ?g.to.as_ref().map(|(id, _)| id.as_str()),
                        why = ?g.to.as_ref().map(|(_, why)| *why),
                        "fence of a virtual desktop that is gone"
                    );
                }
                // Nothing left to name (an empty list): every desktop, rather than none.
                tracing::info!(
                    fence = %f.id,
                    from = ?f.virtual_desktops,
                    to = ?keep,
                    "fence virtual desktops followed the desktops"
                );
                f.virtual_desktops = keep;
                moved += 1;
            }
        }
        if moved > 0 {
            self.state.mark_dirty();
            // Tab pills carry the glyph of their window's binding.
            for host in self.fences.keys().copied().collect::<Vec<_>>() {
                self.refresh_fence(host);
            }
        }
        if listed_changed || moved > 0 {
            self.schedule_save();
        }
        listed_changed || moved > 0
    }

    /// Drops the saved desktop list (Config.virtual_desktops) while the feature is not in use.
    fn forget_desktop_list(&mut self, reason: &'static str) {
        if self.state.config.virtual_desktops.is_empty() {
            return;
        }
        tracing::info!(
            desktops = self.state.config.virtual_desktops.len(),
            reason,
            "virtual desktop list dropped from the configuration"
        );
        self.state.config.virtual_desktops.clear();
        self.state.mark_dirty();
        self.schedule_save();
    }

    /// A snapshot restore, an import or a backup restore replaced the fences: those bound to
    /// desktops Windows no longer lists follow the desktops now, not at the next list change.
    pub(super) fn follow_desktops_after(&mut self, reason: &'static str) {
        let current = Self::read_current_desktop();
        let changed = self.follow_desktops(current.as_deref());
        tracing::info!(
            reason,
            changed,
            desktop = ?current,
            "virtual desktops checked after the fences were replaced"
        );
        if self.desktop_poll {
            self.state.current_desktop = current;
        }
    }

    /// Binds the window of `fence` (every tab of it) to `desktops` (empty: every desktop) and
    /// fades it out or in for the current one. Returns whether anything changed.
    pub(super) fn set_fence_desktops(
        &mut self,
        fence: FenceId,
        desktops: Vec<String>,
        reason: &'static str,
    ) -> bool {
        let host = self.state.host_of(fence);
        // 「桌面」 (and a window holding it as a tab) shows on every desktop: new desktop items
        // land in it.
        if !desktops.is_empty() && self.state.window_holds_inbox(host) {
            tracing::info!(%fence, %host, ?desktops, reason, "virtual desktops refused: the window holds the inbox");
            return false;
        }
        let before = self.state.fence(host).map(|f| f.virtual_desktops.clone());
        if before.as_ref() == Some(&desktops) {
            tracing::info!(%fence, ?desktops, reason, "fence virtual desktops unchanged");
            return false;
        }
        tracing::info!(
            %host,
            from = ?before,
            to = ?desktops,
            current = ?self.state.current_desktop,
            reason,
            "fence virtual desktops set"
        );
        self.state.set_virtual_desktops(host, desktops);
        // The tab pills' glyphs, then the window's.
        self.refresh_fence(host);
        self.apply_desktop_visibility(reason);
        // Picked on one desktop for another: it is gone from here, say where it went (a CLI
        // caller reads the reply instead).
        if reason != "CLI"
            && !self.on_current_desktop(host)
            && let (Some(t), Some(f)) = (&self.tray, self.state.fence(host))
        {
            let names: Vec<String> = self
                .desktop_labels()
                .into_iter()
                .filter(|(key, _, _)| {
                    f.virtual_desktops
                        .iter()
                        .any(|d| d.eq_ignore_ascii_case(key))
                })
                .map(|(_, label, _)| label)
                .collect();
            t.show_info(
                "PecoFence",
                &pecofence_core::i18n::format(
                    "栅栏“{0}”现在只在“{1}”上显示，切换到那个虚拟桌面就能看到它。",
                    &[
                        f.title.clone(),
                        names.join(pecofence_core::i18n::text("、")),
                    ],
                ),
                false,
            );
        }
        self.schedule_save();
        true
    }

    /// `fence set <fence> virtualDesktops …`: `all`, `current`, or desktops by number (Task
    /// View order, from 1), name or `{GUID}`, as a list or one comma-separated string.
    pub(super) fn resolve_desktop_specs(
        &self,
        specs: &[String],
    ) -> std::result::Result<Vec<String>, String> {
        let desktops = self.desktop_labels();
        let mut out: Vec<String> = Vec::new();
        for spec in specs.iter().flat_map(|s| s.split(',')).map(str::trim) {
            let key = if spec.eq_ignore_ascii_case("all") {
                return Ok(Vec::new());
            } else if spec.eq_ignore_ascii_case("current") {
                self.current_desktop_now()
            } else if let Ok(n) = spec.parse::<usize>() {
                n.checked_sub(1)
                    .and_then(|i| desktops.get(i))
                    .map(|d| d.0.clone())
            } else {
                let found: Vec<&String> = desktops
                    .iter()
                    .filter(|(key, label, _)| {
                        key.eq_ignore_ascii_case(spec)
                            || key.trim_matches(['{', '}']).eq_ignore_ascii_case(spec)
                            || label.eq_ignore_ascii_case(spec)
                    })
                    .map(|d| &d.0)
                    .collect();
                if found.len() > 1 {
                    return Err(format!(
                        "{} virtual desktops are named {spec:?}; give the number instead",
                        found.len()
                    ));
                }
                found.first().map(|k| (*k).clone())
            };
            let Some(key) = key else {
                return Err(format!(
                    "no virtual desktop {spec:?}; there are {}",
                    desktops.len()
                ));
            };
            if !out.iter().any(|k| k.eq_ignore_ascii_case(&key)) {
                out.push(key);
            }
        }
        // Every desktop named one by one is every desktop.
        if out.len() >= desktops.len() && desktops.len() > 1 {
            return Ok(Vec::new());
        }
        Ok(out)
    }
}
