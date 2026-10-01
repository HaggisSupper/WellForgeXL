import { useState } from "react";
import type { OperatorScenario } from "../preview/operator-scenarios.ts";
import type { OperatorViewModel } from "../ui/operator-view-model.ts";
import { ReportModal } from "./ReportModal.tsx";

interface ExportToolbarProps {
  scenario: OperatorScenario;
  viewModel: OperatorViewModel;
  glyphSvg: string;
}

export function ExportToolbar({ scenario, viewModel, glyphSvg }: ExportToolbarProps) {
  const [copied, setCopied] = useState(false);
  const [isReportOpen, setIsReportOpen] = useState(false);

  const downloadFile = (content: string, filename: string, type: string) => {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    // Let the browser start consuming the download before releasing the URL.
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };

  const handleDownloadSvg = () => {
    const filename = `wellforge-glyph-${scenario.key}.svg`;
    downloadFile(glyphSvg, filename, "image/svg+xml");
  };

  const handleDownloadJson = () => {
    const payload = JSON.stringify(
      {
        dataSource: "simulated-scenario",
        authority: "Display demo only; no telemetry inference or alarm authority",
        scenarioKey: scenario.key,
        scenarioLabel: scenario.label,
        input: scenario.input,
        assembledViewModel: viewModel,
        timestamp: new Date().toISOString(),
      },
      null,
      2
    );
    const filename = `wellforge-telemetry-${scenario.key}.json`;

    downloadFile(payload, filename, "application/json");
  };

  const handleCopyEvidence = async () => {
    const text = `Simulated scenario — display demo only\nState: ${viewModel.stateLabel}\nActivity: ${viewModel.activityLabel}\nConfidence: ${viewModel.confidenceLabel}\nEvidence:\n${viewModel.evidence.map((e) => `- ${e}`).join("\n")}`;
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <>
      <div className="export-toolbar" aria-label="Export options">
        <button
          type="button"
          className="action-button"
          onClick={() => setIsReportOpen(true)}
          title="Generate Printable Passdown Report"
          style={{ borderColor: "var(--accent)", color: "var(--accent)" }}
        >
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
          Passdown Report
        </button>

        <button
          type="button"
          className="action-button"
          onClick={handleDownloadSvg}
          title="Download SVG Glyph in Browser"
        >
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
          </svg>
          Download SVG
        </button>

        <button
          type="button"
          className="action-button"
          onClick={handleDownloadJson}
          title="Download Simulated Scenario JSON in Browser"
        >
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          Export JSON
        </button>

        <button
          type="button"
          className="action-button"
          onClick={handleCopyEvidence}
          title="Copy Evidence to Clipboard"
        >
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
          {copied ? "Copied!" : "Copy Evidence"}
        </button>
      </div>

      <ReportModal
        isOpen={isReportOpen}
        onClose={() => setIsReportOpen(false)}
        scenario={scenario}
        viewModel={viewModel}
      />
    </>
  );
}
