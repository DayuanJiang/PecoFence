import {interpolate, useCurrentFrame} from "remotion";
import {Base, Brand, clamp, entrance, Panel, Pointer, Reveal} from "../design";

export const Focus = () => {
  const f = useCurrentFrame();
  const visible = interpolate(f,[0,63,72,112,121,147],[1,1,0,0,1,1],clamp);
  return <Base dark>
    <Brand dark section="04 / FIND YOUR FOCUS" />
    <Reveal delay={5} style={{position:"absolute",left:0,top:167,width:"100%",textAlign:"center"}}>
      <div style={{color:"white",fontSize:100,fontWeight:620,letterSpacing:-4.5,lineHeight:1.07}}>
        Your desktop.<br/>Room to breathe.
      </div>
      <div style={{marginTop:26,fontSize:30,color:"#bdd5ea"}}>
        Double-click to hide. Double-click to return.
      </div>
    </Reveal>
    <div style={{opacity:visible}}>
      {[0,1,2].map((index)=><Panel key={index} index={index} width={410}
        style={{position:"absolute",left:275+index*480,top:560}}/>)}
    </div>
    <div style={{opacity:entrance(f,25)}}>
      <Pointer x={1100+interpolate(f,[25,55],[150,0],clamp)} y={920}
        click={f < 100 ? 63 : 113} dark/>
    </div>
  </Base>;
};
