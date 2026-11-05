//! 「双击 Ctrl 打开搜索」 (settings `peek.doubleCtrl`): two short taps of Ctrl alone open the
//! search box, as in Listary. Keyboard raw input is only listened to (`RIDEV_INPUTSINK`), so no
//! key is held back and typing elsewhere does not slow down.

use pecofence_platform::{HWND, rawinput};
use std::cell::RefCell;
use std::time::{Duration, Instant};

const VK_CONTROL: u16 = 0x11;
/// A tap: Ctrl down and up again within this time.
const TAP_MAX: Duration = Duration::from_millis(350);
/// The second tap must start within this time after the first ended.
const GAP_MAX: Duration = Duration::from_millis(400);

#[derive(Default)]
struct Tracker {
    on: bool,
    /// When Ctrl went down, while it is held.
    down_at: Option<Instant>,
    /// Another key was pressed while Ctrl was held (Ctrl+C and so on).
    combined: bool,
    /// When the last lone tap ended.
    last_tap: Option<Instant>,
    /// Watches the mouse while Ctrl is held (Ctrl+click, Ctrl+wheel are no taps).
    mouse: Option<rawinput::MouseWatch>,
}

thread_local! {
    static TRACKER: RefCell<Tracker> = RefCell::new(Tracker::default());
}

/// Starts or stops listening (raw keyboard input to `control`).
pub fn set_enabled(on: bool, control: HWND) {
    let was = TRACKER.with(|t| t.borrow().on);
    if on == was {
        return;
    }
    let result = if on {
        rawinput::register_keyboard_sink(control)
    } else {
        rawinput::unregister_keyboard_sink()
    };
    match result {
        Ok(()) => {
            TRACKER.with(|t| {
                *t.borrow_mut() = Tracker {
                    on,
                    ..Tracker::default()
                }
            });
            tracing::info!(on, "double Ctrl");
        }
        Err(e) => tracing::warn!(error = %e, on, "double Ctrl: raw keyboard registration failed"),
    }
}

/// A `WM_INPUT` on the control window: true when it completed a double tap.
pub fn on_input(lparam: isize) -> bool {
    let Some((vk, up)) = rawinput::key_from_wm_input(lparam) else {
        return false;
    };
    TRACKER.with(|t| {
        let mut t = t.borrow_mut();
        if !t.on {
            return false;
        }
        let now = Instant::now();
        if vk != VK_CONTROL {
            if !up {
                t.combined = true;
                t.last_tap = None;
            }
            return false;
        }
        if !up {
            // Auto-repeat while held keeps the first time.
            if t.down_at.is_none() {
                t.down_at = Some(now);
                t.combined = false;
                t.mouse = rawinput::MouseWatch::start();
            }
            return false;
        }
        let Some(down) = t.down_at.take() else {
            return false;
        };
        if t.mouse.take().is_some_and(|m| m.used()) {
            t.combined = true;
        }
        if t.combined || now - down > TAP_MAX {
            t.last_tap = None;
            return false;
        }
        match t.last_tap {
            Some(prev) if down - prev <= GAP_MAX => {
                t.last_tap = None;
                true
            }
            _ => {
                t.last_tap = Some(now);
                false
            }
        }
    })
}
