import type { GlyphLayer } from "../contracts/rig-state.ts";

interface AssemblyTraceProps {
  assemblyTrace: readonly GlyphLayer[];
  hoveredLayerId: string | null;
  onHoverLayer: (layerId: string | null) => void;
}

export function AssemblyTrace({ assemblyTrace, hoveredLayerId, onHoverLayer }: AssemblyTraceProps) {
  return (
    <details className="assembly-trace" open>
      <summary>Inspect glyph assembly ({assemblyTrace.length} layers)</summary>
      <ol>
        {assemblyTrace.map((layer) => {
          const isHovered = hoveredLayerId === layer.id;
          return (
            <li
              key={layer.id}
              className={`trace-item ${isHovered ? "is-hovered" : ""}`}
              onMouseEnter={() => onHoverLayer(layer.id)}
              onMouseLeave={() => onHoverLayer(null)}
            >
              <span>{layer.label}</span>
              <code>{layer.id}</code>
            </li>
          );
        })}
      </ol>
    </details>
  );
}
