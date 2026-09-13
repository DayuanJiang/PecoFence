import {Video} from "@remotion/media";
import {AbsoluteFill, Easing, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";
import {InstrumentFrame} from "./InstrumentFrame";

export const Tabs = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{color: "#e7feff", overflow: "hidden"}}>
    <Interactive.Div name="Tabs headline" data-review-text="motion-tabs" style={{
      position: "absolute", left: 150, top: 180, fontFamily: "Bahnschrift",
      fontSize: 150, fontWeight: 600, letterSpacing: -4, lineHeight: 1.1,
    }}>Switch project tabs.</Interactive.Div>
    <Interactive.Div name="Tabs accent" style={{
      position: "absolute", left: 155, top: 354, height: 3, width: 360,
      background: "linear-gradient(90deg, #65ffff, #65ffff00)", transformOrigin: "0 50%",
      scale: interpolate(frame, [0,40], [0,1], {extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
    }}/>
    <Interactive.Div name="Native tabs display" style={{
      position: "absolute", left: 105, top: 388, width: 1710, height: 626,
      borderRadius: 10, overflow: "visible",
      boxShadow: "0 35px 90px #000c, 0 0 45px #15d9e315",
    }}>
      <AbsoluteFill style={{overflow:"hidden",clipPath:"polygon(0 0, calc(100% - 24px) 0, 100% 24px, 100% 100%, 24px 100%, 0 calc(100% - 24px))"}}>
      <Interactive.Div name="Push into merged native tabs" style={{
        position: "absolute", width: 1710, height: 626, transformOrigin: "0 0",
        scale: interpolate(frame, [108,156], [1,1.7], {extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1),output:"perceptual-scale"}),
        translate: interpolate(frame, [108,156], ["0px 0px","-596px -40px"], {extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
      }}>
        <Video name="Read before switching" src={staticFile("motion/autosort-flow.mp4")}
          freeze={0} trimBefore={321} durationInFrames={60} muted
          style={{width:1710,height:626,filter:"brightness(.82) saturate(.65) hue-rotate(-22deg) contrast(1.16)"}}/>
        <Video name="Real merge and tab switch" src={staticFile("motion/autosort-flow.mp4")}
          from={60} trimBefore={321} durationInFrames={180} muted
          style={{width:1710,height:626,filter:"brightness(.82) saturate(.65) hue-rotate(-22deg) contrast(1.16)"}}/>
      </Interactive.Div>
      <Interactive.Div name="Merged tab focus halo" style={{
        position: "absolute", left: 442, top: 45, width: 828, height: 535,
        opacity: interpolate(frame, [151,175,205,234], [0,.8,.8,0], {extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
        scale: interpolate(frame, [151,175], [1.055,1], {extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1),output:"perceptual-scale"}),
      }}><InstrumentFrame width={828} height={535}/></Interactive.Div>
      </AbsoluteFill>
      <InstrumentFrame width={1710} height={626}/>
    </Interactive.Div>
  </AbsoluteFill>;
};
