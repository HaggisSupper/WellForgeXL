import { useState } from "react";
import { operatorScenarios } from "../preview/operator-scenarios.ts";
import { assembleRigState } from "../contracts/rig-state.ts";
import { renderRigStateSvg } from "../glyph/render-svg.ts";

interface StateComparisonProps {
  onClose: () => void;
}

export function StateComparison({ onClose }: StateComparisonProps) {
  const [leftKey, setLeftKey] = useState("rotate-on-bottom");
  const [rightKey, setRightKey] = useState("flow-check");

  const leftScenario = operatorScenarios.find((s) => s.key === leftKey) ?? operatorScenarios[0];
  const rightScenario = operatorScenarios.find((s) => s.key === rightKey) ?? operatorScenarios[1];

  const leftAssembly = assembleRigState(leftScenario.input);
  const rightAssembly = assembleRigState(rightScenario.input);

  const leftSvg = renderRigStateSvg(leftScenario.input, { instanceId: "compare-left" });
  const rightSvg = renderRigStateSvg(rightScenario.input, { instanceId: "compare-right" });

  const leftLayerIds = new Set(leftAssembly.layers.map((l) => l.id));
  const rightLayerIds = new Set(rightAssembly.layers.map((l) => l.id));

  const allLayers = Array.from(new Set([...leftAssembly.layers, ...rightAssembly.layers]));

  return (
    <div className="palette-backdrop" onClick={onClose}>
      <div className="report-modal" style={{ maxWidth: "980px" }} onClick={(e) => e.stopPropagation()}>
        <div className="report-modal-header">
          <div>
            <span className="section-label">State Transition Inspector</span>
            <h2>Side-by-Side Glyph Comparison</h2>
          </div>
          <button type="button" className="icon-btn" aria-label="Close comparison" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="report-body">
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "2rem" }}>
            {/* Left Glyph Selector & Stage */}
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", background: "var(--canvas)", padding: "1rem", borderRadius: "var(--radius-md)", border: "1px solid var(--line-soft)" }}>
              <select
                aria-label="Left scenario"
                className="scenario-button"
                style={{ width: "100%", marginBottom: "1rem" }}
                value={leftKey}
                onChange={(e) => setLeftKey(e.target.value)}
              >
                {operatorScenarios.map((s) => (
                  <option key={s.key} value={s.key} style={{ background: "var(--canvas-deep)" }}>
                    {s.label}
                  </option>
                ))}
              </select>

              <div className="glyph-frame" style={{ minHeight: "280px" }} dangerouslySetInnerHTML={{ __html: leftSvg }} />

              <div style={{ width: "100%", marginTop: "0.8rem", textAlign: "center" }}>
                <strong>{leftAssembly.state.label}</strong>
                <p style={{ margin: "0.2rem 0 0", fontSize: "0.8rem", color: "var(--muted)" }}>{leftAssembly.activity.label}</p>
              </div>
            </div>

            {/* Right Glyph Selector & Stage */}
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", background: "var(--canvas)", padding: "1rem", borderRadius: "var(--radius-md)", border: "1px solid var(--line-soft)" }}>
              <select
                aria-label="Right scenario"
                className="scenario-button"
                style={{ width: "100%", marginBottom: "1rem" }}
                value={rightKey}
                onChange={(e) => setRightKey(e.target.value)}
              >
                {operatorScenarios.map((s) => (
                  <option key={s.key} value={s.key} style={{ background: "var(--canvas-deep)" }}>
                    {s.label}
                  </option>
                ))}
              </select>

              <div className="glyph-frame" style={{ minHeight: "280px" }} dangerouslySetInnerHTML={{ __html: rightSvg }} />

              <div style={{ width: "100%", marginTop: "0.8rem", textAlign: "center" }}>
                <strong>{rightAssembly.state.label}</strong>
                <p style={{ margin: "0.2rem 0 0", fontSize: "0.8rem", color: "var(--muted)" }}>{rightAssembly.activity.label}</p>
              </div>
            </div>
          </div>

          {/* Layer Difference Table */}
          <div style={{ marginTop: "1.8rem" }}>
            <span className="section-label">Layer Composition Delta Matrix</span>
            <table className="report-table" style={{ marginTop: "0.6rem" }}>
              <thead>
                <tr>
                  <th>Layer Primitive</th>
                  <th>Slot</th>
                  <th style={{ textAlign: "center" }}>{leftAssembly.state.label}</th>
                  <th style={{ textAlign: "center" }}>{rightAssembly.state.label}</th>
                  <th style={{ textAlign: "center" }}>Delta State</th>
                </tr>
              </thead>
              <tbody>
                {allLayers.map((layer) => {
                  const inLeft = leftLayerIds.has(layer.id);
                  const inRight = rightLayerIds.has(layer.id);

                  let delta = "Unchanged";
                  let deltaColor = "var(--muted)";
                  if (inLeft && !inRight) {
                    delta = "Removed in Right";
                    deltaColor = "var(--warning)";
                  } else if (!inLeft && inRight) {
                    delta = "Added in Right";
                    deltaColor = "var(--accent)";
                  }

                  return (
                    <tr key={layer.id}>
                      <td>
                        <strong>{layer.label}</strong> <code style={{ fontSize: "0.72rem", color: "var(--faint)" }}>{layer.id}</code>
                      </td>
                      <td style={{ textTransform: "capitalize" }}>{layer.slot}</td>
                      <td style={{ textAlign: "center" }}>{inLeft ? "✓" : "—"}</td>
                      <td style={{ textAlign: "center" }}>{inRight ? "✓" : "—"}</td>
                      <td style={{ textAlign: "center", color: deltaColor, fontWeight: "600" }}>{delta}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
