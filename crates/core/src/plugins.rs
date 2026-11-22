//! 插件: plugins from the DayuanJiang/PecoFence-plugins repository. This module reads and
//! checks the repository's signed `index.json` and a plugin's `plugin.json`, builds the
//! addresses of a plugin's files (from the reviewed commit, never from the index) and says what
//! an update may do. It touches neither the network nor the disk.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const REPO: &str = "DayuanJiang/PecoFence-plugins";
/// The project website's copy of the index and the files: the third address, for networks that
/// reach neither GitHub nor jsDelivr.
pub const SITE: &str = "https://pecofence.jiang.jp/plugins";
/// The Ed25519 key `index.json.sig` is checked with: the release key, made 2026-11-21. Its
/// private half exists only as the plugin repository's secret INDEX_SIGNING_KEY, which the
/// workflow that publishes index.json signs with.
pub const INDEX_KEY: [u8; 32] = [
    0x95, 0x17, 0x28, 0x21, 0x9d, 0xdb, 0x87, 0x47, 0x3f, 0x71, 0xbf, 0x70, 0x1b, 0x01, 0x9d, 0xdc,
    0x33, 0x42, 0x10, 0x81, 0xa1, 0xc2, 0x79, 0x76, 0xb4, 0x31, 0x21, 0xd7, 0x13, 0x6a, 0x6b, 0xe1,
];
/// The files whose names are fixed, with the most bytes each may have. A light plugin has only
/// these; a web plugin also has `index.html` and the files it uses (sub-folders allowed), each
/// up to `OTHER_SIZE`. These are the one place of the sizes: the index check, the checker and
/// the plugin repository's rules take them from here (0.5.0 had main.js 200 KB, other files
/// 1 MB, 100 files and 5 MB, see [`V2_SIZES`]). main.js runs in the light engine's 8 MB: 700 KB
/// of dense code (11,000 small functions) still starts there and 800 KB does not, so 512 KB
/// leaves a plugin room for what it keeps.
pub const FILES: [(&str, u64); 4] = [
    ("plugin.json", 64 * 1024),
    ("main.js", 512 * 1024),
    ("README.md", 64 * 1024),
    ("screenshot.png", 300 * 1024),
];
pub const OTHER_SIZE: u64 = 5 * 1024 * 1024;
/// A plugin's most files, and most bytes together.
pub const MAX_FILES: usize = 500;
pub const MAX_TOTAL: u64 = 20 * 1024 * 1024;
/// The sizes 0.5.0 and older check an index entry against (main.js, any other file, files, all
/// bytes): an entry past them is dropped there, so it needs the newer index and `api: 3`.
pub const V2_SIZES: (u64, u64, usize, u64) = (200 * 1024, 1024 * 1024, 100, 5 * 1024 * 1024);

/// The newest plugin interface this PecoFence knows (`api` in plugin.json).
pub const API: u32 = 3;
/// The most bytes an index may have: `index-3.json`, which this PecoFence reads first. The
/// `index.json` of PecoFence 0.5.0 and earlier stays under 1 MB (their limit): the plugin
/// repository's build-index keeps it so.
pub const MAX_INDEX: usize = 16 * 1024 * 1024;
/// The first release that knows interface 3: a plugin using what interface 3 brought says
/// `api: 3` and at least this `minPecoFence` (the plugin repository's checks).
pub const API3_RELEASE: &str = "0.6.0";

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
    /// Reads (`read`, or `true`) or reads and writes (`readwrite`, interface 2) the folders the
    /// user picks.
    #[serde(deserialize_with = "folders_field")]
    pub folders: Folders,
    /// Shows notices and plays their sound.
    pub notify: bool,
    // Interface 2 (each needs `api: 2`, see [`API2_FIELDS`]).
    /// Commands the user may give shortcuts (plugin details); a press sends `command`.
    pub commands: Vec<Command>,
    /// What may be dropped on its fence: `files`, `text`, `links`.
    pub drop: Vec<String>,
    /// Opens the files, folders and programs the user put into its fence (`pf.launch`).
    pub launch: bool,
    /// Sees the media session playing now, and may pause it and skip.
    pub media: Media,
    /// The programs it may know are running or in front (`spotify.exe`).
    pub apps: Vec<String>,
    /// Services of this computer and of a device of the local network, by port.
    pub local: Local,
    // Interface 3 (each needs `api: 3`, see [`API3_FIELDS`]).
    /// A web page may capture the screen (`getDisplayMedia`; the user picks what to share).
    pub screen: bool,
    /// Knows where the pointer is (`pf.pointer`, event `pointer`).
    pub pointer: bool,
    /// Knows where the keyboard focus and the text caret are (`pf.focus`, `pf.caret`).
    pub focus: bool,
    /// A web page may use the camera.
    pub camera: bool,
    /// A web page may use the microphone.
    pub microphone: bool,
    /// A web page may ask where the computer is (geolocation).
    pub location: bool,
    /// A web page may use WebRTC: direct connections to other computers, at any address.
    pub webrtc: bool,
}

/// `local`: services `pf.http` may reach outside the internet (plain http too, never through a
/// proxy), each a port with the program that answers there: on this computer (`loopback`), and
/// on the device whose address the user typed in a `device` setting (`lan`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Local {
    pub loopback: Vec<LocalPort>,
    pub lan: Vec<LocalPort>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocalPort {
    pub port: u16,
    pub app: String,
}

impl Local {
    /// Every (class, port) it names: `("loopback", 11434)`, `("lan", 8123)`.
    pub fn ports(&self) -> Vec<(&'static str, u16)> {
        let loopback = self.loopback.iter().map(|p| ("loopback", p.port));
        loopback
            .chain(self.lan.iter().map(|p| ("lan", p.port)))
            .collect()
    }
}

/// The most ports `local` named together in 0.5.0 and older (any number since interface 3).
pub const V2_LOCAL_PORTS: usize = 4;
/// Ports no plugin request reaches (`local` may not name them, and a request to any host on one
/// is refused): debuggers and remote administration, whose services run code or take over a
/// computer for whoever connects.
pub const REFUSED_PORTS: [(u16, &str); 19] = [
    (22, "SSH"),
    (23, "Telnet"),
    (135, "Windows RPC"),
    (139, "NetBIOS"),
    (445, "SMB"),
    (2375, "Docker"),
    (2376, "Docker over TLS"),
    (3389, "Remote Desktop"),
    (4444, "Selenium WebDriver"),
    (5037, "Android Debug Bridge"),
    (5555, "Android Debug Bridge over the network"),
    (5900, "VNC"),
    (5985, "WinRM"),
    (5986, "WinRM over HTTPS"),
    (6443, "Kubernetes API"),
    (9222, "Chrome and Edge DevTools"),
    (9229, "Node.js inspector"),
    (9515, "ChromeDriver"),
    (10250, "Kubernetes kubelet"),
];

/// Which part of `local` lets a plugin reach address `a`: `loopback` for this computer by its
/// name (`localhost`, `127.x.x.x`) on a `loopback` port, `lan` for any other host on a `lan`
/// port (the app then checks that it is the address the user typed). The port is the address's,
/// else 80 or 443.
pub fn local_class(p: &Permissions, a: &crate::web_address::WebAddress) -> Option<&'static str> {
    let port = a.port.unwrap_or(if a.secure { 443 } else { 80 });
    let has = |list: &[LocalPort]| list.iter().any(|l| l.port == port);
    let this = a.host == "localhost"
        || a.host
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.is_loopback());
    if this && has(&p.local.loopback) {
        Some("loopback")
    } else if has(&p.local.lan) {
        Some("lan")
    } else {
        None
    }
}

/// The host a user typed in a `device` setting (`192.168.1.20`, `HomeAssistant.local.`, or
/// `http://192.168.1.20:8123/` copied from a browser): in lowercase, without scheme, port, path
/// or a dot at the end. None for anything else.
pub fn device_host(typed: &str) -> Option<String> {
    let t = typed.trim();
    let t = (t
        .strip_prefix("http://")
        .or_else(|| t.strip_prefix("https://")))
    .unwrap_or(t);
    crate::web_address::WebAddress::parse(&format!("http://{t}")).map(|a| a.host)
}

/// One command of `commands`: its id, its title by language and (interface 3) the shortcut the
/// plugin brings (`"Ctrl+Shift+M"`), which the user may change or clear.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Command {
    pub id: String,
    #[serde(default)]
    pub title: BTreeMap<String, String>,
    /// Interface 3: the shortcut it brings (`Ctrl+Shift+M`), given to the fence an install makes
    /// unless the user's fence has one for it already (see [`command_key_ok`]).
    #[serde(default)]
    pub key: Option<String>,
}

/// Whether `key` may be a command's own shortcut: a shortcut PecoFence's own boxes would take
/// (`Hotkey::check`: a supported key, Ctrl, Alt or Win, no common Windows shortcut) that also
/// holds Shift or Win. Ctrl+Alt with a letter alone is AltGr on German, French, Polish and other
/// layouts: a plugin that took it would eat a character those users type (AltGr+Q is @).
pub fn command_key_ok(key: &str) -> bool {
    use crate::hotkey::{Hotkey, SHIFT, WIN};
    Hotkey::parse(key)
        .and_then(Hotkey::check)
        .is_ok_and(|h| h.mods & (SHIFT | WIN) != 0)
}

/// What a plugin may do in the folders the user picks for it: none < read < readwrite < any.
/// `readwrite` writes the data files of [`crate::folder_write::DATA_TYPES`]; `any` (interface 3)
/// any file, also bytes (`pf.folder.writeBytes`), each file outside those types marked as coming
/// from the internet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Folders {
    #[default]
    None,
    Read,
    Readwrite,
    Any,
}

impl Folders {
    pub fn reads(self) -> bool {
        self >= Self::Read
    }
    /// `pf.folder.write`, `mkdir` and `trash`.
    pub fn writes(self) -> bool {
        self >= Self::Readwrite
    }
    /// Any kind of file, not only data files, and `pf.folder.writeBytes`.
    pub fn any_kind(self) -> bool {
        self == Self::Any
    }
}

/// `folders` as interface 1 wrote it (`true` reads, `false` is none) or by name.
fn folders_field<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Folders, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Flag(bool),
        Named(Folders),
    }
    Ok(match Either::deserialize(d)? {
        Either::Flag(on) if on => Folders::Read,
        Either::Flag(_) => Folders::None,
        Either::Named(f) => f,
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Media {
    #[default]
    None,
    Read,
    Control,
}

impl Media {
    pub fn reads(self) -> bool {
        self >= Self::Read
    }
    pub fn controls(self) -> bool {
        self == Self::Control
    }
}

/// The permissions interface 2 brought: a manifest using any of them must say `api: 2`, so that
/// a PecoFence of interface 1 (which would not see them) shows 需要更新 PecoFence instead of
/// installing it as asking for nothing more.
pub const API2_FIELDS: [&str; 6] = ["commands", "drop", "launch", "media", "apps", "local"];

impl Permissions {
    /// The fields of [`API2_FIELDS`] that `self` uses.
    pub fn api2_fields(&self) -> Vec<&'static str> {
        let used = [
            !self.commands.is_empty(),
            !self.drop.is_empty(),
            self.launch,
            self.media != Media::None,
            !self.apps.is_empty(),
            !self.local.ports().is_empty(),
        ];
        API2_FIELDS
            .into_iter()
            .zip(used)
            .filter_map(|(f, u)| u.then_some(f))
            .collect()
    }

    /// What `self` uses of interface 3: its new fields ([`API3_FIELDS`]), `folders` `any`, a
    /// command's own `key`, and what interface 2 refused: a host that is an IP address or a name
    /// without a dot, a script in `system`, more than 20 programs or 4 local ports. PecoFence 0.5.0 drops a
    /// plugin with any of these, or would run it without them.
    pub fn api3_uses(&self) -> Vec<&'static str> {
        let flags = [
            self.screen,
            self.pointer,
            self.focus,
            self.camera,
            self.microphone,
            self.location,
            self.webrtc,
        ];
        let mut used: Vec<&'static str> = (API3_FIELDS.into_iter().zip(flags))
            .filter_map(|(f, u)| u.then_some(f))
            .collect();
        let mut hosts = self.network.iter().chain(&self.links);
        let more = [
            (self.folders == Folders::Any, "folders any"),
            (
                self.commands.iter().any(|c| c.key.is_some()),
                "command keys",
            ),
            (hosts.any(|h| !host_of_api2(h)), "IP or dotless hosts"),
            (
                self.system.iter().any(|s| !program_of_api2(s)),
                "scripts in system",
            ),
            (self.system.len() > 20, "more than 20 programs"),
            (
                self.local.ports().len() > LOCAL_PORTS_API2,
                "more than 4 local ports",
            ),
        ];
        used.extend(more.into_iter().filter_map(|(u, f)| u.then_some(f)));
        used
    }
}

/// The permissions interface 3 brought: a manifest using any of them must say `api: 3`, so that
/// PecoFence 0.5.0 (which would not see them) shows 需要更新 PecoFence instead of installing it.
pub const API3_FIELDS: [&str; 7] = [
    "screen",
    "pointer",
    "focus",
    "camera",
    "microphone",
    "location",
    "webrtc",
];
/// The most local ports interface 2 allowed (more need interface 3).
pub const LOCAL_PORTS_API2: usize = 4;

/// A host interface 2 accepted (PecoFence 0.5.0's rule): a name in lower case with a dot, whose
/// last part starts with a letter. An IP address or a name without a dot needs interface 3.
pub fn host_of_api2(host: &str) -> bool {
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

/// A program interface 2 accepted (PecoFence 0.5.0's rule): an `.exe`, by full path or as a name
/// in System32. A script (`.cmd`, `.bat`, `.ps1`) needs interface 3.
pub fn program_of_api2(p: &str) -> bool {
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

/// The file sizes interface 2 allowed (PecoFence 0.5.0's): a larger file, or a plugin of more
/// files or bytes, needs interface 3.
pub const SIZES_API2: [(&str, u64); 4] = [
    ("plugin.json", 64 * 1024),
    ("main.js", 200 * 1024),
    ("README.md", 64 * 1024),
    ("screenshot.png", 300 * 1024),
];
pub const OTHER_SIZE_API2: u64 = 1024 * 1024;
pub const MAX_FILES_API2: usize = 100;
pub const MAX_TOTAL_API2: u64 = 5 * 1024 * 1024;

/// Whether the files of `e` stay within interface 2's sizes ([`SIZES_API2`] and the rest).
pub fn files_of_api2(e: &Entry) -> bool {
    let within = |f: &FileEntry| {
        let fixed = SIZES_API2.iter().find(|(name, _)| *name == f.name);
        f.size <= fixed.map_or(OTHER_SIZE_API2, |(_, max)| *max)
    };
    let total: u64 = e.files.iter().map(|f| f.size).sum();
    e.files.iter().all(within) && e.files.len() <= MAX_FILES_API2 && total <= MAX_TOTAL_API2
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

/// One setting of a plugin fence (插件设置): `switch`, `text`, `number`, `choice`, and of
/// interface 2 (no default, only what the user types) `secret` (kept encrypted under
/// [`SECRETS`]) and `device` (the address of a device of the local network, kept under
/// [`DEVICES`]).
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

/// The fence settings' key under which the app keeps the `secret` settings, encrypted for this
/// Windows account (an import deletes it with every key that starts with `__`).
pub const SECRETS: &str = "__secrets";
/// The fence settings' key under which the app keeps the `device` settings ({key: host}), the
/// addresses `local.lan` may reach (an import deletes it too).
pub const DEVICES: &str = "__devices";

/// A plugin fence's settings as its plugin sees them (`pf.settings`): what the fence saved
/// when it fits the setting, else the setting's default. A `secret` is null here: the app puts
/// in what it decrypts from [`SECRETS`]. A `device` is the host kept under [`DEVICES`], or null.
pub fn fence_settings(
    m: &Manifest,
    saved: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let null = serde_json::Value::Null;
    let value = |s: &Setting| {
        let device = saved.get(DEVICES).and_then(|d| d.get(&s.key));
        let kept = match s.kind.as_str() {
            "secret" => None,
            "device" => Some(device.filter(|v| v.is_string()).unwrap_or(&null)),
            _ => (saved.get(&s.key)).filter(|v| fits(s, v)),
        };
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
    /// `index.json`'s stand-in of a plugin PecoFence 0.5.0 cannot read (which ignores the
    /// field): its manifest and files are not the plugin's, so it counts only for a take-down,
    /// never for an install or an update ([`Index::offer`]).
    #[serde(default, rename = "standIn")]
    pub stand_in: bool,
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
    /// Plugin `id`'s entry, a stand-in too: what a take-down reads.
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.plugins.iter().find(|e| e.manifest.id == id)
    }

    /// Plugin `id`'s entry as it may be installed or updated from: none for a stand-in.
    pub fn offer(&self, id: &str) -> Option<&Entry> {
        self.get(id).filter(|e| !e.stand_in)
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

/// A host of `network` that is no internet name (interface 3): an IPv4 address (`192.168.1.20`,
/// `8.8.8.8`), an IPv6 address in brackets as an address writes it (`[fd00::1]`, in its short
/// lowercase form), or a name without a dot (`nas`). Such a host is reached by the local rules
/// when it is this computer or the local network ([`network_local_class`]).
pub fn valid_network_host(host: &str) -> bool {
    if let Some(inner) = host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
        return inner
            .parse::<std::net::Ipv6Addr>()
            .is_ok_and(|ip| ip.to_string() == inner);
    }
    if host.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
        return host
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.to_string() == host);
    }
    (1..=63).contains(&host.len())
        && host.as_bytes()[0].is_ascii_lowercase()
        && !host.ends_with('-')
        && host
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// Which local rules a request to `network` host `host` goes by (interface 3): `loopback` for
/// this computer (`localhost`, 127/8, `[::1]`), `lan` for the local network (a private,
/// link-local or 100.64/10 address, a name without a dot, a name under `.local`, `.lan`,
/// `.internal` or `.home.arpa`); None for the internet. A `lan` host of `network` was declared
/// in the manifest (and shown before install), so it needs no `device` setting; it connects only
/// to an address of the local network, never through a proxy, and to none of [`REFUSED_PORTS`].
pub fn network_local_class(host: &str) -> Option<&'static str> {
    use crate::web_address::{lan_ip, loopback_ip, read_ip};
    if host == "localhost" || loopback_ip(host) {
        return Some("loopback");
    }
    if read_ip(host).is_some() {
        return lan_ip(host).then_some("lan");
    }
    let local_suffix = [".local", ".lan", ".internal", ".home.arpa"];
    (!host.contains('.') || local_suffix.iter().any(|s| host.ends_with(s))).then_some("lan")
}

/// Whether a web plugin's page may reach `network` host `host` itself (its CSP and
/// Connection-Allowlist list it): an internet name or a public IPv4 address. This computer, the
/// local network, IPv6 addresses (which a CSP cannot name) and IPv4 addresses that are neither
/// public nor local (0.0.0.0, multicast, 240/4) it reaches through `pf.http` only, which keeps
/// their rules.
pub fn page_reaches(host: &str) -> bool {
    if host.starts_with('[') {
        return false;
    }
    if crate::web_address::read_ip(host).is_some() {
        return crate::web_address::public_ip(host);
    }
    network_local_class(host).is_none()
}

/// The kinds of program `system` may name: a program, and (interface 3) a batch file, which
/// Rust's `Command` starts through cmd.exe with arguments it can quote safely (or refuses), and
/// a PowerShell script, which the app starts with System32's powershell.exe.
pub const PROGRAM_KINDS: [&str; 4] = [".exe", ".bat", ".cmd", ".ps1"];

/// A program of `system`: a full path (`C:\…\x.exe`) or the name of a program in System32,
/// ending in one of [`PROGRAM_KINDS`].
pub fn valid_program(p: &str) -> bool {
    let b = p.as_bytes();
    let lower = p.to_ascii_lowercase();
    let exe = (5..=260).contains(&p.len()) && PROGRAM_KINDS.iter().any(|k| lower.ends_with(k));
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

/// A WebView2 runtime of `version` ("157.0.4341.60") applies all a web plugin's page is held
/// to: 152 is the first that reads the page's Connection-Allowlist (dns-prefetch, preconnect and
/// WebRTC to hosts the plugin did not declare). A version that cannot be read does not.
pub fn webview2_enough(version: &str) -> bool {
    let major = version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok());
    major.is_some_and(|m| m >= 152)
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
    let kinds = ["switch", "text", "number", "choice", "secret", "device"];
    // A secret and a device have no default: only what the user types.
    let default_ok = |s: &Setting| match s.kind.as_str() {
        "secret" | "device" => s.default.is_null(),
        _ => fits(s, &s.default),
    };
    let good = |(i, s): (usize, &Setting)| {
        (1..=32).contains(&s.key.len())
            && s.key.bytes().all(|c| c.is_ascii_alphanumeric())
            && !m.settings[..i].iter().any(|t| t.key == s.key)
            && kinds.contains(&s.kind.as_str())
            && short(&s.label, 60)
            && (s.kind != "choice" || (1..=50).contains(&s.options.len()))
            && s.options.iter().all(|o| short(&o.label, 60))
            && default_ok(s)
    };
    if m.settings.len() > 20 || !m.settings.iter().enumerate().all(good) {
        return Err(format!("{}: bad settings", m.id));
    }
    let p = &m.permissions;
    // Before the rules of each field: what interface 2 refused is interface 3's.
    let newer3 = p.api3_uses();
    if m.api < 3 && !newer3.is_empty() {
        return Err(format!("{}: {} need api 3", m.id, newer3.join(", ")));
    }
    let network_ok = |h: &String| valid_host(h) || valid_network_host(h);
    if p.network.len() > 20 || !p.network.iter().all(network_ok) {
        return Err(format!("{}: bad network hosts", m.id));
    }
    if p.links.len() > 20 || !p.links.iter().all(|h| valid_host(h)) {
        return Err(format!("{}: bad links hosts", m.id));
    }
    if !p.system.iter().all(|s| valid_program(s)) {
        return Err(format!("{}: bad system programs", m.id));
    }
    let mut newer = p.api2_fields();
    if m.settings.iter().any(|s| s.kind == "secret") {
        newer.push("secret settings");
    }
    if p.folders.writes() {
        newer.push("folders readwrite");
    }
    let device = m.settings.iter().any(|s| s.kind == "device");
    if device {
        newer.push("device settings");
    }
    if m.api < 2 && !newer.is_empty() {
        return Err(format!("{}: {} need api 2", m.id, newer.join(", ")));
    }
    let ports = p.local.ports();
    let entries = (p.local.loopback.iter()).chain(&p.local.lan);
    let app_named = |l: &LocalPort| !l.app.trim().is_empty() && l.app.chars().count() <= 40;
    if (0..ports.len()).any(|i| ports[..i].contains(&ports[i]) || ports[i].1 == 0)
        || !entries.clone().all(app_named)
    {
        return Err(format!(
            "{}: bad local ports (each once with its program)",
            m.id
        ));
    }
    if let Some((port, what)) =
        (REFUSED_PORTS.iter()).find(|(r, _)| ports.iter().any(|p| p.1 == *r))
    {
        return Err(format!("{}: port {port} ({what}) may not be reached", m.id));
    }
    // A device setting is the address `local.lan` goes to: one is no use without the other.
    if device != !p.local.lan.is_empty() {
        return Err(format!(
            "{}: a device setting and permissions.local.lan go together",
            m.id
        ));
    }
    let once = |list: &[&str]| (0..list.len()).all(|i| !list[..i].contains(&list[i]));
    let ids: Vec<&str> = p.commands.iter().map(|c| c.id.as_str()).collect();
    let command_ok = |c: &Command| {
        (1..=32).contains(&c.id.len())
            && c.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && c.title.get("en").is_some_and(|t| !t.trim().is_empty())
            && short(&c.title, 40)
            && c.key.as_deref().is_none_or(command_key_ok)
    };
    if p.commands.len() > 8 || !once(&ids) || !p.commands.iter().all(command_ok) {
        return Err(format!("{}: bad commands", m.id));
    }
    let drops: Vec<&str> = p.drop.iter().map(String::as_str).collect();
    let drop_kinds = ["files", "text", "links"];
    if !once(&drops) || !drops.iter().all(|d| drop_kinds.contains(d)) {
        return Err(format!("{}: bad drop kinds", m.id));
    }
    let app_ok = |a: &String| valid_plain_name(a) && a.to_ascii_lowercase().ends_with(".exe");
    if p.apps.len() > 20 || !p.apps.iter().all(app_ok) {
        return Err(format!("{}: bad apps", m.id));
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
    if e.manifest.api < 3 && !files_of_api2(e) {
        return Err(format!("{id}: files over the sizes of api 2 need api 3"));
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

/// The index: from GitHub, then jsDelivr, then the project website, `index-3.json` (every
/// plugin) before `index.json` (what PecoFence 0.5.0 reads; taken while the plugin repository
/// publishes no `index-3.json` yet). Its signature is at the same address with `.sig`.
pub fn index_urls() -> [String; 6] {
    let at = |name: &str| {
        [
            format!("https://raw.githubusercontent.com/{REPO}/index/{name}"),
            format!("https://cdn.jsdelivr.net/gh/{REPO}@index/{name}"),
            format!("{SITE}/{name}"),
        ]
    };
    let [a, b, c] = at("index-3.json");
    let [d, e, f] = at("index.json");
    [a, b, c, d, e, f]
}

/// The addresses a test index (`PECOFENCE_PLUGINS_URL`) is read from: `index-3.json` beside an
/// `index.json`, then the address itself.
pub fn test_index_urls(url: &str) -> Vec<String> {
    match url.strip_suffix("/index.json") {
        Some(dir) => vec![format!("{dir}/index-3.json"), url.to_string()],
        None => vec![url.to_string()],
    }
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
/// writing the clipboard when it did not before, or `background`, `notify` turned on.
/// Interface 2: a command or a kind of drop or a program of `apps` not there before, `launch`
/// turned on, `media` from none to read to control, `folders` from none to read to readwrite,
/// a class and port of `local` not there before (`loopback` and `lan` apart). Interface 3:
/// `screen`, `pointer`, `focus`, `camera`, `microphone`, `location` or `webrtc` turned on,
/// `folders` up to `any`, a command's `key` added or changed.
pub fn more_permissions(old: &Permissions, new: &Permissions) -> Vec<&'static str> {
    let old_ports = old.local.ports();
    let new_port = new.local.ports().iter().any(|p| !old_ports.contains(p));
    let added = |o: &[String], n: &[String]| {
        n.iter()
            .any(|h| !o.iter().any(|g| g.eq_ignore_ascii_case(h)))
    };
    let links = [old.links.as_slice(), old.network.as_slice()].concat();
    // Command ids are compared exactly: `Show` is another command than `show`. A command whose
    // own key is new or another one asks too: the key takes effect for the whole system.
    let new_command = (new.commands.iter()).any(|n| {
        !old.commands
            .iter()
            .any(|o| o.id == n.id && (n.key.is_none() || o.key == n.key))
    });
    let on = |o: bool, n: bool| n && !o;
    [
        ("screen", on(old.screen, new.screen)),
        ("pointer", on(old.pointer, new.pointer)),
        ("focus", on(old.focus, new.focus)),
        ("camera", on(old.camera, new.camera)),
        ("microphone", on(old.microphone, new.microphone)),
        ("location", on(old.location, new.location)),
        ("webrtc", on(old.webrtc, new.webrtc)),
        ("commands", new_command),
        ("drop", added(&old.drop, &new.drop)),
        ("launch", new.launch && !old.launch),
        ("media", new.media > old.media),
        ("apps", added(&old.apps, &new.apps)),
        ("local", new_port),
        ("network", added(&old.network, &new.network)),
        ("links", added(&links, &new.links)),
        ("system", added(&old.system, &new.system)),
        (
            "clipboard",
            (new.clipboard.reads() && !old.clipboard.reads())
                || (new.clipboard.writes() && !old.clipboard.writes()),
        ),
        ("background", new.background && !old.background),
        ("folders", new.folders > old.folders),
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
    fn web_pages_need_webview2_152() {
        assert!(webview2_enough("157.0.4341.60") && webview2_enough("152.0.0.0"));
        for old in ["151.0.3405.125", "86.0.616.0", "", "x.1"] {
            assert!(!webview2_enough(old), "{old}");
        }
    }

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
    fn a_stand_in_counts_for_its_take_down_only() {
        let mut stand = entry("stand");
        stand["standIn"] = true.into();
        stand["disabled"] = "taken down".into();
        let (i, dropped) = parse_index(&index(vec![stand, entry("plain")])).unwrap();
        assert!(dropped.is_empty(), "{dropped:?}");
        let s = i.get("stand").unwrap();
        assert!(s.stand_in && s.disabled.as_deref() == Some("taken down"));
        assert!(i.offer("stand").is_none());
        assert!(i.offer("plain").is_some_and(|e| !e.stand_in));
    }

    #[test]
    fn index_keeps_good_entries_and_drops_bad_ones() {
        let bad: [(&str, fn(&mut serde_json::Value)); 9] = [
            ("bad-file", |v| v["files"][0]["name"] = "run.exe".into()),
            ("too-big", |v| {
                v["files"][0]["size"] = (512 * 1024 + 1).into()
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
        assert!(with("links", json!(["192.168.1.1"])).is_err());
        // Interface 3: network may name addresses and names without a dot, as written (api 3).
        let with = |field: &str, list: serde_json::Value| {
            check_manifest(&manifest(|v| {
                v["api"] = json!(3);
                v["permissions"][field] = list;
            }))
        };
        assert!(with("network", json!(["nas"])).is_ok());
        for ok in [
            "192.168.1.1",
            "8.8.8.8",
            "nas",
            "nas-2",
            "localhost",
            "[fd00::1]",
            "[::1]",
        ] {
            assert!(with("network", json!([ok])).is_ok(), "{ok}");
        }
        for bad in [
            "[FD00::1]",
            "[fd00:0::1]",
            "fd00::1",
            "1.2.3",
            "01.2.3.4",
            "Nas",
            "-nas",
            "n_s",
        ] {
            assert!(with("network", json!([bad])).is_err(), "{bad}");
        }
        for ok in [
            "notepad.exe",
            "C:\\Program Files\\Tool\\tool.exe",
            "ipconfig.EXE",
            "C:\\Tools\\backup.cmd",
            "C:\\Tools\\sync.bat",
            "C:\\Tools\\report.ps1",
        ] {
            assert!(with("system", json!([ok])).is_ok(), "{ok}");
        }
        let many: Vec<String> = (0..30).map(|i| format!("p{i}.exe")).collect();
        assert!(with("system", json!(many)).is_ok(), "no count limit");
        for bad in [
            "notepad",
            "x.vbs",
            "x.ps1.txt",
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
        let [raw3, cdn3, site3, raw, cdn, site] = index_urls();
        assert_eq!(
            raw3,
            format!("https://raw.githubusercontent.com/{REPO}/index/index-3.json")
        );
        assert_eq!(
            cdn3,
            format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index-3.json")
        );
        assert_eq!(site3, "https://pecofence.jiang.jp/plugins/index-3.json");
        assert_eq!(
            raw,
            format!("https://raw.githubusercontent.com/{REPO}/index/index.json")
        );
        assert_eq!(
            cdn,
            format!("https://cdn.jsdelivr.net/gh/{REPO}@index/index.json")
        );
        assert_eq!(site, "https://pecofence.jiang.jp/plugins/index.json");
        assert_eq!(
            test_index_urls("http://127.0.0.1:8731/index.json"),
            [
                "http://127.0.0.1:8731/index-3.json",
                "http://127.0.0.1:8731/index.json"
            ]
        );
        assert_eq!(
            test_index_urls("http://127.0.0.1:8731/feed.json"),
            ["http://127.0.0.1:8731/feed.json"]
        );
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
            (json!({ "folders": "readwrite" }), "folders"),
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
    fn the_fields_of_interface_2_need_api_2_and_each_growth_asks() {
        let with = |api: u32, p: serde_json::Value| {
            manifest(|v| {
                v["api"] = api.into();
                v["permissions"] = p;
            })
        };
        let new = [
            json!({ "commands": [{ "id": "show", "title": { "en": "Show" } }] }),
            json!({ "drop": ["files"] }),
            json!({ "launch": true }),
            json!({ "media": "read" }),
            json!({ "apps": ["spotify.exe"] }),
            json!({ "local": { "loopback": [{ "port": 11434, "app": "Ollama" }] } }),
        ];
        assert_eq!(new.len(), API2_FIELDS.len());
        for (p, field) in new.iter().zip(API2_FIELDS) {
            let why = check_manifest(&with(1, p.clone())).unwrap_err();
            assert!(why.contains(field) && why.contains("api 2"), "{why}");
            assert!(check_manifest(&with(2, p.clone())).is_ok(), "{p}");
            // A PecoFence of interface 1 would show 需要更新 PecoFence for it, this one installs.
            assert!(!needs_newer_app(&with(2, p.clone()), "0.5.0"));
            // From nothing to the field: more.
            let more = more_permissions(&Permissions::default(), &with(2, p.clone()).permissions);
            assert_eq!(more, [field]);
        }
        let perms = |p: serde_json::Value| -> Permissions { serde_json::from_value(p).unwrap() };
        let old = perms(json!({
            "commands": [{ "id": "show", "title": { "en": "Show" } }], "drop": ["files"],
            "launch": true, "media": "read", "apps": ["spotify.exe"]
        }));
        assert!(more_permissions(&old, &old).is_empty());
        for (change, field) in [
            (
                json!({ "commands": [{ "id": "show" }, { "id": "add" }] }),
                "commands",
            ),
            (json!({ "commands": [{ "id": "Show" }] }), "commands"),
            (json!({ "drop": ["files", "text"] }), "drop"),
            (json!({ "drop": ["links", "files"] }), "drop"),
            (json!({ "media": "control" }), "media"),
            (json!({ "apps": ["Spotify.exe", "obs64.exe"] }), "apps"),
        ] {
            let mut v = json!({ "commands": [{ "id": "show" }], "drop": ["files"], "launch": true, "media": "read", "apps": ["spotify.exe"] });
            v.as_object_mut()
                .unwrap()
                .extend(change.as_object().unwrap().clone());
            assert_eq!(more_permissions(&old, &perms(v)), [field]);
        }
        // Less is never more: control to read, fewer apps, launch off.
        let less = perms(json!({ "media": "read", "apps": ["SPOTIFY.EXE"] }));
        assert!(
            more_permissions(
                &perms(
                    json!({ "media": "control", "apps": ["spotify.exe", "x.exe"], "launch": true })
                ),
                &less
            )
            .is_empty()
        );
        assert!(Media::Control.reads() && Media::Control.controls() && !Media::Read.controls());
        // Each of these reads as a manifest, and the check refuses it.
        for bad in [
            json!({ "drop": ["folders"] }),
            json!({ "drop": ["files", "files"] }),
            json!({ "drop": ["html"] }),
            json!({ "drop": ["text", "Text"] }),
            json!({ "apps": ["C:\\x\\spotify.exe"] }),
            json!({ "apps": ["spotify"] }),
            json!({ "commands": [{ "id": "a b", "title": { "en": "A" } }] }),
            json!({ "commands": [{ "id": "a", "title": { "zh-CN": "甲" } }] }),
            json!({ "commands": [{ "id": "a", "title": { "en": "A" } }, { "id": "a", "title": { "en": "B" } }] }),
            json!({ "commands": [{ "id": "a", "title": { "en": "x".repeat(41) } }] }),
        ] {
            let m = manifest(|v| {
                v["api"] = 2.into();
                v["permissions"] = bad.clone();
            });
            assert!(check_manifest(&m).is_err(), "{bad}");
        }
        // Text and links are taken (the files-only stopgap is gone).
        let drops = with(2, json!({ "drop": ["text", "links", "files"] }));
        assert!(check_manifest(&drops).is_ok());
        // A secret setting: api 2, no default; the plugin's view of it is null until the app
        // puts in what it decrypts, whatever plain value the fence's settings hold.
        let secret = |api: u32, default: serde_json::Value| {
            manifest(|v| {
                v["api"] = api.into();
                v["settings"] = json!([{ "key": "token", "type": "secret", "default": default }]);
            })
        };
        assert!(check_manifest(&secret(2, json!(null))).is_ok());
        let why = check_manifest(&secret(1, json!(null))).unwrap_err();
        assert!(why.contains("secret settings need api 2"), "{why}");
        assert!(check_manifest(&secret(2, json!("abc"))).is_err());
        let saved = json!({ "token": "plain", SECRETS: { "token": "0a0b" } });
        let shown = fence_settings(&secret(2, json!(null)), saved.as_object().unwrap());
        assert_eq!(shown, json!({ "token": null }));
        // Folders: `true` is read; readwrite needs api 2 and is more than read, read than none.
        let folders = |api: u32, f: serde_json::Value| with(api, json!({ "folders": f }));
        assert_eq!(folders(1, json!(true)).permissions.folders, Folders::Read);
        assert_eq!(folders(1, json!(false)).permissions.folders, Folders::None);
        assert!(check_manifest(&folders(1, json!("read"))).is_ok());
        let why = check_manifest(&folders(1, json!("readwrite"))).unwrap_err();
        assert!(why.contains("folders readwrite need api 2"), "{why}");
        assert!(check_manifest(&folders(2, json!("readwrite"))).is_ok());
        for (from, to, more) in [
            (json!(false), json!("read"), true),
            (json!(true), json!("readwrite"), true),
            (json!("read"), json!("readwrite"), true),
            (json!("readwrite"), json!(true), false),
            (json!("readwrite"), json!("none"), false),
            (json!(true), json!("read"), false),
        ] {
            let got = more_permissions(
                &folders(2, from.clone()).permissions,
                &folders(2, to.clone()).permissions,
            );
            assert_eq!(got.contains(&"folders"), more, "{from} to {to}: {got:?}");
        }
        assert!(Folders::Readwrite.reads() && Folders::Readwrite.writes());
        assert!(Folders::Read.reads() && !Folders::Read.writes() && !Folders::None.reads());
        // An unknown media level does not even read.
        let mut v = entry("countdown");
        v["permissions"] = json!({ "media": "write" });
        assert!(serde_json::from_value::<Manifest>(v.clone()).is_err());
        v["permissions"] = json!({ "folders": "write" });
        assert!(serde_json::from_value::<Manifest>(v).is_err());
    }

    #[test]
    fn the_fields_of_interface_3_need_api_3_and_each_growth_asks() {
        let with = |api: u32, p: serde_json::Value| {
            manifest(|v| {
                v["api"] = api.into();
                v["permissions"] = p;
            })
        };
        let new = [
            json!({ "screen": true }),
            json!({ "pointer": true }),
            json!({ "focus": true }),
            json!({ "camera": true }),
            json!({ "microphone": true }),
            json!({ "location": true }),
            json!({ "webrtc": true }),
        ];
        assert_eq!(new.len(), API3_FIELDS.len());
        for (p, field) in new.iter().zip(API3_FIELDS) {
            let why = check_manifest(&with(2, p.clone())).unwrap_err();
            assert!(why.contains(field) && why.contains("api 3"), "{why}");
            assert!(check_manifest(&with(3, p.clone())).is_ok(), "{p}");
            // PecoFence 0.5.0 (interface 2) shows 需要更新 PecoFence for it, this one installs.
            assert!(with(3, p.clone()).api > 2 && !needs_newer_app(&with(3, p.clone()), "0.5.0"));
            let more = more_permissions(&Permissions::default(), &with(3, p.clone()).permissions);
            assert_eq!(more, [field]);
            let back = more_permissions(&with(3, p.clone()).permissions, &Permissions::default());
            assert!(back.is_empty(), "{back:?}");
        }
        // What interface 2 refused needs api 3 too (and the rules of the field after that).
        for (p, what) in [
            (json!({ "folders": "any" }), "folders any"),
            (
                json!({ "commands": [{ "id": "show", "title": { "en": "Show" }, "key": "Ctrl+Shift+M" }] }),
                "command keys",
            ),
            (
                json!({ "network": ["192.168.1.20"] }),
                "IP or dotless hosts",
            ),
            (json!({ "links": ["nas"] }), "IP or dotless hosts"),
            (
                json!({ "system": ["C:\\Tools\\backup.cmd"] }),
                "scripts in system",
            ),
            (
                json!({ "local": { "loopback": [
                    { "port": 1, "app": "A" }, { "port": 2, "app": "B" }, { "port": 3, "app": "C" },
                    { "port": 4, "app": "D" }, { "port": 5, "app": "E" }] } }),
                "more than 4 local ports",
            ),
        ] {
            let why = check_manifest(&with(2, p.clone())).unwrap_err();
            assert!(why.contains(what) && why.contains("api 3"), "{p}: {why}");
            assert!(with(3, p).permissions.api3_uses().contains(&what));
        }
        assert!(check_manifest(&with(3, json!({ "folders": "any" }))).is_ok());
        assert!(Folders::Any.writes() && Folders::Any > Folders::Readwrite);
        // 0.5.0's own rules, kept to tell what needs api 3.
        assert!(
            host_of_api2("api.github.com") && !host_of_api2("10.0.0.2") && !host_of_api2("nas")
        );
        assert!(program_of_api2("ipconfig.exe") && !program_of_api2("C:\\a\\b.ps1"));
        // Folders up to any asks, as does a command's key added or changed; a key taken away
        // does not.
        let perms = |p: serde_json::Value| -> Permissions { serde_json::from_value(p).unwrap() };
        let rw = perms(json!({ "folders": "readwrite" }));
        assert_eq!(
            more_permissions(&rw, &perms(json!({ "folders": "any" }))),
            ["folders"]
        );
        let keyed = |key: Option<&str>| {
            perms(json!({ "commands": [{ "id": "show", "title": { "en": "Show" }, "key": key }] }))
        };
        assert_eq!(
            more_permissions(&keyed(None), &keyed(Some("Ctrl+Shift+M"))),
            ["commands"]
        );
        assert_eq!(
            more_permissions(&keyed(Some("Ctrl+Shift+M")), &keyed(Some("Ctrl+Shift+N"))),
            ["commands"]
        );
        assert!(more_permissions(&keyed(Some("Ctrl+Shift+M")), &keyed(None)).is_empty());
        assert!(
            more_permissions(&keyed(Some("Win+Shift+M")), &keyed(Some("Win+Shift+M"))).is_empty()
        );
        // An update to interface 3 that asks for more waits for the user.
        let installed = with(2, json!({}));
        let mut offered = with(3, json!({ "screen": true }));
        offered.version = "1.1.0".into();
        assert_eq!(update_kind(&installed, &offered, "0.6.0"), Update::Confirm);
    }

    #[test]
    fn files_over_the_sizes_of_interface_2_need_api_3() {
        let sized = |main: u64, extra: usize| -> Entry {
            let mut v = entry("countdown");
            v["files"][0]["size"] = main.into();
            for i in 0..extra {
                v["files"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({ "name": format!("f{i}.js"), "size": 10, "sha256": HASH }));
            }
            serde_json::from_value(v).unwrap()
        };
        assert!(files_of_api2(&sized(200 * 1024, 0)) && check_entry(&sized(200 * 1024, 0)).is_ok());
        assert!(!files_of_api2(&sized(200 * 1024 + 1, 0)));
        assert!(!files_of_api2(&sized(10, MAX_FILES_API2)));
        assert!(files_of_api2(&sized(10, MAX_FILES_API2 - 3)));
    }

    #[test]
    fn local_ports_are_few_named_and_no_debugger_and_each_new_one_asks() {
        let ha = json!([{ "key": "host", "type": "device", "label": { "en": "Address" } }]);
        let with = |api: u32, local: serde_json::Value, settings: &serde_json::Value| {
            manifest(|v| {
                v["api"] = api.into();
                v["permissions"] = json!({ "local": local });
                v["settings"] = settings.clone();
            })
        };
        let port = |p: u16, app: &str| json!({ "port": p, "app": app });
        let lan = json!({ "lan": [port(8123, "Home Assistant")] });
        assert!(check_manifest(&with(2, lan.clone(), &ha)).is_ok());
        let why = check_manifest(&with(1, lan.clone(), &ha)).unwrap_err();
        assert!(why.contains("local, device settings need api 2"), "{why}");
        // A device setting and `lan` only together; a device has no default.
        let none = json!([]);
        assert!(check_manifest(&with(2, lan.clone(), &none)).is_err());
        let loopback = json!({ "loopback": [port(11434, "Ollama")] });
        assert!(check_manifest(&with(2, loopback.clone(), &none)).is_ok());
        assert!(check_manifest(&with(2, loopback.clone(), &ha)).is_err());
        let mut defaulted = ha.clone();
        defaulted[0]["default"] = "192.168.1.20".into();
        assert!(check_manifest(&with(2, lan.clone(), &defaulted)).is_err());
        for (bad, what) in [
            (
                json!({ "loopback": [port(9222, "Edge")] }),
                "9222 (Chrome and Edge DevTools)",
            ),
            (json!({ "lan": [port(5985, "x")] }), "5985 (WinRM)"),
            (json!({ "loopback": [port(2375, "Docker")] }), "2375"),
            (json!({ "loopback": [port(9229, "Node")] }), "9229"),
            (
                json!({ "loopback": [port(9515, "x")] }),
                "9515 (ChromeDriver)",
            ),
            (
                json!({ "lan": [port(4444, "x")] }),
                "4444 (Selenium WebDriver)",
            ),
            (
                // More than 4 is interface 3's (api 2 here).
                json!({ "loopback": [port(80, "a"), port(81, "b"), port(82, "c"), port(83, "d"), port(84, "e")] }),
                "more than 4 local ports need api 3",
            ),
            (
                json!({ "loopback": [port(80, "a"), port(80, "b")] }),
                "each once",
            ),
            (json!({ "loopback": [port(0, "a")] }), "each once"),
            (json!({ "loopback": [port(80, " ")] }), "its program"),
            (
                json!({ "loopback": [port(80, &"x".repeat(41))] }),
                "its program",
            ),
        ] {
            let settings = if bad["lan"].is_null() { &none } else { &ha };
            let why = check_manifest(&with(2, bad.clone(), settings)).unwrap_err();
            assert!(why.contains(what), "{bad}: {why}");
        }
        // Any number of ports (interface 3); the same port in both classes is two entries; a
        // port out of range does not read.
        let five = json!({ "loopback": [port(80, "a"), port(81, "b"), port(82, "c"), port(83, "d"), port(84, "e")] });
        assert!(check_manifest(&with(3, five, &none)).is_ok());
        let both = json!({ "loopback": [port(8080, "a")], "lan": [port(8080, "a")] });
        assert!(check_manifest(&with(2, both, &ha)).is_ok());
        let mut v = entry("countdown");
        v["permissions"] = json!({ "local": { "loopback": [{ "port": 70000, "app": "a" }] } });
        assert!(serde_json::from_value::<Manifest>(v).is_err());
        // Growth: any class and port not there before asks; moving a port to the other class
        // too; another program name or fewer ports do not.
        let perms = |local: serde_json::Value| -> Permissions {
            serde_json::from_value(json!({ "local": local })).unwrap()
        };
        let old = perms(json!({ "loopback": [port(11434, "Ollama")], "lan": [port(8123, "HA")] }));
        for (new, more) in [
            (
                json!({ "loopback": [port(11434, "Ollama 2")], "lan": [port(8123, "HA")] }),
                false,
            ),
            (json!({ "loopback": [port(11434, "Ollama")] }), false),
            (
                json!({ "loopback": [port(11434, "Ollama"), port(11435, "x")] }),
                true,
            ),
            (
                json!({ "lan": [port(11434, "Ollama"), port(8123, "HA")] }),
                true,
            ),
            (json!({ "loopback": [port(8123, "HA")] }), true),
        ] {
            let found = more_permissions(&old, &perms(new.clone()));
            assert_eq!(found == ["local"], more, "{new}: {found:?}");
            assert!(more || found.is_empty(), "{new}: {found:?}");
        }
        // A device is what the user typed (kept under __devices), never a plain value.
        let m = with(2, lan.clone(), &ha);
        let saved = json!({ "host": "10.0.0.1", DEVICES: { "host": "192.168.1.20" } });
        assert_eq!(
            fence_settings(&m, saved.as_object().unwrap()),
            json!({ "host": "192.168.1.20" })
        );
        let saved = json!({ "host": "10.0.0.1" });
        assert_eq!(
            fence_settings(&m, saved.as_object().unwrap()),
            json!({ "host": null })
        );
    }

    #[test]
    fn network_hosts_of_this_computer_and_the_local_network_go_by_the_local_rules() {
        for (host, class) in [
            ("localhost", Some("loopback")),
            ("127.0.0.1", Some("loopback")),
            ("[::1]", Some("loopback")),
            ("192.168.1.20", Some("lan")),
            ("10.0.0.5", Some("lan")),
            ("100.101.1.2", Some("lan")),
            ("[fd00::1]", Some("lan")),
            ("[fe80::1]", Some("lan")),
            ("nas", Some("lan")),
            ("homeassistant.local", Some("lan")),
            ("router.lan", Some("lan")),
            ("box.home.arpa", Some("lan")),
            ("8.8.8.8", None),
            ("[2606:2800::1]", None),
            ("api.github.com", None),
            ("local.example.com", None),
        ] {
            assert_eq!(network_local_class(host), class, "{host}");
        }
        // A page reaches internet names and public IPv4 itself; the rest only through pf.http.
        assert!(page_reaches("api.github.com") && page_reaches("8.8.8.8"));
        for not in [
            "nas",
            "192.168.1.20",
            "localhost",
            "[2606:2800::1]",
            "x.local",
            "0.0.0.0",
            "224.0.0.251",
            "239.255.255.250",
            "240.0.0.1",
            "255.255.255.255",
        ] {
            assert!(!page_reaches(not), "{not}");
        }
    }

    #[test]
    fn folders_any_writes_and_reads_and_is_more_than_readwrite() {
        let m = manifest(|v| {
            v["api"] = 2.into();
            v["permissions"]["folders"] = "any".into();
        });
        let f = m.permissions.folders;
        assert!(f.reads() && f.writes() && f.any_kind() && f > Folders::Readwrite);
        assert!(!Folders::Readwrite.any_kind());
    }

    #[test]
    fn the_class_of_a_local_address_and_what_a_user_may_type_as_a_device() {
        let p: Permissions = serde_json::from_value(json!({ "local": {
            "loopback": [{ "port": 11434, "app": "Ollama" }],
            "lan": [{ "port": 8123, "app": "HA" }, { "port": 80, "app": "NAS" }],
        } }))
        .unwrap();
        let class =
            |url: &str| local_class(&p, &crate::web_address::WebAddress::parse(url).unwrap());
        assert_eq!(class("http://localhost:11434/api/tags"), Some("loopback"));
        assert_eq!(class("http://127.0.0.1:11434/"), Some("loopback"));
        assert_eq!(class("https://127.9.0.1:11434/"), Some("loopback"));
        assert_eq!(class("http://192.168.1.20:8123/api/states"), Some("lan"));
        assert_eq!(class("http://nas.local/"), Some("lan"));
        // This computer on a lan port is a lan request: the app refuses its address.
        assert_eq!(class("http://127.0.0.1:8123/"), Some("lan"));
        for none in [
            "http://192.168.1.20:11434/",
            "http://localhost:3000/x",
            "https://nas.local/",
            "http://a.com:9222/",
        ] {
            assert_eq!(class(none), None, "{none}");
        }
        assert_eq!(
            device_host(" 192.168.1.20 ").as_deref(),
            Some("192.168.1.20")
        );
        assert_eq!(
            device_host("HomeAssistant.local.").as_deref(),
            Some("homeassistant.local")
        );
        assert_eq!(
            device_host("http://192.168.1.20:8123/lovelace").as_deref(),
            Some("192.168.1.20")
        );
        assert_eq!(device_host("[FE80::1]").as_deref(), Some("[fe80::1]"));
        for bad in ["", "my box", "a@b.local", "[fe80::1%3]", "http://", "a..b"] {
            assert_eq!(device_host(bad), None, "{bad:?}");
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
            v["api"] = (API + 1).into();
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

    #[test]
    fn a_command_brings_a_shortcut_with_shift_or_win_and_pointer_and_focus_read() {
        for good in [
            "Ctrl+Shift+M",
            "Ctrl+Alt+Shift+M",
            "Win+Shift+F9",
            "Alt+Shift+Space",
            "ctrl + shift + f2",
        ] {
            assert!(command_key_ok(good), "{good}");
        }
        // Ctrl+Alt+M is AltGr+M (µ on a German keyboard); no Shift or Win; no modifier; Win
        // alone; Task Manager's Ctrl+Shift+Esc; a key PecoFence does not know; nothing.
        for bad in [
            "Ctrl+Alt+M",
            "Ctrl+M",
            "Alt+F5",
            "Shift+M",
            "M",
            "Win+M",
            "Ctrl+Shift+Esc",
            "Ctrl+Shift+Nope",
            "",
        ] {
            assert!(!command_key_ok(bad), "{bad}");
        }
        let keyed = |key: &str| {
            manifest(|v| {
                v["api"] = API.into();
                v["permissions"] = json!({ "commands": [{ "id": "show", "title": { "en": "Show" }, "key": key }] });
            })
        };
        assert!(check_manifest(&keyed("Ctrl+Shift+M")).is_ok());
        assert_eq!(
            keyed("Ctrl+Shift+M").permissions.commands[0].key.as_deref(),
            Some("Ctrl+Shift+M")
        );
        assert!(check_manifest(&keyed("Ctrl+Alt+M")).is_err());
        let p = manifest(|v| v["permissions"] = json!({ "pointer": true, "focus": true }));
        assert!(p.permissions.pointer && p.permissions.focus);
        assert!(!manifest(|_| {}).permissions.pointer);
    }
}
