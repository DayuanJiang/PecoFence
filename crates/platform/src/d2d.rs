//! Direct2D bits that windows-canvas does not expose: an axis-aligned clip on the device
//! context behind a drawing session.

use crate::bindings::{D2D_RECT_F, D2D1_ANTIALIAS_MODE_ALIASED, ID2D1RenderTarget};
use windows_core::{Interface, Result};

/// A pushed axis-aligned clip; popped on drop, so an early `?` cannot leave it active.
pub struct AxisAlignedClip(ID2D1RenderTarget);

/// Clips subsequent drawing on `target` (any `ID2D1RenderTarget`-derived interface, e.g. the
/// `ID2D1DeviceContext` behind a windows-canvas session) to the rectangle, expressed in the
/// target's current coordinate space (its transform and DPI apply).
pub fn push_axis_aligned_clip(
    target: &impl Interface,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) -> Result<AxisAlignedClip> {
    let rt: ID2D1RenderTarget = target.cast()?;
    let rect = D2D_RECT_F {
        left,
        top,
        right,
        bottom,
    };
    // SAFETY: `rt` is a live render target inside a BeginDraw/EndDraw bracket owned by the
    // caller; the matching pop happens in `AxisAlignedClip::drop` on the same target.
    unsafe { rt.PushAxisAlignedClip(&rect, D2D1_ANTIALIAS_MODE_ALIASED) };
    Ok(AxisAlignedClip(rt))
}

impl Drop for AxisAlignedClip {
    fn drop(&mut self) {
        // SAFETY: pairs with the push in `push_axis_aligned_clip`.
        unsafe { self.0.PopAxisAlignedClip() };
    }
}
