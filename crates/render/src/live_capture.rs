//! GPU-only Windows Graphics Capture input for the liquid-glass renderer.
//!
//! This module only reads the selected window/monitor. It never changes a foreign window,
//! installs a hook, or reads pixels back to the CPU. The caller polls it on the rendering
//! thread, at its chosen rate, and supplies the same D3D11 device used by Direct2D.

#[path = "live_capture_bindings.rs"]
#[allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations
)]
mod b;

use pecofence_core::REALTIME_GLASS_FPS_MAX;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use windows_core::{Error, HRESULT, IUnknown, Interface, Result};

const E_INVALIDARG: HRESULT = HRESULT(0x80070057u32 as i32);
const E_POINTER: HRESULT = HRESULT(0x80004003u32 as i32);
const E_NOTIMPL: HRESULT = HRESULT(0x80004001u32 as i32);
const RO_E_CLOSED: HRESULT = HRESULT(0x80000013u32 as i32);
const FRAME_BUFFERS: i32 = 2;
// Reject pathological sources before asking Windows/the driver to allocate their buffers.
const MAX_DIMENSION: i32 = 16_384;
const MAX_PIXELS: i64 = 64 * 1024 * 1024;

fn checked_size(size: b::SizeInt32) -> Result<b::SizeInt32> {
    if size.Width <= 0
        || size.Height <= 0
        || size.Width > MAX_DIMENSION
        || size.Height > MAX_DIMENSION
        || i64::from(size.Width) * i64::from(size.Height) > MAX_PIXELS
    {
        return Err(Error::new(
            E_INVALIDARG,
            "capture source dimensions exceed the safety limit",
        ));
    }
    Ok(size)
}

/// The frame pool owns the surface until Close. Always return it, including on errors and
/// when dropping an older queued frame in favour of a newer one.
struct Frame(b::Direct3D11CaptureFrame);

impl Drop for Frame {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}

/// A bounded, latest-frame GPU capture. Create, poll, consume its texture and drop on the
/// same rendering thread: the application's Direct2D device is single-threaded.
///
/// `surface()` returns an IDXGISurface as IUnknown to avoid coupling consumers to these
/// private generated bindings. It is our own persistent texture, never a checked-out WGC
/// surface. Consumers may keep a Direct2D bitmap over it while its identity is unchanged.
pub struct LiveCapture {
    _render_thread: std::marker::PhantomData<std::rc::Rc<()>>,
    device: b::ID3D11Device,
    context: b::ID3D11DeviceContext,
    winrt_device: b::IDirect3DDevice,
    closed: Arc<AtomicBool>,
    _closed_revoker: windows_core::EventRevoker,
    _item: b::GraphicsCaptureItem,
    pool: b::Direct3D11CaptureFramePool,
    session: b::GraphicsCaptureSession,
    pool_size: b::SizeInt32,
    texture: Option<b::ID3D11Texture2D>,
    surface: Option<IUnknown>,
    texture_size: (i32, i32),
    /// Physical screen coordinates [left, top, width, height]. This is supplied by the
    /// window-discovery layer, which is responsible for updating it after topology changes.
    bounds: [i32; 4],
    content_size: (i32, i32),
    frame_seq: u64,
    last_frame_at: Option<Instant>,
    frame_timestamp_100ns: Option<i64>,
}

impl LiveCapture {
    /// Capture a top-level window. For wallpaper, target the verified desktop host rather
    /// than a renderer child (WGC can reject child HWNDs). `hwnd` is never modified.
    pub fn for_window(device: &IUnknown, hwnd: isize, bounds: [i32; 4]) -> Result<Self> {
        if hwnd == 0 {
            return Err(Error::new(E_INVALIDARG, "capture window is null"));
        }
        let interop =
            windows_core::factory::<b::GraphicsCaptureItem, b::IGraphicsCaptureItemInterop>()?;
        let item: b::GraphicsCaptureItem =
            unsafe { interop.CreateForWindow(b::HWND(hwnd as *mut core::ffi::c_void))? };
        Self::new(device, item, bounds)
    }

    /// Optional monitor-capture route. The caller must first successfully exclude all of
    /// its own visible windows using WDA_EXCLUDEFROMCAPTURE, or it can capture itself and
    /// create a feedback loop. This method does not change any window's display affinity.
    pub fn for_monitor(device: &IUnknown, monitor: isize, bounds: [i32; 4]) -> Result<Self> {
        if monitor == 0 {
            return Err(Error::new(E_INVALIDARG, "capture monitor is null"));
        }
        let interop =
            windows_core::factory::<b::GraphicsCaptureItem, b::IGraphicsCaptureItemInterop>()?;
        let item: b::GraphicsCaptureItem =
            unsafe { interop.CreateForMonitor(b::HMONITOR(monitor as *mut core::ffi::c_void))? };
        Self::new(device, item, bounds)
    }

    fn new(device: &IUnknown, item: b::GraphicsCaptureItem, bounds: [i32; 4]) -> Result<Self> {
        if !b::GraphicsCaptureSession::IsSupported()? {
            return Err(Error::new(
                E_NOTIMPL,
                "Windows Graphics Capture is unavailable",
            ));
        }
        let size = checked_size(item.Size()?)?;
        let device: b::ID3D11Device = device.cast()?;
        let context = unsafe { device.GetImmediateContext()? };
        let dxgi: b::IDXGIDevice = device.cast()?;
        let winrt_device: b::IDirect3DDevice =
            unsafe { b::CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? }.cast()?;
        let pool = b::Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt_device,
            b::DirectXPixelFormat::B8G8R8A8UIntNormalized,
            FRAME_BUFFERS,
            size,
        )?;
        let session = match pool.CreateCaptureSession(&item) {
            Ok(session) => session,
            Err(error) => {
                let _ = pool.Close();
                return Err(error);
            }
        };
        let closed = Arc::new(AtomicBool::new(false));
        let callback_closed = closed.clone();
        let closed_revoker = match item.Closed(move |_, _| {
            // WGC owns the event thread. No window, D2D or D3D work is permitted here.
            callback_closed.store(true, Ordering::Release);
        }) {
            Ok(revoker) => revoker,
            Err(error) => {
                let _ = session.Close();
                let _ = pool.Close();
                return Err(error);
            }
        };
        // These interfaces are optional on older supported Windows versions. Do not prompt
        // for borderless access or alter Windows privacy settings; the OS may retain its
        // capture border when this application does not have borderless permission.
        let _ = session.SetIsCursorCaptureEnabled(false);
        let _ = session.SetIsBorderRequired(false);
        // Newer Windows can throttle the producer as well as the consumer. The application
        // overrides this initial conservative limit with its user's chosen refresh rate.
        let _ = session.SetMinUpdateInterval(b::TimeSpan { Duration: 333_333 });
        if let Err(error) = session.StartCapture() {
            let _ = session.Close();
            let _ = pool.Close();
            return Err(error);
        }
        Ok(Self {
            _render_thread: std::marker::PhantomData,
            device,
            context,
            winrt_device,
            closed,
            _closed_revoker: closed_revoker,
            _item: item,
            pool,
            session,
            pool_size: size,
            texture: None,
            surface: None,
            texture_size: (0, 0),
            bounds,
            content_size: (size.Width, size.Height),
            frame_seq: 0,
            last_frame_at: None,
            frame_timestamp_100ns: None,
        })
    }

    /// Non-blocking. Discard stale queued frames and copy only the latest available image
    /// into a persistent, shader-readable GPU texture. Returns true only after a copy.
    ///
    /// The consumer should redraw its existing effect graph on true; it must invalidate
    /// cached Direct2D effect inputs because the persistent bitmap's identity is unchanged.
    pub fn poll_latest(&mut self) -> Result<bool> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::new(RO_E_CLOSED, "capture source window was closed"));
        }
        let mut latest = None;
        // A finite drain also bounds work if a fast producer happens to replenish the pool.
        for _ in 0..4 {
            let mut raw = core::ptr::null_mut();
            // TryGetNextFrame is explicitly nullable. Use its ABI instead of a generated
            // non-null projection: windows-core versions map S_OK + null to different
            // errors, including an Error carrying S_OK. A genuine failing HRESULT must
            // always propagate, while a successful null result only means "no frame".
            unsafe {
                (Interface::vtable(&self.pool).TryGetNextFrame)(self.pool.as_raw(), &mut raw)
                    .ok()?;
            }
            if raw.is_null() {
                break;
            }
            latest = Some(Frame(unsafe { b::Direct3D11CaptureFrame::from_raw(raw) }));
        }
        let Some(latest) = latest else {
            return Ok(false);
        };
        let size = checked_size(latest.0.ContentSize()?)?;
        self.content_size = (size.Width, size.Height);
        if size != self.pool_size {
            // All checked-out frames must be returned before Recreate. A frame from a
            // growing source can be clipped by the old pool; wait for a full new frame.
            drop(latest);
            self.pool.Recreate(
                &self.winrt_device,
                b::DirectXPixelFormat::B8G8R8A8UIntNormalized,
                FRAME_BUFFERS,
                size,
            )?;
            self.pool_size = size;
            self.texture = None;
            self.surface = None;
            self.texture_size = (0, 0);
            return Ok(false);
        }

        let winrt_surface = latest.0.Surface()?;
        let access: b::IDirect3DDxgiInterfaceAccess = winrt_surface.cast()?;
        let source: b::ID3D11Texture2D = unsafe { access.GetInterface()? };
        let mut source_desc = b::D3D11_TEXTURE2D_DESC::default();
        unsafe { source.GetDesc(&mut source_desc) };
        if source_desc.Format != b::DXGI_FORMAT_B8G8R8A8_UNORM
            || source_desc.Width < size.Width as u32
            || source_desc.Height < size.Height as u32
        {
            return Err(Error::new(
                E_INVALIDARG,
                "capture frame surface format or size is invalid",
            ));
        }
        if self.texture_size != (size.Width, size.Height) || self.texture.is_none() {
            let desc = b::D3D11_TEXTURE2D_DESC {
                Width: size.Width as u32,
                Height: size.Height as u32,
                MipLevels: 1,
                ArraySize: 1,
                Format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: b::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: b::D3D11_USAGE_DEFAULT,
                BindFlags: (b::D3D11_BIND_SHADER_RESOURCE | b::D3D11_BIND_RENDER_TARGET) as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let mut texture = None;
            unsafe {
                self.device
                    .CreateTexture2D(&desc, None, Some(&mut texture))
                    .ok()?
            };
            let texture = texture.ok_or_else(|| Error::from_hresult(E_POINTER))?;
            let surface: b::IDXGISurface = texture.cast()?;
            self.surface = Some(surface.cast()?);
            self.texture = Some(texture);
            self.texture_size = (size.Width, size.Height);
        }
        let region = b::D3D11_BOX {
            left: 0,
            top: 0,
            front: 0,
            right: size.Width as u32,
            bottom: size.Height as u32,
            back: 1,
        };
        unsafe {
            self.context.CopySubresourceRegion(
                self.texture.as_ref().unwrap(),
                0,
                0,
                0,
                0,
                &source,
                0,
                Some(&region),
            );
        }
        self.frame_timestamp_100ns = latest.0.SystemRelativeTime().ok().map(|t| t.Duration);
        self.last_frame_at = Some(Instant::now());
        self.frame_seq = self.frame_seq.wrapping_add(1);
        // Frame::drop returns the producer surface only after the GPU copy was submitted.
        Ok(true)
    }

    pub fn surface(&self) -> Option<IUnknown> {
        self.surface.clone()
    }

    /// Limit capture production on Windows versions that expose MinUpdateInterval. The
    /// caller must also pace poll_latest; on older Windows this optional interface returns
    /// E_NOINTERFACE and consumer-side pacing remains in effect.
    pub fn set_max_fps(&self, fps: u32) -> Result<()> {
        self.session.SetMinUpdateInterval(b::TimeSpan {
            Duration: 10_000_000 / i64::from(fps.clamp(1, REALTIME_GLASS_FPS_MAX)),
        })
    }

    pub fn frame_seq(&self) -> u64 {
        self.frame_seq
    }

    pub fn content_size(&self) -> (i32, i32) {
        self.content_size
    }

    pub fn bounds(&self) -> [i32; 4] {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: [i32; 4]) {
        self.bounds = bounds;
    }

    pub fn last_frame_at(&self) -> Option<Instant> {
        self.last_frame_at
    }

    /// Source frame time in QPC-relative 100 ns ticks, for diagnostics/latency accounting.
    pub fn frame_timestamp_100ns(&self) -> Option<i64> {
        self.frame_timestamp_100ns
    }
}

impl Drop for LiveCapture {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}
