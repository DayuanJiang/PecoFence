import {AbsoluteFill, CanvasImage, Easing, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";

export const EndCard = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill name="Brand and availability"
    style={{backgroundColor: "#091b30", overflow: "hidden", color: "#f5faff"}}>
    <CanvasImage name="Actual organized desktop" src={staticFile("reviewed/v3/grouped.png")}
      style={{position: "absolute", width: 1920, height: 1080, transformOrigin: "85% 55%",
        scale: interpolate(frame, [0, 150], [1.1, 1.04], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp",
          easing: Easing.bezier(.16, 1, .3, 1), output: "perceptual-scale",
        })}}/>
    <AbsoluteFill name="Text contrast"
      style={{background: "linear-gradient(90deg, #07182a 0%, #07182a 57%, #07182adb 64%, #07182a30 76%, transparent 94%)"}}/>
    <Interactive.Div name="Closing product name" data-review-text="end-brand"
      style={{position: "absolute", left: 150, top: 245, fontFamily: "Segoe UI",
        fontSize: 150, fontWeight: 600, lineHeight: 1.1, letterSpacing: -6}}>
      PecoFence
    </Interactive.Div>
    <Interactive.Div name="Closing purpose" data-review-text="end-purpose"
      style={{position: "absolute", left: 150, top: 450, fontFamily: "Segoe UI",
        fontSize: 96, fontWeight: 400, lineHeight: 1.15, letterSpacing: -3}}>
      Organize your desktop.
    </Interactive.Div>
    <Interactive.Div name="Availability" data-review-text="availability"
      style={{position: "absolute", left: 150, top: 790, fontFamily: "Segoe UI",
        fontSize: 78, fontWeight: 400, lineHeight: 1.15, letterSpacing: -2, color: "#c5dff4"}}>
      Free &amp; open source · Windows 11
    </Interactive.Div>
  </AbsoluteFill>;
};
