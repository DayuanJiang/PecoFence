//! Exercises the real D2D effect graph, cache invalidation and completed-frame timings.
use pecofence_render::{
    Image, MonitorBackdrop, Theme,
    gpu_glass::{GpuGlass, WallpaperCache},
    theme::GlassLevel,
};
use std::{rc::Rc, time::Instant};
use windows_canvas::{ColorF, GpuDevice, Rect};

fn source(image: Image) -> Rc<Vec<MonitorBackdrop>> {
    Rc::new(vec![MonitorBackdrop {
        left: -320,
        top: 0,
        width: image.width as i32,
        height: image.height as i32,
        downscale: 1,
        image,
        gpu: None,
    }])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ole = pecofence_platform::com::OleGuard::init()?;
    let gpu = GpuDevice::new_or_warp()?;
    let mut cache = WallpaperCache::default();
    let mut glass = GpuGlass::default();
    let level = Theme::light().with_liquid_glass().liquid_level(1.0);
    let background = source(Image::solid(1600, 1200, [80; 3]));
    let target = gpu.create_render_target(260, 160)?;
    target.draw(|s| {
        s.clear(ColorF::TRANSPARENT);
        glass.draw(
            s,
            &mut cache,
            &background,
            [-200, 80, 260, 160],
            0.0,
            1.0,
            24.0,
            level,
            1.0,
            0.0,
        )
    })?;
    let pixels = target.read_pixels()?;
    let centre = ((80 * 260 + 130) * 4) as usize;
    println!("opaque centre: {:?}", &pixels[centre..centre + 4]);
    for value in &pixels[centre..centre + 3] {
        assert!(
            (*value as i32 - 80).abs() <= 1,
            "RGB channel composition changed a neutral background"
        );
    }
    assert_eq!(pixels[centre + 3], 255);
    target.draw(|s| {
        s.clear(ColorF::TRANSPARENT);
        glass.draw(
            s,
            &mut cache,
            &background,
            [-190, 90, 260, 160],
            0.0,
            1.0,
            24.0,
            level,
            0.5,
            0.0,
        )
    })?;
    let pixels = target.read_pixels()?;
    println!("half-alpha centre: {:?}", &pixels[centre..centre + 4]);
    for value in &pixels[centre..centre + 3] {
        assert!(
            (*value as i32 - 40).abs() <= 1,
            "the plate reveal must scale premultiplied RGB"
        );
    }
    assert!((pixels[centre + 3] as i32 - 128).abs() <= 1);
    assert_eq!(
        glass.stats().map_builds,
        1,
        "moving must not regenerate the field"
    );
    assert_eq!(cache.uploads, 1, "moving must not upload wallpaper again");

    {
        let mut cache = WallpaperCache::default();
        let mut renderer = GpuGlass::default();
        let mut checker = Image::solid(800, 500, [0; 3]);
        for (i, pixel) in checker.bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            pixel[..3].fill(if (i % 800 + i / 800) % 2 == 0 { 0 } else { 255 });
        }
        let checker = source(checker);
        let unfrosted = GlassLevel {
            frost_sigma_dip: 0.0,
            ..level
        };
        target.draw(|s| {
            s.clear(ColorF::TRANSPARENT);
            renderer.draw(
                s,
                &mut cache,
                &checker,
                [-200, 80, 260, 160],
                0.0,
                1.0,
                24.0,
                unfrosted,
                1.0,
                0.0,
            )
        })?;
        let pixels = target.read_pixels()?;
        // kube's clear glass: without frost the flat face shows the wallpaper pixel for pixel.
        assert_eq!(
            [pixels[centre], pixels[centre + 4]],
            [0, 255],
            "the lens must not move the flat centre"
        );
        assert_eq!(
            pixels[centre + 3],
            255,
            "the centre must not be a transparent hole"
        );
        target.draw(|s| {
            s.clear(ColorF::TRANSPARENT);
            renderer.draw(
                s,
                &mut cache,
                &checker,
                [-200, 80, 260, 160],
                0.0,
                1.0,
                24.0,
                level,
                1.0,
                0.0,
            )
        })?;
        let pixels = target.read_pixels()?;
        println!("frosted checker centre: {:?}", &pixels[centre..centre + 8]);
        for value in [pixels[centre], pixels[centre + 4]] {
            assert!(
                (value as i32 - 127).abs() <= 3,
                "the level's frost must diffuse pixel detail"
            );
        }
        let mut ramp = Image::solid(800, 500, [0; 3]);
        for (i, pixel) in ramp.bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            pixel[..3].fill((i % 800 / 4) as u8);
        }
        let ramp = source(ramp);
        let mut centres = Vec::new();
        for left in [-200, -180] {
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &ramp,
                    [left, 80, 260, 160],
                    0.0,
                    1.0,
                    24.0,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            centres.push(target.read_pixels()?[centre]);
        }
        println!("ramp centres at two window positions: {centres:?}");
        assert!(
            (centres[0] as i32 - 62).abs() <= 1,
            "world-to-window sampling offset is wrong"
        );
        assert_eq!(
            centres[1] - centres[0],
            5,
            "moving must select the new wallpaper position"
        );
        assert_eq!(renderer.stats().map_builds, 1);
        assert_eq!(
            cache.uploads, 2,
            "wallpaper refresh must upload only the changed source"
        );
        println!("clear centre, source refresh and negative desktop coordinates passed");
    }

    {
        let mut cache = WallpaperCache::default();
        let mut renderer = GpuGlass::default();
        let monitors = Rc::new(vec![
            MonitorBackdrop {
                left: -400,
                top: 0,
                width: 400,
                height: 400,
                downscale: 2,
                image: Image::solid(200, 200, [60; 3]),
                gpu: None,
            },
            MonitorBackdrop {
                left: 0,
                top: 0,
                width: 400,
                height: 400,
                downscale: 1,
                image: Image::solid(400, 400, [180; 3]),
                gpu: None,
            },
        ]);
        for (left, expected) in [(-350, 60), (40, 180)] {
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &monitors,
                    [left, 80, 260, 160],
                    0.0,
                    1.0,
                    24.0,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            assert!((target.read_pixels()?[centre] as i32 - expected).abs() <= 1);
        }
        assert_eq!(
            cache.uploads, 2,
            "each monitor texture must upload only once"
        );
        assert_eq!(renderer.stats().map_builds, 1);
        for (h, scale, radius) in [(120, 1.0, 24.0), (120, 2.0, 24.0), (120, 2.0, 12.0)] {
            target.draw(|s| {
                renderer.draw(
                    s,
                    &mut cache,
                    &monitors,
                    [40, 80, 260, h],
                    0.0,
                    scale,
                    radius,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            target.read_pixels()?;
        }
        assert_eq!(
            renderer.stats().map_builds,
            4,
            "size, DPI and radius must invalidate the geometry"
        );
        assert_eq!(
            cache.uploads, 2,
            "geometry must not invalidate the wallpaper"
        );
        renderer.reset();
        cache.clear();
        target.draw(|s| {
            renderer.draw(
                s,
                &mut cache,
                &monitors,
                [40, 80, 260, 160],
                0.0,
                1.0,
                24.0,
                level,
                1.0,
                0.0,
            )
        })?;
        assert!((target.read_pixels()?[centre] as i32 - 180).abs() <= 1);
        assert_eq!(cache.uploads, 4);
        assert_eq!(renderer.stats().map_builds, 1);
        println!(
            "multi-monitor downsampling, size/DPI/radius invalidation and resource reset passed"
        );
    }

    {
        let mut cache = WallpaperCache::default();
        let mut renderer = GpuGlass::default();
        let mut stripes = Image::solid(800, 500, [0; 3]);
        for (i, p) in stripes.bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            p[..3].fill(if (i % 800 / 12) % 2 == 0 { 35 } else { 220 });
        }
        let stripes = source(stripes);
        let rect = Rect::from_xywh(20.0, 20.0, 140.0, 28.0);
        let mut completed = Vec::new();
        for left in [-200, -196, -192, -188] {
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &stripes,
                    [left, 80, 260, 160],
                    0.0,
                    1.0,
                    24.0,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            let plain = target.read_pixels()?;
            let started = Instant::now();
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &stripes,
                    [left, 80, 260, 160],
                    0.0,
                    1.0,
                    24.0,
                    level,
                    1.0,
                    0.0,
                )?;
                renderer.draw_control(s, 100, rect, 1.0, 14.0, 1.0)?;
                Ok(())
            })?;
            let lens = target.read_pixels()?;
            completed.push(started.elapsed().as_secs_f64() * 1000.0);
            let mut changed = 0;
            for y in 0..160 {
                for x in 0..260 {
                    let index = ((y * 260 + x) * 4) as usize;
                    let delta = (plain[index] as i32 - lens[index] as i32).abs();
                    if !(20..160).contains(&x) || !(20..48).contains(&y) || (x == 20 && y == 20) {
                        assert!(
                            delta <= 1,
                            "control drew outside its rounded bounds at {x},{y}"
                        );
                    } else if delta > 8 {
                        changed += 1;
                    }
                    assert_eq!(lens[index + 3], 255);
                }
            }
            assert!(
                changed > 40,
                "glass tab must change background pixels, not just add a border"
            );
        }
        assert_eq!(renderer.stats().map_builds, 1);
        assert_eq!(renderer.stats().control_map_builds, 1);
        assert_eq!(cache.uploads, 1);
        println!(
            "tab lens refraction, rounded bounds and movement cache passed; completed frames={completed:?}ms"
        );
    }

    for (w, h) in [(272, 282), (800, 298), (1152, 1066)] {
        let target = gpu.create_render_target(w, h)?;
        let mut renderer = GpuGlass::default();
        let cold = Instant::now();
        target.draw(|s| {
            s.clear(ColorF::TRANSPARENT);
            renderer.draw(
                s,
                &mut cache,
                &background,
                [-250, 40, w as i32, h as i32],
                0.0,
                2.0,
                24.0,
                level,
                1.0,
                0.0,
            )
        })?;
        target.read_pixels()?;
        let cold_ms = cold.elapsed().as_secs_f64() * 1000.0;
        let mut submit = Vec::new();
        let mut complete = Vec::new();
        for i in 0..30 {
            let started = Instant::now();
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &background,
                    [-250 + i * 3, 40 + i, w as i32, h as i32],
                    0.0,
                    2.0,
                    24.0,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            submit.push(started.elapsed().as_secs_f64() * 1000.0);
            // Includes GPU completion and CPU readback, so asynchronous submission alone
            // cannot make this benchmark claim an unrealistically small frame time.
            target.read_pixels()?;
            complete.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        assert_eq!(renderer.stats().map_builds, 1);
        assert_eq!(cache.uploads, 1);
        submit.sort_by(f64::total_cmp);
        complete.sort_by(f64::total_cmp);
        println!(
            "{w}x{h}: cold={cold_ms:.2}ms, submit median={:.2}ms, completed+readback median={:.2}ms p95={:.2}ms, maps={} wallpaper_uploads={}",
            submit[15],
            complete[15],
            complete[28],
            renderer.stats().map_builds,
            cache.uploads,
        );
    }
    // A roll animation changes the height every frame, so each frame rebuilds the geometry.
    for (w, from) in [(800, 298), (1152, 1066)] {
        let target = gpu.create_render_target(w, from)?;
        let mut renderer = GpuGlass::default();
        let mut complete = Vec::new();
        for h in (64..=from).rev().step_by(16) {
            let started = Instant::now();
            target.draw(|s| {
                s.clear(ColorF::TRANSPARENT);
                renderer.draw(
                    s,
                    &mut cache,
                    &background,
                    [-250, 40, w as i32, h as i32],
                    0.0,
                    2.0,
                    16.0,
                    level,
                    1.0,
                    0.0,
                )
            })?;
            target.read_pixels()?;
            complete.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        let builds = renderer.stats().map_builds;
        assert_eq!(builds as usize, complete.len());
        complete.sort_by(f64::total_cmp);
        println!(
            "{w}x{from} resize: frames={builds}, completed+readback median={:.2}ms max={:.2}ms",
            complete[complete.len() / 2],
            complete[complete.len() - 1],
        );
    }
    println!("GPU material, opacity and movement cache checks passed");
    Ok(())
}
