//! Probe of the Magnifier's capture path: Windows.Graphics.Capture of the primary monitor into
//! Direct2D, drawn zoomed around the pointer, with no CPU copy. Prints whether Windows grants
//! capture without its yellow frame, the frame rate and the cost of a draw, and checks that the
//! picture is not black.
//!
//! `cargo run -p pecofence-render --example magnifier_probe --release -- [zoom] [seconds]`
//! Move the mouse while it runs: Windows sends frames only when the screen changes.
use pecofence_platform::{monitors, window};
use pecofence_render::magnifier::Magnifier;
use std::time::{Duration, Instant};
use windows_canvas::{ColorF, GpuDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let zoom: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(3.0);
    let seconds: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(5.0);

    let _ole = pecofence_platform::com::OleGuard::init()?;
    let gpu = GpuDevice::new_or_warp()?;
    let monitor = monitors::enumerate()
        .into_iter()
        .find(|m| m.primary)
        .ok_or("no primary monitor")?;
    println!(
        "monitor {} {}x{} dpi {}",
        monitor.device_name,
        monitor.bounds.right - monitor.bounds.left,
        monitor.bounds.bottom - monitor.bounds.top,
        monitor.dpi
    );
    let handle = monitor.handle.0 as *mut _;
    let mut magnifier = Magnifier::new(gpu.clone());
    magnifier.poll(handle, None)?;
    println!(
        "capture started: {:?}, borderless (no yellow frame): {:?}",
        magnifier.size(),
        magnifier.borderless()
    );

    let target = gpu.create_render_target(640, 360)?;
    let start = Instant::now();
    let mut draw_ms = Vec::new();
    let mut drawn_once = false;
    while start.elapsed() < Duration::from_secs_f32(seconds) {
        let cursor = window::cursor_pos();
        let centre = (
            (cursor.x - monitor.bounds.left) as f32,
            (cursor.y - monitor.bounds.top) as f32,
        );
        let t = Instant::now();
        let mut fresh = false;
        target.draw(|s| {
            fresh = magnifier.poll(handle, Some(centre))?;
            if fresh || !drawn_once {
                s.clear(ColorF::TRANSPARENT);
                drawn_once |= magnifier.draw(
                    s,
                    [0.0, 0.0, 640.0, 360.0],
                    centre,
                    monitor.scale(),
                    zoom,
                    true,
                )?;
            }
            Ok(())
        })?;
        if fresh {
            draw_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let elapsed = start.elapsed().as_secs_f64();
    let stats = magnifier.stats;
    println!(
        "{} frames in {:.1} s = {:.1} fps ({} draws)",
        stats.frames,
        elapsed,
        stats.frames as f64 / elapsed,
        stats.draws
    );
    if !draw_ms.is_empty() {
        draw_ms.sort_by(|a, b| a.total_cmp(b));
        println!(
            "frame to drawn (poll + wrap + draw): median {:.2} ms, p95 {:.2} ms, max {:.2} ms",
            draw_ms[draw_ms.len() / 2],
            draw_ms[draw_ms.len() * 95 / 100],
            draw_ms[draw_ms.len() - 1]
        );
    }
    let pixels = target.read_pixels()?;
    let lit = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] | p[1] | p[2] != 0)
        .count();
    println!(
        "non-black pixels: {} of {} ({:.0} %)",
        lit,
        pixels.len() / 4,
        100.0 * lit as f64 / (pixels.len() / 4) as f64
    );
    assert!(stats.frames > 0, "no frame arrived");
    assert!(lit > 0, "the picture is black");
    Ok(())
}
