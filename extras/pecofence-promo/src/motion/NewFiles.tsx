import {CanvasImage, Interactive, interpolate, staticFile, useCurrentFrame} from "remotion";

export const NewFiles = () => {
  const frame = useCurrentFrame();
  return <Interactive.Div name="Editorial new-file cues" style={{
    position: "absolute", left: 62, top: 50, width: 440, height: 350,
    background: "linear-gradient(125deg, #03161eee, #041922c9)",
    border: "1px solid #70eefb44", borderRadius: 14,
    boxShadow: "0 16px 40px #0005",
  }}>
    <Interactive.Div name="New files label" style={{
      position: "absolute", left: 30, top: 24, fontFamily: "Bahnschrift",
      fontSize: 58, lineHeight: 1.1, color: "#b9f6fc", letterSpacing: -1,
    }}>New files</Interactive.Div>
    <Interactive.Div name="New PNG file" style={{
      position: "absolute", left: 26, top: 111, width: 388, height: 94,
      display: "flex", alignItems: "center", gap: 20,
      opacity: interpolate(frame, [67, 75], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      translate: interpolate(frame, [67, 79], ["0px 14px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
    }}>
      <CanvasImage src={staticFile("launch/demo-images/Coast.png")}
        style={{width: 88, height: 62, objectFit: "cover", borderRadius: 6}}/>
      <Interactive.Div name="PNG filename" style={{
        fontFamily: "Bahnschrift", fontSize: 50, lineHeight: 1.1,
        color: "#e7feff", letterSpacing: -1,
      }}>Coast.png</Interactive.Div>
    </Interactive.Div>
    <Interactive.Div name="New text file" style={{
      position: "absolute", left: 26, top: 223, width: 388, height: 94,
      display: "flex", alignItems: "center", gap: 20,
      opacity: interpolate(frame, [143, 151], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
      translate: interpolate(frame, [143, 155], ["0px 14px", "0px 0px"], {extrapolateLeft: "clamp", extrapolateRight: "clamp"}),
    }}>
      <CanvasImage src={staticFile("document-icon.png")}
        style={{width: 88, height: 78, objectFit: "contain"}}/>
      <Interactive.Div name="Text filename" style={{
        fontFamily: "Bahnschrift", fontSize: 50, lineHeight: 1.1,
        color: "#e7feff", letterSpacing: -1,
      }}>Notes.txt</Interactive.Div>
    </Interactive.Div>
  </Interactive.Div>;
};
