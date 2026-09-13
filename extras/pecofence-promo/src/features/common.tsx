import React from "react";
import {Video} from "@remotion/media";
import {interpolate, staticFile, useCurrentFrame} from "remotion";
import {Base, Brand, blue, clamp, Reveal} from "../design";

type Step={at:number;label:string;note?:string};
export const FeatureShot:React.FC<{
  number:number;title:string;description:string;file:string;steps:Step[];
  children?:React.ReactNode;
}>=({number,title,description,file,steps,children})=>{
  const f=useCurrentFrame();
  const step=steps.reduce((last,item,index)=>f>=item.at?index:last,0);
  return <Base>
    <Brand section={`${String(number).padStart(2,"0")} / 06  —  FEATURES`}/>
    <Reveal style={{position:"absolute",left:100,top:125,fontSize:65,fontWeight:640,
      letterSpacing:-2.3,lineHeight:1.1}}>{title}</Reveal>
    <Reveal delay={7} style={{position:"absolute",left:104,top:212,fontSize:28,
      color:"#60768b"}}>{description}</Reveal>
    <div style={{position:"absolute",left:100,top:279,width:1720,height:660,
      overflow:"hidden",borderRadius:24,boxShadow:"0 16px 50px #15324d20",background:"#0b1b2d"}}>
      <Video src={staticFile(`features/${file}.mp4`)} muted objectFit="contain"
        style={{width:"100%",height:"100%"}}/>
      {children}
    </div>
    <div style={{position:"absolute",left:105,right:105,top:978,display:"flex",
      alignItems:"center",justifyContent:"space-between",fontSize:23,color:"#607489"}}>
      <div style={{display:"flex",alignItems:"center",gap:12}}>
        <span style={{width:9,height:9,borderRadius:"50%",background:blue}}/>
        <span style={{color:"#193850",fontWeight:600}}>{steps[step].label}</span>
        {steps[step].note&&<span style={{marginLeft:12}}>{steps[step].note}</span>}
      </div>
      <div style={{display:"flex",gap:9}}>{steps.map((_,i)=><div key={i} style={{
        width:35,height:4,borderRadius:4,background:i<=step?blue:"#cedbe6"}}/>)}</div>
    </div>
  </Base>;
};

export const Callout:React.FC<{from:number;to:number;children:React.ReactNode;
  left?:number;top?:number;width?:number}>=({from,to,children,left=40,top=550,width})=>{
  const f=useCurrentFrame();
  if(f<from||f>=to)return null;
  return <div style={{position:"absolute",left,top,width,padding:"14px 22px",borderRadius:12,
    background:"#f6fbfffa",color:"#17314c",boxShadow:"0 7px 25px #00152c22",fontSize:24,
    fontWeight:550,opacity:interpolate(f,[from,from+7,to-7,to],[0,1,1,0],clamp),
    translate:`0 ${interpolate(f,[from,from+10],[10,0],clamp)}px`}}>{children}</div>;
};

export const Keys:React.FC<{labels:string[]}>=({labels})=><div style={{display:"flex",alignItems:"center",gap:10}}>
  {labels.map((key,i)=><React.Fragment key={key}>
    {i>0&&<span style={{fontSize:22,color:"#7291aa"}}>+</span>}
    <span style={{padding:"8px 16px",borderRadius:8,border:"1px solid #bacbda",
      borderBottomWidth:4,background:"white",fontSize:27,fontWeight:600}}>{key}</span>
  </React.Fragment>)}
</div>;
