//! What a plugin may ask of PecoFence (`pf.http`, `pf.clipboard`, `pf.run`, ...): the check of
//! every call against the plugin's installed manifest, and the calls themselves through the
//! [`Host`] the app (or a checker's fake) passes in. Requests follow redirects here, one hop at
//! a time, so that every host a request reaches is checked and seen.

use pecofence_core::FenceId;
use pecofence_core::plugins::{Permissions, valid_plain_name};
use pecofence_core::web_address::WebAddress;
use serde_json::{Value, json};

/// The most bytes an answer to `pf.http` may have.
pub const MAX_ANSWER: usize = 1024 * 1024;
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
        "http.get" | "http.post" => (
            address.as_ref().is_some_and(|a| a.secure) && host_in(&[&p.network]),
            format!("an https address with {shown} in permissions.network"),
        ),
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
        "folder.pick" | "folder.list" | "folder.read" => (p.folders, "permissions.folders".into()),
        "notify" => (p.notify, "permissions.notify".into()),
        "openSystem" => (arg(0) == "taskmgr", "the name taskmgr".into()),
        "data.list" => (true, String::new()),
        n if n.starts_with("sys.") => (true, String::new()),
        _ => (false, "a pf function of that name".into()),
    };
    (!ok).then(|| format!("{name} needs {need}"))
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

/// `location` (a redirect's target) read against the https address `base`.
fn joined(base: &str, location: &str) -> Option<String> {
    if location.contains("://") {
        return Some(location.into());
    }
    if let Some(rest) = location.strip_prefix("//") {
        return Some(format!("https://{rest}"));
    }
    let rest = base.strip_prefix("https://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (origin, path) = base.split_at(8 + end);
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
    for hop in 0..=MAX_HOPS {
        if hop > 0 {
            let refused = refusal(p, &verb, &json!([url]));
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
            json!([url, { "headers": headers, "body": body }]),
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
        if url_host(&next) != url_host(&url) {
            headers.retain(|k, _| {
                !["authorization", "cookie"].contains(&k.to_ascii_lowercase().as_str())
            });
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
    use pecofence_core::plugins::Clipboard;
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
        assert_eq!(get("https://a.com/1", headers), Ok(ok));
        let calls = fake.calls.lock().unwrap().clone();
        let urls: Vec<&str> = calls.iter().map(|c| c.1[0].as_str().unwrap()).collect();
        assert_eq!(
            urls,
            ["https://a.com/1", "https://a.com/2", "https://b.com/3"]
        );
        // The same host keeps the credentials, another host does not; the agent is PecoFence's.
        let sent = |i: usize| calls[i].1[1]["headers"].clone();
        assert_eq!(sent(1)["Authorization"], "token");
        assert_eq!(
            sent(2),
            json!({ "Accept": "x", "User-Agent": "PecoFence-plugin/t 1.0.0" })
        );
        let why = get("https://a.com/away", json!({})).unwrap_err();
        assert!(why.contains("c.com in permissions.network"), "{why}");
        assert!(
            get("https://a.com/loop", json!({}))
                .unwrap_err()
                .contains("more than 5")
        );
        assert_eq!(fake.calls.lock().unwrap().len(), 3 + 1 + 6);
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
    }
}
