import {AbsoluteFill, CanvasImage, interpolate, staticFile, useCurrentFrame} from "remotion";
import {InstrumentFrame} from "./InstrumentFrame";

export const NativePanel = ({asset}: {asset: "work" | "art"}) => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{
    overflow: "visible", boxShadow: "0 35px 90px #000c",
  }}>
    <AbsoluteFill style={{borderRadius:49,overflow:"hidden",backgroundColor:"#03151d"}}>
      <CanvasImage src={staticFile(`motion/autosort-${asset}.png`)} style={{
        width:"100%",height:"100%",filter:"brightness(.82) saturate(.65) hue-rotate(-22deg) contrast(1.16)",
        clipPath:`inset(0 0 ${interpolate(frame,[0,44],[100,0],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}% 0)`,
      }}/>
      <AbsoluteFill style={{
        backgroundImage:"repeating-linear-gradient(0deg, transparent 0px, transparent 5px, #73f8ff30 6px)",
        opacity:.18,
      }}/>
      <div style={{
        position:"absolute",left:0,right:0,height:2,
        top:interpolate(frame,[0,44,180,210],[0,420,0,250],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
        backgroundColor:"#9effff",boxShadow:"0 0 16px #13ffff, 0 -14px 30px #1affff55",
        opacity:interpolate(frame,[0,44,55,150,180,210],[.9,.9,.08,.1,.3,.1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      }}/>
      <AbsoluteFill style={{borderRadius:49,border:"1px solid #43d6e055"}}/>
    </AbsoluteFill>
    <InstrumentFrame width={650} height={420}/>
  </AbsoluteFill>;
};
