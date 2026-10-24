//! The system colour dialog (`ChooseColorW`) behind the settings page's 「自定义颜色」 swatch.

use crate::bindings::*;
use std::sync::Mutex;

/// The dialog's 16 "custom colours" slots, kept for the session like Paint does. `None` until
/// the first open, which starts them white with the Windows accent colour in the first slot.
static CUSTOM_COLORS: Mutex<Option<[COLORREF; 16]>> = Mutex::new(None);

fn colorref([r, g, b]: [u8; 3]) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

/// Opens the full dialog on `initial` (black when `None`). Returns the chosen colour, or
/// `None` when the user cancelled.
pub fn pick(owner: Option<HWND>, initial: Option<[u8; 3]>, accent: [u8; 3]) -> Option<[u8; 3]> {
    // A copy, so the lock is not held through the dialog's modal loop.
    let mut custom = *CUSTOM_COLORS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(|| {
            let mut s = [COLORREF(0xFF_FFFF); 16];
            s[0] = colorref(accent);
            s
        });
    let mut cc = CHOOSECOLORW {
        lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: owner.unwrap_or_default(),
        rgbResult: colorref(initial.unwrap_or_default()),
        lpCustColors: custom.as_mut_ptr(),
        Flags: (CC_RGBINIT | CC_FULLOPEN | CC_ANYCOLOR) as u32,
        ..Default::default()
    };
    // SAFETY: `cc` and the 16 slots it points at outlive the modal call.
    let chosen = unsafe { ChooseColorW(&mut cc) }.as_bool();
    // Slots edited before a cancel are kept too, as in Paint.
    *CUSTOM_COLORS.lock().unwrap_or_else(|e| e.into_inner()) = Some(custom);
    if !chosen {
        return None;
    }
    let c = cc.rgbResult.0;
    Some([c as u8, (c >> 8) as u8, (c >> 16) as u8])
}
