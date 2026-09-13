import {AbsoluteFill, Interactive, interpolate, useCurrentFrame} from "remotion";
import {TechField} from "./TechField";

export const Backdrop = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{backgroundColor: "#020609", overflow: "hidden"}}>
    <AbsoluteFill style={{background: "radial-gradient(ellipse 750px 690px at 1440px 530px, #06424c, #04131c 42%, #020609 75%)"}}/>
    <TechField/>
    <Interactive.Div name="Optical dot field" style={{
      position: "absolute", left: 970, top: 80, width: 850, height: 910,
      backgroundImage: "radial-gradient(#30cadb80 1px, transparent 1.4px)",
      backgroundSize: "28px 28px", opacity: .16,
      maskImage: "radial-gradient(ellipse, transparent 25%, black 70%, transparent 98%)",
    }}/>
    <Interactive.Div name="Perspective floor" style={{
      position: "absolute", left: -320, top: 440, width: 2600, height: 1400,
      backgroundImage: "linear-gradient(#27cdd840 1px, transparent 1px), linear-gradient(90deg, #27cdd840 1px, transparent 1px)",
      backgroundSize: "80px 80px", opacity: .48,
      transform: "perspective(900px) rotateX(65deg)",
      maskImage: "linear-gradient(transparent 10%, black 50%, transparent 95%)",
      translate: interpolate(frame, [0, 900], ["0px 0px", "0px -200px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
    }}/>
    <svg width="1920" height="1080" style={{position:"absolute",inset:0}} fill="none">
      <path d="M48 1016H900L960 956H1150M48 996H890L950 936H1150M70 108V142H125"
        stroke="#2bdded" strokeWidth="1" opacity=".32"/>
      <path d="M48 1016H900L960 956H1150" stroke="#abffff" strokeWidth="2"
        strokeDasharray="40 1100" strokeDashoffset={-frame * 3} opacity=".7"/>
      <path d="M1800 102H1850V150M1850 930V978H1800" stroke="#46cddc" strokeWidth="2" opacity=".45"/>
    </svg>
    <AbsoluteFill style={{
      backgroundImage:"repeating-linear-gradient(0deg, transparent 0px, transparent 5px, #45efff12 6px)",
      opacity:.18,
    }}/>
  </AbsoluteFill>;
};
