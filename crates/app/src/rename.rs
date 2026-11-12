//! Inline fence rename: a small activatable popup hosting an EDIT control placed over the fence
//! title (plan §5.12 activation model **b** — the fence itself stays `WS_EX_NOACTIVATE`).
//!
//! Enter commits, Esc cancels, losing focus commits.

use crate::commands::{Command, CommandQueue};
use pecofence_core::{FenceId, ItemId};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, dwm, edit, monitors, msg, paint};
use pecofence_render::ColorF;
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
    /// An item's box: the frame around its edit (the fence title's edit fills its popup).
    item_box: Option<ItemBox>,
}

/// Most lines the box of an icon's name grows to: the lines a selected icon's name unfolds to
/// (`fence_window`'s `MAX_UNFOLD_LINES`). A longer name scrolls inside the box.
const ITEM_BOX_MAX_LINES: i32 = 6;
/// Narrowest text area of an icon's name box in DIPs (the label of 48 px icons at the default
/// text size), so a smaller icon size still gets a box one can type in.
const ITEM_BOX_MIN_TEXT_DIP: f32 = 64.0;

/// How far the edit of an icon's name box reaches into the frame on each side, in DIPs: the
/// control (GDI text: whole-pixel advances, a font height rounded to a pixel) can lay a name
/// out a pixel or two wider than the label under the icon does (DirectWrite), so a name that
/// just fills its lines there took one line more in the box (「a1 新年活动安排与分工」 at
/// 150 %: two lines under the icon, three in the box). Two DIPs a side are 4, 6 and 6 px at
/// 100, 125 and 150 %. The box itself is no wider for it and the centred text stays where it
/// was. Part S of polish-rename.ps1 compares the lines of names and boxes.
const ITEM_BOX_SLACK_DIP: f32 = 2.0;

/// Where the edit sits in an item's rename box (pixels): `pad_x` in from the left and right
/// edges. An icon's name starts `pad_top` under the top edge and the box is as tall as its
/// lines (`line_h` each, at most `max_lines`) plus `pad_bottom`; a row's name (`max_lines`
/// 1) is one line centred in the box.
#[derive(Clone, Copy, Debug)]
struct ItemBox {
    /// Read by the tests only, since the edit has its own inset (`edit_x`).
    #[cfg_attr(not(test), allow(dead_code))]
    pad_x: i32,
    /// Where the edit itself starts and ends: `pad_x`, less `ITEM_BOX_SLACK_DIP` for an
    /// icon's name.
    edit_x: i32,
    pad_top: i32,
    pad_bottom: i32,
    line_h: i32,
    max_lines: i32,
}

/// Opens (or re-targets) the rename popup over `fence_hwnd`'s title bar, or over `row` (screen
/// rect of the tab's row in a left tab column). `ink` is the colour the fence's text is drawn
/// in (see [`box_colors`]).
pub fn begin_inline_rename(
    fence_hwnd: HWND,
    fence: FenceId,
    current: String,
    row: Option<RECT>,
    queue: CommandQueue,
    ink: ColorF,
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
        None,
        queue,
        ink,
    );
}

/// The colours of a name box: the text in the colour of the name the box replaces (`ink`, as
/// the fence draws it: picked for the plate behind it, or 「文字颜色」 白色 / 黑色), on the field
/// that colour reads on. Light text gets the dark field, dark text the light one, whatever the
/// colour mode: in dark mode over a light wallpaper the names are dark on a light plate, and a
/// dark box was the one dark block in the fence. Returns whether the field is the dark one and
/// the text colour (a COLORREF, the ink's own alpha blended over the field).
fn box_colors(ink: ColorF) -> (bool, u32) {
    let dark = 0.2126 * ink.r + 0.7152 * ink.g + 0.0722 * ink.b > 0.5;
    let [r, g, b, _] = edit::back_color(dark).to_le_bytes();
    let over = |c: f32, field: u8| {
        let a = ink.a.clamp(0.0, 1.0);
        (c * 255.0 * a + field as f32 * (1.0 - a))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    let text = u32::from_le_bytes([over(ink.r, r), over(ink.g, g), over(ink.b, b), 0]);
    (dark, text)
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

/// Opens the rename popup over an item's label rectangle (screen pixels), its text in the
/// names' own font: `font_face` at `font_dip` DIPs. `select_end` limits the initial selection
/// to the first N UTF-16 units (the stem); `None` selects everything.
///
/// `wrap_inset_dip` (the label's side inset inside its cell) is given for an icon's name,
/// centred under the icon: the box is centred too, wraps the name at the label's width like
/// the Windows desktop's and is as tall as the name's lines, growing and shrinking while the
/// user types. `None` is a list or details row: one line over the name column, `text_scale`
/// (`layout::row_text_scale`) taller with Windows' text size. `ink` is the colour the name is
/// drawn in (see [`box_colors`]).
#[allow(clippy::too_many_arguments)]
pub fn begin_item_rename(
    label_rect: RECT,
    dpi: u32,
    font_face: &str,
    font_dip: f32,
    text_scale: f32,
    item: ItemId,
    current: String,
    select_end: Option<usize>,
    wrap_inset_dip: Option<f32>,
    queue: CommandQueue,
    ink: ColorF,
) {
    let (r, item_box) = item_box_place(label_rect, dpi, font_dip, text_scale, wrap_inset_dip);
    begin_rename_at(
        r,
        dpi,
        font_dip,
        RenameTarget::Item(item),
        current,
        select_end,
        Some((item_box, font_face)),
        queue,
        ink,
    );
}

/// Screen rectangle of an item's rename box as it opens and where its edit sits in it
/// (`line_h` is filled in once the edit exists). See [`begin_item_rename`] for the arguments.
fn item_box_place(
    label_rect: RECT,
    dpi: u32,
    font_dip: f32,
    text_scale: f32,
    wrap_inset_dip: Option<f32>,
) -> (RECT, ItemBox) {
    let scale = dpi.max(96) as f32 / 96.0;
    let px = |dip: f32| (dip * scale).round() as i32;
    // Around the text: the 1 px border and 3 DIPs at the sides, the border and 1 DIP above,
    // 1 DIP and the 2 DIP accent line below.
    let (pad_x, pad_top, pad_bottom) = (px(3.0) + 1, px(1.0) + 1, px(3.0));
    let mut r = label_rect;
    let item_box = match wrap_inset_dip {
        Some(inset) => {
            let text_w =
                (r.right - r.left - 2 * px(inset)).max(px(ITEM_BOX_MIN_TEXT_DIP * text_scale));
            let w = text_w + 2 * pad_x;
            r.left = (r.left + r.right - w) / 2;
            r.right = r.left + w;
            // The first text line sits where the label's first line was. One line tall until
            // the edit says how many the name takes (`fit_item_box`).
            r.top -= pad_top;
            r.bottom = r.top + pad_top + px(font_dip * 4.0 / 3.0) + pad_bottom;
            ItemBox {
                pad_x,
                // Never onto the 1 px border.
                edit_x: (pad_x - px(ITEM_BOX_SLACK_DIP)).max(1),
                pad_top,
                pad_bottom,
                line_h: 0,
                max_lines: ITEM_BOX_MAX_LINES,
            }
        }
        None => {
            let min_w = (96.0 * text_scale * scale) as i32;
            if r.right - r.left < min_w {
                let cx = (r.left + r.right) / 2;
                r.left = cx - min_w / 2;
                r.right = cx + min_w / 2;
            }
            r.bottom = r.top + (item_rename_h(text_scale) * scale) as i32;
            ItemBox {
                pad_x,
                edit_x: pad_x,
                pad_top: 0,
                pad_bottom: 0,
                line_h: 0,
                max_lines: 1,
            }
        }
    };
    (r, item_box)
}

/// Height of a row's rename box in DIPs (24 for the default 12 DIP names).
pub fn item_rename_h(text_scale: f32) -> f32 {
    (24.0 * text_scale).round()
}

/// Sizes an icon's name box to the lines its text takes now (the popup keeps its top edge and
/// width) and shows the text from its first line; a text with more lines than the box may
/// show keeps the caret in view instead. Rows keep their one line.
fn fit_item_box(window: &Window, ctl: &edit::EditControl, b: &ItemBox) {
    if b.max_lines <= 1 {
        return;
    }
    let count = ctl.line_count();
    let lines = count.min(b.max_lines);
    let r = window.window_rect();
    let w = r.right - r.left;
    let h = b.pad_top + lines * b.line_h + b.pad_bottom;
    if h != r.bottom - r.top {
        tracing::debug!(
            lines = count,
            shown = lines,
            height = h,
            "item rename box resized"
        );
        let _ = window.set_bounds(r.left, r.top, w, h);
        ctl.set_bounds(b.edit_x, b.pad_top, w - 2 * b.edit_x, lines * b.line_h);
        paint::invalidate(window.hwnd());
    }
    if count <= b.max_lines {
        ctl.scroll_to_top();
    } else {
        ctl.scroll_caret();
    }
}

/// The frame of an item's box around its edit, a Fluent text box like the filter box's: the
/// edit's background, a 1 px border and the accent line of a focused box along the bottom.
fn paint_item_box(hwnd: HWND, dark: bool, dpi: u32) {
    let p = paint::Paint::begin(hwnd);
    let px = |dip: f32| (dip * dpi.max(96) as f32 / 96.0).round() as i32;
    let (w, h) = window::window_rect_size(hwnd);
    let back = edit::back_color(dark);
    let border = if dark { 0x0050_5050 } else { 0x00C8_C8C8 };
    let all = RECT {
        left: 0,
        top: 0,
        right: w,
        bottom: h,
    };
    p.fill(all, back);
    p.round_rect(all, px(4.0), back, border);
    let accent = pecofence_platform::theme::accent_palette().map_or(0x00B8_5F00, |c| {
        let rgb = if dark { c.light2 } else { c.dark1 };
        u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 0])
    });
    p.fill(
        RECT {
            left: px(3.0),
            top: h - px(2.0),
            right: w - px(3.0),
            bottom: h,
        },
        accent,
    );
}

#[allow(clippy::too_many_arguments)]
fn begin_rename_at(
    r: RECT,
    dpi: u32,
    font_dip: f32,
    target: RenameTarget,
    current: String,
    select_end: Option<usize>,
    item_box: Option<(ItemBox, &str)>,
    queue: CommandQueue,
    ink: ColorF,
) {
    // An item's box edits its name in the names' own font.
    let font_face = item_box.map(|(_, face)| face);
    let item_box = item_box.map(|(b, _)| b);
    end_active(false);
    CLASS.with(|c| {
        if c.borrow().is_none() {
            *c.borrow_mut() = WindowClass::register(RENAME_CLASS, ClassOptions::default()).ok();
        }
    });
    let (x, y, w, h) = (r.left, r.top, r.right - r.left, r.bottom - r.top);
    let (dark, text_color) = box_colors(ink);
    tracing::debug!(
        ?ink,
        dark_field = dark,
        text = format_args!("{text_color:06x}"),
        "name box takes the colour of the text it replaces"
    );
    let framed = item_box.is_some();

    let handler: MessageHandler = Box::new(
        move |hwnd: HWND, message: u32, wparam: usize, _lparam: isize| -> Option<isize> {
            match message {
                msg::WM_COMMAND if msg::hi_u16(wparam) == edit::EN_KILLFOCUS => {
                    end_active(true);
                    Some(0)
                }
                // The text changed: an icon's name box follows its lines.
                msg::WM_COMMAND if framed && msg::hi_u16(wparam) == edit::EN_CHANGE => {
                    ACTIVE.with(|a| {
                        if let Ok(a) = a.try_borrow()
                            && let Some(s) = a.as_ref()
                            && let Some(b) = &s.item_box
                        {
                            fit_item_box(&s.window, &s.edit, b);
                        }
                    });
                    Some(0)
                }
                edit::WM_CTLCOLOREDIT_MSG => {
                    ACTIVE.with(|a| a.borrow().as_ref().map(|s| s.edit.ctl_color(wparam)))
                }
                msg::WM_ERASEBKGND if framed => Some(1),
                msg::WM_PAINT if framed => {
                    paint_item_box(hwnd, dark, dpi);
                    Some(0)
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
            .style(style::POPUP | style::CLIPCHILDREN)
            .ex_style(style::EX_TOOLWINDOW | style::EX_TOPMOST)
            .bounds(x, y, w, h)
            .create(handler)
            .ok()
    });
    let Some(window) = created else { return };
    let ctl = match item_box {
        // An icon's name: wraps, sized to its lines below.
        Some(b) if b.max_lines > 1 => edit::EditControl::create_wrapping(
            window.hwnd(),
            w - 2 * b.edit_x,
            h - b.pad_top - b.pad_bottom,
            dpi,
            font_dip,
            &current,
            dark,
            on_key,
        ),
        _ => edit::EditControl::create(window.hwnd(), w, h, dpi, font_dip, &current, dark, on_key),
    };
    let Ok(mut ctl) = ctl else {
        return;
    };
    ctl.set_text_color(text_color);
    if let Some(face) = font_face {
        ctl.set_font(face, font_dip, dpi);
        ctl.set_margins(0);
    }
    let item_box = item_box.map(|b| ItemBox {
        line_h: ctl.line_height(),
        ..b
    });
    if let Some(b) = &item_box {
        // Rounded as a Fluent text box; Windows draws the corners where it can (a GPU).
        let _ = dwm::set_corner_preference(window.hwnd(), dwm::CornerPreference::RoundSmall);
        if b.max_lines > 1 {
            ctl.set_bounds(b.edit_x, b.pad_top, w - 2 * b.edit_x, b.line_h);
            fit_item_box(&window, &ctl, b);
        } else {
            // A row's name: one line, centred in the box, starting where the row's text does.
            ctl.set_bounds(b.edit_x, (h - b.line_h) / 2, w - 2 * b.edit_x, b.line_h);
        }
    }
    window::show_normal(window.hwnd());
    window::bring_to_front(window.hwnd());
    match select_end {
        Some(n) => ctl.focus_and_select(0, n as i32),
        None => ctl.focus_and_select_all(),
    }
    if let Some(b) = &item_box {
        // The selection put the caret after the stem: the name still shows from its start.
        fit_item_box(&window, &ctl, b);
    }
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(RenameSession {
            window,
            edit: ctl,
            target,
            queue,
            committed: false,
            item_box,
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
    // `try_with`: when the app quits with a name box open, the box is destroyed while the
    // thread-local is torn down, and its EN_KILLFOCUS comes back here (a panic at exit before).
    let session = ACTIVE.try_with(|a| a.borrow_mut().take()).ok().flatten();
    if let Some(mut s) = session {
        // Closed by its own keys (Enter, Esc), not by a click elsewhere: the keyboard is still
        // here and goes on to the fence below. Read before the popup is destroyed.
        let had_keyboard = desktop::foreground_window() == s.window.hwnd();
        if commit && !s.committed {
            s.committed = true;
            let text = s.edit.text();
            match s.target {
                RenameTarget::Fence(fence) => {
                    s.queue.push(Command::RenameFence { fence, title: text })
                }
                RenameTarget::Item(item) => {
                    // The wrapping box takes pasted text as it comes: a line break is no
                    // part of a file name.
                    let name = text.replace(['\r', '\n'], "");
                    s.queue.push(Command::RenameItemCommit { item, name })
                }
            }
        } else {
            s.queue.push(Command::RedrawAll);
        }
        if let RenameTarget::Fence(fence) = s.target {
            tracing::debug!(%fence, commit, had_keyboard, "fence name box closed");
            if had_keyboard {
                s.queue.push(Command::FocusFence(fence));
            }
        }
        drop(s.edit);
        drop(s.window);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ITEM_BOX_MAX_LINES, box_colors, has_reserved_char, invalid_file_name, item_box_place,
        rename_hidden_suffix, rename_select_len,
    };
    use pecofence_platform::RECT;
    use pecofence_render::{Image, Theme};

    /// The name box follows the text it replaces: in dark mode over a light wallpaper the
    /// names are dark on a light plate and so is the box; in light mode over a dark one both
    /// are light on dark. Where the mode's own text is kept, so are the box's old colours.
    #[test]
    fn the_name_box_takes_the_colour_of_the_text_it_replaces() {
        // The modes' own text: the colours the box always had (COLORREFs, 0x00BBGGRR).
        assert_eq!(box_colors(Theme::dark().text_primary), (true, 0x00FF_FFFF));
        assert_eq!(
            box_colors(Theme::light().text_primary),
            (false, 0x001B_1B1B)
        );
        // Fluent, text picked from the plate: a plain wallpaper as the fence gets it.
        let crop = |theme: &Theme, grey: u8| {
            let mut image = Image::solid(32, 24, [grey; 3]);
            let m = theme.acrylic_tint();
            image.tint(m.color, m.luminosity_opacity, m.noise, m.chroma);
            image
        };
        let dark = Theme::dark();
        let on_light = dark.over_acrylic(&crop(&dark, 238), 1.0, None, None);
        assert_eq!(box_colors(on_light.text_primary), (false, 0x001B_1B1B));
        let light = Theme::light();
        let on_dark = light.over_acrylic(&crop(&light, 20), 1.0, None, None);
        assert_eq!(box_colors(on_dark.text_primary), (true, 0x00FF_FFFF));
        // Liquid Glass: its navy text on the light field, in both modes.
        for mode in [Theme::dark(), Theme::light()] {
            let glass = mode.with_liquid_glass();
            let picked = glass.over_glass(&Image::solid(16, 16, [238; 3]));
            assert_eq!(box_colors(picked.text_primary), (false, 0x0036_2013));
            let picked = glass.over_glass(&Image::solid(16, 16, [20; 3]));
            assert_eq!(box_colors(picked.text_primary), (true, 0x00FF_FFFF));
        }
        // 「文字颜色」 白色 on a light plate, 黑色 on a dark one: the box goes with the text.
        use pecofence_core::LabelColor;
        let white = on_light.with_label_color(LabelColor::White);
        assert_eq!(box_colors(white.text_primary), (true, 0x00FF_FFFF));
        let black = on_dark.with_label_color(LabelColor::Black);
        assert_eq!(box_colors(black.text_primary), (false, 0x001B_1B1B));
    }

    /// An icon's name box is centred under the icon, as wide as the label's text plus its
    /// frame (inside the cell at the default icon size), with its first line where the label's
    /// was; a row's box keeps the name column and one line.
    #[test]
    fn an_icons_name_box_is_centred_and_stays_in_its_cell() {
        // The label area of a 48 px icon's cell at 100 %: the cell's 80 px, 8 px inset.
        let label = RECT {
            left: 300,
            top: 220,
            right: 380,
            bottom: 252,
        };
        let (r, b) = item_box_place(label, 96, 12.0, 1.0, Some(8.0));
        assert_eq!(
            (r.left, r.right),
            (304, 376),
            "64 px of text, 4 px frame each side"
        );
        assert!(r.left >= label.left && r.right <= label.right);
        // The edit reaches into the frame (never onto the border): the name gets more room
        // than its label had, in a box that is no wider.
        assert!(b.edit_x >= 1 && b.edit_x < b.pad_x);
        assert!(r.right - r.left - 2 * b.edit_x > 64);
        assert_eq!(
            r.top + b.pad_top,
            label.top,
            "first line where the label's was"
        );
        assert_eq!(
            r.bottom - r.top,
            b.pad_top + 16 + b.pad_bottom,
            "one line to start"
        );
        assert_eq!(b.max_lines, ITEM_BOX_MAX_LINES);
        // 150 %: everything scales, still centred on the cell.
        let label = RECT {
            left: 450,
            top: 330,
            right: 570,
            bottom: 378,
        };
        let (r, b) = item_box_place(label, 144, 12.0, 1.0, Some(8.0));
        assert_eq!(r.left + r.right, label.left + label.right);
        assert_eq!(r.right - r.left, 96 + 2 * b.pad_x);
        assert!(r.left >= label.left && r.right <= label.right);
        // A 32 px icon's narrower cell still gets 64 px of text to type in.
        let small = RECT {
            left: 300,
            top: 220,
            right: 364,
            bottom: 252,
        };
        let (r, b) = item_box_place(small, 96, 12.0, 1.0, Some(8.0));
        assert_eq!(r.right - r.left, 64 + 2 * b.pad_x);
        assert_eq!(r.left + r.right, small.left + small.right);
        // A row: the name column as given, one line, 24 px high.
        let row = RECT {
            left: 332,
            top: 162,
            right: 712,
            bottom: 190,
        };
        let (r, b) = item_box_place(row, 96, 12.0, 1.0, None);
        assert_eq!((r.left, r.right, r.top, r.bottom), (332, 712, 162, 186));
        assert_eq!(b.max_lines, 1);
        assert_eq!(
            b.edit_x, b.pad_x,
            "a row's text starts where the row's does"
        );
        // A narrow name column still gets the 96 px minimum, centred.
        let narrow = RECT {
            left: 400,
            top: 162,
            right: 440,
            bottom: 190,
        };
        let (r, _) = item_box_place(narrow, 96, 12.0, 1.0, None);
        assert_eq!((r.left, r.right), (372, 468));
    }

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
