//! 脚本栅栏: native video for a script fence's `video=` row. Windows' Media Foundation media
//! engine (the one behind the video element of browsers) decodes the file on the graphics card,
//! plays its sound, and in frame-server mode hands each new picture to a Direct3D texture of
//! the render stack's own device; Direct2D then paints that texture into the fence like any
//! bitmap. No picture goes through the processor or the disk.
//!
//! Everything here lives on the UI thread. The engine reports on its own threads only through
//! [`Shared`]'s atomics, which [`VideoPlayer::state`] reads.

use crate::video_bindings as b;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};
use windows_core::{BSTR, Error, GUID, HRESULT, Interface, Result, implement};

/// Attribute keys of the media engine's creation attributes (`mfmediaengine.h`; the Windows
/// metadata does not carry them).
const MF_MEDIA_ENGINE_CALLBACK: GUID = GUID::from_u128(0xc60381b8_83a4_41f8_a3d0_de05076849a9);
const MF_MEDIA_ENGINE_DXGI_MANAGER: GUID = GUID::from_u128(0x065702da_1094_486d_8617_ee7cc4ee4648);
const MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT: GUID =
    GUID::from_u128(0x5066893c_8cf9_42bc_8b8a_472212e52726);
const CLSID_MF_MEDIA_ENGINE_CLASS_FACTORY: GUID =
    GUID::from_u128(0xb44392da_499b_446b_a4cb_005fead0e6d5);

/// The picture is kept at most this big: a fence is never wider, and the graphics card scales
/// the video down for the cost of a blit.
const MAX_WIDTH: u32 = 1920;
const MAX_HEIGHT: u32 = 1080;

/// E_FAIL: what a call that gave nothing back is reported as.
const E_FAIL: HRESULT = HRESULT(0x8000_4005_u32 as i32);

/// What the engine's callback leaves for the UI thread (bits of [`Shared::flags`]).
const LOADED: u32 = 1;
const FAILED: u32 = 2;

/// The engine calls back on threads of its own: it only sets these.
#[derive(Default)]
struct Shared {
    flags: AtomicU32,
    /// `MF_MEDIA_ENGINE_ERR_*` of the failure.
    error: AtomicU32,
    /// The failure's HRESULT, when the engine gave one.
    hresult: AtomicU32,
}

#[implement(b::IMFMediaEngineNotify)]
struct Notify(Arc<Shared>);

impl b::IMFMediaEngineNotify_Impl for Notify_Impl {
    fn EventNotify(&self, event: u32, param1: usize, param2: u32) -> Result<()> {
        let shared = &self.0;
        match event as i32 {
            b::MF_MEDIA_ENGINE_EVENT_LOADEDMETADATA => {
                shared.flags.fetch_or(LOADED, Ordering::SeqCst);
            }
            b::MF_MEDIA_ENGINE_EVENT_ERROR => {
                shared.error.store(param1 as u32, Ordering::SeqCst);
                shared.hresult.store(param2, Ordering::SeqCst);
                shared.flags.fetch_or(FAILED, Ordering::SeqCst);
            }
            _ => {}
        }
        Ok(())
    }
}

/// How the video stands, from what the engine reported.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VideoState {
    /// The file's headers are read: its length and picture size are known.
    pub loaded: bool,
    /// It played to its end.
    pub ended: bool,
    /// It cannot be played: `(MF_MEDIA_ENGINE_ERR_*, HRESULT)`. `4` is "not supported" (a
    /// codec Windows lacks), `3` a broken stream.
    pub failed: Option<(u32, i32)>,
    pub paused: bool,
    /// Seconds played.
    pub position: f64,
    /// Seconds in all, 0 while unknown (or for a stream without an end).
    pub duration: f64,
}

/// Starts Media Foundation once per process (it is never shut down: the process ends with it).
fn start_media_foundation() -> Result<()> {
    static START: OnceLock<HRESULT> = OnceLock::new();
    // SAFETY: a plain initialisation call.
    START
        .get_or_init(|| unsafe { b::MFStartup(b::MF_VERSION as u32, b::MFSTARTUP_FULL as u32) })
        .ok()
}

/// A media engine playing one file at a time into a texture. Dropping it stops the sound and
/// frees the engine.
pub struct VideoPlayer {
    engine: b::IMFMediaEngine,
    device: b::ID3D11Device,
    shared: Arc<Shared>,
    frame: Option<VideoFrame>,
    /// A new picture came while none was taken (the fence out of view): the next taking copies
    /// the current one even if no newer one has come by then.
    missed: bool,
    /// Kept alive for the engine, which holds the device through it.
    _manager: b::IMFDXGIDeviceManager,
    _notify: b::IMFMediaEngineNotify,
}

impl VideoPlayer {
    /// A player that starts loading `path`, and plays it once loaded when `playing`. It draws with
    /// `device`, the Direct3D 11 device of the render stack (anything that can be asked for an
    /// `ID3D11Device`, made with BGRA support).
    pub fn open(device: &impl Interface, path: &str, playing: bool) -> Result<Self> {
        let player = Self::create(device)?;
        let url = BSTR::from(path);
        // SAFETY: the engine is live and `url` outlives the call.
        unsafe {
            player.engine.SetSource(&url).ok()?;
            if playing {
                player.engine.Play().ok()?;
            }
        }
        Ok(player)
    }

    fn create(device: &impl Interface) -> Result<Self> {
        start_media_foundation()?;
        let device: b::ID3D11Device = device.cast()?;
        // The engine decodes on threads of its own and uses the device from them.
        let multithread: b::ID3D10Multithread = device.cast()?;
        // SAFETY: plain COM calls on live objects owned here; `token` and `manager` are locals
        // the calls fill.
        unsafe {
            let _ = multithread.SetMultithreadProtected(true);
            let mut token = 0u32;
            let mut manager: Option<b::IMFDXGIDeviceManager> = None;
            b::MFCreateDXGIDeviceManager(&mut token, &mut manager).ok()?;
            let manager = manager.ok_or_else(|| Error::from(E_FAIL))?;
            manager.ResetDevice(&device, token).ok()?;

            let shared = Arc::new(Shared::default());
            let notify: b::IMFMediaEngineNotify = Notify(shared.clone()).into();

            let mut attributes: Option<b::IMFAttributes> = None;
            b::MFCreateAttributes(&mut attributes, 3).ok()?;
            let attributes = attributes.ok_or_else(|| Error::from(E_FAIL))?;
            attributes
                .SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, &notify)
                .ok()?;
            attributes
                .SetUnknown(&MF_MEDIA_ENGINE_DXGI_MANAGER, &manager)
                .ok()?;
            attributes
                .SetUINT32(
                    &MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT,
                    b::DXGI_FORMAT_B8G8R8A8_UNORM as u32,
                )
                .ok()?;
            let factory: b::IMFMediaEngineClassFactory = b::CoCreateInstance(
                &CLSID_MF_MEDIA_ENGINE_CLASS_FACTORY,
                None::<&windows_core::IUnknown>,
                b::CLSCTX_INPROC_SERVER,
            )?;
            let engine = factory.CreateInstance(0, &attributes)?;
            Ok(Self {
                engine,
                device,
                shared,
                frame: None,
                missed: false,
                _manager: manager,
                _notify: notify,
            })
        }
    }

    /// Plays or pauses; a call that changes nothing is skipped.
    pub fn set_playing(&self, playing: bool) {
        // SAFETY: plain calls on the live engine.
        unsafe {
            let paused = self.engine.IsPaused().as_bool();
            if playing && paused {
                let _ = self.engine.Play();
            } else if !playing && !paused {
                let _ = self.engine.Pause();
            }
        }
    }

    /// 0 (silent) to 1.
    pub fn set_volume(&self, volume: f32) {
        // SAFETY: a plain call on the live engine.
        unsafe {
            let volume = f64::from(volume.clamp(0.0, 1.0));
            if (self.engine.GetVolume() - volume).abs() > 0.004 {
                let _ = self.engine.SetVolume(volume);
            }
        }
    }

    /// The playback speed, 1 being normal (the engine keeps the pitch of the sound).
    pub fn set_rate(&self, rate: f32) {
        // SAFETY: plain calls on the live engine.
        unsafe {
            let rate = f64::from(rate.clamp(0.25, 4.0));
            if (self.engine.GetPlaybackRate() - rate).abs() > 0.001 {
                // The default rate too: a file opened later starts at it.
                let _ = self.engine.SetDefaultPlaybackRate(rate);
                let _ = self.engine.SetPlaybackRate(rate);
            }
        }
    }

    /// Moves to `seconds` (the end of a finished video included: it plays again from there).
    pub fn seek(&self, seconds: f64) {
        // SAFETY: a plain call on the live engine.
        unsafe {
            let _ = self.engine.SetCurrentTime(seconds.max(0.0));
        }
    }

    pub fn state(&self) -> VideoState {
        let flags = self.shared.flags.load(Ordering::SeqCst);
        // SAFETY: plain getters on the live engine.
        unsafe {
            let duration = self.engine.GetDuration();
            VideoState {
                loaded: flags & LOADED != 0,
                ended: self.engine.IsEnded().as_bool(),
                failed: (flags & FAILED != 0).then(|| {
                    (
                        self.shared.error.load(Ordering::SeqCst),
                        self.shared.hresult.load(Ordering::SeqCst) as i32,
                    )
                }),
                paused: self.engine.IsPaused().as_bool(),
                position: finite(self.engine.GetCurrentTime()),
                duration: finite(duration),
            }
        }
    }

    /// Keeps the engine's pictures flowing, and when `take`, copies the newest one into the
    /// frame's texture; true then (the fence is to be drawn again). In frame-server mode the engine
    /// waits for its pictures to be asked for: a video whose pictures nobody asks for runs past its
    /// end, so this is called while it plays even when nothing is drawn. The texture is made once
    /// the size of the picture is known, and again when that size changes.
    pub fn update(&mut self, take: bool) -> Result<bool> {
        if self.shared.flags.load(Ordering::SeqCst) & LOADED == 0 {
            return Ok(false);
        }
        // SAFETY: plain calls on the live engine; the out values are locals.
        unsafe {
            // A file without a picture (sound only): nothing to take, and no size to ask for.
            if !self.engine.HasVideo().as_bool() {
                return Ok(false);
            }
            // S_OK: a new picture; S_FALSE: the one the texture holds is still the latest. The
            // generated wrapper folds both into success, so the vtable is called directly.
            let mut pts = 0i64;
            let hr = (Interface::vtable(&self.engine).OnVideoStreamTick)(
                Interface::as_raw(&self.engine),
                &mut pts,
            );
            hr.ok()?;
            let new = hr.0 == 0;
            if !take {
                self.missed |= new;
                return Ok(false);
            }
            if !new && !self.missed {
                return Ok(false);
            }
            let (mut w, mut h) = (0u32, 0u32);
            self.engine
                .GetNativeVideoSize(Some(&mut w), Some(&mut h))
                .ok()?;
            if w == 0 || h == 0 {
                return Ok(false);
            }
            let (w, h) = fit(w, h);
            if self
                .frame
                .as_ref()
                .is_none_or(|f| (f.width, f.height) != (w, h))
            {
                self.frame = Some(VideoFrame::new(&self.device, w, h)?);
            }
            let frame = self.frame.as_ref().expect("made just above");
            let to = b::RECT {
                left: 0,
                top: 0,
                right: w as i32,
                bottom: h as i32,
            };
            let black = b::MFARGB {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbAlpha: 255,
            };
            self.engine
                .TransferVideoFrame(&frame.texture, None, &to, Some(&black))
                .ok()?;
            frame.filled.set(true);
            self.missed = false;
            Ok(true)
        }
    }

    /// The latest picture, once the engine has given one.
    pub fn frame(&self) -> Option<&VideoFrame> {
        self.frame.as_ref().filter(|f| f.filled.get())
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        // SAFETY: the engine is live; Shutdown releases the decoder, the sound and the device.
        unsafe {
            let _ = self.engine.Shutdown();
        }
    }
}

/// `value` unless it is NaN or infinite (the engine's answer for an unknown length).
fn finite(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// The size the picture is kept at: the video's own, held within [`MAX_WIDTH`] x [`MAX_HEIGHT`]
/// in its ratio, and even (video formats like it so).
fn fit(width: u32, height: u32) -> (u32, u32) {
    let scale = (MAX_WIDTH as f64 / width as f64)
        .min(MAX_HEIGHT as f64 / height as f64)
        .min(1.0);
    let even = |v: f64| ((v * scale).round() as u32).max(2) & !1;
    (even(width as f64), even(height as f64))
}

/// The picture of the video: a texture of the render stack's device that the engine draws into
/// and Direct2D reads.
pub struct VideoFrame {
    texture: b::ID3D11Texture2D,
    pub width: u32,
    pub height: u32,
    /// The Direct2D view of the texture, made at the first draw (it needs a device context).
    bitmap: RefCell<Option<b::ID2D1Bitmap1>>,
    /// The engine has drawn into the texture at least once.
    filled: Cell<bool>,
}

impl VideoFrame {
    fn new(device: &b::ID3D11Device, width: u32, height: u32) -> Result<Self> {
        let desc = b::D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: b::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: b::D3D11_USAGE_DEFAULT,
            // The engine renders into it; Direct2D samples it.
            BindFlags: (b::D3D11_BIND_RENDER_TARGET | b::D3D11_BIND_SHADER_RESOURCE) as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut texture: Option<b::ID3D11Texture2D> = None;
        // SAFETY: `desc` and `texture` are locals of the stated types.
        unsafe {
            device
                .CreateTexture2D(&desc, None, Some(&mut texture))
                .ok()?
        };
        let texture = texture.ok_or_else(|| Error::from(E_FAIL))?;
        Ok(Self {
            texture,
            width,
            height,
            bitmap: RefCell::new(None),
            filled: Cell::new(false),
        })
    }

    /// Draws the picture into `dest` (`[left, top, right, bottom]`, in the units of `target`,
    /// a Direct2D device context of the device the player was made with) inside a
    /// `BeginDraw` bracket.
    pub fn draw(&self, target: &impl Interface, dest: [f32; 4]) -> Result<()> {
        let mut slot = self.bitmap.borrow_mut();
        if slot.is_none() {
            let context: b::ID2D1DeviceContext = target.cast()?;
            let surface: b::IDXGISurface = self.texture.cast()?;
            let properties = b::D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: b::D2D1_PIXEL_FORMAT {
                    format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
                    // The picture has no transparency whatever its alpha byte holds.
                    alphaMode: b::D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                bitmapOptions: b::D2D1_BITMAP_OPTIONS_NONE,
                colorContext: std::mem::ManuallyDrop::new(None),
            };
            tracing::info!(
                width = self.width,
                height = self.height,
                "video picture drawn for the first time"
            );
            // SAFETY: the surface belongs to the device the context draws with.
            *slot =
                Some(unsafe { context.CreateBitmapFromDxgiSurface(&surface, Some(&properties))? });
        }
        let bitmap = slot.as_ref().expect("made just above");
        let target: b::ID2D1RenderTarget = target.cast()?;
        let to = b::D2D_RECT_F {
            left: dest[0],
            top: dest[1],
            right: dest[2],
            bottom: dest[3],
        };
        // SAFETY: inside the caller's BeginDraw / EndDraw; the bitmap lives through the call.
        unsafe {
            target.DrawBitmap(
                bitmap,
                Some(&to),
                1.0,
                b::D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picture_sizes() {
        // A video that fits is kept as it is (even sizes), a bigger one comes down in its ratio.
        assert_eq!(fit(640, 360), (640, 360));
        assert_eq!(fit(3840, 2160), (1920, 1080));
        assert_eq!(fit(1920, 800), (1920, 800));
        assert_eq!(fit(1080, 1920), (608, 1080));
        assert_eq!(fit(1, 1), (2, 2));
    }

    #[test]
    fn unknown_lengths_are_zero() {
        assert_eq!(finite(f64::NAN), 0.0);
        assert_eq!(finite(f64::INFINITY), 0.0);
        assert_eq!(finite(12.5), 12.5);
    }
}
