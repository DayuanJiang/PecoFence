import {useCurrentFrame} from "remotion";

const project = (x: number, y: number, z: number, spin: number) => {
  const rx = x * Math.cos(spin) + z * Math.sin(spin);
  const rz = -x * Math.sin(spin) + z * Math.cos(spin);
  const ry = y * Math.cos(.34) - rz * Math.sin(.34);
  const depth = y * Math.sin(.34) + rz * Math.cos(.34);
  const scale = 1700 / (1700 + depth);
  return `${(1440 + rx * scale).toFixed(2)},${(530 + ry * scale).toFixed(2)}`;
};

export const TechField = () => {
  const frame = useCurrentFrame();
  const spin = frame * .004;
  const meridians = Array.from({length: 12}, (_, i) =>
    Array.from({length: 65}, (_, j) => {
      const lat = -Math.PI / 2 + j * Math.PI / 64;
      const lon = i * Math.PI / 6;
      return project(340 * Math.cos(lat) * Math.cos(lon), 340 * Math.sin(lat), 340 * Math.cos(lat) * Math.sin(lon), spin);
    }).join(" "));
  const parallels = [-60,-40,-20,0,20,40,60].map(degrees =>
    Array.from({length: 81}, (_, j) => {
      const lat = degrees * Math.PI / 180, lon = j * Math.PI / 40;
      return project(340 * Math.cos(lat) * Math.cos(lon), 340 * Math.sin(lat), 340 * Math.cos(lat) * Math.sin(lon), spin);
    }).join(" "));
  return <svg width="1920" height="1080" viewBox="0 0 1920 1080"
    style={{position: "absolute", inset: 0, maskImage: "linear-gradient(90deg, transparent 43%, black 55%)"}}>
    <g fill="none" stroke="#31d8e8" strokeWidth="1" opacity=".22">
      {meridians.map((points, i) => <polyline key={`meridian-${i}`} points={points}/>)}
      {parallels.map((points, i) => <polyline key={`parallel-${i}`} points={points}/>)}
    </g>
    <g fill="none" transform={`translate(1440 530) rotate(${frame * .18})`}>
      <circle r="423" stroke="#2eefff" strokeWidth="2" opacity=".45" strokeDasharray="170 1050 90 1348"/>
      <circle r="448" stroke="#67ebf5" strokeWidth="1.5" opacity=".32" strokeDasharray="3 24"/>
      <circle r="466" stroke="#23959e" strokeWidth="1" opacity=".36" strokeDasharray="130 48 12 240"/>
      <path d="M-480 0h40M440 0h40M0-480v40M0 440v40" stroke="#8af7ff" strokeWidth="2" opacity=".7"/>
    </g>
    <g fill="none" transform={`translate(1440 530) rotate(${-27 - frame * .08})`}>
      <ellipse rx="470" ry="175" stroke="#3af7f0" strokeWidth="1.4" opacity=".3"/>
      <ellipse rx="470" ry="175" stroke="#a1fffc" strokeWidth="3" opacity=".58" strokeDasharray="110 2100" strokeDashoffset={-frame * 2.2}/>
    </g>
    {Array.from({length: 48}, (_, i) => {
      const phase = ((frame + i * 23) % 240) / 240;
      const angle = i * 2.399963;
      const radius = 180 + phase * 380;
      const x = 1440 + Math.cos(angle) * radius;
      const y = 530 + Math.sin(angle) * radius * .9;
      return <g key={`particle-${i}`} opacity={Math.sin(phase * Math.PI) * .55}>
        <line x1={x} y1={y} x2={1440 + Math.cos(angle) * (radius - 14)}
          y2={530 + Math.sin(angle) * (radius - 14) * .9} stroke="#3ae5f5" strokeWidth="1"/>
        <circle cx={x} cy={y} r={1 + phase * 1.4} fill="#a6ffff"/>
      </g>;
    })}
  </svg>;
};
