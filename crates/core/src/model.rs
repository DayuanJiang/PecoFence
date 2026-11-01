//! Persistent data model (plan §7). Pure data + serde; no Windows types.

use crate::geometry::{WorkArea, monitor_model};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

/// Published JSON Schema of `config.json` (`pecofence-cli describe --schema Config` prints the
/// same document). Written into the file's `$schema` field so editors validate and complete it.
pub const CONFIG_SCHEMA_URL: &str = "https://pecofence.jiang.jp/schema/config.json";

pub type FenceId = Uuid;
pub type ItemId = Uuid;
pub type RuleId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// `$schema`: URL of the JSON Schema this file follows ([`CONFIG_SCHEMA_URL`]); informative
    /// only, the app ignores its value.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub schema_version: u32,
    pub settings: Settings,
    /// Global item table keyed by id.
    pub items: HashMap<ItemId, Item>,
    /// One layout per monitor configuration.
    pub layouts: Vec<Layout>,
    #[serde(with = "crate::rules::saved")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::rules::RuleSet"))]
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
            schema: None,
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: Uuid,
    pub name: String,
    pub ts: i64,
    pub layouts: Vec<Layout>,
}

pub const MAX_SNAPSHOTS: usize = 20;

/// Settings the file keeps but no version acts on (planned options that were never built).
/// They stay in `config.json` because older versions require them to read the file; the CLI
/// hides them from `settings get` / the schema and refuses to set them.
pub const UNUSED_SETTINGS: &[&str] = &[
    "showRealIconsWhenFencesHidden",
    "telemetry",
    "quickHide.scope",
    "quickHide.alwaysShowAtStartup",
    "quickHide.delayMs",
    "quickHide.autoHideIdleSec",
    "quickHide.autoShowOnUse",
    "quickHide.wallpaperEngineClassWhitelist",
    "rollUp.doubleClickTitle",
    "rollUp.autoOnScreenEdge",
    "rollUp.hoverOpenMs",
    "rollUp.closeGraceMs",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "crate::i18n::Language::legacy_default")]
    pub language: crate::i18n::Language,
    pub backdrop: Backdrop,
    pub theme: ThemeSetting,
    /// Material style is independent of the light/dark preference.
    #[serde(default)]
    pub theme_style: ThemeStyle,
    /// A theme file in the configuration's `themes` folder, by name without `.json`. While it
    /// loads, its `base` takes the place of `themeStyle`.
    #[serde(default)]
    pub custom_theme: Option<String>,
    /// Colour of the item names in every fence; fence titles have their own `titleColor`.
    #[serde(default)]
    pub label_color: LabelColor,
    /// Where every fence's title (or tab strip) sits in its title row.
    #[serde(default)]
    pub title_align: TitleAlign,
    /// Where a fence with two or more tabs shows them.
    #[serde(default)]
    pub tab_bar: TabBar,
    pub icon_size: u32,
    pub quick_hide: QuickHideSettings,
    pub roll_up: RollUpSettings,
    pub show_desktop: ShowDesktopSetting,
    pub hide_real_icons: bool,
    /// With `hideRealIcons`, Windows' desktop icons stay hidden when PecoFence exits or the
    /// session ends, so they do not show for a moment at the next sign-in before PecoFence
    /// starts. A crash still shows them.
    #[serde(default)]
    pub keep_icons_hidden: bool,
    /// While `hideRealIcons` is off, the inbox fence has no window: the items no rule files
    /// show only among Windows' own desktop icons.
    #[serde(default)]
    pub hide_inbox_when_icons_shown: bool,
    /// A rectangle drawn on the empty desktop that touches no fence offers 「在此新建栅栏」.
    #[serde(default = "default_true")]
    pub offer_new_fence: bool,
    /// Unused (see [`UNUSED_SETTINGS`]).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub show_real_icons_when_fences_hidden: bool,
    pub snapping: SnappingSettings,
    pub zorder: ZOrderSetting,
    pub autostart: bool,
    /// Unused (see [`UNUSED_SETTINGS`]).
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub telemetry: bool,
    /// Fences "Peek": a hotkey floats every fence above the current windows.
    #[serde(default)]
    pub peek: PeekSettings,
    /// 「临时显示 Windows 桌面」: a shortcut, written like `peek.customHotkey` ("Ctrl+Alt+D"),
    /// that hides every fence and shows the Windows desktop icons PecoFence hid, until it is
    /// pressed again. Nothing of that is saved. `null` (the default) = no shortcut; the tray
    /// menu and `pecofence-cli windows-desktop` still switch. Text that is not a usable
    /// shortcut is ignored the same way.
    #[serde(default)]
    pub windows_desktop_hotkey: Option<String>,
    #[serde(default)]
    pub icons: IconSettings,
    /// The user's Desktop folder as last seen, so a moved desktop (OneDrive, another drive) can
    /// have its item records re-pointed instead of orphaned.
    #[serde(default)]
    pub desktop_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PeekSettings {
    pub enabled: bool,
    /// Dim everything behind the fences while peeking.
    pub dim: bool,
    /// The shortcut as versions up to 0.2.0 store it; `customHotkey` replaces it when set.
    pub hotkey: PeekHotkey,
    /// Any other shortcut, as text such as `"Ctrl+Shift+F"` or `"Win+Alt+Space"`: modifiers
    /// (Win, Ctrl, Alt, Shift; Ctrl, Alt or Win needed) and a letter, digit, F1 to F24, Num0 to
    /// Num9, Space, Enter, Tab, Esc, Backspace, Insert, Delete, Home, End, PageUp, PageDown or
    /// an arrow key (Left, Up, Right, Down). `null` = use `hotkey`. Text that is not a usable
    /// shortcut is ignored the same way.
    #[serde(default)]
    pub custom_hotkey: Option<String>,
}

impl Default for PeekSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            dim: true,
            hotkey: PeekHotkey::CtrlAltSpace,
            custom_hotkey: None,
        }
    }
}

impl PeekSettings {
    /// The shortcut the user chose: `customHotkey` when it holds a usable one, else `hotkey`.
    pub fn chosen_hotkey(&self) -> crate::hotkey::Hotkey {
        self.custom_hotkey
            .as_deref()
            .and_then(|t| crate::hotkey::Hotkey::parse(t).and_then(|h| h.check()).ok())
            .unwrap_or_else(|| self.hotkey.combo())
    }

    /// Makes `h` the chosen shortcut. Ctrl+Alt+Space is written as `hotkey` alone, so the file
    /// reads the same in every version; any other goes to `customHotkey`, with `hotkey` set to
    /// Ctrl+Alt+Space for the versions that do not know that field.
    pub fn set_hotkey(&mut self, h: crate::hotkey::Hotkey) {
        self.hotkey = PeekHotkey::CtrlAltSpace;
        self.custom_hotkey = (h != crate::hotkey::CTRL_ALT_SPACE).then(|| h.to_string());
    }
}

impl Settings {
    /// The 「临时显示 Windows 桌面」 shortcut, when `windowsDesktopHotkey` holds a usable one.
    pub fn windows_desktop_hotkey(&self) -> Option<crate::hotkey::Hotkey> {
        self.windows_desktop_hotkey
            .as_deref()
            .and_then(|t| crate::hotkey::Hotkey::parse(t).and_then(|h| h.check()).ok())
    }
}

/// The shortcuts versions up to 0.2.0 offered. Fences uses Win+Space, but Windows 10 and 11
/// keep Win+Space and Win+Shift+Space for the input switcher even with a single keyboard
/// layout, so a file that names one of them loads with Ctrl+Alt+Space (`config_store::migrate`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum PeekHotkey {
    WinSpace,
    #[default]
    CtrlAltSpace,
    WinShiftSpace,
}

impl PeekHotkey {
    pub fn combo(self) -> crate::hotkey::Hotkey {
        use crate::hotkey::{ALT, CTRL, Hotkey, SHIFT, WIN};
        match self {
            Self::WinSpace => Hotkey::new(WIN, 0x20),
            Self::CtrlAltSpace => Hotkey::new(CTRL | ALT, 0x20),
            Self::WinShiftSpace => Hotkey::new(WIN | SHIFT, 0x20),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: crate::i18n::Language::System,
            backdrop: Backdrop::Acrylic,
            theme: ThemeSetting::FollowWindowsMode,
            theme_style: ThemeStyle::Fluent,
            custom_theme: None,
            label_color: LabelColor::Auto,
            title_align: TitleAlign::Left,
            tab_bar: TabBar::Top,
            icon_size: 48,
            quick_hide: QuickHideSettings::default(),
            roll_up: RollUpSettings::default(),
            show_desktop: ShowDesktopSetting::KeepVisible,
            hide_real_icons: true,
            keep_icons_hidden: false,
            hide_inbox_when_icons_shown: false,
            offer_new_fence: true,
            show_real_icons_when_fences_hidden: false,
            snapping: SnappingSettings::default(),
            zorder: ZOrderSetting::InsertAboveHost,
            autostart: true,
            telemetry: false,
            peek: PeekSettings::default(),
            windows_desktop_hotkey: None,
            icons: IconSettings::default(),
            desktop_path: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Backdrop {
    /// One material for all fences. Older configurations and snapshots still load.
    #[serde(alias = "micaLike", alias = "solid")]
    Acrylic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ThemeSetting {
    FollowWindowsMode,
    FollowAppMode,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ThemeStyle {
    #[default]
    Fluent,
    LiquidGlass,
}

/// Item name colour (「图标名称颜色」). Auto: Liquid Glass picks dark or light text from the
/// wallpaper under each fence, Fluent follows the colour mode. The halo pairs with the text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum LabelColor {
    #[default]
    Auto,
    White,
    Black,
}

/// Title position in the title row (Fences centres it). The chevron zone on the right stays
/// clear whichever is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TitleAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Where a tabbed fence shows its tabs: side by side in the title row, or listed in a column
/// on its left that takes the title row's place (rolled up, the fence shrinks to the column).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TabBar {
    #[default]
    Top,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ShowDesktopSetting {
    KeepVisible,
    HideWithDesktop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ZOrderSetting {
    InsertAboveHost,
    HwndBottom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct QuickHideSettings {
    pub enabled: bool,
    // The rest is unused (see [`UNUSED_SETTINGS`]).
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub scope: QuickHideScope,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub always_show_at_startup: bool,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub delay_ms: u32,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub auto_hide_idle_sec: u32,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub auto_show_on_use: bool,
    #[cfg_attr(feature = "schema", schemars(skip))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum QuickHideScope {
    All,
    LooseOnly,
    FencesOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct RollUpSettings {
    /// Unused (see [`UNUSED_SETTINGS`]), like `auto_on_screen_edge`, `hover_open_ms` and
    /// `close_grace_ms`.
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub double_click_title: bool,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub auto_on_screen_edge: bool,
    pub hover_peek: bool,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub hover_open_ms: u32,
    #[cfg_attr(feature = "schema", schemars(skip))]
    pub close_grace_ms: u32,
    /// A rolled fence expands on a single title click instead of on hover (Fences "require a
    /// click to expand"). Hover peek is ignored while this is on.
    #[serde(default)]
    pub click_to_expand: bool,
    /// Draw the title (and tabs) only while the mouse is over the fence; a fence's
    /// `appearance.titleOnHover` overrides it.
    #[serde(default)]
    pub title_on_hover: bool,
    /// Show the scrollbar only while the mouse is inside the fence or right after scrolling.
    #[serde(default)]
    pub hide_inactive_scrollbar: bool,
    /// While a fence is expanded (a click or a hover peek), the fences stacked below it slide down
    /// out of its way and slide back when it rolls up. The saved positions stay where they were.
    #[serde(default = "default_true")]
    pub push_neighbors: bool,
    /// Resting the pointer on a tab of a tabbed fence for the hover delay switches to it, as a
    /// click would. On by default; a configuration that already says `false` keeps it.
    #[serde(default = "default_true")]
    pub switch_tab_on_hover: bool,
    /// No item count on a rolled title and no roll chevron: the title bar shows the name only.
    /// Double-clicking the title and the fence menu still roll the fence up and down.
    #[serde(default)]
    pub hide_count_and_chevron: bool,
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
            push_neighbors: true,
            switch_tab_on_hover: true,
            hide_count_and_chevron: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SnappingSettings {
    /// Dragged fences snap to other fences, the screen edges and the screen's centre lines
    /// (Alt held: free).
    pub enabled: bool,
    /// Gap, in DIPs, fences keep to each other and the screen edges while snapping.
    pub gap_px: i32,
    /// Resizing keeps whole icon columns and rows.
    pub size_to_cells: bool,
    /// While snapping, guide lines along the edges a dragged or resized fence lines up with
    /// and the screen centre lines it snapped to.
    pub guide_lines: bool,
}

impl Default for SnappingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            gap_px: 8,
            size_to_cells: false,
            guide_lines: true,
        }
    }
}

/// Identifies a monitor across sessions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonitorIdentity {
    /// The monitor's id: its device instance (`GSM7787#5&2C948443&0&UID24832`), else its EDID
    /// model and connector (`GSM7787#5.0`), else `\\.\DISPLAYn` (all that files from 0.1.3 and
    /// earlier have).
    pub device_path: String,
    /// GDI name (`\\.\DISPLAY1`) at save time.
    #[serde(default)]
    pub gdi_name: String,
    /// Work-area size in DIPs at save time.
    pub work_dip: [f32; 2],
    pub dpi: u32,
    /// Resolution in physical px (taskbar included). With `dpi` it is part of the set: the
    /// same monitors at another resolution or scale have a layout of their own. `[0, 0]` in
    /// layouts saved before that; such a layout takes the resolution it is next shown at.
    #[serde(default)]
    pub screen_px: [i32; 2],
}

impl MonitorIdentity {
    pub fn of(work: &WorkArea) -> Self {
        Self {
            device_path: work.device_path.clone(),
            gdi_name: work.gdi_name.clone(),
            work_dip: [work.width_dip(), work.height_dip()],
            dpi: work.dpi,
            screen_px: work.screen_px(),
        }
    }

    fn knows_mode(&self) -> bool {
        self.screen_px != [0, 0]
    }

    /// `now` runs at the resolution and scale this was saved at (or it was saved without).
    fn same_mode(&self, now: &WorkArea) -> bool {
        !self.knows_mode() || (self.screen_px == now.screen_px() && self.dpi == now.dpi)
    }

    fn gdi_name(&self) -> &str {
        if !self.gdi_name.is_empty() {
            &self.gdi_name
        } else if self.device_path.starts_with('\\') {
            &self.device_path
        } else {
            ""
        }
    }
}

/// Where the fences sit on one set of monitors. Every set shows the same fences (titles,
/// items, tabs, views): the layout shown last holds them as they are, and showing another
/// layout carries them over, keeping that layout's geometry and roll state for each fence it
/// had.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    /// Monitor identities; the layout applies when they are the connected monitors (each found
    /// by id, model or GDI name).
    pub fingerprint: Vec<MonitorIdentity>,
    pub fences: Vec<Fence>,
    /// Raised each time the layout is shown; the highest one is the layout shown last. 0 in
    /// files from 0.1.3 and earlier, which kept separate fences per monitor set.
    #[serde(default)]
    pub shown: u64,
}

/// [`Config::layout_for`]'s answer.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutMatch {
    pub index: usize,
    /// Per saved monitor, the index of the connected one it is.
    pub pairs: Vec<usize>,
    /// Saved id → connected id, for the monitors found under another id.
    pub renamed: Vec<(String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FenceKind {
    Virtual,
    /// The system "桌面" fence that receives everything unassigned. Exactly one per layout.
    Inbox,
    FolderPortal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// "按时间分组": items under 今天 / 昨天 / 本周 / 本月 / 更早 section headers (all
    /// layouts). Implies `sort == Date`.
    #[serde(default)]
    pub group_by_date: bool,
    /// Folders first: folders stay above files in any order but manual and date groups, also
    /// when reversed. Manual order and Group by date turn it off.
    #[serde(default)]
    pub folders_first: bool,
    /// 图标对齐 (#154): where a row of icons that does not fill the width sits (icons view only).
    #[serde(default)]
    pub icon_align: IconAlign,
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
            group_by_date: false,
            folders_first: false,
            icon_align: IconAlign::Left,
        }
    }
}

/// 图标对齐 of the icons view: a row that does not fill the fence width starts at the left
/// edge, or sits in the middle with the leftover width split on both sides (the last, shorter
/// row too). List and details rows always start at the left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum IconAlign {
    #[default]
    Left,
    Center,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AppearanceOverride {
    /// Colour wash over the glass (Fences per-fence colour).
    pub tint_rgb: Option<[u8; 3]>,
    /// Glass opacity multiplier (None = 1): 0.55 更透明, 1.6 更厚实, 0 全透明 — no plate at all
    /// until the pointer is over the fence.
    pub opacity: Option<f32>,
    pub backdrop: Option<Backdrop>,
    /// Title text colour (None = theme text colour).
    #[serde(default)]
    pub title_rgb: Option<[u8; 3]>,
    #[serde(default)]
    pub title_size: Option<TitleSize>,
    /// The title row folds away until the pointer is over the fence (None = the global
    /// `rollUp.titleOnHover`).
    #[serde(default)]
    pub title_on_hover: Option<bool>,
    /// The tint washes only the title row instead of the whole glass.
    #[serde(default)]
    pub tint_title_only: bool,
    /// This fence's bar in a tab strip (None = its tint).
    #[serde(default)]
    pub tab_rgb: Option<[u8; 3]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TitleSize {
    Small,
    #[default]
    Normal,
    Large,
    ExtraLarge,
}

/// Distance between icons in the grid (Fences "icon spacing").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Spacing {
    Compact,
    #[default]
    Normal,
    Loose,
}

/// Global icon rendering tweaks (Fences "Icon Tint" / "Chameleon").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct IconSettings {
    /// Colourise every icon toward this colour.
    pub tint_rgb: Option<[u8; 3]>,
    /// 0..1 — how far toward the tint.
    pub tint_strength: f32,
    /// Chameleon: icons desaturate and fade so they blend with the backdrop.
    pub chameleon: bool,
    /// Windows' shortcut arrow at the bottom-left of the items Explorer draws it on.
    #[serde(default = "default_true")]
    pub shortcut_arrows: bool,
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            tint_rgb: None,
            tint_strength: 0.6,
            chameleon: false,
            shortcut_arrows: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Any other fence: double-clicking a folder, or a shortcut to a folder, shows that folder
    /// inside the fence until the fence goes up from it; off (the default) = open it in Explorer.
    #[serde(default)]
    pub open_folders_inside: bool,
    /// Folder portal: hide the folder glyph before the title.
    #[serde(default)]
    pub hide_title_icon: bool,
    /// No window while it holds no items (为空时自动隐藏; a portal only at its root folder, a
    /// tab or a host of tabs never); the next item to land in it brings it back at its saved
    /// spot.
    #[serde(default)]
    pub hide_when_empty: bool,
    /// Expand upward (向上展开): the bottom edge stays put while the fence rolls up, expands or
    /// changes height, so a fence resting on the taskbar grows away from it. `geometry` is still
    /// the expanded rect; rolled up, the title row sits at its bottom.
    #[serde(default)]
    pub expand_upward: bool,
    /// With `expand_upward`: the bottom edge is the one that stays put (it last opened upward);
    /// rolled up, its title row sits at the bottom of `geometry` instead of at the top.
    #[serde(default)]
    pub bottom_anchored: bool,
    /// 像桌面一样排列 (the inbox only): no plate, title row, rim or shadow; the icons stand
    /// down columns from the top-left of the monitor's work area, around the other fences,
    /// like the Windows desktop with 自动排列图标.
    #[serde(default)]
    pub desktop_layout: bool,
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
            open_folders_inside: false,
            hide_title_icon: false,
            hide_when_empty: false,
            expand_upward: false,
            bottom_anchored: false,
            desktop_layout: false,
            items: Vec::new(),
        }
    }

    pub fn contains_item(&self, id: ItemId) -> bool {
        self.items.iter().any(|r| r.item_id == id)
    }

    /// A double-clicked folder opens inside the fence: `portal_navigate` for a folder portal,
    /// `open_folders_inside` for any other fence.
    pub fn navigates_folders(&self) -> bool {
        if self.kind == FenceKind::FolderPortal {
            self.portal_navigate
        } else {
            self.open_folders_inside
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum AssignedBy {
    User,
    Rule(RuleId),
    Migration,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ItemRef {
    pub item_id: ItemId,
    #[serde(default)]
    pub manual_index: Option<u32>,
    pub assigned_by: AssignedBy,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
        // The key is the path the shell opens while an item has no `Item::path` yet: a letter
        // whose lowercase is more than one char ('İ' → "i\u{307}") stays as it is, or NTFS
        // would not find the file.
        let mut s: String = path
            .replace('/', "\\")
            .chars()
            .map(|c| {
                let mut lower = c.to_lowercase();
                match (lower.next(), lower.next()) {
                    (Some(l), None) => l,
                    _ => c,
                }
            })
            .collect();
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    #[default]
    UserDesktop,
    PublicDesktop,
    Namespace,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum IconKey {
    /// Icon shared by extension (documents, most files).
    ByExt(String),
    /// Icon specific to this file (.exe, .lnk, .ico, folders with custom icons, images).
    ByContent { path: String, mtime: i64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Unix seconds of the last launch from a fence (the "闲置天数" rule condition).
    #[serde(default)]
    pub last_opened: Option<i64>,
    /// The path as spelled on disk (`ABC.zip`); `key` is its case-folded form for matching.
    /// Missing in older files until the next desktop sync.
    #[serde(default)]
    pub path: Option<String>,
}

impl Item {
    /// See [`ItemKey::is_namespace`].
    pub fn is_namespace(&self) -> bool {
        self.key.is_namespace()
    }

    /// The path to hand to the shell and other programs (open, drag, copy, CLI): the on-disk
    /// spelling, else the case-folded key.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref().or_else(|| self.key.as_path())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// 向上展开, and which end of `geometry` the rolled title row sits at.
    expand_upward: bool,
    bottom_anchored: bool,
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
            expand_upward: fence.expand_upward,
            bottom_anchored: fence.bottom_anchored,
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
        fence.expand_upward = self.expand_upward;
        fence.bottom_anchored = self.bottom_anchored;
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
                        fence.expand_upward = group.expand_upward;
                        fence.bottom_anchored = group.bottom_anchored;
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
        // Pass 1b: a fence whose host chain never reaches an unhosted fence sits in a cycle
        // (only a hand-edited file has one); pass 2 would never end on it.
        for i in 0..self.fences.len() {
            let mut cur = self.fences[i].tab_host;
            let mut steps = 0;
            while let Some(h) = cur
                && steps <= ids.len()
            {
                cur = self
                    .fences
                    .iter()
                    .find(|f| f.id == h)
                    .and_then(|f| f.tab_host);
                steps += 1;
            }
            if cur.is_some() {
                self.fences[i].tab_host = None;
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

/// The layout for the connected `monitors`: one whose saved monitors are all found among them,
/// one each. A saved monitor is found by its id; else by model when exactly one saved and one
/// connected monitor left are that model (another port or a dock gives a monitor a new
/// instance); else, when either side has no EDID model (files from 0.1.3 and earlier saved
/// only GDI names), by GDI name. Each must also run at the resolution and scale it was saved
/// at. The layout with the most monitors found by id wins, then the one that knows the
/// resolutions over one saved without them.
pub fn layout_for(layouts: &[Layout], monitors: &[WorkArea]) -> Option<LayoutMatch> {
    let mut best: Option<((usize, usize), LayoutMatch)> = None;
    for (index, l) in layouts.iter().enumerate() {
        let Some((pairs, by_id)) = pair_monitors(&l.fingerprint, monitors) else {
            continue;
        };
        let saved = l.fingerprint.iter().zip(&pairs);
        if !saved.clone().all(|(m, &j)| m.same_mode(&monitors[j])) {
            continue;
        }
        let score = (
            by_id,
            l.fingerprint.iter().filter(|m| m.knows_mode()).count(),
        );
        if best.as_ref().is_some_and(|(b, _)| *b >= score) {
            continue;
        }
        let renamed = saved
            .filter(|(m, j)| m.device_path != monitors[**j].device_path)
            .map(|(m, &j)| (m.device_path.clone(), monitors[j].device_path.clone()))
            .collect();
        best = Some((
            score,
            LayoutMatch {
                index,
                pairs,
                renamed,
            },
        ));
    }
    best.map(|(_, m)| m)
}

/// Per saved monitor, the index of the connected one it is (see [`layout_for`]), and how many
/// were found by id. `None` unless every saved and every connected monitor pairs up.
fn pair_monitors(saved: &[MonitorIdentity], now: &[WorkArea]) -> Option<(Vec<usize>, usize)> {
    if saved.len() != now.len() {
        return None;
    }
    let mut pairs: Vec<Option<usize>> = vec![None; saved.len()];
    let mut taken = vec![false; now.len()];
    for (i, s) in saved.iter().enumerate() {
        if let Some(j) = (0..now.len()).find(|&j| !taken[j] && now[j].device_path == s.device_path)
        {
            (pairs[i], taken[j]) = (Some(j), true);
        }
    }
    let by_id = pairs.iter().flatten().count();
    for i in 0..saved.len() {
        let Some(model) = monitor_model(&saved[i].device_path).filter(|_| pairs[i].is_none())
        else {
            continue;
        };
        let rivals = (0..saved.len())
            .filter(|&k| pairs[k].is_none() && monitor_model(&saved[k].device_path) == Some(model))
            .count();
        let found: Vec<usize> = (0..now.len())
            .filter(|&j| !taken[j] && now[j].model() == Some(model))
            .collect();
        if let ([j], 1) = (found.as_slice(), rivals) {
            (pairs[i], taken[*j]) = (Some(*j), true);
        }
    }
    for i in 0..saved.len() {
        let gdi = saved[i].gdi_name();
        if pairs[i].is_some() || gdi.is_empty() {
            continue;
        }
        let saved_model = monitor_model(&saved[i].device_path);
        if let Some(j) = (0..now.len()).find(|&j| {
            !taken[j]
                && now[j].gdi_name == gdi
                && (saved_model.is_none() || now[j].model().is_none())
        }) {
            (pairs[i], taken[j]) = (Some(j), true);
        }
    }
    let pairs: Option<Vec<usize>> = pairs.into_iter().collect();
    pairs.map(|p| (p, by_id))
}

/// The layout shown last (it holds the fences as they are), `None` in files from 0.1.3 and
/// earlier.
pub fn last_shown(layouts: &[Layout]) -> Option<usize> {
    layouts
        .iter()
        .enumerate()
        .filter(|(_, l)| l.shown > 0)
        .max_by_key(|(_, l)| l.shown)
        .map(|(i, _)| i)
}

impl Config {
    /// See [`layout_for`].
    pub fn layout_for(&self, monitors: &[WorkArea]) -> Option<LayoutMatch> {
        layout_for(&self.layouts, monitors)
    }

    /// See [`last_shown`].
    pub fn last_shown(&self) -> Option<usize> {
        last_shown(&self.layouts)
    }

    /// Makes `index` the layout shown last; false when it already was.
    pub fn mark_shown(&mut self, index: usize) -> bool {
        let top = self.layouts.iter().map(|l| l.shown).max().unwrap_or(0);
        let tied = self
            .layouts
            .iter()
            .enumerate()
            .any(|(i, l)| i != index && l.shown == top);
        if top > 0 && self.layouts[index].shown == top && !tied {
            return false;
        }
        self.layouts[index].shown = top + 1;
        true
    }

    /// Gives layout `to` the fences of layout `from` as they are, each with the geometry and
    /// roll state `to` saved for it. Returns the fences `to` had no spot for: they keep the one
    /// they had in `from`.
    pub fn carry_fences(&mut self, from: usize, to: usize) -> Vec<FenceId> {
        let spots: HashMap<FenceId, (NormGeometry, bool, f32)> = self.layouts[to]
            .fences
            .iter()
            .map(|f| (f.id, (f.geometry.clone(), f.rolled_up, f.expanded_h)))
            .collect();
        let mut fences = self.layouts[from].fences.clone();
        let mut unplaced = Vec::new();
        for f in &mut fences {
            match spots.get(&f.id) {
                Some((geometry, rolled_up, expanded_h)) => {
                    f.geometry = geometry.clone();
                    f.rolled_up = *rolled_up;
                    f.expanded_h = *expanded_h;
                }
                None => unplaced.push(f.id),
            }
        }
        self.layouts[to].fences = fences;
        unplaced
    }

    /// Files from 0.1.3 and earlier kept separate fences per monitor set. Brings the other
    /// layouts' fences into `into` so that none is lost: a fence only another set has is added
    /// (so one deleted on just one set comes back), and an item `into` holds only because it
    /// was routed there goes where another set had it placed by hand.
    pub fn merge_layout_fences(&mut self, into: usize) {
        let others: Vec<Layout> = (0..self.layouts.len())
            .filter(|&i| i != into)
            .map(|i| self.layouts[i].clone())
            .collect();
        let target = &mut self.layouts[into].fences;
        let has_inbox = target.iter().any(|f| f.kind == FenceKind::Inbox);
        for f in others.iter().flat_map(|l| &l.fences) {
            if target.iter().any(|t| t.id == f.id) || (f.kind == FenceKind::Inbox && has_inbox) {
                continue;
            }
            let mut f = f.clone();
            f.items.clear();
            target.push(f);
        }
        for f in others.iter().flat_map(|l| &l.fences) {
            if !target.iter().any(|t| t.id == f.id) {
                continue;
            }
            for r in f.items.iter().filter(|r| r.assigned_by == AssignedBy::User) {
                let held = target
                    .iter()
                    .flat_map(|t| &t.items)
                    .find(|t| t.item_id == r.item_id);
                if held.is_some_and(|h| h.assigned_by == AssignedBy::User) {
                    continue;
                }
                for t in target.iter_mut() {
                    t.items.retain(|x| x.item_id != r.item_id);
                }
                if let Some(t) = target.iter_mut().find(|t| t.id == f.id) {
                    t.items.push(r.clone());
                }
            }
        }
    }

    /// Drops what a broken or hand-merged file lists twice in a layout, keeping the first: a
    /// fence id seen before, and a membership of an item that an earlier fence (or an earlier
    /// entry of the same one) holds already. True when something was dropped.
    pub fn drop_duplicates(&mut self) -> bool {
        let mut changed = false;
        for layout in &mut self.layouts {
            let mut fences = HashSet::new();
            let before = layout.fences.len();
            layout.fences.retain(|f| fences.insert(f.id));
            changed |= layout.fences.len() != before;
            let mut items = HashSet::new();
            for f in &mut layout.fences {
                let before = f.items.len();
                f.items.retain(|r| items.insert(r.item_id));
                changed |= f.items.len() != before;
            }
        }
        changed
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
    fn item_keys_still_name_the_file() {
        // 'İ' lower-cases to "i\u{307}", a spelling NTFS never matches back.
        let key = ItemKey::from_path("C:/Users/İbrahim/Desktop/Plan.TXT");
        assert_eq!(key.as_path(), Some("c:\\users\\İbrahim\\desktop\\plan.txt"));
    }

    /// Feedback #25: programs got `abc.zip` for `ABC.zip`. The on-disk spelling goes out; an
    /// item saved without it falls back to the key.
    #[test]
    fn items_hand_out_the_on_disk_spelling() {
        let json = r#"{"id":"6f1c2a4e-0000-4000-8000-000000000001","key":{"path":"c:\\d\\abc.zip"},
            "origin":"userDesktop","displayName":"ABC","mtime":1,"isFolder":false,"attrs":0,
            "iconKey":{"byExt":".zip"}}"#;
        let mut item: Item = serde_json::from_str(json).unwrap();
        assert_eq!(item.path(), Some("c:\\d\\abc.zip"));
        item.path = Some(r"C:\D\ABC.zip".into());
        assert_eq!(item.path(), Some(r"C:\D\ABC.zip"));
        assert_eq!(item.key, ItemKey::from_path(r"C:\D\ABC.zip"));
        let back: Item = serde_json::from_value(serde_json::to_value(&item).unwrap()).unwrap();
        assert_eq!(back.path(), Some(r"C:\D\ABC.zip"));
    }

    #[test]
    fn peek_shortcuts_read_in_both_directions() {
        use crate::hotkey::{CTRL_ALT_SPACE, Hotkey};
        // What 0.2.0 reads of `peek` (it ignores fields it does not know).
        #[derive(Deserialize)]
        struct PeekV020 {
            hotkey: PeekHotkey,
        }
        let win_alt_space = Hotkey::parse("Win+Alt+Space").unwrap();
        let mut peek = PeekSettings::default();
        peek.set_hotkey(win_alt_space);
        let json = serde_json::to_value(&peek).unwrap();
        assert_eq!(json["customHotkey"], "Win+Alt+Space");
        let old: PeekV020 = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(old.hotkey, PeekHotkey::CtrlAltSpace);
        let back: PeekSettings = serde_json::from_value(json).unwrap();
        assert_eq!(back.chosen_hotkey(), win_alt_space);
        // Ctrl+Alt+Space is stored the way 0.2.0 stores it.
        peek.set_hotkey(CTRL_ALT_SPACE);
        assert_eq!(peek, PeekSettings::default());
        assert_eq!(
            serde_json::to_value(&peek).unwrap()["customHotkey"],
            serde_json::Value::Null
        );

        // A 0.2.0 file reads its Win+Space as the choice (`config_store::migrate` replaces it).
        let v020: PeekSettings = serde_json::from_value(serde_json::json!({
            "enabled": true, "dim": true, "hotkey": "winSpace"
        }))
        .unwrap();
        assert_eq!(v020.custom_hotkey, None);
        assert_eq!(v020.chosen_hotkey().to_string(), "Win+Space");
        // Text that is no usable shortcut leaves `hotkey` in charge.
        for text in ["Ctrl+Space", "Win+L", "Ctrl+Alt+", "nonsense", ""] {
            let peek = PeekSettings {
                custom_hotkey: Some(text.into()),
                hotkey: PeekHotkey::WinShiftSpace,
                ..PeekSettings::default()
            };
            assert_eq!(
                peek.chosen_hotkey().to_string(),
                "Win+Shift+Space",
                "{text}"
            );
        }
    }

    #[test]
    fn unused_settings_are_real_fields_and_may_be_missing() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        for path in UNUSED_SETTINGS {
            let (parent, key) = match path.rsplit_once('.') {
                Some((p, k)) => (format!("/{p}"), k),
                None => (String::new(), *path),
            };
            let obj = json
                .pointer_mut(&parent)
                .and_then(|v| v.as_object_mut())
                .unwrap();
            assert!(obj.remove(key).is_some(), "{path} is not a Settings field");
        }
        // A file written without them (settings get output, a future version) still loads.
        let back: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(
            back.quick_hide.enabled,
            Settings::default().quick_hide.enabled
        );
    }

    #[test]
    fn desktop_settings_missing_from_older_files_keep_the_old_behaviour() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        let obj = json.as_object_mut().unwrap();
        assert!(obj.remove("offerNewFence").is_some());
        assert!(obj.remove("hideInboxWhenIconsShown").is_some());
        assert!(obj.remove("keepIconsHidden").is_some());
        let legacy: Settings = serde_json::from_value(json).unwrap();
        assert!(legacy.offer_new_fence);
        assert!(!legacy.hide_inbox_when_icons_shown);
        assert!(!legacy.keep_icons_hidden);
    }

    #[test]
    fn theme_style_loads_legacy_settings_and_roundtrips_independently_of_mode() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json.as_object_mut().unwrap().remove("themeStyle");
        json.as_object_mut().unwrap().remove("titleAlign");
        json.as_object_mut().unwrap().remove("tabBar");
        json.as_object_mut().unwrap().remove("labelColor");
        json.as_object_mut().unwrap().remove("customTheme");
        let legacy: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(legacy.theme_style, ThemeStyle::Fluent);
        assert_eq!(legacy.custom_theme, None);
        assert_eq!(legacy.title_align, TitleAlign::Left);
        assert_eq!(legacy.tab_bar, TabBar::Top);
        assert_eq!(legacy.label_color, LabelColor::Auto);
        for mode in [
            ThemeSetting::Light,
            ThemeSetting::Dark,
            ThemeSetting::FollowWindowsMode,
            ThemeSetting::FollowAppMode,
        ] {
            let settings = Settings {
                theme: mode,
                theme_style: ThemeStyle::LiquidGlass,
                label_color: LabelColor::White,
                ..legacy.clone()
            };
            let json = serde_json::to_value(&settings).unwrap();
            assert_eq!(json["themeStyle"], "liquidGlass");
            assert_eq!(json["labelColor"], "white");
            assert_eq!(serde_json::from_value::<Settings>(json).unwrap(), settings);
        }
    }

    /// Settings from before `rollUp.hideCountAndChevron` keep the count and chevron; the switch
    /// saves and loads back.
    #[test]
    fn hide_count_and_chevron_defaults_off_and_roundtrips() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json["rollUp"]
            .as_object_mut()
            .unwrap()
            .remove("hideCountAndChevron");
        let legacy: Settings = serde_json::from_value(json).unwrap();
        assert!(!legacy.roll_up.hide_count_and_chevron);
        let mut settings = legacy;
        settings.roll_up.hide_count_and_chevron = true;
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["rollUp"]["hideCountAndChevron"], true);
        assert_eq!(serde_json::from_value::<Settings>(json).unwrap(), settings);
    }

    /// New settings switch tabs on hover; a saved `false` stays off.
    #[test]
    fn switch_tab_on_hover_defaults_on_and_keeps_a_saved_choice() {
        assert!(Settings::default().roll_up.switch_tab_on_hover);
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        let roll_up = json["rollUp"].as_object_mut().unwrap();
        roll_up.remove("switchTabOnHover");
        let missing: Settings = serde_json::from_value(json.clone()).unwrap();
        assert!(missing.roll_up.switch_tab_on_hover);
        json["rollUp"]["switchTabOnHover"] = false.into();
        let saved: Settings = serde_json::from_value(json).unwrap();
        assert!(!saved.roll_up.switch_tab_on_hover);
    }

    /// Fences saved before `openFoldersInside` keep opening folders in Explorer, whatever their
    /// `portalNavigate` (saved as true for every fence).
    #[test]
    fn open_folders_inside_defaults_off() {
        let mut json = serde_json::to_value(Fence::new("A", FenceKind::Virtual, geo())).unwrap();
        json.as_object_mut().unwrap().remove("openFoldersInside");
        let legacy: Fence = serde_json::from_value(json).unwrap();
        assert!(legacy.portal_navigate);
        assert!(!legacy.open_folders_inside);
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
            shown: 0,
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
            last_opened: None,
            path: Some("C:/Users/Me/Desktop/Report.docx".into()),
        };
        f.items.push(ItemRef {
            item_id: item.id,
            manual_index: Some(0),
            assigned_by: AssignedBy::User,
        });
        c.items.insert(item.id, item);
        c.layouts.push(Layout {
            shown: 0,
            fingerprint: vec![MonitorIdentity {
                gdi_name: String::new(),
                screen_px: [0, 0],
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
            shown: 0,
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
    fn duplicates_from_a_broken_file_keep_their_first_entry() {
        let mut c = Config::default();
        let a = Fence::new("A", FenceKind::Virtual, geo());
        let b = Fence::new("B", FenceKind::Inbox, geo());
        let mut dup = Fence::new("Dup", FenceKind::Virtual, geo());
        dup.id = a.id;
        let (aid, bid) = (a.id, b.id);
        c.layouts.push(Layout {
            shown: 0,
            fingerprint: vec![],
            fences: vec![a, b, dup],
        });
        let (x, y) = (Uuid::new_v4(), Uuid::new_v4());
        let member = |item_id| ItemRef {
            item_id,
            manual_index: None,
            assigned_by: AssignedBy::User,
        };
        c.layouts[0].fences[0].items = vec![member(x), member(x)];
        c.layouts[0].fences[1].items = vec![member(y), member(x)];
        c.layouts[0].fences[2].items = vec![member(y)];
        // Another layout lists the same fences and items: that is no duplicate.
        c.layouts.push(c.layouts[0].clone());
        c.layouts[1].fences.truncate(2);
        c.layouts[1].fences[0].items.truncate(1);
        c.layouts[1].fences[1].items.truncate(1);
        let clean = c.layouts[1].clone();

        assert!(c.drop_duplicates());
        let titles: Vec<&str> = c.layouts[0]
            .fences
            .iter()
            .map(|f| f.title.as_str())
            .collect();
        assert_eq!(titles, ["A", "B"]);
        assert_eq!(c.fence_mut(0, aid).unwrap().items, vec![member(x)]);
        assert_eq!(c.fence_mut(0, bid).unwrap().items, vec![member(y)]);
        assert_eq!(c.layouts[1].fences, clean.fences);
        assert!(!c.drop_duplicates());
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
                        // Which end of the rect the rolled title row sits at goes with it.
                        fences[0].expand_upward = rolled;
                        fences[0].bottom_anchored = rolled;
                        let before = fences.clone();
                        let mut layout = Layout {
                            shown: 0,
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
                        assert_eq!(group.expand_upward, before[0].expand_upward);
                        assert_eq!(group.bottom_anchored, before[0].bottom_anchored);
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
            shown: 0,
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
            shown: 0,
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
            shown: 0,
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
    fn tabs_normalize_ends_on_a_host_cycle() {
        let mut a = Fence::new("A", FenceKind::Virtual, geo());
        let mut b = Fence::new("B", FenceKind::Virtual, geo());
        let mut c = Fence::new("C", FenceKind::Virtual, geo());
        let (aid, bid, cid) = (a.id, b.id, c.id);
        // A points into the cycle B -> C -> B and comes first, as in a hand-edited file.
        a.tab_host = Some(bid);
        b.tab_host = Some(cid);
        c.tab_host = Some(bid);
        let mut l = Layout {
            shown: 0,
            fingerprint: vec![],
            fences: vec![a, b, c],
        };
        assert!(l.normalize_tabs());
        assert_eq!(l.host_of(aid), aid);
        assert_eq!(l.host_of(cid), bid);
        assert_eq!(l.tabs_of(bid), vec![bid, cid]);
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
            shown: 0,
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

    fn monitor(id: &str, gdi: &str) -> WorkArea {
        WorkArea {
            device_path: id.into(),
            gdi_name: gdi.into(),
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
            dpi: 96,
            mon_left: 0,
            mon_top: 0,
            mon_right: 1920,
            mon_bottom: 1080,
        }
    }

    fn saved(monitors: &[(&str, &str)]) -> Layout {
        Layout {
            fingerprint: monitors
                .iter()
                .map(|(id, gdi)| MonitorIdentity::of(&monitor(id, gdi)))
                .collect(),
            fences: vec![],
            shown: 0,
        }
    }

    fn found(layouts: &[Layout], now: &[(&str, &str)]) -> Option<LayoutMatch> {
        let now: Vec<WorkArea> = now.iter().map(|(id, gdi)| monitor(id, gdi)).collect();
        layout_for(layouts, &now)
    }

    const D1: &str = r"\\.\DISPLAY1";
    const D2: &str = r"\\.\DISPLAY2";

    #[test]
    fn monitors_are_found_by_id_then_model_then_gdi_name() {
        let both = [saved(&[("DEL1#B", D2), ("GSM7787#A", D1)])];
        let m = found(&both, &[("GSM7787#A", D1), ("DEL1#B", D2)]).unwrap();
        assert_eq!((m.index, m.renamed.len()), (0, 0));
        assert!(found(&both, &[("GSM7787#A", D1)]).is_none());
        // Another port or a dock: a new instance, the same model.
        let m = found(&both, &[("GSM7787#C", D3), ("DEL1#B", D2)]).unwrap();
        assert_eq!(m.renamed, vec![("GSM7787#A".into(), "GSM7787#C".into())]);
        // Two monitors of one model that both moved cannot be told apart.
        let twins = [saved(&[("GSM7787#A", D1), ("GSM7787#B", D2)])];
        assert!(found(&twins, &[("GSM7787#A", D1), ("GSM7787#C", D2)]).is_some());
        assert!(found(&twins, &[("GSM7787#C", D1), ("GSM7787#D", D2)]).is_none());
        // Files from 0.1.3 saved GDI names; a monitor without EDID has nothing else.
        let legacy = [saved(&[(D1, "")])];
        let m = found(&legacy, &[("GSM7787#5.0", D1)]).unwrap();
        assert_eq!(m.renamed, vec![(D1.into(), "GSM7787#5.0".into())]);
        assert!(found(&[saved(&[("GSM7787#A", D1)])], &[(D1, D1)]).is_some());
        // Another monitor on the same GDI source is another monitor.
        assert!(found(&[saved(&[("DEL1#X", D1)])], &[("GSM7787#A", D1)]).is_none());
        // The layout that knows the monitor by id wins over the legacy one.
        let m = found(
            &[saved(&[(D1, "")]), saved(&[("GSM7787#A", D1)])],
            &[("GSM7787#A", D1)],
        );
        assert_eq!(m.unwrap().index, 1);
    }

    const D3: &str = r"\\.\DISPLAY3";

    #[test]
    fn another_resolution_or_scale_is_another_layout() {
        let at = |w: i32, h: i32, dpi: u32| WorkArea {
            right: w,
            bottom: h - 48,
            mon_right: w,
            mon_bottom: h,
            dpi,
            ..monitor("GSM7787#A", D1)
        };
        let big = at(3840, 2160, 192);
        let known = saved(&[]);
        let known = Layout {
            fingerprint: vec![MonitorIdentity::of(&big)],
            ..known
        };
        let layouts = [known.clone()];
        assert!(layout_for(&layouts, std::slice::from_ref(&big)).is_some());
        // A taller taskbar is the same setup.
        let taskbar = WorkArea {
            bottom: 2160 - 96,
            ..big.clone()
        };
        assert!(layout_for(&layouts, &[taskbar]).is_some());
        assert!(layout_for(&layouts, &[at(1920, 1080, 96)]).is_none());
        assert!(layout_for(&layouts, &[at(3840, 2160, 144)]).is_none());
        // Saved before resolutions were: any fits, but the layout that knows them wins.
        let mut old = known.clone();
        old.fingerprint[0].screen_px = [0, 0];
        let both = [old, known];
        assert_eq!(
            layout_for(&both, std::slice::from_ref(&big)).unwrap().index,
            1
        );
        assert_eq!(layout_for(&both, &[at(1920, 1080, 96)]).unwrap().index, 0);
    }

    #[test]
    fn showing_another_set_carries_the_fences_and_keeps_its_spots() {
        let mut c = Config::default();
        let mut a = Fence::new("A", FenceKind::Virtual, geo());
        let b = Fence::new("B", FenceKind::Virtual, geo());
        let mut on_laptop = a.clone();
        on_laptop.geometry.x = 40.0;
        on_laptop.rolled_up = true;
        c.layouts.push(saved(&[("GSM7787#A", D1), ("BOE1#L", D2)]));
        c.layouts.push(saved(&[("BOE1#L", D1)]));
        c.layouts[1].fences = vec![on_laptop, b.clone()];
        // On the big set: A renamed, B deleted, C created.
        a.title = "A2".into();
        let new = Fence::new("C", FenceKind::Virtual, geo());
        c.layouts[0].fences = vec![a.clone(), new.clone()];
        assert_eq!(c.carry_fences(0, 1), vec![new.id]);
        let l = &c.layouts[1].fences;
        assert_eq!(
            l.iter().map(|f| f.id).collect::<Vec<_>>(),
            vec![a.id, new.id]
        );
        assert_eq!(
            (l[0].title.as_str(), l[0].geometry.x, l[0].rolled_up),
            ("A2", 40.0, true)
        );
        assert_eq!(l[1].geometry, new.geometry);
    }

    #[test]
    fn merging_old_per_set_fences_loses_none() {
        let mut c = Config::default();
        let item = |id: ItemId, by: AssignedBy| ItemRef {
            item_id: id,
            manual_index: None,
            assigned_by: by,
        };
        let (x, y, z) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let mut inbox = Fence::new("桌面", FenceKind::Inbox, geo());
        inbox.items = vec![item(y, AssignedBy::Migration)];
        let mut a = Fence::new("A", FenceKind::Virtual, geo());
        a.items = vec![item(x, AssignedBy::Rule(Uuid::new_v4()))];
        let mut other_inbox = Fence::new("桌面", FenceKind::Inbox, geo());
        other_inbox.items = vec![item(x, AssignedBy::User)];
        let mut made_there = Fence::new("N", FenceKind::Virtual, geo());
        made_there.items = vec![
            item(y, AssignedBy::User),
            item(z, AssignedBy::Rule(Uuid::new_v4())),
        ];
        c.layouts.push(saved(&[("GSM7787#A", D1)]));
        c.layouts[0].fences = vec![inbox.clone(), a.clone()];
        c.layouts.push(saved(&[("BOE1#L", D1)]));
        c.layouts[1].fences = vec![other_inbox, a.clone(), made_there.clone()];
        c.merge_layout_fences(0);
        let l = &c.layouts[0];
        let ids: Vec<FenceId> = l.fences.iter().map(|f| f.id).collect();
        assert_eq!(ids, vec![inbox.id, a.id, made_there.id]);
        // Placed by hand there, only routed here: it goes where the hand put it.
        assert_eq!(c.fence_of_item(0, y), Some(made_there.id));
        // A second inbox is not added, and what this set routed stays.
        assert_eq!(c.fence_of_item(0, x), Some(a.id));
        assert_eq!(c.fence_of_item(0, z), None);
    }

    #[test]
    fn the_layout_shown_last_is_remembered() {
        let mut c = Config::default();
        c.layouts.push(saved(&[("A#1", D1)]));
        c.layouts.push(saved(&[("B#1", D1)]));
        assert_eq!(c.last_shown(), None);
        assert!(c.mark_shown(1));
        assert!(!c.mark_shown(1));
        assert_eq!(c.last_shown(), Some(1));
        assert!(c.mark_shown(0));
        assert_eq!(c.last_shown(), Some(0));
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
