import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {execFileSync} from "node:child_process";

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"..");
const take=process.argv[2]??"v3";
if(!/^v[0-9]+$/.test(take))throw Error("Expected an immutable take identifier.");
const assets=path.join(root,"public","reviewed",take);
const capture=JSON.parse(fs.readFileSync(path.join(assets,"capture-manifest.json"),"utf8").replace(/^\uFEFF/,""));
const grouping=JSON.parse(fs.readFileSync(path.join(assets,"grouping-proof.json"),"utf8").replace(/^\uFEFF/,""));
const opened=JSON.parse(fs.readFileSync(path.join(assets,"open-proof.json"),"utf8").replace(/^\uFEFF/,""));
if(capture.groupKind!=="virtual"||capture.sampleCount!==4||grouping.length!==4||opened.nativeOpenCount<1){
  throw Error("Capture has not passed native identity/group/open verification.");
}
for(const clip of [
  {source:"flow-raw.mp4",out:"flow.mp4",filter:"scale=1920:1080,fps=30"},
  {source:"peek-raw.mp4",out:"apps.mp4",filter:"crop=2360:840:100:315,scale=1620:576,fps=30"},
]){
  execFileSync("ffmpeg",["-hide_banner","-loglevel","error","-y",
    "-i",path.join(assets,clip.source),"-vf",clip.filter,"-an","-c:v","libx264",
    "-crf","16","-preset","fast","-pix_fmt","yuv420p","-movflags","+faststart",path.join(assets,clip.out)]);
}
// Extend the screenshot's top edge to retain the title spacing without a seam.
// This derived plate lives outside the immutable raw take.
execFileSync("ffmpeg",["-hide_banner","-loglevel","error","-y",
  "-i",path.join(assets,"before.png"),
  "-filter_complex",
  "[0:v]scale=1920:1080,split=2[body][edge];[body]crop=1920:1000:0:0[lower];[edge]crop=1920:2:0:0,scale=1920:80[upper];[upper][lower]vstack",
  "-frames:v","1","-update","1",path.join(root,"public","reviewed","intro-plate.png")]);
console.log(`Prepared verified native footage from ${take}. Source trims remain frame-based and editable in Remotion.`);
