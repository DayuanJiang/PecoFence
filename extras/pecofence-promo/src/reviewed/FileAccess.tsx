import {Video} from "@remotion/media";
import {AbsoluteFill, CanvasImage, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";

export const FileAccess = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill name="Open Brief over the working application"
    style={{backgroundColor: "#091b30", overflow: "hidden", color: "#f5faff"}}>
    <CanvasImage name="Desktop backdrop" src={staticFile("reviewed/v3/grouped.png")}
      style={{position: "absolute", width: 1920, height: 1080, opacity: .35, filter: "blur(42px)", scale: 1.12}}/>
    <Interactive.Div name="File access benefit" data-review-text="access-title"
      style={{position: "absolute", left: 150, top: 180, fontFamily: "Segoe UI",
        fontSize: 150, fontWeight: 600, lineHeight: 1.1, letterSpacing: -4,
        opacity: interpolate(frame, [120, 126], [1, 0], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp",
        })}}>
      Files above your apps.
    </Interactive.Div>
    <Interactive.Div name="Documented shortcut cue" data-review-text="shortcut"
      style={{position: "absolute", left: 150, top: 195, fontFamily: "Segoe UI",
        fontSize: 100, fontWeight: 600, lineHeight: 1.2, letterSpacing: -2,
        color: "#b7ddff",
        opacity: interpolate(frame, [127, 135], [0, 1], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp",
        })}}>
      Ctrl + Alt + Space
    </Interactive.Div>
    <Interactive.Div name="Working application and native file-open result"
      style={{position: "absolute", left: 150, top: 380, width: 1620, height: 576,
        overflow: "hidden", borderRadius: 18, boxShadow: "0 22px 55px #0004"}}>
      <Video name="Read the access headline" src={staticFile("reviewed/v3/apps.mp4")}
        freeze={0} trimBefore={36} durationInFrames={90} muted
        style={{position: "absolute", width: 1620, height: 576}}/>
      <Video name="Peek and native Brief launch" src={staticFile("reviewed/v3/apps.mp4")}
        from={90} trimBefore={36} durationInFrames={210} muted
        style={{position: "absolute", width: 1620, height: 576}}/>
    </Interactive.Div>
  </AbsoluteFill>;
};
