import {execFileSync} from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const publicDir=path.join(root,"public","launch");
const clips=[
  {name:"tabs",from:3.4,speed:2,duration:7,filter:"scale=1920:1080"},
  {name:"peek",from:1.3,speed:1.25,duration:8,filter:"crop=2560:1020:0:175,scale=1728:688"},
];
for(const clip of clips){
  execFileSync("ffmpeg",[
    "-hide_banner","-loglevel","error","-y",
    "-ss",String(clip.from),"-i",path.join(publicDir,"raw",`${clip.name}-raw.mp4`),
    "-vf",`setpts=(PTS-STARTPTS)/${clip.speed},${clip.filter},fps=30`,
    "-t",String(clip.duration),"-an","-c:v","libx264","-preset","fast","-crf","17",
    "-pix_fmt","yuv420p","-movflags","+faststart",path.join(publicDir,`${clip.name}.mp4`),
  ]);
  console.log(`Prepared current-build ${clip.name}: ${clip.duration}s`);
}
const config=JSON.parse(fs.readFileSync(path.join(root,".capture","launch","hero","app","config","config.json"),"utf8").replace(/^\uFEFF/,""));
const manifest={
  capturedAt:new Date().toISOString(),
  buildModifiedAt:fs.statSync(path.join(root,".capture","launch","hero","app","pecofence.exe")).mtime.toISOString(),
  language:config.settings.language,
  themeStyle:config.settings.themeStyle,
  source:"Native Windows recordings of isolated portable PecoFence demo instances",
  stagedWindow:"Launchpad project board rendered in a Windows Forms window",
  editorialClips:clips,
};
if(manifest.language!=="en"||manifest.themeStyle!=="liquidGlass")throw Error("Capture must use the current English Liquid Glass UI.");
fs.writeFileSync(path.join(publicDir,"capture-manifest.json"),JSON.stringify(manifest,null,2)+"\n");
