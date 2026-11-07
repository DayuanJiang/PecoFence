//! The filter box of a folder fence (筛选): a small activatable popup hosting an EDIT control in
//! the fence's title row, as the inline rename does. Every change filters the fence at once;
//! Enter or a click elsewhere closes the box and keeps the filter, Esc clears it.

use crate::commands::{Command, CommandQueue};
use pecofence_core::FenceId;
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, edit, monitors, msg};
use pecofence_render::ThemeMode;
use std::cell::RefCell;

const FILTER_CLASS: &str = "PecoFence.Filter";

thread_local! {
    static ACTIVE: RefCell<Option<FilterSession>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

struct FilterSession {
    window: Window,
    edit: edit::EditControl,
    fence: FenceId,
    queue: CommandQueue,
}

/// Where the box sits in the title row (the sample's switch, see `App::filter_ui`): over the
/// whole row in place of the title, or in its right half beside the title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    WholeRow,
    RightHalf,
}

/// Opens (or moves) the filter box over `fence_hwnd`'s title row with `current` in it.
pub fn open(
    fence_hwnd: HWND,
    fence: FenceId,
    current: &str,
    cue: &str,
    placement: Placement,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    close();
    let r = window::window_rect(fence_hwnd);
    let dpi = monitors::dpi_for_window(fence_hwnd).max(96);
    let scale = dpi as f32 / 96.0;
    let px = |dip: f32| (dip * scale) as i32;
    let h = px(24.0);
    let right = r.right - px(44.0);
    let left = match placement {
        Placement::WholeRow => r.left + px(8.0),
        Placement::RightHalf => (r.left + (r.right - r.left) * 9 / 20).min(right - px(80.0)),
    };
    let rect = RECT {
        left,
        top: r.top + px(6.0),
        right: right.max(left + px(80.0)),
        bottom: r.top + px(6.0) + h,
    };
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(FILTER_CLASS, ClassOptions::default()).ok();
        }
    });
    let handler: MessageHandler = Box::new(
        move |_hwnd: HWND, message: u32, wparam: usize, _lparam: isize| -> Option<isize> {
            match message {
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_CHANGE => {
                    ACTIVE.with(|a| {
                        if let Some(s) = a.try_borrow().ok().as_ref().and_then(|a| a.as_ref()) {
                            s.queue.push(Command::FenceFilterText {
                                fence: s.fence,
                                text: s.edit.text(),
                            });
                        }
                    });
                    Some(0)
                }
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_KILLFOCUS => {
                    close();
                    Some(0)
                }
                edit::WM_CTLCOLOREDIT_MSG => {
                    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.edit.ctl_color(wparam)))
                }
                msg::WM_DESTROY => Some(0),
                _ => None,
            }
        },
    );
    let (x, y, w, hh) = (
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
    );
    let created = CLASS.with(|c| {
        let c = c.borrow();
        let class = c.as_ref()?;
        // Topmost for the same reason as the rename popup (#99): during Peek and Win+D the
        // fences float in the topmost band.
        WindowBuilder::new(class)
            .title("filter")
            .style(style::POPUP)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(x, y, w, hh)
            .create(handler)
            .ok()
    });
    let Some(window) = created else { return };
    let Ok(ctl) = edit::EditControl::create(
        window.hwnd(),
        w,
        hh,
        dpi,
        14.0,
        current,
        mode == ThemeMode::Dark,
        on_key,
    ) else {
        return;
    };
    ctl.set_cue(cue);
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    ctl.focus_at_end();
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(FilterSession {
            window,
            edit: ctl,
            fence,
            queue,
        })
    });
}

fn on_key(vk: u32) -> bool {
    match vk {
        k if k == msg::VK_RETURN => {
            close();
            true
        }
        k if k == msg::VK_ESCAPE => {
            let session = ACTIVE.with(|a| a.borrow_mut().take());
            if let Some(s) = session {
                s.queue.push(Command::FenceFilterText {
                    fence: s.fence,
                    text: String::new(),
                });
                s.queue.push(Command::FenceFilterClosed(s.fence));
            }
            true
        }
        _ => false,
    }
}

/// The fence whose filter box is open, if one is.
pub fn active_fence() -> Option<FenceId> {
    ACTIVE.with(|a| a.try_borrow().ok()?.as_ref().map(|s| s.fence))
}

/// Closes the box; the filter stays as typed.
pub fn close() {
    let session = ACTIVE.with(|a| a.try_borrow_mut().ok()?.take());
    if let Some(s) = session {
        s.queue.push(Command::FenceFilterClosed(s.fence));
        drop(s.edit);
        drop(s.window);
    }
}
