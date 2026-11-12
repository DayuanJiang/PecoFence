//! The note editor (便签与待办). A note is edited in place: its fence goes on drawing it, now
//! from the editor's text and with the selection and a caret (`Command::NoteEditState`,
//! `fence_window::note_text`), so the note looks the same while it is edited: the same
//! lines, check boxes, colours and background. The typing itself (keys, the input method,
//! the clipboard, undo) is done by a multi-line EDIT control in an invisible popup that has
//! the keyboard (the fence itself stays `WS_EX_NOACTIVATE`, as for the rename popup). The
//! control is moved so that its own caret, which the input method's window follows, is
//! where the fence draws one.
//!
//! Enter starts a new line. Esc, Ctrl+Enter and a click anywhere else (focus loss, or a
//! press in a fence window: `Command::EndNoteEdit`) save.

use crate::commands::{Command, CommandQueue};
use pecofence_core::{FenceId, note};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, edit, msg};
use pecofence_render::ThemeMode;
use std::cell::{Cell, RefCell};

/// The class of the editor's invisible popup, which has the keyboard while a note is edited.
pub const NOTE_EDIT_CLASS: &str = "PecoFence.NoteEdit";
/// The control's side margins in DIPs: the note's own text inset, so the control wraps its
/// lines about where the note does (the input method's window keeps to the control's width).
const MARGIN_DIP: f32 = pecofence_render::fence_chrome::NOTE_PAD_X;
/// Posted by the control after a message that can change its text, selection or caret.
const WM_NOTE_CHANGED: u32 = msg::WM_APP + 0x61;
const TIMER_BLINK: usize = 1;
const VK_BACK: u32 = 0x08;
const VK_END: u32 = 0x23;
const VK_HOME: u32 = 0x24;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;

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

/// Whether an editor is open.
pub fn is_open() -> bool {
    ACTIVE.with(|a| a.borrow().is_some())
}

/// The note whose editor is open.
pub fn open_for() -> Option<FenceId> {
    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.fence))
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
    /// The popup's top-left corner on screen (px).
    origin: (i32, i32),
    /// The end of the selection that stays put while the other moves (UTF-16 units, line
    /// breaks counted as `\n`).
    anchor: Cell<usize>,
    /// The text and selection last told to the fence.
    told: RefCell<Option<(String, usize, usize)>>,
    /// The caret's blink: it shows now.
    caret_on: Cell<bool>,
    /// Where the control was last moved to within the popup.
    placed: Cell<(i32, i32)>,
}

/// What a call needs of the open editor, copied out so that nothing stays borrowed while the
/// control handles a message (it may come back here: a notification, a menu's message loop).
fn active() -> Option<(edit::EditHandle, FenceId, CommandQueue)> {
    ACTIVE.with(|a| {
        a.borrow()
            .as_ref()
            .map(|s| (s.edit.handle(), s.fence, s.queue.clone()))
    })
}

/// Opens the editor of note `fence`, whose content is at `rect` (screen px), with `text`,
/// its text `font_dip` DIPs high (the note's 文字大小). The caret stands at `caret` (UTF-16
/// units of `text`), after the last line without one.
#[allow(clippy::too_many_arguments)]
pub fn open(
    rect: RECT,
    dpi: u32,
    fence: FenceId,
    text: &str,
    queue: CommandQueue,
    mode: ThemeMode,
    font_dip: f32,
    caret: Option<usize>,
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
    // The place of a click was measured in the text the fence shows.
    let caret = caret.filter(|_| committed.is_none());
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
        move |hwnd: HWND, message: u32, wparam: usize, _lparam: isize| -> Option<isize> {
            match message {
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_KILLFOCUS => {
                    end_active(true);
                    Some(0)
                }
                WM_NOTE_CHANGED => {
                    tell_fence();
                    Some(0)
                }
                msg::WM_TIMER if wparam == TIMER_BLINK => {
                    blink(hwnd);
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
        // Nothing of the popup shows and the mouse goes through it to the note. Topmost, like
        // the rename popup: during Peek the fences float in the topmost band. It closes as
        // soon as it loses focus, so it never stays over another app.
        WindowBuilder::new(class)
            .title("note")
            .style(style::POPUP)
            .ex_style(
                style::EX_TOOLWINDOW
                    | style::EX_TOPMOST
                    | style::EX_LAYERED
                    | style::EX_TRANSPARENT,
            )
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else {
        tracing::warn!(%fence, "note editor window failed");
        return;
    };
    window::set_layered_alpha(window.hwnd(), 0);
    // The EDIT control breaks lines at "\r\n" only.
    let shown = text.replace("\r\n", "\n").replace('\n', "\r\n");
    // The control keeps the body text face the note is drawn in and the note's margins, so
    // it wraps its lines about where the note does: Home and End go by its lines.
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
    ctl.notify_changes(WM_NOTE_CHANGED);
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    ctl.focus_at_end();
    let handle = ctl.handle();
    let hwnd = window.hwnd();
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(Session {
            window,
            edit: ctl,
            fence,
            queue,
            origin: (x, y),
            anchor: Cell::new(0),
            told: RefCell::new(None),
            caret_on: Cell::new(true),
            placed: Cell::new((0, 0)),
        })
    });
    if let Some(at) = caret {
        let units: Vec<u16> = handle.text().encode_utf16().collect();
        let at = note::lf_to_crlf_offset(&units, at);
        handle.select(at, at);
    }
    if let Some(ms) = edit::caret_blink_ms() {
        window::set_timer(hwnd, TIMER_BLINK, ms);
    }
    tell_fence();
}

/// Reads the control's text and selection and, when they changed, tells the note's fence
/// (`Command::NoteEditState`). The caret shows again from now (typing restarts its blink).
fn tell_fence() {
    let Some((handle, fence, queue)) = active() else {
        return;
    };
    let raw = handle.text();
    let units: Vec<u16> = raw.encode_utf16().collect();
    let (first, last) = handle.selection();
    let first = note::crlf_to_lf_offset(&units, first);
    let last = note::crlf_to_lf_offset(&units, last);
    let text = raw.replace("\r\n", "\n");
    // A marker just typed on a line that starts with one already: Enter began the line as a
    // to-do and 「[ ]」 was typed all the same, as every to-do needed before. The marker the
    // line began with goes and the typed one stays, so the line shows one check box and
    // holds what was typed. Only when the text has just changed: a caret put there by a
    // click leaves a note as it was written.
    let typed = ACTIVE.with(|a| {
        a.borrow()
            .as_ref()
            .and_then(|s| s.told.borrow().as_ref().map(|told| told.0 != text))
            .unwrap_or(false)
    });
    if first == last && typed {
        let doubled = note::line_spans(&text)
            .into_iter()
            .zip(text.split('\n'))
            .filter(|(s, _)| s.prefix > 0 && first > s.start + s.prefix && first <= s.start + s.len)
            .find_map(|(s, line)| {
                let shown = after_units(line, s.prefix);
                let note::NoteLine::Todo { text: rest, .. } = note::parse_line(shown) else {
                    return None;
                };
                let marker = shown.encode_utf16().count() - rest.encode_utf16().count();
                (first == s.start + s.prefix + marker).then_some((s.start, s.prefix, marker))
            });
        if let Some((start, prefix, marker)) = doubled {
            tracing::debug!(%fence, "note editor: a marker typed after a check box replaces it");
            let from = note::lf_to_crlf_offset(&units, start);
            handle.select(from, note::lf_to_crlf_offset(&units, start + prefix));
            handle.replace_selection("");
            handle.select(from + marker, from + marker);
            return;
        }
    }
    // A bare caret inside a to-do's marker, which is not shown (after Ctrl+Home, Ctrl+Left, a
    // Right from the line before): it goes on to the start of the line's text, so that what
    // is typed lands where the caret shows and the to-do stays one. The control tells its
    // new state right after.
    if first == last
        && let Some(span) = note::line_spans(&text)
            .into_iter()
            .find(|s| first >= s.start && first < s.start + s.prefix.min(s.len))
    {
        tracing::debug!(%fence, at = first, "note editor: the caret leaves a to-do's marker");
        let to = note::lf_to_crlf_offset(&units, span.start + span.prefix.min(span.len));
        handle.select(to, to);
        return;
    }
    ACTIVE.with(|a| {
        let a = a.borrow();
        let Some(s) = a.as_ref() else {
            return;
        };
        // The control tells the selection's two ends in order; the caret is at the one that
        // moved.
        let (anchor, active) = if first == last {
            (first, first)
        } else if last == s.anchor.get() {
            (last, first)
        } else {
            (first, last)
        };
        s.anchor.set(anchor);
        let state = (text, anchor, active);
        if s.told.borrow().as_ref() == Some(&state) {
            return;
        }
        tracing::debug!(
            %fence,
            chars = state.0.chars().count(),
            anchor,
            active,
            "note editor: text or selection changed"
        );
        if let Some(ms) = edit::caret_blink_ms() {
            window::set_timer(s.window.hwnd(), TIMER_BLINK, ms);
        }
        s.caret_on.set(true);
        queue.push(Command::NoteEditState {
            fence,
            text: state.0.clone(),
            anchor,
            active,
        });
        *s.told.borrow_mut() = Some(state);
    });
}

/// `s` without its first `units` UTF-16 units.
fn after_units(s: &str, units: usize) -> &str {
    let mut seen = 0;
    for (i, c) in s.char_indices() {
        if seen >= units {
            return &s[i..];
        }
        seen += c.len_utf16();
    }
    ""
}

/// The blink timer: the caret hides or shows.
fn blink(hwnd: HWND) {
    ACTIVE.with(|a| {
        let a = a.borrow();
        let Some(s) = a.as_ref().filter(|s| s.window.hwnd() == hwnd) else {
            return;
        };
        let on = !s.caret_on.get();
        s.caret_on.set(on);
        s.queue.push(Command::NoteCaretBlink { fence: s.fence, on });
    });
}

/// The fence drew the caret of note `fence` at screen point (`x`, `top`): the control moves
/// so that its own caret is there, and the input method's window with it.
pub fn place_caret(fence: FenceId, x: i32, top: i32) {
    let Some((handle, open, _)) = active() else {
        return;
    };
    if open != fence {
        return;
    }
    let Some((cx, cy)) = handle.caret_pos() else {
        return;
    };
    let to = ACTIVE.with(|a| {
        let a = a.borrow();
        let s = a.as_ref()?;
        let to = (x - s.origin.0 - cx, top - s.origin.1 - cy);
        (s.placed.replace(to) != to).then_some(to)
    });
    if let Some((x, y)) = to {
        handle.move_to(x, y);
    }
}

/// The caret of note `fence`'s editor goes to `offset` (UTF-16 units, line breaks counted as
/// `\n`): `extend` keeps the other end of the selection where it is, `word` selects the word
/// at `offset`.
pub fn set_caret(fence: FenceId, offset: usize, extend: bool, word: bool) {
    let Some((handle, open, _)) = active() else {
        return;
    };
    if open != fence {
        return;
    }
    let raw = handle.text();
    let units: Vec<u16> = raw.encode_utf16().collect();
    let anchor = ACTIVE.with(|a| a.borrow().as_ref().map_or(0, |s| s.anchor.get()));
    let (anchor, active) = if word {
        note::word_at(&raw.replace("\r\n", "\n"), offset)
    } else if extend {
        (anchor, offset)
    } else {
        (offset, offset)
    };
    ACTIVE.with(|a| {
        if let Some(s) = a.borrow().as_ref() {
            s.anchor.set(anchor);
        }
    });
    handle.select(
        note::lf_to_crlf_offset(&units, anchor),
        note::lf_to_crlf_offset(&units, active),
    );
}

/// 「添加待办」 in the note's menu: an empty to-do after the last line of note `fence`, whose
/// editor is open, with the caret in it. It goes in as typed text (Ctrl+Z takes it back).
pub fn add_todo(fence: FenceId) {
    let Some((handle, open, _)) = active() else {
        return;
    };
    if open != fence {
        return;
    }
    let raw = handle.text();
    let end = raw.encode_utf16().count();
    let add = note::new_todo_suffix(&raw.replace("\r\n", "\n"));
    tracing::debug!(%fence, add, "note editor: to-do added from the menu");
    handle.select(end, end);
    if !add.is_empty() {
        handle.replace_selection(&add.replace('\n', "\r\n"));
    }
}

/// Opens the control's text menu (cut, copy, paste, select all) at screen point (`x`, `y`).
pub fn text_menu(x: i32, y: i32) {
    if let Some((handle, ..)) = active() {
        handle.context_menu(x, y);
    }
}

fn on_key(vk: u32) -> bool {
    let ctrl = window::key_down(msg::VK_CONTROL);
    let shift = window::key_down(msg::VK_SHIFT);
    if vk == msg::VK_ESCAPE || (vk == msg::VK_RETURN && ctrl) {
        end_active(true);
        return true;
    }
    let Some((handle, fence, queue)) = active() else {
        return false;
    };
    // Up and Down follow the lines as the fence wraps them, not as the control does.
    if (vk == VK_UP || vk == VK_DOWN) && !ctrl {
        queue.push(Command::NoteCaretLine {
            fence,
            dir: if vk == VK_UP { -1 } else { 1 },
            extend: shift,
        });
        return true;
    }
    // Home, End and Left keep clear of a to-do's marker, which is not shown. Home goes to the
    // start of the line's text and End to its end (a line that wraps counts as one line:
    // the control wraps its own lines at other places than the note). Left at the start of a
    // to-do's text goes to the end of the line before.
    if matches!(vk, VK_HOME | VK_END | VK_LEFT) && !ctrl {
        let raw = handle.text();
        let units: Vec<u16> = raw.encode_utf16().collect();
        let (first, last) = handle.selection();
        let first = note::crlf_to_lf_offset(&units, first);
        let last = note::crlf_to_lf_offset(&units, last);
        let anchor = ACTIVE.with(|a| a.borrow().as_ref().map_or(0, |s| s.anchor.get()));
        let caret = if first != last && last == anchor {
            first
        } else {
            last
        };
        let spans = note::line_spans(&raw.replace("\r\n", "\n"));
        let line = spans.iter().rposition(|s| s.start <= caret).unwrap_or(0);
        let to = spans.get(line).and_then(|span| {
            let text_start = span.start + span.prefix.min(span.len);
            match vk {
                VK_HOME => Some(text_start),
                VK_END => Some(span.start + span.len),
                _ if span.prefix > 0 && caret == text_start && (shift || first == last) => {
                    Some(match line.checked_sub(1).and_then(|i| spans.get(i)) {
                        Some(before) => before.start + before.len,
                        None => caret,
                    })
                }
                _ => None,
            }
        });
        if let Some(to) = to {
            set_caret(fence, to, shift, false);
            return true;
        }
    }
    // Enter in a to-do starts the next line with an empty check box, and in a to-do without
    // text takes the check box away, as lists do in editors (feedback #141). Shift+Enter is
    // the control's plain line break.
    if vk == msg::VK_RETURN && !shift {
        let raw = handle.text();
        let units: Vec<u16> = raw.encode_utf16().collect();
        let (first, last) = handle.selection();
        let text = raw.replace("\r\n", "\n");
        let does = note::todo_enter(
            &text,
            note::crlf_to_lf_offset(&units, first),
            note::crlf_to_lf_offset(&units, last),
        );
        tracing::debug!(%fence, ?does, "note editor: Enter");
        match does {
            Some(note::TodoEnter::NewBox(add)) => {
                handle.replace_selection(&add.replace('\n', "\r\n"));
                return true;
            }
            Some(note::TodoEnter::RemoveBox(from, to)) => {
                handle.select(
                    note::lf_to_crlf_offset(&units, from),
                    note::lf_to_crlf_offset(&units, to),
                );
                handle.replace_selection("");
                return true;
            }
            None => {}
        }
    }
    // Backspace right after a to-do's check box takes the check box away: its marker is not
    // shown, so deleting it character by character would seem to do nothing at first. All of
    // the marker but its last character goes here; the control deletes that one itself when
    // the key's character arrives.
    if vk == VK_BACK && !ctrl {
        let raw = handle.text();
        let units: Vec<u16> = raw.encode_utf16().collect();
        let (first, last) = handle.selection();
        if first == last {
            let at = note::crlf_to_lf_offset(&units, first);
            let text = raw.replace("\r\n", "\n");
            let marker = note::line_spans(&text)
                .into_iter()
                .find(|s| s.prefix > 0 && at == s.start + s.prefix);
            if let Some(span) = marker {
                tracing::debug!(%fence, "note editor: Backspace after a check box removes it");
                let start = note::lf_to_crlf_offset(&units, span.start);
                if first > start + 1 {
                    handle.select(start, first - 1);
                    handle.replace_selection("");
                    handle.select(start + 1, start + 1);
                }
            }
        }
    }
    // Enter on a to-do goes on with the list: the new line starts with an open check box. On
    // a to-do without text the check box goes instead and the list ends, as in any list
    // editor. The control adds the line break itself when the key's character arrives, so
    // the text is made ready for it here: the new line's marker put after the caret, or the
    // empty to-do taken away together with the line break before it.
    if vk == msg::VK_RETURN && !ctrl && !shift {
        let raw = handle.text();
        let units: Vec<u16> = raw.encode_utf16().collect();
        let (first, last) = handle.selection();
        if first == last {
            let at = note::crlf_to_lf_offset(&units, first);
            let text = raw.replace("\r\n", "\n");
            let todo = note::line_spans(&text)
                .into_iter()
                .rfind(|s| s.start <= at)
                .filter(|s| s.prefix > 0 && at >= s.start + s.prefix.min(s.len));
            if let Some(span) = todo {
                let empty = span.len <= span.prefix;
                tracing::debug!(%fence, empty, "note editor: Enter on a to-do");
                if empty {
                    let from = note::lf_to_crlf_offset(&units, span.start.saturating_sub(1));
                    let to = note::lf_to_crlf_offset(&units, span.start + span.len);
                    handle.select(from, to);
                    handle.replace_selection("");
                } else {
                    handle.replace_selection("[ ] ");
                    handle.select(first, first);
                }
            }
        }
    }
    false
}

/// Closes the editor (if open); `commit` hands its text to the App.
pub fn end_active(commit: bool) {
    let session = ACTIVE.with(|a| a.borrow_mut().take());
    if let Some(s) = session {
        // Closed by its own keys (Esc, Ctrl+Enter) or by a click on something that takes no
        // focus (a clock, a script fence): the keyboard is still here and goes on to the
        // note's fence. A click that moved the focus keeps it where it went. Read before the
        // popup is destroyed.
        let had_keyboard = pecofence_platform::desktop::foreground_window() == s.window.hwnd();
        tracing::debug!(fence = %s.fence, commit, had_keyboard, "note editor closing");
        if had_keyboard {
            s.queue.push(Command::FocusFence(s.fence));
        }
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
