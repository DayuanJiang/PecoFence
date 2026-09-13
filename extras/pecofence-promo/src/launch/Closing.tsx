import {AbsoluteFill, interpolate, useCurrentFrame} from "remotion";
import {Mark} from "../design";
import {Desktop, motion, Night} from "./shared";

export const Closing = () => {
  const frame = useCurrentFrame();
  return <Night>
    <Desktop style={{
      scale: interpolate(frame, [0, 150], [1.12, 1.06], motion),
      transformOrigin: "85% 58%",
      opacity: .7,
    }}/>
    <AbsoluteFill style={{
      background: "linear-gradient(90deg, #06182af5 0%, #06182adb 40%, #06182a44 70%, #06182a11)",
    }}/>
    <div style={{
      position: "absolute", left: 126, top: 244, display: "flex", alignItems: "center", gap: 33,
      translate: interpolate(frame, [0, 28], ["0px 22px", "0px 0px"], motion),
    }}>
      <Mark size={112} color="#b3dcff"/>
      <span style={{fontSize: 156, fontWeight: 650, letterSpacing: -9}}>PecoFence</span>
    </div>
    <div style={{
      position: "absolute", left: 130, top: 475, fontSize: 83,
      lineHeight: 1.13, letterSpacing: -3.6, fontWeight: 450,
      opacity: interpolate(frame, [7, 23], [0, 1], motion),
    }}>Make room for focus.</div>
    <div style={{
      position: "absolute", left: 136, bottom: 141, color: "#c7d9ed",
      fontSize: 39, letterSpacing: -.7,
      opacity: interpolate(frame, [20, 38], [0, 1], motion),
    }}>Free &amp; open source <span style={{padding: "0 18px", color: "#83a7c4"}}>·</span> Windows 11</div>
  </Night>;
};
