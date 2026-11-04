//! Bounded GPU-capture and liquid-glass preview; does not launch or take over PecoFence.
//!
//! cargo build -p pecofence-render --example realtime_glass_probe
//! realtime_glass_probe.exe --exit-after 20
//!
//! Captures the existing top-level shell/Progman window using the production GPU path.
//! Only the owned 640x340 preview at (24,72) is created. No WDA, desktop icon changes,
//! wallpaper changes, hooks, screenshot files, configuration writes or process injection.
//! Diagnostic only: reads nine 16x16 GPU tiles once a second and reports pixel statistics.

#[path = "realtime_glass_probe/pixel_sampler.rs"]
mod pixel_sampler;
#[allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations
)]
#[path = "../src/live_capture_bindings.rs"]
mod sample_bindings;

use pecofence_core::{RealtimeGlassQuality, RealtimeGlassSettings};
use pecofence_platform::{desktop, dwm, msg, window};
use pecofence_render::{
    BitmapCache, ColorF, Panel, Rect, RenderStack, RoundedRect, TextFormat, gpu_glass::GpuGlass,
    live_capture::LiveCapture,
};
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, time::Instant};

const WIDTH: i32 = 640;
const HEIGHT: i32 = 340;
const X: i32 = 24;
const Y: i32 = 72;
const TIMER: usize = 1;

struct Options {
    seconds: u32,
    source: isize,
    inspectable: bool,
}

fn options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut seconds = 20;
    let mut source = desktop::shell_window().0 as isize;
    let mut inspectable = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--exit-after" => {
                seconds = arguments
                    .next()
                    .ok_or("--exit-after requires seconds")?
                    .parse::<u32>()?;
            }
            "--source-hwnd" => {
                let value = arguments.next().ok_or("--source-hwnd requires a handle")?;
                source = if let Some(value) = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                {
                    isize::from_str_radix(value, 16)?
                } else {
                    value.parse::<isize>()?
                };
            }
            "--inspectable" => inspectable = true,
            "--help" | "-h" => {
                println!(
                    "realtime_glass_probe [--exit-after SECONDS] [--source-hwnd HWND] [--inspectable]\nDuration 1..300, default 20. Source defaults to the shell/Progman HWND.\nGPU preview plus nine tiny diagnostic pixel samples per second; no desktop or wallpaper settings are changed."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    if !(1..=300).contains(&seconds) {
        return Err("--exit-after must be between 1 and 300 seconds".into());
    }
    if source == 0 {
        return Err("the source window is null".into());
    }
    Ok(Options {
        seconds,
        source,
        inspectable,
    })
}

struct Preview {
    capture: Option<LiveCapture>,
    panel: Panel,
    cache: BitmapCache,
    material: GpuGlass,
    font: TextFormat,
    settings: RealtimeGlassSettings,
    started: Instant,
    capture_frames: u64,
    panel_frames: u64,
    error_count: u64,
    errors: Vec<String>,
    sampler: pixel_sampler::PixelSampler,
}

impl Preview {
    fn tick(&mut self, stack: &RenderStack) -> windows_core::Result<()> {
        let Some(capture) = self.capture.as_mut() else {
            return Ok(());
        };
        if !capture.poll_latest()? {
            return Ok(());
        }
        self.capture_frames += 1;
        let Some(surface) = capture.surface() else {
            return Ok(());
        };
        let (width, height) = capture.content_size();
        self.sampler
            .sample(&surface, (width as u32, height as u32))?;
        self.cache.update_realtime_glass(
            stack,
            &surface,
            (width as u32, height as u32),
            capture.bounds(),
            &self.settings,
        )?;
        let cache = &mut self.cache;
        let material = &mut self.material;
        let font = &self.font;
        let label = format!(
            "Desktop GPU preview | 30 fps limit | Balanced | frame {}",
            self.capture_frames
        );
        let drawn = self.panel.draw(96, |session, width, height| {
            session.clear(ColorF::TRANSPARENT);
            cache.draw_realtime_preview(session, material, [X, Y, WIDTH, HEIGHT], 1.0, 18.0)?;
            let rim = session.create_solid_brush(ColorF::new(1.0, 1.0, 1.0, 0.8))?;
            session.draw_rounded_rect(
                &RoundedRect::uniform(
                    Rect::from_xywh(0.75, 0.75, width - 1.5, height - 1.5),
                    17.25,
                ),
                &rim,
                1.5,
            );
            let plate = session.create_solid_brush(ColorF::new(0.02, 0.025, 0.04, 0.72))?;
            let ink = session.create_solid_brush(ColorF::new(1.0, 1.0, 1.0, 0.98))?;
            session.fill_rounded_rect(
                &RoundedRect::uniform(Rect::from_xywh(12.0, 12.0, width - 24.0, 51.0), 8.0),
                &plate,
            );
            session.draw_text(
                &format!("{label}\nLive wallpaper texture -> GPU blur and refraction"),
                font,
                &Rect::from_xywh(22.0, 18.0, width - 44.0, 43.0),
                &ink,
            );
            Ok(())
        })?;
        if drawn {
            self.panel_frames += 1;
            Ok(())
        } else {
            Err(windows_core::Error::new(
                windows_core::HRESULT(0x80004005u32 as i32),
                "preview drawing reported device loss",
            ))
        }
    }

    fn error(&mut self, error: &dyn std::fmt::Display) {
        self.error_count += 1;
        if self.errors.len() < 8 {
            self.errors.push(error.to_string());
        }
        // The probe is diagnostic, not a recovery daemon. Stop after bounded failures.
        if self.error_count >= 3 {
            window::post_quit(1);
        }
    }
}

fn run() -> Result<Value, Box<dyn std::error::Error>> {
    let options = options()?;
    let _ = window::set_process_dpi_awareness_v2();
    let source = pecofence_platform::HWND(options.source as *mut core::ffi::c_void);
    let source_rect = window::window_rect(source);
    let bounds = [
        source_rect.left,
        source_rect.top,
        source_rect.right - source_rect.left,
        source_rect.bottom - source_rect.top,
    ];
    if bounds[2] <= 0 || bounds[3] <= 0 {
        return Err("the source window has an empty rectangle".into());
    }

    let _ole = pecofence_platform::com::OleGuard::init()?;
    let stack = Rc::new(RenderStack::new()?);
    let capture = LiveCapture::for_window(&stack.capture_device(), options.source, bounds)?;
    let _ = capture.set_max_fps(30);
    let mut panel = Panel::new(&stack)?;
    panel.resize(WIDTH, HEIGHT)?;
    let settings = RealtimeGlassSettings {
        enabled: true,
        fps: 30,
        quality: RealtimeGlassQuality::Balanced,
        blur: 8.0,
        refraction: 1.0,
        ..Default::default()
    };
    let state = Rc::new(RefCell::new(Preview {
        capture: Some(capture),
        panel,
        cache: BitmapCache::new(),
        material: GpuGlass::default(),
        font: TextFormat::new("Segoe UI", 14.0)?,
        settings,
        started: Instant::now(),
        capture_frames: 0,
        panel_frames: 0,
        error_count: 0,
        errors: Vec::new(),
        sampler: pixel_sampler::PixelSampler::new(&stack.capture_device())?,
    }));

    let class = window::WindowClass::register(
        "PecoFence.RealtimeGlassProbe",
        window::ClassOptions::default(),
    )?;
    let handler_state = state.clone();
    let handler_stack = stack.clone();
    let inspectable = options.inspectable;
    let extended_style = window::style::EX_NOREDIRECTIONBITMAP
        | window::style::EX_TOPMOST
        | if inspectable {
            0
        } else {
            window::style::EX_NOACTIVATE | window::style::EX_TOOLWINDOW
        };
    let probe = window::WindowBuilder::new(&class)
        .title("PecoFence desktop GPU glass preview")
        .style(window::style::POPUP)
        .ex_style(extended_style)
        .bounds(X, Y, WIDTH, HEIGHT)
        .create(Box::new(move |_, message, wparam, _| match message {
            msg::WM_TIMER if wparam == TIMER => {
                pecofence_platform::realtime_clock::RealtimeClock::acknowledge();
                if let Ok(mut preview) = handler_state.try_borrow_mut() {
                    if let Err(error) = preview.tick(&handler_stack) {
                        preview.error(&error);
                    }
                }
                Some(0)
            }
            msg::WM_MOUSEACTIVATE if !inspectable => Some(3), // MA_NOACTIVATE
            msg::WM_ERASEBKGND => Some(1),
            msg::WM_CLOSE | msg::WM_DESTROY => {
                window::post_quit(0);
                Some(0)
            }
            _ => None,
        }))?;
    let _ = dwm::set_nonclient_rendering(probe.hwnd(), false);
    let _ = dwm::set_corner_preference(probe.hwnd(), dwm::CornerPreference::DoNotRound);
    let _ = dwm::set_border_color_none(probe.hwnd());
    let target = stack.create_target(probe.hwnd())?;
    let root = stack.compositor.create_container_visual();
    let motion = pecofence_render::motion::Motion::new(&stack)?;
    motion.clip_rounded(&root, WIDTH as f32, HEIGHT as f32, 18.0)?;
    root.children().insert_at_top(&state.borrow().panel.visual);
    target.set_root(&root);

    probe.show_no_activate();
    let precise_clock =
        pecofence_platform::realtime_clock::RealtimeClock::start(probe.hwnd(), TIMER, 30)?;
    window::quit_after(options.seconds * 1000);
    let code = window::run_message_loop();
    drop(precise_clock);
    probe.kill_timer(TIMER);
    probe.hide();
    let mut preview = state.borrow_mut();
    let timestamp = preview
        .capture
        .as_ref()
        .and_then(|capture| capture.frame_timestamp_100ns());
    // Close the owned WGC session before reporting completion.
    preview.capture.take();
    if let Err(error) = preview.sampler.finish() {
        preview.error(&error);
    }
    let mut summary = json!({
        "probe": "realtimeGlass",
        "sourceHwnd": format!("0x{:x}", options.source),
        "sourceClass": desktop::class_name(source),
        "sourceBounds": bounds,
        "previewBounds": [X,Y,WIDTH,HEIGHT],
        "inspectable": inspectable,
        "captureFrames": preview.capture_frames,
        "panelFrames": preview.panel_frames,
        "elapsed": preview.started.elapsed().as_secs_f64(),
        "errors": preview.errors,
        "errorCount": preview.error_count,
        "messageLoopCode": code,
        "lastFrameTimestamp100ns": timestamp,
        "settings": preview.settings,
        "cpuReadback": true,
        "cpuReadbackMode": "diagnostic tiles only; production path remains GPU-only",
        "productionCpuReadback": false,
        "frameCountsProvePixelChanges": false
    });
    summary
        .as_object_mut()
        .unwrap()
        .extend(preview.sampler.summary().as_object().unwrap().clone());
    Ok(summary)
}

fn main() {
    let started = Instant::now();
    match run() {
        Ok(summary) => println!("{summary}"),
        Err(error) => {
            println!(
                "{}",
                json!({
                    "probe": "realtimeGlass", "captureFrames":0, "panelFrames":0,
                    "elapsed":started.elapsed().as_secs_f64(), "errors":[error.to_string()],
                    "errorCount":1, "cpuReadback":false
                })
            );
            std::process::exit(1);
        }
    }
}
