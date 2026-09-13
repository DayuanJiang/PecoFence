import {interpolate, useCurrentFrame} from "remotion";

export const BrandMark = () => {
  const frame = useCurrentFrame();
  return <svg width="88" height="88" viewBox="0 0 64 64" fill="none"
    style={{filter: "drop-shadow(0 0 14px #56ceff55)"}}>
    <path d="M27 8H14a6 6 0 0 0-6 6v13M37 8h13a6 6 0 0 1 6 6v13M8 37v13a6 6 0 0 0 6 6h13M56 37v13a6 6 0 0 1-6 6H37"
      stroke="#a8ecff" strokeWidth="4" strokeLinecap="round" pathLength="1"
      strokeDasharray="1" strokeDashoffset={interpolate(frame, [0, 30], [1, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}/>
    <g style={{opacity: interpolate(frame, [12, 30], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>
      <rect x="22" y="22" width="8" height="8" rx="2" fill="#cdf6ff"/>
      <rect x="35" y="22" width="8" height="8" rx="2" fill="#76baff"/>
      <rect x="22" y="35" width="8" height="8" rx="2" fill="#76baff"/>
      <rect x="35" y="35" width="8" height="8" rx="2" fill="#cdf6ff"/>
    </g>
  </svg>;
};
