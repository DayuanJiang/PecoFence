//! Rendering layer: Windows.UI.Composition visual trees with Direct2D-drawn surfaces.
//!
//! The always-resident fence windows are composed by DWM from GPU surfaces; this crate owns
//! the compositor, the shared GPU device and the drawing helpers. Nothing here knows about
//! fences' data model — it draws what the app tells it to.

#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute,
    clippy::too_many_arguments,
    dead_code,
    unused_imports
)]
pub mod comp;

pub mod backdrop;
pub mod bitmaps;
pub mod custom_theme;
pub mod fence_chrome;
#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute,
    clippy::too_many_arguments,
    clippy::useless_conversion,
    dead_code,
    unused_imports
)]
mod gpu_bindings;
pub mod gpu_glass;
pub mod liquid_glass;
pub mod motion;
pub mod notice;
pub mod panel;
pub mod stack;
pub mod text;
pub mod theme;

pub use backdrop::{GpuPixels, Image, MicaTint, MonitorBackdrop, WallpaperPosition};
pub use bitmaps::BitmapCache;
pub use custom_theme::CustomTheme;
pub use panel::{Clip, Panel};
pub use stack::RenderStack;
pub use theme::{Theme, ThemeMode};
pub use windows_canvas::{
    Bitmap, ColorF, DrawingSession, FontWeight, Matrix3x2, ParagraphAlignment, Rect, RenderTarget,
    RoundedRect, TextAlignment, TextFormat, TextLayout, Vector2, WordWrapping,
};
pub use windows_composition::{ContainerVisual, DesktopWindowTarget, SpriteVisual, Visual};
pub use windows_core::Result;
