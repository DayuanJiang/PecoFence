import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = path.join(root, "public", "autosort", "auto-v2");
const dest = path.join(root, "public", "motion");
const read = name => JSON.parse(fs.readFileSync(path.join(source, name), "utf8").replace(/^\uFEFF/, ""));
const capture = read("capture-manifest.json");
const before = read("before-proof.json");
const proof = read("automatic-proof.json");
if (capture.product !== "PecoFence" || capture.language !== "en" ||
    capture.themeStyle !== "liquidGlass" || !capture.rulesKeepUpdated ||
    !capture.noManualRoutingCommands || capture.nativeFsResyncs < 2 ||
    !before.newFilesAbsentFromDisk || !before.newFilesAbsentFromCatalog ||
    !before.keepUpdated || before.catalogCount !== 2) {
  throw Error("Automatic capture does not satisfy its provenance requirements.");
}
for (const [name, group] of [["Coast.png", "Art"], ["Notes.txt", "Work"]]) {
  const item = proof.find(item => item.name === name);
  if (!item?.newlyCreated || item.group !== group || !item.pathUnchanged ||
      !item.contentUnchanged || !item.mtimeUnchanged ||
      item.assignedBy?.rule !== item.ruleId) {
    throw Error(`Missing automatic routing proof for ${name}.`);
  }
}
fs.mkdirSync(dest, {recursive: true});
for (const panel of [
  {name: "work", x: 960, y: 560},
  {name: "art", x: 1720, y: 560},
]) {
  execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y",
    "-i", path.join(source, "automatically-grouped.png"),
    "-vf", `crop=650:420:${panel.x}:${panel.y}`,
    "-frames:v", "1", "-update", "1", path.join(dest, `autosort-${panel.name}.png`)]);
}
// Normalize the saved variable-frame-rate capture before frame-based editing.
// The source spans 22 seconds. No action is reordered or retimed.
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y",
  "-i", path.join(source, "raw.mp4"),
  "-vf", "fps=30,scale=1920:1080:flags=lanczos,crop=1710:626:110:370,setsar=1",
  "-an", "-c:v", "libx264", "-crf", "15", "-preset", "fast",
  "-pix_fmt", "yuv420p", "-movflags", "+faststart", path.join(dest, "autosort-flow.mp4")]);
const raw = fs.readFileSync(path.join(source, "raw.mp4"));
fs.writeFileSync(path.join(dest, "autosort-provenance.json"), JSON.stringify({
  sourceTake: "autosort/auto-v2",
  sourceSha256: crypto.createHash("sha256").update(raw).digest("hex"),
  sourceFingerprint: capture.sourceFingerprint,
  captureExecutableSha256: capture.executableSha256,
  normalization: "Preserve capture timestamps at 30 fps; scale to 1920x1080, crop [110,370,1710,626].",
  panelCrops: {work: [960, 560, 650, 420], art: [1720, 560, 650, 420]},
  grouping: {sceneStart: 210, holdFrames: 60, trimBefore: 60, playFrames: 180},
  tabs: {sceneStart: 450, holdFrames: 60, trimBefore: 321, playFrames: 180},
  observedSourceFrames: {imageArrives: 95, documentArrives: 171, mergeStarts: 355, workTabActivates: 406},
  editorial: "New files cards and light trails are graphic cues, not a recreated Explorer or product UI. Cues lead the corresponding native arrival by 24 frames. All native panels and routing/tab actions are from the same take. Cyan grading, framing, camera moves and 2-second reading holds are editorial.",
}, null, 2) + "\n");
console.log("Prepared PecoFence automatic routing and tabs from one verified native take.");
