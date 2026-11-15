//! 插件的轻量层: what a plugin fence shows. A plugin's view (`pecofence_plugin_host::view`)
//! becomes script fence rows (`ScriptRowView`), which the fence draws as a script fence's.

use super::*;
use crate::fence_window::{ScriptRowView, ScriptView};
use pecofence_plugin_host as host;

impl App {
    /// The rows fence `fence`'s plugin `view` turns into.
    pub(super) fn plugin_rows(&mut self, _fence: FenceId, view: host::View) -> Vec<ScriptRowView> {
        view.rows
            .into_iter()
            .map(|r| ScriptRowView {
                text: r.text,
                color: r.color,
                size: r.size,
                short: r.short,
                bold: r.bold,
                ..ScriptRowView::default()
            })
            .collect()
    }

    /// A light plugin fence's view: its rows (正在运行… until the first), or why it stopped.
    pub(super) fn light_view(&self, fence: FenceId) -> Option<ScriptView> {
        let stopped = self.light_stopped_rows(fence);
        let rows = stopped.or_else(|| self.light.running.get(&fence)?.0.clone());
        let quiet = rows.is_none() && self.light.running.contains_key(&fence);
        (quiet || rows.is_some()).then(|| ScriptView {
            rows: rows.unwrap_or_default(),
            quiet,
            ..ScriptView::default()
        })
    }
}
