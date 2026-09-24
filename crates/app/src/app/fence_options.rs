//! Per-fence options shared by the fence context menu and the settings page's 「栅栏」 tab:
//! one setter per property (state + window + anchor + save) and the page protocol
//! (`fences[]` details in the state JSON, `setFence` messages, `showFence` navigation).
//!
//! Window-level properties (appearance, lock, quick-hide exclusion, auto height, dock) act on
//! the fence's host window when the fence is a tab; content properties (icon size, spacing,
//! portal flags) act on the fence itself, exactly like the menu did.

use super::*;

/// Fences-style colour choices for the per-fence tint (name, rgb); the settings page builds
/// its 色调 options from this list (`tintPalette` in the state JSON).
const TINT_PALETTE: &[(&str, [u8; 3])] = &[
    ("红", [0xE7, 0x48, 0x56]),
    ("橙", [0xF7, 0x63, 0x0C]),
    ("黄", [0xFF, 0xB9, 0x00]),
    ("绿", [0x10, 0x89, 0x3E]),
    ("青", [0x00, 0xB7, 0xC3]),
    ("蓝", [0x00, 0x78, 0xD4]),
    ("紫", [0x87, 0x64, 0xB8]),
    ("粉", [0xE3, 0x00, 0x8C]),
    ("灰", [0x7A, 0x75, 0x74]),
];

pub(super) fn tint_palette_json() -> serde_json::Value {
    TINT_PALETTE
        .iter()
        .map(|(name, rgb)| serde_json::json!({ "name": pecofence_core::i18n::text(name), "hex": hex(*rgb) }))
        .collect()
}

/// "更透明" / "更厚实" opacity presets (the menu's former values).
const OPACITY_CLEAR: f32 = 0.55;
const OPACITY_SOLID: f32 = 1.6;

fn hex(rgb: [u8; 3]) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// Every property `parse_fence_prop` accepts (the order the error message lists them in).
pub(super) const FENCE_PROPS: &[&str] = &[
    "title",
    "iconSize",
    "spacing",
    "autoHeight",
    "locked",
    "excludeFromQuickHide",
    "opacity",
    "tint",
    "titleColor",
    "titleSize",
    "layout",
    "sort",
    "reverse",
    "groupByDate",
    "labelLines",
    "portalNavigate",
    "portalTitleIcon",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TitleColorChoice {
    Theme,
    /// Follow the fence tint.
    Tint,
    Rgb([u8; 3]),
}

/// One per-fence option, parsed and range-checked (shared by the settings page's `setFence`
/// and the CLI's `fences.setOption`).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum FenceProp {
    Title(String),
    IconSize(u32),
    Spacing(Spacing),
    AutoHeight(bool),
    Locked(bool),
    ExcludeFromQuickHide(bool),
    /// None = the global default; otherwise a preset (see `OPACITY_CLEAR` / `OPACITY_SOLID`).
    Opacity(Option<f32>),
    Tint(Option<[u8; 3]>),
    TitleColor(TitleColorChoice),
    /// None = normal.
    TitleSize(Option<TitleSize>),
    PortalNavigate(bool),
    PortalTitleIcon(bool),
    Layout(ViewLayout),
    Sort(SortMode),
    Reverse(bool),
    GroupByDate(bool),
    LabelLines(u8),
}

/// Parses `prop` / `value` as the settings page and `fence get` spell them. Unknown property
/// → `invalid_value` listing every property; a bad value → `invalid_value` listing what the
/// property accepts.
pub(super) fn parse_fence_prop(
    prop: &str,
    value: &serde_json::Value,
) -> std::result::Result<FenceProp, pecofence_ipc::IpcError> {
    use pecofence_ipc::IpcError;
    let bad = |allowed: &[&str]| {
        IpcError::invalid_value(
            format!("{prop} cannot be {value}; allowed: {}", allowed.join(", ")),
            allowed,
        )
    };
    let as_bool = || value.as_bool().ok_or_else(|| bad(&["true", "false"]));
    let as_str = |allowed: &[&str]| value.as_str().ok_or_else(|| bad(allowed));
    Ok(match prop {
        "title" => {
            let title = as_str(&["<non-empty string>"])?;
            FenceProp::Title(super::ipc::checked_name("title", title)?)
        }
        "iconSize" => {
            const ALLOWED: &[&str] = &["32", "48", "64", "96"];
            match value.as_u64() {
                Some(size @ (32 | 48 | 64 | 96)) => FenceProp::IconSize(size as u32),
                _ => return Err(bad(ALLOWED)),
            }
        }
        "spacing" => {
            const ALLOWED: &[&str] = &["compact", "normal", "loose"];
            FenceProp::Spacing(match as_str(ALLOWED)? {
                "compact" => Spacing::Compact,
                "normal" => Spacing::Normal,
                "loose" => Spacing::Loose,
                _ => return Err(bad(ALLOWED)),
            })
        }
        "autoHeight" => FenceProp::AutoHeight(as_bool()?),
        "locked" => FenceProp::Locked(as_bool()?),
        "excludeFromQuickHide" => FenceProp::ExcludeFromQuickHide(as_bool()?),
        "opacity" => {
            const ALLOWED: &[&str] = &["default", "clear", "solid"];
            FenceProp::Opacity(match as_str(ALLOWED)? {
                "default" => None,
                "clear" => Some(OPACITY_CLEAR),
                "solid" => Some(OPACITY_SOLID),
                _ => return Err(bad(ALLOWED)),
            })
        }
        "tint" => {
            const ALLOWED: &[&str] = &["#RRGGBB", "null"];
            FenceProp::Tint(match value {
                serde_json::Value::Null => None,
                serde_json::Value::String(s) if s.is_empty() => None,
                serde_json::Value::String(s) => Some(parse_hex(s).ok_or_else(|| bad(ALLOWED))?),
                _ => return Err(bad(ALLOWED)),
            })
        }
        "titleColor" => {
            const ALLOWED: &[&str] = &["theme", "tint", "white", "black", "#RRGGBB"];
            FenceProp::TitleColor(match as_str(ALLOWED)? {
                "theme" => TitleColorChoice::Theme,
                "tint" => TitleColorChoice::Tint,
                "white" => TitleColorChoice::Rgb([0xFF, 0xFF, 0xFF]),
                "black" => TitleColorChoice::Rgb([0x00, 0x00, 0x00]),
                other => TitleColorChoice::Rgb(parse_hex(other).ok_or_else(|| bad(ALLOWED))?),
            })
        }
        "titleSize" => {
            const ALLOWED: &[&str] = &["small", "normal", "large"];
            FenceProp::TitleSize(match as_str(ALLOWED)? {
                "small" => Some(TitleSize::Small),
                "normal" => None,
                "large" => Some(TitleSize::Large),
                _ => return Err(bad(ALLOWED)),
            })
        }
        "layout" => {
            const ALLOWED: &[&str] = &["icons", "list", "details"];
            FenceProp::Layout(match as_str(ALLOWED)? {
                "icons" => ViewLayout::Icons,
                "list" => ViewLayout::List,
                "details" => ViewLayout::Details,
                _ => return Err(bad(ALLOWED)),
            })
        }
        "sort" => {
            const ALLOWED: &[&str] = &["manual", "name", "type", "date", "size", "openCount"];
            FenceProp::Sort(match as_str(ALLOWED)? {
                "manual" => SortMode::Manual,
                "name" => SortMode::Name,
                "type" => SortMode::Type,
                "date" => SortMode::Date,
                "size" => SortMode::Size,
                "openCount" => SortMode::OpenCount,
                _ => return Err(bad(ALLOWED)),
            })
        }
        "reverse" => FenceProp::Reverse(as_bool()?),
        "groupByDate" => FenceProp::GroupByDate(as_bool()?),
        "labelLines" => {
            const ALLOWED: &[&str] = &["1", "2", "3"];
            match value.as_u64() {
                Some(n @ 1..=3) => FenceProp::LabelLines(n as u8),
                _ => return Err(bad(ALLOWED)),
            }
        }
        "portalNavigate" => FenceProp::PortalNavigate(as_bool()?),
        "portalTitleIcon" => FenceProp::PortalTitleIcon(as_bool()?),
        _ => {
            return Err(IpcError::invalid_value(
                format!("unknown fence property {prop:?}"),
                FENCE_PROPS,
            ));
        }
    })
}

/// String forms of `FenceView.layout` / `.sort` as the page, `fence get` and `fence set` use.
pub(super) fn layout_name(layout: ViewLayout) -> &'static str {
    match layout {
        ViewLayout::Icons => "icons",
        ViewLayout::List => "list",
        ViewLayout::Details => "details",
    }
}

pub(super) fn sort_name(sort: SortMode) -> &'static str {
    match sort {
        SortMode::Manual => "manual",
        SortMode::Name => "name",
        SortMode::Type => "type",
        SortMode::Date => "date",
        SortMode::Size => "size",
        SortMode::OpenCount => "openCount",
    }
}

impl App {
    pub(super) fn set_fence_auto_height(&mut self, fence: FenceId, on: bool) {
        let host = self.state.host_of(fence);
        self.state.set_auto_height(host, on);
        if let Some(w) = self.fences.get(&host) {
            w.set_auto_height(on);
        }
        self.apply_auto_height(host);
        self.schedule_save();
    }

    pub(super) fn set_fence_locked(&mut self, fence: FenceId, on: bool) {
        let host = self.state.host_of(fence);
        self.state.set_locked(host, on);
        if let Some(w) = self.fences.get(&host) {
            w.set_locked(on);
        }
        self.schedule_save();
    }

    pub(super) fn set_fence_quick_hide_excluded(&mut self, fence: FenceId, on: bool) {
        let host = self.state.host_of(fence);
        self.state.set_exclude_from_quick_hide(host, on);
        if let Some(w) = self.fences.get(&host)
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.set_quick_hide_excluded(w.hwnd(), on);
        }
        self.schedule_save();
    }

    /// `None` = default opacity; otherwise one of the presets (or any imported value).
    pub(super) fn set_fence_opacity(&mut self, fence: FenceId, opacity: Option<f32>) {
        let host = self.state.host_of(fence);
        self.state.set_appearance(host, None, opacity);
        self.apply_fence_appearance(host);
        self.schedule_save();
    }

    /// Tint / title colour / title size in one go (`None` fields = theme defaults). A title
    /// that followed the tint keeps following it; clearing the tint clears such a title.
    pub(super) fn set_fence_style(
        &mut self,
        fence: FenceId,
        tint: Option<[u8; 3]>,
        title_rgb: Option<[u8; 3]>,
        title_size: Option<TitleSize>,
    ) {
        let host = self.state.host_of(fence);
        self.state.set_style(host, tint, title_rgb, title_size);
        self.apply_fence_appearance(host);
        // Tab colour bars follow the tint.
        self.refresh_fence(host);
        self.schedule_save();
    }

    pub(super) fn set_fence_tint(&mut self, fence: FenceId, tint: Option<[u8; 3]>) {
        let host = self.state.host_of(fence);
        let (old_tint, mut title_rgb, title_size) = self.style_of(host);
        let follow = title_rgb.is_some() && title_rgb == old_tint;
        if follow {
            title_rgb = tint;
        }
        self.set_fence_style(fence, tint, title_rgb, title_size);
    }

    /// A tab keeps its own title appearance when it joins or leaves a stack.
    fn set_fence_title_style(
        &mut self,
        fence: FenceId,
        color: Option<[u8; 3]>,
        size: Option<TitleSize>,
    ) {
        let (tint, _, _) = self.style_of(fence);
        self.state.set_style(fence, tint, color, size);
        if self.state.host_of(fence) == fence {
            self.apply_fence_appearance(fence);
        }
        self.refresh_fence(fence);
        self.schedule_save();
    }

    pub(super) fn set_fence_spacing(&mut self, fence: FenceId, spacing: Spacing) {
        self.state.set_spacing(fence, spacing);
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            w.set_spacing(spacing);
        }
        self.apply_column_snap(fence);
        self.apply_auto_height(fence);
        self.schedule_save();
    }

    pub(super) fn set_fence_portal_navigate(&mut self, fence: FenceId, on: bool) {
        self.state.set_portal_navigate(fence, on);
        self.apply_portal_deco(self.state.host_of(fence));
        self.schedule_save();
    }

    pub(super) fn set_fence_title_icon(&mut self, fence: FenceId, show: bool) {
        self.state.set_hide_title_icon(fence, !show);
        self.apply_portal_deco(self.state.host_of(fence));
        self.schedule_save();
    }

    /// (tint, title colour, title size) of the host window's appearance override.
    fn style_of(&self, host: FenceId) -> (Option<[u8; 3]>, Option<[u8; 3]>, Option<TitleSize>) {
        self.state
            .fence(host)
            .and_then(|f| f.appearance.as_ref())
            .map(|a| (a.tint_rgb, a.title_rgb, a.title_size))
            .unwrap_or((None, None, None))
    }

    /// Opens the settings window on the 「栅栏」 page with `fence` selected.
    pub(super) fn open_fence_options(&mut self, fence: FenceId) {
        let already_open = self.settings.is_some();
        self.open_settings();
        if self.settings.is_none() {
            return;
        }
        if already_open {
            self.post_show_fence(fence);
        } else {
            // The page is still loading; `ready` delivers the state and then this.
            self.settings_focus_fence = Some(fence);
        }
    }

    pub(super) fn post_show_fence(&self, fence: FenceId) {
        if let Some(h) = &self.settings {
            h.post_json(&serde_json::json!({ "type": "showFence", "id": fence }).to_string());
        }
    }

    /// The per-fence block of the state JSON. Window-level values come from the host so a
    /// tab shows (and edits) what its window actually uses.
    pub(super) fn fence_options_json(&self, f: &pecofence_core::Fence) -> serde_json::Value {
        let host_id = self.state.host_of(f.id);
        let h = self.state.fence(host_id).unwrap_or(f);
        let (tint, _, _) = self.style_of(host_id);
        let (_, title_rgb, title_size) = self.style_of(f.id);
        let opacity = h.appearance.as_ref().and_then(|a| a.opacity);
        let opacity = match opacity {
            None => "default",
            Some(o) if o < 1.0 => "clear",
            Some(_) => "solid",
        };
        let title_color = match title_rgb {
            None => "theme".to_string(),
            Some(rgb) if tint == Some(rgb) => "tint".to_string(),
            Some([0xFF, 0xFF, 0xFF]) => "white".to_string(),
            Some([0x00, 0x00, 0x00]) => "black".to_string(),
            Some(rgb) => hex(rgb),
        };
        let title_size = match title_size.unwrap_or_default() {
            TitleSize::Small => "small",
            TitleSize::Normal => "normal",
            TitleSize::Large => "large",
        };
        let spacing = match f.view.spacing {
            Spacing::Compact => "compact",
            Spacing::Normal => "normal",
            Spacing::Loose => "loose",
        };
        let portal = (f.kind == FenceKind::FolderPortal).then(|| {
            serde_json::json!({
                "navigate": f.portal_navigate,
                "titleIcon": !f.hide_title_icon,
            })
        });
        serde_json::json!({
            "id": f.id,
            "title": f.title,
            "kind": match f.kind {
                FenceKind::Inbox => "inbox",
                FenceKind::Virtual => "virtual",
                FenceKind::FolderPortal => "portal",
            },
            "host": (host_id != f.id).then(|| self.state.fence(host_id).map(|h| h.title.clone())),
            "iconSize": f.view.icon_size,
            "spacing": spacing,
            "autoHeight": h.view.auto_height,
            "locked": h.locked,
            "excludeFromQuickHide": h.exclude_from_quick_hide,
            "opacity": opacity,
            "tint": tint.map(hex),
            "titleColor": title_color,
            "titleSize": title_size,
            "portal": portal,
        })
    }

    /// `{"type":"setFence","id":…,"prop":…,"value":…}` from the page.
    pub(super) fn on_set_fence(&mut self, v: &serde_json::Value) {
        let Some(fence) = v
            .get("id")
            .and_then(|i| i.as_str())
            .and_then(|i| uuid::Uuid::parse_str(i).ok())
        else {
            return;
        };
        if self.state.fence(fence).is_none() {
            return;
        }
        let Some(prop) = v.get("prop").and_then(|p| p.as_str()) else {
            return;
        };
        let value = v.get("value").cloned().unwrap_or(serde_json::Value::Null);
        if prop == "dockTop" {
            // An action, not a property (the CLI has `fences.dockTop`); the page still sends
            // it through `setFence`.
            self.dock_to_top(self.state.host_of(fence));
        } else {
            match parse_fence_prop(prop, &value) {
                Ok(p) => {
                    self.apply_fence_prop(fence, p);
                }
                Err(error) => {
                    tracing::warn!(prop, %error, "bad fence property from settings page");
                    return;
                }
            }
        }
        self.push_settings_state();
    }

    /// Applies one parsed property to `fence` (window-level ones act on its host, see the
    /// module docs). Returns whether the persisted state actually changed.
    pub(super) fn apply_fence_prop(&mut self, fence: FenceId, prop: FenceProp) -> bool {
        let host = self.state.host_of(fence);
        let before = (
            self.state.fence(fence).cloned(),
            self.state.fence(host).cloned(),
        );
        match prop {
            FenceProp::Title(title) => {
                self.state.rename_fence(fence, &title);
                self.refresh_fence(fence);
                if let Some(w) = self.window_for(fence) {
                    w.redraw();
                }
                self.schedule_save();
            }
            FenceProp::IconSize(size) => self.apply_icon_size(fence, size),
            FenceProp::Spacing(spacing) => self.set_fence_spacing(fence, spacing),
            FenceProp::AutoHeight(on) => self.set_fence_auto_height(fence, on),
            FenceProp::Locked(on) => self.set_fence_locked(fence, on),
            FenceProp::ExcludeFromQuickHide(on) => self.set_fence_quick_hide_excluded(fence, on),
            FenceProp::Opacity(op) => self.set_fence_opacity(fence, op),
            FenceProp::Tint(tint) => self.set_fence_tint(fence, tint),
            FenceProp::TitleColor(choice) => {
                let (tint, _, _) = self.style_of(host);
                let (_, _, title_size) = self.style_of(fence);
                let title_rgb = match choice {
                    TitleColorChoice::Theme => None,
                    TitleColorChoice::Tint => tint,
                    TitleColorChoice::Rgb(rgb) => Some(rgb),
                };
                self.set_fence_title_style(fence, title_rgb, title_size);
            }
            FenceProp::TitleSize(size) => {
                let (_, title_rgb, _) = self.style_of(fence);
                self.set_fence_title_style(fence, title_rgb, size);
            }
            FenceProp::PortalNavigate(on) => self.set_fence_portal_navigate(fence, on),
            FenceProp::PortalTitleIcon(on) => self.set_fence_title_icon(fence, on),
            FenceProp::Layout(layout) => {
                // Same steps as the fence menu's 视图 items.
                self.state.set_layout(fence, layout);
                if let Some(w) = self.window_for(fence)
                    && w.active_fence() == fence
                {
                    w.set_layout(layout);
                }
                self.apply_column_snap(fence);
                self.apply_auto_height(fence);
                self.schedule_save();
            }
            FenceProp::Sort(sort) => {
                self.state.set_sort(fence, sort);
                self.refresh_fence(fence);
                self.schedule_save();
            }
            FenceProp::Reverse(on) => {
                self.state.set_reverse(fence, on);
                self.refresh_fence(fence);
                self.schedule_save();
            }
            FenceProp::GroupByDate(on) => {
                self.state.set_group_by_date(fence, on);
                self.refresh_fence(fence);
                self.schedule_save();
            }
            FenceProp::LabelLines(lines) => {
                if let Some(f) = self.state.fence_mut(fence)
                    && f.view.label_lines != lines
                {
                    f.view.label_lines = lines;
                    self.state.mark_dirty();
                }
                if let Some(w) = self.window_for(fence)
                    && w.active_fence() == fence
                {
                    w.set_label_lines(lines);
                }
                self.apply_column_snap(fence);
                self.apply_auto_height(fence);
                self.schedule_save();
            }
        }
        let after = (
            self.state.fence(fence).cloned(),
            self.state.fence(host).cloned(),
        );
        before != after
    }
}
