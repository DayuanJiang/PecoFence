import {AbsoluteFill, Easing, Interactive, interpolate, useCurrentFrame} from "remotion";
import {BrandMark} from "./BrandMark";
import {NativePanel} from "./NativePanel";

export const Closing = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{color: "#e7feff", overflow: "hidden"}}>
    <Interactive.Div name="Closing mark settles" style={{
      position:"absolute",left:150,top:180,
      scale:interpolate(frame,[0,36],[1.6,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.spring({damping:20}),output:"perceptual-scale"}),
      rotate:interpolate(frame,[0,36],["70deg","0deg"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
    }}><BrandMark/></Interactive.Div>
    <Interactive.Div name="Closing brand" data-review-text="motion-end-brand" style={{
      position:"absolute",left:150,top:300,fontFamily:"Bahnschrift",fontSize:166,fontWeight:600,lineHeight:1.08,letterSpacing:-4,textShadow:"0 0 30px #22e4f322",
      opacity:interpolate(frame,[0,24],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      translate:interpolate(frame,[0,24],["50px 0px","0px 0px"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
    }}>PecoFence</Interactive.Div>
    <Interactive.Div name="Closing promise" data-review-text="motion-end-promise" style={{
      position:"absolute",left:157,top:512,fontFamily:"Bahnschrift",fontSize:88,fontWeight:400,lineHeight:1.15,letterSpacing:-3,color:"#d6fcff",
      opacity:interpolate(frame,[6,30],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
    }}>Make room for focus.</Interactive.Div>
    <Interactive.Div name="Closing availability" data-review-text="motion-availability" style={{
      position:"absolute",left:158,top:700,fontFamily:"Bahnschrift",fontSize:78,fontWeight:400,lineHeight:1.2,letterSpacing:-2,color:"#9bd3dd",
      opacity:interpolate(frame,[6,30],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
    }}>Free &amp; open source<br/>Windows 11</Interactive.Div>
    <Interactive.Div name="Closing Work assembly" style={{
      position:"absolute",left:1090,top:205,width:650,height:420,
      opacity:interpolate(frame,[0,18],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      scale:interpolate(frame,[0,42],[.6,.96],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.spring({damping:20}),output:"perceptual-scale"}),
      rotate:interpolate(frame,[0,42],["-18deg","-5deg"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
      translate:interpolate(frame,[0,42],["360px -190px","0px 0px"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
      transform:`perspective(1800px) rotateY(${interpolate(frame,[0,42],[-30,-7],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}deg)`,
    }}><NativePanel asset="work"/></Interactive.Div>
    <Interactive.Div name="Closing Art assembly" style={{
      position:"absolute",left:1080,top:558,width:650,height:420,
      opacity:interpolate(frame,[9,27],[0,1],{extrapolateLeft:"clamp",extrapolateRight:"clamp"}),
      scale:interpolate(frame,[9,51],[.55,.91],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.spring({damping:20}),output:"perceptual-scale"}),
      rotate:interpolate(frame,[9,51],["22deg","5deg"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
      translate:interpolate(frame,[9,51],["250px 260px","0px 0px"],{extrapolateLeft:"clamp",extrapolateRight:"clamp",easing:Easing.bezier(.16,1,.3,1)}),
      transform:`perspective(1800px) rotateY(${interpolate(frame,[9,51],[26,5],{extrapolateLeft:"clamp",extrapolateRight:"clamp"})}deg)`,
    }}><NativePanel asset="art"/></Interactive.Div>
  </AbsoluteFill>;
};
