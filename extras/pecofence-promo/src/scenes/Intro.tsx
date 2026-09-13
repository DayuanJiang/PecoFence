import {interpolate, useCurrentFrame} from "remotion";
import {Base, Brand, blue, clamp, DesktopIcon, entrance, Reveal} from "../design";

const items = [
  [925,225,-12,"Brand kit"],[1210,155,8,"Ideas"],[1560,285,-7,"Weekly plan"],
  [1090,455,9,"Launch plan"],[1390,440,-11,"Project notes"],[1660,560,7,"Inspiration"],
  [970,710,-5,"Archive"],[1280,750,12,"Meeting notes"],[1555,825,-9,"Design files"],
] as const;

export const Intro = () => {
  const f = useCurrentFrame();
  return <Base>
    <Brand section="A CLEARER DESKTOP" />
    <div style={{position: "absolute", left: 790, top: 110, width: 1150, height: 950,
      borderRadius: "50%", background: "radial-gradient(ellipse, #c9def044, transparent 67%)"}}/>
    {items.map(([x,y,r,label], i) => <DesktopIcon key={label} label={label}
      type={i % 3 === 2 ? "document" : "folder"} size={82}
      style={{left: x, top: y + Math.sin(f / 43 + i) * 8,
        opacity: entrance(f, 7 + i * 2) * .88,
        rotate: `${r * interpolate(f, [0,141], [1,.75], clamp)}deg`,
        scale: interpolate(f, [0,141], [.94, 1.03], clamp)}} />)}
    <Reveal delay={5} style={{position: "absolute", top: 284, left: 110}}>
      <div style={{fontSize: 122, fontWeight: 650, lineHeight: 1.04, letterSpacing: -6}}>
        Less clutter.<br/><span style={{color: blue}}>More clarity.</span>
      </div>
    </Reveal>
    <Reveal delay={28} style={{position: "absolute", top: 609, left: 116,
      fontSize: 34, color: "#596b7e", lineHeight: 1.5}}>
      Meet your desktop,<br/>beautifully organized.
    </Reveal>
    <Reveal delay={40} style={{position: "absolute", bottom: 121, left: 116,
      display: "flex", alignItems: "center", gap: 15, fontSize: 23, color: "#718093"}}>
      <div style={{width: 38, height: 2, background: blue}}/> Made for Windows 11
    </Reveal>
  </Base>;
};
