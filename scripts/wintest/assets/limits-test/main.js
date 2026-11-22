// The test plugin of tests/plugin-limits3.ps1: the steps write a JSON list of calls into its data
// folder (cmd.json, [{ k, call, args }], call such as "http.get" or "run"), which it reads every
// second; it runs each call once and shows one row per key, "<k> ok <answer>" or
// "<k> error <message>", sorted by key. An http answer shows as [status, body length, start of
// the body]; a run as [code, output].
const results = {};
let last = "";
const show = () => pf.show({ rows: Object.keys(results).sort().map((k) => ({ text: k + " " + results[k] })) });
const brief = (call, v) => {
  if (call.startsWith("http.")) return [v.status, v.body.length, v.body.slice(0, 20)];
  if (call === "run") return [v.code, v.output.trim()];
  return v;
};
const run = async (c) => {
  try {
    const [group, name] = c.call.split(".");
    const f = name ? pf[group][name] : pf[group];
    results[c.k] = "ok " + JSON.stringify(brief(c.call, await f(...c.args)));
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
