import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {execFileSync} from "node:child_process";

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"..");
const dir=path.join(root,"public","features");
const specifications=[
  {name:"groups",from:.8,span:16.5,duration:10,crop:"2360:1040:100:260"},
  {name:"rules",from:.7,span:12,duration:12,crop:"2360:960:100:280"},
  {name:"portal",from:0,span:13,duration:13,crop:"1740:840:450:300"},
  {name:"tabs",from:1.0,span:15.4,duration:11,crop:"2360:960:100:280"},
  {name:"peek",from:1,span:11,duration:11,crop:"2460:1090:50:175"},
  {name:"hide",from:.3,span:13,duration:10,crop:"2360:960:100:280"},
];
const timings={};
for(const spec of specifications){
  const rate=spec.span/spec.duration;
  const filter=`trim=start=${spec.from}:duration=${spec.span},setpts=(PTS-STARTPTS)/${rate},crop=${spec.crop},scale=1720:660:force_original_aspect_ratio=decrease,pad=1720:660:(ow-iw)/2:(oh-ih)/2:color=0x0b1b2d,fps=30`;
  const selected=process.argv.length<=2||process.argv.slice(2).includes(spec.name);
  if(selected) execFileSync("ffmpeg",["-hide_banner","-loglevel","error","-y","-i",path.join(dir,`${spec.name}-raw.mp4`),
    "-vf",filter,"-an","-c:v","libx264","-preset","fast","-crf","17","-pix_fmt","yuv420p",
    "-t",String(spec.duration),"-movflags","+faststart",path.join(dir,`${spec.name}.mp4`)]);
  const events=JSON.parse(fs.readFileSync(path.join(dir,`${spec.name}-events.json`),"utf8").replace(/^\uFEFF/,""));
  timings[spec.name]=Object.fromEntries(events.filter(e=>e.ok&&e.event!=="record")
    .map(e=>[e.event,Math.max(0,Math.round((e.time-spec.from)/rate*30))]));
  console.log(`${selected?"Prepared":"Reused"} ${spec.name}: ${spec.duration}s`);
}
fs.writeFileSync(path.join(root,"src","features","timings.ts"),
  `// Captured action times mapped to the edited shots, in frames at 30 fps.\nexport const timings = ${JSON.stringify(timings,null,2)};\n`);
