//! One bounded, replaceable startup backdrop set. All callers run off the UI thread.
//! Exact recipe keys include file stamps, monitor geometry, DPI-scaled tint and material.

use pecofence_render::Image;
use std::io::{self, Read, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"PFWALL01";
const MAX_BYTES: u64 = 128 * 1024 * 1024;
const MAX_KEY: usize = 64 * 1024;
const MAX_IMAGES: usize = 32;

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid startup wallpaper cache",
    )
}

fn pixel_bytes(width: u32, height: u32) -> io::Result<usize> {
    let bytes = (u64::from(width) * u64::from(height))
        .checked_mul(4)
        .ok_or_else(invalid)?;
    if width == 0 || height == 0 || bytes > MAX_BYTES {
        return Err(invalid());
    }
    Ok(bytes as usize)
}

// A corruption check, not a security boundary. The recipe key is compared byte-for-byte.
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn decode(reader: &mut impl Read, key: &str, sizes: &[(u32, u32)]) -> io::Result<Vec<Image>> {
    if key.len() > MAX_KEY || sizes.is_empty() || sizes.len() > MAX_IMAGES {
        return Err(invalid());
    }
    let mut magic = [0; 8];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC || read_u32(reader)? as usize != key.len() {
        return Err(invalid());
    }
    let mut saved_key = vec![0; key.len()];
    reader.read_exact(&mut saved_key)?;
    if saved_key != key.as_bytes() || read_u32(reader)? as usize != sizes.len() {
        return Err(invalid());
    }
    let mut images = Vec::with_capacity(sizes.len());
    let mut total = 0;
    for &(expected_w, expected_h) in sizes {
        let (width, height) = (read_u32(reader)?, read_u32(reader)?);
        if (width, height) != (expected_w, expected_h) {
            return Err(invalid());
        }
        let len = pixel_bytes(width, height)?;
        total += len as u64;
        if total > MAX_BYTES {
            return Err(invalid());
        }
        let mut saved_checksum = [0; 8];
        reader.read_exact(&mut saved_checksum)?;
        let mut bgra = vec![0; len];
        reader.read_exact(&mut bgra)?;
        if checksum(&bgra) != u64::from_le_bytes(saved_checksum) {
            return Err(invalid());
        }
        images.push(Image {
            width,
            height,
            bgra,
        });
    }
    if reader.read(&mut [0])? != 0 {
        return Err(invalid());
    }
    Ok(images)
}

fn encode(writer: &mut impl Write, key: &str, images: &[Image]) -> io::Result<()> {
    if key.len() > MAX_KEY || images.is_empty() || images.len() > MAX_IMAGES {
        return Err(invalid());
    }
    let mut total = 0;
    for image in images {
        let bytes = pixel_bytes(image.width, image.height)?;
        total += bytes as u64;
        if bytes != image.bgra.len() || total > MAX_BYTES {
            return Err(invalid());
        }
    }
    writer.write_all(MAGIC)?;
    writer.write_all(&(key.len() as u32).to_le_bytes())?;
    writer.write_all(key.as_bytes())?;
    writer.write_all(&(images.len() as u32).to_le_bytes())?;
    for image in images {
        writer.write_all(&image.width.to_le_bytes())?;
        writer.write_all(&image.height.to_le_bytes())?;
        writer.write_all(&checksum(&image.bgra).to_le_bytes())?;
        writer.write_all(&image.bgra)?;
    }
    Ok(())
}

pub(super) fn read(path: &Path, key: &str, sizes: &[(u32, u32)]) -> Option<Vec<Image>> {
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_BYTES + MAX_KEY as u64 + 1024 {
        return None;
    }
    decode(&mut io::BufReader::new(file), key, sizes).ok()
}

pub(super) fn write(path: &Path, key: &str, images: &[Image]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(invalid)?;
    std::fs::create_dir_all(parent)?;
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        let mut writer = io::BufWriter::new(file);
        encode(&mut writer, key, images)?;
        writer.flush()?;
        drop(writer);
        // Same-directory replacement: a crash never leaves a partial published cache.
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_key_and_dimensions_are_required() {
        let images = [Image::solid(3, 2, [10, 20, 30])];
        let mut bytes = Vec::new();
        encode(&mut bytes, "file#stamp;dpi=144;glass", &images).unwrap();
        let read = decode(&mut bytes.as_slice(), "file#stamp;dpi=144;glass", &[(3, 2)]).unwrap();
        assert_eq!(read[0].bgra, images[0].bgra);
        for key in [
            "changed-file",
            "file#stamp;dpi=96;glass",
            "file#stamp;dpi=144;acrylic",
        ] {
            assert!(decode(&mut bytes.as_slice(), key, &[(3, 2)]).is_err());
        }
        assert!(decode(&mut bytes.as_slice(), "file#stamp;dpi=144;glass", &[(2, 3)]).is_err());
    }

    #[test]
    fn truncated_corrupt_and_unbounded_entries_are_rejected() {
        let mut bytes = Vec::new();
        encode(&mut bytes, "key", &[Image::solid(2, 2, [2, 3, 4])]).unwrap();
        for end in 0..bytes.len() {
            assert!(decode(&mut &bytes[..end], "key", &[(2, 2)]).is_err());
        }
        *bytes.last_mut().unwrap() ^= 1;
        assert!(decode(&mut bytes.as_slice(), "key", &[(2, 2)]).is_err());
        assert!(pixel_bytes(u32::MAX, u32::MAX).is_err());
        assert!(pixel_bytes(0, 2).is_err());
    }
}
