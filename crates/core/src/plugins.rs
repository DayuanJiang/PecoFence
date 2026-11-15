//! 插件: plugins from the DayuanJiang/PecoFence-plugins repository. This module reads and
//! checks the repository's signed `index.json` and a plugin's `plugin.json`, builds the
//! addresses of a plugin's files (from the reviewed commit, never from the index) and says what
//! an update may do. It touches neither the network nor the disk.

use serde::Deserialize;
use std::collections::BTreeMap;

pub const REPO: &str = "DayuanJiang/PecoFence-plugins";
/// The project website's copy of the index and the files: the third address, for networks that
/// reach neither GitHub nor jsDelivr.
pub const SITE: &str = "https://pecofence.jiang.jp/plugins";
/// The Ed25519 key `index.json.sig` is checked with. A development key: the release key
/// (`tools/make-key.py` in the plugin repository; its private half is the repository secret
/// INDEX_SIGNING_KEY) replaces it before a release offers plugins.
pub const INDEX_KEY: [u8; 32] = [
    0x83, 0xf3, 0x36, 0xdd, 0xc5, 0x6e, 0xa4, 0x18, 0x92, 0x26, 0xc1, 0xb1, 0x0f, 0x01, 0x48, 0x48,
    0xa0, 0xd8, 0x3f, 0xb0, 0x8f, 0xd7, 0x23, 0xb7, 0x41, 0x0b, 0x9f, 0xc3, 0xb8, 0xcd, 0x4b, 0x81,
];
/// The files whose names are fixed, with the most bytes each may have. A light plugin has only
/// these; a web plugin also has `index.html` and the files it uses (sub-folders allowed), each
/// up to `OTHER_SIZE`.
pub const FILES: [(&str, u64); 4] = [
    ("plugin.json", 64 * 1024),
    ("main.js", 200 * 1024),
    ("README.md", 64 * 1024),
    ("screenshot.png", 300 * 1024),
];
pub const OTHER_SIZE: u64 = 1024 * 1024;
/// A plugin's most files, and most bytes together.
pub const MAX_FILES: usize = 100;
pub const MAX_TOTAL: u64 = 5 * 1024 * 1024;

/// The newest plugin interface this PecoFence knows (`api` in plugin.json).
pub const API: u32 = 1;
/// The most bytes `index.json` may have.
pub const MAX_INDEX: usize = 1024 * 1024;

/// What a plugin may do with the clipboard: reading and writing are two separate rights.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Clipboard {
    #[default]
    None,
    /// Only puts things on the clipboard.
    Write,
    Read,
    Readwrite,
}

impl Clipboard {
    /// `read` or `readwrite`: the plugin gets what is copied.
    pub fn reads(self) -> bool {
        matches!(self, Self::Read | Self::Readwrite)
    }
    /// `write` or `readwrite`: the plugin may put things on the clipboard.
    pub fn writes(self) -> bool {
        matches!(self, Self::Write | Self::Readwrite)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Permissions {
    /// The hosts `pf.http` may reach.
    pub network: Vec<String>,
    /// More hosts `pf.open` may open in the browser (those of `network` too).
    pub links: Vec<String>,
    pub clipboard: Clipboard,
    /// Keeps running while its fence is hidden (a web plugin's page is not suspended).
    pub background: bool,
    /// The programs `pf.run` may start (a full path or a name in System32); `pf.run` works only
    /// while 允许插件使用 Windows 底层功能 is on.
    pub system: Vec<String>,
    /// Reads a folder the user picks.
    pub folders: bool,
    /// Shows notices and plays their sound.
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

impl Manifest {
    /// The file the plugin starts from: a light plugin runs `main.js`, a web plugin shows
    /// `index.html` (its `main.js` is optional).
    pub fn main_file(&self) -> &'static str {
        if self.layer == "web" {
            "index.html"
        } else {
            "main.js"
        }
    }
}

/// One setting of a plugin fence (插件设置): `switch`, `text`, `number` or `choice`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Setting {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub label: BTreeMap<String, String>,
    #[serde(default)]
    pub default: serde_json::Value,
    /// A `choice`'s values, each with its label.
    #[serde(default)]
    pub options: Vec<Choice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Choice {
    pub value: String,
    #[serde(default)]
    pub label: BTreeMap<String, String>,
}

/// Whether `v` is a value of setting `s`: true or false, a number, one of the choices, or a text
/// of at most 4,000 characters.
pub fn fits(s: &Setting, v: &serde_json::Value) -> bool {
    match s.kind.as_str() {
        "switch" => v.is_boolean(),
        "number" => v.is_number(),
        "choice" => s.options.iter().any(|o| v.as_str() == Some(&o.value)),
        _ => v.as_str().is_some_and(|t| t.chars().count() <= 4000),
    }
}

/// A plugin fence's settings as its plugin sees them (`pf.settings`): what the fence saved
/// when it fits the setting, else the setting's default.
pub fn fence_settings(
    m: &Manifest,
    saved: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let value = |s: &Setting| {
        let kept = saved.get(&s.key).filter(|v| fits(s, v));
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
    /// Grows with every change of the plugin repository (its count of commits); an index with a
    /// lower one than the index kept is an old copy served again. None in an index: 0.
    pub serial: u64,
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

/// A host name in lower case with a dot, whose last part starts with a letter: no IP address,
/// no `localhost`, no name of the local network.
pub fn valid_host(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    let part_ok = |p: &&str| {
        (1..=63).contains(&p.len())
            && !p.starts_with('-')
            && !p.ends_with('-')
            && p.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    };
    let last = parts.last().copied().unwrap_or("");
    host.len() <= 253
        && parts.len() >= 2
        && parts.iter().all(part_ok)
        && last.len() >= 2
        && last.as_bytes()[0].is_ascii_lowercase()
}

/// A program of `system`: a full path (`C:\…\x.exe`) or the name of a program in System32.
fn valid_program(p: &str) -> bool {
    let b = p.as_bytes();
    let exe = (5..=260).contains(&p.len()) && p.to_ascii_lowercase().ends_with(".exe");
    let name = p
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c));
    let path = b.len() > 3
        && b[0].is_ascii_alphabetic()
        && &b[1..3] == b":\\"
        && !b[3..].iter().any(|c| b"/:*?\"<>|".contains(c) || *c < 32)
        && !p.split('\\').any(|s| s.is_empty() || s == "..");
    exe && (name || path)
}

/// `pf.theme` of a fence: `dark` where it draws light text (a dark background), else `light`.
pub fn theme(light_text: bool) -> &'static str {
    if light_text { "dark" } else { "light" }
}

/// A plain name: letters, digits, `.`, `_` and `-`, at most 64, not starting or ending with a
/// dot, and none of Windows' device names (`con`, `nul`, `com1`, ... with any extension: Windows
/// would open the device). The names `pf.data` keeps files under and pictures of the data folder
/// are such names.
pub fn valid_plain_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let numbered = |p: &str| {
        stem.len() == 4 && stem.starts_with(p) && (b'1'..=b'9').contains(&stem.as_bytes()[3])
    };
    let device =
        ["con", "prn", "aux", "nul"].contains(&stem.as_str()) || numbered("com") || numbered("lpt");
    (1..=64).contains(&name.len())
        && !name.starts_with('.')
        && !name.ends_with('.')
        && !device
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

/// A file's name inside the plugin folder: plain names ([`valid_plain_name`]) joined by `/`. So
/// no `..`, no leading `/`, no backslash and no drive letter.
pub fn valid_file_name(name: &str) -> bool {
    name.len() <= 200 && name.split('/').all(valid_plain_name)
}

/// The checks every manifest passes, in `plugin.json` and in the index. An `api` newer than
/// this PecoFence's passes: its row says 需要更新 PecoFence ([`needs_newer_app`]).
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
    if m.api == 0 || !matches!(m.layer.as_str(), "light" | "web") {
        return Err(format!(
            "{}: api {} or layer {:?} not known",
            m.id, m.api, m.layer
        ));
    }
    let kinds = ["switch", "text", "number", "choice"];
    let good = |(i, s): (usize, &Setting)| {
        (1..=32).contains(&s.key.len())
            && s.key.bytes().all(|c| c.is_ascii_alphanumeric())
            && !m.settings[..i].iter().any(|t| t.key == s.key)
            && kinds.contains(&s.kind.as_str())
            && short(&s.label, 60)
            && (s.kind != "choice" || (1..=50).contains(&s.options.len()))
            && s.options.iter().all(|o| short(&o.label, 60))
            && fits(s, &s.default)
    };
    if m.settings.len() > 20 || !m.settings.iter().enumerate().all(good) {
        return Err(format!("{}: bad settings", m.id));
    }
    let p = &m.permissions;
    for (field, hosts) in [("network", &p.network), ("links", &p.links)] {
        if hosts.len() > 20 || !hosts.iter().all(|h| valid_host(h)) {
            return Err(format!("{}: bad {field} hosts", m.id));
        }
    }
    if p.system.len() > 20 || !p.system.iter().all(|s| valid_program(s)) {
        return Err(format!("{}: bad system programs", m.id));
    }
    Ok(())
}

/// An index entry's checks: its manifest's and its files' (allowed names, each once, within
/// its size, with a SHA-256; the three its layer needs are there; at most `MAX_FILES` and
/// `MAX_TOTAL`).
pub fn check_entry(e: &Entry) -> Result<(), String> {
    check_manifest(&e.manifest)?;
    let (id, web) = (&e.manifest.id, e.manifest.layer == "web");
    let mut total = 0;
    for (i, f) in e.files.iter().enumerate() {
        let fixed = FILES.iter().find(|(name, _)| *name == f.name);
        if !valid_file_name(&f.name) || (fixed.is_none() && !web) {
            return Err(format!("{id}: file {:?} not allowed", f.name));
        }
        if f.size > fixed.map_or(OTHER_SIZE, |(_, max)| *max) || !is_hex(&f.sha256, 64) {
            return Err(format!("{id}: {} too big or without a SHA-256", f.name));
        }
        // Windows does not tell A.js from a.js.
        if e.files[..i]
            .iter()
            .any(|g| g.name.eq_ignore_ascii_case(&f.name))
        {
            return Err(format!("{id}: {} listed twice", f.name));
        }
        total += f.size;
    }
    if e.files.len() > MAX_FILES || total > MAX_TOTAL {
        return Err(format!("{id}: too many files or bytes"));
    }
    if let Some(missing) = ["plugin.json", e.manifest.main_file(), "README.md"]
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
        #[serde(default)]
        serial: u64,
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
        serial: raw.serial,
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

/// Whether `sig` (the text of `index.json.sig`, 128 hex digits) is `key`'s Ed25519 signature
/// of exactly the bytes `index`.
pub fn signed_by(key: &[u8; 32], index: &[u8], sig: &[u8]) -> bool {
    let text = std::str::from_utf8(sig).unwrap_or("").trim();
    if !is_hex(text, 128) {
        return false;
    }
    let mut bytes = [0u8; 64];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap_or(0);
    }
    let Ok(key) = ed25519_dalek::VerifyingKey::from_bytes(key) else {
        return false;
    };
    let sig = ed25519_dalek::Signature::from_bytes(&bytes);
    key.verify_strict(index, &sig).is_ok()
}

/// Reads an installed or downloaded `plugin.json`.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let m: Manifest = serde_json::from_slice(strip_bom(bytes)).map_err(|e| e.to_string())?;
    check_manifest(&m)?;
    Ok(m)
}

/// The index: from GitHub, then jsDelivr, then the project website. Its signature is at the
/// same address with `.sig`.
pub fn index_urls() -> [String; 3] {
    [
        format!("https://raw.githubusercontent.com/{REPO}/index/index.json"),
        format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index.json"),
        format!("{SITE}/index.json"),
    ]
}

/// A plugin file at the reviewed commit, from the same three places.
pub fn file_urls(commit: &str, id: &str, name: &str) -> [String; 3] {
    [
        format!("https://raw.githubusercontent.com/{REPO}/{commit}/plugins/{id}/{name}"),
        format!("https://cdn.jsdelivr.net/gh/{REPO}@{commit}/plugins/{id}/{name}"),
        format!("{SITE}/{commit}/plugins/{id}/{name}"),
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

/// The fields in which `new` asks for more than `old`: a host or program not asked for before
/// (`network`, `links`, `system`; a link that was a network host is not new), reading or
/// writing the clipboard when it did not before, or `background`, `folders`, `notify` turned on.
pub fn more_permissions(old: &Permissions, new: &Permissions) -> Vec<&'static str> {
    let added = |o: &[String], n: &[String]| {
        n.iter()
            .any(|h| !o.iter().any(|g| g.eq_ignore_ascii_case(h)))
    };
    let links = [old.links.as_slice(), old.network.as_slice()].concat();
    [
        ("network", added(&old.network, &new.network)),
        ("links", added(&links, &new.links)),
        ("system", added(&old.system, &new.system)),
        (
            "clipboard",
            (new.clipboard.reads() && !old.clipboard.reads())
                || (new.clipboard.writes() && !old.clipboard.writes()),
        ),
        ("background", new.background && !old.background),
        ("folders", new.folders && !old.folders),
        ("notify", new.notify && !old.notify),
    ]
    .into_iter()
    .filter_map(|(field, more)| more.then_some(field))
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Update {
    /// Nothing newer (a lower version is never taken).
    None,
    /// Newer and asks for nothing more: installed without asking.
    Auto,
    /// Newer and asks for more, or may start programs: waits for the user.
    Confirm,
    /// Newer but needs a newer PecoFence.
    NeedsNewerApp,
}

/// Whether plugin `m` needs a newer PecoFence than version `app`: a newer interface or a
/// higher `minPecoFence`.
pub fn needs_newer_app(m: &Manifest, app: &str) -> bool {
    m.api > API || crate::updates::is_newer(&m.min_pecofence, app)
}

/// What to do with the index's `entry` when `installed` is on this computer and PecoFence is
/// version `app`. A plugin with `system`, or one that changes its layer or its author, always
/// asks.
pub fn update_kind(installed: &Manifest, entry: &Manifest, app: &str) -> Update {
    let (old, new) = (&installed.permissions, &entry.permissions);
    if !crate::updates::is_newer(&entry.version, &installed.version) {
        Update::None
    } else if needs_newer_app(entry, app) {
        Update::NeedsNewerApp
    } else if !new.system.is_empty()
        || installed.layer != entry.layer
        || installed.author != entry.author
        || !more_permissions(old, new).is_empty()
    {
        Update::Confirm
    } else {
        Update::Auto
    }
}

/// What the install warning shows of plugin `m` (its version, layer, author and permissions),
/// as one text: the page sends it back with 安装, and a different one now means the index
/// changed while the warning was open, so the warning is shown again.
pub fn shown_key(m: &Manifest) -> String {
    format!(
        "{} {} {:?} {:?}",
        m.version, m.layer, m.author, m.permissions
    )
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
    use serde_json::json;

    #[test]
    fn plain_names_are_no_device_names_and_end_without_a_dot() {
        for bad in [
            "con",
            "NUL",
            "nul.txt",
            "aux.tar.gz",
            "Com1",
            "lpt9.log",
            "prn.",
            "a.",
            "a..",
            ".a",
        ] {
            assert!(!valid_plain_name(bad), "{bad}");
        }
        for good in [
            "console.js",
            "com10",
            "lpt0",
            "com",
            "nul_",
            "a.b",
            "history-1.json",
        ] {
            assert!(valid_plain_name(good), "{good}");
        }
        assert!(!valid_file_name("images/con.png"));
    }

    const HASH: &str = "db47f4c49907d313d7588e63a07e8b1ed1b6331993499aeae4e578e4a338b488";
    const COMMIT: &str = "a89e80aa191bc052eb10b7ab0c0088f9ffa92a84";

    fn entry(id: &str) -> serde_json::Value {
        json!({
            "id": id, "version": "1.0.1", "api": 1, "layer": "light",
            "name": { "en": "Countdown", "zh-CN": "倒数日" },
            "description": { "en": "Days left." },
            "author": "DayuanJiang", "minPecoFence": "0.4.0",
            "permissions": { "network": [], "clipboard": "none" },
            "files": [
                { "name": "main.js", "size": 2745, "sha256": HASH, "url": "https://evil.example/x" },
                { "name": "plugin.json", "size": 453, "sha256": HASH },
                { "name": "README.md", "size": 900, "sha256": HASH }
            ]
        })
    }

    fn index(plugins: Vec<serde_json::Value>) -> Vec<u8> {
        serde_json::to_vec(&json!({ "format": 1, "commit": COMMIT, "plugins": plugins })).unwrap()
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
            ("twice-file", |v| v["files"][2]["name"] = "MAIN.js".into()),
            ("bad-clip", |v| v["permissions"]["clipboard"] = "all".into()),
            ("bad-host", |v| {
                v["permissions"]["network"] = json!(["a.com/x"])
            }),
            ("no-english", |v| v["name"] = json!({ "zh-CN": "倒数日" })),
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
    fn file_sets_of_both_layers() {
        let check = |layer: &str, files: &[&str]| {
            let mut v = entry("webdemo");
            v["layer"] = layer.into();
            v["files"] = files
                .iter()
                .map(|n| json!({ "name": n, "size": 10, "sha256": HASH }))
                .collect();
            check_entry(&serde_json::from_value(v).unwrap())
        };
        let web = ["plugin.json", "index.html", "README.md"];
        assert!(check("web", &web).is_ok());
        assert!(
            check(
                "web",
                &[&web[..], &["main.js", "img/icon.png", "css/a.b.css"]].concat()
            )
            .is_ok()
        );
        assert!(check("web", &["plugin.json", "main.js", "README.md"]).is_err());
        for bad in [
            "../x.js",
            "/x.js",
            "a\\x.js",
            "C:x.js",
            "img/../x.js",
            ".hidden",
            "a//b.js",
            "a b.js",
        ] {
            assert!(check("web", &[&web[..], &[bad]].concat()).is_err(), "{bad}");
        }
        let light = ["plugin.json", "main.js", "README.md"];
        assert!(check("light", &[&light[..], &["screenshot.png"]].concat()).is_ok());
        assert!(check("light", &[&light[..], &["index.html"]].concat()).is_err());
        assert!(check("light", &[&light[..], &["img/a.png"]].concat()).is_err());
        let many: Vec<String> = (0..MAX_FILES).map(|i| format!("f{i}.js")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        assert!(check("web", &[&web[..], &many].concat()).is_err());
    }

    #[test]
    fn hosts_are_names_with_a_dot_and_programs_exe_files() {
        for ok in ["wttr.in", "api.github.com", "a-b.example.org", "x1.io"] {
            assert!(valid_host(ok), "{ok}");
        }
        for bad in [
            "localhost",
            "intranet",
            "127.0.0.1",
            "10.0.0.8",
            "[::1]",
            "::1",
            "a.com/x",
            "A.com",
            "a.com:8080",
            "-a.com",
            "a-.com",
            "a..com",
            ".a.com",
            "a.com.",
            "a.1",
            "",
        ] {
            assert!(!valid_host(bad), "{bad}");
        }
        let with = |field: &str, list: serde_json::Value| {
            check_manifest(&manifest(|v| v["permissions"][field] = list))
        };
        assert!(with("links", json!(["github.com"])).is_ok());
        assert!(with("links", json!(["localhost"])).is_err());
        assert!(with("network", json!(["192.168.1.1"])).is_err());
        for ok in [
            "notepad.exe",
            "C:\\Program Files\\Tool\\tool.exe",
            "ipconfig.EXE",
        ] {
            assert!(with("system", json!([ok])).is_ok(), "{ok}");
        }
        for bad in [
            "notepad",
            "cmd.bat",
            "..\\x.exe",
            "C:\\a\\..\\x.exe",
            "C:/x.exe",
            "\\\\srv\\x.exe",
            "a b.exe",
        ] {
            assert!(with("system", json!([bad])).is_err(), "{bad}");
        }
    }

    #[test]
    fn index_needs_format_1_and_a_full_commit() {
        let ok = index(vec![]);
        assert_eq!(
            parse_index(&ok).unwrap().0.serial,
            0,
            "no serial reads as 0"
        );
        let with = json!({ "format": 1, "serial": 57, "commit": COMMIT, "plugins": [] });
        let with = parse_index(&serde_json::to_vec(&with).unwrap()).unwrap().0;
        assert_eq!(with.serial, 57);
        let text = String::from_utf8(ok).unwrap();
        assert!(parse_index(text.replace("\"format\":1", "\"format\":2").as_bytes()).is_err());
        assert!(parse_index(text.replace(COMMIT, "index").as_bytes()).is_err());
        assert!(parse_index(text.replace(COMMIT, &COMMIT.to_uppercase()).as_bytes()).is_err());
        assert!(parse_index(b"not json").is_err());
        assert!(parse_index(&vec![b' '; MAX_INDEX + 1]).is_err());
    }

    #[test]
    fn signature_is_checked_over_the_exact_bytes() {
        // RFC 8032 test 1's key, signing this index (made with Python's cryptography).
        let key: [u8; 32] = core::array::from_fn(|i| {
            let hex = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
            u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()
        });
        let data =
            br#"{"format":1,"commit":"a89e80aa191bc052eb10b7ab0c0088f9ffa92a84","plugins":[]}"#;
        let sig = "b01958c8cb79c229436f3c1beb1fa6b26e580a150076e6e97e6d17e3341898802211e5eaa5438d46dcf59f9888fa987357a30e944ab0fcb97ad28eb9f978540a";
        assert!(signed_by(&key, data, sig.as_bytes()));
        assert!(signed_by(&key, data, format!("{sig}\r\n").as_bytes()));
        let empty = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";
        assert!(signed_by(&key, b"", empty.as_bytes()));
        let changed = [&data[..data.len() - 1], b" }"].concat();
        assert!(!signed_by(&key, &changed, sig.as_bytes()));
        assert!(!signed_by(&key, data, empty.as_bytes()));
        assert!(!signed_by(&key, data, sig.to_uppercase().as_bytes()));
        assert!(!signed_by(&key, data, &sig.as_bytes()[..126]));
        assert!(!signed_by(&key, data, b""));
        // The repository's key did not sign it.
        assert!(!signed_by(&INDEX_KEY, data, sig.as_bytes()));
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
        let [raw, cdn, site] = file_urls(COMMIT, "countdown", "img/a.png");
        let path = format!("{COMMIT}/plugins/countdown/img/a.png");
        assert_eq!(
            raw,
            format!("https://raw.githubusercontent.com/{REPO}/{path}")
        );
        assert_eq!(cdn, format!("https://cdn.jsdelivr.net/gh/{REPO}@{path}"));
        assert_eq!(site, format!("https://pecofence.jiang.jp/plugins/{path}"));
        let [raw, cdn, site] = index_urls();
        assert_eq!(
            raw,
            format!("https://raw.githubusercontent.com/{REPO}/index/index.json")
        );
        assert_eq!(
            cdn,
            format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index.json")
        );
        assert_eq!(site, "https://pecofence.jiang.jp/plugins/index.json");
        let test = test_file_url(
            "http://127.0.0.1:8731/index.json",
            COMMIT,
            "countdown",
            "img/a.png",
        );
        assert_eq!(test, format!("http://127.0.0.1:8731/{path}"));
        let page = page_url(COMMIT, "countdown");
        assert_eq!(
            page,
            format!("https://github.com/{REPO}/blob/{COMMIT}/plugins/countdown/README.md")
        );
    }

    #[test]
    fn permissions_compared_field_by_field() {
        let perms = |p: serde_json::Value| -> Permissions { serde_json::from_value(p).unwrap() };
        let old = perms(
            json!({ "network": ["api.github.com"], "links": ["github.com"], "clipboard": "write", "system": ["ipconfig.exe"] }),
        );
        assert!(more_permissions(&old, &old).is_empty());
        assert!(more_permissions(&old, &Permissions::default()).is_empty());
        let fewer = perms(
            json!({ "network": ["api.github.com"], "links": ["api.github.com"], "system": ["IPCONFIG.exe"] }),
        );
        assert!(more_permissions(&old, &fewer).is_empty());
        let grows = [
            (json!({ "network": ["api.github.com", "x.com"] }), "network"),
            (json!({ "links": ["github.com", "x.com"] }), "links"),
            (json!({ "system": ["ipconfig.exe", "ping.exe"] }), "system"),
            (json!({ "clipboard": "read" }), "clipboard"),
            (json!({ "background": true }), "background"),
            (json!({ "folders": true }), "folders"),
            (json!({ "notify": true }), "notify"),
        ];
        for (change, field) in grows {
            let mut v = json!({ "network": ["api.github.com"], "links": ["github.com"], "clipboard": "write", "system": ["ipconfig.exe"] });
            v.as_object_mut()
                .unwrap()
                .extend(change.as_object().unwrap().clone());
            assert_eq!(more_permissions(&old, &perms(v)), [field]);
        }
        // Reading and writing the clipboard are two rights: read to write is more too.
        let clip = |c: &str| perms(json!({ "clipboard": c }));
        for (from, to, more) in [
            ("read", "write", true),
            ("write", "read", true),
            ("none", "write", true),
            ("read", "readwrite", true),
            ("readwrite", "read", false),
            ("readwrite", "write", false),
            ("write", "none", false),
        ] {
            let got = more_permissions(&clip(from), &clip(to));
            assert_eq!(!got.is_empty(), more, "{from} to {to}: {got:?}");
        }
    }

    #[test]
    fn update_kinds() {
        let installed = manifest(|_| {});
        let newer = manifest(|v| v["version"] = "1.0.2".into());
        assert_eq!(update_kind(&installed, &installed, "0.4.0"), Update::None);
        // A lower version is never an update.
        assert_eq!(update_kind(&newer, &installed, "0.4.0"), Update::None);
        assert_eq!(update_kind(&installed, &newer, "0.4.0"), Update::Auto);
        let needs = manifest(|v| {
            v["version"] = "1.1.0".into();
            v["minPecoFence"] = "0.5.0".into();
        });
        assert_eq!(
            update_kind(&installed, &needs, "0.4.0"),
            Update::NeedsNewerApp
        );
        assert_eq!(update_kind(&installed, &needs, "0.5.0"), Update::Auto);
        let api = manifest(|v| {
            v["version"] = "1.1.0".into();
            v["api"] = 2.into();
        });
        assert!(check_manifest(&api).is_ok());
        assert_eq!(
            update_kind(&installed, &api, "9.0.0"),
            Update::NeedsNewerApp
        );
        let more = manifest(|v| {
            v["version"] = "2.0.0".into();
            v["permissions"]["clipboard"] = "read".into();
        });
        assert_eq!(update_kind(&installed, &more, "0.4.0"), Update::Confirm);
        // A plugin that starts programs asks at every update, even with nothing new.
        let system = |version: &str| {
            manifest(|v| {
                v["version"] = version.into();
                v["permissions"]["system"] = json!(["ipconfig.exe"]);
            })
        };
        assert_eq!(
            update_kind(&system("1.0.1"), &system("1.0.2"), "0.4.0"),
            Update::Confirm
        );
        // A light plugin that becomes a web page asks too.
        let web = manifest(|v| {
            v["version"] = "1.0.2".into();
            v["layer"] = "web".into();
        });
        assert_eq!(update_kind(&installed, &web, "0.4.0"), Update::Confirm);
        // So does one whose author is someone else now.
        let other = manifest(|v| {
            v["version"] = "1.0.2".into();
            v["author"] = "someone-else".into();
        });
        assert_eq!(update_kind(&installed, &other, "0.4.0"), Update::Confirm);
    }

    #[test]
    fn the_shown_key_changes_with_what_the_warning_shows() {
        let m = manifest(|_| {});
        assert_eq!(shown_key(&m), shown_key(&m.clone()));
        let changed = [
            manifest(|v| v["version"] = "1.0.2".into()),
            manifest(|v| v["layer"] = "web".into()),
            manifest(|v| v["permissions"]["clipboard"] = "read".into()),
            manifest(|v| v["permissions"]["network"] = json!(["x.com"])),
            manifest(|v| v["permissions"]["notify"] = true.into()),
            manifest(|v| v["author"] = "someone-else".into()),
        ];
        for other in changed {
            assert_ne!(shown_key(&m), shown_key(&other), "{other:?}");
        }
        // The name and the description are not part of what it may do.
        let renamed = manifest(|v| v["description"] = json!({ "en": "Other words." }));
        assert_eq!(shown_key(&m), shown_key(&renamed));
    }

    #[test]
    fn settings_are_checked_and_saved_values_fit() {
        let m = manifest(|v| {
            v["settings"] = json!([
                { "key": "seconds", "type": "switch", "label": { "en": "Seconds" }, "default": false },
                { "key": "city", "type": "text", "default": "" },
                { "key": "days", "type": "number", "default": 3 },
                { "key": "unit", "type": "choice", "default": "c",
                  "options": [{ "value": "c", "label": { "en": "Celsius" } }, { "value": "f" }] }
            ])
        });
        assert!(check_manifest(&m).is_ok());
        let saved = json!({ "seconds": true, "city": 3, "days": 5, "unit": "k", "gone": 1 });
        let settings = fence_settings(&m, saved.as_object().unwrap());
        assert_eq!(
            settings,
            json!({ "seconds": true, "city": "", "days": 5, "unit": "c" })
        );
        let saved = json!({ "city": "Tokyo", "unit": "f" });
        let settings = fence_settings(&m, saved.as_object().unwrap());
        assert_eq!(settings["city"], "Tokyo");
        assert_eq!(settings["unit"], "f");
        for bad in [
            json!({ "settings": [{ "key": "a b", "type": "switch", "default": true }] }),
            json!({ "settings": [{ "key": "a", "type": "file", "default": "" }] }),
            json!({ "settings": [{ "key": "a", "type": "switch", "default": "yes" }] }),
            json!({ "settings": [{ "key": "a", "type": "choice", "default": "x", "options": [{ "value": "y" }] }] }),
            json!({ "settings": [{ "key": "a", "type": "choice", "default": "x" }] }),
            json!({ "settings": [{ "key": "a", "type": "text", "default": "" }, { "key": "a", "type": "text", "default": "" }] }),
            json!({ "api": 0 }),
            json!({ "layer": "native" }),
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
        let only_cn = manifest(|v| v["name"] = json!({ "en": "Countdown", "zh-CN": "倒数日" }));
        assert_eq!(pick(&only_cn.name, "zh-TW"), "倒数日");
        assert_eq!(pick(&BTreeMap::new(), "en"), "");
    }
}
