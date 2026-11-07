//! A bounded, standalone test of the public native host-backdrop path.
//!
//! Generate its private bindings once, then build this example:
//! cargo run -p tool_bindgen -- tools/bindgen/host-backdrop.txt
//! cargo build -p pecofence-render --example host_backdrop_probe
//! host_backdrop_probe.exe --exit-after 15
//!
//! Only creates an owned, nonactivating, click-through 400x260 window at (24, 72).
//! --inspectable creates an ordinary enabled window for UI inspection, still shown without activation.
//! It does not capture pixels, write configuration, reparent windows, or alter WE/Explorer.
//! No repaint loop is used: movement inside the panel must come from the OS compositor.

#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute,
    clippy::too_many_arguments,
    dead_code,
    unused_imports
)]
#[path = "host_backdrop_probe/bindings.rs"]
mod native;

use pecofence_platform::{desktop, dwm, msg, window};
use pecofence_render::{ColorF, Panel, Rect, RenderStack, RoundedRect, TextFormat};
use std::time::Instant;
use windows_core::Interface;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 260;

fn options() -> Result<(u32, bool), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut seconds = 15;
    let mut inspectable = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--exit-after" => {
                seconds = arguments
                    .next()
                    .ok_or("--exit-after requires a duration in seconds")?
                    .parse::<u32>()?;
            }
            "--inspectable" => inspectable = true,
            "--help" | "-h" => {
                println!(
                    "host_backdrop_probe [--exit-after SECONDS] [--inspectable]\nDuration: 1..300; default 15. No capture or configuration changes."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    if !(1..=300).contains(&seconds) {
        return Err("--exit-after must be between 1 and 300 seconds".into());
    }
    Ok((seconds, inspectable))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (seconds, inspectable) = options()?;
    let build = desktop::os_build();
    if build < 22000 {
        return Err("the public Win32 host-backdrop flag requires Windows 11 build 22000+".into());
    }

    // DPI awareness and COM initialization affect this process only.
    let _ = window::set_process_dpi_awareness_v2();
    let _ole = pecofence_platform::com::OleGuard::init()?;
    let stack = RenderStack::new()?;
    let class = window::WindowClass::register(
        "PecoFence.HostBackdropProbe",
        window::ClassOptions::default(),
    )?;
    let extended_style = if inspectable {
        window::style::EX_NOREDIRECTIONBITMAP | window::style::EX_TOPMOST
    } else {
        window::style::EX_NOACTIVATE
            | window::style::EX_TOOLWINDOW
            | window::style::EX_NOREDIRECTIONBITMAP
            | window::style::EX_TRANSPARENT
    };
    let probe = window::WindowBuilder::new(&class)
        .title("PecoFence native host-backdrop probe")
        .style(
            window::style::POPUP
                | if inspectable {
                    0
                } else {
                    native::WS_DISABLED as u32
                },
        )
        .ex_style(extended_style)
        .bounds(24, 72, WIDTH, HEIGHT)
        .create(Box::new(move |_, message, _, _| match message {
            msg::WM_NCHITTEST if !inspectable => Some(msg::HTTRANSPARENT),
            msg::WM_MOUSEACTIVATE if !inspectable => Some(3), // MA_NOACTIVATE
            msg::WM_ERASEBKGND => Some(1),
            msg::WM_CLOSE | msg::WM_DESTROY => {
                window::post_quit(0);
                Some(0)
            }
            _ => None,
        }))?;
    let hwnd = probe.hwnd();
    let enabled = 1_i32;
    // SAFETY: this is our live HWND, and the BOOL-sized value outlives the synchronous call.
    unsafe {
        native::DwmSetWindowAttribute(
            native::HWND(hwnd.0),
            native::DWMWA_USE_HOSTBACKDROPBRUSH as u32,
            (&enabled as *const i32).cast(),
            size_of::<i32>() as u32,
        )
        .ok()?;
    }
    dwm::set_nonclient_rendering(hwnd, false)?;
    let _ = dwm::set_border_color_none(hwnd);
    let _ = dwm::set_corner_preference(hwnd, dwm::CornerPreference::DoNotRound);

    let target = stack.create_target(hwnd)?;
    let root = stack.compositor.create_container_visual();
    target.set_root(&root);
    let motion = pecofence_render::motion::Motion::new(&stack)?;
    motion.clip_rounded(&root, WIDTH as f32, HEIGHT as f32, 18.0)?;

    let compositor: native::Compositor = stack.compositor.as_raw().cast()?;
    let backdrop = compositor.CreateHostBackdropBrush()?;
    let background = stack.compositor.create_sprite_visual();
    background.set_size(WIDTH as f32, HEIGHT as f32);
    let native_background: native::SpriteVisual = background.as_raw().cast()?;
    native_background.SetBrush(&backdrop)?;
    root.children().insert_at_bottom(&background);

    // Draw chrome once. Keep the center unobscured so live/black/stale output is apparent.
    let mut chrome = Panel::new(&stack)?;
    chrome.resize(WIDTH, HEIGHT)?;
    let font = TextFormat::new("Segoe UI", 14.0)?;
    chrome.draw(96, |session, width, height| {
        session.clear(ColorF::TRANSPARENT);
        let rim = session.create_solid_brush(ColorF::new(1.0, 1.0, 1.0, 0.75))?;
        session.draw_rounded_rect(
            &RoundedRect::uniform(
                Rect::from_xywh(0.75, 0.75, width - 1.5, height - 1.5),
                17.25,
            ),
            &rim,
            1.5,
        );
        let label = session.create_solid_brush(ColorF::new(0.02, 0.025, 0.04, 0.72))?;
        let ink = session.create_solid_brush(ColorF::new(1.0, 1.0, 1.0, 0.96))?;
        session.fill_rounded_rect(
            &RoundedRect::uniform(Rect::from_xywh(12.0, 12.0, width - 24.0, 49.0), 8.0),
            &label,
        );
        session.draw_text(
            "Native HostBackdrop - no capture\nWatch the wallpaper motion inside this panel",
            &font,
            &Rect::from_xywh(22.0, 18.0, width - 44.0, 42.0),
            &ink,
        );
        session.fill_rounded_rect(
            &RoundedRect::uniform(
                Rect::from_xywh(12.0, height - 40.0, width - 24.0, 28.0),
                8.0,
            ),
            &label,
        );
        session.draw_text(
            &format!(
                "Auto-close: {seconds}s | {} | DWM refresh",
                if inspectable {
                    "inspectable"
                } else {
                    "click-through"
                }
            ),
            &font,
            &Rect::from_xywh(22.0, height - 36.0, width - 44.0, 22.0),
            &ink,
        );
        Ok(())
    })?;
    root.children().insert_at_top(&chrome.visual);

    println!(
        "Native HostBackdrop probe: build {build}, hwnd=0x{:x}, 400x260 at (24,72), auto-close {seconds}s.\nNo capture session, file writes, or application-side animation timer.\nAn S_OK result alone does not prove correct or live wallpaper content.",
        hwnd.0 as usize
    );
    let started = Instant::now();
    probe.show_no_activate();
    window::quit_after(seconds * 1000);
    let code = window::run_message_loop();
    probe.hide();
    println!(
        "Probe closed after {:.2}s (message-loop code {code}).",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
