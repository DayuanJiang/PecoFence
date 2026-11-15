//! What a light plugin shows (`pf.show(view)`): rows from the top, read from the JSON the
//! plugin passed. The app turns them into script fence rows and draws them.

use serde::Deserialize;

/// At most this many rows; the rest are dropped.
pub const MAX_ROWS: usize = 300;

/// What a plugin shows: rows from the top.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct View {
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Row {
    pub text: String,
    /// What shows when `text` is too wide for the fence (a `fit` row).
    pub short: Option<String>,
    /// `small`, `normal`, `large`, `xlarge`, `huge`, or `fit` (scaled with the fence).
    pub size: Option<String>,
    pub bold: bool,
    pub color: Option<String>,
}

/// The view in `json`, cut to the limits above.
pub fn parse(json: &str) -> serde_json::Result<View> {
    let mut view: View = serde_json::from_str(json)?;
    view.rows.truncate(MAX_ROWS);
    Ok(view)
}
