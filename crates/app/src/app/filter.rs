//! 筛选 in a folder fence or a smart fence: what the fence shows narrows to the names that match
//! the typed text, matched as the Peek search matches names (pinyin too, without its loosest
//! letters-in-order match). The files stay as they are; the filter is not saved, and going
//! into another folder clears it, as Explorer's search box does.

use super::*;
use crate::filter_box;

/// Whether `item` stays in a fence filtered by `query` (lower case, trimmed): of its 类型 (the
/// `kind:` words) and matching the rest.
pub(super) fn filter_keeps(item: &pecofence_core::Item, query: &str) -> bool {
    let (types, text) = pecofence_core::type_filter::split(query);
    if !types.is_empty() {
        let file = item
            .path()
            .and_then(|p| std::path::Path::new(p).file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| item.display_name.clone());
        if !is_of_types(&types, &file, item.is_folder, item.path()) {
            return false;
        }
        if text.is_empty() {
            return true;
        }
    }
    match peek::search_match(&item.display_name, item.path(), &text) {
        Some((score, _)) => !pecofence_core::matcher::is_loose_score(score),
        None => false,
    }
}

/// Whether the file `file` at `path` is of `types`, a shortcut by its real target as the
/// rules read it (so the filter, the Peek's type tab, Everything's words and a saved smart
/// fence agree).
pub(super) fn is_of_types(
    types: &[pecofence_core::type_filter::TypeToken],
    file: &str,
    is_folder: bool,
    path: Option<&str>,
) -> bool {
    pecofence_core::type_filter::matches(types, file, is_folder, || {
        path.and_then(|p| shell::shortcut_target(std::path::Path::new(p)))
    })
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
        let cue = pecofence_core::i18n::format("筛选“{0}”", &[title]);
        let current = self.fence_filters.get(&fence).cloned().unwrap_or_default();
        filter_box::open(
            w.hwnd(),
            fence,
            &current,
            &cue,
            self.queue.clone(),
            self.theme_mode,
        );
        self.sync_filter_view(fence);
    }

    /// 筛选: how many items fence `f` shows now and has without its filter.
    fn filter_counts(&self, f: &pecofence_core::Fence) -> (usize, usize) {
        let items = self.state.items_of(f);
        let shown = match self.fence_filter(f.id) {
            Some(q) => items.iter().filter(|it| filter_keeps(it, &q)).count(),
            None => items.len(),
        };
        (shown, items.len())
    }

    /// 筛选: the count in the open box ("4 / 31", all of them while the text is empty), and the
    /// tag of a filtered single title once its box is closed (its text, and its count unless
    /// the smart fence's count capsule shows it already).
    pub(super) fn sync_filter_view(&self, fence: FenceId) {
        let Some(f) = self.state.fence(fence) else {
            return;
        };
        let text = self.fence_filters.get(&fence).map_or("", |t| t.trim());
        let open = filter_box::active_fence() == Some(fence);
        let host = self.state.host_of(fence);
        if !open && text.is_empty() {
            if let Some(w) = self.fences.get(&host) {
                w.set_filter_tag(None);
            }
            return;
        }
        let (shown, total) = self.filter_counts(f);
        if open {
            let count = if text.is_empty() {
                pecofence_core::i18n::format("{0} 项", &[total.to_string()])
            } else {
                format!("{shown} / {total}")
            };
            filter_box::set_count(&count);
        }
        let single = host == fence && self.state.tabs_of(host).len() <= 1;
        let tag = (single && !open && !text.is_empty()).then(|| {
            let count = if f.smart.is_some() {
                String::new()
            } else {
                format!("{shown} / {total}")
            };
            // The type and the text apart: a chip of its own for the type.
            let (types, typed) = pecofence_core::type_filter::split(text);
            let kind = types
                .iter()
                .map(pecofence_core::type_filter::label)
                .collect::<Vec<_>>()
                .join(" ");
            (kind, typed, count)
        });
        if let Some(w) = self.fences.get(&host) {
            w.set_filter_tag(tag);
        }
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
        self.sync_filter_view(fence);
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
        self.sync_filter_view(fence);
    }

    /// The title of a filtered fence in a tab strip: `<title> · 筛选：<text>`. A single title
    /// keeps its name (its tag shows the filter), as it does while its box shows the text.
    pub(super) fn filter_title(&self, id: FenceId, title: String) -> String {
        let single = self.state.host_of(id) == id && self.state.tabs_of(id).len() <= 1;
        if filter_box::active_fence() == Some(id) || single {
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
