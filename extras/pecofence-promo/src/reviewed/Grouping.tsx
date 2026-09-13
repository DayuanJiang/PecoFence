import {Video} from "@remotion/media";
import {AbsoluteFill, Interactive, staticFile} from "remotion";

export const Grouping = () => {
  return <AbsoluteFill name="Group the same four files"
    style={{backgroundColor: "#091b30", overflow: "hidden", color: "#f5faff"}}>
    <Video name="Read the grouping headline" src={staticFile("reviewed/v3/flow.mp4")}
      freeze={0} trimBefore={51} durationInFrames={60} muted
      style={{position: "absolute", width: 1920, height: 1080}}/>
    <Video name="Native virtual-group moves" src={staticFile("reviewed/v3/flow.mp4")}
      from={60} trimBefore={51} durationInFrames={180} muted
      style={{position: "absolute", width: 1920, height: 1080}}/>
    <Interactive.Div name="Grouping benefit" data-review-text="grouping-title"
      style={{position: "absolute", left: 150, top: 180, fontFamily: "Segoe UI",
        fontSize: 150, fontWeight: 600, lineHeight: 1.1, letterSpacing: -4}}>
      Group files by project.
    </Interactive.Div>
  </AbsoluteFill>;
};
