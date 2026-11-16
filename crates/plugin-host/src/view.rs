//! What a light plugin shows (`pf.show(view)`): rows from the top, read from the JSON the
//! plugin passed and cut to PecoFence's limits. All text is plain text. The app turns the rows
//! into script fence rows and draws them.

use pecofence_core::plugins::valid_plain_name;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// At most this many rows; the rest are dropped.
pub const MAX_ROWS: usize = 300;
/// The longest text of a row, span, cell or drawing step (characters); longer text is cut.
const MAX_TEXT: usize = 4096;
/// At most this many spans or cells in a row.
const MAX_PARTS: usize = 64;
/// At most this many values in a chart and steps in a canvas.
const MAX_VALUES: usize = 1000;
/// The tallest picture, chart or canvas (DIPs).
const MAX_HEIGHT: f32 = 2000.0;
/// A picture may have at most this many pixels on a side, and this many bytes.
pub const MAX_IMAGE_SIDE: u32 = 4096;
pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

/// What a plugin shows: rows from the top.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct View {
    pub rows: Vec<Row>,
    /// The plugin's entries in its fence menu: a click sends `command` {id, via: "menu"}.
    pub menu: Vec<MenuItem>,
}

/// At most this many menu entries, each of at most this many characters (one line).
pub const MAX_MENU: usize = 8;
const MAX_MENU_TEXT: usize = 40;

/// One entry of `view.menu`: plain text, or a separator line.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct MenuItem {
    pub id: String,
    pub title: String,
    pub checked: bool,
    pub disabled: bool,
    pub separator: bool,
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
    /// An emoji or a Segoe Fluent Icons character before the text.
    pub icon: Option<String>,
    pub tip: Option<String>,
    /// The row is clickable: `click` and `dblclick` get `{ id }`.
    pub id: Option<String>,
    /// Pieces of one line, each in its own colour and weight (in place of `text`).
    pub spans: Vec<Span>,
    /// The text or picture is blurred until the pointer is on it ([`blur`]).
    #[serde(deserialize_with = "blur")]
    pub blur: bool,
    /// A to-do: its check box, ticked or not (a click sends `change`).
    pub todo: Option<bool>,
    pub separator: bool,
    pub cells: Vec<Cell>,
    /// A picture: the name of a file in the plugin's data folder.
    pub image: Option<String>,
    /// How tall a picture is drawn at most (DIPs).
    pub height: Option<f32>,
    /// 0 to 1: a thin bar under the text.
    pub progress: Option<f32>,
    pub chart: Option<Chart>,
    pub canvas: Option<Canvas>,
    pub input: Option<Input>,
    /// As a script fence's `pane=` and `follow=`: the rows above the first `pane` row stay,
    /// the others scroll; the row after a `follow` row is kept in view.
    pub pane: Option<String>,
    pub follow: Option<String>,
    /// A right-click on the row: these entries in the plugin's part of the fence menu (in place
    /// of `view.menu`); a click sends `command` {id, via: "menu", row: the row's id}.
    pub menu: Vec<MenuItem>,
    /// The row's look under the pointer, or `"event"`: the plugin hears `enter` and `leave`.
    pub hover: Option<Hover>,
    /// The wheel over the row goes to the plugin (`wheel` {id, steps}), the fence does not scroll.
    pub wheel: bool,
}

/// A row's or a cell's look under the pointer (`{color, text}`), drawn by PecoFence; or, on a
/// row, `"event"`: `enter` and `leave` {row} go to the plugin.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hover {
    pub color: Option<String>,
    pub text: Option<String>,
    pub event: bool,
}

impl<'de> Deserialize<'de> for Hover {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Form {
            Event(String),
            Look {
                #[serde(default)]
                color: Option<String>,
                #[serde(default)]
                text: Option<String>,
            },
        }
        match Form::deserialize(d)? {
            Form::Event(e) if e == "event" => Ok(Hover {
                event: true,
                ..Hover::default()
            }),
            Form::Event(e) => Err(serde::de::Error::custom(format!("hover {e:?}"))),
            Form::Look { color, text } => Ok(Hover {
                color,
                text,
                event: false,
            }),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Span {
    pub text: String,
    pub color: Option<String>,
    pub bold: bool,
}

/// One cell of a row of cells: a button (with an `id`), an icon button (`icon`), a check box
/// (`check`), a slider (`slider`, 0 to 1), a picture or text.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Cell {
    pub id: Option<String>,
    pub text: String,
    pub icon: Option<String>,
    pub tip: Option<String>,
    pub color: Option<String>,
    pub bold: bool,
    pub check: Option<bool>,
    /// A button that is on (its plate stays in the accent colour).
    pub checked: bool,
    pub slider: Option<f32>,
    pub image: Option<String>,
    pub height: Option<f32>,
    /// A fixed width in DIPs, and where the text sits in it: `left`, `center`, `right`.
    pub width: Option<f32>,
    pub align: Option<String>,
    pub fill: bool,
    pub under: bool,
    #[serde(deserialize_with = "blur")]
    pub blur: bool,
    pub hover: Option<Hover>,
    pub wheel: bool,
}

/// `blur` may hide a password: anything but `false` (`1`, `"yes"`, `null`) blurs.
fn blur<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(Value::deserialize(d)?.as_bool() != Some(false))
}

/// A line chart of `values`, `height` DIPs tall.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Chart {
    pub values: Vec<f32>,
    pub color: Option<String>,
    pub height: Option<f32>,
}

/// A drawing as wide as the fence and `height` DIPs tall: its steps in order.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Canvas {
    pub height: f32,
    pub draw: Vec<Step>,
}

/// One drawing step, in DIPs from the canvas' top left: `line: [x1, y1, x2, y2]`,
/// `rect: [x, y, w, h]` (`radius`), `circle: [x, y, r]`, `text` at `at: [x, y]` (`size`, `bold`),
/// or `image` (a file in the data folder) at `at: [x, y, w, h]`. `color` draws the outline or
/// the text, `fill` fills a rect or a circle, `width` is the outline's.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Step {
    pub line: Option<Vec<f32>>,
    pub rect: Option<Vec<f32>>,
    pub circle: Option<Vec<f32>>,
    pub text: Option<String>,
    pub image: Option<String>,
    pub at: Vec<f32>,
    pub color: Option<String>,
    pub fill: Option<String>,
    pub width: Option<f32>,
    pub radius: Option<f32>,
    pub size: Option<f32>,
    pub bold: bool,
}

/// `type: "note"`: the fence's note, which PecoFence edits and keeps; `type: "line"`: a one-line
/// box with `value` and a grey `placeholder` while it is empty; `type: "text"`: a box of `lines`
/// lines (2 to 12) that takes at most `max` characters, whose text the plugin keeps.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Input {
    #[serde(rename = "type", alias = "kind")]
    pub kind: String,
    pub id: Option<String>,
    pub value: String,
    pub placeholder: String,
    pub lines: usize,
    pub max: Option<usize>,
}

/// The most a multi-line box holds: 64 KB (in UTF-8), and as many characters.
pub const MAX_INPUT: usize = 64 * 1024;

/// `text` cut to at most `max` characters and [`MAX_INPUT`] bytes, at a character's end.
pub fn cut_input(text: &mut String, max: usize) {
    let end = (text.char_indices())
        .map(|(at, c)| at + c.len_utf8())
        .take(max)
        .take_while(|end| *end <= MAX_INPUT)
        .last()
        .unwrap_or(0);
    text.truncate(end);
}

fn limit_menu(menu: &mut Vec<MenuItem>) {
    menu.retain(|m| m.separator || (!m.id.is_empty() && !m.title.trim().is_empty()));
    menu.truncate(MAX_MENU);
    for m in menu {
        let line = m.title.lines().next().unwrap_or_default();
        m.title = line.chars().take(MAX_MENU_TEXT).collect();
    }
}

fn cut(text: &mut String) {
    if let Some((at, _)) = text.char_indices().nth(MAX_TEXT) {
        text.truncate(at);
    }
}

fn cut_some(text: &mut Option<String>) {
    if let Some(t) = text {
        cut(t);
    }
}

fn unit(v: Option<f32>) -> Option<f32> {
    v.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0))
}

fn height(v: Option<f32>) -> Option<f32> {
    v.filter(|v| v.is_finite())
        .map(|v| v.clamp(1.0, MAX_HEIGHT))
}

fn name(n: Option<String>) -> Option<String> {
    n.filter(|n| valid_plain_name(n))
}

impl Row {
    fn limit(&mut self) {
        cut(&mut self.text);
        cut_some(&mut self.short);
        cut_some(&mut self.tip);
        self.spans.truncate(MAX_PARTS);
        self.spans.iter_mut().for_each(|s| cut(&mut s.text));
        self.cells.truncate(MAX_PARTS);
        for c in &mut self.cells {
            cut(&mut c.text);
            cut_some(&mut c.tip);
            c.slider = unit(c.slider);
            c.image = name(c.image.take());
            c.height = height(c.height);
            c.width = c
                .width
                .filter(|w| w.is_finite())
                .map(|w| w.clamp(0.0, 4096.0));
        }
        self.image = name(self.image.take());
        self.height = height(self.height);
        self.progress = unit(self.progress);
        if let Some(chart) = &mut self.chart {
            chart.values.truncate(MAX_VALUES);
            chart.values.retain(|v| v.is_finite());
            chart.height = height(chart.height);
        }
        if let Some(canvas) = &mut self.canvas {
            canvas.height = height(Some(canvas.height)).unwrap_or(1.0);
            canvas.draw.truncate(MAX_VALUES);
            for step in &mut canvas.draw {
                cut_some(&mut step.text);
                step.image = name(step.image.take());
            }
        }
        if let Some(input) = &mut self.input {
            cut(&mut input.placeholder);
            if input.kind == "text" {
                input.lines = input.lines.clamp(2, 12);
                input.max = Some(input.max.unwrap_or(MAX_INPUT).clamp(1, MAX_INPUT));
                cut_input(&mut input.value, MAX_INPUT);
            } else {
                cut(&mut input.value);
            }
        }
        limit_menu(&mut self.menu);
        let looks = (self.cells.iter_mut().map(|c| &mut c.hover)).chain([&mut self.hover]);
        for hover in looks.flatten() {
            cut_some(&mut hover.text);
        }
    }
}

/// The view in `json`, cut to the limits above.
pub fn parse(json: &str) -> serde_json::Result<View> {
    read(json).map(|(view, _)| view)
}

/// [`parse`], and the fields left out because they were of the wrong type (`bold`,
/// `cells.id`, ...): the rest of their row, span, cell or drawing step shows. A row that is no
/// object is left out whole.
pub fn read(json: &str) -> serde_json::Result<(View, Vec<String>)> {
    let value: Value = serde_json::from_str(json)?;
    let rows = value.get("rows").and_then(|r| r.as_array());
    let (mut view, mut dropped) = (View::default(), Vec::new());
    for row in rows.into_iter().flatten().take(MAX_ROWS) {
        let row = Row::deserialize(row).ok().or_else(|| {
            let row = without_wrong_fields(row, &mut dropped)?;
            Row::deserialize(&row).ok()
        });
        if let Some(mut row) = row {
            row.limit();
            view.rows.push(row);
        }
    }
    let menu = value.get("menu").cloned().unwrap_or_default();
    view.menu = Vec::<MenuItem>::deserialize(menu).unwrap_or_default();
    limit_menu(&mut view.menu);
    dropped.sort();
    dropped.dedup();
    Ok((view, dropped))
}

/// `row` (one that does not read) without its fields of the wrong type, in its cells, spans
/// and drawing steps too.
fn without_wrong_fields(row: &Value, dropped: &mut Vec<String>) -> Option<Value> {
    let mut row = row.clone();
    if let Some(list) = row.get_mut("cells").and_then(Value::as_array_mut) {
        *list = (list.iter())
            .filter_map(|c| readable::<Cell>(c, "cells.", dropped))
            .collect();
    }
    if let Some(list) = row.get_mut("spans").and_then(Value::as_array_mut) {
        *list = (list.iter())
            .filter_map(|s| readable::<Span>(s, "spans.", dropped))
            .collect();
    }
    let draw = row.get_mut("canvas").and_then(|c| c.get_mut("draw"));
    if let Some(list) = draw.and_then(Value::as_array_mut) {
        *list = (list.iter())
            .filter_map(|s| readable::<Step>(s, "canvas.draw.", dropped))
            .collect();
    }
    readable::<Row>(&row, "", dropped)
}

/// `value` as a `T` reads it: whole when it does, else without the fields that do not read
/// (their names, after `path`, go into `dropped`); None when it is no object.
fn readable<T: DeserializeOwned>(
    value: &Value,
    path: &str,
    dropped: &mut Vec<String>,
) -> Option<Value> {
    if T::deserialize(value).is_ok() {
        return Some(value.clone());
    }
    let Some(fields) = value.as_object() else {
        let part = if path.is_empty() {
            "row"
        } else {
            path.trim_end_matches('.')
        };
        dropped.push(format!("{part} (not an object)"));
        return None;
    };
    let mut kept = serde_json::Map::new();
    for (k, v) in fields {
        let alone = Value::Object([(k.clone(), v.clone())].into_iter().collect());
        if T::deserialize(&alone).is_ok() {
            kept.insert(k.clone(), v.clone());
        } else {
            dropped.push(format!("{path}{k}"));
        }
    }
    Some(Value::Object(kept))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_of_a_row_is_read() {
        let view = parse(
            r##"{ "rows": [
            { "text": "12:00", "short": "12", "size": "fit", "bold": true, "color": "#ff0000",
              "icon": "\uE768", "tip": "a tip", "id": "t", "blur": true, "todo": false,
              "progress": 0.5, "pane": "p", "follow": "f" },
            { "spans": [{ "text": "a", "color": "red" }, { "text": "b", "bold": true }] },
            { "cells": [{ "id": "b", "text": "Go", "width": 40, "align": "right", "fill": true },
                        { "id": "c", "check": true, "text": "On", "under": true },
                        { "id": "s", "slider": 0.25 }, { "id": "i", "icon": "\uE768", "checked": true }] },
            { "image": "pic.png", "height": 80, "blur": true },
            { "chart": { "values": [1, 3, 2], "color": "blue", "height": 30 } },
            { "canvas": { "height": 50, "draw": [{ "line": [0, 0, 10, 10], "color": "red", "width": 2 },
               { "rect": [1, 2, 3, 4], "fill": "#00ff00", "radius": 2 }, { "circle": [5, 5, 4] },
               { "text": "hi", "at": [1, 1], "size": 12 }, { "image": "a.png", "at": [0, 0, 8, 8] }] } },
            { "input": { "id": "n", "type": "note" } },
            { "input": { "id": "q", "kind": "line", "value": "v", "placeholder": "Search" } },
            { "separator": true }
        ] }"##,
        )
        .unwrap();
        let r = &view.rows;
        assert_eq!(r.len(), 9);
        assert_eq!(
            (
                r[0].size.as_deref(),
                r[0].short.as_deref(),
                r[0].id.as_deref()
            ),
            (Some("fit"), Some("12"), Some("t"))
        );
        assert!(r[0].bold && r[0].blur && r[0].todo == Some(false));
        assert_eq!(
            (r[0].progress, r[0].pane.as_deref()),
            (Some(0.5), Some("p"))
        );
        assert_eq!(r[1].spans[1].text, "b");
        assert!(r[1].spans[1].bold && r[1].spans[0].color.as_deref() == Some("red"));
        let c = &r[2].cells;
        assert_eq!(
            (c[0].width, c[0].align.as_deref(), c[1].check, c[2].slider),
            (Some(40.0), Some("right"), Some(true), Some(0.25))
        );
        assert!(c[0].fill && c[1].under && c[3].checked && c[3].icon.is_some());
        assert_eq!(
            (r[3].image.as_deref(), r[3].height),
            (Some("pic.png"), Some(80.0))
        );
        assert_eq!(r[4].chart.as_ref().unwrap().values, vec![1.0, 3.0, 2.0]);
        let canvas = r[5].canvas.as_ref().unwrap();
        assert_eq!((canvas.height, canvas.draw.len()), (50.0, 5));
        assert_eq!(canvas.draw[3].text.as_deref(), Some("hi"));
        assert_eq!(r[6].input.as_ref().unwrap().kind, "note");
        let line = r[7].input.as_ref().unwrap();
        assert_eq!(
            (
                line.kind.as_str(),
                line.value.as_str(),
                line.placeholder.as_str()
            ),
            ("line", "v", "Search")
        );
        assert!(r[8].separator);
    }

    #[test]
    fn views_are_cut_to_the_limits() {
        let rows: Vec<String> = (0..400).map(|i| format!(r#"{{"text":"{i}"}}"#)).collect();
        let view = parse(&format!(r#"{{"rows":[{}]}}"#, rows.join(","))).unwrap();
        assert_eq!(
            (view.rows.len(), view.rows[299].text.as_str()),
            (300, "299")
        );
        let long = "字".repeat(5000);
        let values: Vec<String> = (0..1500).map(|i| i.to_string()).collect();
        let json = format!(
            r#"{{"rows":[{{"text":"{long}","progress":3,"height":99999,
                "cells":[{{"slider":-2}},{{"slider":7}}],"chart":{{"values":[{}]}},
                "canvas":{{"height":-5,"draw":[{{"text":"{long}"}}]}}}}]}}"#,
            values.join(",")
        );
        let view = parse(&json).unwrap();
        let r = &view.rows[0];
        assert_eq!(r.text.chars().count(), 4096);
        assert_eq!((r.progress, r.height), (Some(1.0), Some(2000.0)));
        assert_eq!(
            (r.cells[0].slider, r.cells[1].slider),
            (Some(0.0), Some(1.0))
        );
        assert_eq!(r.chart.as_ref().unwrap().values.len(), 1000);
        let canvas = r.canvas.as_ref().unwrap();
        assert_eq!(canvas.height, 1.0);
        assert_eq!(canvas.draw[0].text.as_ref().unwrap().chars().count(), 4096);
        let cells: Vec<String> = (0..100).map(|_| r#"{"text":"x"}"#.to_string()).collect();
        let view = parse(&format!(
            r#"{{"rows":[{{"cells":[{}]}}]}}"#,
            cells.join(",")
        ))
        .unwrap();
        assert_eq!(view.rows[0].cells.len(), 64);
    }

    #[test]
    fn pictures_come_from_the_data_folder_only() {
        for (name, ok) in [
            ("clip-1.png", true),
            ("a_b.v2.jpg", true),
            ("../config.json", false),
            ("C:\\Windows\\a.png", false),
            (".hidden.png", false),
            ("sub/a.png", false),
            ("", false),
            (&"a".repeat(65), false),
            (&"a".repeat(64), true),
        ] {
            assert_eq!(valid_plain_name(name), ok, "{name}");
            let json = format!(
                r#"{{"rows":[{{"image":{0:?},"cells":[{{"image":{0:?}}}],
                    "canvas":{{"height":9,"draw":[{{"image":{0:?}}}]}}}}]}}"#,
                name
            );
            let row = &parse(&json).unwrap().rows[0];
            let kept = name.to_string();
            let expect = ok.then_some(&kept);
            assert_eq!(row.image.as_ref(), expect, "{name}");
            assert_eq!(row.cells[0].image.as_ref(), expect, "{name}");
            assert_eq!(row.canvas.as_ref().unwrap().draw[0].image.as_ref(), expect);
        }
    }

    #[test]
    fn a_field_of_the_wrong_type_is_left_out_and_the_rest_shows() {
        let (view, dropped) = read(
            r#"{"rows":[{"text":"a","id":5,"bold":null,"color":"red"},{"bold":"yes"},7,
                {"text":"c","cells":[{"id":"b","text":"Go","fill":"yes"},{"id":1,"text":"x"},3],
                 "spans":[{"text":"s","bold":1}],"canvas":{"height":9,"draw":[{"line":[0,0,1,1]},{"text":"t","at":"x"}]}}]}"#,
        )
        .unwrap();
        let texts: Vec<&str> = view.rows.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, ["a", "", "c"]);
        let (a, c) = (&view.rows[0], &view.rows[2]);
        assert_eq!(
            (a.id.as_deref(), a.bold, a.color.as_deref()),
            (None, false, Some("red"))
        );
        let cells: Vec<(Option<&str>, &str)> = c
            .cells
            .iter()
            .map(|c| (c.id.as_deref(), c.text.as_str()))
            .collect();
        assert_eq!(cells, [(Some("b"), "Go"), (None, "x")]);
        assert_eq!(c.spans[0].text, "s");
        let draw = &c.canvas.as_ref().unwrap().draw;
        assert_eq!((draw.len(), draw[1].text.as_deref()), (2, Some("t")));
        assert_eq!(
            dropped,
            [
                "bold",
                "canvas.draw.at",
                "cells (not an object)",
                "cells.fill",
                "cells.id",
                "id",
                "row (not an object)",
                "spans.bold"
            ]
        );
        assert!(parse("not json").is_err());
        assert!(parse("{}").unwrap().rows.is_empty());
    }

    #[test]
    fn row_menus_hover_wheel_and_the_text_box_are_read_to_their_limits() {
        let items: Vec<String> = (0..12)
            .map(|i| format!(r#"{{"id":"m{i}","title":"{}"}}"#, "字".repeat(50)))
            .collect();
        let long = "長".repeat(30_000);
        let (view, dropped) = read(&format!(
            r#"{{"rows":[
            {{"id":"r1","menu":[{0},{{"title":"no id"}},{{"separator":true}}],
              "hover":{{"color":"red","text":"Open"}},"wheel":true}},
            {{"id":"r2","hover":"event","cells":[{{"id":"v","wheel":true,"hover":{{"text":"x"}}}}]}},
            {{"text":"a","hover":"wave","menu":3}},
            {{"input":{{"type":"text","id":"t","lines":40,"max":0,"value":"v"}}}},
            {{"input":{{"type":"text","id":"t","value":"{long}"}}}},
            {{"input":{{"type":"line","value":"{long}"}}}}
        ]}}"#,
            items.join(",")
        ))
        .unwrap();
        let r = &view.rows;
        assert_eq!(r[0].menu.len(), MAX_MENU);
        assert!(r[0].menu.iter().all(|m| m.title.chars().count() == 40));
        let look = r[0].hover.as_ref().unwrap();
        assert_eq!(
            (look.color.as_deref(), look.text.as_deref(), look.event),
            (Some("red"), Some("Open"), false)
        );
        assert!(r[0].wheel && r[1].hover.as_ref().unwrap().event && r[1].cells[0].wheel);
        assert_eq!(
            r[1].cells[0].hover.as_ref().unwrap().text.as_deref(),
            Some("x")
        );
        // A hover of another word and a menu that is no list are left out; the row shows.
        assert_eq!(r[2].text, "a");
        assert!(r[2].hover.is_none() && r[2].menu.is_empty());
        assert_eq!(dropped, ["hover", "menu"]);
        let small = r[3].input.as_ref().unwrap();
        assert_eq!((small.lines, small.max), (12, Some(1)));
        // 64 KB of a multi-line box (three bytes a character), 4096 characters of a one-line one.
        let big = r[4].input.as_ref().unwrap();
        assert_eq!(
            (big.lines, big.max, big.value.len()),
            (2, Some(MAX_INPUT), 65_535)
        );
        assert_eq!(r[5].input.as_ref().unwrap().value.chars().count(), 4096);
        let mut text = "ab長c".to_string();
        cut_input(&mut text, 3);
        assert_eq!(text, "ab長");
    }

    #[test]
    fn a_blur_that_does_not_read_blurs() {
        let (view, dropped) = read(
            r#"{"rows":[{"text":"a","blur":1},{"text":"b","blur":"yes"},{"text":"c","blur":null},
                {"text":"d","blur":false},{"text":"e"},{"cells":[{"text":"f","blur":1},{"text":"g"}]}]}"#,
        )
        .unwrap();
        let rows: Vec<bool> = view.rows.iter().map(|r| r.blur).collect();
        assert_eq!(rows, [true, true, true, false, false, false]);
        let cells: Vec<bool> = view.rows[5].cells.iter().map(|c| c.blur).collect();
        assert_eq!((cells, dropped.len()), (vec![true, false], 0));
    }
}
