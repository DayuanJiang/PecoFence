import {CanvasImage,staticFile} from "remotion";
import {Base,Brand,Mark,Panel,Reveal,blue} from "../design";

export const FeatureIntro=()=> <Base dark>
  <Brand dark section="PRODUCT TOUR"/>
  <Reveal style={{position:"absolute",top:154,left:108,color:"white",fontSize:82,
    fontWeight:620,letterSpacing:-3,lineHeight:1.08}}>A Windows desktop organizer.</Reveal>
  <Reveal delay={8} style={{position:"absolute",left:113,top:265,fontSize:31,color:"#bfd6e9"}}>
    Group files. Automate the sorting. Keep everything within reach.
  </Reveal>
  {[0,1,2].map(i=><Panel key={i} index={i} width={530}
    style={{position:"absolute",left:113+i*575,top:419}}/>)}
  <Reveal delay={18} style={{position:"absolute",left:113,top:940,fontSize:27,color:"#c5dbed"}}>
    {"Groups   ·   Automatic rules   ·   Live folders   ·   Tabs   ·   Peek   ·   Quick hide"}
  </Reveal>
</Base>;

export const FeatureOutro=()=> <Base>
  <Reveal style={{position:"absolute",top:163,left:0,width:"100%",display:"flex",
    justifyContent:"center",alignItems:"center",gap:24}}>
    <Mark size={75}/><span style={{fontSize:96,fontWeight:650,letterSpacing:-5}}>PecoFence</span>
  </Reveal>
  <Reveal delay={8} style={{position:"absolute",top:322,width:"100%",textAlign:"center",
    fontSize:65,fontWeight:580,letterSpacing:-2.2}}>An organized desktop. <span style={{color:blue}}>A smoother workflow.</span></Reveal>
  <div style={{position:"absolute",left:474,top:475,width:972,height:388,overflow:"hidden",
    borderRadius:20,boxShadow:"0 22px 55px #15375424"}}>
    <CanvasImage src={staticFile("features/tabs-result.png")}
      style={{position:"absolute",width:972,height:547,top:-109}}/>
  </div>
  <Reveal delay={20} style={{position:"absolute",top:922,width:"100%",textAlign:"center",
    fontSize:29,color:"#617a91"}}>{"Open source   ·   Lightweight   ·   Windows 11"}</Reveal>
</Base>;
