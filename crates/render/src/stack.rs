//! The per-thread composition stack: dispatcher queue, compositor, GPU device.

use pecofence_platform::HWND;
use std::cell::RefCell;
use windows_canvas::{GpuDevice, RenderTarget};
use windows_composition::{
    CompositionGraphicsDevice, Compositor, DesktopWindowTarget, DispatcherQueueController,
};
use windows_core::Result;

enum QueueOwner {
    /// Queue created by `windows-composition` (DQTAT_COM_ASTA).
    Wrapper(#[allow(dead_code)] DispatcherQueueController),
    /// Queue created by our own binding with DQTAT_COM_NONE (used when the wrapper fails on
    /// an OLE-initialized thread).
    Platform(#[allow(dead_code)] pecofence_platform::dispatcher::DispatcherQueue),
}

/// Owns everything needed to composite on the current (UI) thread.
///
/// Create it once, after `OleGuard::init()`, and keep it alive for the process lifetime.
pub struct RenderStack {
    _queue: QueueOwner,
    pub compositor: Compositor,
    /// The D3D/D2D device and the composition graphics device built on it. Replaced as a unit
    /// after `DXGI_ERROR_DEVICE_REMOVED` (see [`RenderStack::recover`]).
    gpu: RefCell<Gpu>,
    /// True when Direct3D fell back to the WARP software rasterizer at start-up.
    pub is_warp: bool,
    /// Which dispatcher-queue path was used ("wrapper" or "platform").
    pub queue_path: &'static str,
}

struct Gpu {
    device: GpuDevice,
    graphics: CompositionGraphicsDevice,
    generation: u32,
}

thread_local! {
    static DRAW_HOOK: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
}

/// Called after every panel draw on this thread (the app uses it to schedule [`RenderStack::trim`]
/// once drawing goes quiet). Must be cheap and must not draw.
pub fn set_draw_hook(hook: impl Fn() + 'static) {
    DRAW_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
}

pub(crate) fn note_draw() {
    DRAW_HOOK.with(|h| {
        if let Ok(hook) = h.try_borrow()
            && let Some(hook) = hook.as_ref()
        {
            hook();
        }
    });
}

fn create_gpu(compositor: &Compositor, generation: u32) -> Result<(Gpu, bool)> {
    let (device, is_warp) = match GpuDevice::new() {
        Ok(d) => (d, false),
        Err(e) => {
            tracing::warn!(error = %e, "hardware D3D11 device failed; using WARP");
            (GpuDevice::new_warp()?, true)
        }
    };
    let graphics = device.create_graphics_device(compositor)?;
    Ok((
        Gpu {
            device,
            graphics,
            generation,
        },
        is_warp,
    ))
}

impl RenderStack {
    pub fn new() -> Result<Self> {
        let (queue, queue_path) = match DispatcherQueueController::create_on_current_thread() {
            Ok(q) => (QueueOwner::Wrapper(q), "wrapper(DQTAT_COM_ASTA)"),
            Err(e) => {
                tracing::warn!(error = %e, "composition dispatcher queue failed; using DQTAT_COM_NONE fallback");
                (
                    QueueOwner::Platform(
                        pecofence_platform::dispatcher::DispatcherQueue::create_on_current_thread(
                        )?,
                    ),
                    "platform(DQTAT_COM_NONE)",
                )
            }
        };
        let compositor = Compositor::new()?;
        let (gpu, is_warp) = create_gpu(&compositor, 0)?;
        Ok(Self {
            _queue: queue,
            compositor,
            gpu: RefCell::new(gpu),
            is_warp,
            queue_path,
        })
    }

    /// Idle release of GPU scratch memory: Direct2D's intermediate and effect-output caches,
    /// then the driver's internal buffers (`IDXGIDevice3::Trim`, after `ClearState` as its
    /// documentation requires). Uploaded bitmaps and drawing surfaces are kept; the next draw
    /// rebuilds the caches, so call this only once rendering has gone quiet.
    pub fn trim(&self) {
        use crate::gpu_bindings as b;
        use windows_core::Interface;
        let gpu = self.gpu.borrow();
        let result = (|| -> Result<()> {
            let d2d: b::ID2D1Device = gpu.device.d2d_device().cast()?;
            let d3d: b::ID3D11Device = gpu.device.d3d_device().cast()?;
            let dxgi: b::IDXGIDevice3 = d3d.cast()?;
            unsafe {
                d2d.ClearResources(0);
                d3d.GetImmediateContext()?.ClearState();
                dxgi.Trim();
            }
            Ok(())
        })();
        match result {
            Ok(()) => tracing::debug!("GPU caches trimmed"),
            Err(error) => tracing::debug!(%error, "GPU trim unavailable"),
        }
    }

    /// The adapter the fences render on, for feedback reports: its name and the version of its
    /// user-mode driver (`AMD Radeon(TM) Graphics, driver 32.0.21025.10016`).
    pub fn adapter(&self) -> Option<String> {
        use crate::gpu_bindings as b;
        use windows_core::Interface;
        let gpu = self.gpu.borrow();
        let dxgi: b::IDXGIDevice = gpu.device.d3d_device().cast().ok()?;
        // SAFETY: plain DXGI queries on a live device; `desc` is a local of the stated type.
        unsafe {
            let adapter = dxgi.GetAdapter().ok()?;
            let mut desc = b::DXGI_ADAPTER_DESC::default();
            adapter.GetDesc(&mut desc).ok().ok()?;
            let len = desc.Description.iter().position(|&c| c == 0);
            let name = String::from_utf16_lossy(&desc.Description[..len.unwrap_or(128)]);
            // DXGI answers for the D3D10-era IDXGIDevice with the driver's version (the one
            // Device Manager shows); it refuses the newer interfaces.
            let driver = match adapter.CheckInterfaceSupport(&b::IDXGIDevice::IID) {
                Ok(v) => {
                    let v = v as u64;
                    format!(
                        "{}.{}.{}.{}",
                        v >> 48,
                        (v >> 32) & 0xFFFF,
                        (v >> 16) & 0xFFFF,
                        v & 0xFFFF
                    )
                }
                Err(_) => "unknown".to_string(),
            };
            Some(format!("{}, driver {driver}", name.trim()))
        }
    }

    /// The current composition graphics device (allocates drawing surfaces).
    pub fn graphics(&self) -> CompositionGraphicsDevice {
        self.gpu.borrow().graphics.clone()
    }

    /// Increments every time the device is rebuilt; surfaces created under an older generation
    /// are dead and must be recreated.
    pub fn device_generation(&self) -> u32 {
        self.gpu.borrow().generation
    }

    /// Rebuilds the D3D/D2D device and the composition graphics device after device loss
    /// (sleep, driver update). Callers must then recreate every drawing surface.
    pub fn recover(&self) -> Result<()> {
        let generation = self.gpu.borrow().generation + 1;
        let (gpu, is_warp) = create_gpu(&self.compositor, generation)?;
        tracing::warn!(generation, is_warp, "GPU device rebuilt after device loss");
        *self.gpu.borrow_mut() = gpu;
        Ok(())
    }

    /// An off-screen Direct2D target of `width` x `height` device pixels on the current GPU
    /// device, for one-off rasterisation with CPU read-back (the shell drag image).
    pub fn create_render_target(&self, width: u32, height: u32) -> Result<RenderTarget> {
        self.gpu.borrow().device.create_render_target(width, height)
    }

    /// Creates a composition target for a window created on this thread.
    pub fn create_target(&self, hwnd: HWND) -> Result<DesktopWindowTarget> {
        // SAFETY: the caller guarantees `hwnd` is a live window owned by this thread.
        unsafe {
            self.compositor
                .create_desktop_window_target_for_hwnd(pecofence_platform::hwnd_ptr(hwnd), false)
        }
    }
}
