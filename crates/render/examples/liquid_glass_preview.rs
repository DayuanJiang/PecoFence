//! Headless material study using the production crop and Direct2D fence chrome.
//! cargo run -p pecofence-render --example liquid_glass_preview -- .cache/liquid-glass-native.bmp
//! No desktop capture, configuration writes or visible windows.

use pecofence_render::{
    BitmapCache, ColorF, Image, MonitorBackdrop, Rect, RoundedRect, Theme, Vector2,
    fence_chrome::{Backdrop, BackdropCrop, FenceChrome, FenceStyle, TitleDeco, TitleState},
    liquid_glass::{self, GlassOptics},
};
use std::{io::Write, path::Path, time::Instant};
use windows_canvas::{FontWeight, GpuDevice, GradientStop, TextFormat};

fn save_bmp(path: &Path, width: u32, height: u32, pixels: &[u8]) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(54 + pixels.len() as u32).to_le_bytes())?;
    f.write_all(&[0; 4])?;
    f.write_all(&54u32.to_le_bytes())?;
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(width as i32).to_le_bytes())?;
    f.write_all(&(-(height as i32)).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&32u16.to_le_bytes())?;
    f.write_all(&[0; 24])?;
    f.write_all(pixels)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ole = pecofence_platform::com::OleGuard::init()?;
    let stack = pecofence_render::RenderStack::new()?;
    let motion = pecofence_render::motion::Motion::new(&stack)?;
    let root = stack.compositor.create_container_visual();
    // Exercise the actual Windows clip API: reuse during collapse/resize, remove on switching
    // to Fluent, then reinstall. This creates no HWND or visible desktop surface.
    for (w, h) in [(530.0, 368.0), (530.0, 36.0), (1060.0, 72.0)] {
        motion.clip_rounded(&root, w, h, 8.0)?;
    }
    motion.clear_clip(&root)?;
    motion.clip_rounded(&root, 530.0, 368.0, 8.0)?;
    println!("Composition rounded clip: resize, collapse and style switch OK");
    let dest = std::env::args()
        .nth(1)
        .unwrap_or(".cache/liquid-glass-native.bmp".into());
    let gpu = GpuDevice::new_or_warp()?;
    let (width, height) = (1000, 700);
    let canvas = gpu.create_render_target(width, height)?;
    let font = TextFormat::new("Segoe UI", 14.0)?;
    let heading = TextFormat::with_weight("Segoe UI", 30.0, FontWeight(600))?;
    canvas.draw(|s| {
        let paper = s.create_linear_gradient(
            Vector2::new(0.0, 0.0),
            Vector2::new(500.0, 700.0),
            &[
                GradientStop::new(0.0, ColorF::new(0.82, 0.83, 0.85, 1.0)),
                GradientStop::new(1.0, ColorF::new(0.96, 0.965, 0.98, 1.0)),
            ],
        )?;
        s.fill_rect(
            &Rect::from_xywh(0.0, 0.0, width as f32, height as f32),
            &paper,
        );
        let grid = s.create_solid_brush(ColorF::new(0.3, 0.32, 0.4, 0.055))?;
        for x in (0..width).step_by(40) {
            s.fill_rect(&Rect::from_xywh(x as f32, 150.0, 1.0, 490.0), &grid);
        }
        for y in (160..640).step_by(40) {
            s.fill_rect(&Rect::from_xywh(0.0, y as f32, width as f32, 1.0), &grid);
        }
        let ink = s.create_solid_brush(ColorF::new(0.10, 0.11, 0.13, 1.0))?;
        s.draw_text(
            "Liquid Glass",
            &heading,
            &Rect::from_xywh(56.0, 43.0, 800.0, 48.0),
            &ink,
        );
        s.draw_text(
            "Native material study · production Direct2D renderer",
            &font,
            &Rect::from_xywh(58.0, 100.0, 840.0, 28.0),
            &ink,
        );
        let ribbon = s.create_linear_gradient(
            Vector2::new(420.0, 0.0),
            Vector2::new(960.0, 0.0),
            &[
                GradientStop::new(0.0, ColorF::new(0.06, 0.72, 0.85, 1.0)),
                GradientStop::new(0.42, ColorF::new(0.30, 0.30, 0.72, 1.0)),
                GradientStop::new(0.75, ColorF::new(0.90, 0.04, 0.57, 1.0)),
                GradientStop::new(1.0, ColorF::new(0.98, 0.20, 0.42, 1.0)),
            ],
        )?;
        s.fill_rounded_rect(
            &RoundedRect::uniform(Rect::from_xywh(420.0, 532.0, 550.0, 66.0), 33.0),
            &ribbon,
        );
        let green = s.create_solid_brush(ColorF::new(0.20, 0.76, 0.30, 1.0))?;
        s.fill_rounded_rect(
            &RoundedRect::uniform(Rect::from_xywh(60.0, 245.0, 183.0, 78.0), 39.0),
            &green,
        );
        Ok(())
    })?;
    let mut pixels = canvas.read_pixels()?;
    let mut backdrop = MonitorBackdrop {
        left: 0,
        top: 0,
        width: width as i32,
        height: height as i32,
        downscale: 1,
        image: Image {
            width,
            height,
            bgra: pixels.clone(),
        },
    };
    backdrop.image.blur(
        Theme::light()
            .with_liquid_glass()
            .acrylic_tint()
            .blur_sigma_dip,
    );
    let chrome = FenceChrome::new()?;
    // The two lens specimens use the same optics with a circular outline; the large plate
    // uses the actual theme radius, title and bitmap-compositing path.
    for (index, (left, top, w, h, radius, title)) in [
        (94, 231, 164, 104, 52.0, ""),
        (122, 410, 104, 104, 52.0, ""),
        (362, 210, 530, 368, 8.0, "工作空间"),
    ]
    .into_iter()
    .enumerate()
    {
        let started = Instant::now();
        let image = liquid_glass::crop(
            &backdrop,
            [left, top, w, h],
            1.0,
            GlassOptics {
                radius,
                ..Default::default()
            },
        );
        println!(
            "plate {w}x{h}: crop {:.2} ms",
            started.elapsed().as_secs_f64() * 1000.0
        );
        let mut theme = Theme::light().with_liquid_glass().over_glass(&image);
        theme.corner_radius = radius;
        let plate = gpu.create_render_target(w as u32, h as u32)?;
        plate.draw(|s| {
            chrome.draw(
                s,
                &mut BitmapCache::new(),
                &theme,
                1.0,
                w as f32,
                h as f32,
                Backdrop::Glass(
                    BackdropCrop {
                        image: &image,
                        key: "study",
                        height: h as f32,
                    },
                    1.0,
                ),
                title,
                None,
                false,
                0.0,
                1.0,
                &[],
                None,
                TitleDeco::default(),
                TitleState {
                    title: if title.is_empty() { 0.0 } else { 1.0 },
                    ..Default::default()
                },
                FenceStyle::default(),
            )?;
            let ink = s.create_solid_brush(theme.text_primary)?;
            if index == 1 {
                s.fill_rounded_rect(
                    &RoundedRect::uniform(Rect::from_xywh(49.0, 28.0, 5.0, 48.0), 2.5),
                    &ink,
                );
                s.fill_rounded_rect(
                    &RoundedRect::uniform(Rect::from_xywh(27.0, 49.0, 48.0, 5.0), 2.5),
                    &ink,
                );
            } else if index == 2 {
                for (i, name) in ["设计资料", "项目文档", "图片素材"].into_iter().enumerate()
                {
                    let x = 45.0 + i as f32 * 166.0;
                    let color = s.create_solid_brush(
                        [
                            ColorF::new(0.88, 0.65, 0.25, 1.0),
                            ColorF::new(0.25, 0.52, 0.85, 1.0),
                            ColorF::new(0.56, 0.43, 0.76, 1.0),
                        ][i],
                    )?;
                    s.fill_rounded_rect(
                        &RoundedRect::uniform(Rect::from_xywh(x, 75.0, 38.0, 34.0), 6.0),
                        &color,
                    );
                    s.draw_text(
                        name,
                        &font,
                        &Rect::from_xywh(x - 8.0, 121.0, 90.0, 28.0),
                        &ink,
                    );
                }
            }
            Ok(())
        })?;
        let glass = plate.read_pixels()?;
        // Software equivalent of the composition root's rounded clip and soft outside shadow,
        // needed only for this headless export (the application clips on the compositor).
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let distance = |px: f32, py: f32| {
                    let qx = (px - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
                    let qy = (py - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
                    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius
                };
                let (px, py) = ((x - left) as f32 + 0.5, (y - top) as f32 + 0.5);
                let d = distance(px, py);
                let dst = ((y as u32 * width + x as u32) * 4) as usize;
                if d > 0.0 {
                    let shadow_d = distance(px, py - 7.0).max(0.0);
                    let shadow = (-shadow_d * shadow_d / (2.0 * 12.0 * 12.0)).exp() * 0.13;
                    for c in 0..3 {
                        pixels[dst + c] = (pixels[dst + c] as f32 * (1.0 - shadow)) as u8;
                    }
                }
                if x >= left && x < left + w && y >= top && y < top + h {
                    let a = (0.5 - d).clamp(0.0, 1.0);
                    let src = (((y - top) * w + x - left) * 4) as usize;
                    let source_alpha = glass[src + 3] as f32 / 255.0 * a;
                    for c in 0..3 {
                        pixels[dst + c] = (glass[src + c] as f32 * a
                            + pixels[dst + c] as f32 * (1.0 - source_alpha))
                            .round() as u8;
                    }
                }
            }
        }
    }
    save_bmp(Path::new(&dest), width, height, &pixels)?;
    println!("Saved {dest}");
    Ok(())
}
