// `pf`, what a light-layer plugin may use, over the host's functions in `__pf`. Runs before
// the plugin's main.js; afterwards no code can be made from text (no eval, no Function).
"use strict";
(() => {
  const n = globalThis.__pf;
  delete globalThis.__pf;
  const handlers = {}, timers = new Map();
  let nextTimer = 1;
  const pf = {
    lang: n.lang,
    theme: n.theme,
    settings: JSON.parse(n.settings),
    fence: JSON.parse(n.fence),
    on(name, fn) { (handlers[name] = handlers[name] || []).push(fn); },
    show(view) { n.show(JSON.stringify(view)); },
    // A web plugin's main.js: data for its page (the page gets a `message` event).
    post(data) { n.post(JSON.stringify(data === undefined ? null : data)); },
    every(seconds, fn, options) {
      const id = nextTimer++;
      timers.set(id, fn);
      n.every(id, Number(seconds), String((options && options.align) || ""));
      return id;
    },
    cancel(id) { if (timers.delete(id)) n.cancel(id); },
    time: Object.freeze({
      now: () => Date.now(),
      format: (ms, kind) => n.format(Number(ms), String(kind)),
    }),
  };
  const fixed = (name, value) => Object.defineProperty(globalThis, name, { value });
  fixed("pf", pf);
  fixed("__pf_emit", (name, json) => {
    const data = json === undefined ? undefined : JSON.parse(json);
    if (name === "settings") pf.settings = data;
    if (name === "resize") Object.assign(pf.fence, data);
    for (const fn of handlers[name] || []) fn(data);
  });
  fixed("__pf_tick", (id) => { const fn = timers.get(id); if (fn) fn(); });
  const refuse = function () { throw new TypeError("not available to plugins"); };
  refuse.prototype = Function.prototype;
  for (const f of [function () {}, async function () {}, function* () {}, async function* () {}]) {
    Object.defineProperty(Object.getPrototypeOf(f), "constructor", { value: refuse });
  }
  fixed("Function", refuse);
  delete globalThis.eval;
})();
