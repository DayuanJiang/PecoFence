import {Audio} from "@remotion/media";
import {TransitionSeries, linearTiming} from "@remotion/transitions";
import {fade} from "@remotion/transitions/fade";
import {staticFile} from "remotion";
import {Intro} from "./scenes/Intro";
import {Organize} from "./scenes/Organize";
import {Glass} from "./scenes/Glass";
import {Tabs} from "./scenes/Tabs";
import {Focus} from "./scenes/Focus";
import {Outro} from "./scenes/Outro";

export const Promo = () => <>
  <Audio src={staticFile("soundtrack.m4a")} volume={1}/>
  <TransitionSeries>
    <TransitionSeries.Sequence durationInFrames={141} name="Less clutter, more clarity"><Intro/></TransitionSeries.Sequence>
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({durationInFrames:12})}/>
    <TransitionSeries.Sequence durationInFrames={189} name="Everything in its place"><Organize/></TransitionSeries.Sequence>
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({durationInFrames:12})}/>
    <TransitionSeries.Sequence durationInFrames={162} name="Native glass"><Glass/></TransitionSeries.Sequence>
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({durationInFrames:12})}/>
    <TransitionSeries.Sequence durationInFrames={177} name="Tabs and roll-up"><Tabs/></TransitionSeries.Sequence>
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({durationInFrames:12})}/>
    <TransitionSeries.Sequence durationInFrames={147} name="Room to breathe"><Focus/></TransitionSeries.Sequence>
    <TransitionSeries.Transition presentation={fade()} timing={linearTiming({durationInFrames:12})}/>
    <TransitionSeries.Sequence durationInFrames={144} name="Make room for focus"><Outro/></TransitionSeries.Sequence>
  </TransitionSeries>
</>;
