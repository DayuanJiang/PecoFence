//! The Magnifier's picture: monitors captured by Windows.Graphics.Capture and drawn zoomed, all on
//! the GPU. A captured frame is a texture of the app's own Direct3D device; Direct2D wraps it as a
//! bitmap without a copy, and `DrawBitmap` samples the part around the pointer into the fence.
//! Moving the pointer or changing the zoom only changes the source rectangle: no pixel passes
//! through the CPU, the plugin or the fence's script.
//!
//! One capture runs per monitor the pointer is on; the ones it left are stopped, and their frames
//! let go, after [`IDLE_STOP`]. A picture that does not follow (`set_live(false)`) stops every
//! capture and keeps the last frame of each monitor.

use crate::gpu_bindings as b;
use pecofence_platform::capture::{Capture, Frame};
use std::ffi::c_void;
use std::time::{Duration, Instant};
use windows_canvas::{DrawingSession, GpuDevice};
use windows_core::{IUnknown, Interface, Result};

/// A monitor the pointer left is not captured any more after this long.
const IDLE_STOP: Duration = Duration::from_secs(3);

/// What the Magnifier did, for the usage log.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Frames the captures delivered.
    pub frames: u64,
    /// Of those, frames that changed nothing in the part drawn last (Windows 11 24H2 tells what
    /// changed): they cost no redraw.
    pub unseen: u64,
    /// Draws done.
    pub draws: u64,
    /// Captures started.
    pub starts: u64,
}

/// One captured monitor.
struct Source {
    /// The `HMONITOR` as an integer.
    monitor: isize,
    /// None while the picture is held or after the capture failed.
    capture: Option<Capture>,
    /// The newest frame; kept so its buffer is not reused while the bitmap shows it.
    frame: Option<Frame>,
    bitmap: Option<b::ID2D1Bitmap>,
    /// The frame has no bitmap yet.
    fresh: bool,
    used: Instant,
}

/// The monitors, captured and ready to draw zoomed.
pub struct Magnifier {
    gpu: GpuDevice,
    sources: Vec<Source>,
    current: Option<usize>,
    cursor: bool,
    live: bool,
    /// What the last draw showed: the size drawn into (DIPs), the monitor's scale and the zoom.
    /// With the point the pointer is at, it is the part of the screen whose changes matter.
    last_draw: Option<((f32, f32), f32, f32)>,
    pub stats: Stats,
}

/// The part of a monitor to show, in the monitor's pixels: `[left, top, width, height]`.
///
/// `dest` is the size drawn into (DIPs of the fence), `centre` the point to keep in the middle
/// (the pointer, in the monitor's pixels), `src_scale` the monitor's scale (1.0 at 96 DPI) and
/// `zoom` how many times larger it is to look. A content that measures L DIPs takes L * src_scale
/// pixels on its monitor and L * zoom DIPs in the fence, whatever the two scales: "2x" is twice as
/// large to the eye also from a monitor of another DPI. The rectangle slides back inside the
/// monitor at its edges instead of shrinking; a monitor smaller than the rectangle is shown whole.
pub fn source_rect(
    monitor: (f32, f32),
    dest: (f32, f32),
    centre: (f32, f32),
    src_scale: f32,
    zoom: f32,
) -> [f32; 4] {
    let zoom = zoom.clamp(1.0, 20.0);
    let scale = if src_scale.is_finite() && src_scale > 0.0 {
        src_scale
    } else {
        1.0
    };
    let w = (dest.0 * scale / zoom).min(monitor.0).max(1.0);
    let h = (dest.1 * scale / zoom).min(monitor.1).max(1.0);
    let left = (centre.0 - w / 2.0).clamp(0.0, (monitor.0 - w).max(0.0));
    let top = (centre.1 - h / 2.0).clamp(0.0, (monitor.1 - h).max(0.0));
    [left, top, w, h]
}

impl Magnifier {
    pub fn new(gpu: GpuDevice) -> Self {
        Self {
            gpu,
            sources: Vec::new(),
            current: None,
            cursor: true,
            live: true,
            last_draw: None,
            stats: Stats::default(),
        }
    }

    /// The mouse pointer is part of the pictures (applies to the running captures too).
    pub fn set_cursor(&mut self, on: bool) {
        self.cursor = on;
        for s in &self.sources {
            if let Some(c) = &s.capture {
                c.set_cursor(on);
            }
        }
    }

    /// False stops every capture and keeps the last frames: a held picture costs the graphics
    /// card nothing. True lets `poll` start them again.
    pub fn set_live(&mut self, live: bool) {
        self.live = live;
        if !live {
            for s in &mut self.sources {
                s.capture = None;
            }
        }
    }

    /// Windows accepted capture without its yellow frame, for the running captures.
    pub fn borderless(&self) -> Option<bool> {
        self.sources
            .iter()
            .find_map(|s| s.capture.as_ref().map(|c| c.borderless))
    }

    /// Whether any monitor has a picture to draw.
    pub fn has_picture(&self) -> bool {
        self.sources.iter().any(|s| s.frame.is_some())
    }

    fn source(&mut self, monitor: isize) -> usize {
        match self.sources.iter().position(|s| s.monitor == monitor) {
            Some(i) => i,
            None => {
                self.sources.push(Source {
                    monitor,
                    capture: None,
                    frame: None,
                    bitmap: None,
                    fresh: false,
                    used: Instant::now(),
                });
                self.sources.len() - 1
            }
        }
    }

    /// Takes the newest frame of `monitor` (an `HMONITOR`), starting its capture when needed.
    /// True when the picture changed: a frame came and, where Windows says what changed, it
    /// changed something in the part around `centre` (the pointer, in the monitor's pixels) that
    /// the last draw showed. A frame that changed nothing there is kept for the next draw but
    /// costs no redraw. The monitors the pointer left are stopped after a while.
    pub fn poll(&mut self, monitor: *mut c_void, centre: Option<(f32, f32)>) -> Result<bool> {
        let now = Instant::now();
        let at = self.source(monitor as isize);
        self.current = Some(at);
        self.sources[at].used = now;
        let live = self.live;
        let mut started = Ok(());
        if live && self.sources[at].capture.is_none() {
            let device: IUnknown = self.gpu.d3d_device().cast()?;
            match Capture::monitor(monitor, &device, self.cursor) {
                Ok(c) => {
                    tracing::info!(
                        borderless = c.borderless,
                        "magnifier: a monitor is captured"
                    );
                    self.sources[at].capture = Some(c);
                    self.stats.starts += 1;
                }
                Err(e) => started = Err(e),
            }
        }
        let got = self.sources[at]
            .capture
            .as_mut()
            .and_then(|c| c.next_frame());
        let mut changed = got.is_some();
        if let Some(frame) = got {
            if let (Some((dest, scale, zoom)), Some(centre)) = (self.last_draw, centre) {
                let view = source_rect(
                    (frame.width as f32, frame.height as f32),
                    dest,
                    centre,
                    scale,
                    zoom,
                );
                if !frame.changed_in(view) {
                    changed = false;
                    self.stats.unseen += 1;
                }
            }
            let s = &mut self.sources[at];
            s.frame = Some(frame);
            s.fresh = true;
            self.stats.frames += 1;
        }
        // The others: stopped when idle (their last frame stays).
        // A monitor the pointer left: its capture stops, and its frame (a texture as large as the
        // monitor) is let go; the pointer coming back finds a new one within a frame or two.
        let current = self.current;
        for (i, s) in self.sources.iter_mut().enumerate() {
            if Some(i) != current && now.duration_since(s.used) > IDLE_STOP {
                s.capture = None;
                s.frame = None;
                s.bitmap = None;
            }
        }
        started.map(|()| changed)
    }

    /// The size in pixels of the monitor `poll` last looked at, once it has a frame.
    pub fn size(&self) -> Option<(u32, u32)> {
        let f = self.sources.get(self.current?)?.frame.as_ref()?;
        Some((f.width, f.height))
    }

    /// Draws the part of the current monitor around `centre` (its pixels), `zoom` times larger,
    /// into `dest` = [x, y, width, height] of the session (DIPs); `src_scale` is the monitor's
    /// scale (see [`source_rect`]). False when there is no picture yet.
    pub fn draw(
        &mut self,
        session: &DrawingSession<'_>,
        dest: [f32; 4],
        centre: (f32, f32),
        src_scale: f32,
        zoom: f32,
        smooth: bool,
    ) -> Result<bool> {
        let Some(at) = self.current else {
            return Ok(false);
        };
        let context: b::ID2D1DeviceContext = session.raw().cast()?;
        let s = &mut self.sources[at];
        let Some(frame) = &s.frame else {
            return Ok(false);
        };
        if frame.width == 0 || frame.height == 0 || dest[2] <= 0.0 || dest[3] <= 0.0 {
            return Ok(false);
        }
        if s.fresh || s.bitmap.is_none() {
            let surface: b::IDXGISurface = frame.surface.cast()?;
            let properties = b::D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: b::D2D1_PIXEL_FORMAT {
                    format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
                    // The capture's alpha channel is not meaningful.
                    alphaMode: b::D2D1_ALPHA_MODE_IGNORE,
                },
                // One DIP per pixel: the source rectangle is in the monitor's pixels.
                dpiX: 96.0,
                dpiY: 96.0,
                bitmapOptions: b::D2D1_BITMAP_OPTIONS_NONE,
                ..Default::default()
            };
            // SAFETY: `surface` is a live surface of the context's own device.
            let bitmap =
                unsafe { context.CreateBitmapFromDxgiSurface(&surface, Some(&properties))? };
            s.bitmap = Some(bitmap.cast()?);
            s.fresh = false;
        }
        let Some(bitmap) = &s.bitmap else {
            return Ok(false);
        };
        let [left, top, w, h] = source_rect(
            (frame.width as f32, frame.height as f32),
            (dest[2], dest[3]),
            centre,
            src_scale,
            zoom,
        );
        let source = b::D2D_RECT_F {
            left,
            top,
            right: left + w,
            bottom: top + h,
        };
        let target = b::D2D_RECT_F {
            left: dest[0],
            top: dest[1],
            right: dest[0] + dest[2],
            bottom: dest[1] + dest[3],
        };
        let mode = if smooth {
            b::D2D1_INTERPOLATION_MODE_LINEAR
        } else {
            b::D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR
        };
        // SAFETY: both rectangles live through the call; the bitmap belongs to this device.
        unsafe { context.DrawBitmap(bitmap, Some(&target), 1.0, mode, Some(&source), None) };
        self.last_draw = Some(((dest[2], dest[3]), src_scale, zoom));
        self.stats.draws += 1;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::source_rect;

    const MONITOR: (f32, f32) = (3840.0, 2160.0);

    #[test]
    fn the_part_shown_is_the_fence_divided_by_the_zoom() {
        // A 400 x 300 DIP fence at 100 %, 4x: 100 x 75 pixels around the pointer.
        let r = source_rect(MONITOR, (400.0, 300.0), (1000.0, 800.0), 1.0, 4.0);
        assert_eq!(r, [950.0, 762.5, 100.0, 75.0]);
        // 1x shows as many pixels as the fence has DIPs.
        let r = source_rect(MONITOR, (400.0, 300.0), (1000.0, 800.0), 1.0, 1.0);
        assert_eq!((r[2], r[3]), (400.0, 300.0));
    }

    #[test]
    fn a_monitor_of_another_scale_gives_the_same_size_to_the_eye() {
        // The content L DIPs wide takes L * scale pixels on its monitor and L * zoom DIPs in the
        // fence: 150 % source, 2x, a 400 DIP wide fence shows 400 * 1.5 / 2 = 300 pixels.
        let r = source_rect(MONITOR, (400.0, 300.0), (2000.0, 1000.0), 1.5, 2.0);
        assert_eq!((r[2], r[3]), (300.0, 225.0));
        // A 100 % source at the same zoom shows 200 pixels: the content is 1.5 times smaller on
        // the screen in pixels, and the fence draws 2 DIPs for each of its DIPs.
        let r = source_rect(MONITOR, (400.0, 300.0), (2000.0, 1000.0), 1.0, 2.0);
        assert_eq!((r[2], r[3]), (200.0, 150.0));
    }

    #[test]
    fn the_part_slides_back_inside_the_monitor_at_the_edges() {
        for centre in [(0.0, 0.0), (-50.0, -50.0)] {
            let r = source_rect(MONITOR, (400.0, 300.0), centre, 1.0, 4.0);
            assert_eq!(r, [0.0, 0.0, 100.0, 75.0]);
        }
        let r = source_rect(MONITOR, (400.0, 300.0), (3840.0, 2160.0), 1.0, 4.0);
        assert_eq!(r, [3740.0, 2085.0, 100.0, 75.0]);
        let r = source_rect(MONITOR, (400.0, 300.0), (9999.0, 9999.0), 1.0, 4.0);
        assert_eq!((r[0] + r[2], r[1] + r[3]), (3840.0, 2160.0));
    }

    #[test]
    fn a_part_larger_than_the_monitor_is_the_whole_monitor() {
        let r = source_rect((800.0, 600.0), (4000.0, 3000.0), (100.0, 100.0), 1.0, 1.0);
        assert_eq!(r, [0.0, 0.0, 800.0, 600.0]);
    }

    #[test]
    fn nonsense_in_gives_a_usable_rectangle() {
        let r = source_rect(MONITOR, (400.0, 300.0), (10.0, 10.0), f32::NAN, 0.0);
        assert_eq!((r[2], r[3]), (400.0, 300.0));
        let r = source_rect(MONITOR, (0.0, 0.0), (10.0, 10.0), 1.0, 4.0);
        assert!(r[2] >= 1.0 && r[3] >= 1.0);
        let r = source_rect(MONITOR, (400.0, 300.0), (10.0, 10.0), 1.0, 1000.0);
        assert_eq!((r[2], r[3]), (20.0, 15.0));
    }
}
