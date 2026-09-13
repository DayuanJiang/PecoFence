import {AbsoluteFill, Interactive, interpolate, useCurrentFrame} from "remotion";

export const ScanGate = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{overflow: "hidden", pointerEvents: "none"}}>
    <AbsoluteFill style={{
      backgroundColor: "#021014",
      opacity: interpolate(frame, [0, 9, 12, 15, 24], [0, .2, .9, .2, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
    }}/>
    <Interactive.Div name="Electronic scan gate" style={{
      position: "absolute", left: 0, top: 0, height: 1080, width: 5,
      backgroundColor:"#c8ffff",boxShadow:"0 0 22px 8px #2bf7ff88, -70px 0 120px 35px #0fc8e025",
      translate: interpolate(frame, [0,24], ["-100px 0px","2020px 0px"], {extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
    }}/>
    <Interactive.Div name="Scanning raster" style={{
      position:"absolute",inset:0,
      backgroundImage:"repeating-linear-gradient(0deg, transparent 0px, transparent 14px, #4effff36 15px)",
      opacity:interpolate(frame,[0,9,15,24],[0,.5,.5,0],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
    }}/>
    {Array.from({length:9},(_,i)=><div key={i} style={{
      position:"absolute",left:0,top:74+i*112,height:5,width:150+(i%4)*70,
      background:"linear-gradient(90deg, transparent, #5af7ff)",
      opacity:interpolate(frame,[0,5,18,24],[0,.45,.45,0],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      translate:`${interpolate(frame,[0,24],[-800-i*70,2300-i*70],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}px 0px`,
    }}/>)}
  </AbsoluteFill>;
};
