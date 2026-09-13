import {interpolate, useCurrentFrame} from "remotion";

const Pulse = ({arrival, target, startY}: {arrival: number; target: number; startY: number}) => {
  const frame = useCurrentFrame();
  const progress = interpolate(frame,[arrival-24,arrival],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"});
  const opacity = interpolate(frame,[arrival-26,arrival-22,arrival+2,arrival+14],[0,.9,.9,0],{extrapolateLeft:"clamp",extrapolateRight:"clamp"});
  let x: number, y: number;
  if(progress<.25){x=502+30*progress/.25;y=startY+(20-startY)*progress/.25;}
  else if(progress<.9){x=532+(target-562)*(progress-.25)/.65;y=20;}
  else{x=target-30+30*(progress-.9)/.1;y=20+30*(progress-.9)/.1;}
  return <g opacity={opacity}>
    <path d={`M502 ${startY}L532 20H${target-30}L${target} 50`} fill="none" stroke="#2ae7f0" strokeWidth="1.3" opacity=".28"/>
    <circle cx={x} cy={y} r="10" fill="#4cf8ff" opacity=".16"/>
    <circle cx={x} cy={y} r="3.5" fill="#d2ffff"/>
    <circle cx={target} cy="50" r={interpolate(frame,[arrival,arrival+14],[4,19],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}
      fill="none" stroke="#92fbff" strokeWidth="1.5"
      opacity={interpolate(frame,[arrival-1,arrival,arrival+14],[0,.8,0],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}/>
  </g>;
};

export const FileSignals = () => <svg width="1710" height="626" style={{position:"absolute",inset:0,pointerEvents:"none"}}>
  <Pulse arrival={95} target={1424} startY={208}/>
  <Pulse arrival={171} target={854} startY={320}/>
</svg>;
