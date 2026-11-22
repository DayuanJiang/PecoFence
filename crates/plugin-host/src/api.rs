//! What a plugin may ask of PecoFence (`pf.http`, `pf.clipboard`, `pf.run`, ...): the check of
//! every call against the plugin's installed manifest, and the calls themselves through the
//! [`Host`] the app (or a checker's fake) passes in. Requests follow redirects here, one hop at
//! a time, so that every host a request reaches is checked and seen.

use pecofence_core::FenceId;
use pecofence_core::plugins::{
    Permissions, REFUSED_PORTS, local_class, network_local_class, valid_plain_name,
};
use pecofence_core::web_address::WebAddress;
use serde_json::{Value, json};

/// The most bytes an answer to `pf.http` may have (a light plugin's answer must also fit
/// `MAX_ANSWER_JSON` in its engine, so this much reaches only a web page).
pub const MAX_ANSWER: usize = 10 * 1024 * 1024;
/// Redirects a request follows at most.
const MAX_HOPS: usize = 5;

/// What only the app can do (Windows, the network). PecoFence implements it with WinHTTP, the
/// clipboard, sysinfo, the data folder, dialogs and its notice; crates/plugin-check and the unit
/// tests with fakes. plugin-host checks the manifest before every call.
pub trait Host: Send + Sync + 'static {
    /// `name` is the pf path ("http.get", "http.post", "clipboard.text", "clipboard.setText",
    /// "sys.cpu", "data.read", "run", "notify", ...), `args` the call's arguments as a JSON array.
    /// Runs on a worker thread; Ok resolves the plugin's Promise, Err rejects it.
    fn call(&self, fence: FenceId, name: &str, args: Value) -> Result<Value, String>;
    /// Every pf call (allowed or refused) and every JavaScript run, for the checker; the app
    /// leaves it empty.
    fn note(&self, _fence: FenceId, _note: Note) {}
    /// Whether [`Self::note`] wants the notes: the memory a run leaves costs a walk over the
    /// runtime's heap, which the app does not pay for.
    fn wants_notes(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    Call {
        name: String,
        args: Value,
        refused: Option<String>,
    },
    /// One start, timer callback, event or answer: JavaScript time and the runtime's memory
    /// afterwards.
    Ran {
        what: String,
        ms: f64,
        memory: usize,
    },
    /// An error escaped a handler, a timer or a Promise in run `what` (`click`, `timer`,
    /// `answer to http.get`, ...): logged, and the plugin goes on. `at` is the first line of
    /// its stack.
    Error {
        what: String,
        message: String,
        at: String,
    },
}

/// Why the manifest `p` does not allow call `name` with `args`, if it does not (the text names
/// what is missing).
pub fn refusal(p: &Permissions, name: &str, args: &Value) -> Option<String> {
    let arg = |i: usize| args.get(i).and_then(Value::as_str).unwrap_or_default();
    let address = WebAddress::parse(arg(0));
    let host = address.as_ref().map(|a| a.host.clone());
    let host_in = |lists: &[&Vec<String>]| {
        let listed = |h: &str| {
            lists
                .iter()
                .any(|l| l.iter().any(|x| x.eq_ignore_ascii_case(h)))
        };
        host.as_deref().is_some_and(listed)
    };
    let shown = host.clone().unwrap_or_else(|| "this address".into());
    let (reads, writes) = (p.clipboard.reads(), p.clipboard.writes());
    let named = |i: usize| args.get(i).is_none_or(Value::is_null) || valid_plain_name(arg(i));
    let (ok, need) = match name {
        "http.get" | "http.post" => {
            let port = address
                .as_ref()
                .map(|a| a.port.unwrap_or(if a.secure { 443 } else { 80 }));
            if let Some((port, what)) = REFUSED_PORTS.iter().find(|(r, _)| Some(*r) == port) {
                return Some(format!("{name}: port {port} ({what}) may not be reached"));
            }
            (
                (address.as_ref()).is_some_and(|a| class_of(p, a).is_some()),
                format!(
                    "an https address with {shown} in permissions.network, or its port in permissions.local"
                ),
            )
        }
        "open" => (
            host_in(&[&p.network, &p.links]),
            format!("an http or https address with {shown} in permissions.network or links"),
        ),
        "clipboard.text" | "clipboard.files" => (reads, "permissions.clipboard read".into()),
        "clipboard.image" => (reads && named(0), "clipboard read and a data name".into()),
        "clipboard.setText" | "clipboard.setFiles" => {
            (writes, "permissions.clipboard write".into())
        }
        "clipboard.setImage" => (
            writes && valid_plain_name(arg(0)),
            "clipboard write and a data name".into(),
        ),
        "data.read" | "data.write" | "data.remove" => (
            valid_plain_name(arg(0)),
            "a name of letters, digits, . _ - (at most 64, no dot first)".into(),
        ),
        "run" => {
            let program = arg(0);
            let declared = p.system.iter().any(|s| s.eq_ignore_ascii_case(program));
            (declared, format!("{program} in permissions.system"))
        }
        "folder.pick" | "folder.list" | "folder.read" => {
            (p.folders.reads(), "permissions.folders".into())
        }
        "folder.write" | "folder.mkdir" | "folder.trash" => (
            p.folders.writes() && args.get(0).is_some_and(Value::is_string),
            "permissions.folders readwrite and a path".into(),
        ),
        "folder.writeBytes" => (
            p.folders.any_kind() && args.get(0).is_some_and(Value::is_string),
            "permissions.folders any, a path and the bytes in base64".into(),
        ),
        "notify" => (p.notify, "permissions.notify".into()),
        n if n.starts_with("launch.") => (
            p.launch && ["pick", "add", "list", "remove", "open"].contains(&&n[7..]),
            "permissions.launch".into(),
        ),
        "media.current" => (p.media.reads(), "permissions.media".into()),
        "media.control" => (
            p.media.controls() && ["playpause", "next", "previous"].contains(&arg(0)),
            "permissions.media control and playpause, next or previous".into(),
        ),
        "apps.running" | "apps.front" => (!p.apps.is_empty(), "permissions.apps".into()),
        "pointer" => (p.pointer, "permissions.pointer".into()),
        "focus" | "caret" => (p.focus, "permissions.focus".into()),
        "drop.read" | "drop.text" | "drop.save" => (
            p.drop.iter().any(|d| d == "files")
                && args.get(0).is_some_and(Value::is_u64)
                && (name != "drop.save" || valid_plain_name(arg(1))),
            "permissions.drop files, a file number and for save a data name".into(),
        ),
        "openSystem" => (arg(0) == "taskmgr", "the name taskmgr".into()),
        "data.list" => (true, String::new()),
        n if n.starts_with("sys.") => (true, String::new()),
        _ => (false, "a pf function of that name".into()),
    };
    (!ok).then(|| format!("{name} needs {need}"))
}

/// A request to the internet: https to a host of `network` that is no host of this computer or
/// the local network.
fn internet(p: &Permissions, a: &WebAddress) -> bool {
    a.secure && declared(p, a) && network_local_class(&a.host).is_none()
}

/// The host of `a` is one of `network`.
fn declared(p: &Permissions, a: &WebAddress) -> bool {
    p.network.iter().any(|h| h.eq_ignore_ascii_case(&a.host))
}

/// Where a request to `a` goes, if the manifest lets it: `internet`, or `loopback` or `lan` by
/// the local rules (never through a proxy, only to an address of that kind), with whether its
/// host is one `network` declared (interface 3: an address or a name of this computer or the
/// local network, which needs no `device` setting) rather than a `local` port.
fn class_of(p: &Permissions, a: &WebAddress) -> Option<(&'static str, bool)> {
    let listed = declared(p, a)
        .then(|| network_local_class(&a.host))
        .flatten();
    if let Some(class) = listed {
        Some((class, true))
    } else if internet(p, a) {
        Some(("internet", true))
    } else {
        local_class(p, a).map(|class| (class, false))
    }
}

/// The host of an http or https address, in lowercase (None when [`WebAddress`] does not read
/// it).
pub fn url_host(url: &str) -> Option<String> {
    WebAddress::parse(url).map(|a| a.host)
}

/// `args` with its first value, an address the check read, written back as it was read: what
/// WinHTTP and the browser get is exactly what was checked.
fn normalised(mut args: Value) -> Value {
    let read = args
        .get(0)
        .and_then(Value::as_str)
        .and_then(WebAddress::parse);
    if let (Some(address), Some(first)) = (read, args.get_mut(0)) {
        *first = address.to_string().into();
    }
    args
}

/// `location` (a redirect's target) read against the http or https address `base`.
fn joined(base: &str, location: &str) -> Option<String> {
    if location.contains("://") {
        return Some(location.into());
    }
    let scheme = if base.starts_with("https://") {
        "https:"
    } else {
        "http:"
    };
    if let Some(rest) = location.strip_prefix("//") {
        return Some(format!("{scheme}//{rest}"));
    }
    let rest = base.strip_prefix(scheme)?.strip_prefix("//")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (origin, path) = base.split_at(scheme.len() + 2 + end);
    if location.starts_with('/') {
        return Some(format!("{origin}{location}"));
    }
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let dir = path.rfind('/').map_or("/", |i| &path[..=i]);
    Some(format!("{origin}{dir}{location}"))
}

/// Does call `name` (allowed already): a request hop by hop, anything else through the host.
/// `agent` is the User-Agent of the plugin's requests. The web layer calls it too.
pub fn perform(
    host: &dyn Host,
    fence: FenceId,
    p: &Permissions,
    agent: &str,
    name: &str,
    args: Value,
) -> Result<Value, String> {
    if name == "open" {
        return host.call(fence, name, normalised(args));
    }
    if !name.starts_with("http.") {
        return host.call(fence, name, args);
    }
    let args = normalised(args);
    let mut url = args
        .get(0)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let options = args.get(1).cloned().unwrap_or(Value::Null);
    let mut headers = options["headers"].as_object().cloned().unwrap_or_default();
    headers.retain(|k, v| {
        v.is_string() && !["user-agent", "host"].contains(&k.to_ascii_lowercase().as_str())
    });
    headers.insert("User-Agent".into(), agent.into());
    let mut body = options.get("body").cloned().unwrap_or(Value::Null);
    let mut verb = name.to_string();
    // Where the first request goes: the internet, `loopback` or `lan`.
    let mut first = None;
    for hop in 0..=MAX_HOPS {
        // A request to `local` says its class: the host connects only to such an address
        // (and, for `lan`, only to a host the user typed), never through a proxy.
        let address = WebAddress::parse(&url);
        let reach = address.as_ref().and_then(|a| class_of(p, a));
        let declared = reach.is_some_and(|(_, d)| d);
        let class = reach.map_or("internet", |(c, _)| c);
        let local = (class != "internet").then_some(class);
        let first = *first.get_or_insert(class);
        if hop > 0 {
            // A redirect stays in its class: from the internet never to this computer or the
            // local network (307 and 308 would take a POST's body along), nor back.
            let refused = refusal(p, &verb, &json!([url])).or_else(|| {
                (class != first).then(|| format!("a redirect from {first} to {class}"))
            });
            let args = json!([url]);
            host.note(
                fence,
                Note::Call {
                    name: verb.clone(),
                    args,
                    refused: refused.clone(),
                },
            );
            if let Some(why) = refused {
                tracing::info!(%fence, url, why, "plugin request: redirect refused");
                return Err(format!("redirect refused: {why}"));
            }
        }
        let answer = host.call(
            fence,
            &verb,
            json!([url, { "headers": headers, "body": body, "local": local, "declared": declared }]),
        )?;
        if answer["body"]
            .as_str()
            .is_some_and(|b| b.len() > MAX_ANSWER)
        {
            return Err(format!("the answer is larger than {MAX_ANSWER} bytes"));
        }
        let status = answer["status"].as_u64().unwrap_or(0);
        let location = answer["headers"]["location"].as_str();
        let Some(location) = location.filter(|_| [301, 302, 303, 307, 308].contains(&status))
        else {
            return Ok(answer);
        };
        let next = joined(&url, location).ok_or_else(|| format!("bad redirect {location}"))?;
        // To another scheme, host or port: none of the plugin's own headers go along.
        let origin = |u: &str| WebAddress::parse(u).map(|a| (a.secure, a.host, a.port));
        if origin(&next) != origin(&url) {
            headers.retain(|k, _| k == "User-Agent");
        }
        if status == 303 || (verb == "http.post" && status < 307) {
            (verb, body) = ("http.get".into(), Value::Null);
        }
        tracing::debug!(%fence, status, from = url, to = next, "plugin request redirected");
        url = WebAddress::parse(&next).map_or(next, |a| a.to_string());
    }
    Err(format!("more than {MAX_HOPS} redirects"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_core::plugins::{Clipboard, Folders, Media};
    use std::sync::Mutex;

    /// Answers requests from `answers` (by address) and remembers each call.
    #[derive(Default)]
    struct Fake {
        answers: Vec<(&'static str, Value)>,
        calls: Mutex<Vec<(String, Value)>>,
    }

    impl Host for Fake {
        fn call(&self, _: FenceId, name: &str, args: Value) -> Result<Value, String> {
            self.calls.lock().unwrap().push((name.into(), args.clone()));
            let url = args[0].as_str().unwrap_or_default();
            let found = self.answers.iter().find(|(u, _)| *u == url);
            found
                .map(|(_, a)| a.clone())
                .ok_or_else(|| format!("no answer for {url}"))
        }
    }

    fn moved(status: u64, to: &str) -> Value {
        json!({ "status": status, "headers": { "location": to }, "body": "" })
    }

    fn perms(network: &[&str]) -> Permissions {
        Permissions {
            network: network.iter().map(|h| h.to_string()).collect(),
            ..Permissions::default()
        }
    }

    #[test]
    fn every_call_is_checked_against_the_manifest() {
        let p = Permissions {
            network: vec!["api.github.com".into()],
            links: vec!["github.com".into()],
            clipboard: Clipboard::Write,
            system: vec!["ipconfig".into()],
            ..Permissions::default()
        };
        let refused = |name: &str, args: Value| refusal(&p, name, &args);
        assert_eq!(
            refused("http.get", json!(["https://api.github.com/zen"])),
            None
        );
        let why = refused("http.get", json!(["https://example.com/"])).unwrap();
        assert!(why.contains("example.com in permissions.network"), "{why}");
        assert!(refused("http.get", json!(["http://api.github.com/"])).is_some());
        assert!(refused("http.get", json!(["https://evil.com@api.github.com.x/"])).is_some());
        // Where the browser or WinHTTP would read another host than the check did.
        for bad in [
            "https://evil.example\u{0}@api.github.com/",
            "https://evil.com\\@api.github.com/",
            "https://evil.com@api.github.com/",
            "https://api.github.com\\evil.com/",
            "https://api.git\thub.com/",
        ] {
            assert!(refused("http.get", json!([bad])).is_some(), "{bad:?}");
            assert!(
                refused("open", json!([bad.replace("api.", "")])).is_some(),
                "{bad:?}"
            );
        }
        assert_eq!(refused("open", json!(["https://github.com/x"])), None);
        assert!(refused("open", json!(["file:///C:/Windows"])).is_some());
        assert_eq!(refused("clipboard.setText", json!(["a"])), None);
        assert!(
            refused("clipboard.text", json!([]))
                .unwrap()
                .contains("clipboard read")
        );
        assert_eq!(refused("run", json!(["IPCONFIG", ["/all"]])), None);
        assert!(
            refused("run", json!(["cmd.exe", []]))
                .unwrap()
                .contains("permissions.system")
        );
        assert!(
            refused("folder.pick", json!([]))
                .unwrap()
                .contains("permissions.folders")
        );
        assert!(
            refused("notify", json!(["a", "b"]))
                .unwrap()
                .contains("permissions.notify")
        );
        assert_eq!(refused("openSystem", json!(["taskmgr"])), None);
        assert!(refused("openSystem", json!(["regedit"])).is_some());
        assert_eq!(refused("sys.battery", json!([])), None);
        assert!(refused("fetch", json!([])).is_some());
        assert!(
            refused("media.current", json!([]))
                .unwrap()
                .contains("permissions.media")
        );
        assert!(
            refused("apps.running", json!([]))
                .unwrap()
                .contains("permissions.apps")
        );
        // Reading the picked folders is not writing them: write, mkdir and trash need readwrite.
        let folders = |f: Folders| Permissions {
            folders: f,
            ..Permissions::default()
        };
        let (read, both) = (folders(Folders::Read), folders(Folders::Readwrite));
        let path = json!([r"C:\Notes\a.md", "text"]);
        for call in ["folder.write", "folder.mkdir", "folder.trash"] {
            let why = refusal(&read, call, &path).unwrap();
            assert!(why.contains("permissions.folders readwrite"), "{why}");
            assert_eq!(refusal(&both, call, &path), None);
            assert!(refusal(&both, call, &json!([3])).is_some());
        }
        assert_eq!(refusal(&read, "folder.read", &path), None);
        assert_eq!(refusal(&both, "folder.pick", &json!([])), None);
        let media = |m: Media| Permissions {
            media: m,
            apps: vec!["spotify.exe".into()],
            ..Permissions::default()
        };
        let read = media(Media::Read);
        assert_eq!(refusal(&read, "media.current", &json!([])), None);
        assert_eq!(refusal(&read, "apps.running", &json!([])), None);
        assert!(refusal(&read, "media.control", &json!(["next"])).is_some());
        let control = media(Media::Control);
        assert_eq!(
            refusal(&control, "media.control", &json!(["playpause"])),
            None
        );
        assert!(refusal(&control, "media.control", &json!(["seek"])).is_some());
        // Interface 3: the pointer, and the focus with the caret, each by its own permission.
        let seeing = |pointer: bool, focus: bool| Permissions {
            pointer,
            focus,
            ..Permissions::default()
        };
        assert!(
            refused("pointer", json!([]))
                .unwrap()
                .contains("permissions.pointer")
        );
        assert!(
            refused("caret", json!([]))
                .unwrap()
                .contains("permissions.focus")
        );
        assert_eq!(refusal(&seeing(true, false), "pointer", &json!([])), None);
        assert!(refusal(&seeing(true, false), "focus", &json!([])).is_some());
        for call in ["focus", "caret"] {
            assert_eq!(refusal(&seeing(false, true), call, &json!([])), None);
        }
        assert!(refusal(&seeing(false, true), "pointer", &json!([])).is_some());
        for bad in ["", ".hidden", "a/b", "..", "x".repeat(65).as_str(), "名字"] {
            assert!(
                refused("data.write", json!([bad, "x"])).is_some(),
                "{bad:?}"
            );
        }
        assert_eq!(refused("data.write", json!(["history-1.json", "x"])), None);
        assert!(refused("clipboard.setImage", json!(["../x.png"])).is_some());
        assert!(refused("clipboard.setImage", json!([])).is_some());
    }

    #[test]
    fn redirects_are_followed_hop_by_hop_and_each_host_is_checked() {
        let fence = FenceId::new_v4();
        let ok = json!({ "status": 200, "headers": {}, "body": "done" });
        let fake = Fake {
            answers: vec![
                ("https://a.com/1", moved(302, "/2")),
                ("https://a.com/2", moved(301, "https://b.com/3")),
                ("https://b.com/3", ok.clone()),
                ("https://a.com/away", moved(302, "https://c.com/")),
                ("https://a.com/loop", moved(307, "loop")),
                ("https://a.com/port", moved(302, "https://a.com:8443/p")),
                ("https://a.com:8443/p", ok.clone()),
            ],
            ..Fake::default()
        };
        let p = perms(&["a.com", "b.com"]);
        let get = |url: &str, headers: Value| {
            let args = json!([url, { "headers": headers }]);
            perform(
                &fake,
                fence,
                &p,
                "PecoFence-plugin/t 1.0.0",
                "http.get",
                args,
            )
        };
        let headers =
            json!({ "Authorization": "token", "Cookie": "c", "Accept": "x", "User-Agent": "mine" });
        assert_eq!(get("https://a.com/1", headers), Ok(ok.clone()));
        let calls = fake.calls.lock().unwrap().clone();
        let urls: Vec<&str> = calls.iter().map(|c| c.1[0].as_str().unwrap()).collect();
        assert_eq!(
            urls,
            ["https://a.com/1", "https://a.com/2", "https://b.com/3"]
        );
        // The same origin keeps the plugin's headers; another host or port gets none of them,
        // only PecoFence's agent.
        let sent = |i: usize| calls[i].1[1]["headers"].clone();
        assert_eq!(sent(1)["Authorization"], "token");
        assert_eq!(sent(1)["Accept"], "x");
        let agent_only = json!({ "User-Agent": "PecoFence-plugin/t 1.0.0" });
        assert_eq!(sent(2), agent_only);
        let headers = json!({ "Authorization": "token", "X-Key": "k" });
        assert_eq!(get("https://a.com/port", headers), Ok(ok.clone()));
        let calls = fake.calls.lock().unwrap().clone();
        assert_eq!(calls[4].1[1]["headers"], agent_only, "{:?}", calls[4]);
        let why = get("https://a.com/away", json!({})).unwrap_err();
        assert!(why.contains("c.com in permissions.network"), "{why}");
        assert!(
            get("https://a.com/loop", json!({}))
                .unwrap_err()
                .contains("more than 5")
        );
        assert_eq!(fake.calls.lock().unwrap().len(), 3 + 2 + 1 + 6);
    }

    #[test]
    fn the_host_gets_the_address_as_it_was_checked() {
        let fence = FenceId::new_v4();
        let ok = json!({ "status": 200, "headers": {}, "body": "" });
        let fake = Fake {
            answers: vec![("https://a.com/x?y", ok.clone()), ("https://b.com/", ok)],
            ..Fake::default()
        };
        let p = Permissions {
            links: vec!["b.com".into()],
            ..perms(&["a.com"])
        };
        let run = |name: &str, url: &str| {
            let _ = perform(&fake, fence, &p, "ua", name, json!([url]));
        };
        run("http.get", "https://A.com./x?y");
        run("open", "https://B.COM");
        let calls = fake.calls.lock().unwrap();
        let urls: Vec<&str> = calls.iter().map(|c| c.1[0].as_str().unwrap()).collect();
        assert_eq!(urls, ["https://a.com/x?y", "https://b.com/"]);
    }

    #[test]
    fn a_post_redirected_by_303_becomes_a_get_without_its_body() {
        let fence = FenceId::new_v4();
        let ok = json!({ "status": 200, "headers": {}, "body": "" });
        let fake = Fake {
            answers: vec![
                ("https://a.com/form", moved(303, "/done")),
                ("https://a.com/done", ok),
            ],
            ..Fake::default()
        };
        let args = json!(["https://a.com/form", { "body": "x=1" }]);
        perform(&fake, fence, &perms(&["a.com"]), "ua", "http.post", args).unwrap();
        let calls = fake.calls.lock().unwrap();
        assert_eq!(
            (calls[0].0.as_str(), calls[0].1[1]["body"].clone()),
            ("http.post", json!("x=1"))
        );
        assert_eq!(
            (calls[1].0.as_str(), calls[1].1[1]["body"].clone()),
            ("http.get", Value::Null)
        );
    }

    #[test]
    fn local_requests_go_by_port_and_say_their_class() {
        let p: Permissions = serde_json::from_value(json!({
            "network": ["a.com"],
            "local": { "loopback": [{ "port": 11434, "app": "Ollama" }], "lan": [{ "port": 8123, "app": "HA" }] },
        }))
        .unwrap();
        for ok in [
            "http://localhost:11434/api/tags",
            "http://127.0.0.1:11434/api/generate",
            "http://192.168.1.20:8123/api/states",
            "https://homeassistant.local:8123/",
            "https://a.com/",
        ] {
            assert_eq!(refusal(&p, "http.get", &json!([ok])), None, "{ok}");
        }
        for bad in [
            "http://localhost:11435/",
            "http://192.168.1.20:11434/",
            "http://a.com/",
            "http://localhost/",
        ] {
            let why = refusal(&p, "http.post", &json!([bad])).unwrap();
            assert!(why.contains("permissions.local"), "{bad}: {why}");
        }
        assert!(refusal(&p, "open", &json!(["http://localhost:11434/"])).is_some());
        let fence = FenceId::new_v4();
        let ok = json!({ "status": 200, "headers": {}, "body": "" });
        let fake = Fake {
            answers: vec![
                ("http://localhost:11434/api/tags", ok.clone()),
                ("http://192.168.1.20:8123/x", moved(302, "/api/states")),
                ("http://192.168.1.20:8123/api/states", ok.clone()),
                (
                    "http://192.168.1.20:8123/out",
                    moved(302, "http://192.168.1.20:22/"),
                ),
                ("https://a.com/", ok),
            ],
            ..Fake::default()
        };
        let get = |url: &str| perform(&fake, fence, &p, "ua", "http.get", json!([url]));
        get("http://localhost:11434/api/tags").unwrap();
        get("http://192.168.1.20:8123/x").unwrap();
        get("https://a.com/").unwrap();
        let why = get("http://192.168.1.20:8123/out").unwrap_err();
        assert!(why.contains("redirect refused"), "{why}");
        let calls = fake.calls.lock().unwrap();
        let sent: Vec<(&str, &Value)> = (calls.iter())
            .map(|c| (c.1[0].as_str().unwrap(), &c.1[1]["local"]))
            .collect();
        assert_eq!(
            sent,
            [
                ("http://localhost:11434/api/tags", &json!("loopback")),
                ("http://192.168.1.20:8123/x", &json!("lan")),
                ("http://192.168.1.20:8123/api/states", &json!("lan")),
                ("https://a.com/", &Value::Null),
                ("http://192.168.1.20:8123/out", &json!("lan")),
            ]
        );
    }

    #[test]
    fn a_redirect_never_leaves_the_class_of_the_first_request() {
        let p: Permissions = serde_json::from_value(json!({
            "network": ["a.com"],
            "local": { "loopback": [{ "port": 11434, "app": "Ollama" }], "lan": [{ "port": 8123, "app": "HA" }] },
        }))
        .unwrap();
        let ok = json!({ "status": 200, "headers": {}, "body": "" });
        let fake = Fake {
            answers: vec![
                (
                    "https://a.com/in",
                    moved(308, "http://localhost:11434/api/pull"),
                ),
                (
                    "https://a.com/lan",
                    moved(307, "http://192.168.1.20:8123/api"),
                ),
                (
                    "http://192.168.1.20:8123/out",
                    moved(302, "https://a.com/x"),
                ),
                (
                    "http://localhost:11434/up",
                    moved(302, "http://192.168.1.20:8123/"),
                ),
                ("http://localhost:11434/a", moved(302, "/b")),
                ("http://localhost:11434/b", ok.clone()),
            ],
            ..Fake::default()
        };
        let fence = FenceId::new_v4();
        let post = |url: &str| {
            let args = json!([url, { "body": "secret" }]);
            perform(&fake, fence, &p, "ua", "http.post", args)
        };
        for (from, to) in [
            ("https://a.com/in", "internet to loopback"),
            ("https://a.com/lan", "internet to lan"),
            ("http://192.168.1.20:8123/out", "lan to internet"),
            ("http://localhost:11434/up", "loopback to lan"),
        ] {
            let why = post(from).unwrap_err();
            assert!(
                why.contains(&format!("a redirect from {to}")),
                "{from}: {why}"
            );
        }
        assert_eq!(post("http://localhost:11434/a"), Ok(ok));
        // Only the first requests and the one redirect within its class went out.
        let calls = fake.calls.lock().unwrap();
        let urls: Vec<&str> = calls.iter().map(|c| c.1[0].as_str().unwrap()).collect();
        assert_eq!(
            urls,
            [
                "https://a.com/in",
                "https://a.com/lan",
                "http://192.168.1.20:8123/out",
                "http://localhost:11434/up",
                "http://localhost:11434/a",
                "http://localhost:11434/b",
            ]
        );
    }

    #[test]
    fn network_addresses_and_short_names_go_by_the_local_rules_without_a_device() {
        let p = perms(&[
            "nas",
            "192.168.1.20",
            "[fd00::1]",
            "8.8.8.8",
            "localhost",
            "a.com",
        ]);
        let ok = json!({ "status": 200, "headers": {}, "body": "" });
        let fake = Fake {
            answers: vec![
                ("http://nas/x", ok.clone()),
                ("http://192.168.1.20:8123/api", ok.clone()),
                ("https://[fd00::1]/", ok.clone()),
                ("https://8.8.8.8/", ok.clone()),
                ("http://localhost:11434/api", ok.clone()),
                ("https://a.com/", ok.clone()),
                ("http://nas/away", moved(302, "https://a.com/")),
            ],
            ..Fake::default()
        };
        let fence = FenceId::new_v4();
        let get = |url: &str| perform(&fake, fence, &p, "ua", "http.get", json!([url]));
        for url in [
            "http://nas/x",
            "http://192.168.1.20:8123/api",
            "https://[fd00::1]/",
            "https://8.8.8.8/",
            "http://localhost:11434/api",
            "https://a.com/",
        ] {
            assert_eq!(refusal(&p, "http.get", &json!([url])), None, "{url}");
            assert_eq!(get(url), Ok(ok.clone()), "{url}");
        }
        // Plain http to an internet host stays refused; a redirect keeps its class.
        assert!(refusal(&p, "http.get", &json!(["http://8.8.8.8/"])).is_some());
        let why = get("http://nas/away").unwrap_err();
        assert!(why.contains("a redirect from lan to internet"), "{why}");
        let calls = fake.calls.lock().unwrap();
        let sent = |url: &str| {
            calls
                .iter()
                .find(|c| c.1[0] == url)
                .map(|c| c.1[1].clone())
                .unwrap()
        };
        for (url, local) in [
            ("http://nas/x", json!("lan")),
            ("http://192.168.1.20:8123/api", json!("lan")),
            ("https://[fd00::1]/", json!("lan")),
            ("http://localhost:11434/api", json!("loopback")),
            ("https://8.8.8.8/", Value::Null),
            ("https://a.com/", Value::Null),
        ] {
            assert_eq!(sent(url)["local"], local, "{url}");
            assert_eq!(sent(url)["declared"], true, "{url}");
        }
    }

    #[test]
    fn a_request_to_a_refused_port_is_refused_whatever_its_host() {
        let p: Permissions = serde_json::from_value(json!({
            "network": ["localhost", "127.0.0.1", "nas", "a.com"],
            "local": { "loopback": [{ "port": 8080, "app": "x" }] },
        }))
        .unwrap();
        for url in [
            "https://127.0.0.1:6443/",
            "https://localhost:2376/",
            "http://nas:3389/",
            "https://a.com:5986/",
            "https://a.com:9222/json",
        ] {
            let why = refusal(&p, "http.get", &json!([url])).unwrap_or_default();
            assert!(why.contains("may not be reached"), "{url}: {why}");
        }
        assert_eq!(
            refusal(&p, "http.get", &json!(["https://a.com:8443/"])),
            None
        );
    }

    #[test]
    fn folders_readwrite_writes_data_files_and_any_writes_any_file_and_bytes() {
        let with = |folders: Folders| Permissions {
            folders,
            ..Permissions::default()
        };
        let (both, any) = (with(Folders::Readwrite), with(Folders::Any));
        let refused =
            |p: &Permissions, name: &str, path: &str| refusal(p, name, &json!([path, "AAAA"]));
        let path = r"C:\Users\me\Documents\x";
        // The kinds of files readwrite writes are the app's check, after the path's (light_host).
        assert_eq!(refused(&both, "folder.write", &format!("{path}.zip")), None);
        let why = refused(&both, "folder.writeBytes", &format!("{path}.png")).unwrap();
        assert!(why.contains("permissions.folders any"), "{why}");
        for name in [
            "folder.write",
            "folder.writeBytes",
            "folder.mkdir",
            "folder.trash",
        ] {
            assert_eq!(refused(&any, name, &format!("{path}.exe")), None, "{name}");
        }
    }

    #[test]
    fn relative_redirects() {
        let base = "https://a.com/x/y?q=1";
        assert_eq!(joined(base, "z").as_deref(), Some("https://a.com/x/z"));
        assert_eq!(joined(base, "/z").as_deref(), Some("https://a.com/z"));
        assert_eq!(
            joined(base, "//b.com/z").as_deref(),
            Some("https://b.com/z")
        );
        assert_eq!(
            joined("https://a.com", "z").as_deref(),
            Some("https://a.com/z")
        );
        let local = "http://192.168.1.20:8123/a/b";
        assert_eq!(
            joined(local, "c").as_deref(),
            Some("http://192.168.1.20:8123/a/c")
        );
        assert_eq!(
            joined(local, "//nas.local/").as_deref(),
            Some("http://nas.local/")
        );
    }
}
