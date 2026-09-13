import {FeatureShot} from "./common";
import {timings} from "./timings";

export const Groups=()=> <FeatureShot number={1} title="Create your own groups"
  description="Create a group, drop in files, then move and resize it to fit your workflow."
  file="groups" steps={[
    {at:0,label:"Create a group",note:"Choose a place on your desktop"},
    {at:timings.groups["type-name"]??95,label:"Name it Work"},
    {at:timings.groups["drag-file-1"]??130,label:"Drop files into the group"},
    {at:timings.groups["position-group"]??225,label:"Move and resize",note:"Arrange it your way"},
  ]}/>;
