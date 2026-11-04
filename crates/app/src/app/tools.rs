//! 小工具: fences that show a tool in place of desktop items. 便签与待办 shows lines of text (a
//! line starting with `[ ]` is a to-do whose check box ticks with a click); 时钟与日期 shows the
//! time and date.

use super::*;
use pecofence_core::FenceTool;

impl App {
    /// Switches `tool` on in the settings when it is off (a new fence of it would have no
    /// window otherwise).
    fn ensure_tool_on(&mut self, tool: FenceTool) {
        if tool.enabled(&self.state.config.settings.tools) {
            return;
        }
        let mut settings = self.state.config.settings.clone();
        match tool {
            FenceTool::Note => settings.tools.notes = true,
            FenceTool::Clock => settings.tools.clock = true,
        }
        self.apply_settings(settings, "new tool fence");
    }

    /// A clock fence near (`x`, `y`).
    pub(super) fn new_clock_near(&mut self, x: i32, y: i32) {
        self.ensure_tool_on(FenceTool::Clock);
        if !self.fence_room_or_notice() {
            return;
        }
        let rect = self.place_new_fence_dip(240.0, 140.0, x, y, None);
        let title = pecofence_core::i18n::text("时钟");
        let Some(id) = self.state.new_tool_fence(title, rect, FenceTool::Clock) else {
            return;
        };
        self.resync_windows();
        if let Some(w) = self.fences.get(&id) {
            w.show(true);
        }
        self.schedule_save();
    }

    /// Pushes what each window's shown fence needs of its tool (note text, clock seconds),
    /// after the tool settings changed.
    pub(super) fn refresh_tool_views(&mut self) {
        let hosts: Vec<FenceId> = self.fences.keys().copied().collect();
        for host in hosts {
            self.push_tool_view(host);
        }
    }

    /// Note text / clock for the fence `host`'s window shows.
    pub(super) fn push_tool_view(&self, host: FenceId) {
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let shown = w.active_fence();
        let tool = self.state.fence(shown).and_then(|f| f.tool);
        w.set_note(self.state.note_text(shown).map(str::to_string));
        let seconds = self.state.config.settings.tools.clock_seconds;
        w.set_clock((tool == Some(FenceTool::Clock)).then_some(seconds));
    }

    /// A note fence near (`x`, `y`), its editor open. Switches the notes tool on first if it
    /// was off (the fence would have no window otherwise).
    pub(super) fn new_note_near(&mut self, x: i32, y: i32) {
        self.ensure_tool_on(FenceTool::Note);
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
