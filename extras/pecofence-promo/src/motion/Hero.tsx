import {AbsoluteFill, Easing, Interactive, interpolate, useCurrentFrame} from "remotion";
import {BrandMark} from "./BrandMark";
import {NativePanel} from "./NativePanel";

export const Hero = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{color: "#e7feff", overflow: "hidden"}}>
    <Interactive.Div name="Opening brand mark" style={{
      position: "absolute", left: 150, top: 180,
      scale: interpolate(frame, [0, 36], [.65, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.spring({damping: 18}), output: "perceptual-scale"}),
      rotate: interpolate(frame, [0, 36], ["-45deg", "0deg"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
    }}><BrandMark/></Interactive.Div>
    <Interactive.Div name="Opening product name" data-review-text="motion-brand" style={{
      position: "absolute", left: 150, top: 302, fontFamily: "Bahnschrift",
      fontSize: 166, fontWeight: 600, letterSpacing: -4, lineHeight: 1.08,
      textShadow:"0 0 30px #22e4f322",
      opacity: interpolate(frame, [0, 24], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      translate: interpolate(frame, [0, 24], ["0px 35px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
    }}>PecoFence</Interactive.Div>
    <Interactive.Div name="Opening definition" data-review-text="motion-definition" style={{
      position: "absolute", left: 158, top: 522, fontFamily: "Bahnschrift",
      fontSize: 78, fontWeight: 400, letterSpacing: -2, lineHeight: 1.2, color: "#a8dce3",
      opacity: interpolate(frame, [6, 30], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      translate: interpolate(frame, [6, 30], ["0px 24px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
    }}>A desktop organizer<br/>for Windows.</Interactive.Div>
    <Interactive.Div name="Art panel flies in" style={{
      position: "absolute", left: 1090, top: 190, width: 650, height: 420,
      opacity: interpolate(frame, [0, 18], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      scale: interpolate(frame, [0, 44], [.3, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.12,1,.2,1), output: "perceptual-scale"}),
      rotate: interpolate(frame, [0, 44], ["18deg", "-6deg"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
      translate: interpolate(frame, [0, 44], ["390px -210px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
      transform: `perspective(1800px) rotateY(${interpolate(frame, [0,44], [-32,-8], {extrapolateLeft:"clamp",extrapolateRight:"clamp"})}deg)`,
    }}><NativePanel asset="art"/></Interactive.Div>
    <Interactive.Div name="Work panel flies in" style={{
      position: "absolute", left: 1040, top: 560, width: 650, height: 420,
      opacity: interpolate(frame, [10, 28], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      scale: interpolate(frame, [10, 54], [.3, .91], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.12,1,.2,1), output: "perceptual-scale"}),
      rotate: interpolate(frame, [10, 54], ["-20deg", "5deg"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
      translate: interpolate(frame, [10, 54], ["180px 350px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1)}),
      transform: `perspective(1800px) rotateY(${interpolate(frame, [10,54], [28,6], {extrapolateLeft:"clamp",extrapolateRight:"clamp"})}deg)`,
    }}><NativePanel asset="work"/></Interactive.Div>
  </AbsoluteFill>;
};
