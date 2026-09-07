import { assembleRigState, type GlyphLayer, type Severity } from "../contracts/rig-state.ts";
import type { OperatorScenario } from "../preview/operator-scenarios.ts";

export interface OperatorViewModel {
  activityLabel: string;
  stateLabel: string;
  confidenceLabel: string;
  severity: Severity;
  status: {
    kind: ReturnType<typeof assembleRigState>["status"]["kind"];
    reason: string;
    telemetryAgeSeconds: number;
  };
  evidence: readonly string[];
  assemblyTrace: readonly GlyphLayer[];
}

export function createOperatorViewModel(scenario: OperatorScenario): OperatorViewModel {
  const assembly = assembleRigState(scenario.input);
  return {
    activityLabel: assembly.activity.label,
    stateLabel: assembly.state.label,
    confidenceLabel: `${Math.round(assembly.status.confidence * 100)}%`,
    severity: assembly.severity,
    status: {
      kind: assembly.status.kind,
      reason: assembly.status.reason,
      telemetryAgeSeconds: assembly.status.telemetryAgeSeconds,
    },
    evidence: assembly.evidence,
    assemblyTrace: assembly.layers,
  };
}
