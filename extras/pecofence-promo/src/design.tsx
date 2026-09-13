import React from "react";
import {
  AbsoluteFill, CanvasImage, Easing, interpolate, spring,
  staticFile, useCurrentFrame,
} from "remotion";

export const ink = "#142338";
export const blue = "#1377ed";
export const ease = Easing.bezier(.16, 1, .3, 1);
export const clamp = {extrapolateLeft: "clamp", extrapolateRight: "clamp"} as const;
export const entrance = (frame: number, delay = 0, duration = 28) =>
  interpolate(frame, [delay, delay + duration], [0, 1], {...clamp, easing: ease});

export const Mark: React.FC<{size?: number; color?: string}> = ({size = 42, color = blue}) => (
  <svg width={size} height={size} viewBox="0 0 64 64" fill="none">
    <path d="M27 8H14a6 6 0 0 0-6 6v13M37 8h13a6 6 0 0 1 6 6v13M8 37v13a6 6 0 0 0 6 6h13M56 37v13a6 6 0 0 1-6 6H37"
      stroke={color} strokeWidth="6" strokeLinecap="round"/>
    <rect x="22" y="22" width="8" height="8" rx="2" fill={color}/>
    <rect x="35" y="22" width="8" height="8" rx="2" fill={color} opacity=".65"/>
    <rect x="22" y="35" width="8" height="8" rx="2" fill={color} opacity=".65"/>
    <rect x="35" y="35" width="8" height="8" rx="2" fill={color}/>
  </svg>
);

export const Base: React.FC<{dark?: boolean; children: React.ReactNode}> = ({dark, children}) => (
  <AbsoluteFill style={{overflow: "hidden", background: dark ? "#091727" : "#f5f7fa"}}>
    {dark ? <CanvasImage src={staticFile("wallpaper.png")}
      style={{position: "absolute", width: "100%", height: "100%", objectFit: "cover"}} /> :
      <AbsoluteFill style={{background: "radial-gradient(ellipse at 82% 32%, #e0edf9 0%, transparent 55%), radial-gradient(ellipse at 10% 100%, #e6f2ef 0%, transparent 42%)"}} />}
    {children}
  </AbsoluteFill>
);

export const Brand: React.FC<{dark?: boolean; section?: string}> = ({dark, section}) => (
  <div style={{position: "absolute", top: 62, left: 110, right: 110,
    display: "flex", alignItems: "center", justifyContent: "space-between",
    color: dark ? "#fff" : ink}}>
    <div style={{display: "flex", alignItems: "center", gap: 15}}>
      <Mark size={34} color={dark ? "#8bc5ff" : blue}/>
      <span style={{fontSize: 32, fontWeight: 650, letterSpacing: -1.2}}>PecoFence</span>
    </div>
    <span style={{fontSize: 21, fontWeight: 600, letterSpacing: 3.3,
      color: dark ? "#aac0d6" : "#6d7c8e"}}>{section}</span>
  </div>
);

export const Reveal: React.FC<{children: React.ReactNode; delay?: number; style?: React.CSSProperties}> =
({children, delay = 0, style}) => {
  const f = useCurrentFrame();
  return <div style={{...style,
    opacity: interpolate(f, [delay, delay + 20], [0, 1], clamp),
    translate: `0 ${interpolate(f, [delay, delay + 32], [30, 0], {...clamp, easing: ease})}px`,
  }}>{children}</div>;
};

export const Panel: React.FC<{index: number; width?: number; style?: React.CSSProperties}> =
({index, width = 520, style}) =>
  <CanvasImage src={staticFile(`panel-${index}.png`)} style={{
    width, borderRadius: width * .013, boxShadow: "0 30px 65px #020b1c55",
    ...style,
  }}/>;

export const Pointer: React.FC<{x: number; y: number; click?: number; dark?: boolean}> =
({x, y, click = -100, dark}) => {
  const f = useCurrentFrame();
  const p = interpolate(f, [click, click + 17], [0, 1], clamp);
  return <div style={{position: "absolute", left: x, top: y, width: 80, height: 80}}>
    {f >= click && f < click + 17 && <div style={{position: "absolute", left: -40,
      top: -40, width: 100, height: 100, borderRadius: "50%",
      border: `3px solid ${dark ? "#c9e5ff" : blue}`, opacity: 1 - p, scale: .3 + p}} />}
    <svg width="42" height="53" viewBox="0 0 42 53" style={{filter: "drop-shadow(0 3px 5px #0004)"}}>
      <path d="M5 3v38l10-10 8 17 8-4-8-16h14L5 3Z" fill="white" stroke="#183044" strokeWidth="2.5" strokeLinejoin="round"/>
    </svg>
  </div>;
};

export const DesktopIcon: React.FC<{type?: "folder" | "document"; label?: string; size?: number;
  style?: React.CSSProperties; dark?: boolean}> = ({type = "folder", label, size = 66, style, dark}) => (
  <div style={{position: "absolute", width: 134, textAlign: "center", ...style}}>
    <CanvasImage src={staticFile(`${type}-icon.png`)} style={{width: size, height: size,
      imageRendering: "auto", filter: "drop-shadow(0 6px 7px #07163216)"}} />
    {label && <div style={{fontSize: 19, lineHeight: 1.28, marginTop: 10,
      color: dark ? "#eef5ff" : "#4b5c70", fontWeight: 500}}>{label}</div>}
  </div>
);

export const settle = (f: number, delay = 0) => spring({
  frame: f - delay, fps: 30, config: {damping: 22, stiffness: 90, mass: .85},
});
