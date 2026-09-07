export type Severity = "normal" | "advisory" | "warning" | "critical";
export type GlyphStatusKind = "confirmed" | "pending" | "stale" | "conflict" | "unclassified";

export type GlyphSlot =
  | "structure"
  | "activity"
  | "fluid"
  | "movement"
  | "observation"
  | "dysfunction";

export interface GlyphLayer {
  id: string;
  slot: GlyphSlot;
  label: string;
  exclusiveSlot?: boolean;
  protectedZones?: readonly string[];
}

export interface ActivityDefinition {
  id: string;
  label: string;
  layerIds: readonly string[];
}

export interface DysfunctionDefinition {
  id: string;
  label: string;
  severity: Severity;
  layerIds: readonly string[];
}

export interface RigStateDefinition {
  id: string;
  label: string;
  activityId: string;
  dysfunctionIds?: readonly string[];
}

export interface RigStateManifest {
  layers?: readonly GlyphLayer[];
  activities?: readonly ActivityDefinition[];
  dysfunctions?: readonly DysfunctionDefinition[];
  states: readonly RigStateDefinition[];
}

export interface RigStateAssemblyInput {
  stateId: string;
  previousConfirmedStateId?: string;
  confidence: number;
  telemetryAgeSeconds: number;
  evidence: readonly string[];
  reason?: string;
  conflict?: boolean;
}

export interface GlyphStatus {
  kind: GlyphStatusKind;
  confidence: number;
  telemetryAgeSeconds: number;
  reason: string;
}

export interface RigStateAssembly {
  state: RigStateDefinition;
  activity: ActivityDefinition;
  layers: readonly GlyphLayer[];
  dysfunctions: readonly DysfunctionDefinition[];
  severity: Severity;
  status: GlyphStatus;
  evidence: readonly string[];
  accessibleLabel: string;
}

const layers: readonly GlyphLayer[] = [
  { id: "PIPE", slot: "structure", label: "Drill pipe", protectedZones: ["pipe"] },
  { id: "BIT", slot: "structure", label: "Bit", protectedZones: ["bit"] },
  { id: "ROTATION", slot: "activity", label: "Rotating arrow", protectedZones: ["pipe"] },
  { id: "SLIDE_KINK", slot: "activity", label: "Slide bend", protectedZones: ["bit"] },
  { id: "FLOW_LOOP", slot: "fluid", label: "Circulation loop", protectedZones: ["annulus"] },
  { id: "RETURNS_OBSERVE", slot: "observation", label: "Observe returns", protectedZones: ["return-line"] },
  { id: "MOVE_DOWN", slot: "movement", label: "Moving down", exclusiveSlot: true, protectedZones: ["pipe"] },
  { id: "MOVE_UP", slot: "movement", label: "Moving up", exclusiveSlot: true, protectedZones: ["pipe"] },
  { id: "REAM", slot: "activity", label: "Reaming cutter", protectedZones: ["bit"] },
  { id: "BACKREAM", slot: "activity", label: "Backreaming cutter", protectedZones: ["bit"] },
  { id: "CORE", slot: "activity", label: "Core barrel", protectedZones: ["bit"] },
  { id: "CEMENT", slot: "fluid", label: "Cement circulation", protectedZones: ["annulus"] },
  { id: "PACK_OFF", slot: "dysfunction", label: "Pack-off restriction", protectedZones: ["annulus"] },
  { id: "WASHOUT", slot: "dysfunction", label: "Washout leak", protectedZones: ["pipe"] },
  { id: "LOSS", slot: "dysfunction", label: "Lost returns", protectedZones: ["annulus"] },
  { id: "INFLUX", slot: "dysfunction", label: "Influx", protectedZones: ["annulus"] },
  { id: "WELL_CONTROL", slot: "dysfunction", label: "Well-control barrier", protectedZones: ["surface"] },
  { id: "WHIRL", slot: "dysfunction", label: "Whirl vibration", protectedZones: ["pipe"] },
];

const activities: readonly ActivityDefinition[] = [
  { id: "ROTATE_ON_BOTTOM", label: "Rotating on bottom", layerIds: ["PIPE", "BIT", "ROTATION"] },
  { id: "SLIDE_ON_BOTTOM", label: "Sliding on bottom", layerIds: ["PIPE", "BIT", "SLIDE_KINK"] },
  { id: "CIRCULATE_OFF_BOTTOM", label: "Circulating off bottom", layerIds: ["PIPE", "BIT", "FLOW_LOOP"] },
  { id: "CIRC_ROTATE_OFF_BOTTOM", label: "Rotating and circulating off bottom", layerIds: ["PIPE", "BIT", "ROTATION", "FLOW_LOOP"] },
  { id: "HOLD_OBSERVE_RETURNS", label: "Holding and observing returns", layerIds: ["PIPE", "BIT", "RETURNS_OBSERVE"] },
  { id: "TRIP_IN", label: "Tripping in", layerIds: ["PIPE", "BIT", "MOVE_DOWN"] },
  { id: "TRIP_OUT", label: "Tripping out", layerIds: ["PIPE", "BIT", "MOVE_UP"] },
  { id: "REAM", label: "Reaming", layerIds: ["PIPE", "BIT", "ROTATION", "MOVE_DOWN", "REAM"] },
  { id: "BACKREAM", label: "Backreaming", layerIds: ["PIPE", "BIT", "ROTATION", "MOVE_UP", "BACKREAM"] },
  { id: "CORING", label: "Coring", layerIds: ["PIPE", "CORE", "MOVE_DOWN"] },
  { id: "CEMENT_CIRC", label: "Cement circulation", layerIds: ["PIPE", "BIT", "CEMENT"] },
  { id: "WELL_CONTROL", label: "Well-control response", layerIds: ["PIPE", "BIT", "WELL_CONTROL"] },
  { id: "UNCLASSIFIED", label: "Unclassified activity", layerIds: ["PIPE", "BIT", "RETURNS_OBSERVE"] },
];

const dysfunctions: readonly DysfunctionDefinition[] = [
  { id: "PACK_OFF", label: "Pack-off", severity: "warning", layerIds: ["PACK_OFF"] },
  { id: "WASHOUT", label: "Washout", severity: "warning", layerIds: ["WASHOUT"] },
  { id: "LOSS", label: "Losses", severity: "critical", layerIds: ["LOSS"] },
  { id: "INFLUX", label: "Influx / kick", severity: "critical", layerIds: ["INFLUX"] },
  { id: "WHIRL", label: "Whirl", severity: "advisory", layerIds: ["WHIRL"] },
];

const states: readonly RigStateDefinition[] = [
  { id: "ROTATE_ON_BOTTOM", label: "Rotate on bottom", activityId: "ROTATE_ON_BOTTOM" },
  { id: "SLIDE_ON_BOTTOM", label: "Slide on bottom", activityId: "SLIDE_ON_BOTTOM" },
  { id: "CIRCULATE_OFF_BOTTOM", label: "Circulate off bottom", activityId: "CIRCULATE_OFF_BOTTOM" },
  { id: "CIRC_ROTATE_OFF_BOTTOM", label: "Circulate and rotate off bottom", activityId: "CIRC_ROTATE_OFF_BOTTOM" },
  { id: "FLOW_CHECK", label: "Flow check", activityId: "HOLD_OBSERVE_RETURNS" },
  { id: "TRIP_IN", label: "Trip in", activityId: "TRIP_IN" },
  { id: "TRIP_OUT", label: "Trip out", activityId: "TRIP_OUT" },
  { id: "REAM", label: "Ream", activityId: "REAM" },
  { id: "BACKREAM", label: "Backream", activityId: "BACKREAM" },
  { id: "CORING", label: "Coring", activityId: "CORING" },
  { id: "CEMENT_CIRC", label: "Cement circulation", activityId: "CEMENT_CIRC" },
  { id: "PACK_OFF", label: "Pack-off", activityId: "CIRCULATE_OFF_BOTTOM", dysfunctionIds: ["PACK_OFF"] },
  { id: "WASHOUT", label: "Washout", activityId: "ROTATE_ON_BOTTOM", dysfunctionIds: ["WASHOUT"] },
  { id: "LOSS", label: "Losses", activityId: "CIRCULATE_OFF_BOTTOM", dysfunctionIds: ["LOSS"] },
  { id: "INFLUX", label: "Influx / kick", activityId: "CIRCULATE_OFF_BOTTOM", dysfunctionIds: ["INFLUX"] },
  { id: "WHIRL", label: "Whirl", activityId: "ROTATE_ON_BOTTOM", dysfunctionIds: ["WHIRL"] },
  { id: "WELL_CONTROL", label: "Well control", activityId: "WELL_CONTROL" },
  { id: "UNKNOWN", label: "Unknown", activityId: "UNCLASSIFIED" },
];

export const RIG_STATE_MANIFEST: RigStateManifest = { layers, activities, dysfunctions, states };

const uniqueIds = (items: readonly { id: string }[], noun: string): string[] => {
  const seen = new Set<string>();
  const errors: string[] = [];
  for (const item of items) {
    if (seen.has(item.id)) errors.push(`duplicate ${noun} id: ${item.id}`);
    seen.add(item.id);
  }
  return errors;
};

export function validateRigStateManifest(manifest: RigStateManifest = RIG_STATE_MANIFEST): string[] {
  if (!manifest || typeof manifest !== "object") return ["manifest must be an object"];
  const collectionErrors: string[] = [];
  for (const name of ["layers", "activities", "dysfunctions", "states"] as const) {
    const items = manifest[name];
    if (!Array.isArray(items)) collectionErrors.push(`manifest ${name} must be an array`);
    else {
      if (name !== "dysfunctions" && !items.length) collectionErrors.push(`manifest ${name} must not be empty`);
      if (items.some(item => !item || typeof item.id !== "string" || !item.id.trim())) {
        return [...collectionErrors, `manifest ${name} contains an invalid id`];
      }
    }
  }
  const activeLayers = Array.isArray(manifest.layers) ? manifest.layers : [];
  const activeActivities = Array.isArray(manifest.activities) ? manifest.activities : [];
  const activeDysfunctions = Array.isArray(manifest.dysfunctions) ? manifest.dysfunctions : [];
  const activeStates = Array.isArray(manifest.states) ? manifest.states : [];
  const errors = [
    ...collectionErrors,
    ...uniqueIds(activeLayers, "layer"),
    ...uniqueIds(activeActivities, "activity"),
    ...uniqueIds(activeDysfunctions, "dysfunction"),
    ...uniqueIds(activeStates, "state"),
  ];
  const layerIds = new Set(activeLayers.map((item) => item.id));
  const activityIds = new Set(activeActivities.map((item) => item.id));
  const dysfunctionIds = new Set(activeDysfunctions.map((item) => item.id));

  for (const activity of activeActivities) {
    if (!Array.isArray(activity.layerIds) || !activity.layerIds.length) {
      errors.push(`activity ${activity.id} must have layers`);
      continue;
    }
    const usedExclusiveSlots = new Set<GlyphSlot>();
    for (const layerId of activity.layerIds) {
      const layer = activeLayers.find((candidate) => candidate.id === layerId);
      if (!layerIds.has(layerId)) errors.push(`activity ${activity.id} references unknown layer: ${layerId}`);
      if (layer?.exclusiveSlot && usedExclusiveSlots.has(layer.slot)) errors.push(`activity ${activity.id} has duplicate slot: ${layer.slot}`);
      if (layer?.exclusiveSlot) usedExclusiveSlots.add(layer.slot);
    }
  }

  for (const dysfunction of activeDysfunctions) {
    if (!Array.isArray(dysfunction.layerIds) || !dysfunction.layerIds.length) {
      errors.push(`dysfunction ${dysfunction.id} must have layers`);
      continue;
    }
    for (const layerId of dysfunction.layerIds) {
      if (!layerIds.has(layerId)) errors.push(`dysfunction ${dysfunction.id} references unknown layer: ${layerId}`);
    }
  }

  for (const state of activeStates) {
    if (!activityIds.has(state.activityId)) errors.push(`state ${state.id} references unknown activity: ${state.activityId}`);
    if (state.dysfunctionIds !== undefined && !Array.isArray(state.dysfunctionIds)) {
      errors.push(`state ${state.id} dysfunctionIds must be an array`);
      continue;
    }
    for (const dysfunctionId of state.dysfunctionIds ?? []) {
      if (!dysfunctionIds.has(dysfunctionId)) errors.push(`state ${state.id} references unknown dysfunction: ${dysfunctionId}`);
    }
  }

  return errors;
}

function findState(id: string, manifest: RigStateManifest = RIG_STATE_MANIFEST): RigStateDefinition {
  const activeStates = manifest.states ?? states;
  return activeStates.find((state) => state.id === id) ?? activeStates.find((state) => state.id === "UNKNOWN") ?? { id: "UNKNOWN", label: "Unknown", activityId: "UNCLASSIFIED" };
}

function statusFor(input: RigStateAssemblyInput, state: RigStateDefinition): GlyphStatus {
  if (input.conflict) {
    return { kind: "conflict", confidence: input.confidence, telemetryAgeSeconds: input.telemetryAgeSeconds, reason: input.reason ?? "Conflicting telemetry" };
  }
  if (input.telemetryAgeSeconds > 30) {
    return { kind: "stale", confidence: input.confidence, telemetryAgeSeconds: input.telemetryAgeSeconds, reason: input.reason ?? "Telemetry is stale" };
  }
  if (state.id === "UNKNOWN" && input.previousConfirmedStateId) {
    return { kind: "pending", confidence: input.confidence, telemetryAgeSeconds: input.telemetryAgeSeconds, reason: input.reason ?? "Classification pending" };
  }
  if (state.id === "UNKNOWN") {
    return { kind: "unclassified", confidence: input.confidence, telemetryAgeSeconds: input.telemetryAgeSeconds, reason: input.reason ?? "No state classification" };
  }
  return { kind: "confirmed", confidence: input.confidence, telemetryAgeSeconds: input.telemetryAgeSeconds, reason: input.reason ?? "Classified" };
}

export function assembleRigState(input: RigStateAssemblyInput, manifest: RigStateManifest = RIG_STATE_MANIFEST): RigStateAssembly {
  const errors = validateRigStateManifest(manifest);
  if (errors.length) throw new Error(`Invalid rig-state manifest: ${errors.join("; ")}`);
  if (!Number.isFinite(input.confidence) || input.confidence < 0 || input.confidence > 1) {
    throw new RangeError("confidence must be finite and between 0 and 1");
  }
  if (!Number.isFinite(input.telemetryAgeSeconds) || input.telemetryAgeSeconds < 0) {
    throw new RangeError("telemetryAgeSeconds must be finite and non-negative");
  }
  if (input.previousConfirmedStateId !== undefined &&
      (input.previousConfirmedStateId === "UNKNOWN" || !manifest.states.some(state => state.id === input.previousConfirmedStateId))) {
    throw new Error("previousConfirmedStateId must reference a known classified state");
  }
  if (!Array.isArray(input.evidence) || input.evidence.some(item => typeof item !== "string")) {
    throw new TypeError("evidence must be an array of strings");
  }
  const activeLayers = manifest.layers ?? layers;
  const activeActivities = manifest.activities ?? activities;
  const activeDysfunctions = manifest.dysfunctions ?? dysfunctions;

  const requestedState = findState(input.stateId, manifest);
  const retainedState = requestedState.id === "UNKNOWN" && input.previousConfirmedStateId
    ? findState(input.previousConfirmedStateId, manifest)
    : requestedState;

  const fallbackActivity: ActivityDefinition = { id: "UNCLASSIFIED", label: "Unclassified activity", layerIds: ["PIPE", "BIT", "RETURNS_OBSERVE"] };
  const activity = activeActivities.find((candidate) => candidate.id === retainedState.activityId) ?? fallbackActivity;
  const stateDysfunctions = (retainedState.dysfunctionIds ?? [])
    .map((id) => activeDysfunctions.find((candidate) => candidate.id === id))
    .filter((candidate): candidate is DysfunctionDefinition => Boolean(candidate));
  const layerIds = [...activity.layerIds, ...stateDysfunctions.flatMap((item) => item.layerIds)];
  const assembledLayers = layerIds
    .map((id) => activeLayers.find((layer) => layer.id === id))
    .filter((layer): layer is GlyphLayer => Boolean(layer));
  const status = statusFor(input, requestedState);
  const severity: Severity = retainedState.id === "WELL_CONTROL" || stateDysfunctions.some((item) => item.severity === "critical")
    ? "critical"
    : stateDysfunctions.some((item) => item.severity === "warning")
      ? "warning"
      : stateDysfunctions.some((item) => item.severity === "advisory")
        ? "advisory"
        : "normal";
  const suffix = status.kind === "confirmed" ? "" : `, ${status.kind}: ${status.reason}`;

  return {
    state: retainedState,
    activity,
    layers: assembledLayers,
    dysfunctions: stateDysfunctions,
    severity,
    status,
    evidence: input.evidence,
    accessibleLabel: `${severity} ${retainedState.label}; ${activity.label}; confidence ${Math.round(input.confidence * 100)} percent${suffix}`,
  };
}
