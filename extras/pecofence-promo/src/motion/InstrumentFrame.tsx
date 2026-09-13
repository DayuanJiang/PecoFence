import {useCurrentFrame} from "remotion";

export const InstrumentFrame = ({width, height}: {width: number; height: number}) => {
  const frame = useCurrentFrame();
  const w = width + 28, h = height + 28;
  return <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`}
    style={{position: "absolute", left: -14, top: -14, pointerEvents: "none", overflow: "visible"}}>
    <path d={`M82 8H8V82M${w-82} 8H${w-8}V82M8 ${h-82}V${h-8}H82M${w-82} ${h-8}H${w-8}V${h-82}`}
      fill="none" stroke="#54f3fa" strokeWidth="2" opacity=".85"
      style={{filter: "drop-shadow(0 0 6px #2ae6ed66)"}}/>
    <rect x="14" y="14" width={width} height={height} rx="9" fill="none" stroke="#1abac7" opacity=".38"/>
    <rect x="14" y="14" width={width} height={height} rx="9" fill="none"
      stroke="#bbffff" strokeWidth="2" strokeDasharray="38 1400 10 640" strokeDashoffset={-frame * 7} opacity=".8"/>
    <path d={`M${w/2-30} 8h18m24 0h18M${w/2} 2v12M${w/2-30} ${h-8}h18m24 0h18M${w/2} ${h-14}v12`}
      stroke="#65f2f3" strokeWidth="1.5" opacity=".65"/>
  </svg>;
};
