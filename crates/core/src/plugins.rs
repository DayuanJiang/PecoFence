//! 社区插件: plugins from the DayuanJiang/PecoFence-plugins repository. This module reads and
//! checks the repository's `index.json` and a plugin's `plugin.json`, builds the addresses of
//! a plugin's files (from the reviewed commit, never from the index) and says what an update
//! may do. It touches neither the network nor the disk.

use serde::Deserialize;
use std::collections::BTreeMap;

pub const REPO: &str = "DayuanJiang/PecoFence-plugins";
/// The files a plugin may have, with the most bytes each may have.
pub const FILES: [(&str, u64); 5] = [
    ("plugin.json", 64 * 1024),
    ("main.js", 200 * 1024),
    ("index.html", 200 * 1024),
    ("README.md", 64 * 1024),
    ("screenshot.png", 300 * 1024),
];
/// The files an install downloads, those of them the plugin has (this version shows no
/// screenshot).
pub const INSTALL_FILES: [&str; 4] = ["plugin.json", "main.js", "index.html", "README.md"];

/// The files a plugin of `layer` cannot do without: a light plugin runs `main.js`, a web
/// plugin shows `index.html` (its `main.js` is optional).
pub fn needed_files(layer: &str) -> [&'static str; 3] {
    let main = if layer == "web" {
        "index.html"
    } else {
        "main.js"
    };
    ["plugin.json", main, "README.md"]
}
/// The newest plugin interface this PecoFence knows (`api` in plugin.json).
pub const API: u32 = 1;
/// The most bytes `index.json` may have.
pub const MAX_INDEX: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Clipboard {
    #[default]
    None,
    /// Only puts things on the clipboard.
    Write,
    Read,
    Readwrite,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Files {
    #[default]
    #[serde(rename = "data")]
    Data,
    #[serde(rename = "data+read")]
    DataRead,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Permissions {
    pub network: Vec<String>,
    /// More hosts `pf.open` may open in the browser.
    pub links: Vec<String>,
    pub clipboard: Clipboard,
    pub files: Files,
    pub run: bool,
    /// Keeps running while its fence is hidden (a web plugin's page is not suspended).
    pub background: bool,
    /// Programs `pf.run` may start.
    pub system: Vec<String>,
    pub folders: bool,
    pub notify: bool,
}

/// A plugin's `plugin.json`, and the same fields in its index entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub name: BTreeMap<String, String>,
    #[serde(default)]
    pub description: BTreeMap<String, String>,
    pub author: String,
    #[serde(rename = "minPecoFence")]
    pub min_pecofence: String,
    #[serde(default)]
    pub streamable: bool,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default = "first_api")]
    pub api: u32,
    /// `light`: main.js in PecoFence's QuickJS, which draws the plugin's view; `web`: the
    /// plugin's own page (`index.html`) shown in the fence.
    #[serde(default = "light")]
    pub layer: String,
    #[serde(default)]
    pub settings: Vec<Setting>,
}

fn first_api() -> u32 {
    1
}

fn light() -> String {
    "light".into()
}

/// One setting a plugin fence has (栅栏菜单里的插件设置): `switch`, `text`, `number` or `choice`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Setting {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub label: BTreeMap<String, String>,
    #[serde(default)]
    pub default: serde_json::Value,
}

/// A plugin fence's settings as its plugin sees them (`pf.settings`): what the fence saved
/// when it is of the setting's kind, else the setting's default.
pub fn fence_settings(
    m: &Manifest,
    saved: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let value = |s: &Setting| {
        let kept = saved.get(&s.key).filter(|v| match s.kind.as_str() {
            "switch" => v.is_boolean(),
            "number" => v.is_number(),
            _ => v.is_string(),
        });
        (s.key.clone(), kept.unwrap_or(&s.default).clone())
    };
    serde_json::Value::Object(m.settings.iter().map(value).collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

/// One plugin of the index: its manifest, its files and, when it was taken down, why.
#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    #[serde(flatten)]
    pub manifest: Manifest,
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub disabled: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Index {
    pub commit: String,
    pub plugins: Vec<Entry>,
}

impl Index {
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.plugins.iter().find(|e| e.manifest.id == id)
    }
}

fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes)
}

pub fn valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    (3..=40).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn valid_host(host: &str) -> bool {
    (1..=253).contains(&host.len())
        && host
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-')
}

/// The checks every manifest passes, in `plugin.json` and in the index.
pub fn check_manifest(m: &Manifest) -> Result<(), String> {
    let short = |texts: &BTreeMap<String, String>, max: usize| {
        texts.values().all(|t| t.chars().count() <= max)
    };
    if !valid_id(&m.id) {
        return Err(format!("bad id {:?}", m.id));
    }
    if crate::updates::parse_version(&m.version).is_none()
        || crate::updates::parse_version(&m.min_pecofence).is_none()
    {
        return Err(format!("{}: bad version or minPecoFence", m.id));
    }
    if m.author.trim().is_empty() || m.author.chars().count() > 64 {
        return Err(format!("{}: bad author", m.id));
    }
    if m.name.get("en").is_none_or(|n| n.trim().is_empty())
        || !short(&m.name, 60)
        || !short(&m.description, 300)
    {
        return Err(format!("{}: name or description missing or too long", m.id));
    }
    if m.api == 0 || m.api > API || !matches!(m.layer.as_str(), "light" | "web") {
        return Err(format!(
            "{}: api {} or layer {:?} not known",
            m.id, m.api, m.layer
        ));
    }
    let kinds = ["switch", "text", "number", "choice"];
    let good = |s: &Setting| {
        (1..=32).contains(&s.key.len())
            && s.key.bytes().all(|c| c.is_ascii_alphanumeric())
            && kinds.contains(&s.kind.as_str())
            && short(&s.label, 60)
    };
    if m.settings.len() > 20 || !m.settings.iter().all(good) {
        return Err(format!("{}: bad settings", m.id));
    }
    let hosts = &m.permissions.network;
    if hosts.len() > 20 || !hosts.iter().all(|h| valid_host(h)) {
        return Err(format!("{}: bad network hosts", m.id));
    }
    Ok(())
}

/// An index entry's checks: its manifest's and its files' (only the allowed names, each once,
/// within its size, with a SHA-256; the three its layer needs are there).
pub fn check_entry(e: &Entry) -> Result<(), String> {
    check_manifest(&e.manifest)?;
    let id = &e.manifest.id;
    for (i, f) in e.files.iter().enumerate() {
        let Some((_, max)) = FILES.iter().find(|(name, _)| *name == f.name) else {
            return Err(format!("{id}: file {:?} not allowed", f.name));
        };
        if f.size > *max || !is_hex(&f.sha256, 64) {
            return Err(format!("{id}: {} too big or without a SHA-256", f.name));
        }
        if e.files[..i].iter().any(|g| g.name == f.name) {
            return Err(format!("{id}: {} listed twice", f.name));
        }
    }
    if let Some(missing) = needed_files(&e.manifest.layer)
        .iter()
        .find(|n| !e.files.iter().any(|f| f.name == **n))
    {
        return Err(format!("{id}: {missing} missing"));
    }
    Ok(())
}

/// Reads `index.json`: the commit and the entries that pass [`check_entry`] (an entry that does
/// not, or a second one with the same id, is left out and named in the second list).
pub fn parse_index(bytes: &[u8]) -> Result<(Index, Vec<String>), String> {
    #[derive(Deserialize)]
    struct Raw {
        format: u32,
        commit: String,
        plugins: Vec<serde_json::Value>,
    }
    if bytes.len() > MAX_INDEX {
        return Err("index too big".into());
    }
    let raw: Raw = serde_json::from_slice(strip_bom(bytes)).map_err(|e| e.to_string())?;
    if raw.format != 1 {
        return Err(format!("index format {} not known", raw.format));
    }
    if !is_hex(&raw.commit, 40) {
        return Err("index commit is not 40 hex digits".into());
    }
    let mut index = Index {
        commit: raw.commit,
        plugins: Vec::new(),
    };
    let mut dropped = Vec::new();
    for value in raw.plugins {
        let entry = serde_json::from_value::<Entry>(value)
            .map_err(|e| e.to_string())
            .and_then(|e| check_entry(&e).map(|_| e));
        match entry {
            Ok(e) if index.get(&e.manifest.id).is_some() => {
                dropped.push(format!("{}: listed twice", e.manifest.id));
            }
            Ok(e) => index.plugins.push(e),
            Err(why) => dropped.push(why),
        }
    }
    Ok((index, dropped))
}

/// Reads an installed or downloaded `plugin.json`.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let m: Manifest = serde_json::from_slice(strip_bom(bytes)).map_err(|e| e.to_string())?;
    check_manifest(&m)?;
    Ok(m)
}

/// The index: from GitHub first, then from jsDelivr.
pub fn index_urls() -> [String; 2] {
    [
        format!("https://raw.githubusercontent.com/{REPO}/index/index.json"),
        format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index.json"),
    ]
}

/// A plugin file at the reviewed commit: from GitHub first, then from jsDelivr.
pub fn file_urls(commit: &str, id: &str, name: &str) -> [String; 2] {
    [
        format!("https://raw.githubusercontent.com/{REPO}/{commit}/plugins/{id}/{name}"),
        format!("https://cdn.jsdelivr.net/gh/{REPO}@{commit}/plugins/{id}/{name}"),
    ]
}

/// A plugin file on a test server whose index is at `index_url`: `<origin>/<commit>/plugins/...`.
pub fn test_file_url(index_url: &str, commit: &str, id: &str, name: &str) -> String {
    let origin = index_url.split('/').take(3).collect::<Vec<_>>().join("/");
    format!("{origin}/{commit}/plugins/{id}/{name}")
}

/// The plugin's page on GitHub (its README at the reviewed commit).
pub fn page_url(commit: &str, id: &str) -> String {
    format!("https://github.com/{REPO}/blob/{commit}/plugins/{id}/README.md")
}

/// Whether `new` may do more than `old`: a new network host, more clipboard or file access,
/// rows that start programs, or a script that keeps running.
pub fn permissions_grow(old: &Manifest, new: &Manifest) -> bool {
    let (o, n) = (&old.permissions, &new.permissions);
    n.network
        .iter()
        .any(|h| !o.network.iter().any(|g| g.eq_ignore_ascii_case(h)))
        || n.clipboard > o.clipboard
        || n.files > o.files
        || (n.run && !o.run)
        || (new.streamable && !old.streamable)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Update {
    /// Nothing newer.
    None,
    /// Newer and asks for nothing more: installed without asking.
    Auto,
    /// Newer and asks for more: waits for the user.
    Confirm,
    /// Newer but needs a newer PecoFence.
    NeedsNewerApp,
}

/// What to do with the index's `entry` when `installed` is on this computer and PecoFence is
/// version `app`.
pub fn update_kind(installed: &Manifest, entry: &Manifest, app: &str) -> Update {
    if !crate::updates::is_newer(&entry.version, &installed.version) {
        Update::None
    } else if crate::updates::is_newer(&entry.min_pecofence, app) {
        Update::NeedsNewerApp
    } else if permissions_grow(installed, entry) {
        Update::Confirm
    } else {
        Update::Auto
    }
}

/// The text for UI language `lang` ("zh-CN"): that language, another one of the same family,
/// English, or any.
pub fn pick<'a>(texts: &'a BTreeMap<String, String>, lang: &str) -> &'a str {
    let family = lang.split('-').next().unwrap_or(lang);
    texts
        .get(lang)
        .or_else(|| {
            texts
                .iter()
                .find(|(k, _)| k.split('-').next() == Some(family))
                .map(|(_, v)| v)
        })
        .or_else(|| texts.get("en"))
        .or_else(|| texts.values().next())
        .map_or("", String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "db47f4c49907d313d7588e63a07e8b1ed1b6331993499aeae4e578e4a338b488";
    const COMMIT: &str = "a89e80aa191bc052eb10b7ab0c0088f9ffa92a84";

    fn entry(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id, "version": "1.0.1",
            "name": { "en": "Countdown", "zh-CN": "倒数日" },
            "description": { "en": "Days left." },
            "author": "DayuanJiang", "minPecoFence": "0.4.0", "streamable": false,
            "permissions": { "network": [], "clipboard": "none", "files": "data", "run": false },
            "files": [
                { "name": "main.js", "size": 2745, "sha256": HASH, "url": "https://evil.example/x" },
                { "name": "plugin.json", "size": 453, "sha256": HASH },
                { "name": "README.md", "size": 900, "sha256": HASH }
            ]
        })
    }

    fn index(plugins: Vec<serde_json::Value>) -> Vec<u8> {
        serde_json::to_vec(
            &serde_json::json!({ "format": 1, "commit": COMMIT, "plugins": plugins }),
        )
        .unwrap()
    }

    fn manifest(f: impl FnOnce(&mut serde_json::Value)) -> Manifest {
        let mut v = entry("countdown");
        f(&mut v);
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn index_keeps_good_entries_and_drops_bad_ones() {
        let bad: [(&str, fn(&mut serde_json::Value)); 9] = [
            ("bad-file", |v| v["files"][0]["name"] = "run.exe".into()),
            ("too-big", |v| {
                v["files"][0]["size"] = (200 * 1024 + 1).into()
            }),
            ("short-hash", |v| v["files"][1]["sha256"] = "abc".into()),
            ("no-readme", |v| {
                _ = v["files"].as_array_mut().unwrap().pop()
            }),
            ("twice-file", |v| v["files"][2]["name"] = "main.js".into()),
            ("bad-clip", |v| v["permissions"]["clipboard"] = "all".into()),
            ("bad-host", |v| {
                v["permissions"]["network"] = serde_json::json!(["a.com/x"])
            }),
            ("no-english", |v| {
                v["name"] = serde_json::json!({ "zh-CN": "倒数日" })
            }),
            ("Bad_Id", |_| {}),
        ];
        let mut plugins = vec![entry("countdown"), entry("countdown")];
        for (id, spoil) in bad {
            plugins.push(entry(id));
            spoil(plugins.last_mut().unwrap());
        }
        plugins.push(entry("taken-down"));
        plugins.last_mut().unwrap()["disabled"] = "Sends data it does not declare".into();
        let bytes = [b"\xEF\xBB\xBF".to_vec(), index(plugins)].concat();
        let (index, dropped) = parse_index(&bytes).unwrap();
        let ids: Vec<_> = index.plugins.iter().map(|e| &e.manifest.id).collect();
        assert_eq!(ids, ["countdown", "taken-down"]);
        assert_eq!(dropped.len(), 10, "{dropped:?}");
        assert_eq!(index.commit, COMMIT);
        let why = index.get("taken-down").unwrap().disabled.as_deref();
        assert_eq!(why, Some("Sends data it does not declare"));
    }

    #[test]
    fn a_web_plugin_needs_its_page_and_may_have_main_js() {
        let check = |files: &[&str]| {
            let mut v = entry("webdemo");
            v["layer"] = "web".into();
            v["files"] = files
                .iter()
                .map(|n| serde_json::json!({ "name": n, "size": 10, "sha256": HASH }))
                .collect();
            check_entry(&serde_json::from_value(v).unwrap())
        };
        assert!(check(&["plugin.json", "index.html", "README.md"]).is_ok());
        assert!(check(&["plugin.json", "index.html", "main.js", "README.md"]).is_ok());
        assert!(check(&["plugin.json", "main.js", "README.md"]).is_err());
    }

    #[test]
    fn index_needs_format_1_and_a_full_commit() {
        let ok = index(vec![]);
        assert!(parse_index(&ok).is_ok());
        let text = String::from_utf8(ok).unwrap();
        assert!(parse_index(text.replace("\"format\":1", "\"format\":2").as_bytes()).is_err());
        assert!(parse_index(text.replace(COMMIT, "index").as_bytes()).is_err());
        assert!(parse_index(text.replace(COMMIT, &COMMIT.to_uppercase()).as_bytes()).is_err());
        assert!(parse_index(b"not json").is_err());
        assert!(parse_index(&vec![b' '; MAX_INDEX + 1]).is_err());
    }

    #[test]
    fn ids_follow_the_repository_rule() {
        for ok in ["abc", "clipboard-history", "0day", &"a".repeat(40)] {
            assert!(valid_id(ok), "{ok}");
        }
        for bad in [
            "ab",
            "-abc",
            "Abc",
            "a_b",
            "a/b",
            "..",
            "a b",
            &"a".repeat(41),
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
    }

    #[test]
    fn urls_come_from_commit_id_and_name() {
        let [raw, cdn] = file_urls(COMMIT, "countdown", "main.js");
        let path = format!("{COMMIT}/plugins/countdown/main.js");
        assert_eq!(
            raw,
            format!("https://raw.githubusercontent.com/{REPO}/{path}")
        );
        assert_eq!(cdn, format!("https://cdn.jsdelivr.net/gh/{REPO}@{path}"));
        let [raw, cdn] = index_urls();
        assert_eq!(
            raw,
            format!("https://raw.githubusercontent.com/{REPO}/index/index.json")
        );
        assert_eq!(
            cdn,
            format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index.json")
        );
        let test = test_file_url(
            "http://127.0.0.1:8731/index.json",
            COMMIT,
            "countdown",
            "main.js",
        );
        assert_eq!(test, format!("http://127.0.0.1:8731/{path}"));
        let page = page_url(COMMIT, "countdown");
        assert_eq!(
            page,
            format!("https://github.com/{REPO}/blob/{COMMIT}/plugins/countdown/README.md")
        );
    }

    #[test]
    fn permission_growth() {
        let old = manifest(|v| v["permissions"]["network"] = serde_json::json!(["api.github.com"]));
        assert!(!permissions_grow(&old, &old));
        let fewer = manifest(|_| {});
        assert!(!permissions_grow(&old, &fewer));
        let same_host =
            manifest(|v| v["permissions"]["network"] = serde_json::json!(["API.github.com"]));
        assert!(!permissions_grow(&old, &same_host));
        let grows: [fn(&mut serde_json::Value); 6] = [
            |v| v["permissions"]["network"] = serde_json::json!(["api.github.com", "x.com"]),
            |v| v["permissions"]["clipboard"] = "read".into(),
            |v| v["permissions"]["files"] = "data+read".into(),
            |v| v["permissions"]["run"] = true.into(),
            |v| v["streamable"] = true.into(),
            |v| {
                v["permissions"]["network"] = serde_json::json!(["api.github.com"]);
                v["permissions"]["clipboard"] = "readwrite".into();
            },
        ];
        for grow in grows {
            let mut v = entry("countdown");
            v["permissions"]["network"] = serde_json::json!(["api.github.com"]);
            grow(&mut v);
            let new: Manifest = serde_json::from_value(v).unwrap();
            assert!(permissions_grow(&old, &new), "{new:?}");
        }
        let read = manifest(|v| v["permissions"]["clipboard"] = "readwrite".into());
        let less = manifest(|v| v["permissions"]["clipboard"] = "read".into());
        assert!(!permissions_grow(&read, &less));
    }

    #[test]
    fn update_kinds() {
        let installed = manifest(|_| {});
        let newer = manifest(|v| v["version"] = "1.0.2".into());
        assert_eq!(update_kind(&installed, &installed, "0.4.0"), Update::None);
        assert_eq!(update_kind(&newer, &installed, "0.4.0"), Update::None);
        assert_eq!(update_kind(&installed, &newer, "0.4.0"), Update::Auto);
        let needs = manifest(|v| {
            v["version"] = "1.1.0".into();
            v["minPecoFence"] = "0.5.0".into();
        });
        let kind = update_kind(&installed, &needs, "0.4.0");
        assert_eq!(kind, Update::NeedsNewerApp);
        assert_eq!(update_kind(&installed, &needs, "0.5.0"), Update::Auto);
        let more = manifest(|v| {
            v["version"] = "2.0.0".into();
            v["permissions"]["run"] = true.into();
        });
        assert_eq!(update_kind(&installed, &more, "0.4.0"), Update::Confirm);
    }

    #[test]
    fn settings_are_checked_and_saved_values_keep_their_kind() {
        let m = manifest(|v| {
            v["settings"] = serde_json::json!([
                { "key": "seconds", "type": "switch", "label": { "en": "Seconds" }, "default": false },
                { "key": "city", "type": "text", "default": "" }
            ])
        });
        assert!(check_manifest(&m).is_ok());
        let saved = serde_json::json!({ "seconds": true, "city": 3, "gone": 1 });
        let settings = fence_settings(&m, saved.as_object().unwrap());
        assert_eq!(settings, serde_json::json!({ "seconds": true, "city": "" }));
        for bad in [
            serde_json::json!({ "settings": [{ "key": "a b", "type": "switch" }] }),
            serde_json::json!({ "settings": [{ "key": "a", "type": "file" }] }),
            serde_json::json!({ "api": 2 }),
            serde_json::json!({ "layer": "native" }),
        ] {
            let m = manifest(|v| {
                v.as_object_mut()
                    .unwrap()
                    .extend(bad.as_object().unwrap().clone())
            });
            assert!(check_manifest(&m).is_err(), "{bad}");
        }
    }

    #[test]
    fn manifest_reads_with_a_bom_and_is_checked() {
        let text = serde_json::to_vec(&entry("countdown")).unwrap();
        let bom = [b"\xEF\xBB\xBF".to_vec(), text.clone()].concat();
        assert_eq!(parse_manifest(&bom).unwrap().id, "countdown");
        let bad = String::from_utf8(text).unwrap().replace("1.0.1", "one");
        assert!(parse_manifest(bad.as_bytes()).is_err());
    }

    #[test]
    fn texts_by_language() {
        let m = manifest(|v| v["name"]["zh-TW"] = "倒數日".into());
        assert_eq!(pick(&m.name, "zh-CN"), "倒数日");
        assert_eq!(pick(&m.name, "zh-TW"), "倒數日");
        assert_eq!(pick(&m.name, "ja"), "Countdown");
        let only_cn =
            manifest(|v| v["name"] = serde_json::json!({ "en": "Countdown", "zh-CN": "倒数日" }));
        assert_eq!(pick(&only_cn.name, "zh-TW"), "倒数日");
        assert_eq!(pick(&BTreeMap::new(), "en"), "");
    }
}
