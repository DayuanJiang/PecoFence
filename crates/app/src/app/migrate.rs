//! 升级为插件: the tools of earlier versions become fences of the plugins that do the same now.
//! Whenever a configuration comes in (start, import, backup, snapshot), its note fences become
//! `notes` fences, its clocks `clock` fences, and script fences running a built-in example
//! nobody edited become `weather`, `github` or `pc-status` fences. The plugins are installed
//! from the store in the background first (they are the user's own tools: no warning); a fence
//! moves only once its plugin is installed and checked, and until then it stays as it was and
//! works. It runs again at every chance, and a second run finds nothing to do.

use super::App;
use pecofence_core::{BEFORE_PLUGINS, i18n};

impl App {
    /// Moves the fences whose plugin is installed. `fetch`: the missing plugins are installed
    /// (the index is asked for first when there is none); their arrival calls this again.
    pub(super) fn migrate_tools(&mut self, fetch: bool) {
        // A test instance moves its tools only from a test index: other steps files test the
        // tools themselves.
        if self.instance.is_some() && pecofence_core::brand::var("PECOFENCE_PLUGINS_URL").is_err() {
            tracing::debug!("upgrade to plugins: skipped, a test instance without a test index");
            return;
        }
        let moves = (self.state.config).plugin_moves(crate::scripts::example_plugin);
        if moves.is_empty() {
            return;
        }
        let mut plugins: Vec<&'static str> = moves.iter().map(|m| m.plugin).collect();
        plugins.sort_unstable();
        plugins.dedup();
        let (ready, missing): (Vec<&str>, Vec<&str>) = (plugins.into_iter()).partition(|id| {
            self.plugin(id)
                .is_some_and(|p| p.ready && p.disabled.is_none())
        });
        // A note being edited moves once its editor closes (`note_edited` calls again).
        let editing = crate::note_edit::open_for();
        let moving: Vec<_> = (moves.iter())
            .filter(|m| ready.contains(&m.plugin) && Some(m.fence) != editing)
            .collect();
        tracing::info!(
            fences = moves.len(),
            moving = moving.len(),
            ?ready,
            ?missing,
            fetch,
            editing = editing.is_some(),
            "upgrade to plugins: tool fences of an earlier version"
        );
        if fetch && !missing.is_empty() {
            self.install_for_migration(&missing);
        }
        if moving.is_empty() || !self.snapshot_before_plugins() {
            return;
        }
        for m in &moving {
            self.forget_script(m.fence);
            self.state.config.move_to_plugin(m);
            tracing::info!(fence = %m.fence, plugin = m.plugin, "upgrade to plugins: fence moved");
        }
        self.state.mark_dirty();
        self.resync_windows();
        for m in &moving {
            if self.state.fence(m.fence).is_some() {
                self.start_script(m.fence);
            }
        }
        self.refresh_tool_views();
        self.schedule_save();
    }

    /// After a snapshot restore. 升级为插件之前 is the way back to the tools as they were: its
    /// fences stay as they are until the configuration comes in again (the next start).
    pub(super) fn migrate_after_restore(&mut self, snapshot: uuid::Uuid) {
        if snapshot == BEFORE_PLUGINS {
            tracing::info!(
                "upgrade to plugins: 升级为插件之前 restored, its fences stay as they are"
            );
            return;
        }
        self.migrate_tools(true);
    }

    /// 升级为插件之前, saved before the first fence moves. False when the list is full of
    /// snapshots the user saved (the fences wait as they are).
    fn snapshot_before_plugins(&mut self) -> bool {
        let snapshots = &mut self.state.config.snapshots;
        if snapshots.iter().any(|s| s.id == BEFORE_PLUGINS) {
            return true;
        }
        if !super::settings::snapshot_room(snapshots, None) {
            tracing::warn!("upgrade to plugins: waits, the snapshots are all the user's own");
            return false;
        }
        let id = self.state.save_snapshot(i18n::text("升级为插件之前"));
        // Restoring it brings the note texts and scripts back with the fences.
        self.state.keep_tool_data_in(id);
        if let Some(s) = self.state.config.snapshots.iter_mut().find(|s| s.id == id) {
            s.id = BEFORE_PLUGINS;
        }
        tracing::info!("upgrade to plugins: snapshot 升级为插件之前 saved");
        true
    }
}
