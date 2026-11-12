//! Fluent and Liquid Glass design tokens for fence chrome.
//!
//! Interaction tokens follow WinUI; cool neutral surfaces and directional glass edges give
//! the desktop and settings a shared palette. Alpha is applied over the sampled backdrop.

use crate::backdrop::MicaTint;
use windows_canvas::ColorF;

/// Title bar height in DIPs of the built-in themes ([`Theme::title_height`]); for code that
/// has a window but no theme at hand.
pub const TITLE_HEIGHT: f32 = 36.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub liquid_glass: bool,
    /// Fence body fill when no wallpaper backdrop is available (SolidBackgroundFillColorBase).
    pub solid_fill: ColorF,
    /// Extra translucent layer drawn over the Mica-like backdrop (LayerFillColorDefault).
    pub layer_fill: ColorF,
    /// Thinner layer used over the Acrylic backdrop so the glass stays visible.
    pub glass_layer_fill: ColorF,
    /// The single glass edge: a 1 px rim, brighter at the top (ControlElevationBorder idiom).
    pub glass_rim_top: ColorF,
    pub glass_rim_bottom: ColorF,
    /// Tertiary text (TextFillColorTertiary): empty state, item count.
    pub text_tertiary: ColorF,
    /// ControlStrongFillColorDefault: the overlay scrollbar thumb in every state (WinUI keeps
    /// the colour and only changes the width on hover / press).
    pub control_strong: ColorF,
    /// Accent wash shown while a drop hovers the fence.
    pub drop_highlight: ColorF,
    /// Active tab pill in a tabbed fence (LayerOnAcrylicFill-ish).
    pub tab_active_fill: ColorF,
    /// Title bar tint (slightly stronger than the body).
    pub title_fill: ColorF,
    /// 1px outer stroke (CardStrokeColorDefault / SurfaceStrokeColorDefault).
    pub stroke: ColorF,
    /// Divider between title bar and content (DividerStrokeColorDefault).
    pub divider: ColorF,
    /// Primary text (TextFillColorPrimary).
    pub text_primary: ColorF,
    /// Secondary text (TextFillColorSecondary).
    pub text_secondary: ColorF,
    /// Hover fill for items/buttons (SubtleFillColorSecondary: #0FFFFFFF dark / #09000000 light).
    pub subtle_hover: ColorF,
    /// Pressed fill (SubtleFillColorTertiary: #0AFFFFFF dark / #06000000 light).
    pub subtle_pressed: ColorF,
    /// Selected item fill / stroke (accent-tinted, like the desktop ListView) while the fence
    /// is the active window.
    pub selection_fill: ColorF,
    pub selection_stroke: ColorF,
    /// Neutral selection look while another window is active (Explorer LISS_SELECTEDNOTFOCUS):
    /// a step stronger than `subtle_hover` so it still reads as "selected", not "hovered".
    pub selection_fill_inactive: ColorF,
    pub selection_stroke_inactive: ColorF,
    /// Keyboard focus visual (FocusStrokeColorOuter / FocusStrokeColorInner): a 2 px outer and
    /// 1 px inner ring around the cursor item after keyboard navigation.
    pub focus_outer: ColorF,
    pub focus_inner: ColorF,
    /// The opaque accent the selection tokens derive from (tray icon, settings page brand).
    pub accent: ColorF,
    /// Window corner radius in DIPs: Fluent 8, the Windows 11 top-level window radius, and
    /// Liquid Glass 16. The fence clips its own window to it (the composition root, so the
    /// plate, lens and shadow share one silhouette); Windows does not round fences.
    pub corner_radius: f32,
    /// Title bar height in DIPs.
    pub title_height: f32,
    /// Captions sit on bare wallpaper (a fully transparent plate): titles, tabs and labels
    /// get the legibility halo Liquid Glass labels always have.
    pub text_halo: bool,
    /// The recipe the wallpaper behind the fences is blurred and tinted with (Liquid Glass
    /// refracts the result).
    pub material: MicaTint,
    pub lens: Lens,
    /// A theme file's shadow values (None = the style's own).
    pub shadow: ShadowOverride,
    /// A custom theme's light and dark ink, which the text switches to over a bright or a
    /// dark backdrop (None = the built-in style's).
    pub custom_inks: Option<[Ink; 2]>,
    /// The system accent shade of the other colour mode (see [`Self::with_other_accent`]).
    pub other_accent: Option<[u8; 3]>,
    /// The 「文字颜色」 the text was fixed to by [`Self::with_label_color`]; titles and tabs
    /// over glass keep it instead of picking their ink from the wallpaper (Auto: they pick).
    pub label_color: pecofence_core::LabelColor,
    /// A theme file's title colour for fence titles and tabs without a 「标题颜色」 of their own
    /// (None = `text_primary`). It follows the backdrop like the text.
    pub title_text: Option<ColorF>,
    /// A theme file's halo and shadow for text drawn straight over wallpaper: for dark text
    /// (the light mode's values) and for light text (the dark mode's), see [`Self::text_fx`].
    pub text_fx: [TextFx; 2],
    /// The colour of the plate the text was last picked for (`with_quiet_text_on`): a link
    /// in the text is read against it ([`Self::link_ink`]). None where nothing was picked.
    pub plate: Option<ColorF>,
}

/// The halo and soft shadow a theme file gives text over bare wallpaper (None = built in: the
/// ¾ DIP halo of [`TextFx::HALO_WIDTH`], no shadow).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextFx {
    pub halo: Option<ColorF>,
    pub halo_width: Option<f32>,
    pub shadow: Option<ColorF>,
    pub shadow_blur: Option<f32>,
    pub shadow_offset: Option<f32>,
}

impl TextFx {
    pub const NONE: Self = Self {
        halo: None,
        halo_width: None,
        shadow: None,
        shadow_blur: None,
        shadow_offset: None,
    };
    /// The built-in halo: a four-way outline this many DIPs wide.
    pub const HALO_WIDTH: f32 = 0.75;
    /// A shadow's blur and downward offset (DIPs) when the theme sets only its colour.
    pub const SHADOW_BLUR: f32 = 2.0;
    pub const SHADOW_OFFSET: f32 = 1.0;
}

/// Liquid Glass lens strength, as multiples of the built-in refraction and rim light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lens {
    pub refraction: f32,
    pub rim_light: f32,
}

impl Lens {
    pub const BUILT_IN: Self = Self {
        refraction: 1.0,
        rim_light: 1.0,
    };
}

impl Default for Lens {
    fn default() -> Self {
        Self::BUILT_IN
    }
}

/// Shadow values a theme file sets: peak opacity, blur sigma and downward offset in DIPs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowOverride {
    pub opacity: Option<f32>,
    pub blur: Option<f32>,
    pub offset: Option<f32>,
}

impl ShadowOverride {
    pub const NONE: Self = Self {
        opacity: None,
        blur: None,
        offset: None,
    };
}

/// The text and control-feedback tokens that follow the backdrop under them (see
/// [`Theme::over_glass`]): one style's light or dark set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink {
    text_primary: ColorF,
    text_secondary: ColorF,
    text_tertiary: ColorF,
    control_strong: ColorF,
    subtle_hover: ColorF,
    subtle_pressed: ColorF,
    tab_active_fill: ColorF,
    /// The accent tint of a selected item: the dark set's is a light shade for a dark plate,
    /// the light set's a dark shade for a light one.
    selection_fill: ColorF,
    selection_stroke: ColorF,
    selection_fill_inactive: ColorF,
    selection_stroke_inactive: ColorF,
    focus_outer: ColorF,
    focus_inner: ColorF,
    title_text: Option<ColorF>,
}

impl Ink {
    pub fn of(theme: &Theme) -> Self {
        Self {
            text_primary: theme.text_primary,
            text_secondary: theme.text_secondary,
            text_tertiary: theme.text_tertiary,
            control_strong: theme.control_strong,
            subtle_hover: theme.subtle_hover,
            subtle_pressed: theme.subtle_pressed,
            tab_active_fill: theme.tab_active_fill,
            selection_fill: theme.selection_fill,
            selection_stroke: theme.selection_stroke,
            selection_fill_inactive: theme.selection_fill_inactive,
            selection_stroke_inactive: theme.selection_stroke_inactive,
            focus_outer: theme.focus_outer,
            focus_inner: theme.focus_inner,
            title_text: theme.title_text,
        }
    }
}

/// Plate brightness (0..=1) above which dark text is picked: a fence that already has dark
/// text keeps it down to `INK_KEEP_DARK`, one with light text keeps that up to
/// `INK_KEEP_LIGHT`, and one that has not picked yet switches at `INK_NEUTRAL`.
const INK_KEEP_DARK: f32 = 0.50;
const INK_KEEP_LIGHT: f32 = 0.62;
const INK_NEUTRAL: f32 = 0.56;
/// Fluent: light text is weak (under about 3.9 : 1, what names and rows need) on a plate
/// brighter than `LIGHT_TEXT_WEAK`, dark text on one darker than `DARK_TEXT_WEAK`. A fence with
/// at least `WEAK_PLATE_SHARE` of its plate there gets the halo: a busy wallpaper, or light
/// text kept near its threshold. Light text is kept from `LIGHT_TEXT_WEAK` up to
/// `INK_KEEP_LIGHT`: counted from `INK_NEUTRAL` only, an even plate in between (Windows 10's
/// own wallpaper in dark mode, brightness 0.53) had white text at 3.6 : 1 and no halo.
const LIGHT_TEXT_WEAK: f32 = INK_KEEP_DARK;
const DARK_TEXT_WEAK: f32 = 0.40;
const WEAK_PLATE_SHARE: f32 = 0.125;
/// A fence that has the halo keeps it down to this share, so a fence resting or moving near
/// the edge does not switch it with every pixel. One row or column of the 16 x 16 samples is
/// 1/16 of the plate: under that, so a step of one row over a straight edge keeps the state.
const WEAK_PLATE_KEEP: f32 = 0.06;
/// Secondary text (resting tabs, a clock's date, the details header, footers, counts, done
/// to-dos) follows the plate like primary text: where its own strength gives less than
/// `SECONDARY_BAR` : 1 against the plate (4.5 : 1, WCAG's figure for small text and what
/// Windows' own secondary text keeps on Mica), it moves towards the primary text (its alpha,
/// and its colour in a theme file whose secondary text has one of its own) as far as the bar
/// needs, at most `SECONDARY_REACH` of the way, so it never reads as primary text. On a
/// plate of middle brightness that end is reached first; the halo of a weak plate covers the
/// rest, as for primary text. Tertiary text (the count of a rolled fence, the chevron) the
/// same with 3 : 1, WCAG's figure for large text and the parts of a control.
const SECONDARY_BAR: f32 = 4.5;
const SECONDARY_REACH: f32 = 0.6;
const TERTIARY_BAR: f32 = 3.0;
const TERTIARY_REACH: f32 = 0.45;
/// The way is gone in this many steps: a fence dragged over a wallpaper draws its content
/// again only when a step changes ([`Theme::label_look`]).
const QUIET_TEXT_STEPS: u8 = 6;
/// A link in the text keeps as much of the accent's hue as it can: on a plate where the
/// accent gives less than `LINK_BAR` : 1 (3.9 : 1, what names and rows need) it moves towards
/// the text's colour, at most `LINK_REACH` of the way.
const LINK_BAR: f32 = 3.9;
const LINK_REACH: f32 = 0.7;

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// How bright a colour looks, on the scale of a grey's value (a grey gives its own value). A
/// plain weighted mean rates a saturated blue or cyan darker than it is: white text on
/// (47,160,210) has under 3 : 1, on the grey of the same mean 3.4 : 1.
fn brightness(rgb: [f32; 3]) -> f32 {
    luma(rgb.map(|c| c * c)).sqrt()
}

/// WCAG relative luminance of an sRGB colour.
fn relative_luminance(rgb: [f32; 3]) -> f32 {
    luma(rgb.map(|c| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }))
}

/// WCAG contrast of `ink`, drawn at its alpha, against the `plate` under it.
fn contrast_over(ink: ColorF, plate: [f32; 3]) -> f32 {
    let over = [ink.r, ink.g, ink.b];
    let text = relative_luminance([0, 1, 2].map(|i| plate[i] + (over[i] - plate[i]) * ink.a));
    let plate = relative_luminance(plate);
    (text.max(plate) + 0.05) / (text.min(plate) + 0.05)
}

/// `ink` moved towards `primary` (colour and alpha) as little as gives `bar` : 1 against
/// `plate`, at most `reach` of the way.
fn quiet_ink(ink: ColorF, primary: ColorF, plate: [f32; 3], bar: f32, reach: f32) -> ColorF {
    let at = |step: u8| {
        lerp(
            ink,
            primary,
            reach * f32::from(step) / f32::from(QUIET_TEXT_STEPS),
        )
    };
    (0..=QUIET_TEXT_STEPS)
        .map(at)
        .find(|c| contrast_over(*c, plate) >= bar)
        .unwrap_or_else(|| at(QUIET_TEXT_STEPS))
}

/// Up to 16 x 16 pixels (BGRA) spread evenly over `image`.
fn sample_grid(image: &crate::Image) -> impl Iterator<Item = &[u8]> {
    let (cols, rows) = (image.width.min(16), image.height.min(16));
    (0..rows).flat_map(move |row| {
        (0..cols).map(move |col| {
            let x = ((2 * col + 1) * image.width / (2 * cols)).min(image.width - 1);
            let y = ((2 * row + 1) * image.height / (2 * rows)).min(image.height - 1);
            let i = ((y * image.width + x) * 4) as usize;
            &image.bgra[i..i + 4]
        })
    })
}

/// `ink` with its lightness moved towards white or black, keeping the hue, as little as it
/// takes to reach a contrast of 3 : 1 against `band` (WCAG's figure for large text): towards
/// white first with `prefer_lighter` (the text around it is light), the other way when that
/// end cannot reach it. Unchanged when it already has that contrast.
pub fn legible_on(ink: ColorF, band: ColorF, prefer_lighter: bool) -> ColorF {
    let linear = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let relative = |c: ColorF| luma([linear(c.r), linear(c.g), linear(c.b)]);
    let band_l = relative(band);
    let reaches = |c: &ColorF| {
        let l = relative(*c);
        (l.max(band_l) + 0.05) / (l.min(band_l) + 0.05) >= 3.0
    };
    let towards = |end: f32| {
        (0..=20)
            .map(|step| {
                let t = step as f32 / 20.0;
                ColorF::new(
                    ink.r + (end - ink.r) * t,
                    ink.g + (end - ink.g) * t,
                    ink.b + (end - ink.b) * t,
                    ink.a,
                )
            })
            .find(reaches)
    };
    let (first, second) = if prefer_lighter {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    };
    towards(first).or_else(|| towards(second)).unwrap_or(ink)
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            liquid_glass: false,
            solid_fill: ColorF::from_rgba8(0x1B, 0x25, 0x2C, 0xF2),
            layer_fill: ColorF::from_rgba8(0x31, 0x40, 0x48, 0x4D),
            glass_layer_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x09),
            glass_rim_top: ColorF::from_rgba8(0xD9, 0xEC, 0xF5, 0x48),
            glass_rim_bottom: ColorF::from_rgba8(0xD9, 0xEC, 0xF5, 0x0A),
            text_tertiary: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x87),
            control_strong: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x8B),
            drop_highlight: ColorF::from_rgba8(0x60, 0xCD, 0xFF, 0x14),
            tab_active_fill: ColorF::from_rgba8(0xDE, 0xEE, 0xF6, 0x24),
            title_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x00),
            stroke: ColorF::from_rgba8(0xD9, 0xEC, 0xF5, 0x20),
            divider: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x00),
            text_primary: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF),
            text_secondary: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xC5),
            subtle_hover: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x0F),
            subtle_pressed: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x0A),
            selection_fill: ColorF::from_rgba8(0x60, 0xCD, 0xFF, 0x33),
            selection_stroke: ColorF::from_rgba8(0x60, 0xCD, 0xFF, 0x66),
            selection_fill_inactive: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x1F),
            selection_stroke_inactive: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x2E),
            focus_outer: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF),
            focus_inner: ColorF::from_rgba8(0x00, 0x00, 0x00, 0xB3),
            accent: ColorF::from_rgba8(0x60, 0xCD, 0xFF, 0xFF),
            corner_radius: 8.0,
            title_height: TITLE_HEIGHT,
            text_halo: false,
            material: MicaTint::ACRYLIC_DARK,
            lens: Lens::BUILT_IN,
            shadow: ShadowOverride::NONE,
            custom_inks: None,
            other_accent: None,
            label_color: pecofence_core::LabelColor::Auto,
            title_text: None,
            text_fx: [TextFx::NONE; 2],
            plate: None,
        }
    }

    pub const fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            liquid_glass: false,
            solid_fill: ColorF::from_rgba8(0xF4, 0xF7, 0xF8, 0xF2),
            layer_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x80),
            glass_layer_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x40),
            glass_rim_top: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xB0),
            glass_rim_bottom: ColorF::from_rgba8(0x23, 0x42, 0x53, 0x23),
            text_tertiary: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x72),
            control_strong: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x72),
            drop_highlight: ColorF::from_rgba8(0x00, 0x5F, 0xB8, 0x14),
            tab_active_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x99),
            title_fill: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x00),
            stroke: ColorF::from_rgba8(0x23, 0x42, 0x53, 0x1A),
            divider: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x00),
            text_primary: ColorF::from_rgba8(0x00, 0x00, 0x00, 0xE4),
            text_secondary: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x9E),
            subtle_hover: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x09),
            subtle_pressed: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x06),
            selection_fill: ColorF::from_rgba8(0x00, 0x5F, 0xB8, 0x29),
            selection_stroke: ColorF::from_rgba8(0x00, 0x5F, 0xB8, 0x66),
            selection_fill_inactive: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x12),
            selection_stroke_inactive: ColorF::from_rgba8(0x00, 0x00, 0x00, 0x1F),
            focus_outer: ColorF::from_rgba8(0x00, 0x00, 0x00, 0xE4),
            focus_inner: ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xB3),
            accent: ColorF::from_rgba8(0x00, 0x5F, 0xB8, 0xFF),
            corner_radius: 8.0,
            title_height: TITLE_HEIGHT,
            text_halo: false,
            material: MicaTint::ACRYLIC_LIGHT,
            lens: Lens::BUILT_IN,
            shadow: ShadowOverride::NONE,
            custom_inks: None,
            other_accent: None,
            label_color: pecofence_core::LabelColor::Auto,
            title_text: None,
            text_fx: [TextFx::NONE; 2],
            plate: None,
        }
    }

    pub const fn acrylic_tint(&self) -> MicaTint {
        self.material
    }

    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    /// Liquid Glass-inspired chrome. Keep mode and accent independent of the material.
    /// The composition root clips to this radius, including content and height animations.
    /// It matches the GPU bezel width (`liquid_glass::BEZEL_DIP`): kube's refraction
    /// profile needs a corner at least as round as its bezel.
    pub const fn with_liquid_glass(mut self) -> Self {
        self.liquid_glass = true;
        self.corner_radius = 16.0;
        self.material = match self.mode {
            ThemeMode::Dark => MicaTint::LIQUID_DARK,
            ThemeMode::Light => MicaTint::LIQUID_LIGHT,
        };
        self.glass_rim_top = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xCD);
        match self.mode {
            ThemeMode::Dark => {
                self.solid_fill = ColorF::from_rgba8(0x21, 0x29, 0x35, 0xFA);
                self.glass_layer_fill = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x03);
                self.glass_rim_bottom = ColorF::from_rgba8(0xC2, 0xD8, 0xF5, 0x52);
                self.tab_active_fill = ColorF::from_rgba8(0xEC, 0xF3, 0xFF, 0x36);
                self.subtle_hover = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x20);
                self.subtle_pressed = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x12);
                self.text_secondary = ColorF::from_rgba8(0xEC, 0xF2, 0xFC, 0xED);
                self.text_tertiary = ColorF::from_rgba8(0xEC, 0xF2, 0xFC, 0xBB);
            }
            ThemeMode::Light => {
                self.solid_fill = ColorF::from_rgba8(0xF4, 0xF6, 0xFC, 0xFA);
                self.glass_layer_fill = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x03);
                self.glass_rim_bottom = ColorF::from_rgba8(0x4A, 0x61, 0x86, 0x40);
                self.tab_active_fill = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xAE);
                // These go with dark text, so the glass under them is bright: the hover fill
                // darkens it like the pressed one. A white fill left a glass at 254 as it
                // was (under 3 levels of 255); this one gives 19 there and 7 on the darkest
                // glass that still has dark text (`INK_KEEP_DARK`).
                self.subtle_hover = ColorF::from_rgba8(0x20, 0x36, 0x55, 0x18);
                self.subtle_pressed = ColorF::from_rgba8(0x20, 0x36, 0x55, 0x15);
                self.text_primary = ColorF::from_rgba8(0x13, 0x20, 0x36, 0xFF);
                self.text_secondary = ColorF::from_rgba8(0x20, 0x30, 0x49, 0xEB);
                self.text_tertiary = ColorF::from_rgba8(0x20, 0x30, 0x49, 0xCC);
            }
        }
        self
    }

    /// Pick readable text from a sparse luminance sample without recolouring the lens.
    /// Material style, user mode, accent and geometry remain independent.
    pub fn over_glass(self, image: &crate::Image) -> Self {
        self.over_glass_with_appearance(image, 1.0, None)
    }

    /// Floating fences cover application content rather than wallpaper. A temporary
    /// neutral backing prevents document text showing through file names and details.
    pub fn with_floating_backing(mut self) -> Self {
        if self.liquid_glass {
            self.glass_layer_fill = ColorF {
                a: 0.94,
                ..self.solid_fill
            };
        }
        self
    }

    /// The solid veil used by the "more opaque" preset. Share the recipe with
    /// foreground selection so text is evaluated against the material actually drawn.
    pub fn opacity_veil(self, opacity: f32) -> ColorF {
        ColorF {
            a: ((opacity.clamp(0.4, 1.8) - 1.0) * 0.7).clamp(0.0, 0.6),
            ..self.solid_fill
        }
    }

    pub fn over_glass_with_appearance(
        self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
    ) -> Self {
        self.over_glass_with_previous_ink(image, opacity, tint, None)
    }

    /// Hysteresis prevents the caption and all icon labels flashing black/white while a
    /// moving plate samples a background close to the contrast threshold.
    pub fn over_glass_with_previous_ink(
        self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
    ) -> Self {
        if !self.liquid_glass || image.bgra.is_empty() {
            return self;
        }
        let foreground = if self.prefers_dark_text(image, opacity, tint, previous_dark_text) {
            self.ink(ThemeMode::Light)
        } else {
            self.ink(ThemeMode::Dark)
        };
        self.with_ink_of(foreground)
            .with_quiet_text_on(self.plate_color(image, opacity, tint))
    }

    /// A fully transparent plate (外观 → 全透明) while it is hidden: the captions sit on the
    /// wallpaper itself, so the ink is picked from the wallpaper under the fence (`image` with
    /// the material's recipe undone), with Liquid Glass's thresholds and hysteresis, and every
    /// caption gets the halo. The plate of a dark theme over a light wallpaper is dark, and
    /// the ink picked for that plate was white on the white wallpaper.
    pub fn over_bare_wallpaper(
        mut self,
        image: &crate::Image,
        previous_dark_text: Option<bool>,
    ) -> Self {
        self.text_halo = true;
        if image.bgra.is_empty() {
            return self;
        }
        let threshold = match previous_dark_text {
            Some(true) => INK_KEEP_DARK,
            Some(false) => INK_KEEP_LIGHT,
            None => INK_NEUTRAL,
        };
        let foreground = if self.wallpaper_brightness(image) > threshold {
            self.ink(ThemeMode::Light)
        } else {
            self.ink(ThemeMode::Dark)
        };
        self.with_ink_of(foreground)
            .with_quiet_text_on(self.wallpaper_color(image))
    }

    /// A fully transparent plate (外观 → 全透明) while its plate shows (the pointer over the
    /// fence, rolled up, a Peek): the captions sit on the default plate, so the ink is chosen
    /// against the tinted backdrop sample (the plate the pointer reveals); every caption
    /// keeps the halo, which it has on the bare wallpaper ([`Self::over_bare_wallpaper`]).
    pub fn over_clear_plate(
        mut self,
        image: &crate::Image,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
    ) -> Self {
        self.text_halo = true;
        if self.liquid_glass {
            return self.over_glass_with_previous_ink(image, 1.0, tint, previous_dark_text);
        }
        if image.bgra.is_empty() {
            return self;
        }
        let foreground = if self.prefers_dark_text(image, 1.0, tint, previous_dark_text) {
            self.ink(ThemeMode::Light)
        } else {
            self.ink(ThemeMode::Dark)
        };
        self.with_ink_of(foreground)
            .with_quiet_text_on(self.plate_color(image, 1.0, tint))
    }

    /// Fluent with 「文字颜色」 自动: the plate is see-through and takes its brightness from the
    /// wallpaper, not from the colour mode, so the text and control-feedback tokens are picked
    /// from the plate as it is drawn over `image` (this fence's crop of the blurred, tinted
    /// wallpaper), with Liquid Glass's thresholds and hysteresis. A fence that has not picked
    /// yet starts from the colour mode's own ink, which therefore stays near the threshold.
    /// The plate, rim, accent and mode stay. Where part of the plate is too bright for light
    /// text or too dark for dark text (a busy wallpaper, light text kept near its threshold)
    /// the text also gets the halo.
    pub fn over_acrylic(
        self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
    ) -> Self {
        self.over_acrylic_with_previous(image, opacity, tint, previous_dark_text, false)
    }

    /// [`Self::over_acrylic`] for a fence that picked before: `previous_halo` is whether it
    /// had the halo then. Like the text colour, the halo keeps its state near its threshold
    /// (on from `WEAK_PLATE_SHARE`, off under `WEAK_PLATE_KEEP`).
    pub fn over_acrylic_with_previous(
        self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
        previous_halo: bool,
    ) -> Self {
        if self.liquid_glass || image.bgra.is_empty() {
            return self;
        }
        let (brightness, bright_share, dark_share) = self.acrylic_plate_stats(image, opacity, tint);
        let mode_dark_text = self.mode == ThemeMode::Light;
        let threshold = if previous_dark_text.unwrap_or(mode_dark_text) {
            INK_KEEP_DARK
        } else {
            INK_KEEP_LIGHT
        };
        let dark_text = brightness > threshold;
        let mut theme = if dark_text == mode_dark_text {
            self
        } else if dark_text {
            self.with_ink_of(self.ink(ThemeMode::Light))
        } else {
            self.with_ink_of(self.ink(ThemeMode::Dark))
        };
        let weak_share = if dark_text { dark_share } else { bright_share };
        let needed = if previous_halo {
            WEAK_PLATE_KEEP
        } else {
            WEAK_PLATE_SHARE
        };
        theme.text_halo |= weak_share >= needed;
        theme.with_quiet_text_on(self.plate_color(image, opacity, tint))
    }

    /// What [`Self::over_acrylic`] decides from: the mean brightness (0..=1) of the Fluent
    /// plate over `image`, the share of it too bright for light text and the share too dark
    /// for dark text.
    pub fn acrylic_plate_stats(
        &self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
    ) -> (f32, f32, f32) {
        let (mut sum, mut bright, mut dim, mut count) = (0.0, 0, 0, 0);
        for p in sample_grid(image) {
            let plate =
                self.plate_over([p[2], p[1], p[0]].map(|c| c as f32 / 255.0), opacity, tint);
            let value = brightness(plate);
            sum += value;
            bright += (value > LIGHT_TEXT_WEAK) as u32;
            dim += (value < DARK_TEXT_WEAK) as u32;
            count += 1;
        }
        let n = count.max(1) as f32;
        (sum / n, bright as f32 / n, dim as f32 / n)
    }

    /// The mean colour of the plate over `image`, this fence's wallpaper crop (see
    /// [`Self::plate_over`]): what a title or an item name is read against.
    pub fn plate_color(&self, image: &crate::Image, opacity: f32, tint: Option<ColorF>) -> ColorF {
        let (mut sum, mut count) = ([0.0f32; 3], 0);
        for p in sample_grid(image) {
            for (s, c) in sum.iter_mut().zip([p[2], p[1], p[0]]) {
                *s += c as f32 / 255.0;
            }
            count += 1;
        }
        let n = count.max(1) as f32;
        // Every step of the recipe is a straight blend, so the mean plate is the plate of the
        // mean crop.
        let [r, g, b] = self.plate_over(sum.map(|s| s / n), opacity, tint);
        ColorF::new(r, g, b, 1.0)
    }

    /// The colour of the plate where the wallpaper crop has colour `crop`, composed as
    /// `fence_chrome::draw_plate` draws it. Fluent (`Backdrop::Glass`): the crop at
    /// `glass_alpha` over the real wallpaper showing through the window, which is the crop
    /// with the material's recipe ([`crate::Image::tint`]) undone. Liquid Glass's material
    /// leaves the wallpaper as it is. Then the glass layer, the opacity veil and the tint wash.
    fn plate_over(&self, crop: [f32; 3], opacity: f32, tint: Option<ColorF>) -> [f32; 3] {
        let wallpaper = self.wallpaper_under(crop);
        let glass = crate::fence_chrome::glass_alpha(opacity);
        let mut plate = [0, 1, 2].map(|i| wallpaper[i] + (crop[i] - wallpaper[i]) * glass);
        let mut layer = self.glass_layer_fill;
        layer.a *= opacity.min(1.0);
        for overlay in [
            Some(layer),
            Some(self.opacity_veil(opacity)),
            tint.map(|c| ColorF { a: 0.22, ..c }),
        ]
        .into_iter()
        .flatten()
        {
            for (p, o) in plate.iter_mut().zip([overlay.r, overlay.g, overlay.b]) {
                *p += (o - *p) * overlay.a;
            }
        }
        plate
    }

    /// The wallpaper itself where the backdrop crop has colour `crop`: the crop with the
    /// material's recipe ([`crate::Image::tint`]) undone.
    fn wallpaper_under(&self, crop: [f32; 3]) -> [f32; 3] {
        let material = self.material;
        let a = material.color.a.clamp(0.0, 1.0);
        let lo = material.luminosity_opacity.clamp(0.0, 1.0);
        let tint_rgb = [material.color.r, material.color.g, material.color.b];
        // A nearly flat material keeps too little of the wallpaper to work it out again.
        if a < 0.9 && lo < 0.9 {
            let x = [0, 1, 2].map(|i| (crop[i] - tint_rgb[i] * a) / (1.0 - a));
            let (lum_x, chroma) = (luma(x), material.chroma.clamp(0.1, 1.5));
            let lum = (lum_x - luma(tint_rgb) * lo) / (1.0 - lo);
            x.map(|c| (lum + (c - lum_x) / chroma).clamp(0.0, 1.0))
        } else {
            crop
        }
    }

    /// The secondary and tertiary text for `plate`, the colour under the text (see
    /// `SECONDARY_BAR`): each as strong as its bar needs there, within its reach.
    fn with_quiet_text_on(mut self, plate: ColorF) -> Self {
        self.plate = Some(plate);
        let plate = [plate.r, plate.g, plate.b];
        let primary = self.text_primary;
        self.text_secondary = quiet_ink(
            self.text_secondary,
            primary,
            plate,
            SECONDARY_BAR,
            SECONDARY_REACH,
        );
        self.text_tertiary = quiet_ink(
            self.text_tertiary,
            primary,
            plate,
            TERTIARY_BAR,
            TERTIARY_REACH,
        );
        self
    }

    /// The mean colour of the wallpaper under `image`, this fence's backdrop crop.
    fn wallpaper_color(&self, image: &crate::Image) -> ColorF {
        let (mut sum, mut count) = ([0.0f32; 3], 0);
        for p in sample_grid(image) {
            let wallpaper = self.wallpaper_under([p[2], p[1], p[0]].map(|c| c as f32 / 255.0));
            for (s, c) in sum.iter_mut().zip(wallpaper) {
                *s += c;
            }
            count += 1;
        }
        let n = count.max(1) as f32;
        ColorF::new(sum[0] / n, sum[1] / n, sum[2] / n, 1.0)
    }

    /// The mean brightness (0..=1) of the wallpaper under `image`, this fence's backdrop crop.
    pub fn wallpaper_brightness(&self, image: &crate::Image) -> f32 {
        let (mut sum, mut count) = (0.0, 0);
        for p in sample_grid(image) {
            sum += brightness(self.wallpaper_under([p[2], p[1], p[0]].map(|c| c as f32 / 255.0)));
            count += 1;
        }
        sum / count.max(1) as f32
    }

    /// Whether dark text reads better over `image` once the plate's own layers (glass layer,
    /// opacity veil, tint wash) are composited over it.
    fn prefers_dark_text(
        &self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
    ) -> bool {
        let mut sum = 0.0;
        let mut count = 0;
        for p in sample_grid(image) {
            sum += (0.2126 * p[2] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[0] as f32) / 255.0;
            count += 1;
        }
        let luminance = |c: ColorF| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
        let mut brightness = sum / count.max(1) as f32;
        let mut layer = self.glass_layer_fill;
        layer.a *= opacity.clamp(0.4, 1.0);
        for overlay in [
            Some(layer),
            Some(self.opacity_veil(opacity)),
            tint.map(|c| ColorF { a: 0.22, ..c }),
        ]
        .into_iter()
        .flatten()
        {
            brightness += (luminance(overlay) - brightness) * overlay.a;
        }
        let threshold = match previous_dark_text {
            Some(true) => INK_KEEP_DARK,
            Some(false) => INK_KEEP_LIGHT,
            None => INK_NEUTRAL,
        };
        brightness > threshold
    }

    /// The theme the item names, and the titles and tabs without a 「标题颜色」 of their own,
    /// are drawn with. 「文字颜色」 白色 / 黑色 takes the text of this material's dark /
    /// light theme whatever the backdrop, also for those titles in place of a theme file's
    /// title colour; the plate, hover and focus tokens stay as picked for it, and the halo
    /// follows the text.
    pub fn with_label_color(mut self, color: pecofence_core::LabelColor) -> Self {
        self.label_color = color;
        let mode = match color {
            pecofence_core::LabelColor::Auto => return self,
            pecofence_core::LabelColor::White => ThemeMode::Dark,
            pecofence_core::LabelColor::Black => ThemeMode::Light,
        };
        let ink = self.ink(mode);
        self.text_primary = ink.text_primary;
        self.text_secondary = ink.text_secondary;
        self.text_tertiary = ink.text_tertiary;
        self.title_text = None;
        self
    }

    /// The colour of the hairlines inside a fence (the rules of the date groups, the details
    /// header's underline and column dividers): this mode's `stroke` where the text is the
    /// mode's own, the other mode's where the text took the other colour for its plate. A
    /// dark-mode fence over a light wallpaper has a light plate, where the dark mode's light
    /// line does not show.
    pub fn hairline(&self) -> ColorF {
        let light_text = self.text_primary.r > 0.5;
        if light_text == (self.mode == ThemeMode::Dark) {
            self.stroke
        } else if light_text {
            Self::dark().stroke
        } else {
            Self::light().stroke
        }
    }

    /// What the backdrop decides about the content's text: the names' colour, whether they
    /// get a halo, and the secondary text's colour (a date, a footer, a done to-do). The content
    /// is redrawn when this changes.
    pub fn label_look(&self) -> (ColorF, bool, ColorF) {
        (self.text_primary, self.text_halo, self.text_secondary)
    }

    /// The colour of a fence title or tab caption without a 「标题颜色」 of its own: the theme
    /// file's title colour, else `fallback` (the primary or, for a resting tab, secondary text).
    pub fn title_ink(&self, fallback: ColorF) -> ColorF {
        self.title_text.unwrap_or(fallback)
    }

    /// The accent in the shade that goes with the text as it was picked for the plate: the
    /// light shade beside light text, the dark one beside dark text (in dark mode over a
    /// bright wallpaper the text is dark, and the mode's own light shade has 1.5 : 1 there).
    pub fn accent_beside_text(&self) -> ColorF {
        let light_text = self.text_primary.r > 0.5;
        if light_text == (self.mode == ThemeMode::Dark) {
            self.accent
        } else {
            match self.other_accent {
                Some([r, g, b]) => ColorF::from_rgba8(r, g, b, 0xFF),
                None if light_text => Self::dark().accent,
                None => Self::light().accent,
            }
        }
    }

    /// The ink of a link in the text (a web address in a note): the accent in the shade that
    /// goes with the text as it was picked for the plate, the light shade beside light text
    /// and the dark one beside dark text. On a plate of middle brightness the accent reads
    /// worse than the text around it (the light shade 2.2 : 1 on a mid grey, where white has
    /// 3.9 : 1): there it moves towards the text's colour, as secondary text does.
    pub fn link_ink(&self) -> ColorF {
        let accent = self.accent_beside_text();
        match self.plate {
            Some(plate) => quiet_ink(
                accent,
                ColorF {
                    a: 1.0,
                    ..self.text_primary
                },
                [plate.r, plate.g, plate.b],
                LINK_BAR,
                LINK_REACH,
            ),
            None => accent,
        }
    }

    /// The theme file's halo and shadow for text in `ink`: dark text takes the light mode's,
    /// light text the dark mode's.
    pub fn text_fx(&self, ink: ColorF) -> TextFx {
        self.text_fx[(ink.r > 0.5) as usize]
    }

    /// Light text with a halo, whatever the material: icon captions on bare wallpaper, as
    /// Windows draws the desktop icons (像桌面一样排列).
    pub fn desktop_ink(self) -> Self {
        let ink = self.ink(ThemeMode::Dark);
        let mut theme = self.with_ink_of(ink);
        // On bare wallpaper nothing says how bright the ground is: the selection stays the
        // colour mode's.
        theme.selection_fill = self.selection_fill;
        theme.selection_stroke = self.selection_stroke;
        theme.text_halo = true;
        // Light text takes `text_fx[1]`; a theme file's own effects stay.
        if theme.text_fx[1] == TextFx::NONE {
            let black = |a: f32| ColorF {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a,
            };
            theme.text_fx[1] = TextFx {
                halo: Some(black(0.5)),
                halo_width: Some(TextFx::HALO_WIDTH),
                shadow: Some(black(0.9)),
                shadow_blur: Some(TextFx::SHADOW_BLUR),
                shadow_offset: Some(TextFx::SHADOW_OFFSET),
            };
        }
        theme
    }

    /// The ink of this style's `mode` variant: a custom theme's own, else the built-in one.
    /// The selection tint of the theme's own mode is the theme's (the user's accent); the
    /// other mode's takes that mode's accent shade when the theme was given it.
    fn ink(&self, mode: ThemeMode) -> Ink {
        let mut ink = match self.custom_inks {
            Some([light, _]) if mode == ThemeMode::Light => light,
            Some([_, dark]) => dark,
            None => {
                let mut theme = Self::for_mode(mode);
                if self.liquid_glass {
                    theme = theme.with_liquid_glass();
                }
                if let Some(rgb) = self.other_accent {
                    theme = theme.with_accent(rgb);
                }
                Ink::of(&theme)
            }
        };
        if mode == self.mode {
            ink.selection_fill = self.selection_fill;
            ink.selection_stroke = self.selection_stroke;
        }
        ink
    }

    /// Takes the text and control-feedback tokens of `foreground`; material, accent and
    /// geometry stay.
    fn with_ink_of(mut self, foreground: Ink) -> Self {
        self.text_primary = foreground.text_primary;
        self.text_secondary = foreground.text_secondary;
        self.text_tertiary = foreground.text_tertiary;
        self.control_strong = foreground.control_strong;
        self.subtle_hover = foreground.subtle_hover;
        self.subtle_pressed = foreground.subtle_pressed;
        self.tab_active_fill = foreground.tab_active_fill;
        self.selection_fill = foreground.selection_fill;
        self.selection_stroke = foreground.selection_stroke;
        self.selection_fill_inactive = foreground.selection_fill_inactive;
        self.selection_stroke_inactive = foreground.selection_stroke_inactive;
        self.focus_outer = foreground.focus_outer;
        self.focus_inner = foreground.focus_inner;
        self.title_text = foreground.title_text;
        self
    }

    /// Retints the accent-derived tokens (selection, marquee, drop wash, caret) from the user's
    /// system accent — WinUI `AccentFillColorDefault`: pass `AccentLight2` for the dark theme
    /// and `AccentDark1` for the light theme. The alphas stay as in the defaults.
    pub fn with_accent(mut self, rgb: [u8; 3]) -> Self {
        let [r, g, b] = rgb;
        let tint = |a: f32| ColorF {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        };
        self.selection_fill = tint(self.selection_fill.a);
        self.selection_stroke = tint(self.selection_stroke.a);
        self.drop_highlight = tint(self.drop_highlight.a);
        self.accent = tint(1.0);
        self
    }

    /// The system accent shade of the other colour mode (`AccentDark1` for a dark theme,
    /// `AccentLight2` for a light one): the selection takes it when the text switches to the
    /// other mode's because of the wallpaper. Light mode over a dark wallpaper kept its dark
    /// accent tint on the dark plate, where a selected icon showed less than a hovered one
    /// (1.13 : 1 against the plate; the hover fill 1.39 : 1).
    pub fn with_other_accent(mut self, rgb: [u8; 3]) -> Self {
        self.other_accent = Some(rgb);
        self
    }

    /// A theme file's accent. The selection and the marks take it as it is; beside text of
    /// the other colour mode (dark mode over a bright wallpaper, light mode over a dark one)
    /// the marks take its shade for that text ([`Self::accent_beside_text`]), not Windows'
    /// own accent: the same hue, as dark as the light mode's default accent beside dark text,
    /// as light as the dark mode's beside light text. An accent that is already darker or
    /// lighter than that stays as it is.
    pub fn with_file_accent(self, rgb: [u8; 3]) -> Self {
        let lighter = self.mode == ThemeMode::Light;
        let like = if lighter {
            Self::dark().accent
        } else {
            Self::light().accent
        };
        let lightness = |c: [f32; 3]| (c[0].max(c[1]).max(c[2]) + c[0].min(c[1]).min(c[2])) / 2.0;
        let c = rgb.map(|v| v as f32 / 255.0);
        let (from, to) = (lightness(c), lightness([like.r, like.g, like.b]));
        let shade = |v: f32| {
            if lighter && from < to {
                // Towards white.
                1.0 - (1.0 - v) * (1.0 - to) / (1.0 - from)
            } else if !lighter && from > to {
                // Towards black.
                v * to / from
            } else {
                v
            }
        };
        let other = c.map(|v| (shade(v).clamp(0.0, 1.0) * 255.0).round() as u8);
        self.with_accent(rgb).with_other_accent(other)
    }

    /// The accent as 8-bit RGB (tray icon, settings page).
    pub fn accent_rgb8(&self) -> [u8; 3] {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        [c(self.accent.r), c(self.accent.g), c(self.accent.b)]
    }
}

/// `c` with its alpha multiplied by `a` (fading a token in or out).
pub fn with_alpha(c: ColorF, a: f32) -> ColorF {
    ColorF { a: c.a * a, ..c }
}

/// Straight per-channel RGBA interpolation from `a` (t = 0) to `b` (t = 1).
pub fn lerp(a: ColorF, b: ColorF, t: f32) -> ColorF {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: f32, y: f32| x + (y - x) * t;
    ColorF {
        r: mix(a.r, b.r),
        g: mix(a.g, b.g),
        b: mix(a.b, b.b),
        a: mix(a.a, b.a),
    }
}

/// Segoe UI Variable is the Windows 11 system font; DirectWrite exposes the optical sizes as
/// separate legacy family names. Windows 10 does not ship it, so these resolve to Segoe UI
/// there (see `pecofence_platform::fonts`).
pub fn font_text() -> &'static str {
    pecofence_platform::fonts::text_face()
}
pub fn font_small() -> &'static str {
    pecofence_platform::fonts::small_face()
}
pub fn font_display() -> &'static str {
    pecofence_platform::fonts::display_face()
}
/// Glyph font (chevrons, close, etc.): Segoe Fluent Icons, or Segoe MDL2 Assets on Windows 10.
pub fn font_icons() -> &'static str {
    pecofence_platform::fonts::icon_face()
}

#[cfg(test)]
mod tests {
    #[test]
    fn floating_glass_protects_text_without_changing_the_desktop_material() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let desktop = Theme::for_mode(mode).with_liquid_glass();
            let floating = desktop.with_floating_backing();
            assert!(desktop.glass_layer_fill.a < 0.02);
            assert!(floating.glass_layer_fill.a > 0.9);
            for gray in [0, 120, 255] {
                let foreground = floating.over_glass(&crate::Image::solid(32, 24, [gray; 3]));
                assert_eq!(foreground.text_primary.r < 0.5, mode == ThemeMode::Light);
            }
            assert_eq!(floating.corner_radius, desktop.corner_radius);
        }
    }

    use super::*;

    #[test]
    fn liquid_style_keeps_mode_accent_and_uses_its_own_material() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let base = Theme::for_mode(mode).with_accent([12, 100, 190]);
            let glass = base.with_liquid_glass();
            assert_eq!(glass.mode, mode);
            assert_eq!(glass.accent_rgb8(), base.accent_rgb8());
            assert_eq!(glass.selection_fill, base.selection_fill);
            assert!(glass.acrylic_tint().blur_sigma_dip < base.acrylic_tint().blur_sigma_dip);
            assert_eq!(glass.corner_radius, crate::liquid_glass::BEZEL_DIP);
            assert_eq!(glass.title_height, base.title_height);
            assert!(!base.liquid_glass);
        }
    }

    #[test]
    fn clear_plate_picks_ink_from_the_backdrop_and_adds_a_halo() {
        for base in [
            Theme::dark(),
            Theme::light(),
            Theme::dark().with_liquid_glass(),
        ] {
            assert!(!base.text_halo);
            let on_light =
                base.over_clear_plate(&crate::Image::solid(30, 30, [235; 3]), None, None);
            let on_dark = base.over_clear_plate(&crate::Image::solid(30, 30, [20; 3]), None, None);
            assert!(on_light.text_halo && on_dark.text_halo);
            assert!(
                on_light.text_primary.r < 0.5,
                "dark ink over a bright wallpaper"
            );
            assert!(
                on_dark.text_primary.r > 0.5,
                "light ink over a dark wallpaper"
            );
            // Material and accent stay the user's; the selection tint goes with the text.
            assert_eq!(on_light.liquid_glass, base.liquid_glass);
            assert_eq!(on_light.accent, base.accent);
            assert_eq!(on_light.selection_fill, Theme::light().selection_fill);
            assert_eq!(on_dark.selection_fill, Theme::dark().selection_fill);
            assert_eq!(on_light.glass_layer_fill, base.glass_layer_fill);
        }
    }

    /// A backdrop sample as a fence gets it: `rgb` with the theme's material recipe applied.
    fn backdrop_sample(theme: &Theme, rgb: [u8; 3]) -> crate::Image {
        let mut image = crate::Image::solid(30, 30, rgb);
        let tint = theme.acrylic_tint();
        image.tint(tint.color, tint.luminosity_opacity, 0.0, tint.chroma);
        image
    }

    #[test]
    fn hidden_clear_plate_picks_ink_from_the_wallpaper_not_from_the_plate() {
        // A theme file with a dark plate in dark mode (the example 暖色) over a white
        // wallpaper: light ink suits the plate, dark ink the wallpaper the text sits on
        // while the plate is hidden.
        let mut warm = Theme::dark();
        warm.material.color = ColorF::from_rgba8(0x3A, 0x24, 0x14, 0xD9);
        let white = backdrop_sample(&warm, [255; 3]);
        assert!(warm.wallpaper_brightness(&white) > 0.9);
        assert!(warm.over_clear_plate(&white, None, None).text_primary.r > 0.5);
        let bare = warm.over_bare_wallpaper(&white, None);
        assert!(bare.text_primary.r < 0.5, "dark ink on the white wallpaper");
        assert!(bare.text_halo);
        // Light mode over a dark wallpaper: light ink on the wallpaper.
        let light = Theme::light();
        let dark = backdrop_sample(&light, [90, 92, 93]);
        assert!(light.over_bare_wallpaper(&dark, None).text_primary.r > 0.5);
        // The built-in styles over a plain bright or dark wallpaper agree with the plate.
        for base in [
            Theme::dark(),
            Theme::light(),
            Theme::dark().with_liquid_glass(),
            Theme::light().with_liquid_glass(),
        ] {
            let bright = backdrop_sample(&base, [235; 3]);
            let dim = backdrop_sample(&base, [20; 3]);
            assert!(base.over_bare_wallpaper(&bright, None).text_primary.r < 0.5);
            assert!(base.over_bare_wallpaper(&dim, None).text_primary.r > 0.5);
        }
        // Near the threshold the last choice stays.
        let mid = backdrop_sample(&light, [150; 3]);
        assert!(light.over_bare_wallpaper(&mid, Some(true)).text_primary.r < 0.5);
        assert!(light.over_bare_wallpaper(&mid, Some(false)).text_primary.r > 0.5);
    }

    #[test]
    fn clear_glass_adapts_ink_without_darkening_the_background() {
        let base = Theme::dark().with_liquid_glass().with_accent([5, 90, 160]);
        let on_light = base.over_glass(&crate::Image::solid(30, 30, [225, 225, 225]));
        let on_dark = base.over_glass(&crate::Image::solid(30, 30, [25, 25, 25]));
        assert!(on_light.text_primary.r < 0.2);
        assert!(on_dark.text_primary.r > 0.9);
        assert_eq!(on_light.mode, base.mode);
        assert_eq!(on_light.accent_rgb8(), base.accent_rgb8());
        assert_eq!(on_light.acrylic_tint(), on_dark.acrylic_tint());
        let mut image = crate::Image::solid(30, 30, [210, 185, 120]);
        let tint = base.acrylic_tint();
        image.tint(tint.color, tint.luminosity_opacity, tint.noise, tint.chroma);
        assert_eq!(&image.bgra[..4], &[120, 185, 210, 255]);
    }

    #[test]
    fn glass_foreground_accounts_for_opacity_and_tint_without_changing_material() {
        let dark = Theme::dark().with_liquid_glass().with_accent([0, 120, 212]);
        let bright = crate::Image::solid(16, 16, [180, 180, 180]);
        assert!(dark.over_glass(&bright).text_primary.r < 0.2);
        let veiled = dark.over_glass_with_appearance(&bright, 1.6, None);
        assert!(veiled.text_primary.r > 0.9);
        assert_eq!(veiled.solid_fill, dark.solid_fill);
        assert_eq!(veiled.accent_rgb8(), dark.accent_rgb8());
        assert_eq!(veiled.focus_outer, Theme::dark().focus_outer);
        let light = Theme::light().with_liquid_glass();
        let dim = crate::Image::solid(16, 16, [110, 110, 110]);
        assert!(light.over_glass(&dim).text_primary.r > 0.9);
        assert!(
            light
                .over_glass_with_appearance(&dim, 1.6, None)
                .text_primary
                .r
                < 0.2
        );
        let mid = crate::Image::solid(16, 16, [165, 165, 165]);
        let tinted =
            dark.over_glass_with_appearance(&mid, 1.0, Some(ColorF::new(0.0, 0.0, 0.0, 1.0)));
        assert!(tinted.text_primary.r > 0.9);
        assert!(dark.over_glass(&mid).text_primary.r < 0.2);
        let fluent = Theme::light();
        assert_eq!(
            fluent
                .over_glass_with_appearance(&dim, 1.6, None)
                .text_primary,
            fluent.text_primary
        );
    }

    #[test]
    fn liquid_glass_hover_and_pressed_fills_show_on_every_glass() {
        // How far a fill moves the brightness of a plain glass of `grey`, in levels of 255.
        let shift = |fill: ColorF, grey: u8| {
            (luma([fill.r, fill.g, fill.b]) * 255.0 - grey as f32) * fill.a
        };
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let base = Theme::for_mode(mode).with_liquid_glass();
            for grey in (0..=255u8).step_by(5) {
                let picked = base.over_glass(&crate::Image::solid(16, 16, [grey; 3]));
                let hover = shift(picked.subtle_hover, grey);
                let pressed = shift(picked.subtle_pressed, grey);
                // Dark text sits on a bright glass, which the fills darken; light text on a
                // dark one, which they lighten. A white fill on a white glass showed nothing.
                let dark_text = picked.text_primary.r < 0.5;
                assert_eq!(hover < 0.0, dark_text, "glass {grey}: hover {hover}");
                assert!(hover.abs() >= 8.0, "glass {grey}: hover {hover}");
                assert!(pressed.abs() >= 5.0, "glass {grey}: pressed {pressed}");
                // The pressed fill recedes from the hover one, as in the Fluent style.
                assert!(pressed.abs() < hover.abs(), "glass {grey}");
            }
        }
    }

    #[test]
    fn fixed_label_color_overrides_the_backdrop_ink_only() {
        use pecofence_core::LabelColor;
        for base in [
            Theme::dark(),
            Theme::light(),
            Theme::dark().with_liquid_glass(),
            Theme::light().with_liquid_glass(),
        ] {
            for gray in [20, 235] {
                let picked = base.over_glass(&crate::Image::solid(30, 30, [gray; 3]));
                assert_eq!(
                    picked.with_label_color(LabelColor::Auto).text_primary,
                    picked.text_primary
                );
                let white = picked.with_label_color(LabelColor::White);
                let black = picked.with_label_color(LabelColor::Black);
                assert!(white.text_primary.r > 0.9 && white.text_secondary.r > 0.9);
                assert!(black.text_primary.r < 0.2 && black.text_secondary.r < 0.2);
                // The plate and its feedback stay as picked for the backdrop.
                assert_eq!(white.subtle_hover, picked.subtle_hover);
                assert_eq!(black.focus_outer, picked.focus_outer);
                assert_eq!(white.liquid_glass, base.liquid_glass);
            }
        }
    }

    #[test]
    fn accent_retints_only_accent_tokens_and_keeps_alphas() {
        let base = Theme::dark();
        let t = base.with_accent([0x4C, 0xC2, 0xFF]);
        assert_eq!(t.accent_rgb8(), [0x4C, 0xC2, 0xFF]);
        assert_eq!(t.selection_fill.a, base.selection_fill.a);
        assert_eq!(t.selection_stroke.a, base.selection_stroke.a);
        assert_eq!(t.drop_highlight.a, base.drop_highlight.a);
        assert!((t.selection_fill.r - 0x4C as f32 / 255.0).abs() < 1e-6);
        assert_eq!(t.subtle_hover, base.subtle_hover);
        assert_eq!(t.text_primary, base.text_primary);
        // The documented default accent reproduces today's literals exactly.
        let d = Theme::dark().with_accent([0x60, 0xCD, 0xFF]);
        assert_eq!(d.selection_fill, Theme::dark().selection_fill);
    }

    /// A fence's crop of a plain wallpaper, as the backdrop pipeline hands it to a fence of
    /// `theme` (blurred, which leaves a plain colour alone, then tinted).
    fn crop_of(theme: &Theme, wallpaper: [u8; 3]) -> crate::Image {
        let mut image = crate::Image::solid(32, 24, wallpaper);
        let m = theme.acrylic_tint();
        image.tint(m.color, m.luminosity_opacity, m.noise, m.chroma);
        image
    }

    #[test]
    fn fluent_plate_colour_is_the_one_measured_on_screen() {
        // Read from screenshots of 0.3.3: dark mode over a plain (236,239,244) wallpaper shows
        // a (220,222,225) plate, light mode over (19,20,25) a (97,98,102) one.
        for (theme, wallpaper, plate) in [
            (Theme::dark(), [236, 239, 244], [220.0, 222.0, 225.0]),
            (Theme::light(), [19, 20, 25], [97.0, 98.0, 102.0]),
        ] {
            let c = theme.plate_color(&crop_of(&theme, wallpaper), 1.0, None);
            for (got, want) in [c.r, c.g, c.b].into_iter().zip(plate) {
                assert!((got * 255.0 - want).abs() < 4.0, "{c:?} against {plate:?}");
            }
        }
    }

    #[test]
    fn secondary_text_is_raised_to_its_bar_on_the_plate_and_stays_under_primary() {
        let against = |theme: &Theme, ink: ColorF, wallpaper: [u8; 3]| {
            let plate = theme.plate_color(&crop_of(theme, wallpaper), 1.0, None);
            contrast_over(ink, [plate.r, plate.g, plate.b])
        };
        for theme in [Theme::dark(), Theme::light()] {
            for grey in (0..=255).step_by(5) {
                let wallpaper = [grey as u8; 3];
                let picked = theme.over_acrylic(&crop_of(&theme, wallpaper), 1.0, None, None);
                let (first, second, third) = (
                    picked.text_primary,
                    picked.text_secondary,
                    picked.text_tertiary,
                );
                // The text's own colour mode decides what each started from.
                let own = if first.r > 0.5 {
                    Theme::dark()
                } else {
                    Theme::light()
                };
                let most = |ink: ColorF, reach: f32| ink.a + (first.a - ink.a) * reach;
                let (most2, most3) = (
                    most(own.text_secondary, SECONDARY_REACH),
                    most(own.text_tertiary, TERTIARY_REACH),
                );
                // Never as strong as primary text, never weaker than the style's own.
                assert!(own.text_secondary.a <= second.a && second.a <= most2 + 1e-4);
                assert!(own.text_tertiary.a <= third.a && third.a <= most3 + 1e-4);
                assert!(third.a < second.a && second.a < first.a - 0.05, "{grey}");
                // At its bar, or as strong as its reach allows.
                let c2 = against(&theme, second, wallpaper);
                assert!(
                    c2 >= SECONDARY_BAR || second.a >= most2 - 1e-4,
                    "{grey}: {c2} at {}",
                    second.a
                );
                let c3 = against(&theme, third, wallpaper);
                assert!(c3 >= TERTIARY_BAR || third.a >= most3 - 1e-4);
            }
        }
        // A dark or a light plate leaves the style's own secondary text as it is.
        let dark = Theme::dark();
        let on_dark = dark.over_acrylic(&crop_of(&dark, [20, 20, 24]), 1.0, None, None);
        assert_eq!(on_dark.text_secondary, dark.text_secondary);
        assert_eq!(on_dark.text_tertiary, dark.text_tertiary);
        // Dark mode over a mid-bright wallpaper (white text kept, 3.1 : 1 before): raised.
        let on_mid = dark.over_acrylic(&crop_of(&dark, [120, 150, 170]), 1.0, None, None);
        assert!(on_mid.text_primary.r > 0.9, "light text on the mid plate");
        assert!(on_mid.text_secondary.a > dark.text_secondary.a + 0.05);
        // Liquid Glass and a fully transparent fence go by the same rule.
        let glass = Theme::light().with_liquid_glass();
        let image = crate::Image::solid(16, 16, [128, 128, 128]);
        let picked = glass.over_glass(&image);
        assert!(picked.text_secondary.a > glass.text_secondary.a.min(0.94));
        for picked in [
            dark.over_bare_wallpaper(&crop_of(&dark, [120, 150, 170]), None),
            dark.over_clear_plate(&crop_of(&dark, [120, 150, 170]), None, None),
        ] {
            // Whichever text the pick took: stronger than that text's own secondary.
            let own = if picked.text_primary.r > 0.5 {
                Theme::dark()
            } else {
                Theme::light()
            };
            assert!(picked.text_secondary.a > own.text_secondary.a + 0.01);
            assert!(picked.text_secondary.a < picked.text_primary.a - 0.05);
        }
        // A link keeps the accent where that reads and moves towards the text where not.
        assert_eq!(on_dark.link_ink(), dark.accent);
        let link = on_mid.link_ink();
        let plate = on_mid.plate.unwrap();
        let plate = [plate.r, plate.g, plate.b];
        assert!(contrast_over(link, plate) > contrast_over(dark.accent, plate) + 0.5);
        assert!(link.b > link.r + 0.1, "still blue: {link:?}");
        // Accent marks (the bar under the shown tab, a ticked box) take the shade that goes
        // with the text: dark mode over a bright wallpaper has dark text and the dark shade.
        let on_bright = dark.over_acrylic(&crop_of(&dark, [236, 239, 243]), 1.0, None, None);
        assert!(
            on_bright.text_primary.r < 0.5,
            "dark text on the bright plate"
        );
        assert_eq!(on_bright.accent_beside_text(), Theme::light().accent);
        assert_eq!(on_dark.accent_beside_text(), dark.accent);
        let bright = on_bright.plate.unwrap();
        let bright = [bright.r, bright.g, bright.b];
        assert!(contrast_over(dark.accent, bright) < 2.0);
        assert!(contrast_over(on_bright.accent_beside_text(), bright) > 3.0);
        // A theme file's accent reaches these marks beside the other mode's text too, in the
        // shade for that text: orange stays orange and reads on the bright plate (Windows'
        // blue stood there), and the same the other way round.
        let file =
            crate::CustomTheme::parse(r##"{ "base": "fluent", "accent": "#E8710A" }"##).unwrap();
        let themed = file.apply(
            dark.with_accent([0x60, 0xCD, 0xFF])
                .with_other_accent([0, 0x5F, 0xB8]),
        );
        assert_eq!(themed.accent_rgb8(), [0xE8, 0x71, 0x0A]);
        let mark = themed
            .over_acrylic(&crop_of(&themed, [236, 239, 243]), 1.0, None, None)
            .accent_beside_text();
        assert!(
            mark.r > mark.b + 0.4 && mark.r < 0.8,
            "a dark orange: {mark:?}"
        );
        assert!(contrast_over(mark, bright) > 3.0);
        let own = themed.over_acrylic(&crop_of(&themed, [20, 20, 24]), 1.0, None, None);
        assert_eq!(own.accent_beside_text(), themed.accent);
        let themed = file.apply(Theme::light().with_other_accent([0x60, 0xCD, 0xFF]));
        let on_night = themed.over_acrylic(&crop_of(&themed, [19, 20, 25]), 1.0, None, None);
        assert!(
            on_night.text_primary.r > 0.5,
            "light text on the dark plate"
        );
        let mark = on_night.accent_beside_text();
        assert!(
            mark.r > mark.b + 0.3 && mark.g > 0.6,
            "a light orange: {mark:?}"
        );
        // A theme file's secondary text in a colour of its own moves towards its primary
        // text: an opaque brown on a plate of middle brightness stayed at 2.1 : 1.
        let brown = ColorF::from_rgba8(0x6B, 0x4A, 0x2E, 0xFF);
        let deep = ColorF::from_rgba8(0x3B, 0x24, 0x12, 0xFF);
        let plate = [152.0 / 255.0, 129.0 / 255.0, 89.0 / 255.0];
        let moved = quiet_ink(brown, deep, plate, SECONDARY_BAR, SECONDARY_REACH);
        assert!(contrast_over(brown, plate) < 2.3);
        assert!(contrast_over(moved, plate) > 3.0);
        assert!(moved.r < brown.r && moved.r > deep.r);
    }

    #[test]
    fn fluent_text_follows_the_plate_where_the_mode_s_ink_does_not_read() {
        let (dark, light) = (Theme::dark(), Theme::light());
        let over = |theme: Theme, wallpaper: [u8; 3]| {
            theme.over_acrylic(&crop_of(&theme, wallpaper), 1.0, None, None)
        };
        // Dark mode over a light wallpaper: dark text and the light set's feedback, on the
        // same plate, accent and mode.
        let flipped = over(dark, [236, 239, 244]);
        assert!(flipped.text_primary.r < 0.2 && flipped.text_secondary.r < 0.2);
        assert_eq!(flipped.subtle_hover, light.subtle_hover);
        assert_eq!(flipped.control_strong, light.control_strong);
        assert_eq!(flipped.mode, ThemeMode::Dark);
        assert_eq!(flipped.glass_layer_fill, dark.glass_layer_fill);
        assert_eq!(flipped.selection_fill, light.selection_fill);
        assert_eq!(flipped.accent, dark.accent);
        assert_eq!(flipped.acrylic_tint(), dark.acrylic_tint());
        assert!(!flipped.text_halo);
        // Light mode over a dark wallpaper: light text, and the dark set's selection tint (the
        // light mode's dark blue did not show on the dark plate).
        let flipped = over(light, [19, 20, 25]);
        assert!(flipped.text_primary.r > 0.9);
        assert_eq!(flipped.subtle_hover, dark.subtle_hover);
        assert_eq!(flipped.selection_fill, dark.selection_fill);
        assert_eq!(flipped.selection_stroke, dark.selection_stroke);
        // With the user's accent: the other mode's shade of it, at that mode's strength; the
        // mode's own where the text does not switch.
        let (own, other) = ([0x00, 0x67, 0xC0], [0x4C, 0xC2, 0xFF]);
        let accented = light.with_accent(own).with_other_accent(other);
        let flipped = over(accented, [19, 20, 25]);
        assert_eq!(
            flipped.selection_fill,
            dark.with_accent(other).selection_fill
        );
        assert_eq!(flipped.accent_rgb8(), own);
        assert_eq!(
            over(accented, [236, 239, 244]).selection_fill,
            accented.selection_fill
        );
        let glass = accented.with_liquid_glass();
        let kept = glass.over_glass(&crate::Image::solid(30, 30, [235; 3]));
        assert_eq!(kept.selection_fill, glass.selection_fill);
        let turned = glass.over_glass(&crate::Image::solid(30, 30, [20; 3]));
        assert_eq!(
            turned.selection_fill,
            dark.with_accent(other).selection_fill
        );
        assert_eq!(flipped.glass_layer_fill, light.glass_layer_fill);
        // Where the mode's own ink reads, nothing changes: matching wallpapers and the middle
        // greys, which each mode's plate pulls to its own side.
        for wallpaper in [[19, 20, 25], [128, 128, 128]] {
            let same = over(dark, wallpaper);
            assert_eq!(same.text_primary, dark.text_primary);
            assert_eq!(same.subtle_hover, dark.subtle_hover);
        }
        for wallpaper in [[236, 239, 244], [128, 128, 128]] {
            let same = over(light, wallpaper);
            assert_eq!(same.text_primary, light.text_primary);
            assert_eq!(same.subtle_hover, light.subtle_hover);
        }
        // A thicker plate (更厚实) is the mode's own veil: its ink reads again.
        let veiled = dark.over_acrylic(&crop_of(&dark, [236, 239, 244]), 1.8, None, None);
        assert_eq!(veiled.text_primary, dark.text_primary);
        // A dark tint over the whole plate does the same.
        let navy = Some(ColorF::from_rgba8(0x10, 0x18, 0x40, 0xFF));
        let mid = crop_of(&dark, [190, 190, 190]);
        assert!(dark.over_acrylic(&mid, 1.0, None, None).text_primary.r < 0.2);
        assert_eq!(
            dark.over_acrylic(&mid, 1.0, navy, None).text_primary,
            dark.text_primary
        );
        // Liquid Glass picks through `over_glass`; no wallpaper, no pick.
        let glass = dark.with_liquid_glass();
        let bright = crop_of(&dark, [236, 239, 244]);
        assert_eq!(
            glass.over_acrylic(&bright, 1.0, None, None).text_primary,
            glass.text_primary
        );
        assert_eq!(
            dark.over_acrylic(&crate::Image::default(), 1.0, None, None)
                .text_primary,
            dark.text_primary
        );
    }

    #[test]
    fn a_saturated_plate_counts_as_bright_as_it_looks() {
        for gray in [0.0, 0.2, 0.56, 1.0] {
            assert!((brightness([gray; 3]) - gray).abs() < 1e-5);
        }
        // Dark mode over a clear blue wallpaper: the plate is (51,162,209), where white text has
        // 2.8 : 1. Its weighted mean stays under the neutral threshold, its brightness does not.
        let dark = Theme::dark();
        let image = crop_of(&dark, [40, 170, 225]);
        let c = dark.plate_color(&image, 1.0, None);
        assert!(luma([c.r, c.g, c.b]) < INK_NEUTRAL);
        let (plate, too_bright, _) = dark.acrylic_plate_stats(&image, 1.0, None);
        assert!(INK_NEUTRAL < plate && plate < INK_KEEP_LIGHT, "{plate}");
        assert_eq!(too_bright, 1.0);
        // The fence keeps the mode's light text (it has not crossed the keep threshold) and
        // gets the halo, as light text over such a plate always has in Liquid Glass.
        let picked = dark.over_acrylic(&image, 1.0, None, None);
        assert_eq!(picked.text_primary, dark.text_primary);
        assert!(picked.text_halo);
    }

    #[test]
    fn fluent_text_keeps_its_colour_near_the_threshold() {
        let dark = Theme::dark();
        let stats = |gray: u8| dark.acrylic_plate_stats(&crop_of(&dark, [gray; 3]), 1.0, None);
        // Light text kept above the neutral threshold is weak there: it has the halo.
        assert!(stats(150).0 > INK_NEUTRAL);
        for gray in [150, 160] {
            let kept = dark.over_acrylic(&crop_of(&dark, [gray; 3]), 1.0, None, None);
            assert!(kept.text_primary.r > 0.9 && kept.text_halo);
        }
        // Two plates between the two keep thresholds.
        let (low, high) = (stats(150).0, stats(160).0);
        assert!(INK_KEEP_DARK < low && high < INK_KEEP_LIGHT, "{low} {high}");
        let pick = |gray: u8, previous: Option<bool>| {
            dark.over_acrylic(&crop_of(&dark, [gray; 3]), 1.0, None, previous)
                .text_primary
                .r
                < 0.5
        };
        // Not picked yet: the mode's light text stays. Light stays light, dark stays dark.
        for gray in [150, 160] {
            assert!(!pick(gray, None));
            assert!(!pick(gray, Some(false)));
            assert!(pick(gray, Some(true)));
        }
        // Clear of the band both switch.
        assert!(pick(200, Some(false)));
        assert!(!pick(100, Some(true)));
        // Light mode starts from its dark text.
        let light = Theme::light();
        let mid = crop_of(&light, [80; 3]);
        let (brightness, _, _) = light.acrylic_plate_stats(&mid, 1.0, None);
        assert!(INK_KEEP_DARK < brightness && brightness < INK_NEUTRAL);
        let kept = light.over_acrylic(&mid, 1.0, None, None);
        assert_eq!(kept.text_primary, light.text_primary);
        // Dark text still reads well there: no halo.
        assert!(!kept.text_halo);
    }

    #[test]
    fn fluent_text_gets_the_halo_on_a_plate_that_is_light_and_dark() {
        let dark = Theme::dark();
        // A quarter of the fence over a bright part of the wallpaper, the rest over a dark one.
        let mut image = crate::Image::solid(32, 24, [20, 24, 40]);
        for row in 0..6 {
            for col in 0..32 {
                let i = (row * 32 + col) * 4;
                image.bgra[i..i + 3].copy_from_slice(&[240, 235, 230]);
            }
        }
        let m = dark.acrylic_tint();
        image.tint(m.color, m.luminosity_opacity, m.noise, m.chroma);
        let (_, bright_share, _) = dark.acrylic_plate_stats(&image, 1.0, None);
        assert!((bright_share - 0.25).abs() < 0.07, "{bright_share}");
        let busy = dark.over_acrylic(&image, 1.0, None, None);
        assert_eq!(busy.text_primary, dark.text_primary);
        assert!(busy.text_halo);
        // The same with dark text on a mostly bright plate.
        let mut image = crate::Image::solid(32, 24, [240, 235, 230]);
        for row in 0..6 {
            for col in 0..32 {
                let i = (row * 32 + col) * 4;
                image.bgra[i..i + 3].copy_from_slice(&[40, 24, 20]);
            }
        }
        image.tint(m.color, m.luminosity_opacity, m.noise, m.chroma);
        let busy = dark.over_acrylic(&image, 1.0, None, None);
        assert!(busy.text_primary.r < 0.2 && busy.text_halo);
        // An even plate between the two thresholds, as Windows 10's own wallpaper gives in dark
        // mode ((22,139,226) on screen, brightness 0.53). The light text is kept and white has
        // about 3.6 : 1 there; no part of the plate is over the neutral threshold, so it had
        // no halo.
        let even = crop_of(&dark, [140; 3]);
        let (brightness, bright_share, _) = dark.acrylic_plate_stats(&even, 1.0, None);
        assert!(
            LIGHT_TEXT_WEAK < brightness && brightness < INK_NEUTRAL,
            "{brightness}"
        );
        assert_eq!(bright_share, 1.0);
        let kept = dark.over_acrylic(&even, 1.0, None, None);
        assert_eq!(kept.text_primary, dark.text_primary);
        assert!(kept.text_halo);
        // A plain wallpaper where the picked text reads needs none.
        for gray in [20, 128, 236] {
            assert!(
                !dark
                    .over_acrylic(&crop_of(&dark, [gray; 3]), 1.0, None, None)
                    .text_halo
            );
        }
    }

    #[test]
    fn fluent_halo_keeps_its_state_near_its_threshold() {
        let dark = Theme::dark();
        // A dark plate with `bright` of its 256 samples over a bright part of the wallpaper.
        let plate = |bright: usize| {
            let mut image = crate::Image::solid(16, 16, [20, 24, 40]);
            for p in image.bgra.as_chunks_mut::<4>().0.iter_mut().take(bright) {
                p[..3].copy_from_slice(&[240, 235, 230]);
            }
            let m = dark.acrylic_tint();
            image.tint(m.color, m.luminosity_opacity, m.noise, m.chroma);
            image
        };
        let halo = |bright: usize, previous: bool| {
            let picked = dark.over_acrylic_with_previous(&plate(bright), 1.0, None, None, previous);
            assert_eq!(picked.text_primary, dark.text_primary);
            picked.text_halo
        };
        // On from an eighth of the plate (32 samples), off under 6 % (15 samples).
        assert!(!halo(31, false) && halo(32, false));
        assert!(halo(16, true) && !halo(15, true));
        // A fence wobbling over the edge of a bright part, one sample row (16 samples) at a
        // time: it switched with every step, now once.
        let (mut state, mut switches, mut switches_before) = (false, 0, 0);
        let mut state_before = false;
        for bright in [16, 32, 16, 32, 16, 32, 48, 32, 16, 32] {
            let now = halo(bright, state);
            switches += (now != state) as u32;
            state = now;
            let now = halo(bright, false);
            switches_before += (now != state_before) as u32;
            state_before = now;
        }
        assert_eq!(switches, 1);
        assert_eq!(switches_before, 7);
        // Clear of the edge it goes off again.
        assert!(!halo(0, state));
    }

    #[test]
    fn hairlines_follow_the_text_onto_the_other_plate() {
        let (dark, light) = (Theme::dark(), Theme::light());
        // The mode's own text: the mode's own line, as before.
        assert_eq!(dark.hairline(), dark.stroke);
        assert_eq!(light.hairline(), light.stroke);
        let same = dark.over_acrylic(&crop_of(&dark, [19, 20, 25]), 1.0, None, None);
        assert_eq!(same.hairline(), dark.stroke);
        let same = light.over_acrylic(&crop_of(&light, [236, 239, 244]), 1.0, None, None);
        assert_eq!(same.hairline(), light.stroke);
        // Dark mode over a light wallpaper: dark text, so the light mode's dark line; the
        // plate's own rim stays.
        let flipped = dark.over_acrylic(&crop_of(&dark, [236, 239, 244]), 1.0, None, None);
        assert_eq!(flipped.hairline(), light.stroke);
        assert_eq!(flipped.stroke, dark.stroke);
        // Light mode over a dark wallpaper: the dark mode's light line.
        let flipped = light.over_acrylic(&crop_of(&light, [19, 20, 25]), 1.0, None, None);
        assert_eq!(flipped.hairline(), dark.stroke);
        // 白色 / 黑色 take the line with them, like the left tab column's lines.
        let white = light.with_label_color(pecofence_core::LabelColor::White);
        assert_eq!(white.hairline(), dark.stroke);
    }

    #[test]
    fn a_title_in_the_tint_moves_its_lightness_until_it_reads_on_the_band() {
        let linear = |c: f32| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let relative = |c: ColorF| luma([linear(c.r), linear(c.g), linear(c.b)]);
        let contrast = |a: ColorF, b: ColorF| {
            let (la, lb) = (relative(a), relative(b));
            (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
        };
        // Measured on 0.3.3: tint #3b82f6 as the title colour on its own band, 1.1 : 1.
        let tint = ColorF::from_rgba8(0x3B, 0x82, 0xF6, 0xFF);
        let band = ColorF::from_rgba8(109, 126, 150, 0xFF);
        assert!(contrast(tint, band) < 1.3);
        let lifted = legible_on(tint, band, true);
        assert!(contrast(lifted, band) >= 3.0);
        // Still the tint's hue: blue over green over red, and no further than needed.
        assert!(lifted.b > lifted.g && lifted.g > lifted.r);
        assert!(contrast(lifted, band) < 3.6);
        assert_eq!(lifted.a, tint.a);
        // Towards black on request, and when white cannot reach it on a bright band.
        let lowered = legible_on(tint, band, false);
        assert!(contrast(lowered, band) >= 3.0 && relative(lowered) < relative(band));
        let bright = ColorF::from_rgba8(225, 232, 245, 0xFF);
        let on_bright = legible_on(tint, bright, true);
        assert!(contrast(on_bright, bright) >= 3.0 && relative(on_bright) < relative(bright));
        // A title that already reads stays as chosen.
        let dark_band = ColorF::from_rgba8(30, 36, 52, 0xFF);
        assert_eq!(legible_on(tint, dark_band, true), tint);
    }

    #[test]
    fn lerp_and_with_alpha() {
        let a = ColorF::from_rgba8(0, 0, 0, 0);
        let b = ColorF::from_rgba8(255, 255, 255, 255);
        let m = lerp(a, b, 0.5);
        assert!((m.r - 0.5).abs() < 1e-6 && (m.a - 0.5).abs() < 1e-6);
        assert_eq!(lerp(a, b, -1.0), a);
        assert_eq!(lerp(a, b, 2.0), b);
        let h = with_alpha(ColorF::from_rgba8(1, 2, 3, 0x80), 0.5);
        assert!((h.a - 0x80 as f32 / 255.0 * 0.5).abs() < 1e-6);
    }
}
#[test]
fn glass_text_keeps_its_ink_near_threshold_then_switches_on_a_clear_boundary() {
    let base = Theme::light().with_liquid_glass();
    let mut previous = Some(true);
    for gray in [144, 138, 148, 139, 146] {
        let image = crate::Image::solid(31, 29, [gray; 3]);
        let foreground = base.over_glass_with_previous_ink(&image, 1.0, None, previous);
        assert!(
            foreground.text_primary.r < 0.5,
            "minor movement must not flash white"
        );
        previous = Some(foreground.text_primary.r < 0.5);
    }
    let dark = base.over_glass_with_previous_ink(
        &crate::Image::solid(31, 29, [80; 3]),
        1.0,
        None,
        previous,
    );
    assert!(dark.text_primary.r > 0.9);
    for gray in [138, 148, 144] {
        let foreground = base.over_glass_with_previous_ink(
            &crate::Image::solid(31, 29, [gray; 3]),
            1.0,
            None,
            Some(false),
        );
        assert!(
            foreground.text_primary.r > 0.9,
            "return to the threshold must keep white"
        );
    }
    let bright = base.over_glass_with_previous_ink(
        &crate::Image::solid(31, 29, [220; 3]),
        1.0,
        None,
        Some(false),
    );
    assert!(bright.text_primary.r < 0.5);
}
