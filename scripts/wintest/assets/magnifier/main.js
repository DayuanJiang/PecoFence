// Magnifier: a zoomed view of the screen around the mouse pointer. PecoFence captures the screen
// and draws the picture on the graphics card (the `magnifier` row); this code only says how: the
// zoom, what the picture follows (the pointer, the keyboard focus, the text insertion point),
// smoothing, the pointer in the picture and whether the fence stays above the windows. It never
// receives a pixel: PecoFence finds the focus and the caret itself.
//
// Two modes, with check boxes in the plugin settings:
//   - Following (the pointer, the keyboard focus or the text insertion point, any of them: the
//     picture goes to whichever moved last): the picture moves with it. With "Allow locking the
//     picture", a click in the fence (or the Lock command's shortcut) holds the picture where it
//     is, and again lets it follow.
//   - Not following: the picture shows the place the pointer was when it was taken. A click in
//     the fence (or the Lock command's shortcut) takes it again at the pointer's place, which
//     needs "Allow locking the picture".
// The wheel over the fence changes the zoom (away from you: larger).

const ZOOMS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 20];

const TEXTS = {
  en: { lock: "Lock the picture", shot: "Take the picture here", zoomin: "Zoom in", zoomout: "Zoom out",
        hint: "Turn on a Follow option or Allow locking the picture in the plugin settings." },
  "zh-CN": { lock: "锁定画面", shot: "在此处取景", zoomin: "放大", zoomout: "缩小",
        hint: "请在插件设置里打开任一「跟随」选项或「允许锁定画面」。" },
  "zh-TW": { lock: "鎖定畫面", shot: "在此處取景", zoomin: "放大", zoomout: "縮小",
        hint: "請在外掛設定裡開啟任一「跟隨」選項或「允許鎖定畫面」。" },
  ja: { lock: "画像を固定", shot: "ここで撮り直す", zoomin: "拡大", zoomout: "縮小",
        hint: "プラグイン設定で、「追従」のいずれか、または「画像の固定を許可」をオンにしてください。" },
  ko: { lock: "화면 고정", shot: "여기서 다시 촬영", zoomin: "확대", zoomout: "축소",
        hint: "플러그인 설정에서 '따라가기' 옵션 중 하나 또는 '화면 고정 허용'을 켜세요." },
  de: { lock: "Bild fixieren", shot: "Bild hier aufnehmen", zoomin: "Vergrößern", zoomout: "Verkleinern",
        hint: "Aktivieren Sie in den Plugin-Einstellungen eine Folgen-Option oder „Fixieren des Bildes erlauben“." },
  fr: { lock: "Verrouiller l'image", shot: "Prendre l'image ici", zoomin: "Zoom avant", zoomout: "Zoom arrière",
        hint: "Activez une option « Suivre » ou Autoriser le verrouillage de l'image dans les réglages du plugin." },
  es: { lock: "Bloquear la imagen", shot: "Tomar la imagen aquí", zoomin: "Acercar", zoomout: "Alejar",
        hint: "Active una opción «Seguir» o Permitir bloquear la imagen en los ajustes del plugin." },
  "pt-BR": { lock: "Travar a imagem", shot: "Capturar a imagem aqui", zoomin: "Aumentar zoom", zoomout: "Diminuir zoom",
        hint: "Ative uma opção “Seguir” ou Permitir travar a imagem nas configurações do plugin." },
  ru: { lock: "Зафиксировать изображение", shot: "Снять изображение здесь", zoomin: "Увеличить", zoomout: "Уменьшить",
        hint: "Включите один из параметров «Следовать за…» или «Разрешить фиксацию изображения» в настройках плагина." },
  it: { lock: "Blocca l'immagine", shot: "Cattura l'immagine qui", zoomin: "Ingrandisci", zoomout: "Riduci",
        hint: "Attiva un'opzione «Segui» o Consenti il blocco dell'immagine nelle impostazioni del plugin." },
};

const text = () => TEXTS[pf.lang] || TEXTS[String(pf.lang).split("-")[0]] || TEXTS.en;

// Held in follow mode (the picture stays where it was), and the count of pictures taken again in
// the other mode. `zoom` is the wheel's choice; null means the setting's.
let held = false;
let shot = 0;
let zoom = null;

const zoomNow = () => zoom ?? (Number(pf.settings.zoom) || 2);
// Following: the pointer, the keyboard focus or the text insertion point.
const following = () => !!(pf.settings.follow || pf.settings.focus || pf.settings.caret);

function draw() {
  const t = text();
  if (!following() && !pf.settings.lock) {
    // Nothing to follow and nothing to take: say so, and capture nothing.
    pf.show({ rows: [{ text: t.hint, color: "gray" }] });
    return;
  }
  pf.show({
    rows: [{
      id: "m",
      wheel: true,
      tip: pf.settings.lock ? (following() ? t.lock : t.shot) : undefined,
      magnifier: {
        zoom: zoomNow(),
        smooth: pf.settings.smooth,
        pointer: pf.settings.pointer,
        follow: !!pf.settings.follow && !held,
        focus: !!pf.settings.focus && !held,
        caret: !!pf.settings.caret && !held,
        shot,
        top: pf.settings.top,
      },
    }],
    menu: [
      { id: "lock", title: following() ? t.lock : t.shot, checked: held, disabled: !pf.settings.lock },
      { separator: true },
      { id: "zoomin", title: t.zoomin },
      { id: "zoomout", title: t.zoomout },
    ],
  });
}

function lock() {
  if (!pf.settings.lock) return;
  if (following()) held = !held;
  else shot += 1;
  draw();
}

function step(by) {
  const at = ZOOMS.reduce((best, z, i) => (Math.abs(z - zoomNow()) < Math.abs(ZOOMS[best] - zoomNow()) ? i : best), 0);
  zoom = ZOOMS[Math.max(0, Math.min(ZOOMS.length - 1, at + by))];
  draw();
}

pf.on("start", draw);
pf.on("show", draw);
pf.on("lang", draw);
pf.on("settings", () => {
  // A changed zoom setting is the new choice; so is leaving the follow mode that held it.
  zoom = null;
  if (!following() || !pf.settings.lock) held = false;
  draw();
});
pf.on("click", (e) => {
  if (e.id === "m") lock();
});
pf.on("wheel", (e) => {
  if (e.id === "m" && e.steps) step(e.steps > 0 ? 1 : -1);
});
pf.on("command", (e) => {
  if (e.id === "lock") lock();
  else if (e.id === "zoomin") step(1);
  else if (e.id === "zoomout") step(-1);
});
