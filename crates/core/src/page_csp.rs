//! The inline code of a web plugin's HTML file, as the browser hashes it for the page's
//! Content-Security-Policy (`'sha256-…'`): each `<script>` without `src` and each event handler
//! attribute (`onclick="…"`). PecoFence lists exactly these, so the code written in the
//! plugin's file runs and code a page makes at run time (a script element filled from fetched
//! text, `innerHTML` with a handler) does not.

/// The texts to hash: the inline scripts and the handler attributes (decoded), with line
/// breaks as the HTML parser makes them (`\r\n` and `\r` become `\n`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct InlineCode {
    pub scripts: Vec<String>,
    pub handlers: Vec<String>,
}

/// Reads the inline code of an HTML file.
pub fn inline_code(html: &str) -> InlineCode {
    let html = html.replace("\r\n", "\n").replace('\r', "\n");
    let lower = html.to_ascii_lowercase();
    let mut out = InlineCode::default();
    let mut at = 0;
    while let Some(lt) = html[at..].find('<').map(|i| at + i) {
        if html[lt..].starts_with("<!--") {
            at = html[lt..].find("-->").map_or(html.len(), |i| lt + i + 3);
            continue;
        }
        let name_len = html[lt + 1..]
            .bytes()
            .take_while(u8::is_ascii_alphanumeric)
            .count();
        if name_len == 0 {
            at = lt + 1;
            continue;
        }
        let name = &lower[lt + 1..lt + 1 + name_len];
        let (attrs, end) = attributes(&html, lt + 1 + name_len);
        for (key, value) in &attrs {
            if key.starts_with("on") && key.len() > 2 {
                out.handlers.push(decode(value));
            }
        }
        at = end;
        if name == "script" {
            let close = lower[at..].find("</script").map_or(html.len(), |i| at + i);
            if !attrs.iter().any(|(k, _)| k == "src") {
                out.scripts.push(html[at..close].to_string());
            }
            at = close;
        }
    }
    out
}

/// The attributes of a start tag from `at` (after its name) to its `>`, names in lowercase,
/// values as written; and the place after the `>`.
fn attributes(html: &str, mut at: usize) -> (Vec<(String, String)>, usize) {
    let b = html.as_bytes();
    let mut attrs = Vec::new();
    loop {
        while at < b.len() && (b[at].is_ascii_whitespace() || b[at] == b'/') {
            at += 1;
        }
        if at >= b.len() || b[at] == b'>' {
            return (attrs, (at + 1).min(b.len()));
        }
        let start = at;
        while at < b.len() && !b[at].is_ascii_whitespace() && !b"=>/".contains(&b[at]) {
            at += 1;
        }
        let key = html[start..at].to_ascii_lowercase();
        while at < b.len() && b[at].is_ascii_whitespace() {
            at += 1;
        }
        let mut value = String::new();
        if at < b.len() && b[at] == b'=' {
            at += 1;
            while at < b.len() && b[at].is_ascii_whitespace() {
                at += 1;
            }
            if at < b.len() && (b[at] == b'"' || b[at] == b'\'') {
                let q = b[at];
                let end = (at + 1..b.len()).find(|&i| b[i] == q).unwrap_or(b.len());
                value = html[at + 1..end].to_string();
                at = (end + 1).min(b.len());
            } else {
                let start = at;
                while at < b.len() && !b[at].is_ascii_whitespace() && b[at] != b'>' {
                    at += 1;
                }
                value = html[start..at].to_string();
            }
        }
        attrs.push((key, value));
    }
}

/// An attribute value as the browser reads it: the character references decoded (the named
/// ones code is written with, and numeric ones).
fn decode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let end = rest.find(';').filter(|&i| i <= 10);
        let named = end.and_then(|i| match &rest[1..i] {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            n if n.starts_with("#x") || n.starts_with("#X") => u32::from_str_radix(&n[2..], 16)
                .ok()
                .and_then(char::from_u32),
            n if n.starts_with('#') => n[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        });
        match (named, end) {
            (Some(c), Some(i)) => {
                out.push(c);
                rest = &rest[i + 1..];
            }
            _ => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Standard base64 with padding (a CSP hash source).
pub fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for k in 0..4 {
            let c = ABC[(n >> (18 - 6 * k)) as usize & 63] as char;
            out.push(if k <= chunk.len() { c } else { '=' });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_and_handlers_are_found() {
        let html = "<!doctype html><html><head>\r\n<script>window.a = 1;\r\n</script>\
            <script src=\"app.js\" defer></script><SCRIPT type=module>b()</SCRIPT>\
            <!-- <script>commented()</script> --></head>\
            <body><button onclick=\"go(&quot;x&quot;) &amp;&amp; y()\" ONMOUSEOVER='z()'>b</button>\
            <img src=x onerror=boom()><p title=\"on\">t</p></body></html>";
        let c = inline_code(html);
        assert_eq!(
            c.scripts,
            vec!["window.a = 1;\n".to_string(), "b()".to_string()]
        );
        assert_eq!(
            c.handlers,
            vec![
                "go(\"x\") && y()".to_string(),
                "z()".to_string(),
                "boom()".to_string()
            ]
        );
    }

    #[test]
    fn a_script_without_end_takes_the_rest() {
        assert_eq!(
            inline_code("<script>x = '</scr'").scripts,
            vec!["x = '</scr'".to_string()]
        );
        assert_eq!(inline_code("<p>a < b</p><").scripts, Vec::<String>::new());
    }

    #[test]
    fn base64_matches_known_values() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }
}
