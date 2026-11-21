//! Screen capture for the Magnifier: Windows.Graphics.Capture on one monitor. Frames stay on the
//! GPU as DXGI surfaces of the caller's Direct3D 11 device; no pixel is ever read back by the
//! CPU. The frame pool is free-threaded and read by polling ([`Capture::next_frame`]) on the
//! caller's thread, so no event or dispatcher queue is needed.

use crate::capture_bindings as b;
use std::ffi::c_void;
use std::sync::OnceLock;
use windows_core::{IUnknown, Interface, Result};

/// Buffers in the frame pool: one being read, one being written.
const BUFFERS: i32 = 2;

/// One captured frame, the newest at the time of the call. Dropping it gives its buffer back to
/// the pool.
pub struct Frame {
    frame: b::Direct3D11CaptureFrame,
    /// The frame's `IDXGISurface` (also an `ID3D11Texture2D`), on the capture's device.
    pub surface: IUnknown,
    /// The monitor's size in pixels when the frame was taken.
    pub width: u32,
    pub height: u32,
    /// The parts of the screen that changed since the frame [`Capture::next_frame`] returned
    /// before, as `[x, y, width, height]` in pixels (the frames it skipped are added). None
    /// when Windows does not say (before Windows 11 24H2): everything may have changed.
    pub dirty: Option<Vec<[i32; 4]>>,
}

impl Frame {
    /// Whether anything inside `view` = `[left, top, width, height]` (pixels) changed. True when
    /// Windows does not say what changed.
    pub fn changed_in(&self, view: [f32; 4]) -> bool {
        let Some(dirty) = &self.dirty else {
            return true;
        };
        let (right, bottom) = (view[0] + view[2], view[1] + view[3]);
        dirty.iter().any(|r| {
            (r[0] as f32) < right
                && (r[0] + r[2]) as f32 > view[0]
                && (r[1] as f32) < bottom
                && (r[1] + r[3]) as f32 > view[1]
        })
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        let _ = self.frame.Close();
    }
}

/// A running capture of one monitor.
pub struct Capture {
    pool: b::Direct3D11CaptureFramePool,
    session: b::GraphicsCaptureSession,
    device: b::IDirect3DDevice,
    size: b::SizeInt32,
    /// Windows accepted `IsBorderRequired = false`: no yellow frame around the screen.
    pub borderless: bool,
}

/// Whether Windows lets this process capture without its yellow frame. Asked once: the answer
/// holds for the process, and the wait must not be repeated each time a capture starts. A
/// desktop app outside a package may be refused; the capture then shows its frame.
fn borderless_granted() -> bool {
    static GRANTED: OnceLock<bool> = OnceLock::new();
    *GRANTED.get_or_init(request_borderless)
}

fn request_borderless() -> bool {
    let Ok(op) =
        b::GraphicsCaptureAccess::RequestAccessAsync(b::GraphicsCaptureAccessKind::Borderless)
    else {
        return false;
    };
    let (tx, rx) = std::sync::mpsc::channel();
    if op
        .when(move |r| {
            let _ = tx.send(r.is_ok());
        })
        .is_err()
    {
        return false;
    }
    // Waits on a thread-pool completion: never longer than a couple of seconds.
    rx.recv_timeout(std::time::Duration::from_secs(2))
        .unwrap_or(false)
}

impl Capture {
    /// Starts capturing `monitor` (an `HMONITOR`) with `device`, an `ID3D11Device` made with
    /// BGRA support. `cursor` puts the mouse pointer in the frames.
    pub fn monitor(monitor: *mut c_void, device: &IUnknown, cursor: bool) -> Result<Self> {
        let dxgi: b::IDXGIDevice = device.cast()?;
        // The capture's own threads use the device too. This stays on for the app's device after
        // the capture stops: the cost is a lock around each of its calls.
        if let Ok(mt) = device.cast::<b::ID3D11Multithread>() {
            // SAFETY: plain COM call on a live device.
            let _ = unsafe { mt.SetMultithreadProtected(true) };
        }
        // SAFETY: `dxgi` is a live device interface; the call only wraps it.
        let inspectable = unsafe { b::CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? };
        let winrt_device: b::IDirect3DDevice = inspectable.cast()?;

        let interop =
            windows_core::factory::<b::GraphicsCaptureItem, b::IGraphicsCaptureItemInterop>()?;
        // SAFETY: `monitor` is a monitor handle the caller got from the system.
        let item: b::GraphicsCaptureItem =
            unsafe { interop.CreateForMonitor(b::HMONITOR(monitor))? };
        let size = item.Size()?;
        let pool = b::Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt_device,
            b::DirectXPixelFormat::B8G8R8A8UIntNormalized,
            BUFFERS,
            size,
        )?;
        let session = pool.CreateCaptureSession(&item)?;
        let _ = session.SetIsCursorCaptureEnabled(cursor);
        let borderless = borderless_granted() && session.SetIsBorderRequired(false).is_ok();
        session.StartCapture()?;
        Ok(Self {
            pool,
            session,
            device: winrt_device,
            size,
            borderless,
        })
    }

    /// Shows or hides the mouse pointer in the frames.
    pub fn set_cursor(&self, on: bool) {
        let _ = self.session.SetIsCursorCaptureEnabled(on);
    }

    /// The newest frame since the last call, or `None` when the screen did not change (the
    /// system sends frames only when something moved). Older waiting frames are given back.
    pub fn next_frame(&mut self) -> Option<Frame> {
        let mut newest: Option<Frame> = None;
        // What changed in the frames given back here too: the picture the caller sees skips them.
        let mut dirty: Option<Vec<[i32; 4]>> = Some(Vec::new());
        while let Ok(frame) = self.pool.TryGetNextFrame() {
            let this = frame.DirtyRegions().ok().map(|v| {
                (&v).into_iter()
                    .map(|r| [r.X, r.Y, r.Width, r.Height])
                    .collect::<Vec<_>>()
            });
            dirty = dirty.zip(this).map(|(mut all, more)| {
                all.extend(more);
                all
            });
            let content = frame.ContentSize().unwrap_or(self.size);
            let surface = frame
                .Surface()
                .ok()
                .and_then(|s| s.cast::<b::IDirect3DDxgiInterfaceAccess>().ok())
                // SAFETY: asks the surface for its own DXGI interface.
                .and_then(|a| unsafe { a.GetInterface::<b::IDXGISurface>() }.ok());
            let Some(surface) = surface else {
                let _ = frame.Close();
                continue;
            };
            if (content.Width, content.Height) != (self.size.Width, self.size.Height) {
                // The monitor changed its resolution: the next frames come in the new size.
                self.size = content;
                let _ = self.pool.Recreate(
                    &self.device,
                    b::DirectXPixelFormat::B8G8R8A8UIntNormalized,
                    BUFFERS,
                    content,
                );
            }
            newest = Some(Frame {
                frame,
                surface: surface.cast::<IUnknown>().ok()?,
                width: content.Width.max(0) as u32,
                height: content.Height.max(0) as u32,
                dirty: dirty.clone(),
            });
        }
        newest
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}
