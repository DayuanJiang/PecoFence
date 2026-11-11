//! 脚本栅栏 media keys: the keyboard's play/pause, next, previous and stop keys (also sent by
//! many mice and headsets) for a script fence that asks for them with `mediakeys=true` on a row.
//! PecoFence registers them as global hotkeys while such a row is in the output of a script that
//! is running, and sends that script `mediakey:<name>` on its standard input for each press. The
//! keys are given back as soon as no script asks, so another program's player gets them again.

use super::*;
use crate::scripts::Outcome;
use pecofence_platform::hotkey;

/// What a key is called in the line sent to the script, by hotkey id from `FIRST_ID`.
const KEYS: [(u32, &str); 4] = [
    (hotkey::VK_MEDIA_PLAY_PAUSE, "playpause"),
    (hotkey::VK_MEDIA_NEXT_TRACK, "next"),
    (hotkey::VK_MEDIA_PREV_TRACK, "prev"),
    (hotkey::VK_MEDIA_STOP, "stop"),
];

const FIRST_ID: i32 = 10;

/// The `WM_HOTKEY` ids of the media keys.
pub(super) const ID_RANGE: std::ops::Range<i32> = FIRST_ID..FIRST_ID + KEYS.len() as i32;

/// The media keys taken now: which fence's script gets them, and the hotkey ids registered.
#[derive(Default)]
pub(super) struct MediaKeys {
    owner: Option<FenceId>,
    registered: Vec<i32>,
}

/// The name of the key behind hotkey `id`.
fn key_name(id: i32) -> Option<&'static str> {
    KEYS.get(usize::try_from(id - FIRST_ID).ok()?)
        .map(|(_, name)| *name)
}

/// The hotkey id of the key called `name` (for the test script).
pub(super) fn id_for(name: &str) -> Option<i32> {
    KEYS.iter()
        .position(|(_, n)| *n == name)
        .map(|i| FIRST_ID + i as i32)
}

impl App {
    /// Takes the media keys when a running script asks for them, gives them back when none does.
    /// Called after what a script prints is taken in, and when scripts end.
    pub(super) fn sync_media_keys(&mut self) {
        let owner = self.media_keys_owner();
        self.script_runs.media.owner = owner;
        let hwnd = self.control.hwnd();
        if owner.is_some() && self.script_runs.media.registered.is_empty() {
            for (index, (vk, name)) in KEYS.iter().enumerate() {
                let id = FIRST_ID + index as i32;
                if hotkey::register(hwnd, id, hotkey::mods::NOREPEAT, *vk) {
                    self.script_runs.media.registered.push(id);
                } else {
                    tracing::warn!(key = name, "media key already taken by another program");
                }
            }
            tracing::info!(
                keys = self.script_runs.media.registered.len(),
                "media keys taken for a script fence"
            );
        } else if owner.is_none() && !self.script_runs.media.registered.is_empty() {
            for id in std::mem::take(&mut self.script_runs.media.registered) {
                hotkey::unregister(hwnd, id);
            }
            tracing::info!("media keys given back");
        }
    }

    /// The running script fence that asks for the media keys: its last output has a row with
    /// `mediakeys=true`. When several do, the one that printed last.
    fn media_keys_owner(&self) -> Option<FenceId> {
        use pecofence_core::script_output::{ScriptLine, parse};
        if !self.state.config.settings.tools.enabled {
            return None;
        }
        self.script_runs
            .streams
            .keys()
            .copied()
            .filter(|id| {
                self.state
                    .fence(*id)
                    .is_some_and(|f| f.tool == Some(pecofence_core::FenceTool::Script))
            })
            .filter_map(|id| {
                let (Outcome::Done(text), at) = self.script_runs.last_of(id)? else {
                    return None;
                };
                parse(text)
                    .iter()
                    .any(|line| matches!(line, ScriptLine::Row(row) if row.media_keys))
                    .then_some((*at, id))
            })
            .max()
            .map(|(_, id)| id)
    }

    /// A media key was pressed: the script that has them gets a line for it.
    pub(super) fn media_key_pressed(&mut self, id: i32) {
        let (Some(name), Some(owner)) = (key_name(id), self.script_runs.media.owner) else {
            return;
        };
        if !self.send_to_script(owner, &format!("mediakey:{name}")) {
            // The script is gone: the keys are not its any more.
            self.sync_media_keys();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_by_hotkey_id() {
        assert_eq!(key_name(FIRST_ID), Some("playpause"));
        assert_eq!(key_name(FIRST_ID + 1), Some("next"));
        assert_eq!(key_name(FIRST_ID + 2), Some("prev"));
        assert_eq!(key_name(FIRST_ID + 3), Some("stop"));
        assert_eq!(key_name(FIRST_ID + 4), None);
        assert_eq!(key_name(1), None);
        assert!(ID_RANGE.contains(&FIRST_ID) && !ID_RANGE.contains(&(FIRST_ID + 4)));
    }
}
