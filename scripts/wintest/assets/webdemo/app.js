// The test web plugin: a button that asks PecoFence for the time (pf.time.now), a text box, a
// list to scroll. window.demo holds what a test reads; window.selfTest() tries what the plugin
// must not be able to do and says what happened.
"use strict";
const demo = (window.demo = { clicks: 0, now: null, violations: [] });
const list = document.getElementById("list");
for (let i = 1; i <= 40; i++) {
  const li = document.createElement("li");
  li.textContent = `第 ${i} 行  Row ${i}`;
  list.appendChild(li);
}
document.getElementById("btn").addEventListener("click", async () => {
  demo.clicks++;
  try {
    demo.now = await window.pf.time.now();
    document.getElementById("out").textContent = `${new Date(demo.now).toLocaleTimeString()}（第 ${demo.clicks} 次）`;
  } catch (e) {
    document.getElementById("out").textContent = `pf 出错：${e.message}`;
  }
});
document.addEventListener("securitypolicyviolation", (e) => {
  demo.violations.push(`${e.effectiveDirective} ${e.blockedURI}`);
});

const settle = (p, ms = 4000) =>
  Promise.race([p, new Promise((ok) => setTimeout(() => ok("no answer"), ms))]);

window.selfTest = async () => {
  const r = {};
  r.fetchUndeclared = await settle(fetch("https://example.com/", { mode: "no-cors" }).then(() => "loaded", (e) => `refused: ${e.name}`));
  r.fetchDeclared = await settle(fetch("https://example.org/", { mode: "no-cors" }).then(() => "loaded", (e) => `refused: ${e.name}`));
  r.webSocket = await settle(new Promise((ok) => {
    try {
      const ws = new WebSocket("wss://echo.websocket.org/");
      ws.onopen = () => ok("open");
      ws.onerror = () => ok("error");
    } catch (e) { ok(`threw ${e.name}`); }
  }));
  r.windowOpen = String(window.open("https://example.com/", "_blank"));
  r.geolocation = await settle(new Promise((ok) => navigator.geolocation.getCurrentPosition(() => ok("granted"), (e) => ok(`denied code ${e.code}`))));
  r.notification = await settle(Notification.requestPermission());
  r.clipboardRead = await settle(navigator.clipboard.readText().then(() => "read", (e) => `refused: ${e.name}`));
  try { r.eval = String(eval("1 + 1")); } catch (e) { r.eval = `refused: ${e.name}`; }
  r.webrtc = await settle(new Promise((ok) => {
    try {
      const pc = new RTCPeerConnection({ iceServers: [{ urls: "stun:stun.l.google.com:19302" }] });
      const found = [];
      const done = (why) => { pc.close(); ok(`candidates: ${found.join(",") || "none"}${why}`); };
      pc.onicecandidate = (e) => { if (e.candidate) found.push(e.candidate.type || "?"); else done(""); };
      pc.createDataChannel("x");
      pc.createOffer().then((o) => pc.setLocalDescription(o));
      setTimeout(() => done(" (timeout)"), 3500);
    } catch (e) { ok(`threw ${e.name}`); }
  }), 5000);
  // A fresh frame of the same page has its own globals.
  const f = document.createElement("iframe");
  document.body.appendChild(f);
  r.webrtcInFrame = typeof (f.contentWindow && f.contentWindow.RTCPeerConnection);
  f.remove();
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob(["x"], { type: "text/plain" }));
  a.download = "x.txt";
  a.click();
  r.violations = demo.violations.slice();
  return r;
};
// The navigation test unloads the page if it is not refused: run it last, alone.
window.tryNavigate = () => { location.href = "https://example.com/"; return location.href; };

// 网页层 calls and events (web-plugin.ps1 part C).
demo.ticks = 0;
demo.events = [];
demo.messages = [];
demo.nomenu = 0;
setInterval(() => demo.ticks++, 1000);
for (const name of ["show", "hide", "resize", "theme", "settings", "timechange", "clipboard"]) {
  pf.on(name, (data) => demo.events.push(name === "theme" ? `theme:${data}` : name));
}
pf.on("message", (m) => demo.messages.push(m));
document.getElementById("nomenu").addEventListener("contextmenu", (e) => { e.preventDefault(); demo.nomenu++; });
document.getElementById("open").addEventListener("click", () => {
  // An undeclared host: the click lets the call through, the host check refuses it.
  pf.open("https://example.com/").then(() => (demo.open = "opened"), (e) => (demo.open = `refused: ${e.message}`));
});
const outcome = (p) => settle(p.then((v) => ({ ok: v }), (e) => ({ refused: e.message })), 8000);
window.callTest = async () => ({
  now: await outcome(pf.time.now()),
  declared: await outcome(pf.http.get("https://example.org/")),
  undeclared: await outcome(pf.http.get("https://example.com/")),
  clipboard: await outcome(pf.clipboard.text()),
  openNoClick: await outcome(pf.open("https://example.org/")),
  ownFile: await settle(fetch("plugin.json").then((r) => r.json()).then((m) => m.id, (e) => `refused: ${e.name}`)),
  theme: pf.theme,
  lang: pf.lang,
  fence: pf.fence,
  frozen: Object.isFrozen(pf) && Object.isFrozen(pf.http),
});
