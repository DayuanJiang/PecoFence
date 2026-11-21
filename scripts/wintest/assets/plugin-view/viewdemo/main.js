// The test plugin of plugin-view.ps1: every part of a light plugin's view, on five pages the
// buttons of the first row switch. The second row says what the plugin got last.
let page = "rows";
let last = "-";
let on = true;
let level = 0.4;
let query = "";
const done = { todo1: false, todo2: true };

const nav = () => ({
  cells: ["rows", "cells", "draw", "input", "list"].map((p) => ({
    id: "page:" + p,
    text: p,
    checked: p === page,
  })),
});
const status = () => ({ text: "event: " + last, color: "gray" });

const pages = {
  rows: () => [
    { text: "12:34:56", short: "12:34", size: "fit" },
    { text: "small text", size: "small" },
    { text: "normal text" },
    { text: "large text", size: "large" },
    { text: "xlarge text", size: "xlarge" },
    { text: "huge", size: "huge" },
    { text: "bold red", bold: true, color: "red" },
    { text: "#1e90ff with an emoji", color: "#1e90ff", icon: "☀️" },
    { text: "Fluent icon and a tip", icon: "", tip: "a row tip" },
    { text: "click me", id: "row1", tip: "click or double-click" },
    { spans: [{ text: "CPU " }, { text: "23%", color: "green", bold: true }, { text: " and " }, { text: "warm", color: "orange" }] },
    { text: "secret 1234", blur: true },
    { separator: true },
    { text: "a to-do", todo: done.todo1, id: "todo1" },
    { text: "a done to-do", todo: done.todo2, id: "todo2" },
    { text: "Disk C: 62%", progress: 0.62 },
  ],
  cells: () => [
    {
      cells: [
        { id: "b1", text: "Button" },
        { id: "b2", icon: "", tip: "Play" },
        { id: "b3", text: "On", checked: true },
        { id: "c1", check: on, text: "Check" },
      ],
    },
    { cells: [{ text: "Vol" }, { id: "s1", slider: level }, { text: Math.round(level * 100) + "%", width: 40, align: "right" }] },
    {
      cells: [
        { id: "wide", text: "A long text that fills the room and ends with an ellipsis", fill: true },
        { id: "top", text: "Top" },
        { id: "under", text: "Under", under: true },
      ],
    },
    {
      cells: [
        { id: "img", image: "pic.png", height: 40, width: 80 },
        { id: "bc", text: "secret cell", blur: true },
        { id: "hang", text: "Hang" },
      ],
    },
  ],
  draw: () => [
    { image: "pic.png", height: 60, id: "imgrow" },
    { image: "pic.png", height: 40, blur: true },
    { id: "chart", chart: { values: [3, 5, 2, 8, 6, 9, 4, 7, 5, 10], color: "green", height: 40 } },
    {
      id: "canvas",
      canvas: {
        height: 70,
        draw: [
          { rect: [10, 10, 60, 40], fill: "#1e90ff", radius: 6 },
          { circle: [110, 30, 20], color: "red", width: 3 },
          { line: [150, 10, 220, 50], color: "orange", width: 2 },
          { text: "Hi 你好", at: [10, 52], size: 13, bold: true },
          { image: "pic.png", at: [230, 10, 40, 40] },
        ],
      },
    },
  ],
  input: () => [
    { input: { id: "q", type: "line", value: query, placeholder: "Type here" } },
    { text: "typed: " + query },
  ],
  list: () => {
    const rows = [{ text: "pinned header", bold: true }, { pane: "list", text: "Row 1", id: "r1" }];
    for (let i = 2; i <= 120; i++) rows.push({ text: "Row " + i, id: "r" + i });
    return rows;
  },
};

function show() {
  pf.show({ rows: [nav(), status(), ...pages[page]()] });
}

pf.on("start", show);
pf.on("click", (e) => {
  last = "click " + e.id;
  if (e.id.startsWith("page:")) page = e.id.slice(5);
  // Runs past the 50 ms PecoFence allows a call: the fence says why and offers 重新启动.
  if (e.id === "hang") for (;;);
  show();
});
pf.on("dblclick", (e) => {
  last = "dblclick " + e.id;
  show();
});
pf.on("change", (e) => {
  const v = typeof e.value === "number" ? e.value.toFixed(2) : String(e.value);
  last = "change " + e.id + " " + v;
  if (e.id === "c1") on = e.value;
  if (e.id === "s1") level = e.value;
  if (e.id in done) done[e.id] = e.value;
  show();
});
pf.on("text", (e) => {
  last = "text " + e.id + " " + e.text;
  if (e.id === "q") query = e.text;
  show();
});
