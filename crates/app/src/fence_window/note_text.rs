//! 便签与待办: the note while its editor is open. The fence goes on drawing the note itself,
//! from the editor's text, with the selection and a caret (`NoteEditView`), so nothing but
//! the caret changes when editing starts. The mouse in the note's text places the caret and
//! selects; the keys go to the editor's hidden control (`crate::note_edit`).

use super::*;
use pecofence_core::note::{self, NoteLine};
use pecofence_render::fence_chrome::NoteLineDraw;

/// The lines of `text` as they are drawn. A note that is being edited keeps an empty last
/// line after a line break at its end: the caret stands there.
pub(super) fn note_draw_lines(text: &str, editing: bool) -> Vec<NoteLineDraw<'_>> {
    let lines = if editing {
        note::edit_lines(text)
    } else {
        note::lines(text)
    };
    lines
        .into_iter()
        .map(|l| match l {
            NoteLine::Text(text) => NoteLineDraw { text, todo: None },
            NoteLine::Todo { done, text } => NoteLineDraw {
                text,
                todo: Some(done),
            },
        })
        .collect()
}

/// `offset` moved off the second half of a surrogate pair (a hit test counts a pair's first
/// half as one place).
fn whole_char(text: &str, offset: usize) -> usize {
    let units: Vec<u16> = text.encode_utf16().collect();
    let low = |u: u16| (0xDC00..0xE000).contains(&u);
    let high = |u: u16| (0xD800..0xDC00).contains(&u);
    if offset > 0 && offset < units.len() && low(units[offset]) && high(units[offset - 1]) {
        offset + 1
    } else {
        offset
    }
}

impl FenceViewState {
    /// Width of the content in DIPs, which a note's lines wrap to.
    fn note_width(&self) -> f32 {
        self.content_size_px().0 as f32 / self.scale()
    }

    /// Client point (`x`, `y`) is in the text area of the note shown: its content, off the
    /// scrollbar.
    pub(super) fn in_note_text(&self, x: i32, y: i32) -> bool {
        self.note.is_some()
            && !self.rolled_up
            && y >= self.content_y
            && x >= self.content_x
            && self.scrollbar_hit(x, y).is_none()
    }

    /// The place in the note's text (UTF-16 units; the editor's text while it is open)
    /// nearest to client point (`x`, `y`).
    pub(super) fn note_offset_at(&self, x: i32, y: i32) -> Option<usize> {
        let text = match &self.note_edit {
            Some(e) => e.text.as_str(),
            None => self.note.as_deref()?,
        };
        let scale = self.scale();
        let cx = (x - self.content_x) as f32 / scale;
        let cy = (y - self.content_y) as f32 / scale + self.scroll_y;
        let lines = note_draw_lines(text, true);
        let (line, at) = self
            .chrome
            .note_hit(self.note_width(), self.text_size, &lines, cx, cy)
            .ok()?;
        let spans = note::line_spans(text);
        Some(whole_char(text, note::offset_of(&spans, line, at)))
    }

    /// The caret of the note being edited: its x, its top (before scrolling) and its height
    /// in content DIPs.
    pub(super) fn note_caret_dip(&self) -> Option<(f32, f32, f32)> {
        let e = self.note_edit.as_ref()?;
        let spans = note::line_spans(&e.text);
        let (line, at) = note::locate(&spans, e.active);
        let lines = note_draw_lines(&e.text, true);
        self.chrome
            .note_caret(self.note_width(), self.text_size, &lines, line, at)
            .ok()
    }

    /// The place one line on screen above (`dir` < 0) or below the caret of the note being
    /// edited.
    pub(super) fn note_offset_vertical(&self, dir: i8) -> Option<usize> {
        let e = self.note_edit.as_ref()?;
        let spans = note::line_spans(&e.text);
        let (line, at) = note::locate(&spans, e.active);
        let lines = note_draw_lines(&e.text, true);
        let (line, at) = self
            .chrome
            .note_vertical(self.note_width(), self.text_size, &lines, line, at, dir)
            .ok()?;
        Some(whole_char(&e.text, note::offset_of(&spans, line, at)))
    }

    /// Scrolls the note being edited so that the line with the caret shows whole, with a
    /// whole line at the top edge as after the wheel. The caller redraws.
    pub(super) fn note_caret_into_view(&mut self) {
        let Some(e) = self.note_edit.as_ref() else {
            return;
        };
        // The text may just have changed: measure it first.
        let lines = note_draw_lines(&e.text, true);
        let Ok(extent) = self
            .chrome
            .note_extent(self.note_width(), self.text_size, &lines)
        else {
            return;
        };
        drop(lines);
        self.take_tool_extent(extent);
        let Some((_, top, h)) = self.note_caret_dip() else {
            return;
        };
        let view_h = (self.viewport_h_px() as f32 / self.scale()).max(1.0);
        let pad = pecofence_render::fence_chrome::NOTE_PAD_TOP;
        let at = self.scroll_target();
        let tops = &self.tool_line_tops;
        let target = if top - pad < at - 0.5 {
            // Above the view: the caret's own line goes to the top edge.
            tops.iter()
                .copied()
                .rfind(|t| *t <= top - pad + 0.5)
                .unwrap_or(0.0)
        } else if top + h > at + view_h + 0.5 {
            // Below it: the first line from which the caret's line shows whole.
            tops.iter()
                .copied()
                .find(|t| *t >= top + h - view_h - 0.5)
                .unwrap_or(top + h - view_h)
        } else {
            return;
        };
        tracing::debug!(
            fence = %self.active,
            caret_top = top,
            from = at,
            to = target,
            "note editor: scrolled to the caret"
        );
        self.scroll_anim = None;
        self.scroll_y = target.max(0.0);
        self.note_scroll_activity();
    }

    /// The selection and the caret that `redraw_note` adds while the note is edited: the
    /// selected part of each line, and the caret's line and place (None while it is blinked
    /// off).
    #[allow(clippy::type_complexity)]
    pub(super) fn note_edit_marks(
        &self,
    ) -> Option<(Vec<(usize, usize, usize, bool)>, Option<(usize, usize)>)> {
        let e = self.note_edit.as_ref()?;
        let spans = note::line_spans(&e.text);
        let (from, to) = (e.anchor.min(e.active), e.anchor.max(e.active));
        let selection = note::selection_by_line(&spans, from, to);
        let caret = e.caret_on.then(|| note::locate(&spans, e.active));
        Some((selection, caret))
    }
}
