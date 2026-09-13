import {FeatureShot,Callout} from "./common";
import {timings} from "./timings";

export const Rules=()=> <FeatureShot number={2} title="Sort new files automatically"
  description="Configured rules: PNG → Images   ·   TXT → Documents"
  file="rules" steps={[
    {at:0,label:"Rules are ready",note:"New desktop files are sorted by extension"},
    {at:timings.rules["new-image"],label:"homepage-demo.png appears",note:"Automatically placed in Images"},
    {at:timings.rules["new-document"],label:"brief-demo.txt appears",note:"Automatically placed in Documents"},
    {at:300,label:"Virtual groups keep originals in place"},
  ]}>
    <Callout from={timings.rules["new-image"]+12} to={timings.rules["new-document"]-8}
      left={110} top={487}>PNG → Images · Sorted automatically</Callout>
    <Callout from={timings.rules["new-document"]+12} to={344}
      left={973} top={487}>TXT → Documents · Sorted automatically</Callout>
  </FeatureShot>;
