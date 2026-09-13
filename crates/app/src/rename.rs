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

/// Opens (or re-targets) the rename popup over `fence_hwnd`'s title bar.
pub fn begin_inline_rename(
    fence_hwnd: HWND,
    fence: FenceId,
    current: String,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    let r = window::window_rect(fence_hwnd);
    let dpi = monitors::dpi_for_window(fence_hwnd).max(96);
    let scale = dpi as f32 / 96.0;
    let x = r.left + (8.0 * scale) as i32;
    let y = r.top + (4.0 * scale) as i32;
    let w = ((r.right - r.left) - (52.0 * scale) as i32).max((80.0 * scale) as i32);
    let h = (24.0 * scale) as i32;
    begin_rename_at(
        RECT {
            left: x,
            top: y,
            right: x + w,
            bottom: y + h,
        },
        dpi,
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

/// The extension hidden by the shell's display name must survive rename. Item paths are
/// case-folded, so comparing them to the visible name must ignore case too.
pub fn rename_hidden_suffix(display_name: &str, file_name: &str, is_folder: bool) -> String {
    if is_folder {
        return String::new();
    }
    let file_lower = file_name.to_lowercase();
    let display_lower = display_name.to_lowercase();
    file_lower
        .strip_prefix(&display_lower)
        .map(str::to_string)
        .unwrap_or_else(|| {
            std::path::Path::new(file_name)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        })
}

/// Opens the rename popup over an item's label rectangle (screen pixels). `select_end` limits
/// the initial selection to the first N UTF-16 units (the stem); `None` selects everything.
pub fn begin_item_rename(
    label_rect: RECT,
    dpi: u32,
    item: ItemId,
    current: String,
    select_end: Option<usize>,
    queue: CommandQueue,
    mode: ThemeMode,
) {
    let scale = dpi.max(96) as f32 / 96.0;
    let min_w = (96.0 * scale) as i32;
    let h = (24.0 * scale) as i32;
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
        RenameTarget::Item(item),
        current,
        select_end,
        queue,
        mode,
    );
}

fn begin_rename_at(
    r: RECT,
    dpi: u32,
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
        WindowBuilder::new(class)
            .title("rename")
            .style(style::POPUP)
            .ex_style(style::EX_TOOLWINDOW)
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
            end_active(true);
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
    use super::{rename_hidden_suffix, rename_select_len};

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
