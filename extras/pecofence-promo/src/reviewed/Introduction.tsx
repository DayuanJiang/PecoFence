import {AbsoluteFill, CanvasImage, Interactive, staticFile} from "remotion";

export const Introduction = () => <AbsoluteFill name="Product introduction"
  style={{backgroundColor: "#091b30", overflow: "hidden", color: "#f5faff"}}>
  <CanvasImage name="Initial native desktop" src={staticFile("reviewed/intro-plate.png")}
    style={{position: "absolute", width: 1920, height: 1080}}/>
  <Interactive.Div name="Product name" data-review-text="intro-brand"
    style={{position: "absolute", left: 150, top: 180, fontFamily: "Segoe UI",
      fontSize: 150, fontWeight: 600, lineHeight: 1.1, letterSpacing: -6}}>
    PecoFence
  </Interactive.Div>
  <Interactive.Div name="What the product is" data-review-text="definition"
    style={{position: "absolute", left: 150, top: 365, fontFamily: "Segoe UI",
      fontSize: 78, fontWeight: 400, lineHeight: 1.15, letterSpacing: -2, color: "#d1e6f8"}}>
    A desktop organizer for Windows.
  </Interactive.Div>
</AbsoluteFill>;
