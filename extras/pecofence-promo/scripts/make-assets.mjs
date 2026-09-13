import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {execFileSync} from "node:child_process";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pub = path.join(root, "public");
const capture = path.join(root, ".capture");
const seconds = Number(process.argv[2] ?? 30);
const suffix = process.argv[3] ?? (seconds === 30 ? "" : "-features");
const bpm = Number(process.argv[4] ?? 96);
const electronic = suffix === "-tech";
fs.mkdirSync(pub, {recursive: true});
fs.mkdirSync(capture, {recursive: true});

// An original procedural wallpaper, shared by the real application and the film.
const w = 2560, h = 1440;
const pixels = Buffer.alloc(w * h * 3);
for (let y = 0; y < h; y++) {
  for (let x = 0; x < w; x++) {
    const u = x / w, v = y / h;
    const glow = Math.exp(-((u - .69) ** 2 / .13 + (v - .55) ** 2 / .16));
    const teal = Math.exp(-((u - .94) ** 2 / .1 + (v - .1) ** 2 / .22));
    const ribbonY = .83 - .35 * Math.sin(u * 2.3 - .9);
    const ribbon = Math.exp(-(((v - ribbonY) / .075) ** 2));
    const fine = Math.exp(-(((v - ribbonY - .034) / .006) ** 2));
    const i = (y * w + x) * 3;
    const grain = ((x * 19 + y * 31) % 7 - 3) * .24;
    pixels[i] = Math.min(255, 9 + 22 * glow + 6 * teal + 27 * ribbon + 20 * fine + grain);
    pixels[i + 1] = Math.min(255, 19 + 58 * glow + 54 * teal + 42 * ribbon + 30 * fine + grain);
    pixels[i + 2] = Math.min(255, 38 + 112 * glow + 69 * teal + 93 * ribbon + 34 * fine + grain);
  }
}
const ppm = path.join(capture, "wallpaper.ppm");
fs.writeFileSync(ppm, Buffer.concat([Buffer.from(`P6\n${w} ${h}\n255\n`), pixels]));
if (seconds === 30 && suffix === "") {
  execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y", "-i", ppm,
    path.join(pub, "wallpaper.png")]);
}

// Original instrumental at the requested tempo and duration. No external samples.
const rate = 48000, count = rate * seconds;
const left = new Float64Array(count), right = new Float64Array(count);
let seed = 8239;
const random = () => {seed = (seed * 1664525 + 1013904223) >>> 0; return seed / 4294967296 * 2 - 1;};
const hz = (m) => 440 * 2 ** ((m - 69) / 12);
function note(start, dur, midi, volume, pan = 0, type = "pluck") {
  const first = Math.round(start * rate), length = Math.round(dur * rate), freq = hz(midi);
  for (let j = 0; j < length && first + j < count; j++) {
    const t = j / rate, p = j / length;
    const attack = Math.min(1, t / (type === "pad" ? .45 : .008));
    const release = Math.min(1, (dur - t) / (type === "pad" ? .7 : .15));
    const envelope = attack * release * (type === "pad" ? .65 : Math.exp(-t * (electronic ? 5.2 : 3.8)));
    const wave = Math.sin(2 * Math.PI * freq * t) +
      (electronic ? .38 : .24) * Math.sin(2 * Math.PI * freq * 2 * t) +
      (electronic ? .16 : .07) * Math.sin(2 * Math.PI * freq * 3 * t) +
      (electronic ? .05 * Math.sin(2 * Math.PI * freq * 4 * t) : 0);
    const signal = wave * envelope * volume;
    left[first + j] += signal * Math.sqrt((1 - pan) / 2);
    right[first + j] += signal * Math.sqrt((1 + pan) / 2);
    if (type === "pluck") {
      const echo = first + j + Math.round(rate * (electronic ? 30 / bpm : .3125));
      if (echo < count) {
        left[echo] += signal * .19 * Math.sqrt((1 + pan) / 2);
        right[echo] += signal * .19 * Math.sqrt((1 - pan) / 2);
      }
    }
  }
}
const beat = 60 / bpm;
const chords = electronic
  ? [[50,57,62,65,69],[46,53,58,62,65],[48,55,60,64,67],[45,52,57,60,64]]
  : [[50,57,62,66,69],[47,54,59,62,66],[43,50,57,59,62],[45,52,57,61,64]];
const bars = Math.ceil(seconds / (beat * 4));
for (let bar = 0; bar < bars; bar++) {
  const chord = chords[bar % 4], start = bar * beat * 4;
  chord.slice(1).forEach((m, n) => note(start, 3.1, m, electronic ? .025 : .04, (n - 1.5) / 2.4, "pad"));
  note(start, electronic ? 1.05 : 2.1, chord[0] - 12, electronic ? .17 : .14, 0, electronic ? "bass" : "pad");
  [0,1,2,3,2,1,3,4].forEach((n, k) => {
    note(start + k * beat / 2, .95, chord[n] + 12, bar === bars - 1 ? .055 : .085,
      k % 2 ? .32 : -.32);
  });
  if (bar >= (electronic ? 0 : 2) && bar < bars - (electronic ? 1 : 2)) {
    for (let b = 0; b < 4; b++) {
      const offset = Math.round((start + b * beat) * rate);
      for (let j = 0; j < rate * .22 && offset + j < count; j++) {
        const t = j / rate;
        const kick = Math.sin(2 * Math.PI * (48 * t + 38 * .019 * (1 - Math.exp(-t / .019)))) *
          Math.exp(-t * 22) * .19;
        left[offset + j] += kick; right[offset + j] += kick;
      }
      const hatOffset = offset + Math.round(beat * .5 * rate);
      for (let j = 0; j < 2400 && hatOffset + j < count; j++) {
        const noise = random() * Math.exp(-j / 370) * .023;
        left[hatOffset + j] += noise * .7; right[hatOffset + j] += noise;
      }
    }
  }
}
// Soft edit accents and a double-click timed to the quick-hide sequence.
for (const at of (electronic ? [.35,1,7,15,23,24] : suffix === "-motion" ? [.5,1,7,15,23,24] : suffix === "-reading" ? [5,13,21,31] : suffix === "-reviewed" ? [2,8,14,22] : suffix === "-launch" ? [3,8,15,23] : seconds === 30 ? [4.5, 10.5, 15.5, 21, 25.5] : [4,14,26,39,50,61,71])) {
  const first = Math.floor((at - .16) * rate);
  for (let j = 0; j < rate * .4; j++) {
    const p = j / (rate * .4);
    const s = random() * Math.sin(p * Math.PI) ** 3 * .024;
    if (first + j >= 0 && first + j < count) {
      left[first + j] += s; right[first + j] += s;
    }
  }
}
if (electronic) {
  for (const at of [.4,7,15,23]) note(at, .38, 86, .028, -.1, "pluck");
}
for (const at of (["-launch","-reviewed","-reading","-motion","-tech"].includes(suffix) ? [] : seconds === 30 ? [22.72, 22.87, 24.32, 24.47] : [63.1,63.24,64.65,64.8,66.2,66.34])) {
  for (let j = 0; j < 1500; j++) {
    const i = Math.round(at * rate) + j;
    const s = (random() * .05 + Math.sin(j * .34) * .05) * Math.exp(-j / 160);
    left[i] += s; right[i] += s;
  }
}
const wav = Buffer.alloc(44 + count * 4);
wav.write("RIFF", 0); wav.writeUInt32LE(wav.length - 8, 4);
wav.write("WAVEfmt ", 8); wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20);
wav.writeUInt16LE(2, 22); wav.writeUInt32LE(rate, 24); wav.writeUInt32LE(rate * 4, 28);
wav.writeUInt16LE(4, 32); wav.writeUInt16LE(16, 34); wav.write("data", 36);
wav.writeUInt32LE(count * 4, 40);
for (let i = 0; i < count; i++) {
  const t = i / rate;
  const fade = Math.min(1, t / .65, (seconds - t) / 1.5);
  wav.writeInt16LE(Math.round(Math.tanh(left[i] * 1.4) * fade * 27000), 44 + i * 4);
  wav.writeInt16LE(Math.round(Math.tanh(right[i] * 1.4) * fade * 27000), 46 + i * 4);
}
fs.writeFileSync(path.join(pub, `soundtrack${suffix}.wav`), wav);
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y",
  "-i", path.join(pub, `soundtrack${suffix}.wav`), "-af", "loudnorm=I=-18:TP=-1.5:LRA=9",
  "-ar", "48000", "-c:a", "aac", "-b:a", "192k",
  "-movflags", "+faststart", path.join(pub, `soundtrack${suffix}.m4a`)]);
console.log(`Generated original ${seconds}-second stereo soundtrack.`);
