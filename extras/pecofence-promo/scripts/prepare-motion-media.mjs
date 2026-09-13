import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = path.join(root, "public", "reviewed", "v3");
const dest = path.join(root, "public", "motion");
fs.mkdirSync(dest, {recursive: true});
for (const panel of [
  {name: "work", x: 960, y: 560},
  {name: "art", x: 1720, y: 560},
]) {
  execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y",
    "-i", path.join(source, "grouped.png"),
    "-vf", `crop=650:420:${panel.x}:${panel.y}`,
    "-frames:v", "1", "-update", "1", path.join(dest, `${panel.name}.png`)]);
}
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y",
  "-i", path.join(source, "flow.mp4"), "-vf", "crop=1710:626:110:370",
  "-an", "-c:v", "libx264", "-crf", "15", "-preset", "fast",
  "-pix_fmt", "yuv420p", "-movflags", "+faststart", path.join(dest, "flow.mp4")]);
fs.writeFileSync(path.join(dest, "provenance.json"), JSON.stringify({
  sourceTake: "reviewed/v3",
  panelCrops: {work: [960,560,650,420], art: [1720,560,650,420]},
  videoCrop: [110,370,1710,626],
  groupingSignalSourceFrames: [95,120,144,169],
  groupingSignalSceneFrames: [104,129,153,178],
  editorial: "3D staging, cyan grading, wireframes, instrument brackets, scan lines, light packets and scan gates are presentation effects. Native geometry and recorded grouping/tab actions are preserved.",
}, null, 2) + "\n");
console.log("Prepared native Work/Art crops and framed group/tab recording.");
