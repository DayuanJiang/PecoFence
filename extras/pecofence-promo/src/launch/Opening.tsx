import {AbsoluteFill, interpolate, useCurrentFrame} from "remotion";
import {Desktop, motion, Night, Signature} from "./shared";

export const Opening = () => {
  const frame = useCurrentFrame();
  return <Night>
    <Desktop style={{
      scale: interpolate(frame, [0, 90], [1.29, 1.18], motion),
      transformOrigin: "84% 55%",
    }}/>
    <AbsoluteFill style={{
      background: "linear-gradient(90deg, #07182a 0%, #07182a 38%, #07182ae6 49%, transparent 77%)",
    }}/>
    <Signature/>
    <div style={{
      position: "absolute", left: 125, top: 288, fontSize: 174,
      lineHeight: .99, letterSpacing: -9, fontWeight: 620,
      translate: interpolate(frame, [0, 30], ["0px 28px", "0px 0px"], motion),
    }}>A calmer<br/>desktop<span style={{color: "#afd9ff"}}>.</span></div>
    <div style={{
      position: "absolute", left: 132, top: 755,
      fontSize: 43, color: "#bacce0", letterSpacing: -.8,
      opacity: interpolate(frame, [12, 28], [0, 1], motion),
    }}>Your space. Beautifully in order.</div>
  </Night>;
};
