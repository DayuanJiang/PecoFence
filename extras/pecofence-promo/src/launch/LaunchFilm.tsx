import {Audio} from "@remotion/media";
import {TransitionSeries} from "@remotion/transitions";
import {staticFile} from "remotion";
import {Opening} from "./Opening";
import {Beauty} from "./Beauty";
import {Workspaces} from "./Workspaces";
import {WithinReach} from "./WithinReach";
import {Closing} from "./Closing";

export const LaunchFilm = () => <>
  <Audio src={staticFile("soundtrack-launch.m4a")}/>
  <TransitionSeries>
    <TransitionSeries.Sequence durationInFrames={90} name="A calmer desktop">
      <Opening/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Sequence durationInFrames={150} name="Liquid Glass">
      <Beauty/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Sequence durationInFrames={210} name="Workspaces">
      <Workspaces/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Sequence durationInFrames={240} name="Peek">
      <WithinReach/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Sequence durationInFrames={150} name="Make room for focus">
      <Closing/>
    </TransitionSeries.Sequence>
  </TransitionSeries>
</>;
