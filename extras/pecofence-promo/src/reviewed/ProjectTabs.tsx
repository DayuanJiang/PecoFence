import {Video} from "@remotion/media";
import {AbsoluteFill, Easing, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";

export const ProjectTabs = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill name="The same groups become tabs"
    style={{backgroundColor: "#091b30", overflow: "hidden", color: "#f5faff"}}>
    <Interactive.Div name="Tab close-up camera"
      style={{position: "absolute", width: 1920, height: 1080, transformOrigin: "0 0",
        scale: interpolate(frame, [108, 150], [1, 1.7], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp",
          easing: Easing.bezier(.16, 1, .3, 1), output: "perceptual-scale",
        }),
        translate: interpolate(frame, [108, 150], ["0px 0px", "-678px -304px"], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.16, 1, .3, 1),
        })}}>
      <Video name="Read the tabs headline" src={staticFile("reviewed/v3/flow.mp4")}
        freeze={0} trimBefore={231} durationInFrames={60} muted
        style={{position: "absolute", width: 1920, height: 1080}}/>
      <Video name="Native merge and Work selection" src={staticFile("reviewed/v3/flow.mp4")}
        from={60} trimBefore={231} durationInFrames={180} muted
        style={{position: "absolute", width: 1920, height: 1080}}/>
    </Interactive.Div>
    <Interactive.Div name="Tabs benefit" data-review-text="tabs-title"
      style={{position: "absolute", left: 150, top: 180, fontFamily: "Segoe UI",
        fontSize: 150, fontWeight: 600, lineHeight: 1.1, letterSpacing: -4}}>
      Switch project tabs.
    </Interactive.Div>
  </AbsoluteFill>;
};
