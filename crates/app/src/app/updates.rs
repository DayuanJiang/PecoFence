use super::*;

impl App {
    pub(super) fn update_action(&mut self, action: &str) {
        if action == "openReleases" {
            if let Some(url) = self.updater.open_release_url() {
                // URL is constructed from validated package identity, never from the page.
                let _ = shell::shell_execute(Path::new(&url), None, Some(self.control.hwnd()));
            }
            return;
        }
        let operation = match action {
            "checkUpdates" => "Check",
            "downloadUpdate" => "Download",
            "installUpdate" => "Apply",
            "recoverUpdate" => "Recover",
            _ => return,
        };
        if self.updater.busy() {
            return;
        }
        if matches!(operation, "Apply" | "Recover") {
            self.save_config();
            if self.state.is_dirty() || self.state.save_error.is_some() {
                self.updater
                    .fail("Could not save settings before the update".into());
                self.push_update_state();
                return;
            }
        }
        if let Err(error) = self.updater.begin(operation) {
            self.updater.fail(error);
        }
        if self.updater.busy() {
            window::set_timer(self.control.hwnd(), TIMER_UPDATES, 250);
        }
        self.push_update_state();
    }

    pub(super) fn poll_updates(&mut self) {
        let before = self.updater.snapshot();
        if self.updater.poll() {
            // Save again after worker validation: settings/IPC may have changed meanwhile.
            self.save_config();
            // save_config returns false for both a failed save and a clean config.
            if !self.state.is_dirty() && self.state.save_error.is_none() {
                self.updater.detach();
                if let Some(settings) = self.settings.take() {
                    settings.close();
                }
                window::post_quit(0);
            } else {
                self.updater.cancel();
                self.updater
                    .fail("Could not save settings before the update".into());
            }
        }
        if !self.updater.busy() {
            window::kill_timer(self.control.hwnd(), TIMER_UPDATES);
        }
        if self.updater.snapshot() != before {
            self.push_update_state();
        }
    }

    fn push_update_state(&self) {
        if let Some(settings) = &self.settings {
            settings.post_json(
                &serde_json::json!({"type":"updates", "updates":self.updater.snapshot()})
                    .to_string(),
            );
        }
    }
}
