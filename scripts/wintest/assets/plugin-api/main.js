// The test plugin of tests/plugin-api.ps1: calls every pf function and shows one row per check,
// "<name> <result>", sorted by name. The steps give it more to do by writing a command into its
// data folder (cmd.txt), which it reads every second.
const results = {};
// Rows the steps add as they are (a row with fields of the wrong type).
let extra = [];
const show = () => pf.show({ rows: Object.keys(results).sort().map((k) => ({ text: k + " " + results[k] })).concat(extra) });
const put = (k, v) => { results[k] = v; show(); };
// A call that should be refused: "refused <message>" when it throws at once.
const refused = (k, f) => {
  try {
    const p = f();
    put(k, "allowed");
    if (p && p.then) p.then((v) => put(k, "allowed " + JSON.stringify(v)), (e) => put(k, "rejected " + e.message));
  } catch (e) {
    put(k, "refused " + e.message);
  }
};
const check = async (k, f) => {
  try { put(k, "ok " + JSON.stringify(await f())); } catch (e) { put(k, "error " + e.message); }
};

pf.on("start", () => {
  check("http-declared", async () => (await pf.http.get("https://api.github.com/zen")).status);
  refused("http-undeclared", () => pf.http.get("https://example.com/"));
  check("http-redirect", async () => (await pf.http.get("https://api.github.com/repos/DayuanJiang/PecoFence/tarball/main")).status);
  // A user name, a backslash or a NUL before the path: the browser or WinHTTP would read another host.
  refused("http-userinfo", () => pf.http.get("https://evil.example\u0000@api.github.com/"));
  refused("open-backslash", () => pf.open("https://evil.com\\@github.com/"));
  refused("data-name-slash", () => pf.data.write("../x.txt", "a"));
  refused("data-name-dot", () => pf.data.write(".hidden", "a"));
  refused("data-name-device", () => pf.data.write("nul.txt", "a"));
  refused("data-name-dot-end", () => pf.data.write("a.", "a"));
  check("data-roundtrip", async () => {
    await pf.data.write("note.txt", "hello 你好");
    const text = await pf.data.read("note.txt");
    const listed = (await pf.data.list()).includes("note.txt");
    await pf.data.remove("note.txt");
    return [text, listed, await pf.data.read("note.txt")];
  });
  check("sys", async () => {
    const [cpu, memory, drives, ip, network, battery] = await Promise.all(
      ["cpu", "memory", "drives", "ip", "network", "battery"].map((n) => pf.sys[n]()));
    return { cpu: typeof cpu, memory: typeof memory, drives: drives.length, c: drives.some((d) => d.letter === "C" && d.size > 0), ip: typeof ip, network: network.length, battery };
  });
  refused("open-outside-click", () => pf.open("https://github.com/DayuanJiang/PecoFence"));
  refused("open-system-other", () => pf.openSystem("regedit"));
  check("run-switch-off", async () => (await pf.run("ipconfig.exe", ["/?"])).code);
  refused("run-undeclared", () => pf.run("whoami.exe", []));
  refused("folder-pick", () => pf.folder.pick());
  check("notify", () => pf.notify("Test card", "From the plugin-api test", { sound: true }));
  check("time", () => [typeof pf.time.now(), pf.time.format(Date.now()).length > 0]);
});

// The clipboard: what it holds each time it changed; a picture goes into the data folder.
let changes = 0;
pf.on("clipboard", async () => {
  const n = ++changes;
  const text = await pf.clipboard.text();
  const files = await pf.clipboard.files();
  put("clipboard-" + n, JSON.stringify({ text, files }));
  put("clipboard-count", n);
  if (!text && !files) check("image", () => pf.clipboard.image("picture.png"));
});

// Commands from the steps.
let seen = null;
const commands = {
  "read-text": () => check("cmd-read-text", () => pf.clipboard.text()),
  "set-text": () => check("cmd-set-text", () => pf.clipboard.setText("from the plugin 插件")),
  "set-image": () => check("cmd-set-image", () => pf.clipboard.setImage("picture.png")),
  "run": () => check("cmd-run", async () => { const r = await pf.run("ipconfig.exe", []); return [r.code, r.output.length > 100]; }),
  // cmd ends at once and leaves ping running with the output pipe: the answer must not wait for it.
  "run-child": () => check("cmd-run-child", async () => {
    const begun = Date.now();
    const r = await pf.run("cmd.exe", ["/c", "start", "/b", "ping", "-n", "40", "127.0.0.1"]);
    return [r.code, Date.now() - begun < 10000];
  }),
  "read-big": () => check("cmd-read-big", () => pf.data.read("big.txt")),
  // An error escapes the timer's handler: logged, and the plugin goes on.
  "throw": () => { throw new Error("thrown by the throw command"); },
  "bad-row": () => { extra = [{ text: "bad-row shown", id: 5, bold: null }]; show(); },
  // Twelve answers that each take 40 ms: more than 200 ms in a second.
  "busy": () => { for (let i = 0; i < 12; i++) pf.sys.memory().then(() => { const end = Date.now() + 40; while (Date.now() < end) {} }); },
  "hang": () => { for (;;) {} },
};
pf.on("start", async () => { seen = await pf.data.read("last.txt"); });
pf.every(1, async () => {
  const cmd = await pf.data.read("cmd.txt");
  if (!cmd || cmd === seen || !commands[cmd]) return;
  seen = cmd;
  // A command runs once, across restarts too; "hang" again after each restart.
  if (cmd !== "hang") await pf.data.write("last.txt", cmd);
  put("cmd", cmd);
  commands[cmd]();
});
