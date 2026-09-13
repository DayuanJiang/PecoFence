import {CanvasImage, interpolate, staticFile, useCurrentFrame} from "remotion";
import {Base, Brand, blue, clamp, entrance, Panel, Reveal} from "../design";

export const Glass = () => {
  const f = useCurrentFrame();
  return <Base>
    <Brand section="02 / FEEL AT HOME" />
    <Reveal delay={6} style={{position: "absolute", top: 272, left: 110}}>
      <div style={{fontSize: 114, fontWeight: 640, letterSpacing: -5.4, lineHeight: 1.07}}>
        Native feel.<br/><span style={{color: blue}}>Clear thinking.</span>
      </div>
    </Reveal>
    <Reveal delay={26} style={{position: "absolute", left: 116, top: 593, fontSize: 33,
      color: "#607184", lineHeight: 1.5}}>
      Soft glass. Subtle detail.<br/>Right at home on Windows 11.
    </Reveal>
    <div style={{position: "absolute", left: 900, top: 189, width: 1160, height: 810,
      borderRadius: 44, overflow: "hidden", opacity: entrance(f,10),
      rotate: `${interpolate(f,[0,162],[4.5,1.5],clamp)}deg`,
      scale: interpolate(f,[0,162],[.96,1.02],clamp),
      boxShadow: "0 38px 95px #173a5c25"}}>
      <CanvasImage src={staticFile("wallpaper.png")} style={{width: "100%",height: "100%",objectFit:"cover"}}/>
      <Panel index={1} width={845} style={{position: "absolute", left: 86, top: 140,
        boxShadow: "0 45px 70px #0004"}}/>
      <div style={{position:"absolute",left:60,top:103,width:980,height:2,
        background:"linear-gradient(90deg,transparent,#c2e4ff66,transparent)",
        opacity:entrance(f,44)}}/>
    </div>
    <Reveal delay={53} style={{position:"absolute",left:116,bottom:134,fontSize:23,color:"#718296",
      letterSpacing:2.2}}>LIGHT ON YOUR DESKTOP. EASY ON THE EYES.</Reveal>
  </Base>;
};
