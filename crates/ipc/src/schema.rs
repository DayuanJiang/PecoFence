//! JSON Schema export for `pecofence-cli describe --schema <Name>` (feature `describe`).

use schemars::{JsonSchema, schema_for};
use serde_json::Value;

/// Names accepted by [`schema`].
pub const SCHEMA_NAMES: &[&str] = &[
    "Request",
    "Response",
    "Settings",
    "Fence",
    "FenceDto",
    "ItemDto",
    "MonitorDto",
    "StatusDto",
    "SnapshotDto",
    "RuleSet",
    "Rule",
    "Cond",
    "RuleListDto",
    "BackupDto",
];

fn to_value<T: JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).unwrap_or(Value::Null)
}

/// The JSON Schema (draft 2020-12) of one protocol or model type; `None` for unknown names.
pub fn schema(name: &str) -> Option<Value> {
    Some(match name {
        "Request" => to_value::<crate::Request>(),
        "Response" => to_value::<crate::Response>(),
        "Settings" => to_value::<pecofence_core::Settings>(),
        "Fence" => to_value::<pecofence_core::Fence>(),
        "FenceDto" => to_value::<crate::FenceDto>(),
        "ItemDto" => to_value::<crate::ItemDto>(),
        "MonitorDto" => to_value::<crate::MonitorDto>(),
        "StatusDto" => to_value::<crate::StatusDto>(),
        "SnapshotDto" => to_value::<crate::SnapshotDto>(),
        "RuleSet" => to_value::<pecofence_core::rules::RuleSet>(),
        "Rule" => to_value::<pecofence_core::rules::Rule>(),
        "Cond" => to_value::<pecofence_core::rules::Cond>(),
        "RuleListDto" => to_value::<crate::RuleListDto>(),
        "BackupDto" => to_value::<crate::BackupDto>(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_schema_exists_and_is_an_object() {
        for name in SCHEMA_NAMES {
            let s = schema(name).unwrap_or_else(|| panic!("{name}"));
            assert!(s.is_object(), "{name}: {s}");
        }
        assert!(schema("Nope").is_none());
    }

    #[test]
    fn method_schema_lists_dotted_names() {
        let s = schema("Request").unwrap().to_string();
        assert!(s.contains("fences.create"), "{s}");
        assert!(s.contains("settings.patch"));
    }
}
