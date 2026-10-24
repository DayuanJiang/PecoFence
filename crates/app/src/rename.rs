//! Inline fence rename: a small activatable popup hosting an EDIT control placed over the fence
//! title (plan §5.12 activation model **b** — the fence itself stays `WS_EX_NOACTIVATE`).
//!
//! Enter commits, Esc cancels, losing focus commits.

use crate::commands::{Command, CommandQueue};
use pecofence_core::{FenceId, ItemId};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, edit, monitors, msg};
use pecofence_render::ThemeMode;
use std::cell::RefCell;

const RENAME_CLASS: &str = "PecoFence.Rename";

thread_local! {
    static ACTIVE: RefCell<Option<RenameSession>> = const { RefCell::new(None) };
    static CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

/// What the popup renames.
#[derive(Clone, Copy, Debug)]
pub enum RenameTarget {
    Fence(FenceId),
    Item(ItemId),
}

struct RenameSession {
    window: Window,
    edit: edit::EditControl,
    target: RenameTarget,
    queue: CommandQueue,
    committed: bool,
}

/// Opens (or re-targets) the rename popup over `fence_hwnd`'s title bar, or centred in `row`
/// (a screen rect: the tab's row in a title column, or the title's spot anywhere but at the start
/// of a top title row).
pub fn begin_inline_rename(
    fence_hwnd: HWND,
    fence: FenceId,
    current: String,
    row: Option<RECT>,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    let r = window::window_rect(fence_hwnd);
    let dpi = monitors::dpi_for_window(fence_hwnd).max(96);
    let scale = dpi as f32 / 96.0;
    let h = (24.0 * scale) as i32;
    let rect = match row {
        Some(row) => {
            let top = (row.top + row.bottom - h) / 2;
            RECT {
                left: row.left,
                top,
                right: row.right,
                bottom: top + h,
            }
        }
        None => {
            let x = r.left + (8.0 * scale) as i32;
            let y = r.top + (4.0 * scale) as i32;
            let w = ((r.right - r.left) - (52.0 * scale) as i32).max((80.0 * scale) as i32);
            RECT {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            }
        }
    };
    begin_rename_at(
        rect,
        dpi,
        14.0,
        RenameTarget::Fence(fence),
        current,
        None,
        queue,
        mode,
    );
}

/// How much of an item's editing name the rename box selects (UTF-16 units): the stem only, so
/// typing replaces the name and keeps the extension (Explorer since Vista). `None` = everything:
/// folders, hidden extensions (`.lnk`, Explorer's "hide extensions"), leading-dot names.
pub fn rename_select_len(editing_name: &str, path: &str, is_folder: bool) -> Option<usize> {
    if is_folder {
        return None;
    }
    let ext = std::path::Path::new(path).extension()?.to_string_lossy();
    let suffix = format!(".{ext}");
    let lower = editing_name.to_lowercase();
    if !lower.ends_with(&suffix.to_lowercase()) || editing_name.len() <= suffix.len() {
        return None;
    }
    Some(editing_name.encode_utf16().count() - suffix.encode_utf16().count())
}

/// The extension hidden by the shell's display name must survive rename. Older item paths are
/// case-folded, so comparing them to the visible name must ignore case too; the suffix keeps
/// the file's own spelling (`.JPG`).
pub fn rename_hidden_suffix(display_name: &str, file_name: &str, is_folder: bool) -> String {
    if is_folder {
        return String::new();
    }
    let file_lower = file_name.to_lowercase();
    let display_lower = display_name.to_lowercase();
    file_lower
        .strip_prefix(&display_lower)
        .map(|tail| {
            file_name
                .get(file_name.len().saturating_sub(tail.len())..)
                .filter(|t| t.to_lowercase() == tail)
                .unwrap_or(tail)
                .to_string()
        })
        .unwrap_or_else(|| {
            std::path::Path::new(file_name)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        })
}

/// A name no file can have: empty once Win32 drops trailing dots and spaces, or holding a
/// reserved character or one of the 32 control characters Windows refuses (a NUL would end
/// the name early in the rename call).
pub fn invalid_file_name(name: &str) -> bool {
    let name = name.trim().trim_end_matches(['.', ' ']);
    name.is_empty() || has_reserved_char(name) || name.chars().any(|c| c < ' ')
}

/// The characters Explorer refuses in a file name.
pub fn has_reserved_char(name: &str) -> bool {
    name.chars()
        .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
}

/// Opens the rename popup over an item's label rectangle (screen pixels). `select_end` limits
/// the initial selection to the first N UTF-16 units (the stem); `None` selects everything.
/// `text_scale` (`layout::row_text_scale`) makes the box and its 14 DIP text larger with the
/// item names (Windows' text size).
#[allow(clippy::too_many_arguments)]
pub fn begin_item_rename(
    label_rect: RECT,
    dpi: u32,
    text_scale: f32,
    item: ItemId,
    current: String,
    select_end: Option<usize>,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    let scale = dpi.max(96) as f32 / 96.0;
    let min_w = (96.0 * text_scale * scale) as i32;
    let h = (item_rename_h(text_scale) * scale) as i32;
    let mut r = label_rect;
    if r.right - r.left < min_w {
        let cx = (r.left + r.right) / 2;
        r.left = cx - min_w / 2;
        r.right = cx + min_w / 2;
    }
    r.bottom = r.top + h;
    begin_rename_at(
        r,
        dpi,
        14.0 * text_scale,
        RenameTarget::Item(item),
        current,
        select_end,
        queue,
        mode,
    );
}

/// Height of the item rename box in DIPs (24 for the default 12 DIP names).
pub fn item_rename_h(text_scale: f32) -> f32 {
    (24.0 * text_scale).round()
}

#[allow(clippy::too_many_arguments)]
fn begin_rename_at(
    r: RECT,
    dpi: u32,
    font_dip: f32,
    target: RenameTarget,
    current: String,
    select_end: Option<usize>,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    end_active(false);
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(RENAME_CLASS, ClassOptions::default()).ok();
        }
    });
    let (x, y, w, h) = (r.left, r.top, r.right - r.left, r.bottom - r.top);

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
        // Topmost: during Peek and Win+D the fences float in the topmost band, and a normal
        // popup would open behind its fence, seen only faintly through the glass (#99). The
        // popup closes as soon as it loses focus, so it never stays over another app.
        WindowBuilder::new(class)
            .title("rename")
            .style(style::POPUP)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else { return };
    let Ok(ctl) = edit::EditControl::create(
        window.hwnd(),
        w,
        h,
        dpi,
        font_dip,
        &current,
        mode == ThemeMode::Dark,
        on_key,
    ) else {
        return;
    };
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    match select_end {
        Some(n) => ctl.focus_and_select(0, n as i32),
        None => ctl.focus_and_select_all(),
    }
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(RenameSession {
            window,
            edit: ctl,
            target,
            queue,
            committed: false,
        })
    });
}

fn on_key(vk: u32) -> bool {
    match vk {
        k if k == msg::VK_RETURN => {
            // Explorer keeps the editor open on a name with a reserved character; the App says
            // why. An emptied box commits as before (`rename_item_file` keeps the name then).
            let kept = ACTIVE.with(|a| {
                let a = a.borrow();
                let Some(s) = a.as_ref() else {
                    return false;
                };
                if matches!(s.target, RenameTarget::Item(_)) && has_reserved_char(&s.edit.text()) {
                    s.queue.push(Command::RenameItemInvalid);
                    return true;
                }
                false
            });
            if !kept {
                end_active(true);
            }
            true
        }
        k if k == msg::VK_ESCAPE => {
            end_active(false);
            true
        }
        _ => false,
    }
}

/// What the open rename popup targets, if one is open.
pub fn active_target() -> Option<RenameTarget> {
    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.target))
}

/// Finishes the active session (if any). `commit` pushes the new title.
pub fn end_active(commit: bool) {
    let session = ACTIVE.with(|a| a.borrow_mut().take());
    if let Some(mut s) = session {
        if commit && !s.committed {
            s.committed = true;
            let text = s.edit.text();
            match s.target {
                RenameTarget::Fence(fence) => {
                    s.queue.push(Command::RenameFence { fence, title: text })
                }
                RenameTarget::Item(item) => {
                    s.queue.push(Command::RenameItemCommit { item, name: text })
                }
            }
        } else {
            s.queue.push(Command::RedrawAll);
        }
        drop(s.edit);
        drop(s.window);
    }
}

#[cfg(test)]
mod tests {
    use super::{has_reserved_char, invalid_file_name, rename_hidden_suffix, rename_select_len};

    #[test]
    fn names_no_file_can_have_are_invalid() {
        assert!(invalid_file_name(""));
        assert!(invalid_file_name("   "));
        assert!(invalid_file_name("..."));
        assert!(invalid_file_name("bad:name"));
        assert!(invalid_file_name("a/b"));
        assert!(invalid_file_name("what?"));
        assert!(invalid_file_name("note.txt\0junk"));
        assert!(invalid_file_name("a\tb"));
        assert!(!invalid_file_name("note"));
        assert!(!invalid_file_name("report.txt"));
        // Trailing dots and spaces are dropped by Win32, not refused.
        assert!(!invalid_file_name("note. "));
        // The editor's Enter only holds back reserved characters: an emptied box commits.
        assert!(has_reserved_char("bad:name"));
        assert!(has_reserved_char("a|b"));
        assert!(!has_reserved_char(""));
        assert!(!has_reserved_char("..."));
        assert!(!has_reserved_char("note"));
    }

    #[test]
    fn rename_keeps_only_hidden_extensions_despite_case_folded_paths() {
        assert_eq!(rename_hidden_suffix("Report.TXT", "report.txt", false), "");
        assert_eq!(rename_hidden_suffix("Report", "report.txt", false), ".txt");
        assert_eq!(
            rename_hidden_suffix("Shortcut", "shortcut.lnk", false),
            ".lnk"
        );
        assert_eq!(rename_hidden_suffix("Docs.Backup", "docs.backup", true), "");
        assert_eq!(rename_hidden_suffix("报告.PDF", "报告.pdf", false), "");
        // Feedback #25: the hidden extension keeps its case on disk.
        assert_eq!(rename_hidden_suffix("ABC", "ABC.ZIP", false), ".ZIP");
        assert_eq!(rename_hidden_suffix("abc", "ABC.Zip", false), ".Zip");
    }

    #[test]
    fn rename_selects_the_stem_only_when_the_extension_is_shown() {
        assert_eq!(
            rename_select_len("report.pdf", r"C:\d\report.pdf", false),
            Some(6)
        );
        assert_eq!(
            rename_select_len("archive.tar.gz", r"C:\d\archive.tar.gz", false),
            Some(11)
        );
        // Explorer hides `.lnk`: the editing name has no extension to keep.
        assert_eq!(
            rename_select_len("Shortcut", r"C:\d\Shortcut.lnk", false),
            None
        );
        // Leading-dot names have no extension (Rust and Explorer agree).
        assert_eq!(rename_select_len(".bashrc", r"C:\d\.bashrc", false), None);
        assert_eq!(rename_select_len("Docs", r"C:\d\Docs", true), None);
        // Non-ASCII stems count UTF-16 units.
        assert_eq!(
            rename_select_len("报告.docx", r"C:\d\报告.docx", false),
            Some(2)
        );
    }
}
