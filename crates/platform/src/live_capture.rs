//! Windows.Graphics.Capture: the pixels of a window that PrintWindow cannot read.
//!
//! Wallpaper Engine's scene player draws with Direct3D into a window without a redirection
//! bitmap (`WS_EX_NOREDIRECTIONBITMAP`), so `PrintWindow` returns an all-black picture or an
//! older wallpaper's. Windows.Graphics.Capture reads the picture Windows composes for the
//! window instead. When the player is a child of the desktop (`Progman` on Windows 11
//! 24H2+), its top-level window is read too: it holds the player and Explorer's icons, which
//! is what glass over the desktop would show anyway.

use crate::bindings::{HWND, RECT};
use crate::capture_bindings::*;
use crate::window::window_rect;
use std::time::{Duration, Instant};
use windows_core::Interface;

/// Why [`capture`] has no pixels: Windows.Graphics.Capture is missing, or it refused the window.
pub const UNAVAILABLE: &str = "the window cannot be read with Windows.Graphics.Capture";
/// Why [`capture`] has no pixels: a capture session started but presented no frame in time (a
/// player that is paused, or one that just started).
pub const NO_FRAME: &str = "Windows.Graphics.Capture returned no frame";

/// A player that just started can take a moment to present its first frame; a paused one may
/// present none, and then the last capture stays.
const FRAME_WAIT: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(10);

/// The window's rectangle and what `read` makes of its composed pixels (`width` x `height`
/// BGRA rows, alpha undefined). Reads `hwnd` itself when Windows.Graphics.Capture accepts it,
/// otherwise the top-level window that contains it. Call it off the UI thread: it starts a
/// capture session and copies one frame (some 50 to 150 ms for a 4K window). An error says why
/// there are no pixels; the caller falls back to a neutral backdrop.
pub fn capture<T>(
    hwnd: HWND,
    read: impl FnOnce(&[u8], u32, u32) -> T,
) -> Result<(RECT, T), &'static str> {
    let root = crate::desktop::root_ancestor(hwnd);
    let mut targets = vec![hwnd];
    if !root.0.is_null() && root != hwnd {
        targets.push(root);
    }
    // SAFETY: every COM object is released by its wrapper, and the pointers below come from
    // calls that returned success.
    unsafe {
        let (device, context) = device()?;
        let mut session_started = false;
        for target in targets {
            let Some(item) = item(target) else {
                continue;
            };
            let rect = window_rect(target);
            let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
            if w <= 0 || h <= 0 {
                continue;
            }
            match frame(&device, &context, &item, w as u32, h as u32) {
                Ok((pixels, width, height)) => return Ok((rect, read(&pixels, width, height))),
                Err(NO_FRAME) => session_started = true,
                Err(_) => {}
            }
        }
        Err(if session_started {
            NO_FRAME
        } else {
            UNAVAILABLE
        })
    }
}

/// A D3D11 device with the BGRA support Windows.Graphics.Capture needs, and its immediate
/// context.
unsafe fn device() -> Result<(ID3D11Device, ID3D11DeviceContext), &'static str> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut level = D3D_FEATURE_LEVEL_11_0;
    let levels = [
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    ];
    // SAFETY: the calls write the device and its context into the out parameters.
    unsafe {
        D3D11CreateDevice(
            std::ptr::null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT as u32,
            levels.as_ptr(),
            levels.len() as u32,
            D3D11_SDK_VERSION as u32,
            &mut device as *mut _ as *mut *mut core::ffi::c_void,
            &mut level,
            &mut context as *mut _ as *mut *mut core::ffi::c_void,
        )
        .ok()
        .map_err(|_| UNAVAILABLE)?;
    }
    match (device, context) {
        (Some(device), Some(context)) => Ok((device, context)),
        _ => Err(UNAVAILABLE),
    }
}

/// A capture item for `hwnd`, when Windows.Graphics.Capture accepts the window.
unsafe fn item(hwnd: HWND) -> Option<GraphicsCaptureItem> {
    static INTEROP: windows_core::imp::FactoryCache<
        GraphicsCaptureItem,
        IGraphicsCaptureItemInterop,
    > = windows_core::imp::FactoryCache::new();
    let interop = INTEROP.call(|factory| Ok(factory.clone())).ok()?;
    // SAFETY: the interop factory creates a GraphicsCaptureItem for the window.
    unsafe { interop.CreateForWindow(hwnd.0) }.ok()
}

/// One frame of `item` as BGRA rows, with the size Windows composed it at (usually `width` x
/// `height`).
unsafe fn frame(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    item: &GraphicsCaptureItem,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, u32, u32), &'static str> {
    let dxgi: IDXGIDevice = device.cast().map_err(|_| UNAVAILABLE)?;
    let mut direct3d: Option<windows_core::IInspectable> = None;
    // SAFETY: the call writes an IInspectable for the device into `direct3d`.
    unsafe {
        CreateDirect3D11DeviceFromDXGIDevice(
            windows_core::Interface::as_raw(&dxgi),
            &mut direct3d as *mut _ as *mut *mut core::ffi::c_void,
        )
        .ok()
        .map_err(|_| UNAVAILABLE)?;
    }
    let direct3d: IDirect3DDevice = direct3d
        .ok_or(UNAVAILABLE)?
        .cast()
        .map_err(|_| UNAVAILABLE)?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &direct3d,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        2,
        SizeInt32 {
            width: width as i32,
            height: height as i32,
        },
    )
    .map_err(|_| UNAVAILABLE)?;
    let session = pool.CreateCaptureSession(item).map_err(|_| UNAVAILABLE)?;
    // No pointer and no capture border around the player's window while we take the frame.
    if let Ok(session2) = session.cast::<IGraphicsCaptureSession2>() {
        let _ = session2.SetIsCursorCaptureEnabled(false);
    }
    if let Ok(session3) = session.cast::<IGraphicsCaptureSession3>() {
        let _ = session3.SetIsBorderRequired(false);
    }
    session.StartCapture().map_err(|_| UNAVAILABLE)?;
    let deadline = Instant::now() + FRAME_WAIT;
    let mut next = None;
    while Instant::now() < deadline {
        if let Ok(frame) = pool.TryGetNextFrame() {
            next = Some(frame);
            break;
        }
        std::thread::sleep(POLL);
    }
    let pixels = next
        .and_then(|frame| {
            let surface = frame.Surface().ok()?;
            // The surface is not a D3D11 texture itself; this asks it for the one behind it.
            let access: IDirect3DDxgiInterfaceAccess = surface.cast().ok()?;
            let texture: ID3D11Texture2D =
                unsafe { access.GetInterface::<ID3D11Texture2D>() }.ok()?;
            // SAFETY: the texture belongs to the frame, which is alive until here.
            unsafe { read_texture(device, context, &texture) }
        })
        .ok_or(NO_FRAME);
    if let Ok(closable) = session.cast::<IClosable>() {
        let _ = closable.Close();
    }
    if let Ok(closable) = pool.cast::<IClosable>() {
        let _ = closable.Close();
    }
    pixels
}

/// The pixels of `texture`, copied through a staging texture the CPU can read.
unsafe fn read_texture(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    texture: &ID3D11Texture2D,
) -> Option<(Vec<u8>, u32, u32)> {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    // SAFETY: the description is written into `desc`.
    unsafe { texture.GetDesc(&mut desc as *mut _) };
    let (width, height) = (desc.Width, desc.Height);
    if width == 0 || height == 0 {
        return None;
    }
    let mut staging_desc = desc;
    staging_desc.MipLevels = 1;
    staging_desc.ArraySize = 1;
    staging_desc.Usage = D3D11_USAGE_STAGING;
    staging_desc.BindFlags = 0;
    staging_desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ as u32;
    staging_desc.MiscFlags = 0;
    let mut staging: Option<ID3D11Texture2D> = None;
    // SAFETY: the staging texture is created with the frame's own size and format.
    unsafe { device.CreateTexture2D(&staging_desc, None, Some(&mut staging as *mut _)) }
        .ok()
        .ok()?;
    let staging = staging?;
    let destination: ID3D11Resource = staging.cast().ok()?;
    let source: ID3D11Resource = texture.cast().ok()?;
    // SAFETY: both resources have the same size and format.
    unsafe { context.CopyResource(&destination, &source) };
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: the staging texture is neither bound nor in use.
    unsafe {
        context
            .Map(
                &destination,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped as *mut _),
            )
            .ok()
            .ok()?
    };
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    // SAFETY: the mapped rows are `height` rows of `width` BGRA pixels, `RowPitch` apart.
    unsafe {
        for y in 0..height as usize {
            let from = (mapped.pData as *const u8).add(y * mapped.RowPitch as usize);
            let to = pixels.as_mut_ptr().add(y * width as usize * 4);
            std::ptr::copy_nonoverlapping(from, to, width as usize * 4);
        }
        context.Unmap(&destination, 0);
    }
    Some((pixels, width, height))
}
