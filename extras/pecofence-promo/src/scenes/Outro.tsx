import {interpolate, useCurrentFrame} from "remotion";
import {Base, blue, clamp, Mark, Reveal} from "../design";

export const Outro = () => {
  const f = useCurrentFrame();
  return <Base>
    <div style={{position:"absolute",left:585,top:128,width:750,height:750,
      borderRadius:"50%",border:"1px solid #b4d0e866",
      scale:interpolate(f,[0,144],[.94,1.05],clamp)}}/>
    <div style={{position:"absolute",left:488,top:31,width:944,height:944,
      borderRadius:"50%",border:"1px solid #b4d0e833",
      scale:interpolate(f,[0,144],[.94,1.05],clamp)}}/>
    <Reveal delay={7} style={{position:"absolute",top:246,left:0,width:"100%",
      display:"flex",alignItems:"center",justifyContent:"center",gap:33}}>
      <Mark size={105}/>
      <span style={{fontSize:132,fontWeight:650,letterSpacing:-7.5}}>PecoFence</span>
    </Reveal>
    <Reveal delay={21} style={{position:"absolute",left:0,top:470,width:"100%",
      textAlign:"center",fontSize:83,fontWeight:580,letterSpacing:-3.5,lineHeight:1.12}}>
      Make room <span style={{color:blue}}>for focus.</span>
    </Reveal>
    <Reveal delay={35} style={{position:"absolute",left:0,top:605,width:"100%",
      textAlign:"center",fontSize:32,color:"#687c90"}}>Your desktop, beautifully organized.</Reveal>
    <Reveal delay={46} style={{position:"absolute",left:0,top:767,width:"100%",
      display:"flex",justifyContent:"center",gap:37,color:"#566e85",fontSize:26}}>
      <span>Open source</span><span style={{color:"#a5b7c8"}}>·</span>
      <span>Lightweight</span><span style={{color:"#a5b7c8"}}>·</span><span>Windows 11</span>
    </Reveal>
    <Reveal delay={58} style={{position:"absolute",left:895,top:872,width:130,
      height:4,borderRadius:3,background:blue}}><span/></Reveal>
  </Base>;
};
