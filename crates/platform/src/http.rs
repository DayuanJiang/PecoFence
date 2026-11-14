//! HTTPS GET and POST through WinHTTP (the system proxy and Windows' TLS), so the updater and
//! the usage statistics need no HTTP crate. Blocking: call from a worker thread.

use crate::wide::to_wide;
use core::ffi::c_void;
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

const WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY: u32 = 4;
const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;
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

fn check(ok: i32, what: &str) -> Result<(), String> {
    if ok != 0 {
        Ok(())
    } else {
        Err(last_error(what))
    }
}

/// `https://host[:port]/path`; plain `http://` only for local test servers.
fn split_url(url: &str) -> Result<(bool, &str, u16, &str), String> {
    let (secure, rest) = match url.split_once("://") {
        Some(("https", rest)) => (true, rest),
        Some(("http", rest)) if rest.starts_with("127.0.0.1:") => (false, rest),
        _ => return Err(format!("unsupported address {url}")),
    };
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host,
            port.parse().map_err(|_| format!("bad port in {url}"))?,
        ),
        None => (authority, if secure { 443 } else { 80 }),
    };
    if host.is_empty() {
        return Err(format!("no host in {url}"));
    }
    Ok((secure, host, port, if path.is_empty() { "/" } else { path }))
}

/// GETs `url` (following redirects) and returns the body of a 200 response, at most `limit`
/// bytes. `headers` are extra request lines such as `Accept: …`, separated by `\r\n`.
pub fn get(url: &str, user_agent: &str, headers: &str, limit: usize) -> Result<Vec<u8>, String> {
    match request("GET", url, user_agent, headers, &[], limit)? {
        (200, body) => Ok(body),
        (status, _) => Err(format!("HTTP {status} from {url}")),
    }
}

/// POSTs `body` to `url` and returns the HTTP status of the answer. `headers` as for [`get`].
/// An error is a request that got no answer.
pub fn post(url: &str, user_agent: &str, headers: &str, body: &[u8]) -> Result<u32, String> {
    request("POST", url, user_agent, headers, body, 64 * 1024).map(|(status, _)| status)
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
/// (at most `limit` bytes).
fn request(
    verb: &str,
    url: &str,
    user_agent: &str,
    headers: &str,
    body: &[u8],
    limit: usize,
) -> Result<(u32, Vec<u8>), String> {
    let started = Instant::now();
    let (secure, host, port, path) = split_url(url)?;
    let (agent, host, path) = (to_wide(user_agent), to_wide(host), to_wide(path));
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
        let request = Handle::new(
            WinHttpOpenRequest(
                connection.0,
                PCWSTR(verb.as_ptr()),
                PCWSTR(path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                core::ptr::null(),
                if secure { WINHTTP_FLAG_SECURE } else { 0 },
            ),
            "WinHttpOpenRequest",
        )?;
        let extra = to_wide(headers);
        let (extra_ptr, extra_len) = if headers.is_empty() {
            (PCWSTR::null(), 0)
        } else {
            (PCWSTR(extra.as_ptr()), u32::MAX)
        };
        check(
            WinHttpSendRequest(request.0, extra_ptr, extra_len, data, length, length, 0),
            "WinHttpSendRequest",
        )?;
        check(
            WinHttpReceiveResponse(request.0, core::ptr::null_mut()),
            "WinHttpReceiveResponse",
        )?;
        let mut status = 0u32;
        let mut size = size_of::<u32>() as u32;
        check(
            WinHttpQueryHeaders(
                request.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                PCWSTR::null(),
                (&mut status as *mut u32).cast(),
                &mut size,
                core::ptr::null_mut(),
            ),
            "WinHttpQueryHeaders",
        )?;
        if status != 200 {
            return Ok((status, Vec::new()));
        }
        let mut body = Vec::new();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let mut read = 0u32;
            check(
                WinHttpReadData(
                    request.0,
                    chunk.as_mut_ptr().cast(),
                    chunk.len() as u32,
                    &mut read,
                ),
                "WinHttpReadData",
            )?;
            if read == 0 {
                return Ok((status, body));
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
    fn only_https_and_local_test_servers_are_allowed() {
        assert_eq!(
            split_url("https://api.github.com/repos/x/y").unwrap(),
            (true, "api.github.com", 443, "/repos/x/y")
        );
        assert_eq!(
            split_url("http://127.0.0.1:8080").unwrap(),
            (false, "127.0.0.1", 8080, "/")
        );
        assert!(split_url("http://example.com/").is_err());
        assert!(split_url("file:///C:/x").is_err());
    }
}
