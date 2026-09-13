//! GPU bitmap cache: CPU `Image`s uploaded once per D2D device and reused across frames.

use crate::backdrop::Image;
use std::collections::HashMap;
use windows_canvas::{Bitmap, DrawingSession};
use windows_core::Result;

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
