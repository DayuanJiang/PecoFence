//! 便签与待办: note fences, a tool fence (小工具) that shows lines of text in place of desktop
//! items. A line starting with `[ ]` is a to-do whose check box ticks with a click.

use super::*;

impl App {
    /// A note fence near (`x`, `y`), its editor open. Switches the notes tool on first if it
    /// was off (the fence would have no window otherwise).
    pub(super) fn new_note_near(&mut self, x: i32, y: i32) {
        if !self.state.config.settings.tools.notes {
            let mut settings = self.state.config.settings.clone();
            settings.tools.notes = true;
            self.apply_settings(settings, "new note");
        }
        if !self.fence_room_or_notice() {
            return;
        }
        let rect = self.place_new_fence_dip(280.0, 240.0, x, y, None);
        let title = pecofence_core::i18n::text("便签");
        let Some(id) = self.state.new_note(title, rect, "") else {
            return;
        };
        self.resync_windows();
        if let Some(w) = self.fences.get(&id) {
            w.show(true);
        }
        self.schedule_save();
        self.edit_note(id);
    }

    /// Opens the editor over note `fence`'s content.
    pub(super) fn edit_note(&mut self, fence: FenceId) {
        let Some(text) = self.state.note_text(fence).map(str::to_string) else {
            tracing::info!(%fence, "note editor not opened: no such note");
            return;
        };
        let Some(w) = self.window_for(fence) else {
            tracing::info!(%fence, "note editor not opened: the note has no window");
            return;
        };
        let rect = w.content_screen_rect();
        let dpi = monitors::dpi_for_window(w.hwnd());
        tracing::info!(
            %fence,
            rect = ?(rect.left, rect.top, rect.right, rect.bottom),
            dpi,
            chars = text.chars().count(),
            "note editor opened"
        );
        crate::note_edit::open(rect, dpi, fence, &text, self.queue.clone(), self.theme_mode);
    }

    /// A click on a to-do's check box.
    pub(super) fn toggle_note_line(&mut self, fence: FenceId, line: usize) {
        if self.state.toggle_note_line(fence, line) {
            self.push_note(fence);
            self.schedule_save();
        }
    }

    /// The editor closed: its text replaces the note's (line breaks stored as `\n`).
    pub(super) fn note_edited(&mut self, fence: FenceId, text: &str) {
        let text = text.replace("\r\n", "\n");
        if self.state.set_note_text(fence, &text) {
            self.schedule_save();
        }
        self.push_note(fence);
    }

    /// Shows note `fence`'s current text in its window.
    pub(super) fn push_note(&self, fence: FenceId) {
        if let Some(w) = self.window_for(fence)
            && w.active_fence() == fence
        {
            w.set_note(self.state.note_text(fence).map(str::to_string));
        }
    }
}
