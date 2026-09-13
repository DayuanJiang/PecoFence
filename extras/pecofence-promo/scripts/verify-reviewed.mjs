import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import {execFile} from "node:child_process";
import {promisify} from "node:util";
import {fileURLToPath} from "node:url";

const exec = promisify(execFile);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const stem = process.argv[2] ?? "PecoFence-autosort-en-1080p";
const expectedSeconds = Number(process.argv[3] ?? 30);
if (!/^[A-Za-z0-9_-]+$/.test(stem)) throw Error("Expected a filename stem.");
if (![25,30,36].includes(expectedSeconds)) throw Error("Expected a 25-, 30- or 36-second delivery specification.");
const sheetSide = Math.ceil(Math.sqrt(expectedSeconds));
const sheetRows = Math.ceil(expectedSeconds / sheetSide);
const video = path.join(root, "out", `${stem}.mp4`);
const framesDir = path.join(root, "out", `${stem}-frames`);
await fs.mkdir(framesDir, {recursive: true});
const run = (args) => exec("ffmpeg", args, {maxBuffer: 4 * 1024 * 1024});
const jobs = [
  ["probe", () => exec("ffprobe", ["-v", "error", "-show_format", "-show_streams", "-of", "json", video])],
  ["decode", () => run(["-v", "error", "-i", video, "-f", "null", "-"])],
  ["audio", () => run(["-hide_banner", "-i", video, "-af", "volumedetect", "-vn", "-f", "null", "-"])],
  ["black", () => run(["-hide_banner", "-i", video, "-vf", "blackdetect=d=0.1:pix_th=0.1", "-an", "-f", "null", "-"])],
  ["frames", () => run(["-hide_banner", "-loglevel", "error", "-y", "-i", video,
    "-vf", "select=eq(mod(n\\,30)\\,15)", "-fps_mode", "vfr", "-q:v", "2",
    path.join(framesDir, "second-%02d.jpg")])],
  ["contact", () => run(["-hide_banner", "-loglevel", "error", "-y", "-i", video,
    "-vf", `select=eq(mod(n\\,30)\\,15),scale=480:270,tile=${sheetSide}x${sheetRows}:padding=8:margin=8:color=0x091b30`,
    "-frames:v", "1", path.join(root, "out", `${stem}-contact.png`)])],
];
const results = await Promise.allSettled(jobs.map(async ([name, work]) => ({name, ...await work()})));
const failed = results.flatMap((result, i) => result.status === "rejected"
  ? [{job: jobs[i][0], error: String(result.reason)}] : []);
if (failed.length) throw Error(JSON.stringify(failed));
const outputs = Object.fromEntries(results.map(result => [result.value.name, result.value]));
const probe = JSON.parse(outputs.probe.stdout);
const picture = probe.streams.find(stream => stream.codec_type === "video");
const sound = probe.streams.find(stream => stream.codec_type === "audio");
if (picture.width !== 1920 || picture.height !== 1080 || picture.avg_frame_rate !== "30/1" ||
    Number(picture.nb_frames) !== expectedSeconds * 30 || Math.abs(Number(picture.duration) - expectedSeconds) > 0.001 ||
    picture.codec_name !== "h264" || picture.pix_fmt !== "yuv420p" ||
    picture.color_space !== "bt709" || picture.color_range !== "tv" ||
    sound?.codec_name !== "aac" || sound?.channels !== 2) {
  throw Error(`Export format differs from the ${expectedSeconds}-second specification.`);
}
if (outputs.decode.stderr.trim()) throw Error(outputs.decode.stderr);
const audioLevels = outputs.audio.stderr.match(/(?:mean_volume|max_volume):[^\r\n]+/g);
const maxVolume = Number(outputs.audio.stderr.match(/max_volume: ([\d.-]+) dB/)?.[1]);
if (!Number.isFinite(maxVolume) || maxVolume >= 0 || maxVolume < -45) {
  throw Error("Audio is clipping or effectively silent.");
}
const blackIntervals = outputs.black.stderr.match(/black_start:[^\r\n]+/g) ?? [];
if (blackIntervals.length) throw Error(`Unexpected black frames: ${blackIntervals.join("; ")}`);
const bytes = await fs.readFile(video);
const report = {
  file: `out/${stem}.mp4`,
  sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  checkedAt: new Date().toISOString(),
  bytes: bytes.length,
  video: {codec: picture.codec_name, width: picture.width, height: picture.height,
    pixelFormat: picture.pix_fmt, colorSpace: picture.color_space, colorRange: picture.color_range,
    fps: picture.avg_frame_rate, frames: Number(picture.nb_frames), duration: Number(picture.duration)},
  audio: {codec: sound.codec_name, channels: sound.channels, sampleRate: Number(sound.sample_rate), levels: audioLevels},
  fullDecode: "passed",
  blackIntervals,
  contactSheet: `${stem}-contact.png`,
  frameSampling: `${expectedSeconds} full-resolution frames at 0.5, 1.5, ... ${expectedSeconds - 0.5} seconds. second-01.jpg is 0.5 s.`,
  visualReview: "Separate independent reviewer reports are required; this report only verifies export integrity.",
};
await fs.writeFile(path.join(root, "out", `${stem}-verification.json`), JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report, null, 2));
