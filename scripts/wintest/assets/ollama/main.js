// Ollama: the models of Ollama on this computer (http://127.0.0.1:11434, permissions.local),
// a click picks the one to ask, and the box asks it (Ctrl+Enter or 提问); the answer comes
// whole (no streaming), at most about 200 tokens.
"use strict";
const BASE = "http://127.0.0.1:11434";
const T = {
  en: { loading: "Looking for Ollama…", off: "Ollama does not answer on this computer. Start it, then click here.", none: "Ollama has no models yet. Get one with: ollama pull llama3.2", ask: "Ask {0} a question (Ctrl+Enter)", send: "Ask", thinking: "{0} is thinking…", failed: "No answer: {0}" },
  "zh-CN": { loading: "正在查找 Ollama…", off: "这台电脑上的 Ollama 没有回应。启动它以后点这里。", none: "Ollama 还没有模型。可以用 ollama pull llama3.2 下载一个。", ask: "向 {0} 提问（Ctrl+Enter）", send: "提问", thinking: "{0} 正在回答…", failed: "没有得到回答：{0}" },
  "zh-TW": { loading: "正在尋找 Ollama…", off: "這台電腦上的 Ollama 沒有回應。啟動它以後點這裡。", none: "Ollama 還沒有模型。可以用 ollama pull llama3.2 下載一個。", ask: "向 {0} 提問（Ctrl+Enter）", send: "提問", thinking: "{0} 正在回答…", failed: "沒有得到回答：{0}" },
  ja: { loading: "Ollama を探しています…", off: "このパソコンの Ollama が応答しません。起動してからここをクリックしてください。", none: "Ollama にはまだモデルがありません。ollama pull llama3.2 で取得できます。", ask: "{0} に質問する（Ctrl+Enter）", send: "質問", thinking: "{0} が回答中…", failed: "回答がありません：{0}" },
  ko: { loading: "Ollama를 찾는 중…", off: "이 컴퓨터의 Ollama가 응답하지 않습니다. 실행한 뒤 여기를 클릭하세요.", none: "Ollama에 아직 모델이 없습니다. ollama pull llama3.2 로 받을 수 있습니다.", ask: "{0}에게 질문 (Ctrl+Enter)", send: "질문", thinking: "{0}이(가) 답하는 중…", failed: "답이 없습니다: {0}" },
  de: { loading: "Ollama wird gesucht…", off: "Ollama antwortet auf diesem Computer nicht. Starten Sie es und klicken Sie dann hier.", none: "Ollama hat noch keine Modelle. Eines holen mit: ollama pull llama3.2", ask: "{0} etwas fragen (Strg+Eingabe)", send: "Fragen", thinking: "{0} denkt nach…", failed: "Keine Antwort: {0}" },
  fr: { loading: "Recherche d'Ollama…", off: "Ollama ne répond pas sur cet ordinateur. Lancez-le, puis cliquez ici.", none: "Ollama n'a pas encore de modèle. En obtenir un : ollama pull llama3.2", ask: "Poser une question à {0} (Ctrl+Entrée)", send: "Demander", thinking: "{0} réfléchit…", failed: "Pas de réponse : {0}" },
  es: { loading: "Buscando Ollama…", off: "Ollama no responde en este equipo. Ábrelo y luego haz clic aquí.", none: "Ollama aún no tiene modelos. Consigue uno con: ollama pull llama3.2", ask: "Pregunta a {0} (Ctrl+Intro)", send: "Preguntar", thinking: "{0} está pensando…", failed: "Sin respuesta: {0}" },
  "pt-BR": { loading: "Procurando o Ollama…", off: "O Ollama não responde neste computador. Abra-o e clique aqui.", none: "O Ollama ainda não tem modelos. Baixe um com: ollama pull llama3.2", ask: "Pergunte ao {0} (Ctrl+Enter)", send: "Perguntar", thinking: "{0} está pensando…", failed: "Sem resposta: {0}" },
  ru: { loading: "Поиск Ollama…", off: "Ollama на этом компьютере не отвечает. Запустите его и нажмите здесь.", none: "У Ollama пока нет моделей. Загрузите модель: ollama pull llama3.2", ask: "Задать вопрос {0} (Ctrl+Enter)", send: "Спросить", thinking: "{0} думает…", failed: "Нет ответа: {0}" },
  it: { loading: "Ricerca di Ollama…", off: "Ollama non risponde su questo computer. Avvialo, poi fai clic qui.", none: "Ollama non ha ancora modelli. Scaricane uno con: ollama pull llama3.2", ask: "Fai una domanda a {0} (Ctrl+Invio)", send: "Chiedi", thinking: "{0} sta pensando…", failed: "Nessuna risposta: {0}" },
};
const texts = () => T[pf.lang] || T[pf.lang.split("-")[0]] || T.en;
let t = texts();
const fill = (text, v) => text.replace("{0}", v);

// models: null while looking, [] when Ollama answered without any.
let models = null, offline = false, chosen = "", question = "", answer = "", asking = false;
const gb = (bytes) => (bytes / 1e9).toFixed(1) + " GB";

function draw() {
  const rows = [];
  if (offline) rows.push({ id: "retry", text: t.off, color: "gray" });
  else if (!models) rows.push({ text: t.loading, color: "gray" });
  else if (!models.length) rows.push({ text: t.none, color: "gray" });
  for (const m of models || []) {
    const on = m.name === chosen;
    const about = [m.details && m.details.parameter_size, gb(m.size)].filter(Boolean).join(" · ");
    rows.push({ id: "model:" + m.name, spans: [{ text: m.name, bold: on }, { text: "  " + about, color: "gray" }] });
  }
  if (models && models.length) {
    rows.push({ input: { type: "text", id: "question", lines: 3, value: question, placeholder: fill(t.ask, chosen), max: 4000 } });
    rows.push({ cells: [{ text: "", fill: true }, { id: "ask", text: t.send }] });
    if (asking) rows.push({ text: fill(t.thinking, chosen), color: "gray" });
    else if (answer) rows.push({ text: answer, id: "answer" });
  }
  pf.show({ rows });
}

async function load() {
  try {
    const r = await pf.http.get(BASE + "/api/tags");
    if (r.status !== 200) throw new Error("HTTP " + r.status);
    models = (JSON.parse(r.body).models || []).map((m) => ({ name: m.name, size: m.size || 0, details: m.details }));
    offline = false;
    if (!models.some((m) => m.name === chosen)) chosen = models.length ? models[0].name : "";
  } catch (e) {
    offline = true;
  }
  draw();
}

async function ask() {
  if (asking || !chosen || !question.trim()) return;
  asking = true;
  draw();
  try {
    const body = JSON.stringify({ model: chosen, prompt: question, stream: false, options: { num_predict: 200 } });
    const r = await pf.http.post(BASE + "/api/generate", { headers: { "Content-Type": "application/json" }, body });
    const data = JSON.parse(r.body);
    answer = r.status === 200 ? String(data.response || "").trim() : fill(t.failed, data.error || "HTTP " + r.status);
  } catch (e) {
    answer = fill(t.failed, e.message);
  }
  asking = false;
  draw();
}

pf.on("click", (e) => {
  if (e.id === "retry") load();
  if (e.id === "ask") ask();
  if (e.id && e.id.startsWith("model:")) { chosen = e.id.slice(6); draw(); }
});
pf.on("change", (e) => { if (e.id === "question") question = e.text; });
pf.on("submit", (e) => { if (e.id === "question") ask(); });
pf.on("show", load);
pf.on("lang", () => { t = texts(); draw(); });
pf.every(60, load);
draw();
load();
