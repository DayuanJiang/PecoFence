//! GPU bitmap cache: CPU `Image`s uploaded once per D2D device and reused across frames.

use crate::backdrop::{Image, MonitorBackdrop};
use crate::stack::RenderStack;
use std::collections::HashMap;
use std::rc::Rc;
use windows_canvas::{Bitmap, DrawingSession};
use windows_core::{Interface, Result};

#[derive(Default)]
pub struct BitmapCache {
    map: HashMap<String, Bitmap>,
    pub(crate) glass_wallpaper: crate::gpu_glass::WallpaperCache,
}

impl BitmapCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<Bitmap> {
        self.map.get(key).cloned()
    }

    /// Returns the cached bitmap for `key`, uploading `image` on first use.
    pub fn get_or_upload(
        &mut self,
        session: &DrawingSession<'_>,
        key: &str,
        image: &Image,
    ) -> Result<Bitmap> {
        if let Some(b) = self.map.get(key) {
            return Ok(b.clone());
        }
        let bitmap = session.create_bitmap(&image.bgra, image.width, image.height)?;
        self.map.insert(key.to_string(), bitmap.clone());
        Ok(bitmap)
    }

    /// Drops every bitmap whose key starts with `prefix` (an icon key at any size / variant).
    pub fn remove_prefix(&mut self, prefix: &str) {
        self.map.retain(|k, _| !k.starts_with(prefix));
    }

    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    /// Keeps the keyed bitmaps `keep` accepts (wallpaper uploads are not keyed).
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.map.retain(|k, _| keep(k));
    }

    /// Drops the keyed bitmaps (icons, crops) but keeps wallpaper uploads: those are keyed
    /// by their backdrop set and cost a full decode to redo once their pixels are released.
    pub fn clear_keyed(&mut self) {
        self.map.clear();
    }

    /// Releases wallpaper textures whose backdrop set is gone (see `WallpaperCache::prune`).
    pub fn prune_wallpapers(&mut self) {
        self.glass_wallpaper.prune();
    }

    /// Uploads a Liquid Glass wallpaper set no fence draws yet (another virtual desktop's,
    /// prepared in advance), so its full-resolution pixels leave process memory now rather
    /// than at the first draw. The upload goes through a 1x1 surface of the stack's device.
    pub fn preload_wallpaper(
        &mut self,
        stack: &RenderStack,
        sources: &Rc<Vec<MonitorBackdrop>>,
    ) -> Result<()> {
        let surface = stack.graphics().create_drawing_surface(1.0, 1.0)?;
        let (context, _) = surface.begin_draw::<windows_canvas::ID2D1DeviceContext>()?;
        let result = context
            .cast()
            .and_then(|context| self.glass_wallpaper.preload(&context, sources));
        let ended = surface.end_draw();
        // The upload leaves driver staging memory behind, like a panel draw: schedule the trim.
        crate::stack::note_draw();
        result.and(ended)
    }

    /// Drops everything (e.g. after device loss).
    pub fn clear(&mut self) {
        self.map.clear();
        self.glass_wallpaper.clear();
    }

    pub fn glass_wallpaper_uploads(&self) -> u64 {
        self.glass_wallpaper.uploads
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}
