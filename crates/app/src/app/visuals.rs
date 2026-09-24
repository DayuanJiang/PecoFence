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

fn wallpaper_snapshot(wallpaper_override: Option<&str>) -> Result<wallpaper::WallpaperSnapshot> {
    let mut snapshot = wallpaper::query()?;
    if let Some(p) = wallpaper_override {
        for m in &mut snapshot.monitors {
            m.path = Some(PathBuf::from(p));
        }
    }
    Ok(snapshot)
}

/// Build from the snapshot we fingerprinted, rather than querying a possibly newer desktop.
/// A transient decode failure leaves the last good background on screen and is retried.
fn build_backdrops(
    theme: &Theme,
    snapshot: &wallpaper::WallpaperSnapshot,
) -> Result<Vec<MonitorBackdrop>> {
    let started = std::time::Instant::now();
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
                let d = wallpaper::decode_scaled(
                    p,
                    (w as u32 / decode_divisor).max(1),
                    (h as u32 / decode_divisor).max(1),
                )?;
                Image {
                    width: d.width,
                    height: d.height,
                    bgra: d.bgra,
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
    if out.is_empty() {
        return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
            0x8000000Au32 as i32, // E_PENDING: monitor topology is not ready.
        )));
    }
    tracing::info!(
        ?position,
        elapsed_ms = started.elapsed().as_millis(),
        count = out.len(),
        "backdrops ready"
    );
    Ok(out)
}

/// All fences share one material. A missing wallpaper or the diagnostic override uses the
/// renderer's solid fallback without introducing a second user-facing material.
fn build_snapshot_backdrops(
    theme: &Theme,
    snapshot: &wallpaper::WallpaperSnapshot,
    signature: &str,
) -> Result<Rc<BackdropSets>> {
    if pecofence_core::brand::var_os("PECOFENCE_SOLID").is_some() {
        return Ok(Rc::new(BackdropSets::default()));
    }
    let backdrops = build_backdrops(theme, snapshot)?;
    if signature != snapshot.signature() {
        // Explorer finished replacing the image while WIC read it. Do not cache that read
        // under the old metadata, or a later desktop round trip could resurrect it.
        return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
            0x8000000Au32 as i32, // E_PENDING
        )));
    }
    Ok(Rc::new(BackdropSets {
        acrylic: Rc::new(backdrops),
    }))
}

pub(super) fn build_backdrop_sets(
    theme: &Theme,
    wallpaper_override: Option<&str>,
) -> Result<(Rc<BackdropSets>, String)> {
    let snapshot = wallpaper_snapshot(wallpaper_override)?;
    let signature = snapshot.signature();
    let backdrops = build_snapshot_backdrops(theme, &snapshot, &signature)?;
    Ok((backdrops, signature))
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

pub(super) fn tray_icon_image(size: i32, _accent: [u8; 3], _dark: bool) -> Vec<u8> {
    // The product mark, variant E: white corner brackets and a 2×2 grid on an indigo rounded
    // plate. Same geometry as scripts/make-msix-assets.py (64-unit mark), so the tray, the
    // settings window, the exe and the Store tiles all show one icon. Premultiplied BGRA.
    const PLATE: [f32; 3] = [0x47 as f32, 0x68 as f32, 0xDE as f32];
    const SS: i32 = 4; // supersampling per axis
    let side = size as f32;
    let plate_r = side * 0.22;
    // Mark occupies 76% of the side (12% margin), like the taskbar-size tiles.
    let unit = side * 0.76 / 64.0;
    let origin = side * 0.12;
    let sample = |sx: f32, sy: f32| -> (f32, f32) {
        // (plate coverage, mark alpha) at one sample point.
        let dx = (sx - side / 2.0).abs() - (side / 2.0 - plate_r);
        let dy = (sy - side / 2.0).abs() - (side / 2.0 - plate_r);
        let d = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() + dx.max(dy).min(0.0) - plate_r;
        let plate = (0.5 - d).clamp(0.0, 1.0);
        // Mark coordinates in 64-unit space.
        let mx = (sx - origin) / unit;
        let my = (sy - origin) / unit;
        let mut mark = 0.0f32;
        let stroke_half = 3.0;
        // Straight legs of the four brackets as capsules (round caps).
        let legs: [((f32, f32), (f32, f32)); 8] = [
            ((27.0, 8.0), (14.0, 8.0)),
            ((8.0, 14.0), (8.0, 27.0)),
            ((37.0, 8.0), (50.0, 8.0)),
            ((56.0, 14.0), (56.0, 27.0)),
            ((8.0, 37.0), (8.0, 50.0)),
            ((14.0, 56.0), (27.0, 56.0)),
            ((56.0, 37.0), (56.0, 50.0)),
            ((50.0, 56.0), (37.0, 56.0)),
        ];
        for ((ax, ay), (bx, by)) in legs {
            let (vx, vy) = (bx - ax, by - ay);
            let t = (((mx - ax) * vx + (my - ay) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
            let (cx, cy) = (ax + vx * t, ay + vy * t);
            let dist = ((mx - cx).powi(2) + (my - cy).powi(2)).sqrt() - stroke_half;
            mark = mark.max((0.5 - dist * unit).clamp(0.0, 1.0));
        }
        // Quarter arcs of radius 6 around the bracket corners, limited to their quadrant.
        let arcs: [((f32, f32), bool, bool); 4] = [
            ((14.0, 14.0), true, true),
            ((50.0, 14.0), false, true),
            ((14.0, 50.0), true, false),
            ((50.0, 50.0), false, false),
        ];
        for ((cx, cy), left, top) in arcs {
            let in_quadrant =
                (if left { mx <= cx } else { mx >= cx }) && (if top { my <= cy } else { my >= cy });
            if in_quadrant {
                let dist =
                    (((mx - cx).powi(2) + (my - cy).powi(2)).sqrt() - 6.0).abs() - stroke_half;
                mark = mark.max((0.5 - dist * unit).clamp(0.0, 1.0));
            }
        }
        // 2×2 rounded squares (8 units, radius 2); the off-diagonal pair is lighter.
        let squares: [(f32, f32, f32); 4] = [
            (22.0, 22.0, 1.0),
            (35.0, 22.0, 0.65),
            (22.0, 35.0, 0.65),
            (35.0, 35.0, 1.0),
        ];
        for (qx, qy, alpha) in squares {
            let (ccx, ccy) = (qx + 4.0, qy + 4.0);
            let ex = (mx - ccx).abs() - 2.0;
            let ey = (my - ccy).abs() - 2.0;
            let dist =
                (ex.max(0.0).powi(2) + ey.max(0.0).powi(2)).sqrt() + ex.max(ey).min(0.0) - 2.0;
            mark = mark.max((0.5 - dist * unit).clamp(0.0, 1.0) * alpha);
        }
        (plate, mark)
    };
    let mut bgra = vec![0u8; (size * size * 4) as usize];
    let inv = 1.0 / (SS * SS) as f32;
    for y in 0..size {
        for x in 0..size {
            let (mut plate, mut mark) = (0.0f32, 0.0f32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = x as f32 + (sx as f32 + 0.5) / SS as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SS as f32;
                    let (p, m) = sample(px, py);
                    plate += p;
                    mark += m * p;
                }
            }
            plate *= inv;
            mark *= inv;
            if plate <= 0.0 {
                continue;
            }
            // Composite white mark over the plate, then premultiply by the plate coverage.
            let mix = |c: f32| {
                (c * (plate - mark) + 255.0 * mark)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            let i = ((y * size + x) * 4) as usize;
            bgra[i] = mix(PLATE[2]);
            bgra[i + 1] = mix(PLATE[1]);
            bgra[i + 2] = mix(PLATE[0]);
            bgra[i + 3] = (plate * 255.0).round() as u8;
        }
    }
    bgra
}

impl App {
    /// Wallpaper-only refresh: reuse a recent desktop's pixels and preserve icon/geometry
    /// caches. Failed reads must not advance the signature, so the next check can retry.
    pub(super) fn check_wallpaper(&mut self, reason: &str) {
        let Ok(snapshot) = wallpaper_snapshot(self.wallpaper_override.as_deref()) else {
            return;
        };
        let signature = snapshot.signature();
        if self.wallpaper_sig.as_ref() == Some(&signature) {
            return;
        }
        let started = Instant::now();
        let cached = self.wallpaper_cache.get(&signature);
        let cache_hit = cached.is_some();
        let backdrops = match cached {
            Some(backdrops) => backdrops,
            None => match build_snapshot_backdrops(&self.ctx.theme.borrow(), &snapshot, &signature)
            {
                Ok(backdrops) => {
                    self.wallpaper_cache
                        .insert(signature.clone(), backdrops.clone());
                    backdrops
                }
                Err(error) => {
                    tracing::debug!(reason, %error, "wallpaper not ready; retaining last backdrop");
                    return;
                }
            },
        };
        self.wallpaper_sig = Some(signature);
        *self.ctx.backdrops.borrow_mut() = backdrops.clone();
        for window in self.fences.values() {
            window.set_backdrops(backdrops.clone());
        }
        tracing::info!(
            reason,
            cache_hit,
            elapsed_ms = started.elapsed().as_millis(),
            "wallpaper refreshed"
        );
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
                    instance: None,
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
            // Theme/DPI/layout changes invalidate cached material recipes and coordinates.
            self.wallpaper_cache.clear();
            match build_backdrop_sets(&theme, self.wallpaper_override.as_deref()) {
                Ok((backdrops, signature)) => {
                    self.wallpaper_cache
                        .insert(signature.clone(), backdrops.clone());
                    self.wallpaper_sig = Some(signature);
                    backdrops
                }
                Err(error) => {
                    tracing::warn!(%error, "backdrop refresh deferred until wallpaper is readable");
                    self.wallpaper_sig = None;
                    window::post_message(self.control.hwnd(), WM_APP_WALLPAPER, 0, 0);
                    self.ctx.backdrops.borrow().clone()
                }
            }
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

#[cfg(test)]
mod tray_icon_tests {
    use super::tray_icon_image;

    /// Every pixel is premultiplied (colour ≤ alpha) and the corners stay transparent.
    #[test]
    fn tray_icon_is_premultiplied_with_rounded_corners() {
        for size in [16, 20, 24, 32, 48] {
            let px = tray_icon_image(size, [0, 0, 0], true);
            assert_eq!(px.len(), (size * size * 4) as usize);
            for c in px.chunks(4) {
                assert!(
                    c[0] <= c[3] && c[1] <= c[3] && c[2] <= c[3],
                    "not premultiplied"
                );
            }
            assert!(
                px[3] < 40,
                "corner pixel must be (nearly) transparent at {size}"
            );
            let centre = (((size / 2) * size + size / 2) * 4) as usize;
            assert_eq!(px[centre + 3], 255, "plate must be opaque at {size}");
        }
    }

    /// Writes raw BGRA dumps for a visual check: `cargo test -p pecofence tray_icon_dump -- --ignored`.
    #[test]
    #[ignore]
    fn tray_icon_dump() {
        for size in [16, 20, 24, 32, 48, 64] {
            let px = tray_icon_image(size, [0, 0, 0], true);
            std::fs::write(format!("../../.cache/tray-{size}.bgra"), px).unwrap();
        }
    }
}
