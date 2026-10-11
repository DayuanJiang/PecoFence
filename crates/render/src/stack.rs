//! The per-thread composition stack: dispatcher queue, compositor, GPU device.

use pecofence_platform::HWND;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows_canvas::{GpuDevice, RenderTarget};
use windows_composition::{
    CompositionGraphicsDevice, Compositor, DesktopWindowTarget, DispatcherQueueController,
};
use windows_core::{Interface, Result};

/// Keeps the first commit alive and records presentation completion independently of the UI
/// message queue. A missed/reentrant notification can therefore be retried safely.
pub struct PendingPresentation {
    _commit: windows_future::IAsyncAction,
    result: Arc<Mutex<Option<Result<()>>>>,
    pub started: Instant,
}

impl PendingPresentation {
    pub fn take_result(&self) -> Option<Result<()>> {
        self.result.lock().unwrap_or_else(|e| e.into_inner()).take()
    }
}

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
    let started = Instant::now();
    let (device, is_warp) = match GpuDevice::new() {
        Ok(d) => (d, false),
        Err(e) => {
            tracing::warn!(error = %e, "hardware D3D11 device failed; using WARP");
            (GpuDevice::new_warp()?, true)
        }
    };
    tracing::info!(
        elapsed_ms = started.elapsed().as_millis(),
        is_warp,
        "render phase: graphics device ready"
    );
    let started = Instant::now();
    let graphics = device.create_graphics_device(compositor)?;
    tracing::info!(
        elapsed_ms = started.elapsed().as_millis(),
        "render phase: composition graphics device ready"
    );
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
        let started = Instant::now();
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
        tracing::info!(
            elapsed_ms = started.elapsed().as_millis(),
            "render phase: dispatcher queue ready"
        );
        let started = Instant::now();
        let compositor = Compositor::new()?;
        tracing::info!(
            elapsed_ms = started.elapsed().as_millis(),
            "render phase: compositor ready"
        );
        let (gpu, is_warp) = create_gpu(&compositor, 0)?;
        Ok(Self {
            _queue: queue,
            compositor,
            gpu: RefCell::new(gpu),
            is_warp,
            queue_path,
        })
    }

    /// Commits the visual tree, then waits for DWM presentation away from the UI thread.
    /// The caller must keep pumping messages: RequestCommitAsync completion is dispatched by
    /// the window loop. ShowWindow alone does not mean a Composition surface is on screen.
    pub fn present_first_frame(
        &self,
        notify: impl FnOnce() + Send + 'static,
    ) -> Result<PendingPresentation> {
        let compositor: crate::comp::Compositor = self.compositor.as_raw().cast()?;
        let commit = compositor.RequestCommitAsync()?;
        let started = Instant::now();
        let result = Arc::new(Mutex::new(None));
        let completed = result.clone();
        let completion = Arc::new(Mutex::new(Some(notify)));
        commit.when(move |outcome| {
            if let Err(error) = outcome {
                *completed.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(error));
                if let Some(notify) = completion.lock().unwrap_or_else(|e| e.into_inner()).take() {
                    notify();
                }
                return;
            }
            tracing::info!(
                elapsed_ms = started.elapsed().as_millis(),
                "startup phase: composition commit completed"
            );
            let worker_result = completed.clone();
            let worker_notify = completion.clone();
            if let Err(error) = std::thread::Builder::new()
                .name("pecofence-first-frame".into())
                .spawn(move || {
                    let outcome = pecofence_platform::dwm::flush();
                    *worker_result.lock().unwrap_or_else(|e| e.into_inner()) = Some(outcome);
                    if let Some(notify) = worker_notify
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .take()
                    {
                        notify();
                    }
                })
            {
                tracing::warn!(%error, "first-frame presentation worker failed to start");
                *completed.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                    windows_core::Error::from_hresult(windows_core::HRESULT(0x80004005u32 as i32)),
                ));
                if let Some(notify) = completion.lock().unwrap_or_else(|e| e.into_inner()).take() {
                    notify();
                }
            }
        })?;
        Ok(PendingPresentation {
            _commit: commit,
            result,
            started,
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
