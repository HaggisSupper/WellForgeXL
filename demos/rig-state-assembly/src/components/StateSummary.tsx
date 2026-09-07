import type { OperatorViewModel } from "../ui/operator-view-model.ts";
import { formatTelemetryAge } from "../ui/operator-format.ts";
import { AssemblyTrace } from "./AssemblyTrace.tsx";
import { ExportToolbar } from "./ExportToolbar.tsx";
import { LiveTelemetryStream } from "./LiveTelemetryStream.tsx";
import { TransitionCountdown } from "./TransitionCountdown.tsx";
import type { OperatorScenario } from "../preview/operator-scenarios.ts";

interface StateSummaryProps {
  view: OperatorViewModel;
  scenario: OperatorScenario;
  glyphSvg: string;
  hoveredLayerId: string | null;
  onHoverLayer: (layerId: string | null) => void;
}

export function StateSummary({ view, scenario, glyphSvg, hoveredLayerId, onHoverLayer }: StateSummaryProps) {
  const isPending = view.status.kind === "pending";

  return (
    <div className="state-summary">
      <div className="state-summary-title">
        <p className="section-label">Displayed activity</p>
        <h2>{view.activityLabel}</h2>
      </div>
      <p className="state-reason">{view.status.reason}</p>

      {isPending && <TransitionCountdown initialSeconds={19} reason={view.status.reason} />}

      <dl className="telemetry-facts">
        <div>
          <dt>State contract</dt>
          <dd>{view.stateLabel}</dd>
        </div>
        <div>
          <dt>Confidence</dt>
          <dd className="tabular-num">{view.confidenceLabel}</dd>
        </div>
        <div>
          <dt>Newest signal</dt>
          <dd className="tabular-num">{formatTelemetryAge(view.status.telemetryAgeSeconds)}</dd>
        </div>
      </dl>

      <LiveTelemetryStream scenarioKey={scenario.key} />

      <ExportToolbar scenario={scenario} viewModel={view} glyphSvg={glyphSvg} />

      <AssemblyTrace
        assemblyTrace={view.assemblyTrace}
        hoveredLayerId={hoveredLayerId}
        onHoverLayer={onHoverLayer}
      />
    </div>
  );
}
