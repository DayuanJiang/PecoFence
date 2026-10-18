//! Custom themes: a JSON file in the configuration's `themes` folder that starts from Fluent
//! or Liquid Glass and replaces some of its values. Keys at the top level apply to both
//! colour modes; the same keys inside `light` or `dark` apply to that mode and win. Fonts are
//! top-level only.
//!
//! ```json
//! { "base": "liquidGlass", "blur": 6, "light": { "tint": "#FFFFFF14" } }
//! ```

use crate::theme::{Ink, Theme, ThemeMode};
use pecofence_core::ThemeStyle;
use serde_json::{Map, Value};
use windows_canvas::ColorF;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomTheme {
    pub base: ThemeStyle,
    pub fonts: ThemeFonts,
    both: Values,
    light: Values,
    dark: Values,
}

/// One scope of a theme file. Unset values keep the base style's.
#[derive(Clone, Debug, Default, PartialEq)]
struct Values {
    blur: Option<f32>,
    tint: Option<ColorF>,
    luminosity: Option<f32>,
    chroma: Option<f32>,
    noise: Option<f32>,
    refraction: Option<f32>,
    rim_light: Option<f32>,
    /// A Fluent fence then rounds its own window instead of leaving it to Windows.
    corner_radius: Option<f32>,
    shadow_opacity: Option<f32>,
    shadow_blur: Option<f32>,
    shadow_offset: Option<f32>,
    accent: Option<ColorF>,
    /// Indexes into [`COLORS`], in file order.
    colors: Vec<(usize, ColorF)>,
}

/// A theme file's fonts; None keeps the built-in choice.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemeFonts {
    /// Item names (built in: the Windows desktop icon font).
    pub label_family: Option<String>,
    /// Item name em size in DIPs at 100 % text size.
    pub label_size: Option<f32>,
    pub label_weight: Option<i32>,
    /// Titles and tabs (built in: Segoe UI Variable, semibold).
    pub title_family: Option<String>,
    pub title_weight: Option<i32>,
}

impl ThemeFonts {
    fn take(map: &mut Map<String, Value>) -> Result<Self, String> {
        let mut text = |key: &str| match map.remove(key) {
            None => Ok(None),
            Some(Value::String(s)) if !s.trim().is_empty() => Ok(Some(s.trim().to_string())),
            Some(_) => Err(format!(r#""{key}" must be a font name"#)),
        };
        let label_family = text("labelFont")?;
        let title_family = text("titleFont")?;
        let mut number = |key: &str| match map.remove(key) {
            None => Ok(None),
            Some(v) => v
                .as_f64()
                .map(|n| Some(n as f32))
                .ok_or_else(|| format!(r#""{key}" must be a number"#)),
        };
        let weight = |v: f32| (v.round() as i32).clamp(100, 900);
        Ok(Self {
            label_family,
            label_size: number("labelFontSize")?.map(|v| v.clamp(8.0, 24.0)),
            label_weight: number("labelFontWeight")?.map(weight),
            title_family,
            title_weight: number("titleFontWeight")?.map(weight),
        })
    }
}

type Token = fn(&mut Theme) -> &mut ColorF;

/// The colour tokens a theme file can set, by key.
const COLORS: &[(&str, Token)] = &[
    ("solidFill", |t| &mut t.solid_fill),
    ("layerFill", |t| &mut t.layer_fill),
    ("glassLayerFill", |t| &mut t.glass_layer_fill),
    ("glassRimTop", |t| &mut t.glass_rim_top),
    ("glassRimBottom", |t| &mut t.glass_rim_bottom),
    ("stroke", |t| &mut t.stroke),
    ("textPrimary", |t| &mut t.text_primary),
    ("textSecondary", |t| &mut t.text_secondary),
    ("textTertiary", |t| &mut t.text_tertiary),
    ("controlStrong", |t| &mut t.control_strong),
    ("tabActiveFill", |t| &mut t.tab_active_fill),
    ("subtleHover", |t| &mut t.subtle_hover),
    ("subtlePressed", |t| &mut t.subtle_pressed),
    ("selectionFill", |t| &mut t.selection_fill),
    ("selectionStroke", |t| &mut t.selection_stroke),
    ("selectionFillInactive", |t| &mut t.selection_fill_inactive),
    ("selectionStrokeInactive", |t| {
        &mut t.selection_stroke_inactive
    }),
    ("focusOuter", |t| &mut t.focus_outer),
    ("focusInner", |t| &mut t.focus_inner),
    ("dropHighlight", |t| &mut t.drop_highlight),
];

impl CustomTheme {
    /// Reads a theme file. The error names the key that is wrong.
    pub fn parse(text: &str) -> Result<Self, String> {
        // Notepad and other editors may start the file with a byte order mark.
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        let mut map: Map<String, Value> = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let base = match map.remove("base") {
            Some(Value::String(s)) if s == "fluent" => ThemeStyle::Fluent,
            Some(Value::String(s)) if s == "liquidGlass" => ThemeStyle::LiquidGlass,
            _ => return Err(r#""base" must be "fluent" or "liquidGlass""#.into()),
        };
        let mut scope = |key: &str| match map.remove(key) {
            None => Ok(Values::default()),
            Some(Value::Object(m)) => Values::parse(m, &format!("{key}.")),
            Some(_) => Err(format!(r#""{key}" must be an object"#)),
        };
        let light = scope("light")?;
        let dark = scope("dark")?;
        let fonts = ThemeFonts::take(&mut map)?;
        Ok(Self {
            base,
            fonts,
            both: Values::parse(map, "")?,
            light,
            dark,
        })
    }

    /// `theme` (the base style in its mode, with the system accent) with this file's values,
    /// and the light and dark ink the text switches between.
    pub fn apply(&self, theme: Theme) -> Theme {
        let mut out = self.with_values(theme);
        out.custom_inks = Some([ThemeMode::Light, ThemeMode::Dark].map(|mode| {
            let mut variant = Theme::for_mode(mode);
            if theme.liquid_glass {
                variant = variant.with_liquid_glass();
            }
            Ink::of(&self.with_values(variant))
        }));
        out
    }

    fn with_values(&self, theme: Theme) -> Theme {
        let own = match theme.mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        };
        own.apply(self.both.apply(theme))
    }
}

impl Values {
    fn parse(map: Map<String, Value>, scope: &str) -> Result<Self, String> {
        let mut values = Self::default();
        for (key, value) in map {
            let name = format!("{scope}{key}");
            let number = || {
                value
                    .as_f64()
                    .map(|n| n as f32)
                    .ok_or_else(|| format!(r#""{name}" must be a number"#))
            };
            let color = || {
                value.as_str().and_then(hex_color).ok_or_else(|| {
                    format!(r##""{name}" must be a colour like "#RRGGBB" or "#RRGGBBAA""##)
                })
            };
            match key.as_str() {
                "blur" => values.blur = Some(number()?),
                "tint" => values.tint = Some(color()?),
                "luminosity" => values.luminosity = Some(number()?),
                "chroma" => values.chroma = Some(number()?),
                "noise" => values.noise = Some(number()?),
                "refraction" => values.refraction = Some(number()?),
                "rimLight" => values.rim_light = Some(number()?),
                "cornerRadius" => values.corner_radius = Some(number()?),
                "shadowOpacity" => values.shadow_opacity = Some(number()?),
                "shadowBlur" => values.shadow_blur = Some(number()?),
                "shadowOffset" => values.shadow_offset = Some(number()?),
                "accent" => values.accent = Some(color()?),
                _ => match COLORS.iter().position(|(k, _)| *k == key) {
                    Some(i) => values.colors.push((i, color()?)),
                    None => return Err(format!(r#"unknown key "{name}""#)),
                },
            }
        }
        Ok(values)
    }

    fn apply(&self, mut theme: Theme) -> Theme {
        let m = &mut theme.material;
        if let Some(v) = self.blur {
            m.blur_sigma_dip = v.clamp(0.0, 60.0);
        }
        if let Some(c) = self.tint {
            m.color = c;
        }
        if let Some(v) = self.luminosity {
            m.luminosity_opacity = v.clamp(0.0, 1.0);
        }
        if let Some(v) = self.chroma {
            m.chroma = v.clamp(0.0, 1.5);
        }
        if let Some(v) = self.noise {
            m.noise = v.clamp(0.0, 0.2);
        }
        if let Some(v) = self.refraction {
            theme.lens.refraction = v.clamp(0.0, 2.0);
        }
        if let Some(v) = self.rim_light {
            theme.lens.rim_light = v.clamp(0.0, 3.0);
        }
        if let Some(v) = self.corner_radius {
            theme.corner_radius = v.clamp(0.0, 32.0);
            theme.own_corners = true;
        }
        if let Some(v) = self.shadow_opacity {
            theme.shadow.opacity = Some(v.clamp(0.0, 1.0));
        }
        if let Some(v) = self.shadow_blur {
            theme.shadow.blur = Some(v.clamp(0.0, 24.0));
        }
        if let Some(v) = self.shadow_offset {
            theme.shadow.offset = Some(v.clamp(0.0, 16.0));
        }
        if let Some(c) = self.accent {
            theme = theme.with_accent(rgb8(c));
        }
        for &(i, c) in &self.colors {
            *(COLORS[i].1)(&mut theme) = c;
        }
        theme
    }
}

/// `#RRGGBB` or `#RRGGBBAA`.
fn hex_color(text: &str) -> Option<ColorF> {
    let digits = text.strip_prefix('#')?;
    if !matches!(digits.len(), 6 | 8) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    let a = if digits.len() == 8 { byte(6)? } else { 0xFF };
    Some(ColorF::from_rgba8(byte(0)?, byte(2)?, byte(4)?, a))
}

fn rgb8(c: ColorF) -> [u8; 3] {
    let v = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    [v(c.r), v(c.g), v(c.b)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_apply_to_both_modes_and_the_mode_scope_wins() {
        let file = CustomTheme::parse(
            r##"{
                "base": "liquidGlass",
                "blur": 6,
                "refraction": 0.9,
                "textPrimary": "#102030",
                "light": { "tint": "#FFFFFF14", "textPrimary": "#000000" },
                "dark": { "glassLayerFill": "#38383D38" }
            }"##,
        )
        .unwrap();
        assert_eq!(file.base, ThemeStyle::LiquidGlass);
        let light = file.apply(Theme::light().with_liquid_glass());
        let dark = file.apply(Theme::dark().with_liquid_glass());
        for theme in [light, dark] {
            assert_eq!(theme.material.blur_sigma_dip, 6.0);
            assert_eq!(theme.lens.refraction, 0.9);
            assert_eq!(theme.lens.rim_light, 1.0);
        }
        assert_eq!(light.text_primary, ColorF::from_rgba8(0, 0, 0, 0xFF));
        assert_eq!(
            dark.text_primary,
            ColorF::from_rgba8(0x10, 0x20, 0x30, 0xFF)
        );
        assert_eq!(
            light.material.color,
            ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x14)
        );
        assert_eq!(
            dark.material.color,
            Theme::dark().with_liquid_glass().material.color
        );
        assert_eq!(
            dark.glass_layer_fill,
            ColorF::from_rgba8(0x38, 0x38, 0x3D, 0x38)
        );
        assert_eq!(
            light.glass_layer_fill,
            Theme::light().with_liquid_glass().glass_layer_fill
        );
    }

    #[test]
    fn automatic_text_colour_uses_the_theme_s_own_ink() {
        let file = CustomTheme::parse(
            r##"{ "base": "liquidGlass",
                  "light": { "textPrimary": "#203040" },
                  "dark": { "textPrimary": "#F0E0D0" } }"##,
        )
        .unwrap();
        let theme = file.apply(Theme::dark().with_liquid_glass());
        let on_bright = theme.over_glass(&crate::Image::solid(30, 30, [235; 3]));
        let on_dark = theme.over_glass(&crate::Image::solid(30, 30, [20; 3]));
        assert_eq!(
            on_bright.text_primary,
            ColorF::from_rgba8(0x20, 0x30, 0x40, 0xFF)
        );
        assert_eq!(
            on_dark.text_primary,
            ColorF::from_rgba8(0xF0, 0xE0, 0xD0, 0xFF)
        );
        let black = theme.with_label_color(pecofence_core::LabelColor::Black);
        assert_eq!(black.text_primary, on_bright.text_primary);
    }

    #[test]
    fn accent_retints_the_selection_and_explicit_tokens_win() {
        let file = CustomTheme::parse(
            r##"{ "base": "fluent", "accent": "#FF8800", "selectionStroke": "#11223344" }"##,
        )
        .unwrap();
        let theme = file.apply(Theme::light().with_accent([0, 95, 184]));
        assert_eq!(theme.accent_rgb8(), [0xFF, 0x88, 0x00]);
        assert_eq!(theme.selection_fill.a, Theme::light().selection_fill.a);
        assert!((theme.selection_fill.r - 1.0).abs() < 1e-6);
        assert_eq!(
            theme.selection_stroke,
            ColorF::from_rgba8(0x11, 0x22, 0x33, 0x44)
        );
        assert!(!theme.liquid_glass);
    }

    #[test]
    fn values_out_of_range_are_clamped() {
        let file = CustomTheme::parse(
            r#"{ "base": "liquidGlass", "blur": 500, "cornerRadius": -3, "rimLight": 9 }"#,
        )
        .unwrap();
        let theme = file.apply(Theme::dark().with_liquid_glass());
        assert_eq!(theme.material.blur_sigma_dip, 60.0);
        assert_eq!(theme.corner_radius, 0.0);
        assert_eq!(theme.lens.rim_light, 3.0);
    }

    #[test]
    fn a_fluent_theme_with_corners_rounds_its_own_window() {
        assert!(!Theme::light().own_corners && Theme::light().with_liquid_glass().own_corners);
        let file = CustomTheme::parse(r#"{ "base": "fluent", "cornerRadius": 14 }"#).unwrap();
        let theme = file.apply(Theme::light());
        assert_eq!(theme.corner_radius, 14.0);
        assert!(theme.own_corners);
        let plain = CustomTheme::parse(r#"{ "base": "fluent" }"#).unwrap();
        assert!(
            !plain.apply(Theme::light()).own_corners,
            "Windows keeps its corners"
        );
    }

    #[test]
    fn shadow_values_apply_per_mode_and_are_clamped() {
        let file = CustomTheme::parse(
            r#"{ "base": "fluent", "shadowBlur": 30, "shadowOffset": 2,
                 "light": { "shadowOpacity": 0.1 }, "dark": { "shadowOpacity": 3 } }"#,
        )
        .unwrap();
        let light = file.apply(Theme::light()).shadow;
        let dark = file.apply(Theme::dark()).shadow;
        assert_eq!(
            (light.opacity, light.blur, light.offset),
            (Some(0.1), Some(24.0), Some(2.0))
        );
        assert_eq!(dark.opacity, Some(1.0));
        assert_eq!(Theme::dark().shadow, crate::theme::ShadowOverride::NONE);
    }

    #[test]
    fn fonts_come_from_the_top_level() {
        let file = CustomTheme::parse(
            r#"{ "base": "fluent", "labelFont": " Microsoft YaHei UI ", "labelFontSize": 40,
                 "labelFontWeight": 650.4, "titleFont": "Georgia" }"#,
        )
        .unwrap();
        assert_eq!(
            file.fonts,
            ThemeFonts {
                label_family: Some("Microsoft YaHei UI".into()),
                label_size: Some(24.0),
                label_weight: Some(650),
                title_family: Some("Georgia".into()),
                title_weight: None,
            }
        );
        let plain = CustomTheme::parse(r#"{ "base": "fluent" }"#).unwrap();
        assert_eq!(plain.fonts, ThemeFonts::default());
    }

    #[test]
    fn errors_name_the_wrong_key() {
        for (text, needle) in [
            (r#"{ "blur": 4 }"#, "\"base\""),
            (r#"{ "base": "glass" }"#, "\"base\""),
            (
                r#"{ "base": "fluent", "blurr": 4 }"#,
                "unknown key \"blurr\"",
            ),
            (
                r##"{ "base": "fluent", "dark": { "txtPrimary": "#FFFFFF" } }"##,
                "\"dark.txtPrimary\"",
            ),
            (
                r#"{ "base": "fluent", "blur": "6" }"#,
                "\"blur\" must be a number",
            ),
            (
                r#"{ "base": "fluent", "light": { "tint": "white" } }"#,
                "\"light.tint\" must be a colour",
            ),
            (
                r##"{ "base": "fluent", "stroke": "#12345" }"##,
                "\"stroke\" must be a colour",
            ),
            (
                r#"{ "base": "fluent", "light": 1 }"#,
                "\"light\" must be an object",
            ),
            (
                r#"{ "base": "fluent", "light": { "titleFont": "Arial" } }"#,
                "unknown key \"light.titleFont\"",
            ),
            (
                r#"{ "base": "fluent", "titleFont": 3 }"#,
                "\"titleFont\" must be a font name",
            ),
            (
                r#"{ "base": "fluent", "labelFontSize": "big" }"#,
                "\"labelFontSize\" must be a number",
            ),
            (r#"{ "base": "fluent", "#, "line 1"),
        ] {
            let error = CustomTheme::parse(text).unwrap_err();
            assert!(error.contains(needle), "{text}: {error}");
        }
    }

    #[test]
    fn an_empty_theme_is_its_base_style() {
        let file = CustomTheme::parse(r#"{ "base": "liquidGlass" }"#).unwrap();
        let base = Theme::light().with_liquid_glass();
        let theme = file.apply(base);
        assert_eq!(theme.material, base.material);
        assert_eq!(theme.text_primary, base.text_primary);
        assert_eq!(theme.corner_radius, base.corner_radius);
        let image = crate::Image::solid(30, 30, [235; 3]);
        assert_eq!(
            theme.over_glass(&image).text_primary,
            base.over_glass(&image).text_primary
        );
    }
}
