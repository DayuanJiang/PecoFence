//! 在哪个虚拟桌面显示: where a fence bound to a Windows virtual desktop goes when Windows no
//! longer lists that desktop.

use serde::{Deserialize, Serialize};

/// A Windows virtual desktop as PecoFence saw it: Explorer's id (`{GUID}`) and the name given in
/// Task View (`None`: never renamed, Task View calls it 桌面 N by position).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DesktopRef {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl DesktopRef {
    fn is(&self, id: &str) -> bool {
        self.id.eq_ignore_ascii_case(id)
    }
}

/// Why a fence moved to another desktop (for the log).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// The desktop was closed: its windows went to its left neighbour (the right one for the
    /// first desktop), and so does the fence.
    Neighbour,
    /// None of the desktops PecoFence saw last exist (another PC, Windows made new ones): the
    /// desktop with the same Task View name, else the one at the same position.
    SameName,
    SamePosition,
}

/// Where a fence bound to `id` goes now that Windows lists `now` instead of `old` (both in Task
/// View order), when `now` no longer has `id`. `None`: nothing tells (`id` was not among the
/// desktops seen last, or there is no desktop at its place); the caller shows it on every
/// desktop rather than on none.
pub fn desktop_after_change(
    id: &str,
    old: &[DesktopRef],
    now: &[DesktopRef],
) -> Option<(String, Moved)> {
    let at = old.iter().position(|d| d.is(id))?;
    let alive = |d: &&DesktopRef| now.iter().any(|n| n.is(&d.id));
    if old.iter().any(|d| alive(&d)) {
        // Some of them are still there: this one was closed.
        return old[..at]
            .iter()
            .rev()
            .find(alive)
            .or_else(|| old[at + 1..].iter().find(alive))
            .map(|d| (d.id.clone(), Moved::Neighbour));
    }
    // A whole new set: the same name if exactly one desktop has it, else the same position.
    if let Some(name) = &old[at].name {
        let mut same = now.iter().filter(|n| {
            n.name
                .as_deref()
                .is_some_and(|m| m.eq_ignore_ascii_case(name))
        });
        if let (Some(d), None) = (same.next(), same.next()) {
            return Some((d.id.clone(), Moved::SameName));
        }
    }
    now.get(at).map(|d| (d.id.clone(), Moved::SamePosition))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refs(list: &[(&str, Option<&str>)]) -> Vec<DesktopRef> {
        list.iter()
            .map(|(id, name)| DesktopRef {
                id: id.to_string(),
                name: name.map(str::to_string),
            })
            .collect()
    }

    #[test]
    fn a_closed_desktops_fences_go_to_its_left_neighbour() {
        let old = refs(&[("{A}", None), ("{B}", None), ("{C}", None)]);
        let now = refs(&[("{A}", None), ("{C}", None)]);
        assert_eq!(
            desktop_after_change("{B}", &old, &now),
            Some(("{A}".into(), Moved::Neighbour))
        );
        // The first one closed: its right neighbour. Ids compare without regard to case.
        let now = refs(&[("{B}", None), ("{C}", None)]);
        assert_eq!(
            desktop_after_change("{a}", &old, &now),
            Some(("{B}".into(), Moved::Neighbour))
        );
        // Not among the desktops seen last: nothing tells.
        assert_eq!(desktop_after_change("{Z}", &old, &now), None);
        assert_eq!(desktop_after_change("{B}", &[], &now), None);
    }

    #[test]
    fn a_new_set_of_desktops_is_matched_by_name_then_position() {
        let old = refs(&[("{A}", Some("工作")), ("{B}", Some("游戏")), ("{C}", None)]);
        // Another PC: its "游戏" is the first desktop there.
        let now = refs(&[("{X}", Some("游戏")), ("{Y}", Some("工作")), ("{Z}", None)]);
        assert_eq!(
            desktop_after_change("{B}", &old, &now),
            Some(("{X}".into(), Moved::SameName))
        );
        // An unnamed one keeps its position; a position that is gone tells nothing.
        assert_eq!(
            desktop_after_change("{C}", &old, &now),
            Some(("{Z}".into(), Moved::SamePosition))
        );
        let two = refs(&[("{X}", None), ("{Y}", None)]);
        assert_eq!(desktop_after_change("{C}", &old, &two), None);
        // Two desktops of that name: the position decides.
        let twice = refs(&[("{X}", Some("游戏")), ("{Y}", Some("游戏"))]);
        assert_eq!(
            desktop_after_change("{B}", &old, &twice),
            Some(("{Y}".into(), Moved::SamePosition))
        );
    }
}
