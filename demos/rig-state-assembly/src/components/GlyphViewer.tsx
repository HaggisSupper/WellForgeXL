import { useState } from "react";
import type { Severity, GlyphStatusKind } from "../contracts/rig-state.ts";

interface GlyphViewerProps {
  glyphSvg: string;
  severity: Severity;
  statusKind: GlyphStatusKind;
  hoveredLayerId?: string | null;
}

export function GlyphViewer({ glyphSvg, severity, statusKind, hoveredLayerId }: GlyphViewerProps) {
  const [zoom, setZoom] = useState(1);
  const [animationsEnabled, setAnimationsEnabled] = useState(true);

  const handleZoomIn = () => setZoom((prev) => Math.min(prev + 0.15, 1.6));
  const handleZoomOut = () => setZoom((prev) => Math.max(prev - 0.15, 0.75));
  const handleResetZoom = () => setZoom(1);

  return (
    <figure
      className={`glyph-figure severity-${severity} status-${statusKind} ${hoveredLayerId ? "has-hover-highlight" : ""} ${animationsEnabled ? "motion-active" : "motion-paused"}`}
      data-hovered-layer={hoveredLayerId ?? undefined}
    >
      <div className="figure-toolbar">
        <div className="zoom-controls">
          <button type="button" className="icon-btn" onClick={handleZoomOut} title="Zoom out (-)" aria-label="Zoom out">
            −
          </button>
          <span className="zoom-level tabular-num">{Math.round(zoom * 100)}%</span>
          <button type="button" className="icon-btn" onClick={handleZoomIn} title="Zoom in (+)" aria-label="Zoom in">
            +
          </button>
          {zoom !== 1 && (
            <button type="button" className="icon-btn reset-btn" onClick={handleResetZoom} title="Reset zoom">
              Reset
            </button>
          )}
        </div>

        <button
          type="button"
          className={`motion-toggle-btn ${animationsEnabled ? "is-active" : ""}`}
          onClick={() => setAnimationsEnabled(!animationsEnabled)}
          title="Toggle SVG Animations"
        >
          <span className="pulse-indicator" />
          {animationsEnabled ? "Live Motion" : "Static"}
        </button>
      </div>

      <div className="glyph-frame" style={{ transform: `scale(${zoom})`, transition: "transform 180ms ease-out" }}>
        <div dangerouslySetInnerHTML={{ __html: glyphSvg }} />
      </div>

      <figcaption>
        Glyph composition is derived from the canonical state manifest. Critical conditions have a dedicated silhouette and rail.
      </figcaption>
    </figure>
  );
}
