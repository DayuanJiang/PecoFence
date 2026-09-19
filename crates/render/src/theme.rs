//! Fluent and Liquid Glass design tokens for fence chrome.
//!
//! Interaction tokens follow WinUI; cool neutral surfaces and directional glass edges give
//! the desktop and settings a shared palette. Alpha is applied over the sampled backdrop.

use windows_canvas::ColorF;

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
    /// Window corner radius in DIPs. Both materials use 8, the Windows 11 top-level window
    /// radius; Liquid Glass additionally clips the composition root to it (lens and shadow).
    pub corner_radius: f32,
    /// Title bar height in DIPs.
    pub title_height: f32,
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
            title_height: 36.0,
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
            title_height: 36.0,
        }
    }

    pub const fn mica_tint(&self) -> crate::backdrop::MicaTint {
        match self.mode {
            ThemeMode::Dark => crate::backdrop::MicaTint::MICA_DARK,
            ThemeMode::Light => crate::backdrop::MicaTint::MICA_LIGHT,
        }
    }

    pub const fn acrylic_tint(&self) -> crate::backdrop::MicaTint {
        if self.liquid_glass {
            return match self.mode {
                ThemeMode::Dark => crate::backdrop::MicaTint::LIQUID_DARK,
                ThemeMode::Light => crate::backdrop::MicaTint::LIQUID_LIGHT,
            };
        }
        match self.mode {
            ThemeMode::Dark => crate::backdrop::MicaTint::ACRYLIC_DARK,
            ThemeMode::Light => crate::backdrop::MicaTint::ACRYLIC_LIGHT,
        }
    }

    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    /// Liquid Glass-inspired chrome. Keep mode and accent independent of the material.
    /// The composition root clips to this radius, including content and height animations.
    pub const fn with_liquid_glass(mut self) -> Self {
        self.liquid_glass = true;
        self.corner_radius = 8.0;
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
                self.subtle_hover = ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x60);
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
        mut self,
        image: &crate::Image,
        opacity: f32,
        tint: Option<ColorF>,
        previous_dark_text: Option<bool>,
    ) -> Self {
        if !self.liquid_glass || image.bgra.is_empty() {
            return self;
        }
        let mut sum = 0.0;
        let mut count = 0;
        let (cols, rows) = (image.width.min(16), image.height.min(16));
        for row in 0..rows {
            for col in 0..cols {
                let x = ((2 * col + 1) * image.width / (2 * cols)).min(image.width - 1);
                let y = ((2 * row + 1) * image.height / (2 * rows)).min(image.height - 1);
                let i = ((y * image.width + x) * 4) as usize;
                let p = &image.bgra[i..i + 4];
                sum += (0.2126 * p[2] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[0] as f32) / 255.0;
                count += 1;
            }
        }
        let luminance = |c: ColorF| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
        let mut brightness = sum / count as f32;
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
            Some(true) => 0.50,
            Some(false) => 0.62,
            None => 0.56,
        };
        let foreground = if brightness > threshold {
            Self::light().with_liquid_glass()
        } else {
            Self::dark().with_liquid_glass()
        };
        self.text_primary = foreground.text_primary;
        self.text_secondary = foreground.text_secondary;
        self.text_tertiary = foreground.text_tertiary;
        self.control_strong = foreground.control_strong;
        self.subtle_hover = foreground.subtle_hover;
        self.subtle_pressed = foreground.subtle_pressed;
        self.tab_active_fill = foreground.tab_active_fill;
        self.selection_fill_inactive = foreground.selection_fill_inactive;
        self.selection_stroke_inactive = foreground.selection_stroke_inactive;
        self.focus_outer = foreground.focus_outer;
        self.focus_inner = foreground.focus_inner;
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
/// separate legacy family names.
pub const FONT_TEXT: &str = "Segoe UI Variable Text";
pub const FONT_SMALL: &str = "Segoe UI Variable Small";
pub const FONT_DISPLAY: &str = "Segoe UI Variable Display";
/// Segoe Fluent Icons glyph font (chevrons, close, etc.).
pub const FONT_ICONS: &str = "Segoe Fluent Icons";

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
            assert_eq!(glass.corner_radius, 8.0);
            assert_eq!(glass.title_height, base.title_height);
            assert!(!base.liquid_glass);
        }
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
