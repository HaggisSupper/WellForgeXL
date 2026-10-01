import type { OperatorScenario } from "../preview/operator-scenarios.ts";
import type { OperatorViewModel } from "../ui/operator-view-model.ts";
import { formatTelemetryAge } from "../ui/operator-format.ts";
import { renderRigStateSvg } from "../glyph/render-svg.ts";

interface ReportModalProps {
  isOpen: boolean;
  onClose: () => void;
  scenario: OperatorScenario;
  viewModel: OperatorViewModel;
}

export function ReportModal({ isOpen, onClose, scenario, viewModel }: ReportModalProps) {
  if (!isOpen) return null;
  const glyphSvg = renderRigStateSvg(scenario.input, { instanceId: `report-${scenario.key}` });

  const handlePrint = () => {
    window.print();
  };

  const reportDate = new Date().toLocaleString();

  return (
    <div className="palette-backdrop passdown-backdrop" onClick={onClose}>
      <div className="report-modal" role="dialog" aria-modal="true" aria-label="Rig State Passdown Report" onClick={(e) => e.stopPropagation()}>
        <div className="report-modal-header">
          <div>
            <span className="section-label">Operator Handover</span>
            <h2>Rig State Passdown Report</h2>
          </div>
          <div className="report-modal-actions">
            <button type="button" className="action-button" onClick={handlePrint}>
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="6 9 6 2 18 2 18 9" />
                <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
                <rect x="6" y="14" width="12" height="8" />
              </svg>
              Print Report
            </button>
            <button type="button" className="icon-btn" aria-label="Close report" onClick={onClose}>
              ✕
            </button>
          </div>
        </div>

        <div className="report-body" id="print-area">
          <div className="report-header-block">
            <div className="report-brand">
              <span className="brand-mark">WF</span>
              <div>
                <strong style={{ fontSize: "1.1rem" }}>WellForge Rig State Demo</strong>
                <p style={{ margin: 0, fontSize: "0.8rem", color: "var(--muted)" }}>Simulated scenario — no live telemetry, inference, or alarm authority.</p>
              </div>
            </div>
            <div className="report-meta tabular-num">
              <p>Generated: <strong>{reportDate}</strong></p>
              <p>Scenario: <strong>{scenario.label}</strong></p>
            </div>
          </div>

          <div className="report-grid">
            <div className="report-glyph-preview">
              <span className="section-label">Assembled State Glyph</span>
              <div className="glyph-frame" dangerouslySetInnerHTML={{ __html: glyphSvg }} />
            </div>

            <div className="report-details">
              <span className="section-label">Supplied Display Inputs</span>
              <table className="report-table">
                <tbody>
                  <tr>
                    <th>State Contract</th>
                    <td>{viewModel.stateLabel}</td>
                  </tr>
                  <tr>
                    <th>Activity</th>
                    <td>{viewModel.activityLabel}</td>
                  </tr>
                  <tr>
                    <th>Confidence</th>
                    <td className="tabular-num">{viewModel.confidenceLabel}</td>
                  </tr>
                  <tr>
                    <th>Severity</th>
                    <td style={{ textTransform: "capitalize", fontWeight: "bold" }}>{viewModel.severity}</td>
                  </tr>
                  <tr>
                    <th>Liveness</th>
                    <td className="tabular-num">{formatTelemetryAge(viewModel.status.telemetryAgeSeconds)}</td>
                  </tr>
                  <tr>
                    <th>Display Status</th>
                    <td>{viewModel.status.kind}</td>
                  </tr>
                  <tr>
                    <th>Status Rationale</th>
                    <td>{viewModel.status.reason}</td>
                  </tr>
                </tbody>
              </table>

              <span className="section-label" style={{ marginTop: "1.2rem", display: "block" }}>Evidence Ledger</span>
              <ul className="evidence-list">
                {viewModel.evidence.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </div>
          </div>

          <div className="report-signature-block">
            <div>
              <p className="section-label">Operator Sign-Off</p>
              <div className="sig-line" />
            </div>
            <div>
              <p className="section-label">Drilling Engineer Review</p>
              <div className="sig-line" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
