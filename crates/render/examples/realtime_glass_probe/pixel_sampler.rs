//! Diagnostic only: bounded GPU -> CPU samples. This code is not part of PecoFence.

use super::sample_bindings as b;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use windows_core::{Error, HRESULT, IUnknown, Interface, Result};

const TILE: u32 = 16;
const ATLAS: u32 = TILE * 3;
const DXGI_ERROR_WAS_STILL_DRAWING: HRESULT = HRESULT(0x887A000Au32 as i32);

struct Unmap {
    context: b::ID3D11DeviceContext,
    texture: b::ID3D11Texture2D,
}

impl Drop for Unmap {
    fn drop(&mut self) {
        unsafe { self.context.Unmap(&self.texture, 0) };
    }
}

/// Two tiny staging textures let a sample finish before its next one-second read. Map is
/// nonblocking: a busy GPU skips a diagnostic sample instead of stalling the UI thread.
pub struct PixelSampler {
    context: b::ID3D11DeviceContext,
    slots: [b::ID3D11Texture2D; 2],
    pending: [bool; 2],
    write_slot: usize,
    next_at: Instant,
    hashes: HashSet<u64>,
    previous_hash: Option<u64>,
    samples: u64,
    non_black_samples: u64,
    changed_samples: u64,
    brightness_sum: f64,
    skipped_busy: u64,
}

impl PixelSampler {
    pub fn new(device: &IUnknown) -> Result<Self> {
        let device: b::ID3D11Device = device.cast()?;
        let context = unsafe { device.GetImmediateContext()? };
        let desc = b::D3D11_TEXTURE2D_DESC {
            Width: ATLAS,
            Height: ATLAS,
            MipLevels: 1,
            ArraySize: 1,
            Format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: b::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: b::D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: b::D3D11_CPU_ACCESS_READ as u32,
            MiscFlags: 0,
        };
        let make = || -> Result<b::ID3D11Texture2D> {
            let mut texture = None;
            unsafe {
                device
                    .CreateTexture2D(&desc, None, Some(&mut texture))
                    .ok()?
            };
            texture.ok_or_else(|| Error::from_hresult(HRESULT(0x80004003u32 as i32)))
        };
        Ok(Self {
            context,
            slots: [make()?, make()?],
            pending: [false; 2],
            write_slot: 0,
            next_at: Instant::now(),
            hashes: HashSet::new(),
            previous_hash: None,
            samples: 0,
            non_black_samples: 0,
            changed_samples: 0,
            brightness_sum: 0.0,
            skipped_busy: 0,
        })
    }

    pub fn sample(&mut self, surface: &IUnknown, size: (u32, u32)) -> Result<()> {
        let now = Instant::now();
        if now < self.next_at {
            return Ok(());
        }
        self.next_at = now + Duration::from_secs(1);
        self.read_slot(1 - self.write_slot)?;
        if self.pending[self.write_slot] {
            self.skipped_busy += 1;
            return Ok(());
        }
        if size.0 < TILE || size.1 < TILE {
            return Err(Error::new(
                HRESULT(0x80070057u32 as i32),
                "pixel sample source is too small",
            ));
        }
        let source: b::ID3D11Texture2D = surface.cast()?;
        for y in 0..3 {
            for x in 0..3 {
                let left = (size.0 * (x + 1) / 4)
                    .saturating_sub(TILE / 2)
                    .min(size.0 - TILE);
                let top = (size.1 * (y + 1) / 4)
                    .saturating_sub(TILE / 2)
                    .min(size.1 - TILE);
                let region = b::D3D11_BOX {
                    left,
                    top,
                    front: 0,
                    right: left + TILE,
                    bottom: top + TILE,
                    back: 1,
                };
                unsafe {
                    self.context.CopySubresourceRegion(
                        &self.slots[self.write_slot],
                        0,
                        x * TILE,
                        y * TILE,
                        0,
                        &source,
                        0,
                        Some(&region),
                    );
                }
            }
        }
        self.pending[self.write_slot] = true;
        self.write_slot = 1 - self.write_slot;
        Ok(())
    }

    fn read_slot(&mut self, slot: usize) -> Result<()> {
        if !self.pending[slot] {
            return Ok(());
        }
        let texture = &self.slots[slot];
        let mut mapped = b::D3D11_MAPPED_SUBRESOURCE::default();
        let result = unsafe {
            self.context.Map(
                texture,
                0,
                b::D3D11_MAP_READ,
                b::D3D11_MAP_FLAG_DO_NOT_WAIT as u32,
                Some(&mut mapped),
            )
        };
        if result == DXGI_ERROR_WAS_STILL_DRAWING {
            return Ok(());
        }
        result.ok()?;
        let _unmap = Unmap {
            context: self.context.clone(),
            texture: texture.clone(),
        };
        if mapped.pData.is_null() || mapped.RowPitch < ATLAS * 4 {
            return Err(Error::new(
                HRESULT(0x80004005u32 as i32),
                "invalid diagnostic readback mapping",
            ));
        }
        let mut hash = 0xcbf29ce484222325u64;
        let mut brightness = 0.0;
        for y in 0..ATLAS as usize {
            // Only valid row bytes enter the hash, never GPU padding. Ignore alpha:
            // composition capture alpha is not used by the production background material.
            let row = unsafe {
                std::slice::from_raw_parts(
                    mapped.pData.cast::<u8>().add(y * mapped.RowPitch as usize),
                    (ATLAS * 4) as usize,
                )
            };
            for bgra in row.chunks_exact(4) {
                for value in &bgra[..3] {
                    hash = (hash ^ u64::from(*value)).wrapping_mul(0x100000001b3);
                }
                brightness += 0.0722 * f64::from(bgra[0])
                    + 0.7152 * f64::from(bgra[1])
                    + 0.2126 * f64::from(bgra[2]);
            }
        }
        brightness /= f64::from(ATLAS * ATLAS);
        self.samples += 1;
        self.non_black_samples += u64::from(brightness > 1.0);
        self.changed_samples += u64::from(self.previous_hash.is_some_and(|old| old != hash));
        self.hashes.insert(hash);
        self.previous_hash = Some(hash);
        self.brightness_sum += brightness;
        self.pending[slot] = false;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        self.read_slot(0)?;
        self.read_slot(1)
    }

    pub fn summary(&self) -> Value {
        json!({
            "pixelSamples": self.samples,
            "uniqueSampleHashes": self.hashes.len(),
            "nonBlackSamples": self.non_black_samples,
            "meanBrightness": if self.samples > 0 { self.brightness_sum / self.samples as f64 } else { 0.0 },
            "changedSamples": self.changed_samples,
            "pixelSamplesSkippedBusy": self.skipped_busy,
            "pixelSampleLayout": "nine 16x16 tiles at the source quarter/half/three-quarter coordinates",
            "pixelSampleIntervalMs": 1000,
            "pixelSampleBytes": ATLAS * ATLAS * 4,
            "brightnessScale": "0..255",
            "lastSampleHash": self.previous_hash.map(|hash| format!("{hash:016x}")),
            "unchangedPixelsAreNotAnError": true,
        })
    }
}
