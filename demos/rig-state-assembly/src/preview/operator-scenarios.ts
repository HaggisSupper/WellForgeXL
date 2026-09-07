import type { GlyphStatusKind, RigStateAssemblyInput } from "../contracts/rig-state.ts";

export interface OperatorScenario {
  key: string;
  label: string;
  shortLabel: string;
  expectedStatus: GlyphStatusKind;
  input: RigStateAssemblyInput;
}

export const operatorScenarios: readonly OperatorScenario[] = [
  {
    key: "rotate-on-bottom",
    label: "Drilling — rotating on bottom",
    shortLabel: "Rotating",
    expectedStatus: "confirmed",
    input: {
      stateId: "ROTATE_ON_BOTTOM",
      confidence: 0.98,
      telemetryAgeSeconds: 2,
      evidence: ["RPM 120", "WOB 28 klbf", "Block velocity 0 ft/min"],
    },
  },
  {
    key: "flow-check",
    label: "Flow check — hold and observe returns",
    shortLabel: "Flow check",
    expectedStatus: "confirmed",
    input: {
      stateId: "FLOW_CHECK",
      confidence: 0.94,
      telemetryAgeSeconds: 4,
      evidence: ["Pumps off", "Return flow observed", "Block velocity 0 ft/min"],
    },
  },
  {
    key: "pending-slide",
    label: "Transition — retain rotation until slide dwell completes",
    shortLabel: "Pending",
    expectedStatus: "pending",
    input: {
      stateId: "UNKNOWN",
      previousConfirmedStateId: "ROTATE_ON_BOTTOM",
      confidence: 0.42,
      telemetryAgeSeconds: 6,
      reason: "Dwell timer: 19 s remaining",
      evidence: ["RPM decaying", "Block velocity near zero", "Toolface settling"],
    },
  },
  {
    key: "stale-telemetry",
    label: "Telemetry stale — retain confirmed state with liveness warning",
    shortLabel: "Stale",
    expectedStatus: "stale",
    input: {
      stateId: "CIRCULATE_OFF_BOTTOM",
      confidence: 0.91,
      telemetryAgeSeconds: 47,
      reason: "SPP update overdue by 17 s",
      evidence: ["Last SPP 2,840 psi", "Last flow 540 gpm"],
    },
  },
  {
    key: "sensor-conflict",
    label: "Telemetry conflict — show evidence, do not assert a new state",
    shortLabel: "Conflict",
    expectedStatus: "conflict",
    input: {
      stateId: "ROTATE_ON_BOTTOM",
      confidence: 0.58,
      telemetryAgeSeconds: 3,
      conflict: true,
      reason: "Top-drive RPM conflicts with rotary torque trend",
      evidence: ["RPM 75", "Torque 0 klbf·ft", "Flow 530 gpm"],
    },
  },
  {
    key: "losses",
    label: "Critical — losses while circulating",
    shortLabel: "Losses",
    expectedStatus: "confirmed",
    input: {
      stateId: "LOSS",
      confidence: 0.89,
      telemetryAgeSeconds: 4,
      evidence: ["Pit volume −8 bbl", "Flow-out 92% of flow-in", "SPP stable"],
    },
  },
  {
    key: "influx",
    label: "Critical — influx / kick indication",
    shortLabel: "Influx",
    expectedStatus: "confirmed",
    input: {
      stateId: "INFLUX",
      confidence: 0.87,
      telemetryAgeSeconds: 3,
      evidence: ["Flow-out exceeds flow-in", "Pit volume +6 bbl", "Pumps on"],
    },
  },
  {
    key: "well-control",
    label: "Critical — well-control response",
    shortLabel: "Well control",
    expectedStatus: "confirmed",
    input: {
      stateId: "WELL_CONTROL",
      confidence: 0.96,
      telemetryAgeSeconds: 2,
      evidence: ["BOP command confirmed", "Pumps off", "Choke position 23%"],
    },
  },
  {
    key: "coring",
    label: "Coring — core-barrel assembly",
    shortLabel: "Coring",
    expectedStatus: "confirmed",
    input: {
      stateId: "CORING",
      confidence: 0.93,
      telemetryAgeSeconds: 4,
      evidence: ["WOB 12 klbf", "RPM 60", "Core barrel configured"],
    },
  },
  {
    key: "pack-off",
    label: "Warning — pack-off restriction",
    shortLabel: "Pack-off",
    expectedStatus: "confirmed",
    input: {
      stateId: "PACK_OFF",
      confidence: 0.86,
      telemetryAgeSeconds: 3,
      evidence: ["SPP spiking +350 psi", "Flow-out steady", "Torque erratic"],
    },
  },
  {
    key: "washout",
    label: "Warning — drillpipe washout leak",
    shortLabel: "Washout",
    expectedStatus: "confirmed",
    input: {
      stateId: "WASHOUT",
      confidence: 0.82,
      telemetryAgeSeconds: 5,
      evidence: ["SPP dropping -220 psi", "Pumps flow rate constant", "No pit volume loss"],
    },
  },
  {
    key: "whirl",
    label: "Advisory — BHA whirl vibration detected",
    shortLabel: "Whirl",
    expectedStatus: "confirmed",
    input: {
      stateId: "WHIRL",
      confidence: 0.79,
      telemetryAgeSeconds: 2,
      evidence: ["Lateral vibration > 4.2g", "High frequency torque ripple"],
    },
  },
  {
    key: "trip-in",
    label: "Tripping — running drillpipe in hole",
    shortLabel: "Trip in",
    expectedStatus: "confirmed",
    input: {
      stateId: "TRIP_IN",
      confidence: 0.95,
      telemetryAgeSeconds: 3,
      evidence: ["Moving down 45 ft/min", "Pumps off", "Hookload 140 klbf"],
    },
  },
  {
    key: "trip-out",
    label: "Tripping — pulling drillpipe out of hole",
    shortLabel: "Trip out",
    expectedStatus: "confirmed",
    input: {
      stateId: "TRIP_OUT",
      confidence: 0.95,
      telemetryAgeSeconds: 3,
      evidence: ["Moving up 52 ft/min", "Pumps off", "Hookload 195 klbf"],
    },
  },
  {
    key: "reaming",
    label: "Reaming — downward reaming hole restriction",
    shortLabel: "Ream",
    expectedStatus: "confirmed",
    input: {
      stateId: "REAM",
      confidence: 0.91,
      telemetryAgeSeconds: 4,
      evidence: ["RPM 80", "Moving down 12 ft/min", "SPP 2,200 psi", "Reamer cutter engaged"],
    },
  },
  {
    key: "backreaming",
    label: "Backreaming — upward reaming tight spot",
    shortLabel: "Backream",
    expectedStatus: "confirmed",
    input: {
      stateId: "BACKREAM",
      confidence: 0.89,
      telemetryAgeSeconds: 4,
      evidence: ["RPM 70", "Moving up 8 ft/min", "Hookload 210 klbf", "Overpull present"],
    },
  },
  {
    key: "cement-circ",
    label: "Cementing — circulating cement slurry",
    shortLabel: "Cement",
    expectedStatus: "confirmed",
    input: {
      stateId: "CEMENT_CIRC",
      confidence: 0.97,
      telemetryAgeSeconds: 2,
      evidence: ["Cement head connected", "Slurry density 15.8 ppg", "Return flow confirmed"],
    },
  },
];

export function getScenario(key: string): OperatorScenario {
  const scenario = operatorScenarios.find((candidate) => candidate.key === key);
  if (!scenario) throw new Error(`Unknown operator scenario: ${key}`);
  return scenario;
}
