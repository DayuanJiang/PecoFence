//! Result shapes. Enumerations that the settings page already exposes as strings (`opacity`,
//! `layout`, `sort`, …) stay strings here, so `fence get` output can be fed straight back into
//! `fence set`.

use pecofence_core::NormGeometry;
use pecofence_core::rules::{Rule, Target};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A rectangle in physical pixels, virtual-screen coordinates (the primary monitor's top-left
/// is 0,0; monitors to the left have negative `x`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl From<pecofence_core::geometry::PxRect> for Rect {
    fn from(r: pecofence_core::geometry::PxRect) -> Self {
        Self {
            x: r.left,
            y: r.top,
            w: r.right - r.left,
            h: r.bottom - r.top,
        }
    }
}

impl From<Rect> for pecofence_core::geometry::PxRect {
    fn from(r: Rect) -> Self {
        Self {
            left: r.x,
            top: r.y,
            right: r.x + r.w,
            bottom: r.y + r.h,
        }
    }
}

impl Rect {
    /// Parses `x,y,w,h` (the CLI's `--rect` syntax).
    pub fn parse(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split(',').map(str::trim).collect();
        if parts.len() != 4 {
            return Err(format!("expected x,y,w,h but got {text:?}"));
        }
        let mut n = [0i32; 4];
        for (i, p) in parts.iter().enumerate() {
            n[i] = p
                .parse()
                .map_err(|_| format!("{p:?} in {text:?} is not an integer"))?;
        }
        if n[2] <= 0 || n[3] <= 0 {
            return Err(format!("width and height must be positive in {text:?}"));
        }
        Ok(Self {
            x: n[0],
            y: n[1],
            w: n[2],
            h: n[3],
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct StatusDto {
    pub app_version: String,
    pub protocol: u32,
    pub pid: u32,
    /// `PECOFENCE_INSTANCE` of the answering process; `null` for the main instance.
    pub instance: Option<String>,
    pub config_path: String,
    pub fence_count: usize,
    pub item_count: usize,
    pub memory_mb: f64,
    /// `light` | `dark`.
    pub theme_mode: String,
    pub desktop_icons_hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
    /// Stable device id used by `fences.create{monitor}` / `fences.moveToMonitor`.
    pub id: String,
    /// Human label (model name or `\\.\DISPLAY1`).
    pub label: String,
    /// Full monitor rectangle including the taskbar.
    pub rect: Rect,
    /// Area fences may occupy.
    pub work_area: Rect,
    pub dpi: u32,
    pub primary: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PortalDto {
    /// Folder the portal shows.
    pub path: String,
    /// Double-clicking a subfolder navigates inside the portal instead of opening Explorer.
    pub navigate: bool,
    /// Folder glyph shown before the title.
    pub title_icon: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FenceDto {
    pub id: Uuid,
    pub title: String,
    /// `virtual` | `inbox` | `portal`.
    pub kind: String,
    /// Expanded geometry in physical px (what `fences.setBounds` sets). `null` while the fence's
    /// monitor is disconnected.
    pub rect: Option<Rect>,
    /// The window as currently shown (only the title bar when rolled up); `null` for hosted
    /// tabs and disconnected monitors.
    pub window_rect: Option<Rect>,
    /// Monitor device id (see `monitors.list`).
    pub monitor: String,
    pub monitor_connected: bool,
    /// Saved geometry: DIPs relative to the monitor's work area, plus the anchor edge.
    pub geometry: NormGeometry,
    pub rolled_up: bool,
    pub locked: bool,
    pub item_count: usize,
    /// Set when this fence is shown as a tab inside another fence's window.
    pub tab_host: Option<Uuid>,
    /// Tabs hosted by this fence (strip order), empty when it is a plain window.
    pub tabs: Vec<Uuid>,
    /// 32 | 48 | 64 | 96.
    pub icon_size: u32,
    /// `compact` | `normal` | `loose`.
    pub spacing: String,
    pub auto_height: bool,
    pub exclude_from_quick_hide: bool,
    /// `default` | `clear` | `solid`.
    pub opacity: String,
    /// `#RRGGBB` or `null`.
    pub tint: Option<String>,
    /// `theme` | `tint` | `white` | `black` | `#RRGGBB`.
    pub title_color: String,
    /// `small` | `normal` | `large`.
    pub title_size: String,
    /// `icons` | `list` | `details`.
    pub layout: String,
    /// `manual` | `name` | `type` | `date` | `size` | `openCount`.
    pub sort: String,
    pub reverse: bool,
    pub group_by_date: bool,
    pub label_lines: u8,
    pub portal: Option<PortalDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    pub id: Uuid,
    /// Display name as shown under the icon (usually without the extension).
    pub name: String,
    /// Full path; `null` for shell namespace items.
    pub path: Option<String>,
    pub is_folder: bool,
    pub fence: Uuid,
    pub fence_title: String,
    /// `user` | `rule` | `migration` | `portal` (folder portal contents are not assigned).
    pub assigned_by: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub id: Uuid,
    pub name: String,
    /// Unix seconds.
    pub ts: i64,
    pub fence_count: usize,
}

/// One rule plus its position and the resolved target title.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RuleEntry {
    pub index: usize,
    #[serde(flatten)]
    pub rule: Rule,
    /// Title of the target fence (`null` for the inbox).
    pub target_title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RuleListDto {
    pub default_target: Target,
    /// New desktop items are filed automatically.
    pub keep_updated: bool,
    pub list: Vec<RuleEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_parses_and_converts() {
        let r = Rect::parse(" 10, 20 ,300,200").unwrap();
        assert_eq!(
            r,
            Rect {
                x: 10,
                y: 20,
                w: 300,
                h: 200
            }
        );
        let px: pecofence_core::geometry::PxRect = r.into();
        assert_eq!((px.left, px.top, px.right, px.bottom), (10, 20, 310, 220));
        assert_eq!(Rect::from(px), r);
        assert!(Rect::parse("1,2,3").is_err());
        assert!(Rect::parse("1,2,0,3").is_err());
        assert!(Rect::parse("a,2,3,4").is_err());
    }
}
