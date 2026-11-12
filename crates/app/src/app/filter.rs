//! 筛选 in a folder fence or a smart fence: what the fence shows narrows to the names that match
//! the typed text, matched as the Peek search matches names (pinyin too, without its loosest
//! letters-in-order match). The files stay as they are; the filter is not saved, and going
//! into another folder clears it, as Explorer's search box does.

use super::*;
use crate::filter_box;

/// Whether `item` stays in a fence filtered by `query` (lower case, trimmed): matching the text
/// and of its 类型 (the `kind:` words). The text first: 程序 needs a shortcut's target, and a
/// shortcut whose target is still being read stays out until it is known.
pub(super) fn filter_keeps(item: &pecofence_core::Item, query: &str) -> bool {
    let (types, text) = pecofence_core::type_filter::split(query);
    let named = text.is_empty()
        || peek::search_match(&item.display_name, item.path(), &text)
            .is_some_and(|(score, _)| !pecofence_core::matcher::is_loose_score(score));
    if !named || types.is_empty() {
        return named;
    }
    let file = item
        .path()
        .and_then(|p| std::path::Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| item.display_name.clone());
    type_fit(&types, &file, item.is_folder, item.path(), Some(item.mtime)) == Some(true)
}

/// A filter's text as the UI shows it: its `kind:` words as their labels (类型：PDF, in the UI
/// language), then the rest.
pub(super) fn filter_words(text: &str) -> String {
    let (types, typed) = pecofence_core::type_filter::split(text);
    types
        .iter()
        .map(pecofence_core::type_filter::label)
        .chain((!typed.is_empty()).then_some(typed))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether the file `file` at `path` (last written at `mtime`, unix seconds; None: any time) is
/// of `types`, a shortcut by its real target as the rules read it (so the filter, the Peek's
/// type tab, Everything's words and a saved smart fence agree). None while a shortcut's target
/// is being read off the UI thread: the row is held back until it is known.
pub(super) fn type_fit(
    types: &[pecofence_core::type_filter::TypeToken],
    file: &str,
    is_folder: bool,
    path: Option<&str>,
    mtime: Option<i64>,
) -> Option<bool> {
    let target = match path {
        Some(p) if pecofence_core::type_filter::needs_target(types, file, is_folder) => {
            match super::lnk_targets::known(p, mtime) {
                Some(t) => t,
                None => {
                    super::lnk_targets::ask(p, mtime);
                    return None;
                }
            }
        }
        _ => None,
    };
    Some(pecofence_core::type_filter::matches(
        types,
        file,
        is_folder,
        || target,
    ))
}

impl App {
    /// Shortcut targets were read (`TIMER_LNK_TARGETS`): the Peek list with a type tab and the
    /// fences filtered by a type show the rows they held back.
    pub(super) fn lnk_targets_read(&mut self) {
        if !super::lnk_targets::taken() {
            return;
        }
        if crate::peek_search::is_open() && crate::peek_search::type_tab().is_some() {
            let query = crate::peek_search::query()
                .map(|q| q.trim().to_lowercase())
                .unwrap_or_default();
            self.rank_fence_matches(&query);
            self.refresh_search_rows();
        }
        let typed: Vec<FenceId> = self
            .fence_filters
            .iter()
            .filter(|(_, t)| !pecofence_core::type_filter::split(t).0.is_empty())
            .map(|(id, _)| *id)
            .collect();
        for id in typed {
            self.refresh_fence(id);
            self.sync_filter_view(id);
        }
    }

    /// The Peek began: the fences' shortcuts get their targets read, so 程序 has them at hand.
    pub(super) fn prefetch_lnk_targets(&self) {
        let mut asked = 0;
        for f in self.state.fences() {
            for it in self.state.unsorted_items_of(f) {
                if let Some(p) = it.path()
                    && !it.is_folder
                    && p.to_lowercase().ends_with(".lnk")
                    && super::lnk_targets::known(p, Some(it.mtime)).is_none()
                {
                    super::lnk_targets::ask(p, Some(it.mtime));
                    asked += 1;
                }
            }
        }
        tracing::debug!(asked, "type filter: the fences' shortcut targets asked for");
    }

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
            w.label_ink(self.theme_mode),
        );
        self.sync_filter_view(fence);
    }

    /// 筛选: how many items fence `f` shows now and has without its filter.
    fn filter_counts(&self, f: &pecofence_core::Fence) -> (usize, usize) {
        // Counted, never shown: without the sort of the whole folder.
        let items: Vec<&pecofence_core::Item> = self.state.unsorted_items_of(f).collect();
        let shown = match self.fence_filter(f.id) {
            Some(q) => items.iter().filter(|it| filter_keeps(it, &q)).count(),
            None => items.len(),
        };
        (shown, items.len())
    }

    /// 筛选: the count in the open box ("4 / 31", all of them while the text is empty), and the
    /// tag of a filtered single title once its box is closed (its text, and its count unless
    /// the smart fence's count capsule shows it already). Returns how many items the fence
    /// shows when it had to count them.
    pub(super) fn sync_filter_view(&self, fence: FenceId) -> Option<usize> {
        let f = self.state.fence(fence)?;
        let text = self.fence_filters.get(&fence).map_or("", |t| t.trim());
        let open = filter_box::active_fence() == Some(fence);
        let host = self.state.host_of(fence);
        if !open && text.is_empty() {
            if let Some(w) = self.fences.get(&host) {
                w.set_filter_tag(None);
            }
            return None;
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
        Some(shown)
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
        // A filter that replaces most of what the fence shows (the first letter, clearing it)
        // cross-fades the page instead of cutting.
        let host = self.state.host_of(fence);
        if self.state.active_tab_of(host) == fence
            && let Some(w) = self.fences.get(&host)
        {
            w.fade_next_refill();
        }
        self.refresh_fence(fence);
        // The count the box shows, or every item of a fence whose filter is gone. It used to
        // be counted for this line alone, by listing, sorting and matching the whole folder a
        // third time at every letter typed.
        let shown = self.sync_filter_view(fence).unwrap_or_else(|| {
            self.state
                .fence(fence)
                .map_or(0, |f| self.state.unsorted_items_of(f).count())
        });
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
                pecofence_core::i18n::format("{0} · 筛选：{1}", &[title, filter_words(t)])
            }
            _ => title,
        }
    }
}
