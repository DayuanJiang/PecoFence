import {Callout,FeatureShot} from "./common";
import {timings} from "./timings";

export const Hide=()=> <FeatureShot number={6} title="Hide clutter. Keep quick access."
  description="Hide all groups with a desktop double-click, or roll up just one group."
  file="hide" steps={[
    {at:0,label:"Double-click the desktop",note:"Hide all groups"},
    {at:96,label:"Double-click again",note:"Bring everything back"},
    {at:136,label:"Double-click a title",note:"Roll up one group"},
    {at:timings.hide["hover-expand"],label:"Hover to expand",note:"Quick access when you need it"},
  ]}>
    <Callout from={161} to={timings.hide["hover-expand"]-5} left={80} top={560}>One group, rolled up.</Callout>
    <Callout from={timings.hide["hover-expand"]+12} to={296} left={80} top={560}>Hover to expand.</Callout>
  </FeatureShot>;
