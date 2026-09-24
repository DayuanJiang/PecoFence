//! JSON on stdout for results, JSON on stderr for errors. Nothing else is ever printed.

use std::io::Write;

use pecofence_ipc::IpcError;
use serde_json::Value;

/// A command's outcome: the JSON to print plus a secondary warning from the server and whether
/// part of a client-side batch failed (exit code 1 even though there is a result).
#[derive(Debug, Default)]
pub struct Reply {
    pub result: Value,
    pub warning: Option<String>,
    pub partial_failure: bool,
}

impl Reply {
    pub fn new(result: Value) -> Self {
        Self {
            result,
            warning: None,
            partial_failure: false,
        }
    }

    /// The payload as printed: `warning` merged into an object result, or wrapped beside it.
    pub fn payload(self) -> Value {
        match (self.result, self.warning) {
            (result, None) => result,
            (Value::Object(mut map), Some(warning)) => {
                map.insert("warning".into(), Value::String(warning));
                Value::Object(map)
            }
            (result, Some(warning)) => {
                serde_json::json!({ "result": result, "warning": warning })
            }
        }
    }
}

pub fn render(value: &Value, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .unwrap_or_else(|_| "null".into())
}

pub fn print_result(reply: Reply, pretty: bool) -> i32 {
    let exit = if reply.partial_failure { 1 } else { 0 };
    print_text(&render(&reply.payload(), pretty), true);
    exit
}

/// Writes to stdout without panicking when the reader went away (`| head`).
pub fn print_text(text: &str, newline: bool) {
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes());
    if newline {
        let _ = out.write_all(b"\n");
    }
    let _ = out.flush();
}

pub fn error_payload(error: &IpcError) -> Value {
    serde_json::json!({ "error": error })
}

pub fn print_error(error: &IpcError, pretty: bool) -> i32 {
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "{}", render(&error_payload(error), pretty));
    let _ = err.flush();
    error.code.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_ipc::ErrorCode;
    use serde_json::json;

    #[test]
    fn warning_is_merged_into_objects_and_wrapped_otherwise() {
        let mut reply = Reply::new(json!({"changed": true}));
        reply.warning = Some("not saved".into());
        assert_eq!(
            reply.payload(),
            json!({"changed": true, "warning": "not saved"})
        );

        let mut reply = Reply::new(json!([1, 2]));
        reply.warning = Some("w".into());
        assert_eq!(reply.payload(), json!({"result": [1, 2], "warning": "w"}));

        assert_eq!(Reply::new(json!(5)).payload(), json!(5));
    }

    #[test]
    fn error_payload_has_the_documented_shape_and_exit_codes() {
        let e = IpcError::new(ErrorCode::NotRunning, "nope").hint("start it");
        let v = error_payload(&e);
        assert_eq!(v["error"]["code"], "not_running");
        assert_eq!(v["error"]["message"], "nope");
        assert_eq!(v["error"]["hint"], "start it");
        assert!(v["error"].get("details").is_none());
        assert_eq!(e.code.exit_code(), 3);
        assert_eq!(ErrorCode::Timeout.exit_code(), 4);
        assert_eq!(ErrorCode::Usage.exit_code(), 2);
        assert_eq!(ErrorCode::AmbiguousFence.exit_code(), 1);
        let compact = render(&v, false);
        assert!(!compact.contains('\n'));
        assert_eq!(serde_json::from_str::<Value>(&compact).unwrap(), v);
        assert!(render(&v, true).contains('\n'));
    }
}
