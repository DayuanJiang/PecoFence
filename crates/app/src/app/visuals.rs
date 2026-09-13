//! Theme / accent / backdrop / icon-variant / shadow / tray-glyph helpers and visual refresh.

use super::*;

/// The theme for `mode`, retinted with the user's accent when it could be read (WinUI
/// AccentFillColorDefault: AccentLight2 on dark, AccentDark1 on light).
pub(super) fn theme_for(
    mode: ThemeMode,
    style: pecofence_core::ThemeStyle,
    accent: Option<&systheme::AccentPalette>,
) -> Theme {
    let mut theme = Theme::for_mode(mode);
    if style == pecofence_core::ThemeStyle::LiquidGlass {
        theme = theme.with_liquid_glass();
    }
    match accent {
        Some(p) => theme.with_accent(if mode == ThemeMode::Dark {
            p.light2
        } else {
            p.dark1
        }),
        None => theme,
    }
}

pub(super) fn pick_theme_mode(setting: ThemeSetting, args: &Args) -> ThemeMode {
    if args.light {
        return ThemeMode::Light;
    }
    if args.dark {
        return ThemeMode::Dark;
    }
    match setting {
        ThemeSetting::Light => ThemeMode::Light,
        ThemeSetting::Dark => ThemeMode::Dark,
        ThemeSetting::FollowAppMode => {
            if systheme::apps_use_light_theme() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            }
        }
        ThemeSetting::FollowWindowsMode => {
            if systheme::system_uses_light_theme() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            }
        }
    }
}

/// Developer knob: `PECOFENCE_ACRYLIC="tint_alpha,luminosity_opacity,blur_sigma_px"` overrides the
/// acrylic recipe so the material can be tuned without rebuilding.
fn tuned_tint(
    mut tint: pecofence_render::backdrop::MicaTint,
) -> pecofence_render::backdrop::MicaTint {
    if let Ok(spec) = pecofence_core::brand::var("PECOFENCE_ACRYLIC") {
        let parts: Vec<f32> = spec
            .split(',')
            .filter_map(|p| p.trim().parse().ok())
            .collect();
        if let [a, lum, sigma, rest @ ..] = &parts[..] {
            let (a, lum, sigma) = (*a, *lum, *sigma);
            tint.color.a = a;
            tint.luminosity_opacity = lum;
            tint.blur_sigma_dip = sigma;
            if let Some(c) = rest.first() {
                tint.chroma = *c;
            }
            tracing::info!(
                a,
                lum,
                sigma,
                chroma = tint.chroma,
                "acrylic recipe overridden via PECOFENCE_ACRYLIC"
            );
        }
    }
    tint
}

/// Builds one blurred, tinted glass wallpaper image per monitor.
pub fn build_backdrops(theme: &Theme, wallpaper_override: Option<&str>) -> Vec<MonitorBackdrop> {
    let started = std::time::Instant::now();
    let mut snapshot = match wallpaper::query() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "IDesktopWallpaper unavailable; solid backdrop");
            return Vec::new();
        }
    };
    if let Some(p) = wallpaper_override {
        for m in &mut snapshot.monitors {
            m.path = Some(PathBuf::from(p));
        }
    }
    let position = match snapshot.position {
        wallpaper::Position::Center => WallpaperPosition::Center,
        wallpaper::Position::Tile => WallpaperPosition::Tile,
        wallpaper::Position::Stretch => WallpaperPosition::Stretch,
        wallpaper::Position::Fit => WallpaperPosition::Fit,
        wallpaper::Position::Fill => WallpaperPosition::Fill,
        wallpaper::Position::Span => WallpaperPosition::Span,
    };
    // Clear glass needs wallpaper detail for refraction. Acrylic intentionally discards it.
    let downscale = if theme.liquid_glass { 1 } else { 4 };
    let infos = monitors::enumerate();
    let mut out = Vec::new();
    for m in &snapshot.monitors {
        let w = m.rect.right - m.rect.left;
        let h = m.rect.bottom - m.rect.top;
        // IDesktopWallpaper also lists disconnected outputs with an empty rectangle.
        // They must not become a spurious 1px texture over the live monitor's origin.
        if w <= 0 || h <= 0 {
            continue;
        }
        // Blur is specified in DIPs: scale to this monitor's DPI (200 % → twice the pixels).
        let (cx, cy) = (
            (m.rect.left + m.rect.right) / 2,
            (m.rect.top + m.rect.bottom) / 2,
        );
        let dpi = infos
            .iter()
            .find(|i| {
                cx >= i.bounds.left
                    && cx < i.bounds.right
                    && cy >= i.bounds.top
                    && cy < i.bounds.bottom
            })
            .map(|i| i.dpi)
            .unwrap_or(96)
            .max(96);
        let mut tint = if theme.liquid_glass {
            theme.acrylic_tint()
        } else {
            tuned_tint(theme.acrylic_tint())
        };
        tint.blur_sigma_dip *= dpi as f32 / 96.0;
        let image = match &m.path {
            Some(p) => {
                let decode_divisor = if theme.liquid_glass { 1 } else { 2 };
                match wallpaper::decode_scaled(
                    p,
                    (w as u32 / decode_divisor).max(1),
                    (h as u32 / decode_divisor).max(1),
                ) {
                    Ok(d) => Image {
                        width: d.width,
                        height: d.height,
                        bgra: d.bgra,
                    },
                    Err(e) => {
                        tracing::warn!(path = %p.display(), error = %e, "wallpaper decode failed");
                        Image::solid(1, 1, snapshot.background)
                    }
                }
            }
            None => Image::solid(1, 1, snapshot.background),
        };
        out.push(MonitorBackdrop::build(
            &image,
            position,
            m.rect.left,
            m.rect.top,
            w,
            h,
            snapshot.background,
            tint,
            downscale,
        ));
    }
    tracing::info!(
        ?position,
        elapsed_ms = started.elapsed().as_millis(),
        count = out.len(),
        "backdrops ready"
    );
    out
}

/// All fences share one material. A missing wallpaper or the diagnostic override uses the
/// renderer's solid fallback without introducing a second user-facing material.
pub(super) fn build_backdrop_sets(theme: &Theme, wallpaper_override: Option<&str>) -> BackdropSets {
    if pecofence_core::brand::var_os("PECOFENCE_SOLID").is_some() {
        return BackdropSets::default();
    }
    BackdropSets {
        acrylic: Rc::new(build_backdrops(theme, wallpaper_override)),
    }
}

pub(super) fn backdrop_mode_for(b: pecofence_core::Backdrop) -> BackdropMode {
    match b {
        pecofence_core::Backdrop::Acrylic => BackdropMode::Acrylic,
    }
}

pub(super) fn icon_variant_for(s: &pecofence_core::IconSettings) -> IconVariant {
    IconVariant {
        tint: s.tint_rgb,
        tint_strength: (s.tint_strength.clamp(0.0, 1.0) * 100.0).round() as u8,
        chameleon: s.chameleon,
    }
}

pub(super) fn fence_style_for(f: &pecofence_core::Fence) -> FenceStyle {
    let a = f.appearance.as_ref();
    let rgb = |c: [u8; 3]| pecofence_render::ColorF::from_rgba8(c[0], c[1], c[2], 0xFF);
    FenceStyle {
        tint: a.and_then(|a| a.tint_rgb).map(rgb),
        title_color: a.and_then(|a| a.title_rgb).map(rgb),
        title_size: match a.and_then(|a| a.title_size).unwrap_or_default() {
            TitleSize::Small => 0,
            TitleSize::Normal => 1,
            TitleSize::Large => 2,
        },
    }
}

pub(super) fn shadow_style_for(theme: &Theme) -> ShadowStyle {
    let mut shadow = match theme.mode {
        ThemeMode::Dark => ShadowStyle::DARK,
        ThemeMode::Light => ShadowStyle::LIGHT,
    };
    if theme.liquid_glass {
        shadow.sigma = 12.0;
        shadow.offset_y = 5.0;
        shadow.radius = theme.corner_radius;
    }
    shadow
}

pub(super) fn tray_icon_image(size: i32, accent: [u8; 3], _dark: bool) -> Vec<u8> {
    // A rounded "fence" glyph: filled rounded square with two lighter bars.
    let mut bgra = vec![0u8; (size * size * 4) as usize];
    let r = size as f32 * 0.22;
    let (w, h) = (size as f32, size as f32);
    for y in 0..size {
        for x in 0..size {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            // Rounded rect coverage (analytic-ish: distance to inner rect).
            let dx = (fx - w / 2.0).abs() - (w / 2.0 - r);
            let dy = (fy - h / 2.0).abs() - (h / 2.0 - r);
            let d = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() - r;
            let cov = (0.5 - d).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            let bar = ((fy > h * 0.30 && fy < h * 0.42) || (fy > h * 0.58 && fy < h * 0.70))
                && fx > w * 0.22
                && fx < w * 0.78;
            let (cr, cg, cb) = if bar {
                (255, 255, 255)
            } else {
                (accent[0], accent[1], accent[2])
            };
            let a = (cov * 255.0) as u32;
            let i = ((y * size + x) * 4) as usize;
            bgra[i] = (cb as u32 * a / 255) as u8;
            bgra[i + 1] = (cg as u32 * a / 255) as u8;
            bgra[i + 2] = (cr as u32 * a / 255) as u8;
            bgra[i + 3] = a as u8;
        }
    }
    bgra
}

impl App {
    /// Rebuilds the blurred backdrops when the wallpaper (picture, slide, fit, colour) changed
    /// since they were built. Cheap when nothing changed: one COM query, no decode.
    pub(super) fn check_wallpaper(&mut self, reason: &str) {
        let sig = wallpaper::signature();
        if sig == self.wallpaper_sig {
            return;
        }
        tracing::info!(reason, "wallpaper changed; rebuilding backdrops");
        self.wallpaper_sig = sig;
        self.refresh_visuals(true);
    }

    pub(super) fn on_system_settings_changed(&mut self) {
        self.refresh_language();
        self.ctx
            .motion
            .set_enabled(sysparams::client_area_animation());
        self.ctx
            .behavior
            .wheel_lines
            .set(sysparams::wheel_scroll_lines());
        self.apply_icon_title_font();
        // Any broadcast that reaches us after the work area moved (taskbar, DPI, resolution
        // races) must re-layout, or every later set_fence_bounds normalizes against a stale
        // work area and the saved geometry drifts by the taskbar height.
        let fresh = work_areas();
        if fresh != self.state.work_areas {
            tracing::info!("work areas changed behind a settings broadcast; re-laying out");
            self.on_display_changed();
            return;
        }
        self.refresh_visuals(false);
    }

    /// Icon labels follow the desktop's icon-title font (`SPI_GETICONTITLELOGFONT` times the
    /// Accessibility text-size factor, both re-read on WM_SETTINGCHANGE): when it changed,
    /// every fence refits its labels and re-lays out its grid, then settles its auto height.
    pub(super) fn apply_icon_title_font(&mut self) {
        let Some(f) = sysparams::icon_title_font() else {
            return;
        };
        let changed = self
            .ctx
            .chrome
            .set_icon_title_font(
                &f.family,
                f.size_dip,
                pecofence_render::FontWeight(f.weight),
            )
            .unwrap_or(false);
        if !changed {
            return;
        }
        tracing::info!(family = %f.family, size = f.size_dip, "icon-title font changed");
        let ids: Vec<FenceId> = self.fences.keys().copied().collect();
        for id in ids {
            if let Some(w) = self.fences.get(&id) {
                w.on_icon_font_changed();
            }
            self.apply_auto_height(id);
        }
    }

    pub(super) fn refresh_visuals(&mut self, force: bool) {
        let mode = self.theme_override.unwrap_or_else(|| {
            pick_theme_mode(
                self.state.config.settings.theme,
                &Args {
                    light: false,
                    dark: false,
                    wallpaper_override: None,
                    portable: false,
                    no_hide_icons: false,
                    exit_after_ms: None,
                    dump_stats: false,
                    open_settings: false,
                    portal: None,
                    test_script: None,
                },
            )
        });
        let accent = systheme::accent_palette();
        let accent_changed = accent != self.accent;
        let style = self.state.config.settings.theme_style;
        let style_changed = self.ctx.theme.borrow().liquid_glass
            != (style == pecofence_core::ThemeStyle::LiquidGlass);
        if mode == self.theme_mode && !style_changed && !accent_changed && !force {
            return;
        }
        let theme_changed = mode != self.theme_mode || style_changed;
        let theme = theme_for(mode, style, accent.as_ref());
        // The wallpaper sets are accent-independent and expensive: rebuild them only for a
        // mode change (or a forced refresh), not for a Settings › Colours retint.
        let backdrops = if theme_changed || force {
            Rc::new(build_backdrop_sets(
                &theme,
                self.wallpaper_override.as_deref(),
            ))
        } else {
            self.ctx.backdrops.borrow().clone()
        };
        self.theme_mode = mode;
        self.accent = accent;
        *self.ctx.theme.borrow_mut() = theme;
        *self.ctx.backdrops.borrow_mut() = backdrops.clone();
        // Icon bitmaps are accent-independent: an accent-only change (Settings > Colours)
        // keeps them; a mode change or a forced refresh re-extracts them.
        if theme_changed || force {
            self.ctx.bitmaps.borrow_mut().clear();
        }
        let shadow = shadow_style_for(&theme);
        self.ctx.shadow_style.set(shadow);
        if let Some(h) = &self.settings {
            if theme_changed {
                h.set_theme(mode, theme.liquid_glass);
            }
            self.push_settings_state();
        }
        for w in self.fences.values() {
            w.set_theme(theme, backdrops.clone(), shadow);
        }
        tracing::info!(?mode, ?style, accent = ?accent.map(|a| a.accent), "theme refreshed");
    }

    /// Global icon tint / chameleon changed: flush the caches and let icons reload processed.
    pub(super) fn apply_icon_variant(&mut self) {
        let variant = icon_variant_for(&self.state.config.settings.icons);
        if self.ctx.icons.borrow_mut().set_variant(variant) {
            self.ctx.bitmaps.borrow_mut().clear();
            for w in self.fences.values() {
                w.drop_icons();
            }
        }
    }
}
