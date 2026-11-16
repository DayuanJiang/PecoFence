// The test plugin of plugin-ui.ps1: a row with a menu of its own, rows that hear of the pointer
// and a row that looks different under it, a cell and a row that take the wheel, a box of
// several lines, a gray block for the colour check, and rows enough to scroll. The first row
// says what the plugin got last, the second counts the paced events.
let last = "-";
let level = 5;
let text = "first line\nsecond line";
const count = { enter: 0, leave: 0, wheel: 0, change: 0, submit: 0 };

function show() {
  const rows = [
    { text: "event: " + last, color: "gray" },
    { text: "counts: " + Object.entries(count).map(([k, v]) => k + " " + v).join(", "), color: "gray" },
    {
      id: "item1",
      text: "Item one",
      menu: [{ id: "pin", title: "Pin it" }, { separator: true }, { id: "remove", title: "Remove it" }],
    },
    { id: "item2", text: "Item two" },
    { id: "h1", text: "Hover row one", hover: "event" },
    { id: "h2", text: "Hover row two", hover: "event" },
    { text: "plain look", hover: { color: "red", text: "LOOK UNDER POINTER" } },
    { cells: [{ text: "Level" }, { id: "lvl", text: "level " + level, wheel: true, width: 80 }] },
    { id: "wrow", text: "wheel row " + level, wheel: true },
    { input: { type: "text", id: "t", lines: 4, value: text, placeholder: "Write here", max: 200 } },
    { text: "got: " + text.replace(/\n/g, "|") },
    { text: "██████", color: "gray", size: "huge", tip: "gray block" },
  ];
  for (let i = 1; i <= 30; i++) rows.push({ text: "filler " + i });
  pf.show({ menu: [{ id: "fence-cmd", title: "Fence command" }], rows });
}

pf.on("start", show);
pf.on("command", (e) => {
  last = "command " + e.id + " " + e.via + " " + (e.row === undefined ? "-" : e.row);
  show();
});
for (const name of ["enter", "leave"]) {
  pf.on(name, (e) => {
    count[name]++;
    last = name + " " + e.row;
    show();
  });
}
pf.on("wheel", (e) => {
  count.wheel++;
  level += e.steps;
  last = "wheel " + e.id + " " + e.steps;
  show();
});
pf.on("change", (e) => {
  if (e.id !== "t") return;
  count.change++;
  text = e.text;
  last = "change " + e.id + " " + e.text.length;
  show();
});
pf.on("submit", (e) => {
  count.submit++;
  last = "submit " + e.id;
  show();
});
