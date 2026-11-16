// LAN probe (tests/plugin-lan.ps1): requests the address of the `url` setting at the start, on
// each settings change and on a click, and shows one line: "probe <n> <url> -> <status> <body>"
// or "probe <n> <url> -> error <message>", and the device it was told: "device=<host or none>".
"use strict";
let n = 0, line = "";
function draw() {
  pf.show({ rows: [{ id: "again", text: line || "probe idle" }, { text: "device=" + (pf.settings.host || "none") }] });
}
async function probe() {
  const url = pf.settings.url;
  if (!url) return draw();
  const k = ++n;
  try {
    const r = await pf.http.get(url);
    line = "probe " + k + " " + url + " -> " + r.status + " " + String(r.body).slice(0, 80);
  } catch (e) {
    line = "probe " + k + " " + url + " -> error " + e.message;
  }
  draw();
}
pf.on("settings", probe);
pf.on("click", probe);
draw();
probe();
