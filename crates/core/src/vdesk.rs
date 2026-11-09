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

/// Whether a window bound to `desktops` (empty: every desktop) shows on `current` (`None`:
/// Windows names none, there is one desktop). A binding none of whose desktops Windows lists
/// (`listed`, the desktops seen last; empty when unknown) counts as every desktop until the
/// fences follow the change: one restored from a snapshot taken before a desktop was closed must
/// not stay hidden everywhere.
pub fn shows_on(desktops: &[String], current: Option<&str>, listed: &[DesktopRef]) -> bool {
    let Some(current) = current else {
        return true;
    };
    desktops.is_empty()
        || desktops.iter().any(|d| d.eq_ignore_ascii_case(current))
        || (!listed.is_empty() && !desktops.iter().any(|d| listed.iter().any(|l| l.is(d))))
}

/// A desktop of a fence's binding that Windows no longer lists, and where it went (`None`:
/// dropped).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gone {
    pub id: String,
    pub to: Option<(String, Moved)>,
}

/// A fence's desktops once Windows lists `now`, having listed `old` when PecoFence saw the
/// desktops last: `None` while every one of them is listed; else the ones still listed plus,
/// for each gone one, [`desktop_after_change`]'s pick (nothing for an id `old` does not know,
/// such as one from a snapshot or another PC). An empty list is every desktop.
pub fn follow(
    desktops: &[String],
    old: &[DesktopRef],
    now: &[DesktopRef],
) -> Option<(Vec<String>, Vec<Gone>)> {
    let listed = |d: &String| now.iter().any(|n| n.is(d));
    if desktops.iter().all(listed) {
        return None;
    }
    let mut keep: Vec<String> = desktops.iter().filter(|d| listed(d)).cloned().collect();
    let mut gone = Vec::new();
    for id in desktops.iter().filter(|d| !listed(d)) {
        let to = desktop_after_change(id, old, now);
        if let Some((to, _)) = &to
            && !keep.iter().any(|k| k.eq_ignore_ascii_case(to))
        {
            keep.push(to.clone());
        }
        gone.push(Gone { id: id.clone(), to });
    }
    Some((keep, gone))
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

    /// A snapshot restored after its desktop was closed: the id is not among the desktops seen
    /// last either. It shows on every desktop meanwhile, and following drops it.
    #[test]
    fn a_binding_to_desktops_windows_no_longer_lists_counts_as_every_desktop() {
        let now = refs(&[("{A}", None), ("{B}", None)]);
        let stale = vec!["{X}".to_string()];
        assert!(shows_on(&stale, Some("{A}"), &now));
        assert!(shows_on(&stale, Some("{B}"), &now));
        // Unknown list: nothing to judge by, the binding decides.
        assert!(!shows_on(&stale, Some("{A}"), &[]));
        // A listed desktop among the binding: only there.
        let mixed = vec!["{X}".to_string(), "{b}".to_string()];
        assert!(!shows_on(&mixed, Some("{A}"), &now));
        assert!(shows_on(&mixed, Some("{B}"), &now));
        assert!(shows_on(&mixed, None, &now));
        // Following with the same list (nothing was closed since): the stale id goes, and a
        // binding left with nothing is every desktop.
        let (keep, gone) = follow(&stale, &now, &now).unwrap();
        assert!(keep.is_empty());
        assert_eq!(
            gone,
            vec![Gone {
                id: "{X}".into(),
                to: None
            }]
        );
        let (keep, _) = follow(&mixed, &now, &now).unwrap();
        assert_eq!(keep, vec!["{b}".to_string()]);
        assert_eq!(follow(&["{A}".to_string()], &now, &now), None);
        // A closed one still follows its left neighbour.
        let old = refs(&[("{A}", None), ("{C}", None), ("{B}", None)]);
        let (keep, gone) = follow(&["{C}".to_string()], &old, &now).unwrap();
        assert_eq!(keep, vec!["{A}".to_string()]);
        assert_eq!(gone[0].to, Some(("{A}".into(), Moved::Neighbour)));
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
