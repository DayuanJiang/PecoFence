// The test plugin of tests/plugin-folders.ps1: the steps write a JSON list of calls into its data
// folder (cmd.json, [{ k, op, path, text }]), which it reads every second; it runs each call
// once and shows one row per key, "<k> ok <answer>" or "<k> error <message>", sorted by key.
const results = {};
let last = "";
const show = () => pf.show({ rows: Object.keys(results).sort().map((k) => ({ text: k + " " + results[k] })) });
const run = async (c) => {
  try {
    const f = pf.folder[c.op];
    const v = await (c.op === "write" ? f(c.path, c.text) : c.op === "pick" ? f() : f(c.path));
    results[c.k] = "ok " + JSON.stringify(c.op === "list" ? (v.cut ? "cut after " + v.items.length : v.items.map((e) => e.name).sort()) : v);
  } catch (e) {
    results[c.k] = "error " + e.message;
  }
  show();
};
pf.on("start", show);
pf.every(1, async () => {
  const cmd = await pf.data.read("cmd.json");
  if (!cmd || cmd === last) return;
  last = cmd;
  for (const c of JSON.parse(cmd)) await run(c);
});
