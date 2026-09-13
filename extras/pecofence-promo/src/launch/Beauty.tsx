import {interpolate, useCurrentFrame} from "remotion";
import {Desktop, Headline, motion, Night} from "./shared";

export const Beauty = () => {
  const frame = useCurrentFrame();
  return <Night>
    <Desktop style={{
      scale: interpolate(frame, [0, 150], [1.045, 1], motion),
      transformOrigin: "50% 57%",
    }}/>
    <Headline>Beautifully organized.</Headline>
    <div style={{
      position: "absolute", left: 132, bottom: 115,
      fontSize: 42, letterSpacing: -.9, color: "#c2ddf6",
      opacity: interpolate(frame, [20, 40], [0, 1], motion),
    }}>Meet Liquid Glass.</div>
    <div style={{
      position: "absolute", right: 136, bottom: 116, display: "flex",
      alignItems: "center", gap: 16, fontSize: 32, color: "#9cbedb",
      opacity: interpolate(frame, [30, 48], [0, 1], motion),
    }}><span style={{width: 8, height: 8, borderRadius: "50%", background: "#b8e4f7"}}/>
      Your files, grouped your way
    </div>
  </Night>;
};
