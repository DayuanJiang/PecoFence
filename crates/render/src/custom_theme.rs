//! Custom themes: a JSON file in the configuration's `themes` folder that starts from Fluent
//! or Liquid Glass and replaces some of its values. Keys at the top level apply to both
//! colour modes; the same keys inside `light` or `dark` apply to that mode and win. Fonts are
//! top-level only.
//!
//! ```json
//! { "base": "liquidGlass", "blur": 6, "light": { "tint": "#FFFFFF14" } }
//! ```

use crate::theme::{Ink, TextFx, Theme, ThemeMode};
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
    corner_radius: Option<f32>,
    shadow_opacity: Option<f32>,
    shadow_blur: Option<f32>,
    shadow_offset: Option<f32>,
    accent: Option<ColorF>,
    title_color: Option<ColorF>,
    /// Halo and soft shadow of text over bare wallpaper, unclamped.
    text_fx: TextFx,
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
    /// and the light and dark ink the text switches between, each with its halo and shadow.
    pub fn apply(&self, theme: Theme) -> Theme {
        let mut out = self.with_values(theme);
        let variants = [ThemeMode::Light, ThemeMode::Dark].map(|mode| {
            let mut variant = Theme::for_mode(mode);
            if theme.liquid_glass {
                variant = variant.with_liquid_glass();
            }
            self.with_values(variant)
        });
        out.custom_inks = Some(variants.map(|v| Ink::of(&v)));
        out.text_fx = [variants[0].text_fx[0], variants[1].text_fx[1]];
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
                "titleColor" => values.title_color = Some(color()?),
                "textHalo" => values.text_fx.halo = Some(color()?),
                "textHaloWidth" => values.text_fx.halo_width = Some(number()?),
                "textShadow" => values.text_fx.shadow = Some(color()?),
                "textShadowBlur" => values.text_fx.shadow_blur = Some(number()?),
                "textShadowOffset" => values.text_fx.shadow_offset = Some(number()?),
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
        if let Some(c) = self.title_color {
            theme.title_text = Some(c);
        }
        // The text of this mode is the light mode's dark ink or the dark mode's light one.
        let fx = &mut theme.text_fx[(theme.mode == ThemeMode::Dark) as usize];
        let own = self.text_fx;
        fx.halo = own.halo.or(fx.halo);
        fx.shadow = own.shadow.or(fx.shadow);
        if let Some(v) = own.halo_width {
            fx.halo_width = Some(v.clamp(0.0, 3.0));
        }
        if let Some(v) = own.shadow_blur {
            fx.shadow_blur = Some(v.clamp(0.0, 8.0));
        }
        if let Some(v) = own.shadow_offset {
            fx.shadow_offset = Some(v.clamp(0.0, 4.0));
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
    fn a_fluent_theme_sets_its_corner_radius() {
        let file = CustomTheme::parse(r#"{ "base": "fluent", "cornerRadius": 14 }"#).unwrap();
        assert_eq!(file.apply(Theme::light()).corner_radius, 14.0);
        let plain = CustomTheme::parse(r#"{ "base": "fluent" }"#).unwrap();
        assert_eq!(plain.apply(Theme::light()).corner_radius, 8.0);
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
    fn title_colour_follows_the_ink_and_gives_way_to_a_fixed_label_colour() {
        use pecofence_core::LabelColor;
        let file = CustomTheme::parse(
            r##"{ "base": "liquidGlass",
                  "light": { "titleColor": "#5A3A10" },
                  "dark": { "titleColor": "#FFD890" } }"##,
        )
        .unwrap();
        let theme = file.apply(Theme::dark().with_liquid_glass());
        let on_bright = theme.over_glass(&crate::Image::solid(30, 30, [235; 3]));
        let on_dark = theme.over_glass(&crate::Image::solid(30, 30, [20; 3]));
        let primary = on_bright.text_primary;
        assert_eq!(
            on_bright.title_ink(primary),
            ColorF::from_rgba8(0x5A, 0x3A, 0x10, 0xFF)
        );
        assert_eq!(
            on_dark.title_ink(primary),
            ColorF::from_rgba8(0xFF, 0xD8, 0x90, 0xFF)
        );
        // 「图标名称颜色」 白色 / 黑色 wins over the theme's title colour.
        let white = on_bright.with_label_color(LabelColor::White);
        assert_eq!(white.title_ink(white.text_primary), white.text_primary);
        assert_eq!(
            on_bright
                .with_label_color(LabelColor::Auto)
                .title_ink(primary),
            ColorF::from_rgba8(0x5A, 0x3A, 0x10, 0xFF)
        );
        // Fluent follows the colour mode; without the key the title is the primary text.
        let fluent = CustomTheme::parse(r##"{ "base": "fluent", "titleColor": "#102030" }"##)
            .unwrap()
            .apply(Theme::light());
        assert_eq!(
            fluent.title_ink(fluent.text_primary),
            ColorF::from_rgba8(0x10, 0x20, 0x30, 0xFF)
        );
        assert_eq!(
            Theme::light().title_ink(ColorF::TRANSPARENT),
            ColorF::TRANSPARENT
        );
    }

    #[test]
    fn text_halo_and_shadow_pair_with_the_ink_and_are_clamped() {
        let file = CustomTheme::parse(
            r##"{ "base": "liquidGlass", "textHaloWidth": 9, "textShadowBlur": -1,
                  "light": { "textHalo": "#FFFFFF40" },
                  "dark": { "textShadow": "#00000099", "textShadowOffset": 12,
                            "textHaloWidth": 0 } }"##,
        )
        .unwrap();
        // The same pair in either colour mode: the text, not the mode, picks the values.
        for base in [Theme::light(), Theme::dark()] {
            let theme = file.apply(base.with_liquid_glass());
            let dark_text = theme.text_fx(ColorF::from_rgba8(0x13, 0x20, 0x36, 0xFF));
            let light_text = theme.text_fx(ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF));
            assert_eq!(
                dark_text,
                TextFx {
                    halo: Some(ColorF::from_rgba8(0xFF, 0xFF, 0xFF, 0x40)),
                    halo_width: Some(3.0),
                    shadow: None,
                    shadow_blur: Some(0.0),
                    shadow_offset: None,
                }
            );
            assert_eq!(
                light_text,
                TextFx {
                    halo: None,
                    halo_width: Some(0.0),
                    shadow: Some(ColorF::from_rgba8(0, 0, 0, 0x99)),
                    shadow_blur: Some(0.0),
                    shadow_offset: Some(4.0),
                }
            );
        }
        let plain = CustomTheme::parse(r#"{ "base": "fluent" }"#).unwrap();
        assert_eq!(plain.apply(Theme::dark()).text_fx, [TextFx::NONE; 2]);
        assert_eq!(Theme::dark().text_fx, [TextFx::NONE; 2]);
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
            (
                r#"{ "base": "fluent", "dark": { "titleColor": "gold" } }"#,
                "\"dark.titleColor\" must be a colour",
            ),
            (
                r#"{ "base": "fluent", "textShadowBlur": "soft" }"#,
                "\"textShadowBlur\" must be a number",
            ),
            (
                r#"{ "base": "fluent", "textHalo": 1 }"#,
                "\"textHalo\" must be a colour",
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
