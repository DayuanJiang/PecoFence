//! The floating-point control register of the calling thread (MXCSR on x64).
//!
//! Windows' x64 calling convention says a called function leaves the control bits of MXCSR as
//! it found them, but some shell extensions that Windows loads into our process for a context
//! menu unmask floating-point exceptions and return without masking them again. Our own code then ends the process at the next invalid operation it
//! does on purpose: the motion code's packed division computes 0/0 in an unused lane every
//! frame, harmless while the exception is masked (feedback #163: 0xc0000090,
//! STATUS_FLOAT_INVALID_OPERATION, while an item menu was open). The UI thread puts the default
//! back before it runs any code of ours for a window message, and after calls into a menu's
//! handlers.

use std::sync::atomic::{AtomicU32, Ordering};

/// The value Windows starts every thread with: all exceptions masked, round to nearest, no
/// flush to zero.
pub const DEFAULT_MXCSR: u32 = 0x1F80;
/// Everything but the six sticky status flags (bits 0 to 5), which our own 0/0 sets anyway.
const CONTROL_BITS: u32 = !0x3F;
/// How many changes are logged at info level; the rest go to debug (a broken extension can
/// change it on every menu message).
const LOGGED_CHANGES: u32 = 5;

static CHANGES: AtomicU32 = AtomicU32::new(0);

/// MXCSR of the calling thread.
#[cfg(target_arch = "x86_64")]
pub fn mxcsr() -> u32 {
    let mut value: u32 = 0;
    // SAFETY: stmxcsr writes 4 bytes to the local; no other state changes.
    unsafe {
        core::arch::asm!(
            "stmxcsr [{}]",
            in(reg) core::ptr::addr_of_mut!(value),
            options(nostack, preserves_flags)
        );
    }
    value
}

#[cfg(not(target_arch = "x86_64"))]
pub fn mxcsr() -> u32 {
    DEFAULT_MXCSR
}

#[cfg(target_arch = "x86_64")]
fn set_mxcsr(value: u32) {
    // SAFETY: ldmxcsr loads a valid control value (reserved bits clear) from the local.
    unsafe {
        core::arch::asm!(
            "ldmxcsr [{}]",
            in(reg) core::ptr::addr_of!(value),
            options(nostack, preserves_flags, readonly)
        );
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn set_mxcsr(_value: u32) {}

/// Puts the default control bits back when something changed them; `after` says what ran
/// before (for the log). Returns the value found when it was changed.
pub fn restore(after: &str) -> Option<u32> {
    let found = mxcsr();
    if found & CONTROL_BITS == DEFAULT_MXCSR & CONTROL_BITS {
        return None;
    }
    set_mxcsr(DEFAULT_MXCSR);
    let n = CHANGES.fetch_add(1, Ordering::Relaxed) + 1;
    if n <= LOGGED_CHANGES {
        tracing::info!(
            found = format_args!("{found:#06x}"),
            restored = format_args!("{DEFAULT_MXCSR:#06x}"),
            after,
            changes = n,
            "floating-point control word changed by other code; default restored"
        );
    } else {
        tracing::debug!(
            found = format_args!("{found:#06x}"),
            after,
            changes = n,
            "floating-point control word restored again"
        );
    }
    Some(found)
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    #[test]
    fn unmasked_exceptions_are_masked_again() {
        let before = mxcsr();
        // Unmask the invalid-operation exception (bit 7), as such an extension does.
        set_mxcsr(DEFAULT_MXCSR & !0x80);
        assert_eq!(restore("test"), Some(DEFAULT_MXCSR & !0x80));
        assert_eq!(mxcsr() & CONTROL_BITS, DEFAULT_MXCSR & CONTROL_BITS);
        // A 0/0 now only makes a NaN.
        let zero = std::hint::black_box(0.0f32);
        assert!((zero / zero).is_nan());
        set_mxcsr(before);
    }

    #[test]
    fn sticky_flags_alone_change_nothing() {
        let before = mxcsr();
        set_mxcsr(DEFAULT_MXCSR | 0x01);
        assert_eq!(restore("test"), None);
        set_mxcsr(before);
    }
}
