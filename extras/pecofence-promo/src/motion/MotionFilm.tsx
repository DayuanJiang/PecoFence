import {Audio} from "@remotion/media";
import {TransitionSeries} from "@remotion/transitions";
import {AbsoluteFill, staticFile} from "remotion";
import {Backdrop} from "./Backdrop";
import {Hero} from "./Hero";
import {Groups} from "./Groups";
import {Tabs} from "./Tabs";
import {Closing} from "./Closing";
import {ScanGate} from "./ScanGate";

export const MotionFilm = () => <AbsoluteFill>
  <Backdrop/>
  <Audio name="Energetic 128 BPM soundtrack" src={staticFile("soundtrack-energy.m4a")} durationInFrames={900} volume={1}/>
  <TransitionSeries>
    <TransitionSeries.Sequence durationInFrames={210} name="Glass panels assemble">
      <Hero/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Overlay durationInFrames={24}><ScanGate/></TransitionSeries.Overlay>
    <TransitionSeries.Sequence durationInFrames={240} name="New files sort automatically by type">
      <Groups/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Overlay durationInFrames={24}><ScanGate/></TransitionSeries.Overlay>
    <TransitionSeries.Sequence durationInFrames={240} name="Combine and switch project tabs">
      <Tabs/>
    </TransitionSeries.Sequence>
    <TransitionSeries.Overlay durationInFrames={24}><ScanGate/></TransitionSeries.Overlay>
    <TransitionSeries.Sequence durationInFrames={210} name="Make room for focus">
      <Closing/>
    </TransitionSeries.Sequence>
  </TransitionSeries>
</AbsoluteFill>;
