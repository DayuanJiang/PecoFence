//! Material regression render over the real wallpaper, without capturing desktop windows.
//! Args: output.bmp left top width height dpi-scale [gpu]
use pecofence_platform::{com::OleGuard, wallpaper, window};
use pecofence_render::{
    BitmapCache, Image, Matrix3x2, MonitorBackdrop, Theme, WallpaperPosition,
    fence_chrome::{
        Backdrop, BackdropCrop, FenceChrome, FenceStyle, TabDraw, TitleDeco, TitleState,
    },
    gpu_glass::GpuGlass,
    liquid_glass::{self, GlassOptics},
};
use std::{cell::RefCell, io::Write, path::Path, rc::Rc};
use windows_canvas::GpuDevice;

fn save_bmp(path: &Path, image: &Image) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(54 + image.bgra.len() as u32).to_le_bytes())?;
    f.write_all(&[0; 4])?;
    f.write_all(&54u32.to_le_bytes())?;
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(image.width as i32).to_le_bytes())?;
    f.write_all(&(-(image.height as i32)).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&32u16.to_le_bytes())?;
    f.write_all(&[0; 24])?;
    f.write_all(&image.bgra)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    window::set_process_dpi_awareness_v2();
    let _ole = OleGuard::init()?;
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .get(1)
        .map(String::as_str)
        .unwrap_or(".cache/material.bmp");
    let arg =
        |i: usize, fallback: i32| args.get(i).and_then(|v| v.parse().ok()).unwrap_or(fallback);
    let (left, top, w, h) = (arg(2, 734), arg(3, 1652), arg(4, 800), arg(5, 298));
    let scale: f32 = args.get(6).and_then(|v| v.parse().ok()).unwrap_or(2.0);
    let snapshot = wallpaper::query()?;
    let monitor = snapshot
        .monitors
        .iter()
        .find(|m| {
            left >= m.rect.left
                && top >= m.rect.top
                && left + w <= m.rect.right
                && top + h <= m.rect.bottom
        })
        .ok_or("Requested plate must fit one monitor")?;
    let path = monitor.path.as_ref().ok_or("Picture wallpaper required")?;
    let mw = monitor.rect.right - monitor.rect.left;
    let mh = monitor.rect.bottom - monitor.rect.top;
    let decoded = wallpaper::decode_scaled(path, mw as u32, mh as u32)?;
    let image = Image {
        width: decoded.width,
        height: decoded.height,
        bgra: decoded.bgra,
    };
    let position = match snapshot.position {
        wallpaper::Position::Center => WallpaperPosition::Center,
        wallpaper::Position::Tile => WallpaperPosition::Tile,
        wallpaper::Position::Stretch => WallpaperPosition::Stretch,
        wallpaper::Position::Fit => WallpaperPosition::Fit,
        wallpaper::Position::Fill => WallpaperPosition::Fill,
        wallpaper::Position::Span => WallpaperPosition::Span,
    };
    let theme = Theme::dark().with_liquid_glass();
    let background = MonitorBackdrop::build(
        &image,
        position,
        monitor.rect.left,
        monitor.rect.top,
        mw,
        mh,
        snapshot.background,
        theme.acrylic_tint(),
        1,
    );
    let sources = Rc::new(vec![background]);
    let background = &sources[0];
    let gpu_material = args.get(7).is_some_and(|s| s == "gpu");
    let refracted = if gpu_material {
        liquid_glass::foreground_sample(&sources, [left, top, w, h])
    } else {
        liquid_glass::crop(background, [left, top, w, h], scale, GlassOptics::default())
    };
    let theme = theme.over_glass(&refracted);
    let gpu = GpuDevice::new_or_warp()?;
    let target = gpu.create_render_target(w as u32, h as u32)?;
    let chrome = FenceChrome::new()?;
    let material = RefCell::new(GpuGlass::default());
    let tab_mode = args.get(8).map(String::as_str).unwrap_or("");
    let title_size = args.get(9).and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);
    let tabs: Vec<_> = if tab_mode.is_empty() {
        Vec::new()
    } else {
        let mut x = 8.0;
        ["文件与文档", "图片"]
            .into_iter()
            .enumerate()
            .map(|(i, title)| {
                let w = pecofence_render::text::measure_width(title, chrome.tab_format(title_size))
                    + 24.0;
                let tab = TabDraw {
                    key: 100 + i as u128,
                    x,
                    w,
                    title,
                    title_size,
                    title_color: None,
                    active: i == if tab_mode == "second" { 1 } else { 0 },
                    hover: if tab_mode == "hover" && i == 1 {
                        1.0
                    } else {
                        0.0
                    },
                    pressed: tab_mode == "pressed" && i == 0,
                    color: None,
                    drop_target: if tab_mode == "drop" && i == 1 {
                        1.0
                    } else {
                        0.0
                    },
                    dragging: false,
                    alpha: 1.0,
                };
                x += w + 4.0;
                tab
            })
            .collect()
    };
    target.draw(|session| {
        session.set_transform(&Matrix3x2 {
            m11: scale,
            m22: scale,
            ..Matrix3x2::identity()
        });
        chrome.draw(
            session,
            &mut BitmapCache::new(),
            &theme,
            scale,
            w as f32 / scale,
            h as f32 / scale,
            if gpu_material {
                Backdrop::GpuGlass {
                    material: &material,
                    wallpaper: &sources,
                    rect: [left, top, w, h],
                    scale,
                }
            } else {
                Backdrop::Glass(
                    BackdropCrop {
                        image: &refracted,
                        key: "plate",
                        height: h as f32 / scale,
                    },
                    scale,
                )
            },
            "文件与文档",
            None,
            false,
            0.0,
            1.0,
            &tabs,
            None,
            TitleDeco::default(),
            TitleState {
                title: 1.0,
                ..Default::default()
            },
            FenceStyle {
                title_size,
                ..Default::default()
            },
        )
    })?;
    let plate = target.read_pixels()?;
    let pad = 24;
    let mut result = background.crop_screen_rect(left - pad, top - pad, w + 2 * pad, h + 2 * pad);
    if (result.width, result.height) != ((w + 2 * pad) as u32, (h + 2 * pad) as u32) {
        return Err("Requested plate needs 24px of wallpaper around it".into());
    }
    let radius = (theme.corner_radius * scale).min(w.min(h) as f32 * 0.5);
    for y in 0..h {
        for x in 0..w {
            let qx = (x as f32 + 0.5 - w as f32 * 0.5).abs() - (w as f32 * 0.5 - radius);
            let qy = (y as f32 + 0.5 - h as f32 * 0.5).abs() - (h as f32 * 0.5 - radius);
            let distance = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
            let coverage = (0.5 - distance).clamp(0.0, 1.0);
            let src = ((y * w + x) * 4) as usize;
            let dst = (((y + pad) as u32 * result.width + (x + pad) as u32) * 4) as usize;
            let alpha = plate[src + 3] as f32 / 255.0 * coverage;
            for c in 0..3 {
                result.bgra[dst + c] = (plate[src + c] as f32 * coverage
                    + result.bgra[dst + c] as f32 * (1.0 - alpha))
                    .round() as u8;
            }
        }
    }
    save_bmp(Path::new(output), &result)?;
    println!("Saved {output}");
    Ok(())
}
