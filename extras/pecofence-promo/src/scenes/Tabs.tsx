import {Video} from "@remotion/media";
import {interpolate, staticFile, useCurrentFrame} from "remotion";
import {Base, Brand, blue, clamp, entrance, Reveal} from "../design";

export const Tabs = () => {
  const f = useCurrentFrame();
  return <Base>
    <Brand section="03 / MAKE SPACE" />
    <Reveal delay={4} style={{position:"absolute",left:110,top:233}}>
      <div style={{fontSize:111,fontWeight:640,letterSpacing:-5.4,lineHeight:1.08}}>
        More space.<br/><span style={{color:blue}}>Same desktop.</span>
      </div>
    </Reveal>
    <Reveal delay={24} style={{position:"absolute",left:116,top:553,fontSize:32,
      color:"#607184",lineHeight:1.5}}>Bring groups together.<br/>Tuck them away when you’re done.</Reveal>
    <div style={{position:"absolute",left:800,top:295,width:1020,height:535,borderRadius:30,
      overflow:"hidden",background:"#091c31",boxShadow:"0 35px 70px #18375624",
      opacity:entrance(f,12),translate:`0 ${(1-entrance(f,12))*35}px`}}>
      <Video src={staticFile("product-detail.mp4")} muted trimBefore={75} objectFit="contain"
        style={{width:"100%",height:"100%",
          scale:interpolate(f,[40,110],[1,1.3],clamp),transformOrigin:"8% 50%"}}/>
    </div>
    <Reveal delay={45} style={{position:"absolute",left:815,top:875,
      display:"flex",gap:35,color:"#52697e",fontSize:25}}>
      <div><span style={{color:blue,marginRight:10}}>▤</span> Tabbed groups</div>
      <div style={{opacity:interpolate(f,[100,120],[.35,1],clamp)}}>
        <span style={{color:blue,marginRight:10}}>⌃</span> Roll up on demand</div>
    </Reveal>
  </Base>;
};
