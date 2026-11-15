// `pf`, what a light-layer plugin may use, over the host's functions in `__pf`. Runs before
// the plugin's main.js; afterwards no code can be made from text (no eval, no Function).
"use strict";
(() => {
  const n = globalThis.__pf;
  delete globalThis.__pf;
  const handlers = {}, timers = new Map(), waiting = new Map();
  let nextTimer = 1;
  // A call PecoFence does on a worker thread: a refused one throws at once, else its answer
  // comes as a Promise.
  const call = (name, args) => {
    const id = n.call(name, JSON.stringify(args));
    return new Promise((resolve, reject) => waiting.set(id, [resolve, reject]));
  };
  const timer = (seconds, fn, align, once) => {
    const id = nextTimer++;
    n.timer(id, Number(seconds), String(align || ""), once);
    timers.set(id, fn);
    return id;
  };
  const pf = {
    lang: n.lang,
    theme: n.theme,
    settings: JSON.parse(n.settings),
    fence: JSON.parse(n.fence),
    on(name, fn) { (handlers[name] = handlers[name] || []).push(fn); },
    show(view) { n.show(JSON.stringify(view)); },
    // A web plugin's main.js: data for its page (the page gets a `message` event).
    post(data) { n.post(JSON.stringify(data === undefined ? null : data)); },
    every: (seconds, fn, options) => timer(seconds, fn, options && options.align, false),
    after: (seconds, fn) => timer(seconds, fn, "", true),
    cancel(id) { if (timers.delete(id)) n.cancel(id); },
    time: Object.freeze({
      now: () => Date.now(),
      format: (ms, kind) => n.format(Number(ms), String(kind || "")),
    }),
    open: (url) => call("open", [String(url)]),
    openSystem: (name) => call("openSystem", [String(name)]),
    run: (program, args) => call("run", [String(program), (args || []).map(String)]),
    notify: (title, body, options) => call("notify", [String(title), String(body || ""), options || {}]),
  };
  for (const [group, names] of Object.entries({
    http: ["get", "post"],
    data: ["read", "write", "list", "remove"],
    clipboard: ["text", "files", "image", "setText", "setFiles", "setImage"],
    sys: ["cpu", "memory", "drives", "ip", "network", "battery"],
    folder: ["pick", "list", "read"],
    launch: ["pick", "add", "list", "remove", "open"],
  })) {
    pf[group] = Object.freeze(Object.fromEntries(names.map((k) => [k, (...args) => call(`${group}.${k}`, args)])));
  }
  const fixed = (name, value) => Object.defineProperty(globalThis, name, { value });
  fixed("pf", pf);
  fixed("__pf_emit", (name, json) => {
    const data = json === undefined ? undefined : JSON.parse(json);
    if (name === "settings") pf.settings = data;
    if (name === "resize") Object.assign(pf.fence, data);
    if (name === "theme") pf.theme = data;
    for (const fn of handlers[name] || []) fn(data);
  });
  fixed("__pf_tick", (id, once) => {
    const fn = timers.get(id);
    if (once) timers.delete(id);
    if (fn) fn();
  });
  fixed("__pf_done", (id, ok, json) => {
    const w = waiting.get(id);
    if (!w) return;
    waiting.delete(id);
    if (ok) w[0](JSON.parse(json));
    else w[1](new Error(json));
  });
  const refuse = function () { throw new TypeError("not available to plugins"); };
  refuse.prototype = Function.prototype;
  for (const f of [function () {}, async function () {}, function* () {}, async function* () {}]) {
    Object.defineProperty(Object.getPrototypeOf(f), "constructor", { value: refuse });
  }
  fixed("Function", refuse);
  delete globalThis.eval;
})();
