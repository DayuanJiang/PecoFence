// Daily log: a line typed into the box goes, with the time, into today's Markdown file in a folder
// the user picked (<folder>\2026-11\2026-11-16.md); the fence shows today's lines. The plugin may
// write only inside that folder (folders: "readwrite") and only data files.
const texts = () => pf.lang.startsWith("zh")
  ? { pick: "选择文件夹…", trash: "把今天的记录移到回收站", none: "点这里选一个文件夹，记录会写在那里。",
      hint: "写下现在在做的事，按 Ctrl+Enter 记下", empty: "今天还没有记录。", failed: "没有写进去：" }
  : { pick: "Choose a folder…", trash: "Move today's log to the Recycle Bin", none: "Click here to choose the folder the log goes into.",
      hint: "Write what you are doing, Ctrl+Enter to log it", empty: "Nothing logged today.", failed: "Not written: " };
let t = texts();
// The folder of this fence, kept in a data file named after the fence (it goes with the fence).
const kept = () => pf.fence.id + "-folder.txt";
let folder = null;
let lines = [];
let draft = "";
let problem = "";

const pad = (n) => String(n).padStart(2, "0");
function today() {
  const d = new Date(pf.time.now());
  const month = d.getFullYear() + "-" + pad(d.getMonth() + 1);
  const dir = folder + "\\" + month;
  return { dir, file: dir + "\\" + month + "-" + pad(d.getDate()) + ".md" };
}

function draw() {
  const rows = [];
  if (!folder) {
    rows.push({ id: "pick", text: t.none, color: "blue" });
  } else {
    rows.push({ text: folder.split("\\").pop(), color: "gray", size: "small", tip: folder });
    rows.push({ input: { type: "text", id: "entry", lines: 3, value: draft, placeholder: t.hint, max: 2000 } });
    if (problem) rows.push({ text: t.failed + problem, color: "red" });
    if (!lines.length) rows.push({ text: t.empty, color: "gray" });
    for (const l of lines.slice(-30)) rows.push({ text: l });
  }
  pf.show({ rows, menu: [{ id: "pick", title: t.pick }, { id: "trash", title: t.trash, disabled: !folder }] });
}

async function load() {
  folder = (await pf.data.read(kept())) || null;
  lines = [];
  if (folder) {
    const text = await pf.folder.read(today().file).catch(() => "");
    lines = text.split("\n").filter((l) => l.startsWith("- ")).map((l) => l.slice(2));
  }
  draw();
}

async function pick() {
  const picked = await pf.folder.pick();
  if (!picked) return;
  await pf.data.write(kept(), picked);
  problem = "";
  load();
}

async function log(text) {
  const line = pf.time.format(pf.time.now(), "time") + " " + text.replace(/\s*\n\s*/g, " ").trim();
  const { dir, file } = today();
  try {
    await pf.folder.mkdir(dir);
    const before = await pf.folder.read(file).catch(() => "");
    await pf.folder.write(file, (before || "# " + file.split("\\").pop().slice(0, -3) + "\n\n") + "- " + line + "\n");
    draft = "";
    problem = "";
  } catch (e) {
    problem = e.message;
  }
  load();
}

pf.on("start", load);
pf.on("lang", () => { t = texts(); draw(); });
pf.on("click", (c) => { if (c.id === "pick") pick(); });
pf.on("change", (c) => { if (c.id === "entry") draft = c.text; });
pf.on("submit", (c) => { if (c.id === "entry" && draft.trim()) log(draft); });
pf.on("command", async (c) => {
  if (c.id === "pick") pick();
  if (c.id === "trash" && folder) {
    await pf.folder.trash(today().file).catch((e) => { problem = e.message; });
    load();
  }
});
