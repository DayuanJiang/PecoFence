//! Eight-second, opaque owned window for testing covered-fence pause/resume.
//!
//! No capture, desktop/window manipulation, wallpaper changes, persistence or hooks.
//! It creates one ordinary, enabled, non-layered window over the primary work area.
//! Closing it or pressing Escape ends the probe early. It never becomes topmost.

use pecofence_platform::{RECT, monitors, msg, paint, window};
use serde_json::json;
use std::{io::Write, time::Instant};

const EXIT_TIMER: usize = 1;
const SECONDS: u32 = 8;
const TITLE: &str = "PecoFence bounded occlusion test (8 seconds)";
const BACKGROUND: u32 = 0x00382B24; // COLORREF, opaque dark blue-gray.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = window::set_process_dpi_awareness_v2();
    let primary = monitors::enumerate()
        .into_iter()
        .find(|monitor| monitor.primary)
        .ok_or("no primary monitor is available")?;
    let area = primary.work_area;
    let width = area.right - area.left;
    let height = area.bottom - area.top;
    if width <= 0 || height <= 0 {
        return Err("the primary work area is empty".into());
    }

    let class = window::WindowClass::register(
        "PecoFence.BoundedOcclusionProbe",
        window::ClassOptions {
            background: Some(BACKGROUND),
            ..Default::default()
        },
    )?;
    let font = paint::Font::new("Segoe UI", 24.0, primary.dpi.max(96));
    let probe = window::WindowBuilder::new(&class)
        .title(TITLE)
        .style(window::style::OVERLAPPEDWINDOW)
        .ex_style(0)
        .bounds(area.left, area.top, width, height)
        .create(Box::new(move |hwnd, message, wparam, _| match message {
            msg::WM_PAINT => {
                let drawing = paint::Paint::begin(hwnd);
                // Paint beyond the client bounds; GDI clips this to the owned client area.
                let client = RECT {
                    left: 0,
                    top: 0,
                    right: width,
                    bottom: height,
                };
                drawing.fill(client, BACKGROUND);
                drawing.text(
                    RECT {
                        left: 24,
                        top: (height / 2 - 72).max(0),
                        right: width - 24,
                        bottom: height / 2,
                    },
                    TITLE,
                    &font,
                    0x00FFFFFF,
                    paint::Align::Center,
                );
                drawing.text(
                    RECT {
                        left: 24,
                        top: height / 2,
                        right: width - 24,
                        bottom: height / 2 + 72,
                    },
                    "This window closes automatically. Escape also exits.",
                    &font,
                    0x00E0D0C0,
                    paint::Align::Center,
                );
                Some(0)
            }
            msg::WM_TIMER if wparam == EXIT_TIMER => {
                window::post_quit(0);
                Some(0)
            }
            msg::WM_KEYDOWN if wparam == msg::VK_ESCAPE as usize => {
                window::post_quit(0);
                Some(0)
            }
            msg::WM_CLOSE | msg::WM_DESTROY => {
                window::post_quit(0);
                Some(0)
            }
            _ => None,
        }))?;

    let started = Instant::now();
    // This only changes our own ordinary window's position. No foreign window is altered.
    probe.set_bounds_z(area.left, area.top, width, height, window::ZOrder::Top, 0)?;
    probe.show_no_activate();
    probe.set_timer(EXIT_TIMER, SECONDS * 1000);
    let _ = window::pump_messages();
    println!(
        "{}",
        json!({
            "event":"ready", "probe":"boundedOcclusion", "seconds":SECONDS,
            "title":TITLE, "bounds":[area.left,area.top,width,height],
            "topmost":false, "capture":false
        })
    );
    std::io::stdout().flush()?;
    let code = window::run_message_loop();
    probe.kill_timer(EXIT_TIMER);
    probe.hide();
    println!(
        "{}",
        json!({
            "event":"closed", "probe":"boundedOcclusion",
            "elapsedSeconds":started.elapsed().as_secs_f64(), "messageLoopCode":code
        })
    );
    Ok(())
}
