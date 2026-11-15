// Launcher: the programs, files and folders the user put into this fence; a click opens one.
// It knows each item by its id, name and picture only: PecoFence keeps the paths and opens them
// as a fence icon opens.
const zh = pf.lang.startsWith("zh");
const t = zh
  ? { program: "添加程序…", file: "添加文件…", empty: "把程序、文件或文件夹拖到这里，或者右键这个栅栏来添加。" }
  : { program: "Add a program…", file: "Add a file…", empty: "Drop programs, files or folders here, or right-click this fence to add one." };
let items = [];
let last = { id: "", at: 0 };

function draw() {
  const rows = items.map((it) => ({
    cells: [
      it.icon ? { id: it.id, image: it.icon, width: 20, height: 20 } : { id: it.id, icon: "\uE8A5" },
      { id: it.id, text: it.name, fill: true },
    ],
  }));
  if (!rows.length) rows.push({ text: t.empty });
  pf.show({ rows, menu: [{ id: "program", title: t.program }, { id: "file", title: t.file }] });
}

async function load() {
  items = await pf.launch.list();
  draw();
}

pf.on("start", load);
// An item removed in the plugin's details.
pf.on("settings", load);
pf.on("command", async (c) => {
  // `show` comes from its shortcut: PecoFence has shown the fence already.
  if ((c.id === "program" || c.id === "file") && (await pf.launch.pick(c.id))) load();
});
pf.on("drop", async (d) => {
  // Added while the drop is handled: pf.launch.add works only here.
  const added = d.files.map((_, i) => pf.launch.add(i));
  await Promise.allSettled(added);
  load();
});
pf.on("click", (c) => {
  const now = pf.time.now();
  // The second click of a double-click opens nothing more.
  if (c.id === last.id && now - last.at < 600) return;
  last = { id: c.id, at: now };
  pf.launch.open(c.id).catch(() => {});
});
