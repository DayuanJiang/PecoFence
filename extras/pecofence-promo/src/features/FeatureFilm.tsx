import {Audio} from "@remotion/media";
import {AbsoluteFill,Sequence,staticFile} from "remotion";
import {FeatureIntro,FeatureOutro} from "./Bookends";
import {Groups} from "./Groups";
import {Rules} from "./Rules";
import {Portal} from "./Portal";
import {TabsDemo} from "./TabsDemo";
import {Peek} from "./Peek";
import {Hide} from "./Hide";

export const FeatureFilm=()=> <AbsoluteFill style={{background:"#f5f7fa"}}>
  <Audio src={staticFile("soundtrack-features.m4a")}/>
  <Sequence durationInFrames={120} name="What is PecoFence?"><FeatureIntro/></Sequence>
  <Sequence from={120} durationInFrames={300} name="01 Create groups"><Groups/></Sequence>
  <Sequence from={420} durationInFrames={360} name="02 Automatic rules"><Rules/></Sequence>
  <Sequence from={780} durationInFrames={390} name="03 Live folder portals"><Portal/></Sequence>
  <Sequence from={1170} durationInFrames={330} name="04 Tabbed groups"><TabsDemo/></Sequence>
  <Sequence from={1500} durationInFrames={330} name="05 Peek from any app"><Peek/></Sequence>
  <Sequence from={1830} durationInFrames={300} name="06 Quick hide and roll-up"><Hide/></Sequence>
  <Sequence from={2130} durationInFrames={120} name="PecoFence"><FeatureOutro/></Sequence>
</AbsoluteFill>;
