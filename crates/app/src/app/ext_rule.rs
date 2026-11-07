//! The item menu's 「以后把 .pdf 文件都放到这里」: one click makes an extension rule that sends
//! files of that type to the fence the item is in.

use super::welcome::Welcome;
use super::*;

/// Shortcuts show without their extension, and a rule for every `.lnk` would take all of
/// them: the offer is for other files only.
const NOT_OFFERED: &[&str] = &[".lnk", ".url"];

impl App {
    /// The extension (`.pdf`, lower case) the item menu offers a rule for: one file with an
    /// extension, in a fence that holds desktop items (not a folder fence, not a folder opened
    /// inside a fence: rules sort the desktop only).
    pub(super) fn ext_rule_offer(&self, fence: FenceId, items: &[ItemId]) -> Option<String> {
        let [only] = items else {
            return None;
        };
        let item = self.state.item(*only)?;
        if item.is_folder || item.is_namespace() || self.state.portal_path(fence).is_some() {
            return None;
        }
        if !self.state.fence(fence).is_some_and(|f| f.holds_items()) {
            return None;
        }
        let name = Path::new(item.path()?)
            .file_name()?
            .to_string_lossy()
            .into_owned();
        let ext = pecofence_core::rules::ext_of(&name);
        (ext.len() > 1
            && !NOT_OFFERED.contains(&ext.as_str())
            && !pecofence_core::rules::is_temporary(&name, None))
        .then_some(ext)
    }

    /// Runs the offer: asks first when a rule already sends `ext` elsewhere, then puts the rule
    /// first in the list and tells where rules are edited.
    pub(super) fn add_ext_rule(&mut self, fence: FenceId, ext: &str, owner: HWND) {
        let target = if self.state.inbox_id() == Some(fence) {
            Target::Inbox
        } else {
            Target::Fence(fence)
        };
        let title = self.target_title(target);
        let rules = &self.state.config.rules;
        if let Some(i) = rules.extension_rule(ext) {
            let rule = &rules.list[i];
            if rule.target == target {
                tracing::info!(
                    index = i,
                    "extension rule: already routes here, nothing changed"
                );
                self.show_welcome(Welcome::RuleAdded {
                    ext: ext.to_string(),
                    fence: title,
                    already: true,
                });
                return;
            }
            let prompt = pecofence_core::i18n::format(
                "已有整理规则“{0}”把 {1} 文件放到“{2}”。改成放到“{3}”吗？",
                &[
                    rule.name.clone(),
                    ext.to_string(),
                    self.target_title(rule.target),
                    title.clone(),
                ],
            );
            if !window::confirm(owner, &prompt, pecofence_core::i18n::text("整理规则")) {
                tracing::info!(
                    index = i,
                    "extension rule: another target kept (answered No)"
                );
                return;
            }
        }
        let name = pecofence_core::i18n::format("{0} 文件", &[ext.to_string()]);
        let had = self.state.config.rules.list.len();
        let id = self
            .state
            .config
            .rules
            .put_extension_first(ext, target, &name);
        let added = self.state.config.rules.list.len() > had;
        tracing::info!(
            rule = %id,
            added,
            to_inbox = target == Target::Inbox,
            keep_updated = self.state.config.rules.keep_updated,
            "extension rule set from the item menu"
        );
        self.state.mark_dirty();
        self.schedule_save();
        self.push_settings_state();
        self.show_welcome(Welcome::RuleAdded {
            ext: ext.to_string(),
            fence: title,
            already: false,
        });
    }

    /// A rule target's name as the user sees it (「桌面」 for the inbox).
    fn target_title(&self, target: Target) -> String {
        let id = match target {
            Target::Fence(id) => Some(id),
            Target::Inbox => self.state.inbox_id(),
        };
        id.and_then(|id| self.state.fence(id))
            .map(|f| self.state.display_title(f))
            .unwrap_or_else(|| pecofence_core::i18n::text("桌面").to_string())
    }
}
