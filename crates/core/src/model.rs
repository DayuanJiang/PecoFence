//! Persistent data model (plan §7). Pure data + serde; no Windows types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

pub type FenceId = Uuid;
pub type ItemId = Uuid;
pub type RuleId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub schema_version: u32,
    pub settings: Settings,
    /// Global item table keyed by id.
    pub items: HashMap<ItemId, Item>,
    /// One layout per monitor configuration.
    pub layouts: Vec<Layout>,
    pub rules: crate::rules::RuleSet,
    #[serde(default)]
    pub undo_log: Vec<Assignment>,
    /// Saved layouts the user can return to (Fences "snapshots").
    #[serde(default)]
    pub snapshots: Vec<Snapshot>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: Settings::default(),
            items: HashMap::new(),
            layouts: Vec::new(),
            rules: crate::rules::RuleSet::default(),
            undo_log: Vec::new(),
            snapshots: Vec::new(),
        }
    }
}

/// A named copy of every layout (fence geometry, membership, view flags) taken at `ts`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: Uuid,
    pub name: String,
    pub ts: i64,
    pub layouts: Vec<Layout>,
}

pub const MAX_SNAPSHOTS: usize = 20;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "crate::i18n::Language::legacy_default")]
    pub language: crate::i18n::Language,
    pub backdrop: Backdrop,
    pub theme: ThemeSetting,
    /// Material style is independent of the light/dark preference.
    #[serde(default)]
    pub theme_style: ThemeStyle,
    pub icon_size: u32,
    pub quick_hide: QuickHideSettings,
    pub roll_up: RollUpSettings,
    pub show_desktop: ShowDesktopSetting,
    pub hide_real_icons: bool,
    pub show_real_icons_when_fences_hidden: bool,
    pub snapping: SnappingSettings,
    pub zorder: ZOrderSetting,
    pub autostart: bool,
    #[serde(default)]
    pub telemetry: bool,
    /// Fences "Peek": a hotkey floats every fence above the current windows.
    #[serde(default)]
    pub peek: PeekSettings,
    #[serde(default)]
    pub icons: IconSettings,
    /// The user's Desktop folder as last seen, so a moved desktop (OneDrive, another drive) can
    /// have its item records re-pointed instead of orphaned.
    #[serde(default)]
    pub desktop_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeekSettings {
    pub enabled: bool,
    /// Dim everything behind the fences while peeking.
    pub dim: bool,
    pub hotkey: PeekHotkey,
}

impl Default for PeekSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            dim: true,
            hotkey: PeekHotkey::CtrlAltSpace,
        }
    }
}

/// Peek hotkey choices. Fences uses Win+Space, but Windows reserves Win+Space (and
/// Win+Shift/Ctrl+Space) for the input-language switcher whenever more than one keyboard
/// layout is installed, so Ctrl+Alt+Space is the default and the others are offered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PeekHotkey {
    WinSpace,
    #[default]
    CtrlAltSpace,
    WinShiftSpace,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: crate::i18n::Language::System,
            backdrop: Backdrop::Acrylic,
            theme: ThemeSetting::FollowWindowsMode,
            theme_style: ThemeStyle::Fluent,
            icon_size: 48,
            quick_hide: QuickHideSettings::default(),
            roll_up: RollUpSettings::default(),
            show_desktop: ShowDesktopSetting::KeepVisible,
            hide_real_icons: true,
            show_real_icons_when_fences_hidden: false,
            snapping: SnappingSettings::default(),
            zorder: ZOrderSetting::InsertAboveHost,
            autostart: true,
            telemetry: false,
            peek: PeekSettings::default(),
            icons: IconSettings::default(),
            desktop_path: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Backdrop {
    /// One material for all fences. Older configurations and snapshots still load.
    #[serde(alias = "micaLike", alias = "solid")]
    Acrylic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeSetting {
    FollowWindowsMode,
    FollowAppMode,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeStyle {
    #[default]
    Fluent,
    LiquidGlass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShowDesktopSetting {
    KeepVisible,
    HideWithDesktop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ZOrderSetting {
    InsertAboveHost,
    HwndBottom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickHideSettings {
    pub enabled: bool,
    pub scope: QuickHideScope,
    pub always_show_at_startup: bool,
    pub delay_ms: u32,
    pub auto_hide_idle_sec: u32,
    pub auto_show_on_use: bool,
    #[serde(default)]
    pub wallpaper_engine_class_whitelist: Vec<String>,
}

impl Default for QuickHideSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            scope: QuickHideScope::All,
            always_show_at_startup: true,
            delay_ms: 300,
            auto_hide_idle_sec: 0,
            auto_show_on_use: true,
            wallpaper_engine_class_whitelist: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickHideScope {
    All,
    LooseOnly,
    FencesOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollUpSettings {
    pub double_click_title: bool,
    pub auto_on_screen_edge: bool,
    pub hover_peek: bool,
    pub hover_open_ms: u32,
    pub close_grace_ms: u32,
    /// A rolled fence expands on a single title click instead of on hover (Fences "require a
    /// click to expand"). Hover peek is ignored while this is on.
    #[serde(default)]
    pub click_to_expand: bool,
    /// Draw the title (and tabs) only while the mouse is over the fence.
    #[serde(default)]
    pub title_on_hover: bool,
    /// Show the scrollbar only while the mouse is inside the fence or right after scrolling.
    #[serde(default)]
    pub hide_inactive_scrollbar: bool,
}

impl Default for RollUpSettings {
    fn default() -> Self {
        Self {
            double_click_title: true,
            auto_on_screen_edge: true,
            hover_peek: true,
            hover_open_ms: 400,
            close_grace_ms: 400,
            click_to_expand: false,
            title_on_hover: false,
            hide_inactive_scrollbar: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnappingSettings {
    pub enabled: bool,
    pub gap_px: i32,
    pub size_to_cells: bool,
    pub guide_lines: bool,
}

impl Default for SnappingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            gap_px: 8,
            size_to_cells: false,
            guide_lines: false,
        }
    }
}

/// Identifies a monitor across sessions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorIdentity {
    /// Device path (`GSM1388#4&125707d6&0&UID8388688`), falling back to `\\.\DISPLAYn`.
    pub device_path: String,
    /// Work-area size in DIPs at save time.
    pub work_dip: [f32; 2],
    pub dpi: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    /// Sorted monitor identities; a layout applies when the current set matches by device path.
    pub fingerprint: Vec<MonitorIdentity>,
    pub fences: Vec<Fence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FenceKind {
    Virtual,
    /// The system "桌面" fence that receives everything unassigned. Exactly one per layout.
    Inbox,
    FolderPortal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ItemSourceSpec {
    Desktop,
    Folder {
        path: String,
        #[serde(default)]
        recursive: bool,
        #[serde(default)]
        filter: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Anchor {
    LeftTop,
    LeftBottom,
    LeftVCenter,
    RightTop,
    RightBottom,
    RightVCenter,
    HCenterTop,
    HCenterBottom,
    Center,
}

/// Fence geometry relative to a monitor's work area, in DIPs (plan §7.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormGeometry {
    pub monitor: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub work_w: f32,
    pub work_h: f32,
    pub anchor: Anchor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortMode {
    Manual,
    Name,
    Type,
    Date,
    Size,
    /// Most-opened first (launch count kept per item).
    OpenCount,
}

/// How a fence lays out its items (Fences 6 "view style").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewLayout {
    /// Icon grid with labels underneath.
    #[default]
    Icons,
    /// Compact rows: small icon + name.
    List,
    /// Rows with sortable columns: 名称 / 修改日期 / 类型 / 大小.
    Details,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FenceView {
    pub icon_size: u32,
    pub sort: SortMode,
    pub label_lines: u8,
    pub auto_height: bool,
    /// Reverse the sort order (Fences "反向").
    #[serde(default)]
    pub reverse: bool,
    #[serde(default)]
    pub layout: ViewLayout,
    #[serde(default)]
    pub spacing: Spacing,
    /// Details view column widths (修改日期, 类型, 大小) in DIPs; None = defaults.
    #[serde(default)]
    pub column_widths: Option<[f32; 3]>,
    /// Details columns shown (修改日期, 类型, 大小); None = all.
    #[serde(default)]
    pub columns_visible: Option<[bool; 3]>,
}

impl Default for FenceView {
    fn default() -> Self {
        Self {
            icon_size: 48,
            sort: SortMode::Manual,
            label_lines: 2,
            auto_height: false,
            reverse: false,
            layout: ViewLayout::Icons,
            spacing: Spacing::Normal,
            column_widths: None,
            columns_visible: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceOverride {
    /// Colour wash over the glass (Fences per-fence colour).
    pub tint_rgb: Option<[u8; 3]>,
    pub opacity: Option<f32>,
    pub backdrop: Option<Backdrop>,
    /// Title text colour (None = theme text colour).
    #[serde(default)]
    pub title_rgb: Option<[u8; 3]>,
    #[serde(default)]
    pub title_size: Option<TitleSize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TitleSize {
    Small,
    #[default]
    Normal,
    Large,
}

/// Distance between icons in the grid (Fences "icon spacing").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Spacing {
    Compact,
    #[default]
    Normal,
    Loose,
}

/// Global icon rendering tweaks (Fences "Icon Tint" / "Chameleon").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconSettings {
    /// Colourise every icon toward this colour.
    pub tint_rgb: Option<[u8; 3]>,
    /// 0..1 — how far toward the tint.
    pub tint_strength: f32,
    /// Chameleon: icons desaturate and fade so they blend with the backdrop.
    pub chameleon: bool,
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            tint_rgb: None,
            tint_strength: 0.6,
            chameleon: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fence {
    pub id: FenceId,
    pub title: String,
    pub kind: FenceKind,
    pub source: ItemSourceSpec,
    pub geometry: NormGeometry,
    pub rolled_up: bool,
    /// Height (DIPs) to restore when un-rolling.
    pub expanded_h: f32,
    pub view: FenceView,
    #[serde(default)]
    pub appearance: Option<AppearanceOverride>,
    #[serde(default)]
    pub exclude_from_quick_hide: bool,
    /// Position and size cannot be changed with the mouse (Fences "锁定").
    #[serde(default)]
    pub locked: bool,
    /// Shown as a tab inside another fence's window (Fences 6 tabbed fences). A hosted fence has
    /// no window of its own; its geometry is kept for when it is split out again.
    #[serde(default)]
    pub tab_host: Option<FenceId>,
    /// On a host: which tab's items the window shows (`None` = the host's own).
    #[serde(default)]
    pub active_tab: Option<FenceId>,
    /// On a host: strip order of its tabs (may place the host itself anywhere). Ids that are no
    /// longer tabs are ignored; tabs missing here are appended in layout order.
    #[serde(default)]
    pub tab_order: Vec<FenceId>,
    /// Folder portal: double-clicking a subfolder opens it inside the portal (Fences
    /// "Navigate"); off = open it in Explorer.
    #[serde(default = "default_true")]
    pub portal_navigate: bool,
    /// Folder portal: hide the folder glyph before the title.
    #[serde(default)]
    pub hide_title_icon: bool,
    #[serde(default)]
    pub items: Vec<ItemRef>,
}

impl Fence {
    pub fn new(title: &str, kind: FenceKind, geometry: NormGeometry) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            kind,
            source: ItemSourceSpec::Desktop,
            expanded_h: geometry.h,
            geometry,
            rolled_up: false,
            view: FenceView::default(),
            appearance: None,
            exclude_from_quick_hide: false,
            locked: false,
            tab_host: None,
            active_tab: None,
            tab_order: Vec::new(),
            portal_navigate: true,
            hide_title_icon: false,
            items: Vec::new(),
        }
    }

    pub fn contains_item(&self, id: ItemId) -> bool {
        self.items.iter().any(|r| r.item_id == id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssignedBy {
    User,
    Rule(RuleId),
    Migration,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRef {
    pub item_id: ItemId,
    #[serde(default)]
    pub manual_index: Option<u32>,
    pub assigned_by: AssignedBy,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKey {
    /// Lower-cased, normalized absolute path.
    Path(String),
    /// Base64 PIDL for namespace items (later).
    Pidl(String),
}

impl ItemKey {
    /// Normalizes a filesystem path into the canonical key form.
    pub fn from_path(path: &str) -> Self {
        let mut s = path.replace('/', "\\").to_lowercase();
        while s.ends_with('\\') && s.len() > 3 {
            s.pop();
        }
        ItemKey::Path(s)
    }

    pub fn as_path(&self) -> Option<&str> {
        match self {
            ItemKey::Path(p) => Some(p),
            ItemKey::Pidl(_) => None,
        }
    }

    /// A shell namespace item (Recycle Bin, This PC, ...) keyed by its `::{CLSID}` parsing
    /// name: no file behind it, so rename, portal and location commands do not apply.
    pub fn is_namespace(&self) -> bool {
        matches!(self, ItemKey::Path(p) if p.starts_with("::{"))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    #[default]
    UserDesktop,
    PublicDesktop,
    Namespace,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IconKey {
    /// Icon shared by extension (documents, most files).
    ByExt(String),
    /// Icon specific to this file (.exe, .lnk, .ico, folders with custom icons, images).
    ByContent { path: String, mtime: i64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: ItemId,
    pub key: ItemKey,
    pub origin: Origin,
    pub display_name: String,
    #[serde(default)]
    pub file_id: Option<u128>,
    pub mtime: i64,
    pub is_folder: bool,
    pub attrs: u32,
    pub icon_key: IconKey,
    #[serde(default)]
    pub orphaned_since: Option<i64>,
    /// File size in bytes (0 for folders); used by "按大小" sorting.
    #[serde(default)]
    pub size: u64,
    /// How often the item was launched from a fence ("按打开次数" sorting).
    #[serde(default)]
    pub open_count: u32,
}

impl Item {
    /// See [`ItemKey::is_namespace`].
    pub fn is_namespace(&self) -> bool {
        self.key.is_namespace()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    pub ts: i64,
    pub item_id: ItemId,
    pub from: Option<FenceId>,
    pub to: FenceId,
    pub rule_id: Option<RuleId>,
}

/// Runtime-only: a "New → …" created from fence X's background menu should land in X.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingCreation {
    pub fence_id: FenceId,
    pub since: std::time::Instant,
}

/// Reversible change of tab ownership. Content, names and appearance are deliberately
/// excluded: cancelling a drag must not undo filesystem refreshes or unrelated edits.
#[derive(Clone, Debug)]
pub struct TabDetach {
    pub tab: FenceId,
    pub source_host: FenceId,
    pub remaining_host: FenceId,
    before: Vec<TabPlacement>,
}

#[derive(Clone, Debug)]
struct TabPlacement {
    id: FenceId,
    host: Option<FenceId>,
    active: Option<FenceId>,
    order: Vec<FenceId>,
    geometry: NormGeometry,
    rolled: bool,
    expanded_h: f32,
    auto_height: bool,
}

impl TabPlacement {
    fn capture(fence: &Fence) -> Self {
        Self {
            id: fence.id,
            host: fence.tab_host,
            active: fence.active_tab,
            order: fence.tab_order.clone(),
            geometry: fence.geometry.clone(),
            rolled: fence.rolled_up,
            expanded_h: fence.expanded_h,
            auto_height: fence.view.auto_height,
        }
    }

    fn restore(&self, fence: &mut Fence) {
        fence.tab_host = self.host;
        fence.active_tab = self.active;
        fence.tab_order = self.order.clone();
        fence.geometry = self.geometry.clone();
        fence.rolled_up = self.rolled;
        fence.expanded_h = self.expanded_h;
        fence.view.auto_height = self.auto_height;
    }
}

impl Layout {
    /// Every tab can leave, including the fence that currently owns the HWND. A
    /// remaining tab takes ownership of the group and keeps its frame and strip order.
    pub fn detach_tab(&mut self, tab: FenceId) -> Option<TabDetach> {
        self.fences.iter().find(|f| f.id == tab)?;
        let source_host = self.host_of(tab);
        let order = self.tabs_of(source_host);
        let index = order.iter().position(|id| *id == tab)?;
        if order.len() < 2 {
            return None;
        }
        let remaining: Vec<_> = order.iter().copied().filter(|id| *id != tab).collect();
        let remaining_host = if tab == source_host {
            remaining[0]
        } else {
            source_host
        };
        let active = self.active_tab_of(source_host);
        let next_active = if active == tab {
            remaining[index.min(remaining.len() - 1)]
        } else {
            active
        };
        let before: Vec<_> = self
            .fences
            .iter()
            .filter(|f| order.contains(&f.id))
            .map(TabPlacement::capture)
            .collect();
        let group = before.iter().find(|p| p.id == source_host)?.clone();
        for fence in &mut self.fences {
            if fence.id == tab {
                fence.tab_host = None;
                fence.active_tab = None;
                fence.tab_order.clear();
            } else if remaining.contains(&fence.id) {
                if fence.id == remaining_host {
                    fence.tab_host = None;
                    fence.active_tab = (next_active != remaining_host).then_some(next_active);
                    fence.tab_order = remaining.clone();
                    if tab == source_host {
                        fence.geometry = group.geometry.clone();
                        fence.rolled_up = group.rolled;
                        fence.expanded_h = group.expanded_h;
                        fence.view.auto_height = group.auto_height;
                    }
                } else {
                    fence.tab_host = Some(remaining_host);
                    fence.active_tab = None;
                    fence.tab_order.clear();
                }
            }
        }
        Some(TabDetach {
            tab,
            source_host,
            remaining_host,
            before,
        })
    }

    pub fn cancel_tab_detach(&mut self, change: &TabDetach) -> bool {
        // A later delete/merge owns the state now; don't undo it with a stale drag.
        if self.host_of(change.tab) != change.tab
            || self.host_of(change.remaining_host) != change.remaining_host
            || change
                .before
                .iter()
                .any(|p| !self.fences.iter().any(|f| f.id == p.id))
        {
            return false;
        }
        let remaining = self.tabs_of(change.remaining_host);
        if remaining.len() + 1 != change.before.len()
            || change
                .before
                .iter()
                .filter(|p| p.id != change.tab)
                .any(|p| !remaining.contains(&p.id))
        {
            return false;
        }
        for placement in &change.before {
            if let Some(fence) = self.fences.iter_mut().find(|f| f.id == placement.id) {
                placement.restore(fence);
            }
        }
        true
    }

    /// The window a fence is shown in: itself, or the fence hosting it as a tab.
    pub fn host_of(&self, id: FenceId) -> FenceId {
        self.fences
            .iter()
            .find(|f| f.id == id)
            .and_then(|f| f.tab_host)
            .filter(|h| {
                self.fences
                    .iter()
                    .any(|f| f.id == *h && f.tab_host.is_none())
            })
            .unwrap_or(id)
    }

    /// Tabs of a host window in strip order: the host's `tab_order` first (pruned to ids that
    /// are still its tabs), then anything missing — the host itself, then its hosted fences in
    /// layout order. A fence that is itself hosted has no tabs; an empty `tab_order` gives the
    /// pre-reorder result.
    pub fn tabs_of(&self, host: FenceId) -> Vec<FenceId> {
        let natural: Vec<FenceId> = std::iter::once(host)
            .chain(
                self.fences
                    .iter()
                    .filter(|f| f.tab_host == Some(host))
                    .map(|f| f.id),
            )
            .collect();
        let mut out: Vec<FenceId> = Vec::with_capacity(natural.len());
        if let Some(h) = self.fences.iter().find(|f| f.id == host) {
            for id in &h.tab_order {
                if natural.contains(id) && !out.contains(id) {
                    out.push(*id);
                }
            }
        }
        for id in natural {
            if !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }

    /// Places `tab` at strip index `to` (clamped) in `host`'s window. Returns false when `tab`
    /// is not one of the host's tabs or nothing changes.
    pub fn reorder_tab(&mut self, host: FenceId, tab: FenceId, to: usize) -> bool {
        let current = self.tabs_of(host);
        let Some(from) = current.iter().position(|id| *id == tab) else {
            return false;
        };
        let mut order = current.clone();
        order.remove(from);
        let to = to.min(order.len());
        order.insert(to, tab);
        if order == current {
            return false;
        }
        match self.fences.iter_mut().find(|f| f.id == host) {
            Some(h) => {
                h.tab_order = order;
                true
            }
            None => false,
        }
    }

    /// The fence whose items a host window currently shows.
    pub fn active_tab_of(&self, host: FenceId) -> FenceId {
        self.fences
            .iter()
            .find(|f| f.id == host)
            .and_then(|f| f.active_tab)
            .filter(|t| {
                self.fences
                    .iter()
                    .any(|f| f.id == *t && f.tab_host == Some(host))
            })
            .unwrap_or(host)
    }

    /// Repairs tab links after loading or deleting: dangling hosts, chains (a tab hosted by a
    /// tab) and active tabs that are not tabs any more.
    pub fn normalize_tabs(&mut self) -> bool {
        let mut changed = false;
        let ids: Vec<FenceId> = self.fences.iter().map(|f| f.id).collect();
        // Pass 1: drop dangling / self references.
        for f in &mut self.fences {
            if let Some(h) = f.tab_host
                && (h == f.id || !ids.contains(&h))
            {
                f.tab_host = None;
                changed = true;
            }
        }
        // Pass 2: flatten chains — a host that is itself hosted moves its tabs up to its host.
        loop {
            let chain: Option<(FenceId, FenceId)> = self.fences.iter().find_map(|f| {
                let h = f.tab_host?;
                let host = self.fences.iter().find(|x| x.id == h)?;
                host.tab_host.map(|hh| (f.id, hh))
            });
            match chain {
                Some((id, new_host)) => {
                    if let Some(f) = self.fences.iter_mut().find(|f| f.id == id) {
                        f.tab_host = if new_host == id { None } else { Some(new_host) };
                    }
                    changed = true;
                }
                None => break,
            }
        }
        // Pass 3: active tabs must be real tabs of that host.
        let hosted: Vec<(FenceId, FenceId)> = self
            .fences
            .iter()
            .filter_map(|f| f.tab_host.map(|h| (f.id, h)))
            .collect();
        for f in &mut self.fences {
            if let Some(a) = f.active_tab
                && !hosted.iter().any(|(t, h)| *t == a && *h == f.id)
            {
                f.active_tab = None;
                changed = true;
            }
        }
        // Pass 4: tab_order only lists a host's own tabs; hosted fences keep none.
        for f in &mut self.fences {
            let before = f.tab_order.len();
            if f.tab_host.is_some() {
                f.tab_order.clear();
            } else {
                let id = f.id;
                f.tab_order
                    .retain(|t| *t == id || hosted.iter().any(|(tab, h)| tab == t && *h == id));
            }
            if f.tab_order.len() != before {
                changed = true;
            }
        }
        changed
    }
}

impl Config {
    /// Finds the layout whose fingerprint matches `device_paths` (order-insensitive), if any.
    pub fn layout_for(&self, device_paths: &[String]) -> Option<usize> {
        let mut wanted: Vec<&str> = device_paths.iter().map(String::as_str).collect();
        wanted.sort_unstable();
        self.layouts.iter().position(|l| {
            let mut have: Vec<&str> = l
                .fingerprint
                .iter()
                .map(|m| m.device_path.as_str())
                .collect();
            have.sort_unstable();
            have == wanted
        })
    }

    pub fn fence_mut(&mut self, layout: usize, id: FenceId) -> Option<&mut Fence> {
        self.layouts
            .get_mut(layout)?
            .fences
            .iter_mut()
            .find(|f| f.id == id)
    }

    /// Which fence (if any) in `layout` holds `item`?
    pub fn fence_of_item(&self, layout: usize, item: ItemId) -> Option<FenceId> {
        self.layouts
            .get(layout)?
            .fences
            .iter()
            .find(|f| f.contains_item(item))
            .map(|f| f.id)
    }

    /// Moves `item` into `to` (removing it from any other fence in the layout).
    pub fn assign(
        &mut self,
        layout: usize,
        item: ItemId,
        to: FenceId,
        by: AssignedBy,
    ) -> Option<Assignment> {
        let from = self.fence_of_item(layout, item);
        if from == Some(to) {
            return None;
        }
        let l = self.layouts.get_mut(layout)?;
        let tpos = l.fences.iter().position(|f| f.id == to)?;
        for f in &mut l.fences {
            f.items.retain(|r| r.item_id != item);
        }
        let target = &mut l.fences[tpos];
        let rule_id = match &by {
            AssignedBy::Rule(r) => Some(*r),
            _ => None,
        };
        target.items.push(ItemRef {
            item_id: item,
            manual_index: None,
            assigned_by: by,
        });
        let a = Assignment {
            ts: now_unix(),
            item_id: item,
            from,
            to,
            rule_id,
        };
        if rule_id.is_some() {
            self.undo_log.push(a.clone());
            if self.undo_log.len() > 50 {
                let excess = self.undo_log.len() - 50;
                self.undo_log.drain(..excess);
            }
        }
        Some(a)
    }
}

fn default_true() -> bool {
    true
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_style_loads_legacy_settings_and_roundtrips_independently_of_mode() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json.as_object_mut().unwrap().remove("themeStyle");
        let legacy: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(legacy.theme_style, ThemeStyle::Fluent);
        for mode in [
            ThemeSetting::Light,
            ThemeSetting::Dark,
            ThemeSetting::FollowWindowsMode,
            ThemeSetting::FollowAppMode,
        ] {
            let settings = Settings {
                theme: mode,
                theme_style: ThemeStyle::LiquidGlass,
                ..legacy.clone()
            };
            let json = serde_json::to_value(&settings).unwrap();
            assert_eq!(json["themeStyle"], "liquidGlass");
            assert_eq!(serde_json::from_value::<Settings>(json).unwrap(), settings);
        }
    }

    fn geo() -> NormGeometry {
        NormGeometry {
            monitor: "m".into(),
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 200.0,
            work_w: 1920.0,
            work_h: 1040.0,
            anchor: Anchor::LeftTop,
        }
    }

    #[test]
    fn legacy_materials_migrate_without_changing_layout_or_appearance() {
        let mut config = Config::default();
        let mut fence = Fence::new("工作空间", FenceKind::Virtual, geo());
        fence.appearance = Some(AppearanceOverride {
            backdrop: Some(Backdrop::Acrylic),
            opacity: Some(0.55),
            tint_rgb: Some([12, 34, 56]),
            ..Default::default()
        });
        config.layouts.push(Layout {
            fingerprint: vec![],
            fences: vec![fence],
        });
        config.snapshots.push(Snapshot {
            id: Uuid::new_v4(),
            name: "旧布局".into(),
            ts: 1,
            layouts: config.layouts.clone(),
        });

        for legacy in ["micaLike", "solid", "acrylic"] {
            let mut json = serde_json::to_value(&config).unwrap();
            json["settings"]["backdrop"] = legacy.into();
            json["layouts"][0]["fences"][0]["appearance"]["backdrop"] = legacy.into();
            json["snapshots"][0]["layouts"][0]["fences"][0]["appearance"]["backdrop"] =
                legacy.into();
            let migrated: Config = serde_json::from_value(json).unwrap();
            assert_eq!(migrated.settings.backdrop, Backdrop::Acrylic);
            for layout in [&migrated.layouts[0], &migrated.snapshots[0].layouts[0]] {
                let fence = &layout.fences[0];
                assert_eq!(fence.id, config.layouts[0].fences[0].id);
                assert_eq!(fence.geometry, geo());
                assert_eq!(fence.title, "工作空间");
                assert_eq!(fence.appearance, config.layouts[0].fences[0].appearance);
            }
            let saved = serde_json::to_value(migrated).unwrap();
            assert_eq!(saved["settings"]["backdrop"], "acrylic");
            assert_eq!(
                saved["layouts"][0]["fences"][0]["appearance"]["backdrop"],
                "acrylic"
            );
            assert_eq!(
                saved["snapshots"][0]["layouts"][0]["fences"][0]["appearance"]["backdrop"],
                "acrylic"
            );
        }
    }

    #[test]
    fn roundtrip_json() {
        let mut c = Config::default();
        let mut f = Fence::new("程序", FenceKind::Virtual, geo());
        let item = Item {
            id: Uuid::new_v4(),
            key: ItemKey::from_path("C:/Users/Me/Desktop/Report.docx"),
            origin: Origin::UserDesktop,
            display_name: "Report".into(),
            file_id: None,
            mtime: 1,
            is_folder: false,
            attrs: 0,
            icon_key: IconKey::ByExt(".docx".into()),
            orphaned_since: None,
            size: 0,
            open_count: 0,
        };
        f.items.push(ItemRef {
            item_id: item.id,
            manual_index: Some(0),
            assigned_by: AssignedBy::User,
        });
        c.items.insert(item.id, item);
        c.layouts.push(Layout {
            fingerprint: vec![MonitorIdentity {
                device_path: "m".into(),
                work_dip: [1920.0, 1040.0],
                dpi: 96,
            }],
            fences: vec![f],
        });
        let json = serde_json::to_string_pretty(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.layouts[0].fences[0].title, "程序");
        assert_eq!(back.items.len(), 1);
        assert_eq!(
            back.items.values().next().unwrap().key,
            ItemKey::Path("c:\\users\\me\\desktop\\report.docx".into())
        );
    }

    #[test]
    fn assign_moves_between_fences_and_logs_rule_assignments() {
        let mut c = Config::default();
        let a = Fence::new("A", FenceKind::Virtual, geo());
        let b = Fence::new("B", FenceKind::Inbox, geo());
        let (aid, bid) = (a.id, b.id);
        c.layouts.push(Layout {
            fingerprint: vec![],
            fences: vec![a, b],
        });
        let item = Uuid::new_v4();
        let rule = Uuid::new_v4();
        assert!(c.assign(0, item, bid, AssignedBy::User).is_some());
        assert_eq!(c.fence_of_item(0, item), Some(bid));
        assert!(c.undo_log.is_empty());
        let moved = c.assign(0, item, aid, AssignedBy::Rule(rule)).unwrap();
        assert_eq!(moved.from, Some(bid));
        assert_eq!(c.fence_of_item(0, item), Some(aid));
        assert_eq!(c.undo_log.len(), 1);
        assert!(c.assign(0, item, aid, AssignedBy::User).is_none());
    }

    #[test]
    fn every_tab_can_detach_at_every_position_and_restore_the_group() {
        let orders = [
            vec![0, 1],
            vec![1, 0],
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];
        for order in orders {
            for active_index in 0..order.len() {
                for detach_index in 0..order.len() {
                    for rolled in [false, true] {
                        let mut fences: Vec<_> = (0..order.len())
                            .map(|i| {
                                let mut f =
                                    Fence::new(&format!("Fence {i}"), FenceKind::Virtual, geo());
                                f.geometry.x = 100.0 * i as f32;
                                f.geometry.w = 240.0 + 40.0 * i as f32;
                                f.view.auto_height = i == 1;
                                f.items.push(ItemRef {
                                    item_id: Uuid::new_v4(),
                                    manual_index: None,
                                    assigned_by: AssignedBy::User,
                                });
                                f
                            })
                            .collect();
                        let ids: Vec<_> = fences.iter().map(|f| f.id).collect();
                        for f in &mut fences[1..] {
                            f.tab_host = Some(ids[0]);
                        }
                        let strip: Vec<_> = order.iter().map(|&i| ids[i]).collect();
                        let active = strip[active_index];
                        fences[0].tab_order = strip.clone();
                        fences[0].active_tab = (active != ids[0]).then_some(active);
                        fences[0].rolled_up = rolled;
                        let before = fences.clone();
                        let mut layout = Layout {
                            fingerprint: vec![],
                            fences,
                        };
                        let tab = strip[detach_index];
                        let change = layout.detach_tab(tab).unwrap();
                        let remaining: Vec<_> =
                            strip.iter().copied().filter(|id| *id != tab).collect();
                        let root = if tab == ids[0] { remaining[0] } else { ids[0] };
                        assert_eq!(change.source_host, ids[0]);
                        assert_eq!(change.remaining_host, root);
                        assert_eq!(layout.host_of(tab), tab);
                        assert_eq!(layout.tabs_of(tab), vec![tab]);
                        assert_eq!(layout.tabs_of(root), remaining);
                        for id in &remaining {
                            assert_eq!(layout.host_of(*id), root);
                        }
                        let next = if active == tab {
                            remaining[detach_index.min(remaining.len() - 1)]
                        } else {
                            active
                        };
                        assert_eq!(layout.active_tab_of(root), next);
                        let group = layout.fences.iter().find(|f| f.id == root).unwrap();
                        assert_eq!(group.geometry, before[0].geometry);
                        assert_eq!(group.rolled_up, rolled);
                        assert_eq!(group.view.auto_height, before[0].view.auto_height);
                        for old in &before {
                            let current = layout.fences.iter().find(|f| f.id == old.id).unwrap();
                            assert_eq!(current.title, old.title);
                            assert_eq!(current.items, old.items);
                            assert_eq!(current.source, old.source);
                        }
                        assert!(
                            !layout.normalize_tabs(),
                            "detach must already produce a valid graph"
                        );
                        let persisted: Layout =
                            serde_json::from_str(&serde_json::to_string(&layout).unwrap()).unwrap();
                        assert_eq!(persisted.tabs_of(root), remaining);
                        assert!(layout.cancel_tab_detach(&change));
                        assert_eq!(
                            layout.fences, before,
                            "cancel must restore IDs, order, active tab and geometry"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn detach_undo_preserves_content_edits_and_rejects_a_later_merge() {
        let mut a = Fence::new("A", FenceKind::Virtual, geo());
        let mut b = Fence::new("B", FenceKind::Virtual, geo());
        let (aid, bid) = (a.id, b.id);
        b.tab_host = Some(aid);
        a.tab_order = vec![aid, bid];
        let mut layout = Layout {
            fingerprint: vec![],
            fences: vec![a, b],
        };
        let change = layout.detach_tab(aid).unwrap();
        layout.fences[0].title = "Renamed while dragging".into();
        assert!(layout.cancel_tab_detach(&change));
        assert_eq!(layout.fences[0].title, "Renamed while dragging");
        let change = layout.detach_tab(aid).unwrap();
        let other = Fence::new("Other group", FenceKind::Virtual, geo());
        let other_id = other.id;
        layout.fences.push(other);
        layout.fences[1].tab_host = Some(other_id);
        assert!(
            !layout.cancel_tab_detach(&change),
            "a later merge of the remaining group owns its state"
        );
        layout.fences[1].tab_host = None;
        layout.fences[0].tab_host = Some(bid);
        assert!(!layout.cancel_tab_detach(&change));
        let mut single = Layout {
            fingerprint: vec![],
            fences: vec![Fence::new("only", FenceKind::Virtual, geo())],
        };
        assert!(single.detach_tab(single.fences[0].id).is_none());
    }

    #[test]
    fn tabs_normalize_dangling_hosts_and_chains() {
        let mut a = Fence::new("A", FenceKind::Virtual, geo());
        let mut b = Fence::new("B", FenceKind::Virtual, geo());
        let mut c = Fence::new("C", FenceKind::Virtual, geo());
        let (aid, bid, cid) = (a.id, b.id, c.id);
        b.tab_host = Some(aid);
        c.tab_host = Some(bid); // chain: c hosted by a tab
        a.active_tab = Some(cid); // not (yet) a direct tab of a
        let mut l = Layout {
            fingerprint: vec![],
            fences: vec![a, b, c],
        };
        assert!(l.normalize_tabs());
        assert_eq!(l.tabs_of(aid), vec![aid, bid, cid]);
        assert_eq!(l.host_of(cid), aid);
        assert_eq!(l.active_tab_of(aid), cid);
        // Deleting the host leaves its tabs dangling → they become windows again.
        l.fences.remove(0);
        assert!(l.normalize_tabs());
        assert_eq!(l.host_of(bid), bid);
        assert_eq!(l.tabs_of(bid), vec![bid]);
        assert!(!l.normalize_tabs());
    }

    #[test]
    fn tabs_of_honours_tab_order_and_normalize_prunes_stale_ids() {
        let a = Fence::new("A", FenceKind::Virtual, geo());
        let mut b = Fence::new("B", FenceKind::Virtual, geo());
        let mut c = Fence::new("C", FenceKind::Virtual, geo());
        let (aid, bid, cid) = (a.id, b.id, c.id);
        b.tab_host = Some(aid);
        c.tab_host = Some(aid);
        let mut l = Layout {
            fingerprint: vec![],
            fences: vec![a, b, c],
        };
        assert_eq!(l.tabs_of(aid), vec![aid, bid, cid]);
        // Host may sit anywhere in the strip.
        assert!(l.reorder_tab(aid, aid, 2));
        assert_eq!(l.tabs_of(aid), vec![bid, cid, aid]);
        // Out-of-range clamps to the end; no-op reorder reports false.
        assert!(l.reorder_tab(aid, bid, 99));
        assert_eq!(l.tabs_of(aid), vec![cid, aid, bid]);
        assert!(!l.reorder_tab(aid, bid, 5));
        assert!(!l.reorder_tab(aid, Uuid::new_v4(), 0));
        // Stale ids are pruned by normalize; a detached tab drops out of the order.
        l.fences[0].tab_order.push(Uuid::new_v4());
        l.fences[2].tab_host = None;
        assert!(l.normalize_tabs());
        assert_eq!(l.fences[0].tab_order, vec![aid, bid]);
        assert_eq!(l.tabs_of(aid), vec![aid, bid]);
        assert!(!l.normalize_tabs());
    }

    #[test]
    fn layout_lookup_is_order_insensitive() {
        let mut c = Config::default();
        c.layouts.push(Layout {
            fingerprint: vec![
                MonitorIdentity {
                    device_path: "b".into(),
                    work_dip: [1.0, 1.0],
                    dpi: 96,
                },
                MonitorIdentity {
                    device_path: "a".into(),
                    work_dip: [1.0, 1.0],
                    dpi: 96,
                },
            ],
            fences: vec![],
        });
        assert_eq!(c.layout_for(&["a".into(), "b".into()]), Some(0));
        assert_eq!(c.layout_for(&["a".into()]), None);
    }
}

#[cfg(test)]
mod namespace_key_tests {
    use super::*;

    #[test]
    fn namespace_keys_are_detected_after_normalization() {
        let bin = ItemKey::from_path("::{645FF040-5081-101B-9F08-00AA002F954E}");
        assert!(bin.is_namespace());
        assert_eq!(
            bin.as_path(),
            Some("::{645ff040-5081-101b-9f08-00aa002f954e}")
        );
        assert!(!ItemKey::from_path(r"C:\Users\me\Desktop\a.txt").is_namespace());
        assert!(!ItemKey::Pidl("AAAA".into()).is_namespace());
    }
}
