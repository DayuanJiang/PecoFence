import {interpolate, useCurrentFrame} from "remotion";
import {Base, Brand, clamp, DesktopIcon, entrance, Panel, Reveal, settle} from "../design";

export const Organize = () => {
  const f = useCurrentFrame();
  return <Base dark>
    <Brand dark section="01 / GET ORGANIZED" />
    <Reveal delay={4} style={{position: "absolute", top: 165, width: "100%", textAlign: "center"}}>
      <div style={{fontSize: 100, fontWeight: 620, lineHeight: 1.1, letterSpacing: -4, color: "white"}}>
        Everything in its place.
      </div>
      <div style={{fontSize: 32, color: "#b9d0e6", marginTop: 22}}>Group your files. Keep your flow.</div>
    </Reveal>
    {[0,1,2].map((index) => {
      const delay = 25 + index * 9;
      const p = settle(f, delay);
      return <div key={index} style={{position: "absolute", left: 118 + index * 571,
        top: 420 + (1 - p) * 105, scale: .93 + .07 * p,
        opacity: entrance(f, delay, 20)}}>
        <Panel index={index} width={542}/>
      </div>;
    })}
    {Array.from({length: 9}, (_, i) => {
      const start = 18 + i * 4, p = entrance(f,start,32);
      const group = i % 3;
      return f < start + 34 ? <DesktopIcon key={i}
        type={group === 2 ? "document" : "folder"} dark size={58}
        style={{left: interpolate(p,[0,1],[380 + (i * 157) % 1210, 165 + group * 571 + (Math.floor(i/3)%3)*130]),
          top: interpolate(p,[0,1],[180 + (i%2)*110,510]), opacity: interpolate(f,
            [start-5,start,start+22,start+34],[0,1,1,0],clamp),
          rotate: `${(1-p) * (i%2 ? 18 : -18)}deg`}}/> : null;
    })}
    <Reveal delay={90} style={{position: "absolute", top: 908, width: "100%", textAlign: "center",
      color: "#c2d8ea", fontSize: 29, letterSpacing: .2}}>
      Automatic rules. Zero busywork.
    </Reveal>
  </Base>;
};
