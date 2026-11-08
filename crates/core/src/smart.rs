//! Smart fences (智能栅栏): a fence that shows the live result of a saved search instead of
//! desktop items. The search is one or more folders (or every drive), the conditions of the
//! sorting rules and a time window, all of them or any one of them (`match`); with Everything
//! running it becomes an Everything search, without it the app reads the folders and tests the
//! same conditions itself (`matches`). The files are never moved: the fence lists them where
//! they are.

use crate::rules::{self, Cond, ItemFacts, StrOp, TypeCategory};
use serde::{Deserialize, Serialize};

/// What a smart fence searches. Saved on the fence (`Fence::smart`); versions without smart
/// fences ignore the field and show an empty fence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SmartSpec {
    /// The template it was made from ([`Template::key`]); None for one written by hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// The folders searched, together; empty = every drive (only with Everything). A search
    /// saved before there could be several has one `folder`, read as the only one.
    #[serde(
        default,
        alias = "folder",
        deserialize_with = "one_or_many",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub folders: Vec<String>,
    /// Subfolders of the folders too.
    #[serde(default = "default_true")]
    pub subfolders: bool,
    /// Whether a file must meet every condition, the time window, the name and the Everything
    /// text, or one of them is enough. The folders and subfolders always apply. Not written
    /// when it is `all`, so a search saved before `any` reads and writes back the same.
    #[serde(default, rename = "match", skip_serializing_if = "SmartMatch::is_all")]
    pub matching: SmartMatch,
    /// The conditions of the sorting rules (all of them, or one with `match` any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<Cond>,
    /// Only what was created or written in the last this many days (the later of the two: a
    /// download keeps the server's date as its write time with some download tools).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_days: Option<u32>,
    /// Only names that match this text, as the Peek search and 筛选 match names (every word,
    /// pinyin too, not the loosest letters-in-order match). Tested by the app on every file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// An Everything search typed by the user, sent with the folders, the conditions and the
    /// time window, which all still apply (as does the name; with `match` any, one of them is
    /// enough). Needs Everything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub everything: Option<String>,
    /// Most results shown, newest first.
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// 全部 / 任一 of a smart fence's search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SmartMatch {
    /// Every condition must hold.
    #[default]
    All,
    /// One condition is enough.
    Any,
}

impl SmartMatch {
    pub fn is_all(&self) -> bool {
        *self == SmartMatch::All
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SmartMatch::All => "all",
            SmartMatch::Any => "any",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "all" => Some(SmartMatch::All),
            "any" => Some(SmartMatch::Any),
            _ => None,
        }
    }
}

/// `folders` as a list, or one folder as text (`folder` of a search saved before there could
/// be several).
fn one_or_many<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(Option<String>),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(one) => one.into_iter().collect(),
        OneOrMany::Many(many) => many,
    })
}

fn default_true() -> bool {
    true
}

fn default_limit() -> u32 {
    SmartSpec::DEFAULT_LIMIT
}

/// An empty search: every drive, every file (with Everything), subfolders, newest 200.
impl Default for SmartSpec {
    fn default() -> Self {
        Self {
            template: None,
            folders: Vec::new(),
            subfolders: true,
            matching: SmartMatch::All,
            all_of: Vec::new(),
            within_days: None,
            name: None,
            everything: None,
            limit: Self::DEFAULT_LIMIT,
        }
    }
}

impl SmartSpec {
    pub const DEFAULT_LIMIT: u32 = 200;
    /// Most entries the folder reading without Everything looks at.
    pub const MAX_SCANNED: usize = 20_000;
    /// Results asked of Everything when the app filters names itself.
    pub const NAME_FILTER_ASK: u32 = 5_000;

    /// Whether the search can run without Everything: it has a folder to read and no
    /// Everything-only text.
    pub fn works_without_everything(&self) -> bool {
        !self.folders.is_empty() && self.everything.is_none()
    }

    /// Whether a condition looks at shortcut targets (the target condition, the program type),
    /// which costs a COM round trip per shortcut to read.
    pub fn needs_shortcut_targets(&self) -> bool {
        self.all_of.iter().any(|c| match c {
            Cond::ShortcutTarget { .. } => true,
            Cond::Type(cats) => cats.contains(&TypeCategory::Programs),
            _ => false,
        })
    }

    /// Whether only the app can tell which files match: the name filter, or a condition
    /// Everything has no term for (shortcut target, creation time or weekday, idle days).
    /// Everything's count is then more than the fence shows.
    pub fn app_checks(&self) -> bool {
        self.name.as_deref().is_some_and(|n| !n.trim().is_empty())
            || self.all_of.iter().any(|c| everything_term(c).is_none())
    }

    /// Why the search cannot be saved (a 来源 condition, which says where a desktop icon came
    /// from and means nothing for files found anywhere; a limit out of range). None when it can.
    pub fn problem(&self) -> Option<String> {
        if self.all_of.iter().any(|c| matches!(c, Cond::Origin(_))) {
            return Some("origin is not a smart fence condition".into());
        }
        if !(1..=2000).contains(&self.limit) {
            return Some(format!("limit {} is not in 1..2000", self.limit));
        }
        if self.folders.iter().any(|f| f.trim().is_empty()) {
            return Some("an empty folder".into());
        }
        None
    }

    /// The Everything search text: the folders (always), then the user's own Everything text,
    /// the conditions Everything can test and the time window (`since_date`, see
    /// [`everything_date`]): all of them, or with `any` one of them; over every drive, the
    /// system folders are left out. The app tests every condition again on the answer
    /// (`matches`), so a condition left out here only costs results.
    pub fn everything_query(&self, since_date: Option<&str>) -> String {
        let mut terms: Vec<String> = Vec::new();
        let folders: Vec<String> = self
            .folders
            .iter()
            .map(|f| {
                let f = f.trim_end_matches('\\');
                if self.subfolders {
                    quote(&format!("{f}\\"))
                } else {
                    format!("parent:{}", quote(f))
                }
            })
            .collect();
        match folders.len() {
            0 => {
                // Every drive: not Windows itself, program data or the Recycle Bin.
                terms.extend(
                    [
                        "!\"C:\\Windows\\\"",
                        "!\\AppData\\",
                        "!\"\\$Recycle.Bin\\\"",
                    ]
                    .map(str::to_string),
                );
            }
            1 => terms.extend(folders),
            _ => terms.push(format!("<{}>", folders.join("|"))),
        }
        // The other rows: None for one only the app can test.
        let mut rows: Vec<Option<String>> = Vec::new();
        if let Some(raw) = self.everything.as_deref().map(str::trim)
            && !raw.is_empty()
        {
            rows.push(Some(raw.to_string()));
        }
        rows.extend(self.all_of.iter().map(everything_term));
        if self.within_days.is_some() {
            rows.push(since_date.map(|date| format!("<dm:>={date}|dc:>={date}>")));
        }
        if self.name.as_deref().is_some_and(|n| !n.trim().is_empty()) {
            rows.push(None);
        }
        match self.matching {
            SmartMatch::All => terms.extend(rows.into_iter().flatten()),
            // One row is enough: Everything can narrow the search only when it can test every
            // row (a row only the app tests may hold for any file). Each row is grouped on its
            // own: Everything's `|` binds closer than a space.
            SmartMatch::Any => {
                if let Some(rows) = rows.into_iter().collect::<Option<Vec<String>>>() {
                    match rows.len() {
                        0 => {}
                        1 => terms.extend(rows),
                        _ => terms.push(format!(
                            "<{}>",
                            rows.iter()
                                .map(|r| format!("<{r}>"))
                                .collect::<Vec<_>>()
                                .join("|")
                        )),
                    }
                }
            }
        }
        terms.join(" ")
    }

    /// Whether one file found (by Everything or in the folders) belongs in the fence: it was
    /// created or written inside the time window, its name matches and every condition holds,
    /// or with `any` one of these holds. `newest` (the later of creation and last write) and
    /// `now` are Unix seconds. With `any` and an Everything text, Everything's answer is taken
    /// as it is: the app cannot tell which row a file matched.
    pub fn matches(&self, facts: &ItemFacts, newest: i64, now: i64) -> bool {
        if rules::is_temporary(&facts.file_name, facts.age_secs) {
            return false;
        }
        let window = self
            .within_days
            .map(|days| newest >= now - i64::from(days) * 86_400);
        let name = self
            .name
            .as_deref()
            .map(|q| q.trim().to_lowercase())
            .filter(|q| !q.is_empty())
            .map(|q| {
                crate::matcher::match_name(&facts.file_name, &q)
                    .is_some_and(|m| !crate::matcher::is_loose_score(m.score))
            });
        let mut tests = window
            .into_iter()
            .chain(name)
            .chain(self.all_of.iter().map(|c| rules::cond_matches(c, facts)));
        match self.matching {
            SmartMatch::All => tests.all(|ok| ok),
            SmartMatch::Any => {
                let typed = self
                    .everything
                    .as_deref()
                    .is_some_and(|t| !t.trim().is_empty());
                let mut any_row = false;
                for ok in tests {
                    if ok {
                        return true;
                    }
                    any_row = true;
                }
                // No row at all, or one only Everything tested: what was found matches.
                typed || !any_row
            }
        }
    }

    /// How many results to ask Everything for: the name filter and the conditions Everything
    /// cannot test are tested here on the answer, so it takes more than the fence shows.
    pub fn everything_max(&self) -> u32 {
        if self.app_checks() {
            self.limit.max(Self::NAME_FILTER_ASK)
        } else {
            self.limit
        }
    }
}

/// Quotes an Everything term; a double quote inside becomes the `&quot:` entity.
fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "&quot:"))
}

/// The Everything form of a rule condition, when it has one: the same files or more (the app
/// tests every condition again on the answer), never fewer.
fn everything_term(cond: &Cond) -> Option<String> {
    Some(match cond {
        Cond::Ext(list) => {
            let list: Vec<&str> = list.iter().map(String::as_str).collect();
            exts_term(&list)?
        }
        Cond::Type(cats) => {
            let mut folders = false;
            let mut all: Vec<&str> = Vec::new();
            for cat in cats {
                match cat {
                    TypeCategory::Folders => folders = true,
                    // A program or a shortcut is told by more than its extension (a shortcut's
                    // target, a setup's name): these extensions take every one of them in.
                    TypeCategory::Programs => {
                        all.extend([".exe", ".msi", ".bat", ".cmd", ".appref-ms", ".lnk"])
                    }
                    TypeCategory::Installers => all.extend([
                        ".msi",
                        ".msix",
                        ".msixbundle",
                        ".appx",
                        ".appxbundle",
                        ".appinstaller",
                        ".exe",
                    ]),
                    TypeCategory::Shortcuts => all.extend([".lnk", ".url", ".website"]),
                    c => all.extend_from_slice(rules::category_exts(*c)),
                }
            }
            match (folders, exts_term(&all)) {
                (true, Some(e)) => format!("<folder:|{e}>"),
                (true, None) => "folder:".into(),
                (false, Some(e)) => e,
                (false, None) => return None,
            }
        }
        Cond::Name { op, value } => match op {
            StrOp::Contains => quote(value),
            StrOp::NotContains => format!("!{}", quote(value)),
            StrOp::StartsWith => format!("startwith:{}", quote(value)),
            StrOp::EndsWith => format!("endwith:{}", quote(value)),
            StrOp::Is => format!("wfn:{}", quote(value)),
        },
        Cond::Glob(g) => format!("wfn:{}", quote(g)),
        Cond::ExactName(names) => {
            let parts: Vec<String> = names.iter().map(|n| format!("wfn:{}", quote(n))).collect();
            format!("<{}>", parts.join("|"))
        }
        Cond::FilesOnly | Cond::PlainFiles => "file:".into(),
        Cond::FoldersOnly => "folder:".into(),
        Cond::SizeMb { min, max } => {
            let bytes = |mb: f64| (mb * 1024.0 * 1024.0).round() as u64;
            let mut t = Vec::new();
            if let Some(m) = min {
                t.push(format!("size:>={}", bytes(*m)));
            }
            if let Some(m) = max {
                t.push(format!("size:<={}", bytes(*m)));
            }
            t.join(" ")
        }
        Cond::ShortcutTarget { .. }
        | Cond::CreatedTime { .. }
        | Cond::CreatedWeekday(_)
        | Cond::Origin(_)
        | Cond::IdleDays { .. } => return None,
    })
}

/// Extensions as one Everything term: `ext:` for plain ones, a name ending for one with a dot
/// inside (`tar.gz`: Everything's `ext:` sees only `gz`), OR'ed. None for an empty list.
fn exts_term(list: &[&str]) -> Option<String> {
    let mut plain: Vec<String> = Vec::new();
    let mut endings: Vec<String> = Vec::new();
    for e in list {
        let e = e.trim().trim_start_matches('.');
        if e.is_empty() {
            continue;
        }
        if e.contains('.') {
            endings.push(format!("endwith:{}", quote(&format!(".{e}"))));
        } else if !plain.iter().any(|p| p.eq_ignore_ascii_case(e)) {
            plain.push(e.to_string());
        }
    }
    let mut parts = endings;
    if !plain.is_empty() {
        parts.insert(0, format!("ext:{}", plain.join(";")));
    }
    match parts.len() {
        0 => None,
        1 => parts.pop(),
        _ => Some(format!("<{}>", parts.join("|"))),
    }
}

/// A local date as Everything's `dm:>=` takes it (`YYYY-MM-DD`, any Everything version and
/// region). The app passes the local date of `now - within_days`; its own test (`matches`) then
/// drops what is older to the second.
pub fn everything_date(d: crate::CivilDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// The searches offered by the editor of a new smart fence (新建智能栅栏…).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    /// 最近 7 天下载的文件: the Downloads folder, files written in the last 7 days.
    Downloads,
    /// 最近一周的截图: the Screenshots folder, the last 7 days.
    Screenshots,
    /// 大于 1 GB 的文件: every drive (Everything).
    Large,
    /// 所有 .psd 文件: every drive (Everything).
    Psd,
    /// 最近打开的文件: Windows' Recent folder (shortcuts to what was opened).
    Recent,
}

/// Folders a template searches, found by the app (known folders of the user).
#[derive(Clone, Debug, Default)]
pub struct KnownFolders {
    pub downloads: Option<String>,
    pub screenshots: Option<String>,
    pub recent: Option<String>,
}

impl Template {
    pub const ALL: [Template; 5] = [
        Template::Downloads,
        Template::Screenshots,
        Template::Large,
        Template::Psd,
        Template::Recent,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Template::Downloads => "downloads",
            Template::Screenshots => "screenshots",
            Template::Large => "large",
            Template::Psd => "psd",
            Template::Recent => "recent",
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }

    /// The fence's title (and the menu entry).
    pub fn title(self) -> &'static str {
        match self {
            Template::Downloads => crate::i18n::text("最近 7 天下载"),
            Template::Screenshots => crate::i18n::text("最近一周的截图"),
            Template::Large => crate::i18n::text("大于 1 GB 的文件"),
            Template::Psd => crate::i18n::text("所有 .psd 文件"),
            Template::Recent => crate::i18n::text("最近打开的文件"),
        }
    }

    /// Whether it searches every drive, which only Everything can do.
    pub fn needs_everything(self) -> bool {
        matches!(self, Template::Large | Template::Psd)
    }

    /// The search; None when its folder is not known on this computer.
    pub fn spec(self, folders: &KnownFolders) -> Option<SmartSpec> {
        let mut spec = SmartSpec {
            template: Some(self.key().to_string()),
            all_of: vec![Cond::FilesOnly],
            ..SmartSpec::default()
        };
        match self {
            Template::Downloads => {
                spec.folders = vec![folders.downloads.clone()?];
                spec.within_days = Some(7);
            }
            Template::Screenshots => {
                spec.folders = vec![folders.screenshots.clone()?];
                spec.within_days = Some(7);
                spec.all_of.push(Cond::Type(vec![TypeCategory::Images]));
            }
            Template::Large => {
                spec.all_of.push(Cond::SizeMb {
                    min: Some(1024.0),
                    max: None,
                });
            }
            Template::Psd => spec.all_of.push(Cond::Ext(vec![".psd".into()])),
            Template::Recent => {
                spec.folders = vec![folders.recent.clone()?];
                spec.subfolders = false;
                spec.all_of.push(Cond::Ext(vec![".lnk".into()]));
                spec.limit = 40;
            }
        }
        Some(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folders() -> KnownFolders {
        KnownFolders {
            downloads: Some(r"C:\Users\pf\Downloads".into()),
            screenshots: Some(r"C:\Users\pf\Pictures\Screenshots".into()),
            recent: Some(r"C:\Users\pf\AppData\Roaming\Microsoft\Windows\Recent".into()),
        }
    }

    #[test]
    fn downloads_template_searches_the_folder_tree_by_date() {
        let spec = Template::Downloads.spec(&folders()).unwrap();
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Users\pf\Downloads\" file: <dm:>=2026-10-31|dc:>=2026-10-31>"#
        );
    }

    #[test]
    fn every_drive_leaves_out_system_folders() {
        let q = Template::Large
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert_eq!(
            q,
            r#"!"C:\Windows\" !\AppData\ !"\$Recycle.Bin\" file: size:>=1073741824"#
        );
        let q = Template::Psd
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert!(q.ends_with("file: ext:psd"), "{q}");
    }

    #[test]
    fn one_folder_without_subfolders_uses_parent() {
        let q = Template::Recent
            .spec(&folders())
            .unwrap()
            .everything_query(None);
        assert_eq!(
            q,
            r#"parent:"C:\Users\pf\AppData\Roaming\Microsoft\Windows\Recent" file: ext:lnk"#
        );
    }

    #[test]
    fn conditions_become_everything_terms() {
        let t = |c: Cond| everything_term(&c);
        assert_eq!(
            t(Cond::Name {
                op: StrOp::StartsWith,
                value: "IMG_".into()
            })
            .unwrap(),
            r#"startwith:"IMG_""#
        );
        assert_eq!(
            t(Cond::Glob("*.tar.gz".into())).unwrap(),
            r#"wfn:"*.tar.gz""#
        );
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Music])).unwrap(),
            "ext:mp3;flac;wav;m4a;aac;ogg;wma;opus"
        );
        assert_eq!(t(Cond::IdleDays { min: 30 }), None);
    }

    /// What Everything is asked for takes in every file the rule condition does (the app tests
    /// the condition again on the answer): folders together with file types, shortcuts by
    /// their extensions, extensions with a dot inside by the name's ending.
    #[test]
    fn type_and_extension_terms_take_in_every_match() {
        let t = |c: Cond| everything_term(&c).unwrap();
        assert_eq!(
            t(Cond::Type(vec![
                TypeCategory::Folders,
                TypeCategory::Archives
            ])),
            "<folder:|ext:zip;7z;rar;tar;gz;bz2;xz;iso;cab>"
        );
        assert_eq!(t(Cond::Type(vec![TypeCategory::Folders])), "folder:");
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Shortcuts])),
            "ext:lnk;url;website"
        );
        assert_eq!(
            t(Cond::Type(vec![TypeCategory::Programs])),
            "ext:exe;msi;bat;cmd;appref-ms;lnk"
        );
        assert_eq!(
            t(Cond::Ext(vec![
                "tar.gz".into(),
                ".pdf".into(),
                "PDF".into()
            ])),
            r#"<ext:pdf|endwith:".tar.gz">"#
        );
        assert_eq!(t(Cond::Ext(vec!["tar.gz".into()])), r#"endwith:".tar.gz""#);
        assert_eq!(everything_term(&Cond::Ext(vec![" ".into()])), None);
        // Every category's own test agrees with the term: a file it matches has an extension
        // the term lists (or is a folder).
        for cat in rules::CLASSIFY_ORDER {
            let term = t(Cond::Type(vec![*cat]));
            for ext in rules::category_exts(*cat) {
                assert!(
                    term.contains(ext.trim_start_matches('.')),
                    "{cat:?} {ext} in {term}"
                );
            }
        }
        // A condition Everything cannot test asks for more results.
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        assert_eq!(spec.everything_max(), spec.limit);
        spec.all_of.push(Cond::IdleDays { min: 30 });
        assert_eq!(spec.everything_max(), SmartSpec::NAME_FILTER_ASK);
    }

    /// The user's own Everything text narrows the search: the folder, the conditions and the
    /// time window go with it, and the name and conditions are tested on the answer as always.
    #[test]
    fn typed_everything_text_keeps_the_conditions_and_the_window() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.everything = Some("  invoice|receipt ".into());
        spec.all_of.push(Cond::Ext(vec!["pdf".into()]));
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Users\pf\Downloads\" invoice|receipt file: ext:pdf <dm:>=2026-10-31|dc:>=2026-10-31>"#
        );
        assert!(!spec.works_without_everything());
        spec.name = Some("2026".into());
        let now = 1_800_000_000;
        let f = |n: &str| ItemFacts {
            file_name: n.into(),
            ..Default::default()
        };
        assert!(spec.matches(&f("invoice 2026.pdf"), now, now));
        assert!(!spec.matches(&f("invoice 2025.pdf"), now, now), "name");
        assert!(
            !spec.matches(&f("invoice 2026.docx"), now, now),
            "condition"
        );
        assert!(
            !spec.matches(&f("invoice 2026.pdf"), now - 8 * 86_400, now),
            "time window"
        );
    }

    #[test]
    fn the_window_and_conditions_decide_a_match() {
        let spec = Template::Screenshots.spec(&folders()).unwrap();
        let now = 1_800_000_000;
        let png = ItemFacts {
            file_name: "Screenshot 2026-11-07.png".into(),
            ..Default::default()
        };
        assert!(spec.matches(&png, now - 3 * 86_400, now));
        assert!(!spec.matches(&png, now - 8 * 86_400, now));
        let txt = ItemFacts {
            file_name: "notes.txt".into(),
            ..Default::default()
        };
        assert!(!spec.matches(&txt, now, now));
        let partial = ItemFacts {
            file_name: "setup.exe.crdownload".into(),
            ..Default::default()
        };
        assert!(
            !Template::Downloads
                .spec(&folders())
                .unwrap()
                .matches(&partial, now, now)
        );
    }

    #[test]
    fn old_configs_and_minimal_json_read() {
        let spec: SmartSpec = serde_json::from_str(r#"{"folder":"D:\\x"}"#).unwrap();
        assert!(spec.subfolders);
        assert_eq!(spec.limit, SmartSpec::DEFAULT_LIMIT);
        assert_eq!(spec.folders, vec![r"D:\x".to_string()]);
        assert_eq!(spec.matching, SmartMatch::All);
        assert!(spec.works_without_everything());
        let back: SmartSpec = serde_json::from_value(serde_json::to_value(&spec).unwrap()).unwrap();
        assert_eq!(back, spec);
        // `all` is not written: a search without `match` reads and writes back the same.
        let json = serde_json::to_value(&spec).unwrap();
        assert!(json.get("match").is_none(), "{json}");
        assert_eq!(json["folders"], serde_json::json!([r"D:\x"]));
        let every: SmartSpec = serde_json::from_str(r#"{"folder":null}"#).unwrap();
        assert!(every.folders.is_empty());
        let any: SmartSpec =
            serde_json::from_str(r#"{"folders":["D:\\a","E:\\b"],"match":"any"}"#).unwrap();
        assert_eq!(any.matching, SmartMatch::Any);
        assert_eq!(serde_json::to_value(&any).unwrap()["match"], "any");
        assert_eq!(any.folders.len(), 2);
    }

    /// Several folders are searched together, with or without their subfolders; the
    /// conditions go after them as before.
    #[test]
    fn several_folders_are_one_search() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.folders.push(r"D:\Inbox\".into());
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#"<"C:\Users\pf\Downloads\"|"D:\Inbox\"> file: <dm:>=2026-10-31|dc:>=2026-10-31>"#
        );
        spec.subfolders = false;
        assert!(
            spec.everything_query(None)
                .starts_with(r#"<parent:"C:\Users\pf\Downloads"|parent:"D:\Inbox">"#),
            "{}",
            spec.everything_query(None)
        );
    }

    /// 任一: the folders always apply, one of the other rows is enough. Everything gets the
    /// rows OR'ed, each grouped on its own; a row only the app can test leaves Everything the
    /// folders alone and the app decides.
    #[test]
    fn any_needs_one_row_and_keeps_the_folders() {
        let mut spec = SmartSpec {
            template: None,
            folders: vec![r"C:\Docs".into()],
            subfolders: true,
            matching: SmartMatch::Any,
            all_of: vec![
                Cond::Ext(vec!["pdf".into(), "docx".into()]),
                Cond::Name {
                    op: StrOp::Contains,
                    value: "发票".into(),
                },
            ],
            within_days: None,
            name: None,
            everything: None,
            limit: 200,
        };
        assert_eq!(
            spec.everything_query(None),
            r#""C:\Docs\" <<ext:pdf;docx>|<"发票">>"#
        );
        assert!(!spec.app_checks());
        let now = 1_800_000_000;
        let f = |n: &str| ItemFacts {
            file_name: n.into(),
            ..Default::default()
        };
        assert!(spec.matches(&f("report.pdf"), now, now));
        assert!(spec.matches(&f("发票 2026.png"), now, now));
        assert!(!spec.matches(&f("photo.png"), now, now));
        assert!(
            !spec.matches(&f("setup.pdf.crdownload"), now, now),
            "temporary"
        );
        // The time window is one row like the others.
        spec.within_days = Some(7);
        assert!(spec.matches(&f("photo.png"), now - 86_400, now));
        assert!(!spec.matches(&f("photo.png"), now - 8 * 86_400, now));
        assert_eq!(
            spec.everything_query(Some("2026-10-31")),
            r#""C:\Docs\" <<ext:pdf;docx>|<"发票">|<<dm:>=2026-10-31|dc:>=2026-10-31>>>"#
        );
        // A row only the app tests: Everything is asked for the folder, the app decides.
        spec.all_of.push(Cond::IdleDays { min: 30 });
        assert_eq!(spec.everything_query(Some("2026-10-31")), r#""C:\Docs\""#);
        assert!(spec.app_checks());
        assert_eq!(spec.everything_max(), SmartSpec::NAME_FILTER_ASK);
        // No row at all: everything in the folders.
        spec.all_of.clear();
        spec.within_days = None;
        assert!(spec.matches(&f("photo.png"), now, now));
        // A typed Everything text: Everything's answer stands.
        spec.everything = Some("invoice".into());
        spec.all_of.push(Cond::Ext(vec!["pdf".into()]));
        assert!(spec.matches(&f("invoice.png"), now, now));
        assert_eq!(
            spec.everything_query(None),
            r#""C:\Docs\" <<invoice>|<ext:pdf>>"#
        );
    }

    #[test]
    fn a_search_with_origin_cannot_be_saved() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        assert_eq!(spec.problem(), None);
        spec.all_of.push(Cond::Origin(crate::Origin::PublicDesktop));
        assert!(spec.problem().is_some());
    }

    #[test]
    fn the_name_filter_matches_like_the_peek_search() {
        let mut spec = Template::Downloads.spec(&folders()).unwrap();
        spec.name = Some("bg".into());
        let now = 1_800_000_000;
        let f = |n: &str| ItemFacts {
            file_name: n.into(),
            ..Default::default()
        };
        assert!(
            spec.matches(&f("季度报告.pdf"), now, now),
            "pinyin initials"
        );
        assert!(!spec.matches(&f("旧合同.pdf"), now, now));
        spec.name = Some("pdf".into());
        assert!(spec.matches(&f("旧合同.pdf"), now, now));
        assert!(
            !spec.matches(&f("photo_d_final.png"), now, now),
            "loose match left out"
        );
        assert_eq!(spec.everything_max(), SmartSpec::NAME_FILTER_ASK);
    }

    #[test]
    fn dates_are_iso() {
        assert_eq!(
            everything_date(crate::CivilDate::new(2026, 3, 5)),
            "2026-03-05"
        );
    }
}
