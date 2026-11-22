//! HTTPS GET and POST through WinHTTP (the system proxy and Windows' TLS), so the updater and
//! the usage statistics need no HTTP crate. Blocking: call from a worker thread.

use crate::wide::to_wide;
use core::ffi::c_void;
use pecofence_core::web_address::{WebAddress, lan_ip, loopback_ip, public_ip, read_ip};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::time::{Duration, Instant};
use windows_core::PCWSTR;

windows_core::link!("winhttp.dll" "system" fn WinHttpOpen(agent: PCWSTR, access: u32, proxy: PCWSTR, bypass: PCWSTR, flags: u32) -> *mut c_void);
windows_core::link!("winhttp.dll" "system" fn WinHttpConnect(session: *mut c_void, server: PCWSTR, port: u16, reserved: u32) -> *mut c_void);
windows_core::link!("winhttp.dll" "system" fn WinHttpOpenRequest(connect: *mut c_void, verb: PCWSTR, object: PCWSTR, version: PCWSTR, referrer: PCWSTR, accept: *const PCWSTR, flags: u32) -> *mut c_void);
windows_core::link!("winhttp.dll" "system" fn WinHttpSetTimeouts(handle: *mut c_void, resolve: i32, connect: i32, send: i32, receive: i32) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpSendRequest(request: *mut c_void, headers: PCWSTR, headers_len: u32, optional: *const c_void, optional_len: u32, total_len: u32, context: usize) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpReceiveResponse(request: *mut c_void, reserved: *mut c_void) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpQueryHeaders(request: *mut c_void, info: u32, name: PCWSTR, buffer: *mut c_void, len: *mut u32, index: *mut u32) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpReadData(request: *mut c_void, buffer: *mut c_void, to_read: u32, read: *mut u32) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpCloseHandle(handle: *mut c_void) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpSetOption(handle: *mut c_void, option: u32, buffer: *const c_void, len: u32) -> i32);
windows_core::link!("winhttp.dll" "system" fn WinHttpSetStatusCallback(handle: *mut c_void, callback: StatusCallback, flags: u32, reserved: usize) -> isize);
type StatusCallback = unsafe extern "system" fn(*mut c_void, usize, u32, *mut c_void, u32);

const WINHTTP_ACCESS_TYPE_NO_PROXY: u32 = 1;
const WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY: u32 = 4;
const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
const WINHTTP_QUERY_RAW_HEADERS_CRLF: u32 = 22;
const WINHTTP_OPTION_REDIRECT_POLICY: u32 = 88;
const WINHTTP_OPTION_REDIRECT_POLICY_NEVER: u32 = 0;
const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;
const WINHTTP_CALLBACK_STATUS_RESOLVING_NAME: u32 = 0x1;
const WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER: u32 = 0x4;
/// Resolving a name (0x1, 0x2) and connecting (0x4, 0x8).
const WINHTTP_CALLBACK_FLAGS: u32 = 0xF;
/// WinHTTP's timeouts apply per operation; a server trickling bytes still has to finish.
const TOTAL_TIME: Duration = Duration::from_secs(600);

/// The calling thread's last error; WinHTTP's own codes (12xxx) have no system text, so
/// `Display` shows the code.
fn last_error(what: &str) -> String {
    format!("{what}: {}", windows_core::Error::from_thread())
}

struct Handle(*mut c_void);

impl Handle {
    fn new(raw: *mut c_void, what: &str) -> Result<Self, String> {
        if raw.is_null() {
            Err(last_error(what))
        } else {
            Ok(Self(raw))
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: a handle WinHTTP returned and nobody else closes.
        unsafe { WinHttpCloseHandle(self.0) };
    }
}

/// Where a request may connect to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reach {
    /// PecoFence's own requests (the updater, the statistics): not checked.
    App,
    /// A plugin's request to the internet: public addresses only, through the system proxy.
    Internet,
    /// A plugin's request to `local`: this computer (`Loopback`) or the device the user typed
    /// (`Lan`); only an address its name resolved to when it was checked, never through a
    /// proxy.
    Loopback,
    Lan,
}

impl Reach {
    fn fits(self, ip: &str) -> bool {
        match self {
            Self::App => true,
            Self::Internet => public_ip(ip),
            Self::Loopback => loopback_ip(ip),
            Self::Lan => lan_ip(ip),
        }
    }
}

/// A request handle. For a plugin's request WinHTTP's status callback (`on_status`) checks the
/// address each connection goes to and closes the handle itself when it refuses one.
struct Request {
    handle: *mut c_void,
    host: String,
    reach: Reach,
    /// A `local` request connects only to these, the addresses of its kind its name resolved to
    /// when it was checked (plain http to the one it was sent to).
    pinned: Vec<std::net::IpAddr>,
    /// The name WinHTTP resolved last is not the host: it connects to a proxy.
    proxy: AtomicBool,
    /// The address a connection was refused for.
    refused: OnceLock<String>,
}

impl Drop for Request {
    fn drop(&mut self) {
        if self.refused.get().is_none() {
            // SAFETY: a handle WinHTTP returned that the status callback did not close.
            unsafe { WinHttpCloseHandle(self.handle) };
        }
    }
}

/// A plugin's request connects only to public addresses: a host it declared may resolve, or be
/// made to resolve, to this computer or the local network (`public_ip`). Through a proxy the
/// address is the proxy's, which the user set up, and the proxy resolves the host. `request`
/// looks the host up first and refuses before any connection; this checks the address WinHTTP
/// really connects to (a name whose answer changed since). Closing here cancels the request
/// before it sends a byte, though the connection to that address may already be opened.
unsafe extern "system" fn on_status(
    _: *mut c_void,
    context: usize,
    status: u32,
    info: *mut c_void,
    _: u32,
) {
    // SAFETY: `context` is the `Request` that `request` passed and keeps alive during its calls;
    // `info` is a NUL-terminated wide string for these two statuses.
    let Some(request) = (unsafe { (context as *const Request).as_ref() }) else {
        return;
    };
    if info.is_null() {
        return;
    }
    let text = unsafe { PCWSTR(info as *const u16).to_string() }.unwrap_or_default();
    // Only a request to the internet may go through a proxy (a local one has none).
    if status == WINHTTP_CALLBACK_STATUS_RESOLVING_NAME && request.reach == Reach::Internet {
        let proxy = !text.eq_ignore_ascii_case(&request.host);
        request.proxy.store(proxy, Relaxed);
        if proxy {
            tracing::debug!(host = %request.host, proxy = %text, "plugin request goes through a proxy; its address is not checked");
        }
    } else if status == WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER && !request.proxy.load(Relaxed)
    {
        let pinned = |ip| request.pinned.is_empty() || request.pinned.contains(&ip);
        let fits = request.reach.fits(&text) && read_ip(&text).is_some_and(pinned);
        let reach = request.reach;
        tracing::debug!(host = %request.host, ip = %text, ?reach, fits, "plugin request connects");
        if !fits {
            tracing::info!(host = %request.host, ip = %text, ?reach, "plugin request refused: the address it connects to is not of its kind");
            if request.refused.set(text).is_ok() {
                // SAFETY: the request's handle, closed once; closing cancels the request.
                unsafe { WinHttpCloseHandle(request.handle) };
            }
        }
    }
}

fn refusal(host: &str, ip: &str, reach: Reach) -> String {
    let what = match reach {
        Reach::Loopback => "not an address of this computer",
        Reach::Lan => "not an address of the local network",
        _ => "an address of this computer or the local network",
    };
    format!("{host} resolves to {ip}, {what}: refused")
}

fn check(ok: i32, what: &str) -> Result<(), String> {
    if ok != 0 {
        Ok(())
    } else {
        Err(last_error(what))
    }
}

/// `https://host[:port]/path`; plain `http://` only for local test servers and a plugin's
/// `local` requests. Read by [`WebAddress`], the reading the plugin host checks a plugin's
/// address with.
fn split_url(url: &str, reach: Reach) -> Result<(bool, String, u16, String), String> {
    let a = WebAddress::parse(url).ok_or_else(|| format!("unsupported address {url}"))?;
    let local = matches!(reach, Reach::Loopback | Reach::Lan);
    if !a.secure && a.host != "127.0.0.1" && !local {
        return Err(format!("unsupported address {url}"));
    }
    let port = a.port.unwrap_or(if a.secure { 443 } else { 80 });
    Ok((a.secure, a.host, port, a.path))
}

/// GETs `url` (following redirects) and returns the body of a 200 response, at most `limit`
/// bytes. `headers` are extra request lines such as `Accept: …`, separated by `\r\n`.
pub fn get(url: &str, user_agent: &str, headers: &str, limit: usize) -> Result<Vec<u8>, String> {
    match request("GET", url, user_agent, headers, &[], limit, Reach::App)? {
        (200, _, body) => Ok(body),
        (status, ..) => Err(format!("HTTP {status} from {url}")),
    }
}

/// A plugin's request (`pf.http`): sends `verb` and returns the status, the response's header
/// lines and its body (whatever the status, at most `limit` bytes). A redirect is returned as
/// it is: the plugin host checks its target before it follows it.
pub fn fetch(
    verb: &str,
    url: &str,
    user_agent: &str,
    headers: &str,
    body: &[u8],
    limit: usize,
) -> Result<(u32, String, Vec<u8>), String> {
    request(verb, url, user_agent, headers, body, limit, Reach::Internet)
}

/// A plugin's request to `permissions.local`, as [`fetch`] but http or https, to this computer
/// (127/8, ::1) or with `lan` to the local network (the host the user typed; the app checked
/// that), never through a proxy. The name is resolved here and the request goes to the first
/// address of that kind (IPv4 first), named in the Host header as the plugin asked.
pub fn fetch_local(
    verb: &str,
    url: &str,
    user_agent: &str,
    headers: &str,
    body: &[u8],
    limit: usize,
    lan: bool,
) -> Result<(u32, String, Vec<u8>), String> {
    let reach = if lan { Reach::Lan } else { Reach::Loopback };
    request(verb, url, user_agent, headers, body, limit, reach)
}

/// POSTs `body` to `url` and returns the HTTP status of the answer. `headers` as for [`get`].
/// An error is a request that got no answer.
pub fn post(url: &str, user_agent: &str, headers: &str, body: &[u8]) -> Result<u32, String> {
    request(
        "POST",
        url,
        user_agent,
        headers,
        body,
        64 * 1024,
        Reach::App,
    )
    .map(|(status, ..)| status)
}

/// The address a `local` request to `name` connects to: the first it resolves to that is of
/// `reach` (IPv4 first: a service on this computer often listens on 127.0.0.1 alone), as
/// WinHTTP takes it (an IPv6 address in brackets, with its zone), and all of that kind it
/// resolved to (where an https request, which WinHTTP resolves again, may connect).
fn local_address(
    name: &str,
    port: u16,
    reach: Reach,
) -> Result<(String, Vec<std::net::IpAddr>), String> {
    use std::net::{SocketAddr, ToSocketAddrs};
    let found: Vec<SocketAddr> = (name, port)
        .to_socket_addrs()
        .map_err(|e| format!("{name} could not be resolved: {e}"))?
        .collect();
    let chosen = (found.iter())
        .filter(|a| reach.fits(&a.ip().to_string()))
        .min_by_key(|a| a.is_ipv6());
    tracing::info!(
        host = name,
        ?reach,
        ?found,
        ?chosen,
        "plugin local request: the name resolved"
    );
    let fitting = (found.iter())
        .filter_map(|a| read_ip(&a.to_string()))
        .filter(|ip| reach.fits(&ip.to_string()));
    let fitting = fitting.collect();
    match chosen {
        Some(SocketAddr::V4(a)) => Ok((a.ip().to_string(), fitting)),
        Some(SocketAddr::V6(a)) if a.scope_id() != 0 => {
            Ok((format!("[{}%{}]", a.ip(), a.scope_id()), fitting))
        }
        Some(SocketAddr::V6(a)) => Ok((format!("[{}]", a.ip()), fitting)),
        None => {
            let first = found
                .first()
                .map_or("no address".into(), |a| a.ip().to_string());
            Err(refusal(name, &first, reach))
        }
    }
}

/// The SHA-256 of `data` in lowercase hex, for a header that carries the hash of a body.
pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The SHA-256 of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use crate::bindings::{BCRYPT_SHA256_ALG_HANDLE, BCryptHash};
    let mut hash = [0u8; 32];
    // SAFETY: `hash` is a live buffer of the length passed, the length of a SHA-256. Should
    // Windows refuse, the zeros left in it are a hash the receiver turns down.
    let _ = unsafe {
        BCryptHash(
            BCRYPT_SHA256_ALG_HANDLE,
            None,
            data,
            hash.as_mut_ptr(),
            hash.len() as u32,
        )
    };
    hash
}

/// Sends `verb` for `url` with `body`; returns the status and, for a 200, the response's body
/// (at most `limit` bytes). A plugin's request (`reach` not `App`): no redirect is followed, the
/// header lines come back and so does the body of any status.
fn request(
    verb: &str,
    url: &str,
    user_agent: &str,
    headers: &str,
    body: &[u8],
    limit: usize,
    reach: Reach,
) -> Result<(u32, String, Vec<u8>), String> {
    let started = Instant::now();
    let plugin = reach != Reach::App;
    let (secure, name, port, path) = split_url(url, reach)?;
    let (mut server, mut headers) = (name.clone(), headers.to_string());
    let mut pinned = Vec::new();
    // An IPv6 host is `[...]` in an address and bare for the resolver.
    let lookup = name.trim_start_matches('[').trim_end_matches(']');
    if reach == Reach::Internet {
        use std::net::ToSocketAddrs;
        let found = (lookup, port).to_socket_addrs().into_iter().flatten();
        if let Some(ip) = found.map(|a| a.ip().to_string()).find(|ip| !public_ip(ip)) {
            tracing::info!(host = %name, %ip, "plugin request refused before connecting: the host resolves to this computer or the local network");
            return Err(refusal(&name, &ip, reach));
        }
    } else if plugin {
        let (ip, fitting) = local_address(lookup, port, reach)?;
        // Plain http goes to the address checked here; https by name, whose certificate names
        // it, and `on_status` lets it connect only to an address checked here.
        pinned = if secure {
            fitting
        } else {
            read_ip(&ip).into_iter().collect()
        };
        if !secure {
            let host = if port == 80 {
                name.clone()
            } else {
                format!("{name}:{port}")
            };
            headers = [format!("Host: {host}"), headers].join("\r\n");
            server = ip;
        }
    }
    let headers = headers.trim_end_matches("\r\n");
    let (agent, host, path) = (to_wide(user_agent), to_wide(&server), to_wide(&path));
    let verb = to_wide(verb);
    let length = u32::try_from(body.len()).map_err(|_| "the request body is too large")?;
    let data = if body.is_empty() {
        core::ptr::null()
    } else {
        body.as_ptr().cast()
    };
    // A request to `local` never goes through the system proxy (Clash and the like).
    let access = match reach {
        Reach::Loopback | Reach::Lan => WINHTTP_ACCESS_TYPE_NO_PROXY,
        _ => WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
    };
    // A model on this computer or the local network may think for a minute or two first.
    let receive = if matches!(reach, Reach::Loopback | Reach::Lan) {
        120_000
    } else {
        30_000
    };
    // SAFETY: the wide strings and `body` outlive the calls; handles close in reverse order
    // on drop.
    unsafe {
        let session = Handle::new(
            WinHttpOpen(
                PCWSTR(agent.as_ptr()),
                access,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            ),
            "WinHttpOpen",
        )?;
        check(
            WinHttpSetTimeouts(session.0, 15_000, 15_000, 30_000, receive),
            "WinHttpSetTimeouts",
        )?;
        let connection = Handle::new(
            WinHttpConnect(session.0, PCWSTR(host.as_ptr()), port, 0),
            "WinHttpConnect",
        )?;
        let handle = WinHttpOpenRequest(
            connection.0,
            PCWSTR(verb.as_ptr()),
            PCWSTR(path.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            core::ptr::null(),
            if secure { WINHTTP_FLAG_SECURE } else { 0 },
        );
        if handle.is_null() {
            return Err(last_error("WinHttpOpenRequest"));
        }
        let request = Request {
            handle,
            host: name,
            reach,
            pinned,
            proxy: AtomicBool::new(false),
            refused: OnceLock::new(),
        };
        if plugin {
            if WinHttpSetStatusCallback(handle, on_status, WINHTTP_CALLBACK_FLAGS, 0) == -1 {
                return Err(last_error("WinHttpSetStatusCallback"));
            }
            let never = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
            check(
                WinHttpSetOption(
                    handle,
                    WINHTTP_OPTION_REDIRECT_POLICY,
                    (&never as *const u32).cast(),
                    4,
                ),
                "WinHttpSetOption",
            )?;
        }
        let extra = to_wide(headers);
        let (extra_ptr, extra_len) = if headers.is_empty() {
            (PCWSTR::null(), 0)
        } else {
            (PCWSTR(extra.as_ptr()), u32::MAX)
        };
        // For `on_status`, set only for a plugin's request.
        let context = &request as *const Request as usize;
        let sent = WinHttpSendRequest(handle, extra_ptr, extra_len, data, length, length, context);
        if let Some(ip) = request.refused.get() {
            return Err(refusal(&request.host, ip, reach));
        }
        check(sent, "WinHttpSendRequest")?;
        check(
            WinHttpReceiveResponse(handle, core::ptr::null_mut()),
            "WinHttpReceiveResponse",
        )?;
        let mut status = 0u32;
        let mut size = size_of::<u32>() as u32;
        check(
            WinHttpQueryHeaders(
                handle,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                PCWSTR::null(),
                (&mut status as *mut u32).cast(),
                &mut size,
                core::ptr::null_mut(),
            ),
            "WinHttpQueryHeaders",
        )?;
        let mut lines = String::new();
        if plugin {
            let mut size = 0u32;
            let raw = WINHTTP_QUERY_RAW_HEADERS_CRLF;
            let none = core::ptr::null_mut();
            WinHttpQueryHeaders(handle, raw, PCWSTR::null(), none, &mut size, none.cast());
            let mut buf = vec![0u16; size as usize / 2 + 1];
            let ptr = buf.as_mut_ptr().cast();
            if WinHttpQueryHeaders(handle, raw, PCWSTR::null(), ptr, &mut size, none.cast()) != 0 {
                lines = String::from_utf16_lossy(&buf[..size as usize / 2]);
            }
        } else if status != 200 {
            return Ok((status, lines, Vec::new()));
        }
        let mut body = Vec::new();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let mut read = 0u32;
            check(
                WinHttpReadData(
                    handle,
                    chunk.as_mut_ptr().cast(),
                    chunk.len() as u32,
                    &mut read,
                ),
                "WinHttpReadData",
            )?;
            if read == 0 {
                return Ok((status, lines, body));
            }
            body.extend_from_slice(&chunk[..read as usize]);
            if body.len() > limit {
                return Err(format!("{url} is larger than {limit} bytes"));
            }
            if started.elapsed() > TOTAL_TIME {
                return Err(format!("{url} took longer than {} s", TOTAL_TIME.as_secs()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    /// Serves `responses` in order, one connection each, on a local port.
    fn serve(responses: Vec<String>) -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        port
    }

    fn response(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn gets_a_body_and_rejects_errors_and_oversized_bodies() {
        let port = serve(vec![
            response("200 OK", "hello"),
            response("404 Not Found", "missing"),
            response("200 OK", "0123456789"),
        ]);
        let url = format!("http://127.0.0.1:{port}/latest");
        assert_eq!(
            get(&url, "PecoFence-test", "Accept: text/plain", 100).unwrap(),
            b"hello"
        );
        assert!(
            get(&url, "PecoFence-test", "", 100)
                .unwrap_err()
                .contains("HTTP 404")
        );
        assert!(
            get(&url, "PecoFence-test", "", 4)
                .unwrap_err()
                .contains("larger")
        );
    }

    #[test]
    fn a_plugin_request_never_connects_to_this_computer() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!(
            "http://127.0.0.1:{}/",
            listener.local_addr().unwrap().port()
        );
        let error = fetch("GET", &url, "PecoFence-test", "", &[], 100).unwrap_err();
        assert!(error.contains("resolves to 127.0.0.1"), "{error}");
        // Refused before WinHTTP connected anywhere.
        let accepted = listener.accept().map(|_| ()).map_err(|e| e.kind());
        assert_eq!(accepted, Err(std::io::ErrorKind::WouldBlock));
        // The app's own requests (a local test server of the updater) are not checked.
        let port = serve(vec![response("200 OK", "hello")]);
        let url = format!("http://127.0.0.1:{port}/");
        assert_eq!(get(&url, "PecoFence-test", "", 100).unwrap(), b"hello");
    }

    #[test]
    fn a_local_request_reaches_only_its_kind_of_address_and_names_the_host_asked_for() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 4096];
            let n = stream.read(&mut request).unwrap();
            stream
                .write_all(response("200 OK", "models").as_bytes())
                .unwrap();
            String::from_utf8_lossy(&request[..n]).to_string()
        });
        // `localhost` resolves to ::1 first; the service listens on 127.0.0.1 alone.
        let url = format!("http://localhost:{port}/api/tags");
        let (status, _, body) =
            fetch_local("GET", &url, "PecoFence-test", "Accept: x", &[], 100, false).unwrap();
        assert_eq!((status, body.as_slice()), (200, b"models".as_slice()));
        let request = seen.join().unwrap();
        let hosts: Vec<&str> = request
            .lines()
            .filter(|l| l.to_ascii_lowercase().starts_with("host:"))
            .collect();
        assert_eq!(hosts, [format!("Host: localhost:{port}")], "{request}");
        assert!(request.contains("Accept: x"), "{request}");
        // This computer is no device of the local network, and no address goes to `loopback`
        // that is not this computer's.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!(
            "http://127.0.0.1:{}/",
            listener.local_addr().unwrap().port()
        );
        let error = fetch_local("GET", &url, "t", "", &[], 100, true).unwrap_err();
        assert!(
            error.contains("not an address of the local network"),
            "{error}"
        );
        let accepted = listener.accept().map(|_| ()).map_err(|e| e.kind());
        assert_eq!(accepted, Err(std::io::ErrorKind::WouldBlock));
        let error =
            fetch_local("GET", "http://10.255.255.1:9/", "t", "", &[], 100, false).unwrap_err();
        assert!(error.contains("not an address of this computer"), "{error}");
    }

    #[test]
    fn the_status_callback_refuses_a_local_address_unless_a_proxy_connects() {
        let status = |request: &Request, status: u32, text: &str| {
            let text = to_wide(text);
            let context = request as *const Request as usize;
            unsafe {
                on_status(
                    core::ptr::null_mut(),
                    context,
                    status,
                    text.as_ptr() as _,
                    0,
                )
            };
        };
        // A local request has no proxy: another name resolved is no reason to skip the check.
        // It connects only to an address it was checked for (pinned), not to another one of
        // its kind (a name whose answer changed since).
        let pin = |ip: &str| vec![read_ip(ip).unwrap()];
        for (reach, resolved, ip, pinned, refused) in [
            (
                Reach::Internet,
                "a.example",
                "10.0.0.1",
                vec![],
                Some("10.0.0.1"),
            ),
            (Reach::Internet, "a.example", "93.184.215.14", vec![], None),
            (Reach::Internet, "127.0.0.1", "127.0.0.1", vec![], None),
            (Reach::Lan, "a.example", "192.168.1.20", vec![], None),
            (
                Reach::Lan,
                "a.example",
                "127.0.0.1",
                vec![],
                Some("127.0.0.1"),
            ),
            (
                Reach::Lan,
                "proxy.example",
                "93.184.215.14",
                vec![],
                Some("93.184.215.14"),
            ),
            (Reach::Loopback, "a.example", "::1", vec![], None),
            (
                Reach::Loopback,
                "proxy.example",
                "10.0.0.1",
                vec![],
                Some("10.0.0.1"),
            ),
            (
                Reach::Lan,
                "a.example",
                "192.168.1.20",
                pin("192.168.1.20"),
                None,
            ),
            (
                Reach::Lan,
                "a.example",
                "192.168.1.21",
                pin("192.168.1.20"),
                Some("192.168.1.21"),
            ),
            (Reach::Loopback, "a.example", "[::1]", pin("::1"), None),
            (
                Reach::Loopback,
                "a.example",
                "127.0.0.2",
                pin("127.0.0.1"),
                Some("127.0.0.2"),
            ),
        ] {
            let session =
                unsafe { WinHttpOpen(PCWSTR::null(), 1, PCWSTR::null(), PCWSTR::null(), 0) };
            let request = Request {
                handle: session,
                host: "a.example".into(),
                reach,
                pinned,
                proxy: AtomicBool::new(false),
                refused: OnceLock::new(),
            };
            status(&request, WINHTTP_CALLBACK_STATUS_RESOLVING_NAME, resolved);
            status(&request, WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER, ip);
            assert_eq!(
                request.refused.get().map(String::as_str),
                refused,
                "{reach:?} {resolved} {ip}"
            );
        }
    }

    #[test]
    fn only_https_and_local_test_servers_are_allowed() {
        assert_eq!(
            split_url("https://api.github.com/repos/x/y", Reach::App).unwrap(),
            (true, "api.github.com".into(), 443, "/repos/x/y".into())
        );
        assert_eq!(
            split_url("http://127.0.0.1:8080", Reach::App).unwrap(),
            (false, "127.0.0.1".into(), 8080, "/".into())
        );
        assert!(split_url("http://example.com/", Reach::App).is_err());
        assert!(split_url("http://example.com/", Reach::Internet).is_err());
        assert_eq!(
            split_url("http://nas.local:8123/api", Reach::Lan).unwrap(),
            (false, "nas.local".into(), 8123, "/api".into())
        );
        assert!(split_url("file:///C:/x", Reach::App).is_err());
        // The host is the one the plugin host checked: no user name, backslash or NUL.
        assert!(split_url("https://evil.example\u{0}@api.github.com/", Reach::App).is_err());
        assert!(split_url("https://evil.com\\@api.github.com/", Reach::App).is_err());
    }
}
