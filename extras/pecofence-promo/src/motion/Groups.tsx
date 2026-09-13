import {Video} from "@remotion/media";
import {AbsoluteFill, Easing, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";
import {InstrumentFrame} from "./InstrumentFrame";
import {FileSignals} from "./FileSignals";
import {NewFiles} from "./NewFiles";

export const Groups = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{color: "#e7feff", overflow: "hidden"}}>
    <Interactive.Div name="Automatic sorting headline" data-review-text="motion-grouping" style={{
      position: "absolute", left: 150, top: 180, fontFamily: "Bahnschrift",
      fontSize: 150, fontWeight: 600, letterSpacing: -4, lineHeight: 1.1,
    }}>Auto-sort by file type.</Interactive.Div>
    <Interactive.Div name="Grouping accent" style={{
      position: "absolute", left: 155, top: 354, height: 3, width: 360,
      background: "linear-gradient(90deg, #65ffff, #65ffff00)", transformOrigin: "0 50%",
      scale: interpolate(frame, [0, 40], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
    }}/>
    <Interactive.Div name="Native desktop lands in perspective" style={{
      position: "absolute", left: 105, top: 388, width: 1710, height: 626,
      borderRadius: 10, overflow: "visible",
      boxShadow: "0 35px 90px #000c, 0 0 45px #15d9e315",
      scale: interpolate(frame, [0, 36], [.8, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.spring({damping: 22}), output: "perceptual-scale"}),
      translate: interpolate(frame, [0, 36], ["0px 160px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
      transform: `perspective(2000px) rotateX(${interpolate(frame, [0,36], [16,0], {extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)})}deg)`,
    }}>
      <AbsoluteFill style={{overflow:"hidden",clipPath:"polygon(0 0, calc(100% - 24px) 0, 100% 24px, 100% 100%, 24px 100%, 0 calc(100% - 24px))"}}>
        <Video name="Read before automatic sorting" src={staticFile("motion/autosort-flow.mp4")}
          freeze={0} trimBefore={60} durationInFrames={60} muted
          style={{width:1710,height:626,filter:"brightness(.82) saturate(.65) hue-rotate(-22deg) contrast(1.16)"}}/>
        <Video name="New PNG and TXT route automatically" src={staticFile("motion/autosort-flow.mp4")}
          from={60} trimBefore={60} durationInFrames={180} muted
          style={{width:1710,height:626,filter:"brightness(.82) saturate(.65) hue-rotate(-22deg) contrast(1.16)"}}/>
      </AbsoluteFill>
      <NewFiles/>
      <FileSignals/>
      <InstrumentFrame width={1710} height={626}/>
    </Interactive.Div>
    <Interactive.Div name="Moving frame highlight" style={{
      position: "absolute", left: 105, top: 388, width: 230, height: 2,
      background: "linear-gradient(90deg, transparent, #bcf8ff, transparent)",
      opacity: interpolate(frame, [35,55,190,225], [0,.85,.85,0], {extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      translate: interpolate(frame, [35,225], ["0px 0px","1480px 0px"], {extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
    }}/>
  </AbsoluteFill>;
};
