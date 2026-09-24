//! Auto-sorting rule engine (plan §8). Pure functions over `ItemFacts`.

use crate::model::{FenceId, Origin, RuleId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Inbox,
    Fence(FenceId),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub enabled: bool,
    pub target: Target,
    /// All conditions must hold (AND).
    pub all_of: Vec<Cond>,
    pub priority_class: Class,
    /// The "快速添加" template this rule came from (see [`Template::key`]), so the template
    /// is offered once even after the user renames the rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
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
        } else if all_of.iter().any(|c| {
            matches!(
                c,
                Cond::CreatedTime { .. } | Cond::CreatedWeekday(_) | Cond::IdleDays { .. }
            )
        }) {
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
            template: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Setup packages: `.msi`/`.msix`/`.appx` families and `.exe` files whose name says
    /// setup/install. Listed after `Programs` in the enum but matched first when a rule using
    /// it precedes the Programs rule (first match wins).
    Installers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum StrOp {
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Is,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Days since the item was last written or last opened from a fence, whichever is later,
    /// is at least `min`. Never matches when the age is unknown.
    IdleDays {
        min: u32,
    },
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
    /// Whole days since the later of last write and last open; `None` when unknown.
    pub idle_days: Option<u32>,
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
        TypeCategory::Installers => {
            if facts.is_folder {
                return false;
            }
            match ext.as_str() {
                ".msi" | ".msix" | ".msixbundle" | ".appx" | ".appxbundle" | ".appinstaller" => {
                    true
                }
                ".exe" => {
                    let name = facts.file_name.to_lowercase();
                    name.contains("setup") || name.contains("install")
                }
                _ => false,
            }
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
        Cond::IdleDays { min } => facts.idle_days.is_some_and(|d| d >= *min),
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

    /// Whether any enabled rule depends on the clock (idle days), so it must be re-run
    /// periodically rather than only when the desktop changes.
    pub fn has_idle_rules(&self) -> bool {
        self.list
            .iter()
            .any(|r| r.enabled && r.all_of.iter().any(|c| matches!(c, Cond::IdleDays { .. })))
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

/// Idle threshold of the "待清理" template: installers and archives untouched this long are
/// gathered (never deleted) so the user can decide about them.
pub const CLEANUP_IDLE_DAYS: u32 = 30;

/// "快速添加" presets: one click creates a fence and the rule that fills it. Titles are
/// localized when the template is applied and stored as plain data, like the first-run fences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    Images,
    Music,
    Video,
    Archives,
    Installers,
    /// Installers and archives idle for [`CLEANUP_IDLE_DAYS`]: gathered, not deleted.
    Cleanup,
}

impl Template {
    pub const ALL: [Template; 6] = [
        Template::Images,
        Template::Music,
        Template::Video,
        Template::Archives,
        Template::Installers,
        Template::Cleanup,
    ];

    /// Stable identifier used by the settings page and stored in [`Rule::template`].
    pub fn key(self) -> &'static str {
        match self {
            Template::Images => "images",
            Template::Music => "music",
            Template::Video => "video",
            Template::Archives => "archives",
            Template::Installers => "installers",
            Template::Cleanup => "cleanup",
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }

    /// Fence title and rule name in the current UI language.
    pub fn title(self) -> String {
        crate::i18n::text(match self {
            Template::Images => "图片",
            Template::Music => "音乐",
            Template::Video => "视频",
            Template::Archives => "压缩包",
            Template::Installers => "安装包",
            Template::Cleanup => "待清理",
        })
        .to_string()
    }

    /// The rule that fills `fence`, tagged with this template's key.
    pub fn rule(self, fence: FenceId) -> Rule {
        let conds = match self {
            Template::Images => vec![Cond::Type(vec![TypeCategory::Images])],
            Template::Music => vec![Cond::Type(vec![TypeCategory::Music])],
            Template::Video => vec![Cond::Type(vec![TypeCategory::Video])],
            Template::Archives => vec![Cond::Type(vec![TypeCategory::Archives])],
            Template::Installers => vec![Cond::Type(vec![TypeCategory::Installers])],
            Template::Cleanup => vec![
                Cond::Type(vec![TypeCategory::Installers, TypeCategory::Archives]),
                Cond::IdleDays {
                    min: CLEANUP_IDLE_DAYS,
                },
            ],
        };
        let mut rule = Rule::new(&self.title(), Target::Fence(fence), conds);
        rule.template = Some(self.key().to_string());
        rule
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
    fn installers_are_setup_packages_and_setup_named_exes() {
        let is = |name: &str| category_matches(TypeCategory::Installers, &facts(name));
        assert!(is("vlc-3.0.21-win64.msi"));
        assert!(is("App.msixbundle"));
        assert!(is("Steam-Setup.exe"));
        assert!(is("node-v22-x64-installer.exe"));
        assert!(!is("notepad.exe"));
        assert!(!is("setup.txt"));
        let mut folder = facts("Setup");
        folder.is_folder = true;
        assert!(!category_matches(TypeCategory::Installers, &folder));
    }

    #[test]
    fn idle_days_needs_a_known_age() {
        let c = Cond::IdleDays { min: 30 };
        let mut f = facts("old-setup.exe");
        assert!(!cond_matches(&c, &f), "unknown age never matches");
        f.idle_days = Some(29);
        assert!(!cond_matches(&c, &f));
        f.idle_days = Some(30);
        assert!(cond_matches(&c, &f));
    }

    #[test]
    fn cleanup_template_gathers_only_idle_installers_and_archives() {
        let fence = Uuid::new_v4();
        let rule = Template::Cleanup.rule(fence);
        assert_eq!(rule.template.as_deref(), Some("cleanup"));
        assert_eq!(rule.priority_class, Class::Type);
        let rs = RuleSet {
            list: vec![rule],
            ..Default::default()
        };
        let mut old_zip = facts("backup.zip");
        old_zip.idle_days = Some(45);
        assert!(
            matches!(rs.evaluate(&old_zip), Decision::Route { target: Target::Fence(t), .. } if t == fence)
        );
        let mut fresh_zip = facts("backup.zip");
        fresh_zip.idle_days = Some(2);
        assert_eq!(fresh_zip.file_name, "backup.zip");
        assert_eq!(rs.evaluate(&fresh_zip), Decision::Default(Target::Inbox));
        let mut old_doc = facts("thesis.docx");
        old_doc.idle_days = Some(400);
        assert_eq!(rs.evaluate(&old_doc), Decision::Default(Target::Inbox));
        assert!(rs.has_idle_rules());
        assert!(!RuleSet::default_presets(fence, fence, fence).has_idle_rules());
        assert_eq!(Template::parse("cleanup"), Some(Template::Cleanup));
        assert_eq!(Template::parse("nope"), None);
    }

    #[test]
    fn idle_template_rule_is_recognised_for_ordering() {
        let f = Uuid::new_v4();
        let cleanup = Template::Cleanup.rule(f);
        let installers = Template::Installers.rule(f);
        assert!(
            cleanup
                .all_of
                .iter()
                .any(|c| matches!(c, Cond::IdleDays { .. }))
        );
        assert!(
            !installers
                .all_of
                .iter()
                .any(|c| matches!(c, Cond::IdleDays { .. }))
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
