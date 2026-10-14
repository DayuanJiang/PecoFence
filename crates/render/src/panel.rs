//! A sprite visual backed by a Direct2D drawing surface, sized in physical pixels and drawn
//! in device-independent pixels.

use crate::comp;
use crate::stack::RenderStack;
use std::cell::RefCell;
use windows_canvas::{
    DrawingSession, ID2D1DeviceContext, Matrix3x2, Rect, check_device_lost, is_device_lost,
};
use windows_composition::{
    CompositionDrawingSurface, CompositionGraphicsDevice, Compositor, SpriteVisual,
};
use windows_core::{Interface, Result};

/// Clip control for the surface being drawn (see [`Panel::draw_ex`]). windows-canvas has no
/// clip API, so this reaches the device context behind the session.
pub struct Clip<'a> {
    context: &'a ID2D1DeviceContext,
}

impl Clip<'_> {
    /// Runs `f` with drawing clipped to `rect` (DIPs, the same space as the session's drawing
    /// calls); the clip is popped afterwards even when `f` fails.
    pub fn scoped(&self, rect: &Rect, f: impl FnOnce() -> Result<()>) -> Result<()> {
        let _clip = pecofence_platform::d2d::push_axis_aligned_clip(
            self.context,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
        )?;
        f()
    }
}

/// Ends the surface draw only on the panic path (mirrors the windows-canvas composition bridge).
struct EndDrawGuard<'a>(&'a CompositionDrawingSurface);

impl Drop for EndDrawGuard<'_> {
    fn drop(&mut self) {
        let _ = self.0.end_draw();
    }
}

thread_local! {
    /// Surfaces that `Panel::resize` took off their visual while their last update was still
    /// waiting for the composition commit, each with the commit batch of that update.
    static RETIRED: RefCell<Vec<(CompositionDrawingSurface, comp::CompositionCommitBatch)>> =
        const { RefCell::new(Vec::new()) };
}

/// Releases the retired surfaces whose last update has been committed.
pub fn release_retired_surfaces() {
    RETIRED.with(|r| {
        if let Ok(mut r) = r.try_borrow_mut() {
            r.retain(|(_, batch)| !batch.IsEnded().unwrap_or(true));
        }
    });
}

pub struct Panel {
    pub visual: SpriteVisual,
    surface: CompositionDrawingSurface,
    width_px: i32,
    height_px: i32,
    /// Make the replacement surface and its brush in `resize`.
    graphics: CompositionGraphicsDevice,
    compositor: Compositor,
    raw_compositor: comp::Compositor,
    /// The commit batch of the last draw; until it has ended, that update is still pending.
    drawn_in: RefCell<Option<comp::CompositionCommitBatch>>,
}

impl Panel {
    pub fn new(stack: &RenderStack) -> Result<Self> {
        let surface = stack.graphics().create_drawing_surface(1.0, 1.0)?;
        let visual = stack.compositor.create_sprite_visual();
        visual.set_brush(&stack.compositor.create_surface_brush(&surface));
        visual.set_size(1.0, 1.0);
        Ok(Self {
            visual,
            surface,
            width_px: 1,
            height_px: 1,
            graphics: stack.graphics(),
            compositor: stack.compositor.clone(),
            raw_compositor: stack.compositor.as_raw().cast()?,
            drawn_in: RefCell::new(None),
        })
    }

    pub fn size_px(&self) -> (i32, i32) {
        (self.width_px, self.height_px)
    }

    /// Replaces the backing surface with one from the (rebuilt) graphics device, keeping the
    /// visual in place. Contents must be redrawn afterwards.
    pub fn recreate(&mut self, stack: &RenderStack) -> Result<()> {
        let surface = stack
            .graphics()
            .create_drawing_surface(self.width_px as f32, self.height_px as f32)?;
        self.visual
            .set_brush(&stack.compositor.create_surface_brush(&surface));
        self.surface = surface;
        self.graphics = stack.graphics();
        self.drawn_in.get_mut().take();
        Ok(())
    }

    /// Resizes the backing surface (physical pixels). Contents must be redrawn afterwards.
    ///
    /// A surface drawn earlier in the same composition batch is not resized. When such a
    /// pending update is thrown away (by `Resize`, or by dropping the surface) and the panel is
    /// drawn again before the commit, DirectComposition keeps the update's GPU memory for as
    /// long as the graphics device lives: one surface's worth per relayout, where a fence
    /// redraws for its new icon size and then snaps to whole cells. The panel gets a new
    /// surface instead, and the old one is released once its update has been committed.
    pub fn resize(&mut self, width_px: i32, height_px: i32) -> Result<bool> {
        let width_px = width_px.max(1);
        let height_px = height_px.max(1);
        if (width_px, height_px) == (self.width_px, self.height_px) {
            return Ok(false);
        }
        release_retired_surfaces();
        let pending = self
            .drawn_in
            .get_mut()
            .take()
            .filter(|batch| !batch.IsEnded().unwrap_or(true));
        if let Some(batch) = pending {
            let surface = self
                .graphics
                .create_drawing_surface(width_px as f32, height_px as f32)?;
            self.visual
                .set_brush(&self.compositor.create_surface_brush(&surface));
            let old = std::mem::replace(&mut self.surface, surface);
            RETIRED.with(|r| r.borrow_mut().push((old, batch)));
            tracing::debug!(
                width_px,
                height_px,
                "surface replaced: its update is not committed yet"
            );
        } else {
            self.surface.resize(width_px, height_px)?;
        }
        self.visual.set_size(width_px as f32, height_px as f32);
        self.width_px = width_px;
        self.height_px = height_px;
        Ok(true)
    }

    /// Redraws the whole surface. `draw` receives the session with a DIP transform applied
    /// for `dpi`, plus the surface size in DIPs. Returns `Ok(false)` on device loss.
    pub fn draw(
        &self,
        dpi: u32,
        draw: impl FnOnce(&DrawingSession<'_>, f32, f32) -> Result<()>,
    ) -> Result<bool> {
        self.draw_ex(dpi, |session, _clip, w, h| draw(session, w, h))
    }

    /// [`Panel::draw`] plus a [`Clip`] handle for the callback. Same BeginDraw/EndDraw bracket
    /// and device-loss classification as `CanvasCompositionExt::draw`, done here because that
    /// bridge keeps the device context private.
    pub fn draw_ex(
        &self,
        dpi: u32,
        draw: impl FnOnce(&DrawingSession<'_>, &Clip<'_>, f32, f32) -> Result<()>,
    ) -> Result<bool> {
        let scale = dpi as f32 / 96.0;
        let width_dip = self.width_px as f32 / scale;
        let height_dip = self.height_px as f32 / scale;
        let (context, (offset_x, offset_y)) = match self.surface.begin_draw::<ID2D1DeviceContext>()
        {
            Ok(v) => v,
            Err(e) if is_device_lost(e.code()) => return Ok(false),
            Err(e) => return Err(e),
        };
        let guard = EndDrawGuard(&self.surface);
        release_retired_surfaces();
        *self.drawn_in.borrow_mut() = self
            .raw_compositor
            .GetCommitBatch(comp::CompositionBatchTypes::Animation)
            .ok();
        let draw_result = {
            let session = DrawingSession::from_borrowed_context_with_dpi(
                &context,
                Matrix3x2::translation(offset_x as f32, offset_y as f32),
                96.0,
            );
            session.set_transform(&Matrix3x2 {
                m11: scale,
                m12: 0.0,
                m21: 0.0,
                m22: scale,
                m31: 0.0,
                m32: 0.0,
            });
            let clip = Clip { context: &context };
            draw(&session, &clip, width_dip, height_dip)
        };
        std::mem::forget(guard);
        let end_result = self.surface.end_draw();
        crate::stack::note_draw();
        if check_device_lost(&draw_result) || check_device_lost(&end_result) {
            return Ok(false);
        }
        end_result?;
        draw_result.map(|()| true)
    }
}
