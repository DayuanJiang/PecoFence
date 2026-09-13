//! A sprite visual backed by a Direct2D drawing surface, sized in physical pixels and drawn
//! in device-independent pixels.

use crate::stack::RenderStack;
use windows_canvas::{
    DrawingSession, ID2D1DeviceContext, Matrix3x2, Rect, check_device_lost, is_device_lost,
};
use windows_composition::{CompositionDrawingSurface, SpriteVisual};
use windows_core::Result;

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

pub struct Panel {
    pub visual: SpriteVisual,
    surface: CompositionDrawingSurface,
    width_px: i32,
    height_px: i32,
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
        Ok(())
    }

    /// Resizes the backing surface (physical pixels). Contents must be redrawn afterwards.
    pub fn resize(&mut self, width_px: i32, height_px: i32) -> Result<bool> {
        let width_px = width_px.max(1);
        let height_px = height_px.max(1);
        if (width_px, height_px) == (self.width_px, self.height_px) {
            return Ok(false);
        }
        self.surface.resize(width_px, height_px)?;
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
        if check_device_lost(&draw_result) || check_device_lost(&end_result) {
            return Ok(false);
        }
        end_result?;
        draw_result.map(|()| true)
    }
}
