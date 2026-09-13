//! Auto-sorting rule engine (plan §8). Pure functions over `ItemFacts`.

use crate::model::{FenceId, Origin, RuleId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Inbox,
    Fence(FenceId),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSet {
    pub default_target: Target,
    pub keep_updated: bool,
    /// Ordered; the first matching rule wins (after class ordering).
    pub list: Vec<Rule>,
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            default_target: Target::Inbox,
            keep_updated: true,
            list: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Class {
    /// Shortcut-target rules always evaluate first (Stardock 4.07 behaviour).
    Target,
    Type,
    Name,
    Time,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub enabled: bool,
    pub target: Target,
    /// All conditions must hold (AND).
    pub all_of: Vec<Cond>,
    pub priority_class: Class,
}

impl Rule {
    pub fn new(name: &str, target: Target, all_of: Vec<Cond>) -> Self {
        let priority_class = if all_of
            .iter()
            .any(|c| matches!(c, Cond::ShortcutTarget { .. }))
        {
            Class::Target
        } else if all_of
            .iter()
            .any(|c| matches!(c, Cond::Type(_) | Cond::Ext(_)))
        {
            Class::Type
        } else if all_of
            .iter()
            .any(|c| matches!(c, Cond::Name { .. } | Cond::Glob(_) | Cond::ExactName(_)))
        {
            Class::Name
        } else if all_of
            .iter()
            .any(|c| matches!(c, Cond::CreatedTime { .. } | Cond::CreatedWeekday(_)))
        {
            Class::Time
        } else {
            Class::Custom
        };
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            enabled: true,
            target,
            all_of,
            priority_class,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TypeCategory {
    Programs,
    Folders,
    Documents,
    Images,
    Music,
    Video,
    Archives,
    Shortcuts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrOp {
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Is,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "cond", content = "value")]
pub enum Cond {
    Type(Vec<TypeCategory>),
    Ext(Vec<String>),
    ExactName(Vec<String>),
    Name {
        op: StrOp,
        value: String,
    },
    Glob(String),
    ShortcutTarget {
        op: StrOp,
        value: String,
    },
    FoldersOnly,
    FilesOnly,
    SizeMb {
        min: Option<f64>,
        max: Option<f64>,
    },
    /// Minutes since midnight, local time; wraps past midnight when `from > to`.
    CreatedTime {
        from_min: u16,
        to_min: u16,
    },
    /// 0 = Monday … 6 = Sunday.
    CreatedWeekday(Vec<u8>),
    Origin(Origin),
}

/// Everything the engine may look at for one item.
#[derive(Clone, Debug, Default)]
pub struct ItemFacts {
    /// File name including extension (`Report.docx`, `Steam.lnk`).
    pub file_name: String,
    pub is_folder: bool,
    pub size_bytes: u64,
    /// Resolved shortcut target path / URL, if the item is a shortcut.
    pub shortcut_target: Option<String>,
    /// Extension of the shortcut target (e.g. `.exe`), lower-case with dot.
    pub shortcut_target_ext: Option<String>,
    pub created_minutes_local: Option<u16>,
    pub created_weekday: Option<u8>,
    pub origin: Origin,
    pub is_hidden: bool,
    pub is_system: bool,
}

/// Extensions that are never routed (downloads in flight, Office lock files…).
pub const TEMPORARY_EXTS: &[&str] = &[
    ".tmp",
    ".bak",
    ".crdownload",
    ".part",
    ".partial",
    ".download",
    ".!ut",
    ".opdownload",
];

pub fn ext_of(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(i) if i > 0 => file_name[i..].to_lowercase(),
        _ => String::new(),
    }
}

/// Is this a temporary/in-flight file that rules must skip?
pub fn is_temporary(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.starts_with("~$") || TEMPORARY_EXTS.iter().any(|e| lower.ends_with(e))
}

fn category_matches(cat: TypeCategory, facts: &ItemFacts) -> bool {
    let ext = ext_of(&facts.file_name);
    let target_ext = facts.shortcut_target_ext.as_deref().unwrap_or("");
    match cat {
        TypeCategory::Folders => facts.is_folder,
        TypeCategory::Programs => {
            !facts.is_folder
                && (matches!(
                    ext.as_str(),
                    ".exe" | ".msi" | ".bat" | ".cmd" | ".appref-ms"
                ) || (ext == ".lnk"
                    && matches!(target_ext, ".exe" | ".msi" | ".bat" | ".cmd" | "")))
        }
        TypeCategory::Shortcuts => {
            !facts.is_folder && matches!(ext.as_str(), ".lnk" | ".url" | ".website")
        }
        TypeCategory::Documents => {
            !facts.is_folder
                && matches!(
                    ext.as_str(),
                    ".doc"
                        | ".docx"
                        | ".xls"
                        | ".xlsx"
                        | ".ppt"
                        | ".pptx"
                        | ".pdf"
                        | ".txt"
                        | ".md"
                        | ".rtf"
                        | ".odt"
                        | ".ods"
                        | ".odp"
                        | ".csv"
                        | ".one"
                        | ".epub"
                        | ".json"
                        | ".xml"
                )
        }
        TypeCategory::Images => {
            !facts.is_folder
                && matches!(
                    ext.as_str(),
                    ".png"
                        | ".jpg"
                        | ".jpeg"
                        | ".gif"
                        | ".bmp"
                        | ".webp"
                        | ".heic"
                        | ".svg"
                        | ".tif"
                        | ".tiff"
                        | ".ico"
                        | ".psd"
                        | ".raw"
                        | ".avif"
                )
        }
        TypeCategory::Music => {
            !facts.is_folder
                && matches!(
                    ext.as_str(),
                    ".mp3" | ".flac" | ".wav" | ".m4a" | ".aac" | ".ogg" | ".wma" | ".opus"
                )
        }
        TypeCategory::Video => {
            !facts.is_folder
                && matches!(
                    ext.as_str(),
                    ".mp4" | ".mkv" | ".avi" | ".mov" | ".wmv" | ".webm" | ".m4v" | ".ts"
                )
        }
        TypeCategory::Archives => {
            !facts.is_folder
                && matches!(
                    ext.as_str(),
                    ".zip" | ".7z" | ".rar" | ".tar" | ".gz" | ".bz2" | ".xz" | ".iso" | ".cab"
                )
        }
    }
}

fn str_op(op: StrOp, haystack: &str, needle: &str) -> bool {
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    match op {
        StrOp::Contains => h.contains(&n),
        StrOp::NotContains => !h.contains(&n),
        StrOp::StartsWith => h.starts_with(&n),
        StrOp::EndsWith => h.ends_with(&n),
        StrOp::Is => h == n,
    }
}

/// Minimal glob: `*` and `?`, case-insensitive.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    fn rec(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some('*'), _) => rec(&p[1..], t) || (!t.is_empty() && rec(p, &t[1..])),
            (Some('?'), Some(_)) => rec(&p[1..], &t[1..]),
            (Some(a), Some(b))
                if a.eq_ignore_ascii_case(b) || a.to_lowercase().eq(b.to_lowercase()) =>
            {
                rec(&p[1..], &t[1..])
            }
            _ => false,
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    rec(&p, &t)
}

pub fn cond_matches(cond: &Cond, facts: &ItemFacts) -> bool {
    match cond {
        Cond::Type(cats) => cats.iter().any(|c| category_matches(*c, facts)),
        Cond::Ext(exts) => {
            let ext = ext_of(&facts.file_name);
            exts.iter().any(|e| {
                let e = e.to_lowercase();
                let e = if e.starts_with('.') {
                    e
                } else {
                    format!(".{e}")
                };
                e == ext
            })
        }
        Cond::ExactName(names) => names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&facts.file_name)),
        Cond::Name { op, value } => str_op(*op, &facts.file_name, value),
        Cond::Glob(g) => glob_match(g, &facts.file_name),
        Cond::ShortcutTarget { op, value } => facts
            .shortcut_target
            .as_deref()
            .map(|t| str_op(*op, t, value))
            .unwrap_or(false),
        Cond::FoldersOnly => facts.is_folder,
        Cond::FilesOnly => !facts.is_folder,
        Cond::SizeMb { min, max } => {
            let mb = facts.size_bytes as f64 / (1024.0 * 1024.0);
            min.is_none_or(|m| mb >= m) && max.is_none_or(|m| mb <= m)
        }
        Cond::CreatedTime { from_min, to_min } => match facts.created_minutes_local {
            Some(m) => {
                if from_min <= to_min {
                    m >= *from_min && m <= *to_min
                } else {
                    m >= *from_min || m <= *to_min
                }
            }
            None => false,
        },
        Cond::CreatedWeekday(days) => facts
            .created_weekday
            .map(|d| days.contains(&d))
            .unwrap_or(false),
        Cond::Origin(o) => facts.origin == *o,
    }
}

/// Outcome of evaluating the rule set for one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Temporary/system item: leave it alone for now.
    Skip,
    /// Matched `rule`, go to its target.
    Route { target: Target, rule: RuleId },
    /// No rule matched: default target.
    Default(Target),
}

impl RuleSet {
    pub fn evaluate(&self, facts: &ItemFacts) -> Decision {
        if is_temporary(&facts.file_name)
            || facts.is_system
            || facts.file_name.eq_ignore_ascii_case("desktop.ini")
        {
            return Decision::Skip;
        }
        let mut ordered: Vec<&Rule> = self.list.iter().filter(|r| r.enabled).collect();
        // Stable sort: Target-class first, everything else keeps user order.
        ordered.sort_by_key(|r| {
            if r.priority_class == Class::Target {
                0
            } else {
                1
            }
        });
        for rule in ordered {
            if !rule.all_of.is_empty() && rule.all_of.iter().all(|c| cond_matches(c, facts)) {
                return Decision::Route {
                    target: rule.target,
                    rule: rule.id,
                };
            }
        }
        Decision::Default(self.default_target)
    }

    /// Stardock-style first-run presets: 程序 / 文件夹 / 文件与文档 (+ 下载 by name).
    pub fn default_presets(programs: FenceId, folders: FenceId, documents: FenceId) -> Self {
        Self {
            default_target: Target::Inbox,
            keep_updated: true,
            list: vec![
                Rule::new(
                    crate::i18n::text("程序与快捷方式"),
                    Target::Fence(programs),
                    vec![Cond::Type(vec![
                        TypeCategory::Programs,
                        TypeCategory::Shortcuts,
                    ])],
                ),
                Rule::new(
                    crate::i18n::text("文件夹"),
                    Target::Fence(folders),
                    vec![Cond::Type(vec![TypeCategory::Folders])],
                ),
                Rule::new(
                    crate::i18n::text("文件与文档"),
                    Target::Fence(documents),
                    vec![Cond::Type(vec![
                        TypeCategory::Documents,
                        TypeCategory::Images,
                        TypeCategory::Music,
                        TypeCategory::Video,
                        TypeCategory::Archives,
                    ])],
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(name: &str) -> ItemFacts {
        ItemFacts {
            file_name: name.into(),
            origin: Origin::UserDesktop,
            ..Default::default()
        }
    }

    #[test]
    fn temp_files_are_skipped() {
        let rs = RuleSet::default();
        assert_eq!(rs.evaluate(&facts("setup.exe.crdownload")), Decision::Skip);
        assert_eq!(rs.evaluate(&facts("~$report.docx")), Decision::Skip);
        assert_eq!(rs.evaluate(&facts("desktop.ini")), Decision::Skip);
    }

    #[test]
    fn presets_route_by_type_and_default_to_inbox() {
        let (p, f, d) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let rs = RuleSet::default_presets(p, f, d);
        assert!(
            matches!(rs.evaluate(&facts("Steam.lnk")), Decision::Route { target: Target::Fence(t), .. } if t == p)
        );
        let mut folder = facts("Projects");
        folder.is_folder = true;
        assert!(
            matches!(rs.evaluate(&folder), Decision::Route { target: Target::Fence(t), .. } if t == f)
        );
        assert!(
            matches!(rs.evaluate(&facts("photo.JPG")), Decision::Route { target: Target::Fence(t), .. } if t == d)
        );
        assert_eq!(
            rs.evaluate(&facts("weird.xyz")),
            Decision::Default(Target::Inbox)
        );
    }

    #[test]
    fn target_class_rules_win_regardless_of_order() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let rs = RuleSet {
            default_target: Target::Inbox,
            keep_updated: true,
            list: vec![
                Rule::new(
                    "all lnk",
                    Target::Fence(a),
                    vec![Cond::Ext(vec!["lnk".into()])],
                ),
                Rule::new(
                    "games",
                    Target::Fence(b),
                    vec![Cond::ShortcutTarget {
                        op: StrOp::Contains,
                        value: "steam".into(),
                    }],
                ),
            ],
        };
        let mut f = facts("Dota.lnk");
        f.shortcut_target = Some("C:\\Program Files\\Steam\\steam.exe".into());
        assert!(
            matches!(rs.evaluate(&f), Decision::Route { target: Target::Fence(t), .. } if t == b)
        );
    }

    #[test]
    fn glob_and_time_conditions() {
        assert!(glob_match("*.PNG", "shot.png"));
        assert!(glob_match("report-??.docx", "report-01.docx"));
        assert!(!glob_match("*.png", "shot.jpg"));
        let c = Cond::CreatedTime {
            from_min: 22 * 60,
            to_min: 6 * 60,
        };
        let mut f = facts("x");
        f.created_minutes_local = Some(23 * 60);
        assert!(cond_matches(&c, &f));
        f.created_minutes_local = Some(12 * 60);
        assert!(!cond_matches(&c, &f));
    }
}
