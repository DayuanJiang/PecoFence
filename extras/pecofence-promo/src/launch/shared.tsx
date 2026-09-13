import React from "react";
import {AbsoluteFill, CanvasImage, Easing, interpolate, staticFile, useCurrentFrame} from "remotion";
import {Mark} from "../design";

export const motion = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
  easing: Easing.bezier(.16, 1, .3, 1),
} as const;

export const Night: React.FC<{children: React.ReactNode}> = ({children}) =>
  <AbsoluteFill style={{background: "#0b1b31", color: "#f7fbff", overflow: "hidden"}}>
    {children}
  </AbsoluteFill>;

export const Desktop: React.FC<{style?: React.CSSProperties}> = ({style}) =>
  <CanvasImage src={staticFile("launch/raw/hero-result.png")} style={{
    position: "absolute", width: 1920, height: 1080, ...style,
  }}/>;

export const Signature = () => <div style={{
  position: "absolute", left: 130, top: 92, display: "flex",
  gap: 19, alignItems: "center", color: "#f4f8ff",
}}>
  <Mark size={43} color="#a9d1ff"/>
  <span style={{fontSize: 42, fontWeight: 600, letterSpacing: -1.6}}>PecoFence</span>
</div>;

export const Headline: React.FC<{children: React.ReactNode; size?: number}> =
({children, size = 150}) => {
  const frame = useCurrentFrame();
  return <div style={{
    position: "absolute", left: 125, top: 105, right: 110,
    fontSize: size, fontWeight: 620, lineHeight: 1.1, letterSpacing: -7,
    opacity: interpolate(frame, [0, 10], [0, 1], motion),
    translate: interpolate(frame, [0, 24], ["0px 24px", "0px 0px"], motion),
    textShadow: "0 4px 35px #00112424",
  }}>{children}</div>;
};
