//! Draws the fence "chrome" (backdrop, title bar, divider, stroke) and the icon grid content.
//!
//! Liquid Glass shares its rounded geometry with the compositor clip and header controls.

use crate::backdrop::Image;
use crate::bitmaps::BitmapCache;
use crate::gpu_bindings as b;
use crate::panel::Clip;
use crate::theme::{Theme, font_small, font_text, lerp, with_alpha};
use std::cell::{Ref, RefCell};
use std::mem::ManuallyDrop;
use std::rc::Rc;
use windows_canvas::{
    Brush, ColorF, DrawingSession, FontWeight, GradientStop, Matrix3x2, Paint, ParagraphAlignment,
    Rect, RoundedRect, TextAlignment, TextFormat, TextLayout, Vector2, WordWrapping,
};
use windows_core::{Interface, Result};

/// Icon-label font: the system icon-title font (`SPI_GETICONTITLELOGFONT` scaled by the
/// Settings › Accessibility › Text size factor, see `sysparams::icon_title_font`) with its
/// measured line height, so the cell grid grows with the text like the desktop's does.
struct LabelFont {
    format: TextFormat,
    family: String,
    size: f32,
    weight: i32,
    /// Height of one text line in DIPs (16 for the default 12 px).
    line_h: f32,
}

/// Line height of the built-in 12 px label font, and the fallback when measuring fails.
pub const DEFAULT_LABEL_LINE_H: f32 = 16.0;
/// Horizontal breathing room inside each icon cell. A wider gutter keeps neighbouring
/// multi-line CJK labels from visually running together in narrow panes.
pub const ICON_LABEL_SIDE_INSET: f32 = 8.0;

/// Titles and tab captions at the four title sizes (12 / 14 / 16 / 18 px), rebuilt when a
/// theme names its own title font.
struct TitleFonts {
    titles: [TextFormat; 4],
    /// Same face, size and weight as standalone titles, centred inside their controls.
    tabs: [TextFormat; 4],
    /// The left tab column's resting captions: regular weight, leading (the shown tab uses
    /// the title format).
    tab_lists: [TextFormat; 4],
    face: Option<String>,
    weight: i32,
}

pub struct FenceChrome {
    title_fonts: RefCell<TitleFonts>,
    count_format: TextFormat,
    label_font: RefCell<LabelFont>,
    glyph_format: TextFormat,
    /// Single-line, vertically centred Caption text for list rows and column headers.
    row_format: TextFormat,
    /// Same, right-aligned (size column).
    row_format_right: TextFormat,
    /// Header sort indicator (8 px chevron).
    sort_glyph_format: TextFormat,
    /// Empty-state text: Body 14, top-aligned (Explorer "This folder is empty.").
    empty_format: TextFormat,
    /// 14 px Fluent glyph beside the title (portal folder icon).
    title_glyph_format: TextFormat,
    /// "按时间分组" section captions: Caption 12 / 600, leading, vertically centred.
    group_format: TextFormat,
    /// Tab match badge number: 11 / 600, centred.
    badge_format: TextFormat,
}

/// A "按时间分组" section header band in surface DIPs (already scrolled; for rows already
/// offset below the fixed column header, like the rows themselves).
pub struct GroupHeaderDraw<'a> {
    pub y: f32,
    pub h: f32,
    pub text: &'a str,
}

/// A wallpaper crop for one fence. The bitmap is uploaded once per `key` (a per-window cache
/// key the owner invalidates whenever the crop changes) and stretched over `height` DIPs: the
/// surface height at rest, the taller end of a height animation while one runs — the moving
/// bottom edge then reveals or conceals finished pixels instead of re-cropping per frame.
#[derive(Clone, Copy)]
pub struct BackdropCrop<'a> {
    pub image: &'a Image,
    pub key: &'a str,
    /// DIP size the crop covers, from the window's top left (wider or taller than the surface
    /// while the window shrinks or grows).
    pub width: f32,
    pub height: f32,
}

/// What to paint behind the fence content.
pub enum Backdrop<'a> {
    /// Solid fallback colour from the theme.
    Solid,
    /// A pre-blurred, pre-tinted crop of the wallpaper (any size; stretched to the fence).
    MicaLike(BackdropCrop<'a>),
    /// A pre-blurred acrylic crop of the wallpaper plus the DPI scale (for 1:1 grain tiling).
    Glass(BackdropCrop<'a>, f32),
    /// GPU-resident wallpaper and geometry-only displacement map.
    GpuGlass {
        material: &'a RefCell<crate::gpu_glass::GpuGlass>,
        wallpaper: &'a Rc<Vec<crate::MonitorBackdrop>>,
        rect: [i32; 4],
        scale: f32,
    },
    /// DWM paints the material (system Acrylic); only a light tint layer is drawn here.
    SystemMaterial,
}

/// One icon cell to draw, in content-local DIPs (already scrolled).
pub struct ItemCell<'a> {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub icon_key: &'a str,
    pub icon: Option<&'a Image>,
    pub label: &'a str,
    /// PointerOver fill alpha 0..=1 (SubtleFillColorSecondary fades over 83 ms).
    pub hover: f32,
    /// Selection fill / stroke alpha 0..=1 (fades over 83 ms on select / deselect).
    pub selection: f32,
    /// Button held on this item: SubtleFillColorTertiary on top (WinUI ListViewItem pressed).
    pub pressed: bool,
    /// Keyboard cursor item after keyboard navigation: WinUI focus visual (2 px + 1 px ring).
    pub focused: bool,
    pub dimmed: bool,
    /// Icon extraction failed for good: draw a generic document glyph, not a loading tile.
    pub failed: bool,
    /// A folder being hovered by a drag: Explorer's drop highlight (selection look), alpha
    /// 0..=1 (fades over 83 ms).
    pub drop_target: f32,
    /// Whole-cell opacity 0..=1: an item fading in after it appeared or fading out after it
    /// was removed (layout motion). State layers are skipped below 1.
    pub alpha: f32,
    /// Icon bitmap opacity 0..=1 while it cross-fades in over the loading tile (167 ms).
    pub icon_alpha: f32,
    /// The selected / focused item's full name (Windows desktop: the focused icon's label
    /// unfolds past the two-line limit, overlapping the row below). Painted last.
    pub full_label: Option<(&'a str, u32)>,
}

/// Which details column a header caption / sort indicator refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeaderColumn {
    Name,
    Date,
    Type,
    Size,
}

/// Column x/width split used by the details view (mirrors the app's `DetailColumns`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowColumns {
    pub name_x: f32,
    pub name_w: f32,
    pub date: Option<(f32, f32)>,
    pub type_: Option<(f32, f32)>,
    pub size: Option<(f32, f32)>,
}

/// One row of the List / Details view, in content DIPs (already scrolled, below the header).
pub struct RowCell<'a> {
    pub y: f32,
    pub h: f32,
    pub icon_key: &'a str,
    pub icon: Option<&'a Image>,
    pub name: &'a str,
    pub date: &'a str,
    pub type_name: &'a str,
    pub size: &'a str,
    pub hover: f32,
    pub selection: f32,
    pub pressed: bool,
    pub focused: bool,
    pub dimmed: bool,
    pub failed: bool,
    /// Drop-target highlight alpha 0..=1 (see [`ItemCell::drop_target`]).
    pub drop_target: f32,
    /// Whole-row opacity (see [`ItemCell::alpha`]).
    pub alpha: f32,
    /// Icon cross-fade (see [`ItemCell::icon_alpha`]).
    pub icon_alpha: f32,
}

/// Overlay scrollbar in content DIPs (WinUI ScrollBar). `width` runs 2 (resting indicator)
/// to 6 DIP (expanded, pointer near the bar or dragging it) and `alpha` 0..=1 fades the whole
/// bar (inactive-scrollbar rule); `parts` 0..=1 reveals the expanded state's 12 DIP track fill
/// and the two arrow buttons at its ends. All are sampled from client tweens by the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarDraw {
    pub thumb_y: f32,
    pub thumb_h: f32,
    pub width: f32,
    pub alpha: f32,
    /// Track fill + arrow buttons opacity (83 ms with the expand / contract).
    pub parts: f32,
    /// Vertical extent of the bar (top of the scrollable band, its height): the track fill
    /// spans it, the buttons sit at its ends, the thumb travels between them.
    pub band_top: f32,
    pub band_h: f32,
    /// PointerOver fill alpha of the up / down button.
    pub arrow_hover: [f32; 2],
    /// The up / down button is held (RepeatButton pressed).
    pub arrow_pressed: [bool; 2],
}

impl ScrollbarDraw {
    pub const REST_WIDTH: f32 = 2.0;
    pub const EXPANDED_WIDTH: f32 = 6.0;
    /// WinUI ScrollBarSize: the expanded track's width.
    pub const BAR_W: f32 = 12.0;
    /// Height of each arrow RepeatButton at the ends of the track.
    pub const BUTTON_H: f32 = 12.0;
    /// Segoe Fluent Icons CaretUpSolid8 / CaretDownSolid8 (ScrollBar arrow glyphs).
    const ARROW_UP: &'static str = "\u{EDDB}";
    const ARROW_DOWN: &'static str = "\u{EDDC}";
    /// Arrow glyph scale while its button is pressed (ScrollBarButtonArrowScalePressed).
    const ARROW_SCALE_PRESSED: f32 = 0.875;

    /// Left edge of a thumb `width` DIPs wide inside a surface `total_w` wide: the thumb keeps
    /// its centre on the 12 DIP track's centre line, 6 DIP from the right edge (the track and
    /// the arrow buttons span `total_w - 12 .. total_w`), so 2 DIP sits at `total_w - 7` and
    /// 6 DIP at `total_w - 9` — WinUI scales the thumb about its centre inside the track.
    pub fn thumb_x(total_w: f32, width: f32) -> f32 {
        total_w - ScrollbarDraw::BAR_W / 2.0 - width / 2.0
    }
}

pub struct RowsDraw<'a> {
    pub rows: &'a [RowCell<'a>],
    /// Section headers interleaved with the rows (drawn in the scrolling band).
    pub group_headers: &'a [GroupHeaderDraw<'a>],
    pub columns: RowColumns,
    pub icon_size: f32,
    /// 0 for the List layout (no header).
    pub header_h: f32,
    pub sort_column: Option<HeaderColumn>,
    pub sort_descending: bool,
    /// Per-column hover fill alpha (0..=1); absent = 0.
    pub header_hover: &'a [(HeaderColumn, f32)],
    /// Header cell with the button held (fires on release, like a Win32 header control).
    pub header_pressed: Option<HeaderColumn>,
    pub scrollbar: Option<ScrollbarDraw>,
    pub empty_text: Option<&'a str>,
    /// Fence-wide drop wash + accent ring alpha 0..=1 (a drag hovers the fence itself).
    pub drop_highlight: f32,
    pub marquee: Option<Rect>,
    /// Insertion caret (accent bar) while an internal drag reorders the rows, and its alpha.
    pub insert_caret: Option<Rect>,
    pub insert_caret_alpha: f32,
    /// 0 = accent selection (active window), 1 = neutral grey (another window is active).
    pub selection_inactive: f32,
    /// Window position (DIPs) of the surface's top-left corner: the left tab column's width
    /// and the title row's height (0 beside a left tab column). The glass behind the text is
    /// sampled in window coordinates.
    pub origin_x: f32,
    pub origin_y: f32,
    /// The rows end here (surface DIPs): 「调整大小时按整行整列对齐」 leaves the height under
    /// the whole rows empty. `f32::INFINITY` = the surface's bottom.
    pub rows_bottom: f32,
}

/// Pixel snapping for hairlines. The panel applies a uniform DIP→px scale, so a 1 DIP stroke
/// at a .5 DIP offset smears over two device pixels at 125 / 150 %; Explorer's hairlines are
/// exactly one device pixel, layout-rounded to the grid at every DPI.
#[derive(Clone, Copy, Debug)]
struct Px {
    scale: f32,
}

impl Px {
    fn new(scale: f32) -> Self {
        Self {
            scale: if scale.is_finite() && scale > 0.0 {
                scale
            } else {
                1.0
            },
        }
    }

    /// One hairline in DIPs: 1 device px up to 175 %, 2 px from 200 % (a 1 epx XAML border).
    fn hair(self) -> f32 {
        self.scale.floor().max(1.0) / self.scale
    }

    /// Snaps a DIP coordinate to the device-pixel grid.
    fn snap(self, v: f32) -> f32 {
        (v * self.scale).round() / self.scale
    }

    /// Snaps every edge of `r` to the grid, then insets by half a hairline so a `hair()`-wide
    /// stroke covers whole pixels.
    fn stroke_rect(self, r: Rect) -> Rect {
        let h = self.hair() / 2.0;
        let (l, t, rr, b) = (
            self.snap(r.left) + h,
            self.snap(r.top) + h,
            self.snap(r.right) - h,
            self.snap(r.bottom) - h,
        );
        Rect {
            left: l,
            top: t,
            right: rr.max(l),
            bottom: b.max(t),
        }
    }

    /// A rounded hairline ring inside `r` with the given outer corner radius.
    fn ring(self, r: Rect, radius: f32) -> RoundedRect {
        RoundedRect::uniform(self.stroke_rect(r), (radius - self.hair() / 2.0).max(0.0))
    }
}

/// Per-fence look: colour wash, title colour and size (Fences per-fence appearance).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FenceStyle {
    pub tint: Option<ColorF>,
    /// The tint washes only the title row.
    pub tint_title_only: bool,
    pub title_color: Option<ColorF>,
    /// 0 = small (12), 1 = normal (14), 2 = large (16), 3 = extra large (18).
    pub title_size: u8,
    /// Title position in the row (the global 标题对齐 setting).
    pub title_align: pecofence_core::TitleAlign,
    /// Width in DIPs of the left tab column (标签栏位置 → 左侧, two or more tabs). The column
    /// is the fence's title bar, so there is no title row; 0 = the tabs, if any, run along the
    /// title row.
    pub tab_rail: f32,
    /// No item count on a rolled title and no roll chevron (the global
    /// 「隐藏标题栏右侧的项目数和折叠箭头」 setting).
    pub hide_count_and_chevron: bool,
}

/// Left tab column metrics (DIPs): the free strip above the first tab (a handle to drag the
/// fence by), each tab row's height, its inset from the window's left edge and from the
/// divider, and the room under the last tab that holds the collapse chevron. Collapsed to the
/// column the fence has no chevron (a tab click expands it): the last tab keeps the top strip's
/// room under it.
pub const RAIL_TOP: f32 = 8.0;
pub const RAIL_ROW_H: f32 = 32.0;
pub const RAIL_ROW_INSET: f32 = 4.0;
pub const RAIL_ROW_RIGHT: f32 = 6.0;
pub const RAIL_BOTTOM: f32 = 44.0;
pub const RAIL_COLLAPSED_BOTTOM: f32 = RAIL_TOP;

/// Title-row decorations of a folder portal: a folder glyph before the title and, once the
/// portal has navigated into a subfolder, an "up" button: the folder glyph itself, or one at the
/// far left while the glyph is hidden.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TitleDeco {
    pub folder_icon: bool,
    pub up_button: bool,
}

/// Interaction state of the title row, sampled per draw: fade alphas (0..=1) from the app's
/// client tweens plus the instant pressed flags (WinUI swaps pressed brushes at KeyTime 0).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TitleState {
    /// Title-row reveal ("title on hover"): 0 leaves only glass and rim.
    pub title: f32,
    /// The title row is hovered: full-row pill (single-title fences) and the chevron reveal.
    pub hover: f32,
    /// Up button (portal) hover fill / pressed.
    pub up_hover: f32,
    pub up_pressed: bool,
    /// Roll-up chevron hover fill / pressed.
    pub chevron_hover: f32,
    pub chevron_pressed: bool,
    /// Rolled item-count caption.
    pub count: f32,
    /// Another fence is being dragged over this title (drop here to merge it as a tab): the
    /// accent wash + ring alpha, 83 ms in / 167 ms out.
    pub merge_hint: f32,
    /// Title-on-hover fold: the plate (glass, tint, rim) starts this many DIPs below the
    /// window top, so a resting fence begins at its content; 0 once the title row is shown.
    pub plate_top: f32,
    /// How far a fully transparent fence (外观 → 全透明) has cleared its plate: 0 = drawn,
    /// 1 = no glass, tint or rim at all (the resting look away from the pointer).
    pub plate_clear: f32,
    /// A tab strip too wide for its room scrolls: where its pills are cut. None = it fits.
    pub strip_clip: Option<StripClip>,
}

/// The visible span of a scrolled tab strip (DIPs). An end with more pills behind it fades out
/// over `STRIP_FADE_W`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StripClip {
    pub left: f32,
    /// The right end beside the roll-up chevron.
    pub right: f32,
    /// The right end while the chevron is hidden (an expanded fence, pointer away from the
    /// title): the strip runs on under the chevron's place.
    pub rest_right: f32,
    /// Where the last pill ends: the right end fades while pills lie past it.
    pub content_right: f32,
    pub fade_left: bool,
}

/// Width (DIPs) of the fade at an end of a scrolled tab strip with more pills behind it.
pub const STRIP_FADE_W: f32 = 12.0;

/// Starts drawing a scrolled tab strip into a layer cut to `clip` (`title_h` tall) whose
/// opacity falls to 0 over `STRIP_FADE_W` at the ends with pills behind them. The right end
/// moves from `rest_right` to `right` as the chevron fades in (`chevron` 0..=1). The returned
/// brush must live until `pop_strip_fade` ends the layer.
fn push_strip_fade(
    session: &DrawingSession<'_>,
    clip: StripClip,
    title_h: f32,
    chevron: f32,
) -> Result<b::ID2D1Brush> {
    let right = clip.rest_right + (clip.right - clip.rest_right) * chevron.clamp(0.0, 1.0);
    let fade_right = clip.content_right > right + 0.5;
    let fade = (STRIP_FADE_W / (right - clip.left).max(1.0)).min(0.5);
    let opaque = ColorF::new(0.0, 0.0, 0.0, 1.0);
    let end = |hidden: bool| if hidden { ColorF::TRANSPARENT } else { opaque };
    let gradient = session.create_linear_gradient(
        Vector2::new(clip.left, 0.0),
        Vector2::new(right, 0.0),
        &[
            GradientStop::new(0.0, end(clip.fade_left)),
            GradientStop::new(fade, opaque),
            GradientStop::new(1.0 - fade, opaque),
            GradientStop::new(1.0, end(fade_right)),
        ],
    )?;
    let brush: b::ID2D1Brush = gradient.as_raw_brush().cast()?;
    let context: b::ID2D1DeviceContext = session.raw().cast()?;
    let params = b::D2D1_LAYER_PARAMETERS1 {
        contentBounds: b::D2D_RECT_F {
            left: clip.left,
            top: 0.0,
            right,
            bottom: title_h,
        },
        maskTransform: windows_numerics::Matrix3x2::identity(),
        opacity: 1.0,
        opacityBrush: ManuallyDrop::new(Some(brush.clone())),
        ..Default::default()
    };
    unsafe { context.PushLayer(&params, None::<&b::ID2D1Layer>) };
    drop(ManuallyDrop::into_inner(params.opacityBrush));
    Ok(brush)
}

/// Ends the layer `push_strip_fade` started.
fn pop_strip_fade(session: &DrawingSession<'_>, _brush: b::ID2D1Brush) {
    if let Ok(context) = session.raw().cast::<b::ID2D1DeviceContext>() {
        unsafe { context.PopLayer() };
    }
}

impl TitleDeco {
    /// Width of the up button zone (DIPs) — the app hit-tests the same rectangle.
    pub const UP_X: f32 = 6.0;
    pub const UP_W: f32 = 28.0;
    pub const ICON_W: f32 = 24.0;
    /// Roll-up chevron button: a 32 × 32 box at the right end of the title row.
    pub const CHEVRON_W: f32 = 32.0;
    pub const CHEVRON_RIGHT_MARGIN: f32 = 4.0;

    /// The up button box (DIPs) for a title row `title_h` tall.
    pub fn up_box(title_h: f32) -> Rect {
        Rect::from_xywh(Self::UP_X, (title_h - 28.0) / 2.0, Self::UP_W, 28.0)
    }

    /// The chevron box (DIPs) for a fence `width` wide with a title row `title_h` tall; the
    /// app hit-tests the same rectangle.
    pub fn chevron_box(width: f32, title_h: f32) -> Rect {
        Rect::from_xywh(
            width - Self::CHEVRON_W - Self::CHEVRON_RIGHT_MARGIN,
            (title_h - Self::CHEVRON_W) / 2.0,
            Self::CHEVRON_W,
            Self::CHEVRON_W,
        )
    }

    /// The collapse chevron of a left tab column `rail_w` wide in a window `height` tall,
    /// centred in the room under the tabs (`RAIL_BOTTOM`), never above the last tab's end
    /// `rows_end`: while the fence collapses or expands the moving bottom edge hides it instead
    /// of it sliding over the tab. The app hit-tests the same box.
    pub fn rail_chevron_box(rail_w: f32, height: f32, rows_end: f32) -> Rect {
        let gap = (RAIL_BOTTOM - Self::CHEVRON_W) * 0.5;
        Rect::from_xywh(
            ((rail_w - Self::CHEVRON_W) * 0.5).max(0.0),
            (height - Self::CHEVRON_W - gap).max(rows_end + gap),
            Self::CHEVRON_W,
            Self::CHEVRON_W,
        )
    }

    /// The portal up button at the end of the shown tab's `row` in a left tab column; the app
    /// hit-tests the same box.
    pub fn rail_up_box(row: Rect) -> Rect {
        Rect::from_xywh(
            row.right - 26.0,
            row.top + (row.height() - 24.0) * 0.5,
            24.0,
            24.0,
        )
    }

    /// The up button sits at the far left (the folder glyph is hidden, so it cannot be it).
    pub fn up_at_left(&self) -> bool {
        self.up_button && !self.folder_icon
    }

    /// Where the title text starts (left-aligned).
    pub fn title_x(&self) -> f32 {
        let mut x = 16.0;
        if self.up_at_left() {
            x += Self::UP_W;
        }
        if self.folder_icon {
            x += Self::ICON_W;
        }
        x
    }
}

/// Least room kept free at the right end of the title row: the roll-up chevron.
pub const TITLE_RIGHT_RESERVE: f32 = 40.0;

/// Left edge of a title-row unit `unit_w` DIPs wide (the title with its folder glyph, or the
/// whole tab strip) that may occupy `start..end` of a row `width` DIPs wide. Centred means
/// centred on the whole fence, pushed aside only when it would reach into `start` / `end`.
pub fn aligned_x(
    align: pecofence_core::TitleAlign,
    width: f32,
    start: f32,
    end: f32,
    unit_w: f32,
) -> f32 {
    let last = (end - unit_w.max(0.0)).max(start);
    match align {
        pecofence_core::TitleAlign::Left => start,
        pecofence_core::TitleAlign::Center => ((width - unit_w) * 0.5).clamp(start, last),
        pecofence_core::TitleAlign::Right => last,
    }
}

/// One tab header in the title row (DIPs, x/w along the title).
pub struct TabDraw<'a> {
    pub key: u128,
    pub x: f32,
    pub w: f32,
    pub title: &'a str,
    pub title_size: u8,
    pub title_color: Option<ColorF>,
    /// A folder portal tab: its folder glyph is drawn before the caption (`TAB_ICON_W` wider).
    pub folder_icon: bool,
    /// The shown tab of a folder portal inside a subfolder: its folder glyph is drawn as the up
    /// button (`FenceChrome::tab_up_box`), with the up button's hover and pressed looks.
    pub up_button: bool,
    pub active: bool,
    /// PointerOver pill alpha 0..=1.
    pub hover: f32,
    /// Button held on this tab (TabViewItem pressed: layer fill, tertiary caption).
    pub pressed: bool,
    /// The tab's colour (its own, else its fence tint), drawn as a thin bar under the caption.
    pub color: Option<ColorF>,
    /// Files are being dragged over this tab: drawn like a folder drop target (selection
    /// fill + accent ring), alpha 0..=1.
    pub drop_target: f32,
    /// This tab is being dragged along the strip: it rides under the pointer (`x` is the
    /// pointer-driven position, not its slot) and paints above its neighbours.
    pub dragging: bool,
    /// Whole-pill opacity 0..=1 (a tab that just joined the strip fades in).
    pub alpha: f32,
    /// Peek search: how many matches this tab holds, drawn as an accent badge after the
    /// caption (see [`FenceChrome::tab_badge_width`]).
    pub badge: Option<&'a str>,
}

/// Width of the "N 项" caption a rolled fence shows before its chevron (DIPs).
pub const ROLLED_COUNT_W: f32 = 64.0;

/// Tab match badge (WinUI InfoBadge): 16 DIPs high, 5 DIPs of padding beside the number, 6 DIPs
/// after the caption.
const TAB_BADGE_H: f32 = 16.0;
const TAB_BADGE_PAD: f32 = 5.0;
const TAB_BADGE_GAP: f32 = 6.0;

/// Nested controls share the fence's radius, inset from its outer edge and limited by
/// their own height. This also applies to hover, pressed, drop and merge feedback.
pub fn header_control_radius(theme: &Theme, rect: Rect) -> f32 {
    if theme.liquid_glass {
        (theme.corner_radius - 4.0)
            .max(0.0)
            .min(rect.width().min(rect.height()) * 0.5)
    } else {
        4.0
    }
}

/// Room a folder glyph takes before a tab caption (DIPs).
const TAB_ICON_W: f32 = 20.0;

/// Width the folder glyph adds before a tab caption: `TAB_ICON_W` for a folder tab, else 0.
pub fn tab_icon_w(folder_icon: bool) -> f32 {
    if folder_icon { TAB_ICON_W } else { 0.0 }
}

/// A tab pill's caption as `FenceChrome::draw` lays it out: where it starts, how wide it is
/// drawn, its fitted text and the badge's width after it.
struct TabCaption {
    x: f32,
    w: f32,
    fitted: String,
    badge_w: f32,
}

/// The up button that takes a folder glyph's place, the glyph centred at `centre`: a 20 DIP
/// square around it, filled on hover and press.
fn glyph_up_square(centre: f32, title_h: f32) -> Rect {
    Rect::from_xywh(centre - 10.0, (title_h - 20.0) * 0.5, 20.0, 20.0)
}

/// Where a tab pill's folder glyph is centred when its caption starts at `caption_x`.
fn tab_glyph_centre(caption_x: f32) -> f32 {
    caption_x - 2.0 - TAB_ICON_W * 0.5
}

/// Where captions start in a left tab column row: after the colour dot when any tab has a
/// colour, and after the folder glyph when any tab has one, so the names stay aligned.
fn rail_text_left(colored: bool, folders: bool) -> f32 {
    (if colored { 24.0 } else { 10.0 }) + tab_icon_w(folders)
}

pub fn tab_text_padding(theme: &Theme) -> f32 {
    if theme.liquid_glass { 24.0 } else { 16.0 }
}

/// Least caption inset once a crowded strip squeezes the pills.
const TAB_TEXT_MIN_PADDING: f32 = 8.0;

/// Caption inset inside a pill `w` DIPs wide for a caption `text_w` wide: the theme's
/// padding while it fits, then shrinking to `TAB_TEXT_MIN_PADDING` before the caption is
/// elided, so a narrow fence keeps short tab names readable.
pub fn tab_text_inset(theme: &Theme, w: f32, text_w: f32) -> f32 {
    (w - text_w).clamp(TAB_TEXT_MIN_PADDING, tab_text_padding(theme))
}

/// Counts are secondary: a short collapsed fence must retain space for its title/tabs. A tab
/// strip that does not fit scrolls, so room for two of its narrowest tabs is enough.
pub fn header_count_width(width: f32, tabs: usize, title_x: f32) -> f32 {
    let (left, captions) = if tabs > 1 {
        let shown = tabs.min(2) as f32;
        (8.0, shown * 48.0 + (shown - 1.0) * 4.0)
    } else {
        (title_x, 64.0)
    };
    if width - left - 44.0 - captions >= ROLLED_COUNT_W {
        ROLLED_COUNT_W
    } else {
        0.0
    }
}

/// The portal up button's hover / pressed fill behind its glyph: transparent at rest,
/// SubtleFill fading in on hover, SubtleFillColorTertiary while pressed (a glass lens on liquid
/// glass).
fn draw_up_box_fill(
    session: &DrawingSession<'_>,
    backdrop: &Backdrop<'_>,
    theme: &Theme,
    scale: f32,
    box_r: Rect,
    state: TitleState,
    alpha: f32,
) -> Result<()> {
    if theme.liquid_glass && (state.up_pressed || state.up_hover > 0.0) {
        draw_header_glass(
            session,
            backdrop,
            theme,
            scale,
            3,
            box_r,
            if state.up_pressed { 0.85 } else { 0.45 },
            alpha
                * if state.up_pressed {
                    1.0
                } else {
                    state.up_hover
                },
        )?;
    } else if state.up_pressed {
        let pill = session.create_solid_brush(with_alpha(theme.subtle_pressed, alpha))?;
        session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &pill);
    } else if state.up_hover > 0.0 {
        let pill =
            session.create_solid_brush(with_alpha(theme.subtle_hover, state.up_hover * alpha))?;
        session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &pill);
    }
    Ok(())
}

fn draw_header_glass(
    session: &DrawingSession<'_>,
    backdrop: &Backdrop<'_>,
    theme: &Theme,
    scale: f32,
    key: u128,
    rect: Rect,
    amount: f32,
    alpha: f32,
) -> Result<()> {
    if alpha <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return Ok(());
    }
    let radius = header_control_radius(theme, rect);
    if let Backdrop::GpuGlass { material, .. } = backdrop {
        let mut material = material.borrow_mut();
        material.set_lens(theme.lens);
        material.draw_control(session, key, rect, scale, radius, amount * alpha)?;
    }
    let white = ColorF::new(1.0, 1.0, 1.0, 1.0);
    // A thin optical wash, not the opaque LayerOnAcrylic fill used by Fluent tabs.
    let wash = session.create_solid_brush(with_alpha(white, (0.018 + 0.075 * amount) * alpha))?;
    session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &wash);
    let rim = session.create_linear_gradient(
        Vector2::new(rect.left, rect.top),
        Vector2::new(rect.right, rect.bottom),
        &[
            GradientStop::new(0.0, with_alpha(white, (0.18 + 0.43 * amount) * alpha)),
            GradientStop::new(0.4, with_alpha(white, (0.04 + 0.08 * amount) * alpha)),
            GradientStop::new(
                0.65,
                ColorF::new(0.06, 0.12, 0.16, (0.02 + 0.06 * amount) * alpha),
            ),
            GradientStop::new(1.0, with_alpha(white, (0.08 + 0.21 * amount) * alpha)),
        ],
    )?;
    let px = Px::new(scale);
    session.draw_rounded_rect(&px.ring(rect, radius), &rim, px.hair());
    Ok(())
}

fn header_foreground(
    backdrop: &Backdrop<'_>,
    theme: &Theme,
    key: u128,
    area: Rect,
    scale: f32,
    opacity: f32,
    tint: Option<ColorF>,
) -> Theme {
    if let Backdrop::GpuGlass {
        material,
        wallpaper,
        rect,
        ..
    } = backdrop
    {
        return material.borrow_mut().control_foreground(
            key,
            *theme,
            wallpaper,
            [
                rect[0] + (area.left * scale).round() as i32,
                rect[1] + (area.top * scale).round() as i32,
                (area.width() * scale).round().max(1.0) as i32,
                (area.height() * scale).round().max(1.0) as i32,
            ],
            opacity,
            tint,
        );
    }
    *theme
}

/// The fence plate inside `outer` (the whole surface, or the part below a folded title row):
/// backdrop material, opacity veil, tint wash, sheen and the 1 px rim, all at `plate_a`
/// (a fully transparent fence fades them in only while the pointer is over it). `opacity` is
/// the per-fence multiplier, already clamped.
#[allow(clippy::too_many_arguments)]
fn draw_plate(
    session: &DrawingSession<'_>,
    bitmaps: &mut BitmapCache,
    theme: &Theme,
    scale: f32,
    width: f32,
    height: f32,
    outer: Rect,
    radius: f32,
    backdrop: &Backdrop<'_>,
    opacity: f32,
    plate_a: f32,
    pill_a: f32,
    hover: f32,
    style: FenceStyle,
) -> Result<()> {
    let px = Px::new(scale);
    let plate_top = outer.top;
    // The crop is anchored at the window's top left and may cover more than the surface (size
    // animation): the surface clips it, and the compositor clip cuts it at a folded top.
    let crop_rect = |crop: &BackdropCrop<'_>| {
        Rect::from_xywh(0.0, 0.0, crop.width.max(width), crop.height.max(height))
    };
    match *backdrop {
        Backdrop::GpuGlass {
            material,
            wallpaper,
            rect,
            scale,
        } => {
            // A folded title row moves the lens (and its bezel) down to the plate top.
            let inset = (plate_top * scale).round() as i32;
            let lens = [rect[0], rect[1] + inset, rect[2], (rect[3] - inset).max(1)];
            material.borrow_mut().set_lens(theme.lens);
            let result = material.borrow_mut().draw(
                session,
                &mut bitmaps.glass_wallpaper,
                wallpaper,
                lens,
                inset as f32,
                scale,
                radius,
                opacity.min(1.0) * plate_a,
                hover,
            );
            if let Err(error) = result {
                if windows_canvas::is_device_lost(error.code()) {
                    return Err(error);
                }
                let fallback = if theme.text_primary.r > 0.5 {
                    Theme::dark().solid_fill
                } else {
                    Theme::light().solid_fill
                };
                let fill = session.create_solid_brush(with_alpha(fallback, plate_a))?;
                session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &fill);
            } else {
                let mut c = theme.glass_layer_fill;
                c.a *= opacity.min(1.0) * plate_a;
                let layer = session.create_solid_brush(c)?;
                session.fill_rect(&outer, &layer);
            }
        }
        Backdrop::MicaLike(crop) if crop.image.width > 0 && crop.image.height > 0 => {
            let bitmap = bitmaps.get_or_upload(session, crop.key, crop.image)?;
            session.draw_bitmap(&bitmap, &crop_rect(&crop), glass_alpha(opacity) * plate_a);
            let mut c = theme.layer_fill;
            c.a *= opacity.min(1.0) * plate_a;
            let layer = session.create_solid_brush(c)?;
            session.fill_rect(&outer, &layer);
        }
        Backdrop::Glass(crop, scale) if crop.image.width > 0 && crop.image.height > 0 => {
            let bitmap = if theme.liquid_glass {
                match bitmaps.get(crop.key) {
                    Some(bitmap) => bitmap,
                    None => {
                        let overlay = crate::liquid_glass::refracted_overlay(
                            crop.image,
                            width,
                            crop.height.max(height),
                            radius,
                        );
                        bitmaps.get_or_upload(session, crop.key, &overlay)?
                    }
                }
            } else {
                bitmaps.get_or_upload(session, crop.key, crop.image)?
            };
            let alpha = if theme.liquid_glass {
                opacity.min(1.0)
            } else {
                glass_alpha(opacity)
            };
            session.draw_bitmap(&bitmap, &crop_rect(&crop), alpha * plate_a);
            let mut c = theme.glass_layer_fill;
            c.a *= opacity.min(1.0) * plate_a;
            let layer = session.create_solid_brush(c)?;
            session.fill_rect(&outer, &layer);
            // Acrylic grain: tile the noise texture at exactly one texel per device pixel.
            if !theme.liquid_glass {
                let tile = crate::backdrop::noise_tile();
                let noise = bitmaps.get_or_upload(session, NOISE_BITMAP_KEY, tile)?;
                let step = crate::backdrop::NOISE_TILE_PX as f32 / scale.max(0.5);
                let mut y = 0.0;
                while y < height {
                    let mut x = 0.0;
                    while x < width {
                        session.draw_bitmap(&noise, &Rect::from_xywh(x, y, step, step), plate_a);
                        x += step;
                    }
                    y += step;
                }
            }
        }
        Backdrop::SystemMaterial => {
            let layer = session.create_solid_brush(with_alpha(theme.layer_fill, plate_a))?;
            session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &layer);
        }
        _ => {
            let fill = session.create_solid_brush(with_alpha(theme.solid_fill, plate_a))?;
            session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &fill);
        }
    }

    if opacity > 1.0 {
        let veil = session.create_solid_brush(with_alpha(theme.opacity_veil(opacity), plate_a))?;
        session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &veil);
    }
    if let Some(mut tint) = style.tint {
        // Per-fence colour: a translucent wash so the glass still reads as glass.
        tint.a = 0.22 * plate_a;
        if style.tint_title_only && style.tab_rail > 0.0 {
            // A left tab column is the title bar: the plate's rounded left corners, a straight
            // right edge where the icons start.
            let edge = (style.tab_rail / outer.width().max(1.0)).min(1.0);
            let soft = (px.hair() / outer.width().max(1.0)).min(1.0 - edge);
            let wash = session.create_linear_gradient(
                Vector2::new(outer.left, 0.0),
                Vector2::new(outer.right, 0.0),
                &[
                    GradientStop::new(0.0, tint),
                    GradientStop::new(edge, tint),
                    GradientStop::new(edge + soft, ColorF::TRANSPARENT),
                ],
            )?;
            session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &wash);
        } else if style.tint_title_only {
            // Only the title row (what of it the fold leaves): the plate's rounded top
            // corners, a straight bottom edge where the icons start.
            let band = theme.title_height.min(height) - plate_top;
            if band > 0.0 {
                let edge = band / outer.height().max(1.0);
                let soft = (px.hair() / outer.height().max(1.0)).min(1.0 - edge.min(1.0));
                let wash = session.create_linear_gradient(
                    Vector2::new(0.0, plate_top),
                    Vector2::new(0.0, outer.bottom),
                    &[
                        GradientStop::new(0.0, tint),
                        GradientStop::new(edge.min(1.0), tint),
                        GradientStop::new((edge + soft).min(1.0), ColorF::TRANSPARENT),
                    ],
                )?;
                session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &wash);
            }
        } else {
            let wash = session.create_solid_brush(tint)?;
            session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &wash);
        }
    }

    // A quiet, directional reflection gives sampled materials depth without adding a
    // header band. It is static at rest; existing title hover tweens gently lift the edge.
    let sampled_material = matches!(
        backdrop,
        Backdrop::Glass(..) | Backdrop::MicaLike(..) | Backdrop::GpuGlass { .. }
    );
    if sampled_material && !theme.liquid_glass {
        let sheen = session.create_linear_gradient(
            Vector2::new(0.0, plate_top),
            Vector2::new(width, height.max(plate_top + 1.0)),
            &[
                GradientStop::new(0.0, with_alpha(theme.glass_rim_top, 0.16 * plate_a)),
                GradientStop::new(0.48, ColorF::TRANSPARENT),
                GradientStop::new(1.0, with_alpha(theme.glass_rim_bottom, 0.12 * plate_a)),
            ],
        )?;
        session.fill_rounded_rect(&RoundedRect::uniform(outer, radius), &sheen);
    }

    // A single 1 px rim, lit from the upper left and snapped to device pixels at every DPI.
    let rim_rect = px.ring(outer, radius);
    if sampled_material && theme.liquid_glass {
        // The GPU material lights its own rim (kube's specular ring).
        if !matches!(backdrop, Backdrop::GpuGlass { .. }) {
            let saved = session.transform();
            session.set_transform(&(Matrix3x2::translation(0.0, plate_top) * saved));
            let drawn = crate::liquid_glass::draw_reflection(
                session,
                theme,
                scale,
                width,
                outer.height(),
                pill_a,
                opacity,
            );
            session.set_transform(&saved);
            drawn?;
        }
    } else if sampled_material {
        let rim = session.create_linear_gradient(
            Vector2::new(0.0, plate_top),
            Vector2::new(width * 0.65, height.max(plate_top + 1.0)),
            &[
                GradientStop::new(
                    0.0,
                    with_alpha(theme.glass_rim_top, (1.0 + 0.2 * pill_a) * plate_a),
                ),
                GradientStop::new(
                    0.55,
                    with_alpha(
                        lerp(theme.glass_rim_top, theme.glass_rim_bottom, 0.72),
                        plate_a,
                    ),
                ),
                GradientStop::new(1.0, with_alpha(theme.glass_rim_bottom, plate_a)),
            ],
        )?;
        session.draw_rounded_rect(&rim_rect, &rim, px.hair());
    } else {
        let stroke = session.create_solid_brush(with_alpha(theme.stroke, plate_a))?;
        session.draw_rounded_rect(&rim_rect, &stroke, px.hair());
    }
    Ok(())
}

/// Halo under a caption drawn in `ink` straight over wallpaper (Liquid Glass labels, a fully
/// transparent plate): a dark rim under light text, a light one under dark text.
fn halo_color(ink: ColorF, alpha: f32) -> ColorF {
    if ink.r > 0.5 {
        ColorF::new(0.0, 0.0, 0.0, 0.38 * alpha)
    } else {
        ColorF::new(1.0, 1.0, 1.0, 0.55 * alpha)
    }
}

/// Draws `text` in `rect`, over a four-way `halo` outline (¾ DIP each way) when one is given.
fn draw_caption(
    session: &DrawingSession<'_>,
    text: &str,
    format: &TextFormat,
    rect: &Rect,
    brush: &Brush,
    halo: Option<&Brush>,
) {
    let Some(halo) = halo else {
        session.draw_text(text, format, rect, brush);
        return;
    };
    // Shaped once for the five passes: `draw_text` lays the text out anew on every call.
    let Ok(layout) = TextLayout::new(text, format, rect.width(), rect.height()) else {
        session.draw_text(text, format, rect, brush);
        return;
    };
    for (dx, dy) in [(-0.75, 0.0), (0.75, 0.0), (0.0, -0.75), (0.0, 0.75)] {
        session.draw_text_layout(Vector2::new(rect.left + dx, rect.top + dy), &layout, halo);
    }
    session.draw_text_layout(Vector2::new(rect.left, rect.top), &layout, brush);
}

fn draw_legible_text(
    session: &DrawingSession<'_>,
    text: &str,
    format: &TextFormat,
    rect: &Rect,
    brush: &Brush,
    halo: Option<&Brush>,
    spread: f32,
) {
    if let Some(halo) = halo {
        let shifted = Rect {
            top: rect.top + spread,
            bottom: rect.bottom + spread,
            ..*rect
        };
        session.draw_text(text, format, &shifted, halo);
    }
    session.draw_text(text, format, rect, brush);
}

/// The hairline of a "按时间分组" section header: from 8 DIPs after the caption's end
/// (`text_end`) to `right`, one device pixel, vertically centred on the band.
fn draw_group_rule(
    session: &DrawingSession<'_>,
    px: Px,
    header: &GroupHeaderDraw<'_>,
    text_end: f32,
    right: f32,
    line: &Brush,
) {
    let x0 = px.snap(text_end + 8.0);
    let x1 = px.snap(right);
    if x1 > x0 {
        let y = px.snap(header.y + header.h / 2.0);
        session.fill_rect(&Rect::from_xywh(x0, y, x1 - x0, px.hair()), line);
    }
}

/// Rotation of the ChevronUp glyph (degrees) for a roll progress 0 (expanded) ..= 1 (rolled):
/// half a turn, so the resting rolled state is ChevronDown.
pub fn chevron_angle(roll_t: f32) -> f32 {
    180.0 * roll_t.clamp(0.0, 1.0)
}

/// How selected a row of a left tab column looks (0..=1): the shown tab while the fence is
/// `expanded` (1 - roll progress; a collapsed column shows its tabs alike), a dragged tab always.
pub fn rail_selected(active: bool, dragging: bool, expanded: f32) -> f32 {
    if dragging {
        1.0
    } else if active {
        expanded.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Opacity of a left tab column's chevron: the column's hover, faded out by the roll progress
/// (0 expanded ..= 1 collapsed to the column, which has no chevron).
pub fn rail_chevron_alpha(hover: f32, roll_t: f32) -> f32 {
    hover.clamp(0.0, 1.0) * (1.0 - roll_t.clamp(0.0, 1.0))
}

/// `BitmapCache` key of the acrylic noise tile (one upload per device).
/// How much of the fence plate is the (blurred, tinted) wallpaper crop and how much is the real
/// desktop behind the window showing through. 1.0 = the old fully opaque plate; the default lets
/// the desktop bleed through, which is what "transparency" means to a user. The per-fence
/// opacity setting (外观 → 更透明 / 更厚实) scales it.
pub const GLASS_OPACITY: f32 = 0.55;

fn glass_alpha(opacity: f32) -> f32 {
    (GLASS_OPACITY * opacity).clamp(0.1, 1.0)
}

const NOISE_BITMAP_KEY: &str = "backdrop:noise";

/// Extra selection-fill alpha of a hovered selected item (LISS_HOTSELECTED / WinUI
/// SelectedPointerOver): dark accent 0x33 -> 0x40, light 0x29 -> 0x36.
const HOT_SELECTED_LIFT: f32 = 0x0D as f32 / 255.0;

pub struct ContentDraw<'a> {
    pub items: &'a [ItemCell<'a>],
    /// Section headers between the row blocks of grouped items.
    pub group_headers: &'a [GroupHeaderDraw<'a>],
    pub icon_size: f32,
    pub label_lines: u8,
    pub line_h: f32,
    /// Cell top pad above the icon and gap between icon and label (GridMetrics).
    pub icon_top: f32,
    pub label_gap: f32,
    /// Overlay scrollbar thumb when the content overflows.
    pub scrollbar: Option<ScrollbarDraw>,
    pub empty_text: Option<&'a str>,
    /// Fence-wide drop wash + accent ring alpha 0..=1 (a drag hovers the fence itself).
    pub drop_highlight: f32,
    /// Rubber-band selection rectangle in content DIPs (already scrolled), while dragging.
    pub marquee: Option<Rect>,
    /// Insertion caret (accent bar) while an internal drag reorders the grid, and its alpha.
    pub insert_caret: Option<Rect>,
    pub insert_caret_alpha: f32,
    /// 0 = accent selection (active window), 1 = neutral grey (another window is active).
    pub selection_inactive: f32,
    /// The items end here (surface DIPs): 「调整大小时按整行整列对齐」 leaves the height under
    /// the whole rows empty. `f32::INFINITY` = the surface's bottom.
    pub items_bottom: f32,
}

/// The brushes an item-state pass needs, created once per content draw. The per-item alphas
/// only recolour them (`ID2D1SolidColorBrush::SetColor`, no allocation), so a selection fade
/// over hundreds of items costs no COM objects per frame.
struct StateBrushes {
    /// Recoloured per item: subtle (hover / pressed) fill, selection fill and stroke.
    hover: Brush,
    sel_fill: Brush,
    sel_stroke: Brush,
    /// Fixed colours.
    focus_outer: Brush,
    focus_inner: Brush,
    /// Scratch brushes recoloured per use (never two of the same at once): faded label text
    /// (primary / secondary), the icon loading tile, the drop wash + ring and the insert
    /// caret. Every colour that varies per frame or per item goes through these, so 30 items
    /// fading in cost no brush allocations per frame.
    scratch_a: Brush,
    scratch_b: Brush,
    /// The label halo over wallpaper (Liquid Glass, text_halo), recoloured per cell.
    halo: Brush,
}

impl StateBrushes {
    fn new(session: &DrawingSession<'_>, theme: &Theme) -> Result<Self> {
        Ok(Self {
            hover: session.create_solid_brush(theme.subtle_hover)?,
            sel_fill: session.create_solid_brush(theme.selection_fill)?,
            sel_stroke: session.create_solid_brush(theme.selection_stroke)?,
            focus_outer: session.create_solid_brush(theme.focus_outer)?,
            focus_inner: session.create_solid_brush(theme.focus_inner)?,
            scratch_a: session.create_solid_brush(theme.text_primary)?,
            scratch_b: session.create_solid_brush(theme.text_secondary)?,
            halo: session.create_solid_brush(halo_color(theme.text_primary, 1.0))?,
        })
    }

    /// `halo` recoloured to `c`.
    fn halo(&self, c: ColorF) -> &Brush {
        self.halo.set_color(c);
        &self.halo
    }

    /// `scratch_a` recoloured to `c`.
    fn a(&self, c: ColorF) -> &Brush {
        self.scratch_a.set_color(c);
        &self.scratch_a
    }

    /// `scratch_b` recoloured to `c`.
    fn b(&self, c: ColorF) -> &Brush {
        self.scratch_b.set_color(c);
        &self.scratch_b
    }
}

/// Paints one item's state layers (selection, hover, pressed, focus ring) under its content.
/// Precedence: a drop target is an accent selection (its alpha fades in over 83 ms and wins
/// over a fading selection); selection and hover blend by alpha so a fading selection still
/// shows the hover beneath it. WinUI ListViewItem states: pressed swaps the subtle layer from
/// SubtleFillColorSecondary to SubtleFillColorTertiary (Pressed *replaces* PointerOver, so the
/// surface recedes rather than darkens); a selected item shows only its selection while
/// pressed (SelectedPressed = Selected); a hovered selected item lifts its selection fill
/// (SelectedPointerOver / LISS_HOTSELECTED), riding the 83 ms hover fade.
#[allow(clippy::too_many_arguments)]
fn draw_item_state(
    session: &DrawingSession<'_>,
    theme: &Theme,
    brushes: &StateBrushes,
    px: Px,
    rect: Rect,
    hover: f32,
    selection: f32,
    inactive: f32,
    pressed: bool,
    focused: bool,
    drop_target: f32,
) {
    let drop = drop_target.clamp(0.0, 1.0);
    let sel = selection.max(drop);
    // One subtle layer: PointerOver = SubtleFillColorSecondary, Pressed = SubtleFillColorTertiary
    // (an instant colour swap of the same layer; its alpha keeps following the hover fade).
    // Kept under a partial selection so the two fades never pop.
    let subtle_a = hover.clamp(0.0, 1.0) * (1.0 - sel);
    if subtle_a > 0.0 {
        let subtle = if pressed {
            theme.subtle_pressed
        } else {
            theme.subtle_hover
        };
        brushes.hover.set_color(with_alpha(subtle, subtle_a));
        session.fill_rounded_rect(&RoundedRect::uniform(rect, 4.0), &brushes.hover);
    }
    if sel > 0.0 {
        // Accent while this fence is the active window; neutral grey otherwise (drop targets
        // stay accent — drag feedback is not selection).
        let t = inactive * (1.0 - drop);
        let fill = lerp(theme.selection_fill, theme.selection_fill_inactive, t);
        // Hot-selected: the fill strengthens under the pointer (drag hover is already 0).
        let hot = hover.clamp(0.0, 1.0) * selection.clamp(0.0, 1.0);
        let fill = ColorF {
            a: (fill.a + HOT_SELECTED_LIFT * hot).min(1.0),
            ..fill
        };
        let stroke = lerp(theme.selection_stroke, theme.selection_stroke_inactive, t);
        brushes.sel_fill.set_color(with_alpha(fill, sel));
        brushes.sel_stroke.set_color(with_alpha(stroke, sel));
        session.fill_rounded_rect(&RoundedRect::uniform(rect, 4.0), &brushes.sel_fill);
        session.draw_rounded_rect(&px.ring(rect, 4.0), &brushes.sel_stroke, px.hair());
    }
    if focused {
        // FocusVisual: 2 px outer stroke on the cell edge, 1 px inner stroke just inside it.
        let o = Rect::from_xywh(
            rect.left + 1.0,
            rect.top + 1.0,
            (rect.right - rect.left - 2.0).max(0.0),
            (rect.bottom - rect.top - 2.0).max(0.0),
        );
        session.draw_rounded_rect(&RoundedRect::uniform(o, 3.0), &brushes.focus_outer, 2.0);
        let i = Rect::from_xywh(
            rect.left + 2.0,
            rect.top + 2.0,
            (rect.right - rect.left - 4.0).max(0.0),
            (rect.bottom - rect.top - 4.0).max(0.0),
        );
        session.draw_rounded_rect(&px.ring(i, 2.0), &brushes.focus_inner, px.hair());
    }
}

/// Fence-wide drop feedback (a drag hovers the fence itself), faded by `alpha`: the accent
/// wash (plan §6.2 dropWash, accent @ 0x14) filled inside the 1 px rim with rounded bottom
/// corners concentric with the DWM window corner (the top edge abuts the title row, so it stays
/// straight — the rounded top corners are pushed above the surface), plus the 2 px opaque
/// accent ring (dropRing) one hairline inside the rim, again concentric with it. The rim
/// pixels themselves stay untinted.
fn draw_drop_wash(
    session: &DrawingSession<'_>,
    theme: &Theme,
    brushes: &StateBrushes,
    width: f32,
    height: f32,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let alpha = alpha.min(1.0);
    let r = (theme.corner_radius - 1.0).max(0.0);
    session.fill_rounded_rect(
        &RoundedRect::uniform(
            Rect::from_xywh(1.0, -r, (width - 2.0).max(0.0), (height - 1.0 + r).max(0.0)),
            r,
        ),
        brushes.a(with_alpha(theme.drop_highlight, alpha)),
    );
    // 2 px stroke centred 2 DIP inside the edge: it spans 1..3 DIP, so its outer edge touches
    // the rim's inner edge and its centreline radius is the window radius minus 2.
    session.draw_rounded_rect(
        &RoundedRect::uniform(
            Rect::from_xywh(2.0, 2.0, (width - 4.0).max(0.0), (height - 4.0).max(0.0)),
            (theme.corner_radius - 2.0).max(0.0),
        ),
        brushes.b(with_alpha(theme.accent, alpha)),
        2.0,
    );
}

/// Insertion caret (2 DIP accent bar) at `r`, faded by `alpha`; rows clip it to `min_top`.
fn draw_insert_caret(
    session: &DrawingSession<'_>,
    theme: &Theme,
    brushes: &StateBrushes,
    r: Rect,
    min_top: f32,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let r = Rect {
        left: r.left.round(),
        top: r.top.round().max(min_top),
        right: r.right.round(),
        bottom: r.bottom.round(),
    };
    if r.bottom > r.top {
        session.fill_rounded_rect(
            &RoundedRect::uniform(r, 1.0),
            brushes.a(with_alpha(theme.selection_stroke, alpha.min(1.0))),
        );
    }
}

/// Icon box of an item: the bitmap (aspect-preserved inside `size`), cross-faded over the
/// loading tile while `icon_alpha` < 1; a generic document glyph when extraction failed;
/// the tile alone while the icon is still pending. `alpha` fades the whole thing.
#[allow(clippy::too_many_arguments)]
fn draw_item_icon(
    session: &DrawingSession<'_>,
    theme: &Theme,
    brushes: &StateBrushes,
    bitmaps: &mut BitmapCache,
    icon_key: &str,
    icon: Option<&Image>,
    failed: bool,
    dimmed: bool,
    alpha: f32,
    icon_alpha: f32,
    x: f32,
    y: f32,
    size: f32,
    tile_radius: f32,
) -> Result<()> {
    let tile = |a: f32| {
        if a <= 0.0 {
            return;
        }
        session.fill_rounded_rect(
            &RoundedRect::uniform(Rect::from_xywh(x, y, size, size), tile_radius),
            brushes.a(with_alpha(theme.subtle_pressed, a)),
        );
    };
    if let Some(img) = icon {
        let icon_alpha = icon_alpha.clamp(0.0, 1.0);
        if icon_alpha < 1.0 && !failed {
            tile(alpha * (1.0 - icon_alpha));
        }
        let bmp = bitmaps.get_or_upload(session, icon_key, img)?;
        // Preserve aspect ratio inside the icon box (thumbnails are not square).
        let (iw, ih) = (img.width as f32, img.height as f32);
        let k = (size / iw).min(size / ih);
        let (dw, dh) = (iw * k, ih * k);
        let dest = Rect::from_xywh(x + (size - dw) / 2.0, y + (size - dh) / 2.0, dw, dh);
        let dim = if dimmed { 0.5 } else { 1.0 };
        session.draw_bitmap(&bmp, &dest, dim * alpha * icon_alpha);
    } else if failed {
        // No icon obtainable (broken shortcut, missing handler): a generic document glyph
        // from Segoe Fluent Icons instead of a permanent grey tile.
        let glyph_format = TextFormat::new(crate::theme::font_icons(), size * 0.72)?
            .with_alignment(TextAlignment::Center)
            .with_paragraph_alignment(ParagraphAlignment::Center)
            .with_word_wrapping(WordWrapping::NoWrap);
        session.draw_text(
            "\u{E7C3}",
            &glyph_format,
            &Rect::from_xywh(x, y, size, size),
            brushes.a(with_alpha(theme.text_secondary, alpha)),
        );
    } else {
        // Placeholder while the icon loads.
        tile(alpha);
    }
    Ok(())
}

/// Overlay scrollbar (Windows 11 / WinUI ScrollBar). Collapsed: a ControlStrong 2 DIP pill.
/// Expanded (`parts` > 0): the 12 DIP track fill (LayerFill stands in for the in-app acrylic)
/// with an arrow RepeatButton at each end — 8 px caret in ControlStrong, TextSecondary when
/// hovered / pressed, SubtleFill hover pill, 0.875 glyph scale while pressed — and the thumb
/// widened to 6 DIP on top. Widths, alphas and the parts reveal come from the caller's tweens.
fn draw_scrollbar(
    session: &DrawingSession<'_>,
    theme: &Theme,
    arrow_format: &TextFormat,
    width: f32,
    bar: ScrollbarDraw,
) -> Result<()> {
    if bar.alpha <= 0.0 || bar.width <= 0.0 {
        return Ok(());
    }
    let parts = (bar.parts.clamp(0.0, 1.0) * bar.alpha).clamp(0.0, 1.0);
    if parts > 0.0 && bar.band_h > 0.0 {
        let x = width - ScrollbarDraw::BAR_W;
        let track = Rect::from_xywh(x, bar.band_top, ScrollbarDraw::BAR_W, bar.band_h);
        let fill = session.create_solid_brush(with_alpha(theme.layer_fill, parts))?;
        session.fill_rounded_rect(
            &RoundedRect::uniform(track, ScrollbarDraw::BAR_W / 2.0),
            &fill,
        );
        let boxes = [
            Rect::from_xywh(
                x,
                bar.band_top,
                ScrollbarDraw::BAR_W,
                ScrollbarDraw::BUTTON_H,
            ),
            Rect::from_xywh(
                x,
                bar.band_top + bar.band_h - ScrollbarDraw::BUTTON_H,
                ScrollbarDraw::BAR_W,
                ScrollbarDraw::BUTTON_H,
            ),
        ];
        let glyphs = [ScrollbarDraw::ARROW_UP, ScrollbarDraw::ARROW_DOWN];
        for i in 0..2 {
            let box_r = boxes[i];
            let hover = bar.arrow_hover[i].clamp(0.0, 1.0);
            let pressed = bar.arrow_pressed[i];
            if pressed {
                // WinUI ScrollBarButtonBackgroundPressed is SubtleFillColorTertiary (the plan's
                // ControlStrong is the glyph colour, not the plate).
                let bg = session.create_solid_brush(with_alpha(theme.subtle_pressed, parts))?;
                session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &bg);
            } else if hover > 0.0 {
                let bg =
                    session.create_solid_brush(with_alpha(theme.subtle_hover, hover * parts))?;
                session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &bg);
            }
            let hot = if pressed { 1.0 } else { hover };
            let color = lerp(theme.control_strong, theme.text_secondary, hot);
            let glyph = session.create_solid_brush(with_alpha(color, parts))?;
            if pressed {
                let saved = session.transform();
                let centre = Vector2::new(
                    box_r.left + box_r.width() / 2.0,
                    box_r.top + box_r.height() / 2.0,
                );
                let s = ScrollbarDraw::ARROW_SCALE_PRESSED;
                session.set_transform(&(Matrix3x2::scale_around(s, s, centre) * saved));
                session.draw_text(glyphs[i], arrow_format, &box_r, &glyph);
                session.set_transform(&saved);
            } else {
                session.draw_text(glyphs[i], arrow_format, &box_r, &glyph);
            }
        }
    }
    let brush = session.create_solid_brush(with_alpha(theme.control_strong, bar.alpha))?;
    let w = bar
        .width
        .clamp(ScrollbarDraw::REST_WIDTH, ScrollbarDraw::EXPANDED_WIDTH);
    session.fill_rounded_rect(
        &RoundedRect::uniform(
            Rect::from_xywh(
                ScrollbarDraw::thumb_x(width, w),
                bar.thumb_y,
                w,
                bar.thumb_h,
            ),
            w / 2.0,
        ),
        &brush,
    );
    Ok(())
}

/// Locale for all fence text. DirectWrite fills Han, kana and Hangul from the font of this
/// locale; like Explorer, it follows the Windows display language, not PecoFence's, because
/// fences show file names. Traditional Chinese, Japanese and Korean get their own UI font;
/// everything else gets Microsoft YaHei UI (upstream's `en-us` would pick Yu Gothic UI and draw
/// Chinese names with Japanese glyph forms).
fn text_locale() -> &'static str {
    match pecofence_platform::locale::ui_language() {
        "zh-TW" => "zh-TW",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        _ => "zh-CN",
    }
}

impl FenceChrome {
    pub fn new() -> Result<Self> {
        let locale = text_locale();
        // WinUI type ramp: Body Strong 14/600 for the title, Caption 12 (Small optical size) for
        // labels and the rolled item count, Body 14 for the empty state. No letter-spacing.
        let count_format = TextFormat::with_locale(font_small(), 12.0, FontWeight::NORMAL, locale)?
            .with_alignment(TextAlignment::Trailing)
            .with_paragraph_alignment(ParagraphAlignment::Center)
            .with_word_wrapping(WordWrapping::NoWrap);
        let label_font = RefCell::new(LabelFont {
            format: Self::build_label_format(font_small(), 12.0, FontWeight(400))?,
            family: "Segoe UI".to_string(),
            size: 12.0,
            weight: 400,
            line_h: DEFAULT_LABEL_LINE_H,
        });
        let glyph_format =
            TextFormat::with_locale(crate::theme::font_icons(), 12.0, FontWeight::NORMAL, locale)?
                .with_alignment(TextAlignment::Center)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap);
        let row_format = TextFormat::with_locale(font_small(), 12.0, FontWeight::NORMAL, locale)?
            .with_alignment(TextAlignment::Leading)
            .with_paragraph_alignment(ParagraphAlignment::Center)
            .with_word_wrapping(WordWrapping::NoWrap);
        let row_format_right =
            TextFormat::with_locale(font_small(), 12.0, FontWeight::NORMAL, locale)?
                .with_alignment(TextAlignment::Trailing)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap);
        let sort_glyph_format =
            TextFormat::with_locale(crate::theme::font_icons(), 8.0, FontWeight::NORMAL, locale)?
                .with_alignment(TextAlignment::Center)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap);
        let empty_format = TextFormat::with_locale(font_text(), 14.0, FontWeight::NORMAL, locale)?
            .with_alignment(TextAlignment::Center)
            .with_paragraph_alignment(ParagraphAlignment::Top)
            .with_word_wrapping(WordWrapping::Wrap);
        let title_glyph_format =
            TextFormat::with_locale(crate::theme::font_icons(), 14.0, FontWeight::NORMAL, locale)?
                .with_alignment(TextAlignment::Center)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap);
        let group_format = TextFormat::with_locale(font_small(), 12.0, FontWeight(600), locale)?
            .with_alignment(TextAlignment::Leading)
            .with_paragraph_alignment(ParagraphAlignment::Center)
            .with_word_wrapping(WordWrapping::NoWrap);
        let badge_format = TextFormat::with_locale(font_small(), 11.0, FontWeight(600), locale)?
            .with_alignment(TextAlignment::Center)
            .with_paragraph_alignment(ParagraphAlignment::Center)
            .with_word_wrapping(WordWrapping::NoWrap);
        let chrome = Self {
            title_fonts: RefCell::new(Self::build_title_fonts(None, 600)?),
            count_format,
            label_font,
            glyph_format,
            row_format,
            row_format_right,
            sort_glyph_format,
            empty_format,
            title_glyph_format,
            group_format,
            badge_format,
        };
        // Labels follow the desktop's icon-title font from the start (12 px / 16 stays the
        // fallback when the system call fails).
        if let Some(f) = pecofence_platform::sysparams::icon_title_font() {
            let _ = chrome.set_icon_title_font(&f.family, f.size_dip, FontWeight(f.weight));
        }
        Ok(chrome)
    }

    /// `face` for every title size, or the built-in faces (the Small optical size at 12 px).
    fn build_title_fonts(face: Option<&str>, weight: i32) -> Result<TitleFonts> {
        let locale = text_locale();
        let format =
            |size: f32, weight: i32, align: TextAlignment| -> Result<TextFormat> {
                let built_in = if size < 13.0 {
                    font_small()
                } else {
                    font_text()
                };
                Ok(TextFormat::with_locale(
                    face.unwrap_or(built_in),
                    size,
                    FontWeight(weight),
                    locale,
                )?
                .with_alignment(align)
                .with_paragraph_alignment(ParagraphAlignment::Center)
                .with_word_wrapping(WordWrapping::NoWrap))
            };
        let sizes = [12.0, 14.0, 16.0, 18.0];
        let set = |weight: i32, align: TextAlignment| -> Result<[TextFormat; 4]> {
            Ok([
                format(sizes[0], weight, align)?,
                format(sizes[1], weight, align)?,
                format(sizes[2], weight, align)?,
                format(sizes[3], weight, align)?,
            ])
        };
        Ok(TitleFonts {
            titles: set(weight, TextAlignment::Leading)?,
            tabs: set(weight, TextAlignment::Center)?,
            tab_lists: set(FontWeight::NORMAL.0, TextAlignment::Leading)?,
            face: face.map(str::to_string),
            weight,
        })
    }

    /// A theme's title font (`face`, `weight`; None = Segoe UI Variable, semibold) for titles
    /// and tabs. Returns whether anything changed (the caller then lays the title rows out).
    pub fn set_title_font(&self, face: Option<&str>, weight: Option<i32>) -> Result<bool> {
        let weight = weight.unwrap_or(600);
        {
            let cur = self.title_fonts.borrow();
            if cur.face.as_deref() == face && cur.weight == weight {
                return Ok(false);
            }
        }
        *self.title_fonts.borrow_mut() = Self::build_title_fonts(face, weight)?;
        Ok(true)
    }

    fn build_label_format(family: &str, size: f32, weight: FontWeight) -> Result<TextFormat> {
        Ok(
            TextFormat::with_locale(family, size, weight, text_locale())?
                .with_alignment(TextAlignment::Center)
                .with_paragraph_alignment(ParagraphAlignment::Top)
                .with_word_wrapping(WordWrapping::Wrap),
        )
    }

    /// Adopts the system icon-title font (`family` / `size` DIPs / `weight`) for labels and
    /// re-measures the line height. "Segoe UI" maps to the Small optical size of Segoe UI
    /// Variable so the default keeps its Fluent look; any other family is used verbatim.
    /// Returns whether anything changed (the caller then re-fits labels and re-lays out).
    pub fn set_icon_title_font(&self, family: &str, size: f32, weight: FontWeight) -> Result<bool> {
        let size = size.clamp(8.0, 40.0);
        {
            let cur = self.label_font.borrow();
            if cur.family == family && cur.size == size && cur.weight == weight.0 {
                return Ok(false);
            }
        }
        let face = if family.eq_ignore_ascii_case("Segoe UI") {
            font_small()
        } else {
            family
        };
        let format = Self::build_label_format(face, size, weight)?;
        let line_h = TextLayout::new("Ag", &format, 10_000.0, 10_000.0)
            .map(|l| l.metrics().height.ceil())
            .unwrap_or(DEFAULT_LABEL_LINE_H)
            .max(1.0);
        *self.label_font.borrow_mut() = LabelFont {
            format,
            family: family.to_string(),
            size,
            weight: weight.0,
            line_h,
        };
        Ok(true)
    }

    /// Label text format (a cheap COM clone; borrow it for one fit / draw).
    pub fn label_format(&self) -> TextFormat {
        self.label_font.borrow().format.clone()
    }

    /// One label line in DIPs for the current icon-title font (16 at the default 12 px).
    pub fn label_line_h(&self) -> f32 {
        self.label_font.borrow().line_h
    }

    /// Title at 12 / 14 / 16 / 18 px for the per-fence "title size" option (0 to 3).
    pub fn title_format(&self, size: u8) -> Ref<'_, TextFormat> {
        Ref::map(self.title_fonts.borrow(), |f| {
            &f.titles[usize::from(size.min(3))]
        })
    }

    /// Layout, drawing and tooltip fitting all use the same title size.
    pub fn tab_format(&self, size: u8) -> Ref<'_, TextFormat> {
        Ref::map(self.title_fonts.borrow(), |f| {
            &f.tabs[usize::from(size.min(3))]
        })
    }

    /// Width a match badge reading `text` adds to a tab pill, its gap to the caption included.
    pub fn tab_badge_width(&self, text: &str) -> f32 {
        (crate::text::measure_width(text, &self.badge_format) + 2.0 * TAB_BADGE_PAD)
            .max(TAB_BADGE_H)
            + TAB_BADGE_GAP
    }

    /// Single-line row text format (for fitting names to the name column).
    pub fn row_format(&self) -> &TextFormat {
        &self.row_format
    }

    /// "按时间分组" section caption format (12 px semibold, like the small title).
    pub fn group_format(&self) -> &TextFormat {
        &self.group_format
    }

    /// Does a tab caption fit its pill of width `w` without ellipsis (tooltip decision)?
    pub fn tab_title_fits(
        &self,
        title: &str,
        w: f32,
        size: u8,
        folder_icon: bool,
        theme: &Theme,
    ) -> bool {
        let content_w =
            crate::text::measure_width(title, &self.tab_format(size)) + tab_icon_w(folder_icon);
        content_w <= (w - tab_text_inset(theme, w, content_w)).max(0.0)
    }

    /// Lays out the caption of a tab pill `w` DIPs wide, its `x` counted from the pill's left
    /// edge. With a folder glyph or a badge, they and the caption are centred in the pill as
    /// one group.
    fn tab_caption(
        &self,
        theme: &Theme,
        w: f32,
        title: &str,
        size: u8,
        folder_icon: bool,
        badge: Option<&str>,
    ) -> TabCaption {
        let fmt = &self.tab_format(size);
        let badge_w = badge.map_or(0.0, |b| self.tab_badge_width(b));
        let icon_w = tab_icon_w(folder_icon);
        let title_w = crate::text::measure_width(title, fmt);
        let padding = tab_text_inset(theme, w - badge_w, title_w + icon_w);
        let caption_w = (w - badge_w - icon_w - padding).max(0.0);
        let fitted = crate::text::fit_width(title, fmt, caption_w);
        // An unelided caption keeps the width measured above.
        let shown_w = if badge.is_some() || folder_icon {
            if fitted == title {
                title_w
            } else {
                crate::text::measure_width(&fitted, fmt)
            }
            .min(caption_w)
        } else {
            caption_w
        };
        TabCaption {
            x: icon_w + (w - icon_w - shown_w - badge_w) * 0.5,
            w: shown_w,
            fitted,
            badge_w,
        }
    }

    /// The up button of a folder tab's pill at `x`, `w` wide, in a title-row strip: the square
    /// around its folder glyph, as drawn. The app hit-tests the same box.
    pub fn tab_up_box(
        &self,
        theme: &Theme,
        x: f32,
        w: f32,
        title: &str,
        size: u8,
        badge: Option<&str>,
    ) -> Rect {
        let caption = self.tab_caption(theme, w, title, size, true, badge);
        glyph_up_square(tab_glyph_centre(x + caption.x), theme.title_height)
    }

    /// Lays out a single title (no tab strip) in a title row `width` wide that keeps `reserve`
    /// DIPs free for the rolled item count: where its unit (the folder glyph, then the title)
    /// starts, the fitted title and the room the title gets.
    fn lone_title_place(
        &self,
        title: &str,
        size: u8,
        align: pecofence_core::TitleAlign,
        width: f32,
        deco: TitleDeco,
        reserve: f32,
    ) -> (f32, String, f32) {
        let fmt = &self.title_format(size);
        let start = if deco.up_at_left() {
            16.0 + TitleDeco::UP_W
        } else {
            16.0
        };
        let glyph_w = if deco.folder_icon {
            TitleDeco::ICON_W
        } else {
            0.0
        };
        let end = width - TITLE_RIGHT_RESERVE - reserve;
        let title_w = (end - start - glyph_w).max(0.0);
        let fitted = crate::text::fit_width(title, fmt, title_w);
        let unit_w = glyph_w + crate::text::measure_width(&fitted, fmt).min(title_w);
        (aligned_x(align, width, start, end, unit_w), fitted, title_w)
    }

    /// The up button of a single folder title inside a subfolder: the square around its folder
    /// glyph, which moves with the title's alignment. The app hit-tests the same box.
    pub fn lone_up_box(
        &self,
        title: &str,
        size: u8,
        align: pecofence_core::TitleAlign,
        width: f32,
        reserve: f32,
        title_h: f32,
    ) -> Rect {
        let deco = TitleDeco {
            folder_icon: true,
            up_button: true,
        };
        let (x, _, _) = self.lone_title_place(title, size, align, width, deco, reserve);
        glyph_up_square(x + 8.0, title_h)
    }

    /// Width a caption gets in a row of a `rail_w` DIP left tab column: after the colour dot's
    /// room when any tab has a colour and the folder glyph's when any tab has one, before the
    /// badge and the portal up button.
    pub fn rail_title_room(
        &self,
        rail_w: f32,
        colored: bool,
        folders: bool,
        badge: Option<&str>,
        up: bool,
    ) -> f32 {
        let row_w = (rail_w - RAIL_ROW_INSET - RAIL_ROW_RIGHT).max(0.0);
        let text_left = rail_text_left(colored, folders);
        let badge_w = badge.map_or(0.0, |b| self.tab_badge_width(b) - TAB_BADGE_GAP);
        let badge_gap = if badge_w > 0.0 { 6.0 } else { 0.0 };
        let up_w = if up { 26.0 } else { 0.0 };
        (row_w - text_left - 8.0 - badge_w - badge_gap - up_w).max(0.0)
    }

    /// Caption format of a left tab column row: semibold for the shown tab.
    fn rail_title_format(&self, size: u8, active: bool) -> Ref<'_, TextFormat> {
        let i = usize::from(size.min(3));
        Ref::map(self.title_fonts.borrow(), |f| {
            if active {
                &f.titles[i]
            } else {
                &f.tab_lists[i]
            }
        })
    }

    /// `title` shows whole in `room` DIPs of a left tab column row.
    pub fn rail_title_fits(&self, title: &str, room: f32, size: u8, active: bool) -> bool {
        crate::text::measure_width(title, &self.rail_title_format(size, active)) <= room
    }

    /// Draws the chrome into a `width` x `height` DIP area whose origin is (0, 0). `scale` is
    /// the DIP→device-pixel factor (hairline snapping); `state` carries the sampled fades.
    /// `rolled_up` is the state the fence is rolling to (or resting in) and `roll_t` the roll
    /// progress 0 (expanded) ..= 1 (rolled) that turns the chevron. `bitmaps` holds the
    /// uploaded wallpaper crop and the acrylic noise tile across frames.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        session: &DrawingSession<'_>,
        bitmaps: &mut BitmapCache,
        theme: &Theme,
        scale: f32,
        width: f32,
        height: f32,
        backdrop: Backdrop<'_>,
        title: &str,
        caption: Option<&str>,
        rolled_up: bool,
        roll_t: f32,
        opacity: f32,
        tabs: &[TabDraw<'_>],
        pill: Option<(f32, f32)>,
        deco: TitleDeco,
        state: TitleState,
        style: FenceStyle,
    ) -> Result<()> {
        session.clear(ColorF::TRANSPARENT);
        let px = Px::new(scale);
        // Per-fence opacity: <1 thins the layer over the glass, >1 adds a solid veil on top.
        let opacity = opacity.clamp(0.4, 1.8);
        // The plate is the whole surface, or only the part below the title row while title on
        // hover keeps that row folded away (the compositor clip cuts the surface at its top).
        // A fully transparent fence fades the whole plate with `plate_a`.
        let plate_top = state.plate_top.clamp(0.0, (height - 1.0).max(0.0));
        let plate_a = 1.0 - state.plate_clear.clamp(0.0, 1.0);
        let outer = Rect::from_xywh(0.0, plate_top, width, height - plate_top);
        let radius = theme
            .corner_radius
            .min(width.min(outer.height()) * 0.5)
            .max(0.0);
        // Title-row reveal ("title on hover"); a rolled fence always shows its title row.
        let title_a = if rolled_up {
            1.0
        } else {
            state.title.clamp(0.0, 1.0)
        };
        let title_h = theme.title_height.min(height);
        let pill_a = state.hover * title_a;
        // Two or more tabs on the left: the column is the title bar, there is no title row.
        let column = style.tab_rail > 0.0 && tabs.len() > 1;

        if plate_a > 0.0 {
            draw_plate(
                session,
                bitmaps,
                theme,
                scale,
                width,
                height,
                outer,
                radius,
                &backdrop,
                opacity,
                plate_a,
                pill_a,
                state.hover,
                style,
            )?;
        }

        // No header band, no divider: the title sits directly on the glass. Hovering a
        // single-title fence shows an inset pill (SubtleFill, 83 ms fade), never a full-width
        // strip; a tab strip gets its feedback from the individual tab pills instead.
        if pill_a > 0.0 && tabs.len() <= 1 {
            let row = Rect::from_xywh(4.0, 4.0, (width - 8.0).max(0.0), (title_h - 8.0).max(0.0));
            if theme.liquid_glass {
                draw_header_glass(session, &backdrop, theme, scale, 1, row, 0.28, pill_a)?;
            } else {
                let pill = session.create_solid_brush(with_alpha(theme.subtle_hover, pill_a))?;
                session.fill_rounded_rect(
                    &RoundedRect::uniform(row, header_control_radius(theme, row)),
                    &pill,
                );
            }
        }

        if state.merge_hint > 0.0 {
            // Drag-to-merge target: accent wash over the title row plus the selection ring,
            // fading in over 83 ms and out over 167 ms.
            let a = state.merge_hint.min(1.0);
            let wash = session.create_solid_brush(with_alpha(theme.drop_highlight, a))?;
            // Over the left tab column when that is the title bar.
            let row = if column {
                Rect::from_xywh(
                    4.0,
                    4.0,
                    (style.tab_rail - 8.0).max(0.0),
                    (height - 8.0).max(0.0),
                )
            } else {
                Rect::from_xywh(4.0, 4.0, (width - 8.0).max(0.0), (title_h - 8.0).max(0.0))
            };
            let radius = header_control_radius(theme, row);
            if theme.liquid_glass {
                draw_header_glass(session, &backdrop, theme, scale, 2, row, 0.5, a)?;
            }
            session.fill_rounded_rect(&RoundedRect::uniform(row, radius), &wash);
            let ring = session.create_solid_brush(with_alpha(theme.selection_stroke, a))?;
            session.draw_rounded_rect(&px.ring(row, radius), &ring, px.hair());
        }
        // A test-only marker lets a lossless screen recording locate the actually
        // presented window independently of GetWindowRect and render submission times.
        #[cfg(debug_assertions)]
        if title == "运动探针" && pecofence_core::brand::var_os("PECOFENCE_PRESENT_PROBE").is_some()
        {
            let marker = session.create_solid_brush(ColorF::new(1.0, 0.0, 1.0, 1.0))?;
            session.fill_rect(
                &Rect::from_xywh(width * 0.5 - 2.0, height * 0.5 + 6.0, 4.0, 4.0),
                &marker,
            );
        }
        if column {
            let halo = if theme.text_halo {
                Some(session.create_solid_brush(ColorF::TRANSPARENT)?)
            } else {
                None
            };
            self.draw_tab_list(
                session,
                &backdrop,
                theme,
                scale,
                opacity,
                style.tint,
                tabs,
                pill,
                halo.as_ref(),
                style.tab_rail,
                height,
                1.0 - roll_t.clamp(0.0, 1.0),
                deco,
                state,
            )?;
            // ChevronLeft (E76B) collapses the fence to its column, revealed with the column's
            // hover like the title row's chevron. A collapsed column has none (a tab click
            // expands it): the chevron turns as it fades out with the collapse, and back in
            // as the fence expands.
            if style.hide_count_and_chevron {
                return Ok(());
            }
            let chevron_a = rail_chevron_alpha(state.hover, roll_t);
            let rows_end = tabs.iter().map(|t| t.x + t.w).fold(0.0, f32::max);
            let box_r = TitleDeco::rail_chevron_box(style.tab_rail, height, rows_end);
            return self.draw_chevron(
                session, &backdrop, theme, scale, opacity, style.tint, box_r, "\u{E76B}",
                chevron_a, state, roll_t,
            );
        }
        if title_a <= 0.0 {
            // "鼠标悬停时才显示标题栏": the glass and rim stay, the title row stays empty.
            return Ok(());
        }
        // Title text; leave room for the roll-up chevron on the right, ellipsize the rest.
        let title_width = crate::text::measure_width(title, &self.title_format(style.title_size))
            .min((width - deco.title_x() - TITLE_RIGHT_RESERVE).max(1.0));
        let sample_x = aligned_x(
            style.title_align,
            width,
            deco.title_x(),
            width - TITLE_RIGHT_RESERVE,
            title_width,
        );
        let foreground = header_foreground(
            &backdrop,
            theme,
            5,
            Rect::from_xywh(sample_x, 0.0, title_width, title_h),
            scale,
            opacity,
            style.tint,
        );
        // Captions over bare wallpaper (a fully transparent plate) get a halo.
        let halo = if theme.text_halo {
            Some(session.create_solid_brush(ColorF::TRANSPARENT)?)
        } else {
            None
        };
        let text = session.create_solid_brush(with_alpha(
            style.title_color.unwrap_or(foreground.text_primary),
            title_a,
        ))?;
        let secondary =
            session.create_solid_brush(with_alpha(foreground.text_secondary, title_a))?;
        let tertiary = session.create_solid_brush(with_alpha(foreground.text_tertiary, title_a))?;
        let title_format = &self.title_format(style.title_size);
        // Rolled plate: the item count sits before the chevron so a rolled fence still tells
        // you what is inside (it fades out as the fence expands), unless the user hid it.
        let caption = caption.filter(|_| !style.hide_count_and_chevron);
        let count_a = state.count.clamp(0.0, 1.0) * title_a;
        let count_w = if caption.is_some() && (rolled_up || count_a > 0.0) {
            header_count_width(width, tabs.len(), deco.title_x())
        } else {
            0.0
        };
        // The title's ellipsis point moves with the caption fade instead of stepping when
        // the fade ends: the reserve is the caption's width scaled by its alpha (held at the
        // full width while rolling up, so the caption never fades in over the title).
        let title_reserve = if caption.is_some() {
            count_w * count_a.max(if rolled_up { 1.0 } else { 0.0 })
        } else {
            0.0
        };
        // The roll-up chevron: shown while rolled or rolling, otherwise on hover, never when the
        // user hid it.
        let chevron_shown = if style.hide_count_and_chevron {
            0.0
        } else if rolled_up || roll_t > 0.0 {
            1.0
        } else {
            state.hover.clamp(0.0, 1.0)
        };
        if tabs.len() > 1 {
            let strip_fade = match state.strip_clip {
                Some(clip) => Some(push_strip_fade(session, clip, title_h, chevron_shown)?),
                None => None,
            };
            let inset = if theme.liquid_glass { 4.0 } else { 6.0 };
            let pill_h = (title_h - inset * 2.0).max(0.0);
            let tab_rect = |x, w| Rect::from_xywh(x, inset, w, pill_h);
            let marker = |x: f32, w: f32, color: ColorF, alpha: f32| -> Result<()> {
                let marker_w = (w - 24.0).clamp(0.0, 20.0);
                if marker_w > 0.0 {
                    let brush = session.create_solid_brush(with_alpha(color, alpha))?;
                    session.fill_rounded_rect(
                        &RoundedRect::uniform(
                            Rect::from_xywh(
                                x + (w - marker_w) * 0.5,
                                title_h - inset - 3.0,
                                marker_w,
                                2.0,
                            ),
                            1.0,
                        ),
                        &brush,
                    );
                }
                Ok(())
            };
            // Paint the resting lenses first; the moving selection must stay above them.
            if theme.liquid_glass {
                for tab in tabs.iter().filter(|t| !t.dragging) {
                    if tab.active && pill.is_none() {
                        continue;
                    }
                    draw_header_glass(
                        session,
                        &backdrop,
                        theme,
                        scale,
                        tab.key,
                        tab_rect(tab.x, tab.w),
                        0.16 + 0.32 * tab.hover + 0.3 * tab.drop_target,
                        title_a * tab.alpha.clamp(0.0, 1.0),
                    )?;
                }
            }
            if let Some((x, w)) = pill {
                let rect = tab_rect(x, w);
                if theme.liquid_glass {
                    draw_header_glass(session, &backdrop, theme, scale, 0, rect, 0.78, title_a)?;
                } else {
                    let fill =
                        session.create_solid_brush(with_alpha(theme.tab_active_fill, title_a))?;
                    session.fill_rounded_rect(
                        &RoundedRect::uniform(rect, header_control_radius(theme, rect)),
                        &fill,
                    );
                }
                let color = tabs
                    .iter()
                    .find(|t| t.active)
                    .and_then(|t| t.color)
                    .unwrap_or(theme.accent);
                marker(x, w, color, title_a)?;
            }
            // A dragged tab (lens, marker and caption) is always above its neighbours.
            for tab in tabs
                .iter()
                .filter(|t| !t.dragging)
                .chain(tabs.iter().filter(|t| t.dragging))
            {
                let ta = title_a * tab.alpha.clamp(0.0, 1.0);
                if ta <= 0.0 {
                    continue;
                }
                let rect = tab_rect(tab.x, tab.w);
                let radius = header_control_radius(theme, rect);
                if pill.is_none() && (tab.active || tab.pressed || tab.dragging) {
                    if theme.liquid_glass {
                        let amount = if tab.pressed && !tab.dragging {
                            0.94
                        } else {
                            0.78
                        };
                        draw_header_glass(session, &backdrop, theme, scale, 0, rect, amount, ta)?;
                    } else {
                        let fill =
                            session.create_solid_brush(with_alpha(theme.tab_active_fill, ta))?;
                        session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
                    }
                } else if !theme.liquid_glass && tab.hover > 0.0 && !tab.active {
                    let fill = session
                        .create_solid_brush(with_alpha(theme.subtle_hover, tab.hover * ta))?;
                    session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
                }
                if tab.drop_target > 0.0 {
                    let a = tab.drop_target.min(1.0) * ta;
                    let fill = session.create_solid_brush(with_alpha(theme.selection_fill, a))?;
                    session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
                    let ring = session.create_solid_brush(with_alpha(theme.selection_stroke, a))?;
                    session.draw_rounded_rect(&px.ring(rect, radius), &ring, px.hair());
                }
                let fmt = &self.tab_format(tab.title_size);
                let icon_w = tab_icon_w(tab.folder_icon);
                let TabCaption {
                    x: caption_dx,
                    w: shown_w,
                    fitted,
                    badge_w,
                } = self.tab_caption(
                    theme,
                    tab.w,
                    tab.title,
                    tab.title_size,
                    tab.folder_icon,
                    tab.badge,
                );
                let caption_x = tab.x + caption_dx;
                // A persistent marker identifies the selected tab even on uniform wallpaper. It
                // stays under the glyph and caption when a badge widens the pill.
                let (marker_x, marker_w) = if tab.badge.is_some() {
                    let w = tab.w - badge_w;
                    (caption_x - icon_w + (icon_w + shown_w - w) * 0.5, w)
                } else {
                    (tab.x, tab.w)
                };
                if pill.is_none() && tab.active {
                    marker(marker_x, marker_w, tab.color.unwrap_or(theme.accent), ta)?;
                } else if !tab.active
                    && let Some(color) = tab.color
                {
                    marker(marker_x, marker_w, color, ta * 0.7)?;
                }
                let foreground =
                    header_foreground(&backdrop, theme, tab.key, rect, scale, opacity, style.tint);
                let color = if tab.pressed && !tab.dragging {
                    foreground.text_tertiary
                } else {
                    tab.title_color
                        .unwrap_or(if tab.active || tab.drop_target >= 0.5 {
                            foreground.text_primary
                        } else {
                            foreground.text_secondary
                        })
                };
                let brush = session.create_solid_brush(with_alpha(color, ta))?;
                if let Some(halo) = &halo {
                    halo.set_color(halo_color(color, ta));
                }
                if tab.folder_icon {
                    let slot = Rect::from_xywh(caption_x - icon_w - 2.0, 0.0, icon_w, title_h);
                    if tab.up_button {
                        // Inside a subfolder the folder glyph is the up button: Up (Segoe
                        // Fluent E74A) in the caption's ink, with the lone portal button's
                        // hover fill and pressed look in a square around it.
                        let box_r = glyph_up_square(tab_glyph_centre(caption_x), title_h);
                        draw_up_box_fill(session, &backdrop, theme, scale, box_r, state, ta)?;
                        let ink = session.create_solid_brush(with_alpha(
                            if state.up_pressed {
                                foreground.text_tertiary
                            } else {
                                color
                            },
                            ta,
                        ))?;
                        session.draw_text("\u{E74A}", &self.title_glyph_format, &slot, &ink);
                    } else {
                        session.draw_text("\u{E8B7}", &self.title_glyph_format, &slot, &brush);
                    }
                }
                draw_caption(
                    session,
                    &fitted,
                    fmt,
                    &Rect::from_xywh(caption_x, 0.0, shown_w, title_h),
                    &brush,
                    halo.as_ref(),
                );
                if let Some(badge) = tab.badge {
                    let r = Rect::from_xywh(
                        caption_x + shown_w + TAB_BADGE_GAP,
                        ((title_h - TAB_BADGE_H) * 0.5).round(),
                        badge_w - TAB_BADGE_GAP,
                        TAB_BADGE_H,
                    );
                    let fill = session.create_solid_brush(with_alpha(theme.accent, ta))?;
                    session.fill_rounded_rect(&RoundedRect::uniform(r, TAB_BADGE_H * 0.5), &fill);
                    // TextOnAccentFillColorPrimary: black on the light dark-mode accent,
                    // white on the deep light-mode one.
                    let on_accent = if theme.mode == crate::theme::ThemeMode::Dark {
                        ColorF::from_rgba8(0, 0, 0, 0xFF)
                    } else {
                        ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF)
                    };
                    let text = session.create_solid_brush(with_alpha(on_accent, ta))?;
                    session.draw_text(badge, &self.badge_format, &r, &text);
                }
            }
            if let Some(brush) = strip_fade {
                pop_strip_fade(session, brush);
            }
        } else {
            if deco.up_at_left() {
                // Up (Segoe Fluent E74A) in a 28×28 box: transparent at rest, SubtleFill
                // fading in on hover, SubtleFillColorTertiary + tertiary glyph while pressed.
                let box_r = TitleDeco::up_box(title_h);
                draw_up_box_fill(session, &backdrop, theme, scale, box_r, state, title_a)?;
                session.draw_text(
                    "\u{E74A}",
                    &self.glyph_format,
                    &box_r,
                    if state.up_pressed {
                        &tertiary
                    } else {
                        &secondary
                    },
                );
            }
            // The folder glyph travels with the title to the chosen alignment; a separate up
            // button stays at the far left.
            let (mut x, fitted, title_w) = self.lone_title_place(
                title,
                style.title_size,
                style.title_align,
                width,
                deco,
                title_reserve,
            );
            if deco.folder_icon {
                // Folder glyph (E8B7) aligned with the title baseline. Inside a subfolder it is
                // the up button (E74A), as in a folder tab.
                let slot = Rect::from_xywh(x - 2.0, 0.0, 20.0, title_h);
                if deco.up_button {
                    let box_r = glyph_up_square(x + 8.0, title_h);
                    draw_up_box_fill(session, &backdrop, theme, scale, box_r, state, title_a)?;
                    session.draw_text(
                        "\u{E74A}",
                        &self.title_glyph_format,
                        &slot,
                        if state.up_pressed {
                            &tertiary
                        } else {
                            &secondary
                        },
                    );
                } else {
                    session.draw_text("\u{E8B7}", &self.title_glyph_format, &slot, &secondary);
                }
                x += TitleDeco::ICON_W;
            }
            if let Some(halo) = &halo {
                halo.set_color(halo_color(
                    style.title_color.unwrap_or(foreground.text_primary),
                    title_a,
                ));
            }
            draw_caption(
                session,
                &fitted,
                title_format,
                &Rect::from_xywh(x, 0.0, title_w, title_h),
                &text,
                halo.as_ref(),
            );
        }
        if count_a > 0.0
            && count_w > 0.0
            && let Some(caption) = caption
        {
            let count_rect = Rect::from_xywh(
                width - TitleDeco::CHEVRON_W - TitleDeco::CHEVRON_RIGHT_MARGIN - count_w,
                0.0,
                count_w - 4.0,
                title_h,
            );
            let foreground =
                header_foreground(&backdrop, theme, 6, count_rect, scale, opacity, style.tint);
            let count =
                session.create_solid_brush(with_alpha(foreground.text_tertiary, count_a))?;
            if let Some(halo) = &halo {
                halo.set_color(halo_color(foreground.text_tertiary, count_a));
            }
            draw_caption(
                session,
                caption,
                &self.count_format,
                &count_rect,
                &count,
                halo.as_ref(),
            );
        }

        // Chevron (Segoe Fluent Icons ChevronUp E70E, turned 180° by the roll progress so it
        // points down once rolled — WinUI Expander rotates its chevron with the height) in a
        // 32×32 button box: revealed with the title-row hover (83 ms), always visible while
        // rolled so the state stays discoverable (tertiary at rest, secondary as the row is
        // hovered); per-control hover pill and pressed fill like a Fluent transparent icon
        // button.
        // `rolled_up` is the target state: while an expand is still rolling (`roll_t` > 0)
        // the chevron stays fully shown until the height has settled, so an expand started
        // with the pointer off the title row (context menu, keyboard) does not pop it to the
        // hover alpha on its first frame.
        if style.hide_count_and_chevron {
            return Ok(());
        }
        let chevron_a = chevron_shown * title_a;
        let box_r = TitleDeco::chevron_box(width, title_h);
        self.draw_chevron(
            session, &backdrop, theme, scale, opacity, style.tint, box_r, "\u{E70E}", chevron_a,
            state, roll_t,
        )
    }

    /// The roll chevron (`glyph`, turned 180° by the roll progress `roll_t`) in its 32×32
    /// button box at opacity `chevron_a`: per-control hover pill and pressed fill like a Fluent
    /// transparent icon button, tertiary at rest and secondary as its row is hovered.
    #[allow(clippy::too_many_arguments)]
    fn draw_chevron(
        &self,
        session: &DrawingSession<'_>,
        backdrop: &Backdrop<'_>,
        theme: &Theme,
        scale: f32,
        opacity: f32,
        tint: Option<ColorF>,
        box_r: Rect,
        glyph: &str,
        chevron_a: f32,
        state: TitleState,
        roll_t: f32,
    ) -> Result<()> {
        if chevron_a <= 0.0 {
            return Ok(());
        }
        let foreground = header_foreground(backdrop, theme, 7, box_r, scale, opacity, tint);
        if theme.liquid_glass && (state.chevron_pressed || state.chevron_hover > 0.0) {
            draw_header_glass(
                session,
                backdrop,
                theme,
                scale,
                4,
                box_r,
                if state.chevron_pressed { 0.85 } else { 0.45 },
                chevron_a
                    * if state.chevron_pressed {
                        1.0
                    } else {
                        state.chevron_hover
                    },
            )?;
        } else if state.chevron_pressed {
            let pill = session.create_solid_brush(with_alpha(theme.subtle_pressed, chevron_a))?;
            session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &pill);
        } else if state.chevron_hover > 0.0 {
            let pill = session.create_solid_brush(with_alpha(
                theme.subtle_hover,
                state.chevron_hover * chevron_a,
            ))?;
            session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &pill);
        }
        let color = if state.chevron_pressed {
            foreground.text_tertiary
        } else {
            lerp(
                foreground.text_tertiary,
                foreground.text_secondary,
                state.hover,
            )
        };
        let brush = session.create_solid_brush(with_alpha(color, chevron_a))?;
        let saved = session.transform();
        let centre = Vector2::new(
            box_r.left + box_r.width() / 2.0,
            box_r.top + box_r.height() / 2.0,
        );
        session.set_transform(&(Matrix3x2::rotation_around(chevron_angle(roll_t), centre) * saved));
        session.draw_text(glyph, &self.glyph_format, &box_r, &brush);
        session.set_transform(&saved);
        Ok(())
    }

    /// Draws a left tab column `rail_w` DIPs wide down the whole window (`height`): the fence's
    /// title bar when its tabs are on the left. Each tab is a 32 DIP row inset 4 DIPs from the
    /// window edge (WinUI NavigationView items), a hairline separates neighbours, the shown tab
    /// is semibold on the selection fill (`pill` while it glides to a newly selected tab), and
    /// a tab's own colour is a 6 DIP dot before its name. `expanded` (1 - roll progress) fades
    /// the line between the column and the items and the shown tab's selection fill, semibold
    /// caption and up button: a fence collapsed to the column shows its tabs alike.
    #[allow(clippy::too_many_arguments)]
    fn draw_tab_list(
        &self,
        session: &DrawingSession<'_>,
        backdrop: &Backdrop<'_>,
        theme: &Theme,
        scale: f32,
        opacity: f32,
        tint: Option<ColorF>,
        tabs: &[TabDraw<'_>],
        pill: Option<(f32, f32)>,
        halo: Option<&Brush>,
        rail_w: f32,
        height: f32,
        expanded: f32,
        deco: TitleDeco,
        state: TitleState,
    ) -> Result<()> {
        let px = Px::new(scale);
        let dark = theme.mode == crate::theme::ThemeMode::Dark;
        let line = |alpha: f32| {
            if dark {
                ColorF::new(1.0, 1.0, 1.0, alpha)
            } else {
                ColorF::new(0.0, 0.0, 0.0, alpha)
            }
        };
        // The divider stops short of the window's rounded corners.
        let expanded = expanded.clamp(0.0, 1.0);
        if expanded > 0.0 {
            let inset = theme.corner_radius.max(8.0);
            let brush = session.create_solid_brush(line(0.18 * expanded))?;
            session.fill_rect(
                &Rect::from_xywh(
                    px.snap(rail_w) - px.hair(),
                    inset,
                    px.hair(),
                    (height - inset * 2.0).max(0.0),
                ),
                &brush,
            );
        }
        let row = |y: f32, h: f32| {
            Rect::from_xywh(
                RAIL_ROW_INSET,
                y,
                (rail_w - RAIL_ROW_INSET - RAIL_ROW_RIGHT).max(0.0),
                h,
            )
        };
        let separator = session.create_solid_brush(line(if dark { 0.16 } else { 0.13 }))?;
        for pair in tabs.windows(2) {
            if pair[0].dragging || pair[1].dragging {
                continue;
            }
            let r = row(pair[0].x, pair[0].w);
            let y = px.snap((pair[0].x + pair[0].w + pair[1].x) * 0.5);
            let a = pair[0].alpha.min(pair[1].alpha).clamp(0.0, 1.0);
            if a > 0.0 {
                separator.set_color(line(if dark { 0.16 } else { 0.13 } * a));
                session.fill_rect(
                    &Rect::from_xywh(r.left + 10.0, y, (r.width() - 20.0).max(0.0), px.hair()),
                    &separator,
                );
            }
        }
        let selected = |rect: Rect, a: f32| -> Result<()> {
            if theme.liquid_glass {
                draw_header_glass(session, backdrop, theme, scale, 0, rect, 0.78, a)
            } else {
                let fill = session.create_solid_brush(with_alpha(theme.tab_active_fill, a))?;
                session.fill_rounded_rect(
                    &RoundedRect::uniform(rect, header_control_radius(theme, rect)),
                    &fill,
                );
                Ok(())
            }
        };
        if let Some((y, h)) = pill {
            selected(row(y, h), expanded)?;
        }
        let colored = tabs.iter().any(|t| t.color.is_some());
        let folders = tabs.iter().any(|t| t.folder_icon);
        let text_left = rail_text_left(colored, folders);
        // A dragged tab is always above its neighbours.
        for tab in tabs
            .iter()
            .filter(|t| !t.dragging)
            .chain(tabs.iter().filter(|t| t.dragging))
        {
            let ta = tab.alpha.clamp(0.0, 1.0);
            if ta <= 0.0 {
                continue;
            }
            let rect = row(tab.x, tab.w);
            let radius = header_control_radius(theme, rect);
            // How selected the row looks, and how much of that its own fill shows (none while
            // the pill glides). Press and hover feedback take the rest: on every row of a
            // collapsed column, the shown tab's included.
            let sel = rail_selected(tab.active, tab.dragging, expanded);
            let own = if pill.is_none() { sel } else { 0.0 };
            if own > 0.0 {
                selected(rect, ta * own)?;
            }
            if tab.pressed && own < 1.0 {
                let a = ta * (1.0 - own);
                let fill = session.create_solid_brush(with_alpha(theme.subtle_pressed, a))?;
                session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
            } else if tab.hover > 0.0 && sel < 1.0 {
                let a = tab.hover * ta * (1.0 - sel);
                let fill = session.create_solid_brush(with_alpha(theme.subtle_hover, a))?;
                session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
            }
            if tab.drop_target > 0.0 {
                let a = tab.drop_target.min(1.0) * ta;
                let fill = session.create_solid_brush(with_alpha(theme.selection_fill, a))?;
                session.fill_rounded_rect(&RoundedRect::uniform(rect, radius), &fill);
                let ring = session.create_solid_brush(with_alpha(theme.selection_stroke, a))?;
                session.draw_rounded_rect(&px.ring(rect, radius), &ring, px.hair());
            }
            if let Some(color) = tab.color {
                let dot = Rect::from_xywh(
                    rect.left + 10.0,
                    rect.top + rect.height() * 0.5 - 3.0,
                    6.0,
                    6.0,
                );
                let brush = session.create_solid_brush(with_alpha(color, ta))?;
                session.fill_rounded_rect(&RoundedRect::uniform(dot, 3.0), &brush);
            }
            let foreground =
                header_foreground(backdrop, theme, tab.key, rect, scale, opacity, tint);
            let color = if tab.pressed && !tab.dragging {
                foreground.text_secondary
            } else {
                tab.title_color.unwrap_or(foreground.text_primary)
            };
            // A navigated portal's up button ends the shown tab's row (its room stays while the
            // fence collapses, so the caption does not re-fit at the end).
            let up = tab.active && deco.up_button;
            let room = self.rail_title_room(rail_w, colored, folders, tab.badge, up);
            let caption = Rect::from_xywh(rect.left + text_left, rect.top, room, rect.height());
            if tab.folder_icon {
                let brush = session.create_solid_brush(with_alpha(color, ta))?;
                session.draw_text(
                    "\u{E8B7}",
                    &self.title_glyph_format,
                    &Rect::from_xywh(
                        caption.left - TAB_ICON_W - 2.0,
                        rect.top,
                        TAB_ICON_W,
                        rect.height(),
                    ),
                    &brush,
                );
            }
            // The shown tab's semibold caption cross-fades to the others' regular weight as the
            // fence collapses to the column.
            for (strong, a) in [(true, sel), (false, 1.0 - sel)] {
                let a = a * ta;
                if a <= 0.0 {
                    continue;
                }
                let brush = session.create_solid_brush(with_alpha(color, a))?;
                if let Some(halo) = halo {
                    halo.set_color(halo_color(color, a));
                }
                let fmt = &self.rail_title_format(tab.title_size, strong);
                let fitted = crate::text::fit_width(tab.title, fmt, room);
                draw_caption(session, &fitted, fmt, &caption, &brush, halo);
            }
            // Like the selection fill, the up button is gone from a collapsed column.
            let up_a = ta * expanded;
            if up && up_a > 0.0 {
                // Up (Segoe Fluent E74A): SubtleFill on hover, SubtleFillColorTertiary while
                // pressed, as in the title row.
                let box_r = TitleDeco::rail_up_box(rect);
                let fill = if state.up_pressed {
                    Some(with_alpha(theme.subtle_pressed, up_a))
                } else if state.up_hover > 0.0 {
                    Some(with_alpha(theme.subtle_hover, state.up_hover * up_a))
                } else {
                    None
                };
                if let Some(fill) = fill {
                    let brush = session.create_solid_brush(fill)?;
                    session.fill_rounded_rect(&RoundedRect::uniform(box_r, 4.0), &brush);
                }
                let glyph = session.create_solid_brush(with_alpha(
                    if state.up_pressed {
                        foreground.text_tertiary
                    } else {
                        foreground.text_secondary
                    },
                    up_a,
                ))?;
                session.draw_text("\u{E74A}", &self.glyph_format, &box_r, &glyph);
            }
            if let Some(badge) = tab.badge {
                let badge_w = self.tab_badge_width(badge) - TAB_BADGE_GAP;
                let up_w = if up { 26.0 } else { 0.0 };
                let r = Rect::from_xywh(
                    rect.right - 8.0 - badge_w - up_w,
                    rect.top + ((rect.height() - TAB_BADGE_H) * 0.5).round(),
                    badge_w,
                    TAB_BADGE_H,
                );
                let fill = session.create_solid_brush(with_alpha(theme.accent, ta))?;
                session.fill_rounded_rect(&RoundedRect::uniform(r, TAB_BADGE_H * 0.5), &fill);
                // TextOnAccentFillColorPrimary, as on the title-row pills.
                let on_accent = if dark {
                    ColorF::from_rgba8(0, 0, 0, 0xFF)
                } else {
                    ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF)
                };
                let text = session.create_solid_brush(with_alpha(on_accent, ta))?;
                session.draw_text(badge, &self.badge_format, &r, &text);
            }
        }
        Ok(())
    }

    /// Draws the List / Details rows into a content surface of `width` x `height` DIPs. The
    /// header (if any) is fixed at the top; `rows` are already offset by the scroll position
    /// and positioned below the header.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rows(
        &self,
        session: &DrawingSession<'_>,
        clip: Option<&Clip<'_>>,
        theme: &Theme,
        scale: f32,
        bitmaps: &mut BitmapCache,
        width: f32,
        height: f32,
        rows: &RowsDraw<'_>,
        backdrop: Option<(&Backdrop<'_>, f32, Option<ColorF>)>,
    ) -> Result<()> {
        session.clear(ColorF::TRANSPARENT);
        let px = Px::new(scale);
        let state = StateBrushes::new(session, theme)?;
        draw_drop_wash(session, theme, &state, width, height, rows.drop_highlight);
        let selected = session.create_solid_brush(theme.selection_fill)?;
        let selected_stroke = session.create_solid_brush(theme.selection_stroke)?;
        let secondary = session.create_solid_brush(theme.text_secondary)?;
        let cols = rows.columns;
        let header_h = rows.header_h;
        let halo = if theme.liquid_glass || theme.text_halo {
            Some(session.create_solid_brush(ColorF::TRANSPARENT)?)
        } else {
            None
        };
        let spread = 1.0 / scale.max(1.0);
        let paint_text = |text: &str,
                          format: &TextFormat,
                          rect: Rect,
                          key: u128,
                          muted: bool,
                          alpha: f32,
                          right: bool| {
            // Only the glass backdrop samples under the text; measuring builds a layout.
            let foreground = backdrop.map_or(*theme, |(background, opacity, tint)| {
                let measured = crate::text::measure_width(text, format)
                    .min(rect.width())
                    .max(1.0);
                let area = Rect::from_xywh(
                    rows.origin_x
                        + if right {
                            rect.right - measured
                        } else {
                            rect.left
                        },
                    rect.top + rows.origin_y,
                    measured,
                    rect.height(),
                );
                header_foreground(background, theme, key, area, scale, opacity, tint)
            });
            let color = if muted {
                foreground.text_secondary
            } else {
                foreground.text_primary
            };
            let brush = state.a(with_alpha(color, alpha));
            if let Some(halo) = &halo {
                halo.set_color(if color.r > 0.5 {
                    ColorF::new(0.0, 0.0, 0.0, 0.28 * alpha)
                } else {
                    ColorF::new(1.0, 1.0, 1.0, 0.28 * alpha)
                });
            }
            draw_legible_text(session, text, format, &rect, brush, halo.as_ref(), spread);
        };
        // Dates, types, sizes and the column captions are not shortened like the names: each
        // is clipped to its column so a long one does not run into the next.
        let paint_clipped = |text: &str,
                             format: &TextFormat,
                             rect: Rect,
                             key: u128,
                             alpha: f32,
                             right: bool|
         -> Result<()> {
            let paint = || {
                paint_text(text, format, rect, key, true, alpha, right);
                Ok(())
            };
            match clip {
                Some(clip) => clip.scoped(&rect, paint),
                None => paint(),
            }
        };

        let mut paint_rows = || -> Result<()> {
            // Section headers scroll with the rows, under the fixed column header. The caption
            // starts where the 名称 caption does; the hairline runs from its end to the margin.
            if !rows.group_headers.is_empty() {
                let line = session.create_solid_brush(theme.stroke)?;
                let text_x = cols.name_x + 4.0;
                let right = width - 8.0;
                for (i, h) in rows.group_headers.iter().enumerate() {
                    if h.y + h.h <= header_h || h.y > height {
                        continue;
                    }
                    let text_w = crate::text::measure_width(h.text, &self.group_format)
                        .min((right - text_x).max(1.0))
                        .max(1.0);
                    paint_text(
                        h.text,
                        &self.group_format,
                        Rect::from_xywh(text_x, h.y, text_w, h.h),
                        0x30000 + i as u128,
                        true,
                        1.0,
                        false,
                    );
                    draw_group_rule(session, px, h, text_x + text_w, right, &line);
                }
            }
            let mut visible_slot = 0u128;
            for row in rows.rows {
                if row.y + row.h <= header_h || row.y > height {
                    continue;
                }
                let alpha = row.alpha.clamp(0.0, 1.0);
                if alpha <= 0.0 {
                    continue;
                }
                let rect = Rect::from_xywh(4.0, row.y + 1.0, (width - 8.0).max(0.0), row.h - 2.0);
                if alpha >= 1.0 {
                    draw_item_state(
                        session,
                        theme,
                        &state,
                        px,
                        rect,
                        row.hover,
                        row.selection,
                        rows.selection_inactive,
                        row.pressed,
                        row.focused,
                        row.drop_target,
                    );
                }
                let icon_x = cols.name_x;
                let icon_y = row.y + (row.h - rows.icon_size) / 2.0;
                draw_item_icon(
                    session,
                    theme,
                    &state,
                    bitmaps,
                    row.icon_key,
                    row.icon,
                    row.failed,
                    row.dimmed,
                    alpha,
                    row.icon_alpha,
                    icon_x,
                    icon_y,
                    rows.icon_size,
                    3.0,
                )?;
                let text_x = icon_x + rows.icon_size + 8.0;
                let name_w = (cols.name_x + cols.name_w - text_x).max(0.0);
                // Contrast follows each visible text cell, not the average of the whole
                // fence. Stable viewport slots bound the hysteresis cache while scrolling.
                let key = 0x10000 + visible_slot * 8;
                paint_text(
                    row.name,
                    &self.row_format,
                    Rect::from_xywh(text_x, row.y, name_w, row.h),
                    key,
                    row.dimmed,
                    alpha,
                    false,
                );
                if let Some((x, w)) = cols.date {
                    paint_clipped(
                        row.date,
                        &self.row_format,
                        Rect::from_xywh(x, row.y, w, row.h),
                        key + 1,
                        alpha,
                        false,
                    )?;
                }
                if let Some((x, w)) = cols.type_ {
                    paint_clipped(
                        row.type_name,
                        &self.row_format,
                        Rect::from_xywh(x, row.y, w, row.h),
                        key + 2,
                        alpha,
                        false,
                    )?;
                }
                if let Some((x, w)) = cols.size {
                    paint_clipped(
                        row.size,
                        &self.row_format_right,
                        Rect::from_xywh(x, row.y, w, row.h),
                        key + 3,
                        alpha,
                        true,
                    )?;
                }
                visible_slot += 1;
            }
            Ok(())
        };
        let bottom = rows.rows_bottom.min(height);
        match clip {
            Some(clip) if header_h > 0.0 || bottom < height => {
                // A row scrolled partly under the fixed header must not paint through its
                // captions. The header has no opaque fill (the surface is transparent over the
                // backdrop), so clip the rows to the band below it instead (and above the
                // empty height under the whole rows).
                clip.scoped(
                    &Rect::from_xywh(0.0, header_h, width, (bottom - header_h).max(0.0)),
                    paint_rows,
                )?;
            }
            _ => paint_rows()?,
        }

        // Fixed header over the rows (Details only): column captions (all secondary, like
        // Explorer's header), a full-width hairline underneath and the sort chevron centred at
        // the top of the sorted column.
        if header_h > 0.0 {
            let line = session.create_solid_brush(theme.stroke)?;
            let hair = px.hair();
            // Column dividers (grab handles for resizing) in the gutter left of each column.
            for col in [cols.date, cols.type_, cols.size].into_iter().flatten() {
                let (y0, y1) = (px.snap(6.0), px.snap(header_h - 6.0));
                session.fill_rect(
                    &Rect::from_xywh(px.snap(col.0 - 4.5), y0, hair, (y1 - y0).max(0.0)),
                    &line,
                );
            }
            session.fill_rect(
                &Rect::from_xywh(0.0, px.snap(header_h) - hair, width, hair),
                &line,
            );
            let mut captions: Vec<(f32, f32, &str, HeaderColumn, bool)> = Vec::new();
            captions.push((
                cols.name_x + 4.0,
                cols.name_w - 4.0,
                pecofence_core::i18n::text("名称"),
                HeaderColumn::Name,
                false,
            ));
            if let Some((x, w)) = cols.date {
                captions.push((
                    x,
                    w,
                    pecofence_core::i18n::text("修改日期"),
                    HeaderColumn::Date,
                    false,
                ));
            }
            if let Some((x, w)) = cols.type_ {
                captions.push((
                    x,
                    w,
                    pecofence_core::i18n::text("类型"),
                    HeaderColumn::Type,
                    false,
                ));
            }
            if let Some((x, w)) = cols.size {
                captions.push((
                    x,
                    w,
                    pecofence_core::i18n::text("大小"),
                    HeaderColumn::Size,
                    true,
                ));
            }
            for (index, (x, w, text, col, right)) in captions.into_iter().enumerate() {
                let cell_rect = Rect::from_xywh(x - 4.0, 2.0, w + 8.0, header_h - 6.0);
                let cell = RoundedRect::uniform(cell_rect, header_control_radius(theme, cell_rect));
                let hover = rows
                    .header_hover
                    .iter()
                    .find(|(c, _)| *c == col)
                    .map_or(0.0, |(_, a)| *a);
                if rows.header_pressed == Some(col) {
                    let b = session.create_solid_brush(theme.subtle_pressed)?;
                    session.fill_rounded_rect(&cell, &b);
                } else if hover > 0.0 {
                    let b = session.create_solid_brush(with_alpha(theme.subtle_hover, hover))?;
                    session.fill_rounded_rect(&cell, &b);
                }
                let fmt = if right {
                    &self.row_format_right
                } else {
                    &self.row_format
                };
                paint_clipped(
                    text,
                    fmt,
                    Rect::from_xywh(x, 0.0, w, header_h - 1.0),
                    0x20000 + index as u128,
                    1.0,
                    right,
                )?;
                if rows.sort_column == Some(col) {
                    // 8 px chevron flush with the header top, horizontally centred in the cell
                    // (the Win32 header control's sort arrow), same grey as the captions.
                    let glyph = if rows.sort_descending {
                        "\u{E70D}"
                    } else {
                        "\u{E70E}"
                    };
                    paint_text(
                        glyph,
                        &self.sort_glyph_format,
                        Rect::from_xywh(x + (w - 12.0) / 2.0, 0.0, 12.0, 10.0),
                        0x20010 + index as u128,
                        true,
                        1.0,
                        false,
                    );
                }
            }
        }

        if let Some(m) = rows.marquee
            && m.right > m.left
            && m.bottom > m.top
        {
            let m = Rect {
                left: m.left.round(),
                top: m.top.round().max(header_h),
                right: m.right.round(),
                bottom: m.bottom.round(),
            };
            if m.bottom > m.top {
                session.fill_rounded_rect(&RoundedRect::uniform(m, 4.0), &selected);
                session.draw_rounded_rect(&px.ring(m, 4.0), &selected_stroke, px.hair());
            }
        }

        if let Some(r) = rows.insert_caret {
            draw_insert_caret(session, theme, &state, r, header_h, rows.insert_caret_alpha);
        }

        if let Some(bar) = rows.scrollbar {
            draw_scrollbar(session, theme, &self.sort_glyph_format, width, bar)?;
        }

        if let Some(text) = rows.empty_text
            && rows.rows.is_empty()
        {
            let avail = (height - header_h).max(0.0);
            let y0 = empty_text_top(avail);
            session.draw_text(
                text,
                &self.empty_format,
                &Rect::from_xywh(
                    12.0,
                    header_h + y0,
                    (width - 24.0).max(0.0),
                    (avail - y0).max(0.0),
                ),
                &secondary,
            );
        }
        Ok(())
    }

    /// Draws the item grid into a content surface of `width` x `height` DIPs (origin 0,0).
    /// `clip` cuts the items off at `content.items_bottom`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_content(
        &self,
        session: &DrawingSession<'_>,
        clip: Option<&Clip<'_>>,
        theme: &Theme,
        scale: f32,
        bitmaps: &mut BitmapCache,
        width: f32,
        height: f32,
        content: &ContentDraw<'_>,
    ) -> Result<()> {
        session.clear(ColorF::TRANSPARENT);
        let px = Px::new(scale);
        let state = StateBrushes::new(session, theme)?;
        draw_drop_wash(
            session,
            theme,
            &state,
            width,
            height,
            content.drop_highlight,
        );

        let selected = session.create_solid_brush(theme.selection_fill)?;
        let selected_stroke = session.create_solid_brush(theme.selection_stroke)?;
        let label = session.create_solid_brush(theme.text_primary)?;
        let label_dim = session.create_solid_brush(theme.text_secondary)?;

        let mut paint_items = || -> Result<()> {
            // "按时间分组" section headers: secondary caption flush with the label inset, then an
            // Explorer-style hairline to the right margin. Static bands (the items glide), painted
            // first so an unfolded label may overdraw them like it overdraws the row below.
            if !content.group_headers.is_empty() {
                let line = session.create_solid_brush(theme.stroke)?;
                // The same local halo the labels get over mixed wallpaper.
                let halo = if theme.liquid_glass || theme.text_halo {
                    Some(session.create_solid_brush(if theme.text_secondary.r > 0.5 {
                        ColorF::new(0.0, 0.0, 0.0, 0.28)
                    } else {
                        ColorF::new(1.0, 1.0, 1.0, 0.28)
                    })?)
                } else {
                    None
                };
                let spread = 1.0 / scale.max(1.0);
                let text_x = ICON_LABEL_SIDE_INSET;
                let right = width - 8.0;
                for h in content.group_headers {
                    if h.y + h.h <= 0.0 || h.y > height {
                        continue;
                    }
                    let text_w = crate::text::measure_width(h.text, &self.group_format)
                        .min((right - text_x).max(1.0))
                        .max(1.0);
                    let rect = Rect::from_xywh(text_x, h.y, text_w, h.h);
                    draw_legible_text(
                        session,
                        h.text,
                        &self.group_format,
                        &rect,
                        &label_dim,
                        halo.as_ref(),
                        spread,
                    );
                    draw_group_rule(session, px, h, text_x + text_w, right, &line);
                }
            }

            // The cell whose label unfolds is painted after every other cell: like the desktop's
            // focused icon it overdraws the row below.
            let mut deferred: Option<&ItemCell<'_>> = None;
            for cell in content.items {
                if cell.y + cell.h < 0.0 || cell.y > height {
                    continue;
                }
                if cell.full_label.is_some() && deferred.is_none() {
                    deferred = Some(cell);
                    continue;
                }
                self.draw_cell(
                    session, theme, bitmaps, &state, px, content, cell, &label, &label_dim, None,
                )?;
            }
            if let Some(cell) = deferred {
                // The caller measured the unfolded text when it fitted it (`fit_lines_counted`):
                // no second layout per frame here.
                let unfold = cell.full_label.map(|(text, lines)| (text, lines.max(1)));
                self.draw_cell(
                    session, theme, bitmaps, &state, px, content, cell, &label, &label_dim, unfold,
                )?;
            }
            Ok(())
        };
        match clip {
            Some(clip) if content.items_bottom < height => clip.scoped(
                &Rect::from_xywh(0.0, 0.0, width, content.items_bottom.max(0.0)),
                paint_items,
            )?,
            _ => paint_items()?,
        }

        if let Some(m) = content.marquee
            && m.right > m.left
            && m.bottom > m.top
        {
            // Plan §6.2: marqueeFill with a 4 px corner radius plus the selection stroke,
            // snapped to whole DIPs so the 1 px stroke stays crisp.
            let m = Rect {
                left: m.left.round(),
                top: m.top.round(),
                right: m.right.round(),
                bottom: m.bottom.round(),
            };
            session.fill_rounded_rect(&RoundedRect::uniform(m, 4.0), &selected);
            session.draw_rounded_rect(&px.ring(m, 4.0), &selected_stroke, px.hair());
        }

        if let Some(r) = content.insert_caret {
            draw_insert_caret(
                session,
                theme,
                &state,
                r,
                f32::MIN,
                content.insert_caret_alpha,
            );
        }

        if let Some(bar) = content.scrollbar {
            draw_scrollbar(session, theme, &self.sort_glyph_format, width, bar)?;
        }

        if let Some(text) = content.empty_text
            && content.items.is_empty()
        {
            let secondary = session.create_solid_brush(theme.text_secondary)?;
            let y0 = empty_text_top(height);
            session.draw_text(
                text,
                &self.empty_format,
                &Rect::from_xywh(12.0, y0, (width - 24.0).max(0.0), (height - y0).max(0.0)),
                &secondary,
            );
        }
        Ok(())
    }
}

impl FenceChrome {
    /// One icon cell: state layers, icon box and label. `unfold` = (full name, line count):
    /// the label grows to that many lines and the state pill grows with it (Windows desktop
    /// focused-icon behaviour); the content surface clips whatever runs past the bottom.
    #[allow(clippy::too_many_arguments)]
    fn draw_cell(
        &self,
        session: &DrawingSession<'_>,
        theme: &Theme,
        bitmaps: &mut BitmapCache,
        state: &StateBrushes,
        px: Px,
        content: &ContentDraw<'_>,
        cell: &ItemCell<'_>,
        label: &Brush,
        label_dim: &Brush,
        unfold: Option<(&str, u32)>,
    ) -> Result<()> {
        let alpha = cell.alpha.clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return Ok(());
        }
        let base_lines = content.label_lines as f32;
        let lines = unfold.map_or(base_lines, |(_, n)| (n as f32).max(base_lines));
        let extra = (lines - base_lines) * content.line_h;
        let rect = Rect::from_xywh(
            cell.x + 2.0,
            cell.y + 2.0,
            cell.w - 4.0,
            cell.h - 4.0 + extra,
        );
        if alpha >= 1.0 {
            draw_item_state(
                session,
                theme,
                state,
                px,
                rect,
                cell.hover,
                cell.selection,
                content.selection_inactive,
                cell.pressed,
                cell.focused,
                cell.drop_target,
            );
        }
        let icon_x = cell.x + (cell.w - content.icon_size) / 2.0;
        let icon_y = cell.y + content.icon_top;
        draw_item_icon(
            session,
            theme,
            state,
            bitmaps,
            cell.icon_key,
            cell.icon,
            cell.failed,
            cell.dimmed,
            alpha,
            cell.icon_alpha,
            icon_x,
            icon_y,
            content.icon_size,
            4.0,
        )?;
        let label_rect = Rect::from_xywh(
            cell.x + ICON_LABEL_SIDE_INSET,
            icon_y + content.icon_size + content.label_gap,
            (cell.w - ICON_LABEL_SIDE_INSET * 2.0).max(1.0),
            content.line_h * lines,
        );
        let text = unfold.map_or(cell.label, |(t, _)| t);
        let resting = if cell.dimmed { label_dim } else { label };
        // A fading cell recolours the scratch brush instead of allocating one per frame.
        let brush = if alpha < 1.0 {
            let c = if cell.dimmed {
                theme.text_secondary
            } else {
                theme.text_primary
            };
            state.a(with_alpha(c, alpha))
        } else {
            resting
        };
        // A local halo protects labels over mixed light/dark wallpaper (Liquid Glass, a fully
        // transparent plate). It does not frost or darken the glass between icons.
        let halo = (theme.liquid_glass || theme.text_halo)
            .then(|| state.halo(halo_color(theme.text_primary, alpha)));
        draw_caption(
            session,
            text,
            &self.label_font.borrow().format,
            &label_rect,
            brush,
            halo,
        );

        Ok(())
    }
}

/// Top of the empty-state text inside an area `avail` DIPs tall: Explorer places "This folder
/// is empty." in the upper part of the view; very short fences keep it vertically centred
/// (20 DIP = the Body 14 line height).
fn empty_text_top(avail: f32) -> f32 {
    if avail < 96.0 {
        ((avail - 20.0) * 0.5).max(0.0)
    } else {
        avail * 0.24
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_alignment_centres_on_the_fence_and_keeps_clear_of_the_chevron() {
        use pecofence_core::TitleAlign::{Center, Left, Right};
        // A 300 DIP row whose title may use 16..260 (the chevron zone follows).
        assert_eq!(aligned_x(Left, 300.0, 16.0, 260.0, 80.0), 16.0);
        assert_eq!(aligned_x(Center, 300.0, 16.0, 260.0, 80.0), 110.0);
        assert_eq!(aligned_x(Right, 300.0, 16.0, 260.0, 80.0), 180.0);
        // Centring a long title would reach into the chevron zone: it moves left instead.
        assert_eq!(aligned_x(Center, 300.0, 16.0, 260.0, 230.0), 30.0);
        // Wider than the room (the caller ellipsizes): every alignment starts at the left.
        for align in [Left, Center, Right] {
            assert_eq!(aligned_x(align, 300.0, 16.0, 260.0, 400.0), 16.0);
        }
    }

    #[test]
    fn stacked_and_standalone_titles_have_identical_metrics() {
        let chrome = FenceChrome::new().unwrap();
        let mut widths = Vec::new();
        for size in 0..=2 {
            for caption in ["文件与文档", "Folder pictures", "资料 Archives 2026"] {
                let single = TextLayout::new(caption, &chrome.title_format(size), 1000.0, 100.0)
                    .unwrap()
                    .metrics();
                let tab = TextLayout::new(caption, &chrome.tab_format(size), 1000.0, 100.0)
                    .unwrap()
                    .metrics();
                assert!((single.width - tab.width).abs() < 0.001);
                assert!((single.height - tab.height).abs() < 0.001);
                for width in [0.0, 2.0, 8.0, 20.0, 80.0] {
                    let text = crate::text::fit_width(caption, &chrome.tab_format(size), width);
                    assert!(
                        crate::text::measure_width(&text, &chrome.tab_format(size))
                            <= width + 0.001
                    );
                }
            }
            widths.push(crate::text::measure_width(
                "文件与文档",
                &chrome.tab_format(size),
            ));
        }
        assert!(widths[0] < widths[1] && widths[1] < widths[2]);
    }

    #[test]
    fn glass_header_controls_follow_the_plate_and_leave_room_for_captions() {
        let theme = Theme::light().with_liquid_glass();
        // 16 DIP plate minus the 4 DIP inset; a 2 DIP row would clamp to its own half height.
        for width in [40.0, 80.0, 160.0] {
            let rect = Rect::from_xywh(8.0, 4.0, width, 28.0);
            assert_eq!(header_control_radius(&theme, rect), 12.0);
        }
        let chevron = TitleDeco::chevron_box(320.0, 36.0);
        assert_eq!(
            header_control_radius(&theme, chevron),
            12.0f32.min(chevron.width().min(chevron.height()) * 0.5)
        );
        assert_eq!(
            header_control_radius(&theme, Rect::from_xywh(0.0, 0.0, 40.0, 2.0)),
            1.0
        );
        assert_eq!(header_count_width(136.0, 1, 16.0), 0.0);
        assert_eq!(header_count_width(136.0, 2, 16.0), 0.0);
        assert_eq!(header_count_width(400.0, 2, 16.0), ROLLED_COUNT_W);
        // Six tabs scroll beside the count instead of shrinking under it.
        assert_eq!(header_count_width(320.0, 6, 16.0), ROLLED_COUNT_W);
        assert_eq!(header_count_width(200.0, 6, 16.0), 0.0);
    }

    #[test]
    fn squeezed_tabs_trade_padding_for_their_captions() {
        let chrome = FenceChrome::new().unwrap();
        for theme in [Theme::dark().with_liquid_glass(), Theme::dark()] {
            let full = tab_text_padding(&theme);
            let caption = crate::text::measure_width("游戏", &chrome.tab_format(1));
            // Room to spare: the theme's padding.
            assert_eq!(tab_text_inset(&theme, caption + full + 10.0, caption), full);
            // A crowded strip (a 172 DIP fence with three tabs gives ~37 DIP pills): the
            // padding shrinks so the two characters still fit.
            let squeezed = caption + TAB_TEXT_MIN_PADDING + 1.0;
            assert!(tab_text_inset(&theme, squeezed, caption) < full);
            assert!(chrome.tab_title_fits("游戏", squeezed, 1, false, &theme));
            // Below the minimum padding the caption is elided, never drawn over the edge.
            assert_eq!(
                tab_text_inset(&theme, caption, caption),
                TAB_TEXT_MIN_PADDING
            );
            assert!(!chrome.tab_title_fits("游戏", caption, 1, false, &theme));
            // A folder tab's glyph takes its room from the caption.
            assert!(chrome.tab_title_fits("游戏", squeezed + TAB_ICON_W, 1, true, &theme));
            assert!(!chrome.tab_title_fits("游戏", squeezed, 1, true, &theme));
        }
    }

    #[test]
    fn a_folder_tabs_up_button_is_the_square_around_its_glyph() {
        let chrome = FenceChrome::new().unwrap();
        for theme in [Theme::dark().with_liquid_glass(), Theme::dark()] {
            let caption = crate::text::measure_width("项目", &chrome.tab_format(1));
            let w = caption + TAB_ICON_W + tab_text_padding(&theme);
            let r = chrome.tab_up_box(&theme, 100.0, w, "项目", 1, None);
            // The glyph starts the centred group (glyph, then caption) and keeps its slot.
            let glyph_mid = 100.0 + (w - TAB_ICON_W - caption) * 0.5 - 2.0 + TAB_ICON_W * 0.5;
            assert_eq!((r.width(), r.height()), (20.0, 20.0));
            assert!((r.left + 10.0 - glyph_mid).abs() < 0.01);
            assert_eq!(r.top + 10.0, theme.title_height * 0.5);
            assert!(r.left >= 100.0 && r.right <= 100.0 + w);
        }
    }

    #[test]
    fn a_lone_folder_titles_up_button_is_its_glyph() {
        use pecofence_core::TitleAlign::{Left, Right};
        let chrome = FenceChrome::new().unwrap();
        // The glyph keeps its place before the title; no separate button pushes the title.
        let up = TitleDeco {
            folder_icon: true,
            up_button: true,
        };
        assert_eq!(up.title_x(), 16.0 + TitleDeco::ICON_W);
        let left = chrome.lone_up_box("2026", 1, Left, 300.0, 0.0, 36.0);
        assert_eq!(
            (left.left, left.top, left.right, left.bottom),
            (14.0, 8.0, 34.0, 28.0)
        );
        // It moves with the title, which keeps clear of the rolled count.
        let right = chrome.lone_up_box("2026", 1, Right, 300.0, ROLLED_COUNT_W, 36.0);
        assert!(right.left > left.left);
        assert!(right.right <= 300.0 - TITLE_RIGHT_RESERVE - ROLLED_COUNT_W);
        // With the glyph hidden, the button stays at the far left.
        let hidden = TitleDeco {
            folder_icon: false,
            up_button: true,
        };
        assert!(hidden.up_at_left());
        assert_eq!(hidden.title_x(), 16.0 + TitleDeco::UP_W);
    }

    #[test]
    fn left_column_names_stay_aligned_after_a_dot_and_a_folder_glyph() {
        assert_eq!(rail_text_left(false, false), 10.0);
        assert_eq!(rail_text_left(true, false), 24.0);
        assert_eq!(rail_text_left(false, true), 10.0 + TAB_ICON_W);
        assert_eq!(rail_text_left(true, true), 24.0 + TAB_ICON_W);
        let chrome = FenceChrome::new().unwrap();
        assert_eq!(
            chrome.rail_title_room(160.0, true, false, None, false)
                - chrome.rail_title_room(160.0, true, true, None, false),
            TAB_ICON_W
        );
    }

    #[test]
    fn chevron_turns_half_a_circle_with_the_roll() {
        assert_eq!(chevron_angle(0.0), 0.0);
        assert_eq!(chevron_angle(0.5), 90.0);
        assert_eq!(chevron_angle(1.0), 180.0);
        // Overshooting curves stay inside the half turn.
        assert_eq!(chevron_angle(1.2), 180.0);
        assert_eq!(chevron_angle(-0.1), 0.0);
    }

    /// The column's chevron is centred in the room under the tabs; while the window is shorter
    /// than that (mid collapse or expand) it stays under the last tab, hidden by the bottom edge.
    #[test]
    fn rail_chevron_stays_under_the_last_tab() {
        let r = TitleDeco::rail_chevron_box(100.0, 200.0, 76.0);
        assert_eq!((r.left, r.top, r.width()), (34.0, 162.0, 32.0));
        assert_eq!(TitleDeco::rail_chevron_box(100.0, 84.0, 76.0).top, 82.0);
    }

    /// Collapsed to its column, a fence marks no tab as the shown one; a dragged tab keeps its
    /// fill throughout.
    #[test]
    fn collapsed_column_shows_its_tabs_alike() {
        assert_eq!(rail_selected(true, false, 1.0), 1.0);
        assert_eq!(rail_selected(true, false, 0.25), 0.25);
        assert_eq!(rail_selected(true, false, 0.0), 0.0);
        assert_eq!(rail_selected(false, false, 1.0), 0.0);
        assert_eq!(rail_selected(false, true, 0.0), 1.0);
        assert_eq!(rail_selected(true, false, 1.3), 1.0);
    }

    /// A left tab column shows its chevron on hover while expanded and none once collapsed to
    /// the column; in between it fades with the roll, never popping at either end.
    #[test]
    fn rail_chevron_fades_with_the_collapse() {
        assert_eq!(rail_chevron_alpha(1.0, 0.0), 1.0);
        assert_eq!(rail_chevron_alpha(0.0, 0.0), 0.0);
        assert_eq!(rail_chevron_alpha(1.0, 1.0), 0.0);
        assert_eq!(rail_chevron_alpha(1.0, 0.25), 0.75);
        assert_eq!(rail_chevron_alpha(0.5, 0.5), 0.25);
        // Overshooting curves stay inside 0..=1.
        assert_eq!(rail_chevron_alpha(1.0, 1.2), 0.0);
        assert_eq!(rail_chevron_alpha(1.0, -0.1), 1.0);
    }

    /// The thumb keeps its centre while it widens, and that centre is the 12 DIP track's
    /// (6 DIP from the right edge, where the track fill and arrow buttons are centred too).
    #[test]
    fn scrollbar_thumb_widens_about_its_centre() {
        let w = 300.0;
        let rest = ScrollbarDraw::thumb_x(w, ScrollbarDraw::REST_WIDTH);
        let wide = ScrollbarDraw::thumb_x(w, ScrollbarDraw::EXPANDED_WIDTH);
        assert_eq!(rest, w - 7.0);
        assert_eq!(wide, w - 9.0);
        assert_eq!(
            rest + ScrollbarDraw::REST_WIDTH / 2.0,
            w - ScrollbarDraw::BAR_W / 2.0
        );
        let centre = |x: f32, width: f32| x + width / 2.0;
        assert_eq!(
            centre(rest, ScrollbarDraw::REST_WIDTH),
            centre(wide, ScrollbarDraw::EXPANDED_WIDTH)
        );
        // Intermediate widths stay between the two end positions.
        let mid = ScrollbarDraw::thumb_x(w, 4.0);
        assert!(mid < rest && mid > wide);
    }

    #[test]
    fn hairlines_are_whole_device_pixels_at_every_dpi() {
        for scale in [1.0f32, 1.25, 1.5, 1.75, 2.0, 2.5] {
            let px = Px::new(scale);
            let hair_px = px.hair() * scale;
            assert!(
                (hair_px - hair_px.round()).abs() < 1e-4,
                "{scale}: {hair_px}"
            );
            assert!(hair_px >= 1.0);
            // A snapped coordinate lands on a pixel boundary.
            let v = px.snap(7.3) * scale;
            assert!((v - v.round()).abs() < 1e-4, "{scale}");
            // The stroke rect is centred on whole pixels: edge ± hair/2 is on the grid.
            let r = px.stroke_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0));
            let edge = (r.left - px.hair() / 2.0) * scale;
            assert!((edge - edge.round()).abs() < 1e-4, "{scale}");
            let w = (r.right - r.left + px.hair()) * scale;
            assert!((w - w.round()).abs() < 1e-3, "{scale}: {w}");
        }
        // 100 %: identity, the classic 0.5 centring.
        let px = Px::new(1.0);
        assert_eq!(px.hair(), 1.0);
        let r = px.stroke_rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0));
        assert_eq!((r.left, r.top, r.right, r.bottom), (0.5, 0.5, 9.5, 9.5));
        assert_eq!(Px::new(0.0).hair(), 1.0);
        assert_eq!(Px::new(f32::NAN).scale, 1.0);
    }

    #[test]
    fn chevron_and_up_boxes() {
        let c = TitleDeco::chevron_box(300.0, 36.0);
        assert_eq!(
            (c.left, c.top, c.right, c.bottom),
            (264.0, 2.0, 296.0, 34.0)
        );
        let u = TitleDeco::up_box(36.0);
        assert_eq!((u.left, u.top, u.right, u.bottom), (6.0, 4.0, 34.0, 32.0));
    }

    #[test]
    fn empty_text_sits_in_the_upper_part_or_centres_when_short() {
        assert_eq!(empty_text_top(200.0), 48.0);
        assert_eq!(empty_text_top(60.0), 20.0);
        assert_eq!(empty_text_top(10.0), 0.0);
    }
}
