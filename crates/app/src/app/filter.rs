//! 筛选 in a folder fence or a smart fence: what the fence shows narrows to the names that match
//! the typed text, matched as the Peek search matches names (pinyin too, without its loosest
//! letters-in-order match). The files stay as they are; the filter is not saved, and going
//! into another folder clears it, as Explorer's search box does.

use super::*;
use crate::filter_box;

/// Whether `item` stays in a fence filtered by `query` (lower case, trimmed).
pub(super) fn filter_keeps(item: &pecofence_core::Item, query: &str) -> bool {
    match peek::search_match(&item.display_name, item.path(), query) {
        Some((score, _)) => !pecofence_core::matcher::is_loose_score(score),
        None => false,
    }
}

impl App {
    /// Fences that can be filtered: those showing a folder, and smart fences.
    pub(super) fn filterable(&self, id: FenceId) -> bool {
        self.state.portal_path(id).is_some() || self.state.smart_spec(id).is_some()
    }

    /// The filter of fence `id` for matching (lower case, trimmed); None when it has none.
    pub(super) fn fence_filter(&self, id: FenceId) -> Option<String> {
        self.fence_filters
            .get(&id)
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
    }

    /// Ctrl+F or 筛选… in fence `fence`: opens its filter box over the title row.
    pub(super) fn open_fence_filter(&mut self, fence: FenceId, why: &str) {
        if !self.filterable(fence) {
            tracing::info!(%fence, why, "filter: this fence shows no folder");
            return;
        }
        let host = self.state.host_of(fence);
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        tracing::info!(%fence, why, filtered = self.fence_filter(fence).is_some(), "filter opened");
        let title = self
            .state
            .fence(fence)
            .map(|f| self.state.display_title(f))
            .unwrap_or_default();
        let cue = pecofence_core::i18n::format("筛选「{0}」", &[title]);
        let current = self.fence_filters.get(&fence).cloned().unwrap_or_default();
        filter_box::open(
            w.hwnd(),
            fence,
            &current,
            &cue,
            self.queue.clone(),
            self.theme_mode,
        );
    }

    /// The filter of fence `fence` is now `text` (empty: none): the fence shows what matches.
    pub(super) fn set_fence_filter(&mut self, fence: FenceId, text: String, why: &str) {
        let before = self.fence_filter(fence);
        if text.is_empty() {
            self.fence_filters.remove(&fence);
        } else {
            self.fence_filters.insert(fence, text);
        }
        if self.fence_filter(fence) == before {
            return;
        }
        self.refresh_fence(fence);
        let shown = self
            .state
            .fence(fence)
            .map_or(0, |f| self.item_views(f).len());
        tracing::info!(%fence, why, chars = self.fence_filter(fence).map_or(0, |t| t.chars().count()), shown, "filter changed");
    }

    /// Drops fence `fence`'s filter (another folder shown, the fence deleted).
    pub(super) fn clear_fence_filter(&mut self, fence: FenceId, why: &str) {
        if filter_box::active_fence() == Some(fence) {
            filter_box::close();
        }
        if self.fence_filters.remove(&fence).is_some() {
            tracing::info!(%fence, why, "filter cleared");
        }
    }

    /// The title of a filtered fence: `<title> · 筛选：<text>`; the plain title while its box
    /// shows the text.
    pub(super) fn filter_title(&self, id: FenceId, title: String) -> String {
        if filter_box::active_fence() == Some(id) {
            return title;
        }
        match self.fence_filters.get(&id).map(|t| t.trim()) {
            Some(t) if !t.is_empty() => {
                pecofence_core::i18n::format("{0} · 筛选：{1}", &[title, t.to_string()])
            }
            _ => title,
        }
    }
}
