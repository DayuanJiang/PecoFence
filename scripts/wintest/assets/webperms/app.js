// The test web plugin of plugin-web-perms.ps1: each window.t.* tries one thing a page may ask the
// browser for and answers with what happened, as text the steps read over CDP. Run with a user
// gesture (CDP userGesture) where the browser needs one.
"use strict";
const settle = (p, ms = 4000) =>
  Promise.race([p, new Promise((ok) => setTimeout(() => ok("pending"), ms))]);
const stream = async (p) => {
  const s = await settle(p);
  if (s === "pending") return "pending";
  const tracks = s.getTracks().map((t) => `${t.kind}:${t.label}`);
  s.getTracks().forEach((t) => t.stop());
  return `stream ${tracks.join(",")}`;
};
const why = (e) => `${e.name}: ${e.message}`;

window.t = {
  screen: () => stream(navigator.mediaDevices.getDisplayMedia({ video: true })).catch(why),
  camera: () => stream(navigator.mediaDevices.getUserMedia({ video: true })).catch(why),
  microphone: () => stream(navigator.mediaDevices.getUserMedia({ audio: true })).catch(why),
  location: () =>
    settle(
      new Promise((ok) =>
        navigator.geolocation.getCurrentPosition(
          () => ok("position"),
          (e) => ok(`error ${e.code}: ${e.message}`),
          { timeout: 3500 },
        ),
      ),
    ),
  notification: async () => {
    try {
      const asked = await settle(Notification.requestPermission());
      if (asked !== "granted") return `permission ${asked}`;
      const n = new Notification("PecoFence test", { body: "网页通知 web notification" });
      return await settle(
        new Promise((ok) => {
          n.onshow = () => ok("shown");
          n.onerror = () => ok("error");
        }),
      );
    } catch (e) {
      return why(e);
    }
  },
  // Two peer connections of this page talk over a data channel (host candidates only).
  webrtc: async () => {
    if (typeof RTCPeerConnection !== "function") return `no RTCPeerConnection (${typeof RTCPeerConnection})`;
    try {
      const a = new RTCPeerConnection(), b = new RTCPeerConnection();
      a.onicecandidate = (e) => e.candidate && b.addIceCandidate(e.candidate);
      b.onicecandidate = (e) => e.candidate && a.addIceCandidate(e.candidate);
      const ch = a.createDataChannel("t");
      const got = new Promise((ok) => {
        b.ondatachannel = (e) => (e.channel.onmessage = (m) => ok(`message ${m.data}`));
      });
      ch.onopen = () => ch.send("hi");
      await a.setLocalDescription(await a.createOffer());
      await b.setRemoteDescription(a.localDescription);
      await b.setLocalDescription(await b.createAnswer());
      await a.setRemoteDescription(b.localDescription);
      const r = await settle(got, 6000);
      a.close();
      b.close();
      return r === "pending" ? `no message (ice ${a.iceConnectionState})` : r;
    } catch (e) {
      return why(e);
    }
  },
  // Inline code of the page's own file runs; code made at run time from text does not.
  inline: async () => {
    const r = {};
    r.inlineScript = window.inlineRan === true;
    r.inlineStyle = getComputedStyle(document.getElementById("styled")).color;
    r.styleAttribute = getComputedStyle(document.getElementById("styled")).width;
    document.getElementById("handler").click();
    r.handler = window.handlerRan === 1;
    const data = document.getElementById("dataimg");
    r.dataImage = data.complete && data.naturalWidth === 1;
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 2;
    const blob = await new Promise((ok) => canvas.toBlob(ok));
    const img = document.getElementById("blobimg");
    r.blobImage = await settle(
      new Promise((ok) => {
        img.onload = () => ok(img.naturalWidth === 2);
        img.onerror = () => ok(false);
        img.src = URL.createObjectURL(blob);
      }),
      3000,
    );
    const s = document.createElement("script");
    s.textContent = "window.madeAtRunTime = true;";
    document.body.appendChild(s);
    r.runtimeScript = window.madeAtRunTime === true;
    try {
      // eslint-disable-next-line no-eval
      r.evaluated = (0, eval)("1 + 1") === 2;
    } catch (e) {
      r.evaluated = false;
    }
    r.blobWorker = await settle(
      new Promise((ok) => {
        try {
          const w = new Worker(URL.createObjectURL(new Blob(["postMessage(1)"])));
          w.onmessage = () => ok(true);
          w.onerror = () => ok(false);
        } catch (e) {
          ok(false);
        }
      }),
      3000,
    );
    return r;
  },
};
document.getElementById("out").textContent = "app.js 已运行";
