// 正在播放: the song or video playing now (Windows' media session) with its cover, buttons for
// previous, play or pause and next, and a line for each program of `apps` that runs.
"use strict";
const T = {
  en: { none: "Nothing is playing", paused: "Paused", prev: "Previous", play: "Play or pause", next: "Next", runs: "{0} is running" },
  "zh-CN": { none: "现在没有在播放的内容", paused: "已暂停", prev: "上一首", play: "播放或暂停", next: "下一首", runs: "{0} 在运行" },
  "zh-TW": { none: "現在沒有在播放的內容", paused: "已暫停", prev: "上一首", play: "播放或暫停", next: "下一首", runs: "{0} 正在執行" },
  ja: { none: "再生中のものはありません", paused: "一時停止中", prev: "前へ", play: "再生または一時停止", next: "次へ", runs: "{0} が起動しています" },
  ko: { none: "재생 중인 항목이 없습니다", paused: "일시 정지됨", prev: "이전", play: "재생 또는 일시 정지", next: "다음", runs: "{0} 실행 중" },
  de: { none: "Gerade läuft nichts", paused: "Angehalten", prev: "Zurück", play: "Wiedergabe oder Pause", next: "Weiter", runs: "{0} läuft" },
  fr: { none: "Rien en lecture", paused: "En pause", prev: "Précédent", play: "Lecture ou pause", next: "Suivant", runs: "{0} est ouvert" },
  es: { none: "No se reproduce nada", paused: "En pausa", prev: "Anterior", play: "Reproducir o pausar", next: "Siguiente", runs: "{0} está abierto" },
  "pt-BR": { none: "Nada está tocando", paused: "Pausado", prev: "Anterior", play: "Tocar ou pausar", next: "Próxima", runs: "{0} está aberto" },
  ru: { none: "Сейчас ничего не играет", paused: "Пауза", prev: "Назад", play: "Воспроизвести или пауза", next: "Вперёд", runs: "{0} запущен" },
  it: { none: "Non è in riproduzione niente", paused: "In pausa", prev: "Precedente", play: "Riproduci o metti in pausa", next: "Successivo", runs: "{0} è aperto" },
};
const t = T[pf.lang] || T[pf.lang.split("-")[0]] || T.en;
// Segoe Fluent Icons: previous, play, pause, next.
const [PREV, PLAY, PAUSE, NEXT] = [0xe892, 0xe768, 0xe769, 0xe893].map((c) => String.fromCharCode(c));

let now = null, running = [];
// spotify.exe -> Spotify
const name = (exe) => exe.replace(/\.exe$/i, "").replace(/^./, (c) => c.toUpperCase());
const clock = (s) => Math.floor(s / 60) + ":" + String(Math.floor(s % 60)).padStart(2, "0");

function draw() {
  const rows = [];
  if (now) {
    if (now.app) rows.push({ text: name(now.app), size: "small", color: "gray" });
    if (now.cover) rows.push({ image: now.cover, height: 140 });
    rows.push({ text: now.title || "…", bold: true });
    const by = [now.artist, now.album].filter(Boolean).join(" · ");
    if (by) rows.push({ text: by, color: "gray" });
    const time = now.duration > 0 ? clock(now.position) + " / " + clock(now.duration) : "";
    const state = [now.playing ? "" : t.paused, time].filter(Boolean).join(" · ");
    if (state) rows.push({ text: state, size: "small", progress: now.duration > 0 ? now.position / now.duration : undefined });
    rows.push({ cells: [
      { id: "previous", icon: PREV, tip: t.prev },
      { id: "playpause", icon: now.playing ? PAUSE : PLAY, tip: t.play },
      { id: "next", icon: NEXT, tip: t.next },
    ] });
  } else {
    rows.push({ text: t.none, color: "gray" });
  }
  for (const exe of running) rows.push({ text: t.runs.replace("{0}", name(exe)), size: "small" });
  pf.show({ rows });
}

async function read() {
  now = await pf.media.current();
  draw();
}
async function apps() {
  running = await pf.apps.running();
  draw();
}

pf.on("media", read);
pf.on("show", () => { read(); apps(); });
pf.on("click", (e) => {
  if (!now || !["previous", "playpause", "next"].includes(e.id)) return;
  pf.media.control(e.id);
  // The button turns at once; the next media event says what the app did.
  if (e.id === "playpause") { now.playing = !now.playing; draw(); }
});
pf.every(5, apps);
draw();
read();
apps();
