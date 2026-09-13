import {Video} from "@remotion/media";
import {AbsoluteFill, interpolate, staticFile, useCurrentFrame} from "remotion";
import {Desktop, Headline, motion, Night} from "./shared";

export const WithinReach = () => {
  const frame = useCurrentFrame();
  return <Night>
    <Desktop style={{filter: "blur(48px)", scale: 1.15, opacity: .6}}/>
    <AbsoluteFill style={{background: "linear-gradient(#08172955, #081729aa)"}}/>
    <Headline>One shortcut away.</Headline>
    <div style={{
      position: "absolute", left: 96, top: 325, width: 1728, height: 688,
      borderRadius: 22, overflow: "hidden", boxShadow: "0 25px 75px #0005",
      border: "1px solid #d5ecff22",
    }}>
      <Video src={staticFile("launch/peek.mp4")} muted
        style={{position: "absolute", width: 1728, height: 688}}/>
    </div>
    <div style={{
      position: "absolute", right: 132, top: 269, display: "flex", alignItems: "center",
      gap: 12, fontSize: 32, fontWeight: 500,
      opacity: interpolate(frame, [43, 55, 157, 168], [0, 1, 1, 0], motion),
      translate: interpolate(frame, [43, 62], ["0px 12px", "0px 0px"], motion),
    }}>
      <span style={{color: "#c7e1f9", marginRight: 8}}>Peek</span>
      {["Ctrl", "Alt", "Space"].map(key => <span key={key} style={{
        background: "#e8f4ff16", border: "1px solid #d2eaff45",
        padding: "5px 15px 7px", borderRadius: 9, lineHeight: 1,
      }}>{key}</span>)}
    </div>
  </Night>;
};
