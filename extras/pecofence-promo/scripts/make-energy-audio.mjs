import fs from "node:fs";
import path from "node:path";
import {execFileSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const output = path.join(root, "public");
const rate = 48000, seconds = 30, count = rate * seconds;
const bpm = 128, beat = 60 / bpm, bar = beat * 4;
const music = [new Float64Array(count), new Float64Array(count)];
const drums = [new Float64Array(count), new Float64Array(count)];
const effects = [new Float64Array(count), new Float64Array(count)];
const cuts = [7,15,23], events = [];
let seed = 931284;
const noise = () => {
  seed = (1664525 * seed + 1013904223) >>> 0;
  return seed / 2147483648 - 1;
};
const hz = midi => 440 * 2 ** ((midi - 69) / 12);
const snap = time => cuts.find(cut => Math.abs(cut - time) < .04) ?? time;
const kickTimes = Array.from({length: 64}, (_, i) => snap(i * beat));

function add(bus, start, duration, amplitude, pan, voice) {
  const first = Math.round(start * rate), length = Math.round(duration * rate);
  const gainL = Math.sqrt((1-pan)/2), gainR = Math.sqrt((1+pan)/2);
  for (let j = 0; j < length; j++) {
    const index = first + j;
    if (index >= count) break;
    if (index < 0) continue;
    const value = voice(j/rate, j) * amplitude;
    bus[0][index] += value * gainL;
    bus[1][index] += value * gainR;
  }
}

function kick(time, strength = 1) {
  events.push({type: "kick", time});
  add(drums, time, .32, .86*strength, 0, t => {
    const phase = 2*Math.PI*(50*t + 130*.022*(1-Math.exp(-t/.022)));
    const body = Math.sin(phase)*Math.exp(-t*14);
    const click = noise()*Math.exp(-t*260)*.12;
    return (body+click)*Math.min(1,t/.0012)*Math.min(1,(.32-t)/.04);
  });
}

function clap(time, strength = 1) {
  events.push({type: "clap", time});
  let low = 0, smooth = 0;
  add(drums, time, .22, .25*strength, .03, t => {
    const n = noise();
    low += .09*(n-low);
    smooth += .68*(n-low-smooth);
    const bursts = Math.exp(-t*65) +
      (t>.01 ? .7*Math.exp(-(t-.01)*60) : 0) +
      (t>.023 ? .6*Math.exp(-(t-.023)*35) : 0);
    const body = Math.sin(2*Math.PI*185*t)*Math.exp(-t*38)*.22;
    return (smooth*bursts+body)*Math.min(1,t/.0008)*Math.min(1,(.22-t)/.03);
  });
}

function hat(time, open, amplitude, pan) {
  let low = 0;
  const duration = open ? .16 : .065;
  add(drums, time, duration, amplitude, pan, t => {
    const n = noise();
    low += .45*(n-low);
    const metal = (Math.sin(2*Math.PI*7400*t)+Math.sin(2*Math.PI*9130*t))*.11;
    return (n-low+metal)*Math.exp(-t*(open?28:100))*
      Math.min(1,t/.0006)*Math.min(1,(duration-t)/.012);
  });
}

function bass(time, midi, duration, amplitude) {
  const frequency = hz(midi);
  add(music, time, duration, amplitude, 0, t => {
    let saw = 0;
    for(let h=1;h<=7;h++) saw += Math.sin(2*Math.PI*frequency*h*t)/(h*(1+.19*h));
    const sub = Math.sin(2*Math.PI*frequency/2*t)*.4;
    const gate = Math.min(1,t/.005)*Math.min(1,(duration-t)/.035);
    return Math.tanh((saw+sub)*1.25)*gate*Math.exp(-t*1.1);
  });
}

function synth(time, midi, duration, amplitude, pan=0, bright=false) {
  const frequency = hz(midi);
  const tone = t => {
    const detune = .3*Math.sin(2*Math.PI*frequency*1.004*t);
    const wave = Math.sin(2*Math.PI*frequency*t)+detune+
      .27*Math.sin(2*Math.PI*frequency*2*t)+(bright?.17:.06)*Math.sin(2*Math.PI*frequency*3*t);
    return wave*Math.min(1,t/.004)*Math.min(1,(duration-t)/.04)*Math.exp(-t*(bright?7:4));
  };
  add(music,time,duration,amplitude,pan,tone);
  add(music,time+beat*.75,duration,amplitude*.16,-pan,tone);
}

function rise(cut) {
  let lp=0;
  add(effects,cut-.65,.65,.11,-.06,t=>{
    const n=noise(), progress=t/.65;
    lp+=(.05+progress*.5)*(n-lp);
    return (n-lp)*Math.sin(progress*Math.PI/2)**2*Math.min(1,t/.03);
  });
  add(effects,cut,.48,.12,.08,t=>{
    const metallic=Math.sin(2*Math.PI*(1800*t-900*t*t))*.18;
    return (noise()*.5+metallic)*Math.exp(-t*10)*Math.min(1,t/.002)*Math.min(1,(.48-t)/.06);
  });
  events.push({type:"transition-impact",time:cut});
  for(const offset of [.375,.1875,.09375]) clap(cut-offset,.22);
}

const chords=[
  [50,57,62,65,69],
  [46,53,58,62,65],
  [53,60,65,69,72],
  [48,55,60,64,67],
];
for(let i=0;i<kickTimes.length;i++){
  const time=kickTimes[i];
  kick(time,time<7?.78:1);
  if(i%2===1) clap(time,time<3.75?.7:1);
  hat(time,false,.065,-.28);
  hat(i*beat+beat/2,true,.09,.32);
  if(i>=12&&i<62){
    hat(i*beat+beat/4,false,.022,-.48);
    hat(i*beat+beat*3/4,false,.028,.46);
  }
}
for(let b=0;b<16;b++){
  const chord=chords[b%4], start=b*bar, power=start<7?.78:1;
  const bassOffsets=[0,0,7,0,12,0,7,0];
  for(let step=0;step<8;step++){
    bass(start+step*beat/2+.015,chord[0]-12+bassOffsets[step],beat*.4,.28*power);
  }
  for(const step of [.5,1.75,2.5,3.5]){
    chord.slice(1,4).forEach((m,i)=>synth(start+step*beat,m,.27,.045*power,(i-1)*.35));
  }
  [0,2,3,2,1,2,4,3].forEach((degree,step)=>{
    synth(start+step*beat/2,chord[degree]+12,.22,.052*power,step%2?.25:-.25,true);
  });
}
for(const cut of cuts) rise(cut);
synth(29.0625,62,.9,.085,-.1);
synth(29.0625,65,.9,.07,.1);
synth(29.0625,69,.9,.065,.3);

let peak=0, kickIndex=0;
for(let i=0;i<count;i++){
  const time=i/rate;
  while(kickIndex+1<kickTimes.length&&kickTimes[kickIndex+1]<=time) kickIndex++;
  const since=time-kickTimes[kickIndex];
  const duck=.17+.83*Math.min(1,Math.max(0,since)/(beat*.48))**.65;
  const fade=Math.min(1,time/.005,(seconds-time)/.32);
  for(let channel=0;channel<2;channel++){
    const sum=drums[channel][i]+effects[channel][i]+music[channel][i]*duck;
    const sample=Math.tanh(sum*.92)/.92*fade;
    if(!Number.isFinite(sample)) throw Error(`Invalid audio sample at ${i}`);
    music[channel][i]=sample;
    peak=Math.max(peak,Math.abs(sample));
  }
}
if(peak<.001) throw Error("The generated track is silent.");
const gain=.92/peak;
const wav=Buffer.alloc(44+count*4);
wav.write("RIFF",0);wav.writeUInt32LE(wav.length-8,4);
wav.write("WAVEfmt ",8);wav.writeUInt32LE(16,16);wav.writeUInt16LE(1,20);
wav.writeUInt16LE(2,22);wav.writeUInt32LE(rate,24);wav.writeUInt32LE(rate*4,28);
wav.writeUInt16LE(4,32);wav.writeUInt16LE(16,34);wav.write("data",36);
wav.writeUInt32LE(count*4,40);
for(let i=0;i<count;i++){
  wav.writeInt16LE(Math.round(music[0][i]*gain*32766),44+i*4);
  wav.writeInt16LE(Math.round(music[1][i]*gain*32766),46+i*4);
}
fs.mkdirSync(output,{recursive:true});
fs.writeFileSync(path.join(output,"soundtrack-energy.wav"),wav);
execFileSync("ffmpeg",["-hide_banner","-loglevel","error","-y",
  "-i",path.join(output,"soundtrack-energy.wav"),
  "-af","highpass=f=28,loudnorm=I=-14:TP=-1.5:LRA=7",
  "-ar","48000","-ac","2","-c:a","aac","-b:a","256k","-movflags","+faststart",
  path.join(output,"soundtrack-energy.m4a")]);
const manifest={
  duration:seconds,bpm,sampleRate:rate,channels:2,bars:16,
  provenance:"Original synthesized drums, bass, chord stabs, lead and effects; no external samples or voices.",
  rhythm:"Four-on-the-floor kick, backbeat claps, offbeat open hats, sixteenth-note percussion and syncopated sidechained bass.",
  targetIntegratedLufs:-14,targetTruePeakDbtp:-1.5,
  transitionAccents:cuts,kicks:kickTimes.length,
  events:events.sort((a,b)=>a.time-b.time),
};
fs.writeFileSync(path.join(output,"soundtrack-energy-manifest.json"),JSON.stringify(manifest,null,2)+"\n");
console.log(`Generated original ${seconds}s / ${bpm} BPM dance soundtrack with ${kickTimes.length} kicks and accents at ${cuts.join(", ")}s.`);
