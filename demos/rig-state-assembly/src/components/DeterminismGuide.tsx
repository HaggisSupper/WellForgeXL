import { useState } from "react";

interface GuideItem {
  id: string;
  category: "deterministic" | "conditional" | "non-deterministic";
  title: string;
  sensors: string[];
  ruleOrBoundary: string;
  mitigation: string;
}

const guideItems: GuideItem[] = [
  {
    id: "rotate-drill",
    category: "deterministic",
    title: "Rotate on Bottom (Rotary Drilling)",
    sensors: ["WOB", "RPM", "SPP", "Bit Depth"],
    ruleOrBoundary: "WOB > 5 klbf AND RPM > 10 AND SPP > 500 psi AND Bit Depth == Hole Depth",
    mitigation: "Illustrative source proposal only; automatic classification and confidence calibration are not implemented in this demo.",
  },
  {
    id: "flow-check-det",
    category: "deterministic",
    title: "Flow Check (Hold & Observe Returns)",
    sensors: ["SPP", "Block Velocity", "Return Line Sensor"],
    ruleOrBoundary: "SPP < 100 psi AND Pumps == OFF AND Block Velocity == 0",
    mitigation: "Emits RETURNS_OBSERVE primitive without circulation arrows to prevent false circulation inference.",
  },
  {
    id: "tripping-det",
    category: "deterministic",
    title: "Tripping In / Out",
    sensors: ["Block Velocity", "SPP", "Hookload"],
    ruleOrBoundary: "|Block Velocity| > 15 ft/min AND SPP < 200 psi AND Directional Depth Delta",
    mitigation: "Distinguishes directional pipe movement from stationary circulation.",
  },
  {
    id: "slide-dwell",
    category: "conditional",
    title: "Rotary to Slide Dwell Transition",
    sensors: ["RPM", "Toolface", "Dwell Timer"],
    ruleOrBoundary: "RPM drops to 0, toolface orientation settling (15s - 30s dwell timer)",
    mitigation: "Retains previous confirmed activity while displaying 'pending' badge and countdown timer.",
  },
  {
    id: "stale-telemetry-cond",
    category: "conditional",
    title: "Stale Signal / Overdue Telemetry",
    sensors: ["Telemetry Age (s)", "Sensor Heartbeat"],
    ruleOrBoundary: "Signal age > 30 seconds without channel update",
    mitigation: "Displays 'stale' condition tone without misclassifying active state.",
  },
  {
    id: "casing-vs-drillpipe",
    category: "non-deterministic",
    title: "Casing Running vs. Heavy Drillpipe Tripping",
    sensors: ["Hookload", "Block Velocity"],
    ruleOrBoundary: "Identical surface movement and load drop signatures",
    mitigation: "Requires RFID pipe tally tags or manual IADC DDR Plus™ pipe code selection.",
  },
  {
    id: "washout-vs-nozzle",
    category: "non-deterministic",
    title: "Drillpipe Washout vs. Bit Nozzle Loss",
    sensors: ["SPP", "Flow In"],
    ruleOrBoundary: "Both cause sudden SPP drop (ΔSPP < 0) at constant pump rate",
    mitigation: "Requires downhole PWD pressure pulse decoding or acoustic leak analysis.",
  },
  {
    id: "bha-whirl",
    category: "non-deterministic",
    title: "BHA Lateral Whirl Vibration Modes",
    sensors: ["Surface RPM", "Surface Torque"],
    ruleOrBoundary: "Vibrations dampened by drillstring length before reaching surface sensors",
    mitigation: "Requires MWD lateral accelerometer and PWD high-frequency burst transmission.",
  },
  {
    id: "kick-fluid-type",
    category: "non-deterministic",
    title: "Kick Influx Phase (Gas vs. Oil vs. Water)",
    sensors: ["Pit Volume Gain", "Flow Out"],
    ruleOrBoundary: "Surface flow out confirms volume influx, but cannot determine fluid density",
    mitigation: "Requires downhole mud gas chromatograph or surface pit sample analysis.",
  },
];

interface DeterminismGuideProps {
  onClose: () => void;
}

export function DeterminismGuide({ onClose }: DeterminismGuideProps) {
  const [filter, setFilter] = useState<"all" | "deterministic" | "conditional" | "non-deterministic">("all");

  const filteredItems = guideItems.filter((item) => filter === "all" || item.category === filter);

  return (
    <div className="palette-backdrop" onClick={onClose}>
      <div className="report-modal" style={{ maxWidth: "920px" }} onClick={(e) => e.stopPropagation()}>
        <div className="report-modal-header">
          <div>
            <span className="section-label">Unvalidated Design Reference</span>
            <h2>Sensor Determinism & Rig State Boundary Guide</h2>
          </div>
          <button type="button" className="icon-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="report-body">
          <p>Illustrative source proposals, not validated operating thresholds or an implemented classifier. This demo accepts supplied state IDs. Numerical Core remains the single numerical authority.</p>
          {/* Category Filter Tabs */}
          <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1.2rem" }}>
            {(["all", "deterministic", "conditional", "non-deterministic"] as const).map((cat) => (
              <button
                key={cat}
                type="button"
                className={`action-button ${filter === cat ? "is-active" : ""}`}
                style={{
                  textTransform: "capitalize",
                  borderColor: filter === cat ? "var(--accent)" : "var(--line)",
                  color: filter === cat ? "var(--accent)" : "var(--muted)",
                }}
                onClick={() => setFilter(cat)}
              >
                {cat === "all" ? "All Categories" : cat}
              </button>
            ))}
          </div>

          <div style={{ display: "grid", gap: "1rem" }}>
            {filteredItems.map((item) => (
              <div
                key={item.id}
                style={{
                  padding: "1rem 1.2rem",
                  borderRadius: "var(--radius-md)",
                  background: "var(--canvas)",
                  border: "1px solid var(--line-soft)",
                  borderLeft: `4px solid ${
                    item.category === "deterministic"
                      ? "var(--accent)"
                      : item.category === "conditional"
                      ? "var(--warning)"
                      : "var(--critical)"
                  }`,
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <h3 style={{ margin: 0, fontSize: "1rem" }}>{item.title}</h3>
                  <span
                    className="hotkey-badge"
                    style={{
                      textTransform: "uppercase",
                      color:
                        item.category === "deterministic"
                          ? "var(--accent)"
                          : item.category === "conditional"
                          ? "var(--warning)"
                          : "var(--critical)",
                    }}
                  >
                    {item.category}
                  </span>
                </div>

                <div style={{ margin: "0.6rem 0 0", fontSize: "0.83rem", color: "var(--muted)" }}>
                  <p style={{ margin: "0.2rem 0" }}>
                    <strong>Sensors Involved:</strong> {item.sensors.join(", ")}
                  </p>
                  <p style={{ margin: "0.2rem 0", fontFamily: "var(--font-mono)", color: "var(--ink)", fontSize: "0.78rem" }}>
                    <strong>Criteria / Signature:</strong> {item.ruleOrBoundary}
                  </p>
                  <p style={{ margin: "0.4rem 0 0", color: "var(--faint)", fontSize: "0.8rem" }}>
                    <strong>Engineering Solution:</strong> {item.mitigation}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
