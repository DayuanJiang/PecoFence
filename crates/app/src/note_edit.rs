//! The note editor (便签与待办): an activatable popup over a note fence's content hosting a
//! multi-line EDIT control (the fence itself stays `WS_EX_NOACTIVATE`, as for the rename popup).
//!
//! Enter starts a new line. Esc, Ctrl+Enter and a click anywhere else (focus loss) save.

use crate::commands::{Command, CommandQueue};
use pecofence_core::FenceId;
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, edit, msg};
use pecofence_render::ThemeMode;
use std::cell::RefCell;

const NOTE_EDIT_CLASS: &str = "PecoFence.NoteEdit";
/// Side margin in DIPs.
const MARGIN_DIP: f32 = 10.0;

thread_local! {
    static ACTIVE: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
    /// The text the last editor closed with, until the App has taken it (`take_committed`):
    /// an editor opened on that note before then starts from it, not from the older text the
    /// App still has.
    static COMMITTED: RefCell<Option<(FenceId, String)>> = const { RefCell::new(None) };
}

/// Closes the editor without going through the command queue (PecoFence is quitting): the note
/// and its text, if one was open.
pub fn take_open() -> Option<(FenceId, String)> {
    let session = ACTIVE.with(|a| a.borrow_mut().take())?;
    let text = session.edit.text();
    Some((session.fence, text))
}

/// The App applied the editor's text for `fence`: forget it.
pub fn take_committed(fence: FenceId) {
    COMMITTED.with(|c| {
        let mut c = c.borrow_mut();
        if c.as_ref().is_some_and(|(id, _)| *id == fence) {
            *c = None;
        }
    });
}

struct Session {
    window: Window,
    edit: edit::EditControl,
    fence: FenceId,
    queue: CommandQueue,
}

/// Opens the editor over `rect` (screen px) with `text`, the caret after the last line, its
/// text `font_dip` DIPs high (the note's 文字大小).
#[allow(clippy::too_many_arguments)]
pub fn open(
    rect: RECT,
    dpi: u32,
    fence: FenceId,
    text: &str,
    queue: CommandQueue,
    mode: ThemeMode,
    font_dip: f32,
) {
    // An editor still open (or just closed) on this note holds newer text than the App has:
    // its commit is only queued. Start from that text.
    end_active(true);
    let committed = COMMITTED.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|(id, _)| *id == fence)
            .map(|(_, t)| t.clone())
    });
    let text = committed.as_deref().unwrap_or(text);
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(NOTE_EDIT_CLASS, ClassOptions::default()).ok();
        }
    });
    let (x, y) = (rect.left, rect.top);
    let (w, h) = (
        (rect.right - rect.left).max(80),
        (rect.bottom - rect.top).max(60),
    );
    let handler: MessageHandler = Box::new(
        move |_hwnd: HWND, message: u32, wparam: usize, _lparam: isize| -> Option<isize> {
            match message {
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_KILLFOCUS => {
                    end_active(true);
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
    let created = CLASS.with(|c| {
        let c = c.borrow();
        let class = c.as_ref()?;
        // Topmost, like the rename popup: during Peek the fences float in the topmost band. It
        // closes as soon as it loses focus, so it never stays over another app.
        WindowBuilder::new(class)
            .title("note")
            .style(style::POPUP)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else {
        tracing::warn!(%fence, "note editor window failed");
        return;
    };
    // The EDIT control breaks lines at "\r\n" only.
    let shown = text.replace("\r\n", "\n").replace('\n', "\r\n");
    let ctl = match edit::EditControl::create_multiline(
        window.hwnd(),
        w,
        h,
        dpi,
        font_dip,
        MARGIN_DIP,
        &shown,
        mode == ThemeMode::Dark,
        on_key,
    ) {
        Ok(ctl) => ctl,
        Err(e) => {
            tracing::warn!(%fence, error = %e, "note editor control failed");
            return;
        }
    };
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    ctl.focus_at_end();
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(Session {
            window,
            edit: ctl,
            fence,
            queue,
        })
    });
}

fn on_key(vk: u32) -> bool {
    let ctrl = window::key_down(msg::VK_CONTROL);
    if vk == msg::VK_ESCAPE || (vk == msg::VK_RETURN && ctrl) {
        end_active(true);
        return true;
    }
    false
}

/// Closes the editor (if open); `commit` hands its text to the App.
pub fn end_active(commit: bool) {
    let session = ACTIVE.with(|a| a.borrow_mut().take());
    if let Some(s) = session {
        if commit {
            let text = s.edit.text();
            tracing::info!(fence = %s.fence, chars = text.chars().count(), "note editor closed");
            COMMITTED.with(|c| *c.borrow_mut() = Some((s.fence, text.clone())));
            s.queue.push(Command::NoteEdited {
                fence: s.fence,
                text,
            });
        }
        drop(s.edit);
        drop(s.window);
    }
}
