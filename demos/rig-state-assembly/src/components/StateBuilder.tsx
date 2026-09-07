import { useState } from "react";
import { RIG_STATE_MANIFEST, assembleRigState, type RigStateAssemblyInput } from "../contracts/rig-state.ts";
import { renderRigStateSvg } from "../glyph/render-svg.ts";

interface StateBuilderProps {
  onApplyAssemblyInput: (input: RigStateAssemblyInput) => void;
  onClose: () => void;
}

export function StateBuilder({ onApplyAssemblyInput, onClose }: StateBuilderProps) {
  const [stateId, setStateId] = useState("ROTATE_ON_BOTTOM");
  const [previousStateId, setPreviousStateId] = useState("ROTATE_ON_BOTTOM");
  const [confidence, setConfidence] = useState(0.92);
  const [telemetryAgeSeconds, setTelemetryAgeSeconds] = useState(4);
  const [conflict, setConflict] = useState(false);
  const [reason, setReason] = useState("Custom operator simulation");
  const [evidenceText, setEvidenceText] = useState("RPM 120\nWOB 26 klbf\nSPP 2450 psi");

  const evidence = evidenceText.split("\n").filter((line) => line.trim().length > 0);

  const customInput: RigStateAssemblyInput = {
    stateId,
    previousConfirmedStateId: stateId === "UNKNOWN" && previousStateId ? previousStateId : undefined,
    confidence,
    telemetryAgeSeconds,
    conflict,
    reason,
    evidence,
  };

  const previewAssembly = assembleRigState(customInput);
  const previewSvg = renderRigStateSvg(customInput, { instanceId: "builder-preview" });

  const handleApply = () => {
    onApplyAssemblyInput(customInput);
    onClose();
  };

  return (
    <div className="palette-backdrop" onClick={onClose}>
      <div className="report-modal" style={{ maxWidth: "900px" }} onClick={(e) => e.stopPropagation()}>
        <div className="report-modal-header">
          <div>
            <span className="section-label">Interactive Workbench</span>
            <h2>Custom Telemetry Simulator & State Builder</h2>
          </div>
          <button type="button" className="icon-btn" aria-label="Close builder" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="report-body" style={{ display: "grid", gridTemplateColumns: "340px 1fr", gap: "1.5rem" }}>
          {/* Controls Form */}
          <div className="builder-form" style={{ display: "grid", gap: "1rem" }}>
            <div>
              <label className="section-label">Target Rig State</label>
              <select
                aria-label="Target Rig State"
                className="scenario-button"
                style={{ width: "100%", marginTop: "0.3rem" }}
                value={stateId}
                onChange={(e) => setStateId(e.target.value)}
              >
                {RIG_STATE_MANIFEST.states.map((s) => (
                  <option key={s.id} value={s.id} style={{ background: "var(--canvas-deep)" }}>
                    {s.label} ({s.id})
                  </option>
                ))}
              </select>
            </div>

            {stateId === "UNKNOWN" && (
              <div>
                <label className="section-label">Previous Confirmed State</label>
                <select
                  aria-label="Previous Confirmed State"
                  className="scenario-button"
                  style={{ width: "100%", marginTop: "0.3rem" }}
                  value={previousStateId}
                  onChange={(e) => setPreviousStateId(e.target.value)}
                >
                  <option value="">None — unclassified</option>
                  {RIG_STATE_MANIFEST.states
                    .filter((s) => s.id !== "UNKNOWN")
                    .map((s) => (
                      <option key={s.id} value={s.id} style={{ background: "var(--canvas-deep)" }}>
                        {s.label} ({s.id})
                      </option>
                    ))}
                </select>
              </div>
            )}

            <div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <label className="section-label">Confidence</label>
                <span className="tabular-num" style={{ fontSize: "0.8rem", color: "var(--accent)" }}>
                  {Math.round(confidence * 100)}%
                </span>
              </div>
              <input
                aria-label="Confidence"
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={confidence}
                onChange={(e) => setConfidence(parseFloat(e.target.value))}
                style={{ width: "100%", marginTop: "0.3rem" }}
              />
            </div>

            <div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <label className="section-label">Telemetry Signal Age</label>
                <span className="tabular-num" style={{ fontSize: "0.8rem", color: telemetryAgeSeconds > 30 ? "var(--warning)" : "var(--muted)" }}>
                  {telemetryAgeSeconds}s
                </span>
              </div>
              <input
                aria-label="Telemetry Signal Age"
                type="range"
                min="0"
                max="60"
                step="1"
                value={telemetryAgeSeconds}
                onChange={(e) => setTelemetryAgeSeconds(parseInt(e.target.value, 10))}
                style={{ width: "100%", marginTop: "0.3rem" }}
              />
            </div>

            <div>
              <label className="section-label" style={{ display: "flex", alignItems: "center", gap: "0.5rem", cursor: "pointer" }}>
                <input
                  type="checkbox"
                  checked={conflict}
                  onChange={(e) => setConflict(e.target.checked)}
                />
                Simulate Sensor Conflict
              </label>
            </div>

            <div>
              <label className="section-label">Status Rationale / Reason</label>
              <input
                aria-label="Status Rationale / Reason"
                type="text"
                className="palette-input"
                style={{ border: "1px solid var(--line)", padding: "0.4rem 0.6rem", borderRadius: "6px", width: "100%", marginTop: "0.3rem" }}
                value={reason}
                onChange={(e) => setReason(e.target.value)}
              />
            </div>

            <div>
              <label className="section-label">Evidence Items (one per line)</label>
              <textarea
                aria-label="Evidence Items (one per line)"
                rows={3}
                className="palette-input"
                style={{ border: "1px solid var(--line)", padding: "0.4rem 0.6rem", borderRadius: "6px", width: "100%", marginTop: "0.3rem", resize: "vertical" }}
                value={evidenceText}
                onChange={(e) => setEvidenceText(e.target.value)}
              />
            </div>

            <button type="button" className="action-button" style={{ borderColor: "var(--accent)", color: "var(--accent)", justifySelf: "start" }} onClick={handleApply}>
              Apply Custom State to Dashboard
            </button>
          </div>

          {/* Live Render Preview */}
          <div className="builder-preview" style={{ display: "flex", flexDirection: "column", alignItems: "center", background: "var(--canvas)", padding: "1.2rem", borderRadius: "var(--radius-md)", border: "1px solid var(--line-soft)" }}>
            <span className="section-label">Assembled Assembly Preview</span>
            <div className="glyph-frame" style={{ minHeight: "280px" }} dangerouslySetInnerHTML={{ __html: previewSvg }} />

            <div style={{ width: "100%", marginTop: "1rem", fontSize: "0.82rem", color: "var(--muted)" }}>
              <p><strong>Resolved Severity:</strong> <span style={{ textTransform: "capitalize", color: `var(--${previewAssembly.severity})` }}>{previewAssembly.severity}</span></p>
              <p><strong>Resolved Status:</strong> <span style={{ textTransform: "capitalize" }}>{previewAssembly.status.kind} ({previewAssembly.status.reason})</span></p>
              <p><strong>Assembled Layers ({previewAssembly.layers.length}):</strong></p>
              <div style={{ display: "flex", flexWrap: "wrap", gap: "0.3rem", marginTop: "0.3rem" }}>
                {previewAssembly.layers.map((l) => (
                  <span key={l.id} className="hotkey-badge" style={{ margin: 0 }}>
                    {l.id}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
