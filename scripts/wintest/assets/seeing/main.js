// The test plugin of tests/plugin-seeing.ps1: where the pointer, the keyboard focus and the text
// caret are, from the events and from the calls (read once a second), and the commands it got.
// Each row starts with a word the steps look for; a rectangle is "x,y,width,height".
const at = { pointer: "none", monitor: "none", focus: "none", caret: "none", pull: "none" };
const count = { pointer: 0, focus: 0 };
const commands = [];
const rect = (r) => (r ? `${r.x},${r.y},${r.width},${r.height}` : "none");
const show = () =>
  pf.show({
    rows: [
      { text: `pointer ${at.pointer} monitor ${at.monitor} events ${count.pointer}` },
      { text: `focus ${at.focus} caret ${at.caret} events ${count.focus}` },
      { text: `pull ${at.pull}` },
      { text: `commands ${commands.join(" ") || "none"}` },
    ],
  });

// The calls: pf.pointer, pf.focus and pf.caret, or the error a plugin without the permission gets.
async function pull() {
  try {
    const [p, f, c] = await Promise.all([pf.pointer(), pf.focus(), pf.caret()]);
    at.pull = `pointer ${p.x},${p.y} focus ${rect(f)} caret ${rect(c)}`;
  } catch (e) {
    at.pull = `refused ${e.message}`;
  }
  show();
}

pf.on("start", () => {
  show();
  pull();
  pf.every(1, pull);
});
pf.on("pointer", (p) => {
  at.pointer = `${p.x},${p.y}`;
  at.monitor = rect(p.monitor);
  count.pointer++;
  show();
});
pf.on("focus", (f) => {
  at.focus = rect(f.focus);
  at.caret = rect(f.caret);
  count.focus++;
  show();
});
pf.on("command", (c) => {
  commands.push(`${c.id}/${c.via}`);
  show();
});
