//! HTTPS GET and POST through WinHTTP (the system proxy and Windows' TLS), so the updater and
//! the usage statistics need no HTTP crate. Blocking: call from a worker thread.

use crate::wide::to_wide;
use core::ffi::c_void;
use pecofence_core::web_address::{WebAddress, public_ip};
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

/// A request handle. For a plugin's request WinHTTP's status callback (`on_status`) checks the
/// address each connection goes to and closes the handle itself when it refuses one.
struct Request {
    handle: *mut c_void,
    host: String,
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
    if status == WINHTTP_CALLBACK_STATUS_RESOLVING_NAME {
        let proxy = !text.eq_ignore_ascii_case(&request.host);
        request.proxy.store(proxy, Relaxed);
        if proxy {
            tracing::debug!(host = %request.host, proxy = %text, "plugin request goes through a proxy; its address is not checked");
        }
    } else if status == WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER && !request.proxy.load(Relaxed)
    {
        let public = public_ip(&text);
        tracing::debug!(host = %request.host, ip = %text, public, "plugin request connects");
        if !public {
            tracing::info!(host = %request.host, ip = %text, "plugin request refused: the host resolves to this computer or the local network");
            if request.refused.set(text).is_ok() {
                // SAFETY: the request's handle, closed once; closing cancels the request.
                unsafe { WinHttpCloseHandle(request.handle) };
            }
        }
    }
}

fn refusal(host: &str, ip: &str) -> String {
    format!("{host} resolves to {ip}, an address of this computer or the local network: refused")
}

fn check(ok: i32, what: &str) -> Result<(), String> {
    if ok != 0 {
        Ok(())
    } else {
        Err(last_error(what))
    }
}

/// `https://host[:port]/path`; plain `http://` only for local test servers. Read by
/// [`WebAddress`], the reading the plugin host checks a plugin's address with.
fn split_url(url: &str) -> Result<(bool, String, u16, String), String> {
    let a = WebAddress::parse(url).ok_or_else(|| format!("unsupported address {url}"))?;
    if !a.secure && a.host != "127.0.0.1" {
        return Err(format!("unsupported address {url}"));
    }
    let port = a.port.unwrap_or(if a.secure { 443 } else { 80 });
    Ok((a.secure, a.host, port, a.path))
}

/// GETs `url` (following redirects) and returns the body of a 200 response, at most `limit`
/// bytes. `headers` are extra request lines such as `Accept: …`, separated by `\r\n`.
pub fn get(url: &str, user_agent: &str, headers: &str, limit: usize) -> Result<Vec<u8>, String> {
    match request("GET", url, user_agent, headers, &[], limit, false)? {
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
    request(verb, url, user_agent, headers, body, limit, true)
}

/// POSTs `body` to `url` and returns the HTTP status of the answer. `headers` as for [`get`].
/// An error is a request that got no answer.
pub fn post(url: &str, user_agent: &str, headers: &str, body: &[u8]) -> Result<u32, String> {
    request("POST", url, user_agent, headers, body, 64 * 1024, false).map(|(status, ..)| status)
}

/// The SHA-256 of `data` in lowercase hex, for a header that carries the hash of a body.
pub fn sha256_hex(data: &[u8]) -> String {
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
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Sends `verb` for `url` with `body`; returns the status and, for a 200, the response's body
/// (at most `limit` bytes). `plugin`: no redirect is followed, the header lines come back and
/// so does the body of any status.
fn request(
    verb: &str,
    url: &str,
    user_agent: &str,
    headers: &str,
    body: &[u8],
    limit: usize,
    plugin: bool,
) -> Result<(u32, String, Vec<u8>), String> {
    let started = Instant::now();
    let (secure, name, port, path) = split_url(url)?;
    if plugin {
        use std::net::ToSocketAddrs;
        let found = (name.as_str(), port)
            .to_socket_addrs()
            .into_iter()
            .flatten();
        if let Some(ip) = found.map(|a| a.ip().to_string()).find(|ip| !public_ip(ip)) {
            tracing::info!(host = %name, %ip, "plugin request refused before connecting: the host resolves to this computer or the local network");
            return Err(refusal(&name, &ip));
        }
    }
    let (agent, host, path) = (to_wide(user_agent), to_wide(&name), to_wide(&path));
    let verb = to_wide(verb);
    let length = u32::try_from(body.len()).map_err(|_| "the request body is too large")?;
    let data = if body.is_empty() {
        core::ptr::null()
    } else {
        body.as_ptr().cast()
    };
    // SAFETY: the wide strings and `body` outlive the calls; handles close in reverse order
    // on drop.
    unsafe {
        let session = Handle::new(
            WinHttpOpen(
                PCWSTR(agent.as_ptr()),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            ),
            "WinHttpOpen",
        )?;
        check(
            WinHttpSetTimeouts(session.0, 15_000, 15_000, 30_000, 30_000),
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
            return Err(refusal(&request.host, ip));
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
        for (resolved, ip, refused) in [
            ("a.example", "10.0.0.1", Some("10.0.0.1")),
            ("a.example", "93.184.215.14", None),
            ("127.0.0.1", "127.0.0.1", None),
        ] {
            let session =
                unsafe { WinHttpOpen(PCWSTR::null(), 1, PCWSTR::null(), PCWSTR::null(), 0) };
            let request = Request {
                handle: session,
                host: "a.example".into(),
                proxy: AtomicBool::new(false),
                refused: OnceLock::new(),
            };
            status(&request, WINHTTP_CALLBACK_STATUS_RESOLVING_NAME, resolved);
            status(&request, WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER, ip);
            assert_eq!(
                request.refused.get().map(String::as_str),
                refused,
                "{resolved} {ip}"
            );
        }
    }

    #[test]
    fn only_https_and_local_test_servers_are_allowed() {
        assert_eq!(
            split_url("https://api.github.com/repos/x/y").unwrap(),
            (true, "api.github.com".into(), 443, "/repos/x/y".into())
        );
        assert_eq!(
            split_url("http://127.0.0.1:8080").unwrap(),
            (false, "127.0.0.1".into(), 8080, "/".into())
        );
        assert!(split_url("http://example.com/").is_err());
        assert!(split_url("file:///C:/x").is_err());
        // The host is the one the plugin host checked: no user name, backslash or NUL.
        assert!(split_url("https://evil.example\u{0}@api.github.com/").is_err());
        assert!(split_url("https://evil.com\\@api.github.com/").is_err());
    }
}
