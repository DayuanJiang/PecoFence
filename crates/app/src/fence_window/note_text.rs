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

/// 便签与待办: the press held on the text of a to-do in a note that is not being edited.
/// Released where it was pressed it is the click it always was (`click`: the editor opens
/// with the caret there). Once the pointer leaves the system's drag threshold it is a drag:
/// the line follows the pointer up and down, the lines it passes make room, and the release
/// leaves it where it is shown. A check box and a web address are no handles: they are
/// buttons, whose press is taken back by sliding off them.
pub(super) struct TodoDrag {
    /// Client point of the press.
    pub(super) start: (i32, i32),
    pub(super) click: Command,
    /// The pressed to-do's line in the note's text.
    pub(super) from: usize,
    /// The line it is shown at (it changes as the to-do passes its neighbours).
    pub(super) to: usize,
    /// The pointer left the drag threshold: the line is lifted.
    pub(super) moved: bool,
    /// The pointer is outside the fence: the line shows back in its place, and a release
    /// there leaves it there.
    pub(super) outside: bool,
    /// The pointer's y minus the line's top at the press (DIPs).
    pub(super) grab_dy: f32,
    /// The pointer's client y now. Its place in the note's text is worked out from the
    /// scroll offset whenever it is needed (`lifted_todo`), so the line stays under the
    /// pointer whatever scrolls the note: the edge scroll, the wheel, a key.
    pub(super) client_y: i32,
    /// Where the note was scrolled to at the press: a drag that is taken back goes there.
    pub(super) scroll_from: f32,
    /// The edge scroll: when the pointer came into the zone, and the frame it last moved.
    pub(super) auto: Option<(Instant, Instant)>,
    /// Pressed with Alt in an open editor, which is closing: the keyboard goes from the
    /// editor to the fence, and that can take the mouse capture away once. The press
    /// survives it, and the next move takes the capture again.
    pub(super) editor_closing: bool,
}

/// The line that line `j` of a text is shown at while its line `from` is shown at `to`.
fn shown_at(j: usize, from: usize, to: usize) -> usize {
    if j == from {
        return to;
    }
    let among_others = if j < from { j } else { j - 1 };
    if among_others < to {
        among_others
    } else {
        among_others + 1
    }
}

impl FenceViewState {
    /// The dragged to-do once it is lifted: its line in the note's text, the line it is
    /// shown at, the pointer's y in the text as the note is scrolled now (content DIPs
    /// before scrolling) and the pointer's y minus the line's top at the press.
    fn lifted_todo(&self) -> Option<(usize, usize, f32, f32)> {
        let d = self.todo_drag.as_ref().filter(|d| d.moved && !d.outside)?;
        let pointer = (d.client_y - self.content_y) as f32 / self.scale() + self.scroll_y;
        Some((d.from, d.to, pointer, d.grab_dy))
    }

    /// The note's text as it is shown: while a to-do is dragged, with that line at the
    /// place it has reached.
    pub(super) fn note_shown(&self) -> Option<String> {
        let text = self.note.as_deref()?;
        let moved = self
            .lifted_todo()
            .and_then(|(from, to, ..)| note::move_lines(text, from, 1, to));
        Some(moved.unwrap_or_else(|| text.to_string()))
    }

    /// The top (content DIPs before scrolling) and the height of each line of `text` as the
    /// note draws them.
    fn note_row_spans(&self, text: &str) -> Vec<(f32, f32)> {
        let lines = note_draw_lines(text, false);
        self.chrome
            .note_row_spans(self.note_width(), self.text_size, &lines)
            .unwrap_or_default()
    }

    /// Where each line of the note's text is drawn now (its top, by its line in the text):
    /// its place, the glide to it, or under the pointer for the dragged to-do, kept inside
    /// the text. With them, the place of each line as shown (`note_row_spans`).
    fn note_line_tops(&self, now: Instant) -> (Vec<f32>, Vec<(f32, f32)>) {
        let spans = self
            .note_shown()
            .map(|shown| self.note_row_spans(&shown))
            .unwrap_or_default();
        let lifted = self.lifted_todo();
        let at = |j: usize| lifted.map_or(j, |(from, to, ..)| shown_at(j, from, to));
        let mut tops: Vec<f32> = (0..spans.len())
            .map(|j| spans.get(at(j)).map_or(0.0, |s| s.0))
            .collect();
        for (j, t) in self.note_slide.iter().filter(|(_, t)| !t.is_done(now)) {
            if let Some(top) = tops.get_mut(*j) {
                *top = t.value_at(now);
            }
        }
        if let Some((from, to, pointer, grab)) = lifted
            && let (Some(first), Some(last), Some(own)) =
                (spans.first(), spans.last(), spans.get(to))
            && let Some(top) = tops.get_mut(from)
        {
            *top = (pointer - grab).clamp(first.0, (last.0 + last.1 - own.1).max(first.0));
        }
        (tops, spans)
    }

    /// What `redraw_note` adds while a to-do is dragged or lines glide to their places: how
    /// far below its place each shown line is drawn (whole pixels), and the lifted line with
    /// how much of its plate shows. None while every line rests.
    #[allow(clippy::type_complexity)]
    pub(super) fn note_drag_draw(
        &mut self,
        now: Instant,
    ) -> Option<(Vec<f32>, Option<(usize, f32)>)> {
        self.note_slide.retain(|(_, t)| !t.is_done(now));
        if self
            .note_lift
            .as_ref()
            .is_some_and(|(_, t)| t.is_done(now) && t.target() <= 0.0)
        {
            self.note_lift = None;
        }
        let lifted = self.lifted_todo();
        if self.note_slide.is_empty() && self.note_lift.is_none() && lifted.is_none() {
            return None;
        }
        let at = |j: usize| lifted.map_or(j, |(from, to, ..)| shown_at(j, from, to));
        let (tops, spans) = self.note_line_tops(now);
        let scale = self.scale();
        let mut offsets = vec![0.0; spans.len()];
        for (j, top) in tops.iter().enumerate() {
            if let (Some(offset), Some(span)) = (offsets.get_mut(at(j)), spans.get(at(j))) {
                *offset = ((top - span.0) * scale).round() / scale;
            }
        }
        let lift = self
            .note_lift
            .as_ref()
            .map(|(j, t)| (at(*j), t.value_at(now)));
        Some((offsets, lift))
    }

    /// Whether lines of the note are still moving (a glide, the plate of the lifted line
    /// coming or going): the frame clock goes on.
    pub(super) fn note_lines_moving(&self, now: Instant) -> bool {
        !self.note_slide.is_empty()
            || self
                .note_lift
                .as_ref()
                .is_some_and(|(_, t)| !t.is_done(now))
    }

    /// After the lines changed places: every line whose top (`prev`, by its line in the
    /// text) is not its place glides there over 167 ms point-to-point, as the tabs beside a
    /// dragged tab do (at once with animations off). The dragged to-do keeps following the
    /// pointer.
    fn sync_note_slides(&mut self, prev: &[f32], now: Instant) {
        let (_, spans) = self.note_line_tops(now);
        let lifted = self.lifted_todo().map(|l| (l.0, l.1));
        let dur = self.anim_dur(motion::FAST);
        self.note_slide.retain(|(_, t)| !t.is_done(now));
        for (j, from) in prev.iter().enumerate() {
            let known = self.note_slide.iter().position(|(k, _)| *k == j);
            let slot = match lifted {
                Some((dragged, _)) if dragged == j => None,
                Some((dragged, to)) => spans.get(shown_at(j, dragged, to)),
                None => spans.get(j),
            };
            match (slot, known) {
                (Some(slot), Some(k)) => {
                    self.note_slide[k]
                        .1
                        .retarget(slot.0, dur, Curve::PointToPoint, now)
                }
                (Some(slot), None) if (from - slot.0).abs() >= 0.5 => self
                    .note_slide
                    .push((j, Tween::new(*from, slot.0, dur, Curve::PointToPoint, now))),
                (None, Some(k)) => {
                    self.note_slide.remove(k);
                }
                _ => {}
            }
        }
        self.content_busy = true;
        self.frames.request();
    }

    /// The to-do whose line is at client y `y` of the note as it is shown (the editor's
    /// text while it is open): its line in the text, and the point's y minus the line's top
    /// (DIPs).
    pub(super) fn todo_line_at(&self, y: i32) -> Option<(usize, f32)> {
        let text = match &self.note_edit {
            Some(e) => e.text.as_str(),
            None => self.note.as_deref()?,
        };
        let cy = (y - self.content_y) as f32 / self.scale() + self.scroll_y;
        let spans = self.note_row_spans(text);
        // A line owns its text and half the gap to its neighbours (3 DIPs or more).
        let line = spans.iter().position(|(top, h)| cy < top + h + 3.0)?;
        let top = spans[line].0;
        let todo = matches!(note::lines(text).get(line), Some(NoteLine::Todo { .. }));
        (todo && cy >= top - 3.0).then_some((line, cy - top))
    }

    /// Button-down on the text of a note that is not being edited, at client point (`x`,
    /// `y`). `click` is what a click there does (the editor opens). On a to-do the press is
    /// held here: its click waits for the release, since the press may become a drag of the
    /// line (the caller takes the mouse capture). Anywhere else nothing can be dragged and
    /// `click` comes back to be done at once, as it always was.
    pub(super) fn begin_todo_press(&mut self, x: i32, y: i32, click: Command) -> Option<Command> {
        let Some((line, grab_dy)) = self.todo_line_at(y) else {
            return Some(click);
        };
        tracing::debug!(
            fence = %self.active,
            line,
            "to-do pressed: the click waits for the release"
        );
        self.todo_drag = Some(TodoDrag {
            start: (x, y),
            click,
            from: line,
            to: line,
            moved: false,
            outside: false,
            grab_dy,
            client_y: y,
            scroll_from: self.scroll_target(),
            auto: None,
            editor_closing: self.note_edit.is_some(),
        });
        None
    }

    /// The pointer moved to client point (`x`, `y`) with a to-do press held. None while the
    /// press is still a click (inside the drag threshold); Some(true) on the move that
    /// lifts the to-do, Some(false) while it follows the pointer.
    pub(super) fn todo_drag_move(&mut self, x: i32, y: i32) -> Option<bool> {
        // A press with Alt in an open editor: the editing ends first (its text becomes the
        // note's), the line is lifted on the move after that.
        if self.note_edit.is_some() {
            return None;
        }
        let d = self.todo_drag.as_ref()?;
        let from = d.from;
        let started = !d.moved;
        if started {
            let (tx, ty) = window::drag_threshold();
            if (x - d.start.0).abs() <= tx && (y - d.start.1).abs() <= ty {
                return None;
            }
        }
        let was_outside = d.outside;
        let now = Instant::now();
        // Out of the fence by more than the distance that tears a tab off: the line goes
        // back to its place, and comes back under the pointer when the pointer does. The
        // lines glide from where they are drawn at that moment.
        let far = (TAB_DETACH_DIP * self.scale()) as i32;
        let (w, h) = self.chrome_panel.size_px();
        let outside = x < -far || y < -far || x >= w + far || y >= h + far;
        let prev = (outside != was_outside).then(|| self.note_line_tops(now).0);
        if let Some(d) = self.todo_drag.as_mut() {
            d.moved = true;
            d.outside = outside;
            d.client_y = y;
            d.editor_closing = false;
        }
        if started {
            self.hide_tip();
            self.note_lift = Some((
                from,
                self.motion
                    .tween(0.0, 1.0, motion::FASTER, Curve::Linear, now),
            ));
            tracing::debug!(fence = %self.active, line = from, "to-do drag starts");
        }
        if let Some(prev) = prev {
            tracing::debug!(fence = %self.active, outside, "to-do drag: pointer out of the fence");
            self.sync_note_slides(&prev, now);
        }
        // The line changes places with its neighbours and is drawn once per frame
        // (`redraw_note` from `tick`), not once per mouse message: a mouse reports up to
        // 1000 times a second, and every look at the lines lays their text out.
        self.content_busy = true;
        self.frames.request();
        Some(started)
    }

    /// The dragged to-do changes places with the lines it has passed: one line per step
    /// (its middle past the neighbour's middle, as a dragged tab), repeated so that a fast
    /// move crosses several lines at once. Called before the note is drawn and before the
    /// line is dropped.
    pub(super) fn reorder_dragged_todo(&mut self, now: Instant) {
        let Some((from, shown_to, pointer, grab)) = self.lifted_todo() else {
            return;
        };
        let Some(text) = self.note.clone() else {
            return;
        };
        let (prev, _) = self.note_line_tops(now);
        let mut to = shown_to;
        for _ in 0..prev.len() {
            let shown = note::move_lines(&text, from, 1, to).unwrap_or_else(|| text.clone());
            let spans = self.note_row_spans(&shown);
            let Some(over) = tab_reorder_at_pointer(to, pointer, grab, &spans) else {
                break;
            };
            to = over;
        }
        if to != shown_to {
            if let Some(d) = self.todo_drag.as_mut() {
                d.to = to;
            }
            tracing::debug!(fence = %self.active, from, to, "to-do drag: the line changes places");
            self.sync_note_slides(&prev, now);
        }
    }

    /// One frame of the edge scroll of a dragged to-do, by the rule of a dragged icon
    /// (`drag_scroll_zone`, `drag_scroll_speed`): with the pointer in the 20 DIPs at the top
    /// or bottom edge of what the fence shows of the note, or past that edge, the note
    /// scrolls that way, faster the nearer the edge, after the same short wait. The line
    /// stays under the pointer and changes places as the lines come by (`reorder_dragged_todo`
    /// before the draw). True while it scrolled.
    pub(super) fn todo_autoscroll(&mut self, now: Instant) -> bool {
        let Some((client_y, auto)) = self
            .todo_drag
            .as_ref()
            .filter(|d| d.moved && !d.outside)
            .map(|d| (d.client_y, d.auto))
        else {
            return false;
        };
        let (top, bottom) = (self.content_y, self.content_y + self.content_size_px().1);
        let zone_px = (DRAG_SCROLL_ZONE_DIP * self.scale()) as i32;
        let zone = drag_scroll_zone(client_y.clamp(top, bottom), top, bottom, zone_px)
            .filter(|_| self.scroll_max() > 0.0);
        let Some((dir, depth)) = zone else {
            if let Some(d) = self.todo_drag.as_mut() {
                d.auto = None;
            }
            return false;
        };
        let (armed_at, last) = auto.unwrap_or((now, now));
        if let Some(d) = self.todo_drag.as_mut() {
            d.auto = Some((armed_at, now));
        }
        if now.saturating_duration_since(armed_at) < DRAG_SCROLL_ARM {
            // Still waiting: nothing moved, the clock goes on.
            self.frames.request();
            return false;
        }
        let dt = now.saturating_duration_since(last).as_secs_f32().min(0.05);
        let (layout, view_h) = self.scroll_extent();
        let speed = drag_scroll_speed(depth, self.scroll_row_step(&layout));
        let old = self.scroll_y;
        self.scroll_anim = None;
        self.scroll_y = (old + f32::from(dir) * speed * dt).clamp(0.0, layout.max_scroll(view_h));
        if self.scroll_y == old {
            return false;
        }
        self.note_scroll_activity();
        true
    }

    /// The note press ends: the button was released (`commit`), or Esc or a lost capture
    /// took it back. A press that never moved gives its click. A lifted to-do settles into
    /// the place it is shown at (the note's text changes here, and the command returned
    /// tells the app to store it) or, taken back or released outside the fence, into the
    /// place it came from. The caller pushes the command and redraws the content.
    pub(super) fn end_todo_drag(&mut self, commit: bool) -> Option<Command> {
        self.todo_drag.as_ref()?;
        let now = Instant::now();
        if commit {
            // The pointer may have moved since the last frame.
            self.reorder_dragged_todo(now);
        }
        let (prev, _) = self.note_line_tops(now);
        let d = self.todo_drag.take()?;
        if !d.moved {
            return commit.then_some(d.click);
        }
        let from = d.from;
        let to = if commit && !d.outside { d.to } else { from };
        // The edge scroll moved the note: a line that goes back takes the note back to where
        // it was scrolled, a dropped one leaves it with a whole line at its top edge.
        if (self.scroll_y - d.scroll_from).abs() > 0.5 {
            let rest = if to == from {
                d.scroll_from
            } else {
                self.tool_line_snap(self.scroll_y)
            };
            tracing::debug!(fence = %self.active, from = self.scroll_y, to = rest, "to-do drag ends: the note's scroll comes to rest");
            self.scroll_to(rest, motion::FAST);
        }
        let moved = self
            .note
            .as_deref()
            .and_then(|text| note::move_lines(text, from, 1, to));
        tracing::debug!(
            fence = %self.active,
            from,
            to,
            commit,
            outside = d.outside,
            "to-do drag ends"
        );
        // The tops the lines are drawn at, by the line each one is from now on.
        let mut tops = prev.clone();
        if let Some(text) = moved {
            for (j, top) in prev.iter().enumerate() {
                if let Some(slot) = tops.get_mut(shown_at(j, from, to)) {
                    *slot = *top;
                }
            }
            self.note_slide.clear();
            self.note = Some(text);
            self.note_boxes.clear();
            self.note_links.clear();
        }
        let shown = self
            .note_lift
            .as_ref()
            .map_or(1.0, |(_, t)| t.value_at(now));
        self.note_lift = Some((
            to,
            self.motion
                .tween(shown, 0.0, motion::FAST, Curve::Linear, now),
        ));
        self.sync_note_slides(&tops, now);
        (to != from).then_some(Command::MoveNoteLine {
            fence: self.active,
            from,
            to,
        })
    }

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

#[cfg(test)]
mod tests {
    use super::shown_at;

    /// While a to-do is dragged, every other line keeps its order around the place the
    /// dragged one is shown at.
    #[test]
    fn lines_around_a_dragged_todo() {
        let shown = |from, to| (0..5).map(|j| shown_at(j, from, to)).collect::<Vec<_>>();
        // The fourth line shown second, the second line shown fourth, a line in its place.
        assert_eq!(shown(3, 1), [0, 2, 3, 1, 4]);
        assert_eq!(shown(1, 3), [0, 3, 1, 2, 4]);
        assert_eq!(shown(2, 2), [0, 1, 2, 3, 4]);
        // The first line to the end and the last one to the top.
        assert_eq!(shown(0, 4), [4, 0, 1, 2, 3]);
        assert_eq!(shown(4, 0), [1, 2, 3, 4, 0]);
    }
}
