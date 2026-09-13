import {FeatureShot} from "./common";
import {timings} from "./timings";

export const TabsDemo=()=> <FeatureShot number={4} title="Combine groups into tabs"
  description="Switch between related groups in one window. Separate them when you need more space."
  file="tabs" steps={[
    {at:0,label:"Two groups, one window",note:"Keep related groups together as tabs"},
    {at:timings.tabs["switch-media"],label:"Switch to Art",note:"A different group, in the same window"},
    {at:timings.tabs["switch-work"],label:"Switch to Work",note:"Your work files are one click away"},
    {at:timings.tabs["separate-tab"],label:"Separate the tab",note:"Two independent groups again"},
  ]}/>;
