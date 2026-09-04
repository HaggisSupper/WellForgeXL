export interface ScenePoint {
  x: number;
  y: number;
  z: number;
}

export interface SceneBounds {
  minimum: ScenePoint;
  maximum: ScenePoint;
}

export interface SceneProvenanceV1 {
  algorithm: string;
  profileVersion: string;
  backend: string;
  inputRevision: string | null;
  warnings: string[];
}

export type ScenePrimitiveV1 =
  | { kind: "polyline"; points: ScenePoint[] }
  | { kind: "marker"; id: string; label: string; point: ScenePoint };

export interface SceneLayerV1 {
  id: string;
  name: string;
  visibleByDefault: boolean;
  selectable: boolean;
  color: string;
  primitives: ScenePrimitiveV1[];
}

export interface SceneDocumentV1 {
  schemaVersion: "wellforge.scene/v1";
  sceneId: string;
  title: string;
  coordinateFrame: "north-east-tvd-m";
  layers: SceneLayerV1[];
  bounds: SceneBounds;
  provenance: SceneProvenanceV1;
}

export class SceneContractError extends Error {
  readonly code = "INVALID_SCENE_CONTRACT";

  constructor(message: string) {
    super(message);
    this.name = "SceneContractError";
  }
}

/** Converts untyped IPC data immediately into the immutable 3Dmk V1 shape. */
export function parseSceneDocumentV1(value: unknown): SceneDocumentV1 {
  const scene = record(value, "scene");
  const schemaVersion = text(scene.schemaVersion, "schemaVersion");
  const coordinateFrame = text(scene.coordinateFrame, "coordinateFrame");
  if (schemaVersion !== "wellforge.scene/v1" || coordinateFrame !== "north-east-tvd-m") {
    throw new SceneContractError("3Dmk requires wellforge.scene/v1 in north-east-tvd-m coordinates");
  }

  const parsed: SceneDocumentV1 = {
    schemaVersion,
    sceneId: text(scene.sceneId, "sceneId"),
    title: text(scene.title, "title"),
    coordinateFrame,
    bounds: bounds(scene.bounds),
    provenance: provenance(scene.provenance),
    layers: list(scene.layers, "layers").map(layer),
  };
  if (parsed.layers.length === 0) throw new SceneContractError("3Dmk scene must contain at least one layer");
  return parsed;
}

function layer(value: unknown): SceneLayerV1 {
  const input = record(value, "layer");
  const primitives = list(input.primitives, "layer.primitives").map(primitive);
  if (primitives.length === 0) throw new SceneContractError("3Dmk layer must contain at least one primitive");
  const color = text(input.color, "layer.color");
  if (!/^#[0-9a-fA-F]{6}$/.test(color)) throw new SceneContractError("3Dmk layer color must be a six-digit hexadecimal value");
  return {
    id: text(input.id, "layer.id"),
    name: text(input.name, "layer.name"),
    visibleByDefault: flag(input.visibleByDefault, "layer.visibleByDefault"),
    selectable: flag(input.selectable, "layer.selectable"),
    color,
    primitives,
  };
}

function primitive(value: unknown): ScenePrimitiveV1 {
  const input = record(value, "primitive");
  const kind = text(input.kind, "primitive.kind");
  if (kind === "polyline") {
    const points = list(input.points, "polyline.points").map((point) => scenePoint(point, "polyline.point"));
    if (points.length === 0) throw new SceneContractError("3Dmk polyline must contain at least one point");
    return { kind, points };
  }
  if (kind === "marker") {
    return { kind, id: text(input.id, "marker.id"), label: text(input.label, "marker.label"), point: scenePoint(input.point, "marker.point") };
  }
  throw new SceneContractError("3Dmk primitive kind is not supported by scene/v1");
}

function provenance(value: unknown): SceneProvenanceV1 {
  const input = record(value, "provenance");
  const revision = input.inputRevision;
  if (revision !== null && typeof revision !== "string") throw new SceneContractError("provenance.inputRevision must be string or null");
  return {
    algorithm: text(input.algorithm, "provenance.algorithm"),
    profileVersion: text(input.profileVersion, "provenance.profileVersion"),
    backend: text(input.backend, "provenance.backend"),
    inputRevision: revision,
    warnings: list(input.warnings, "provenance.warnings").map((warning) => text(warning, "provenance.warning")),
  };
}

function bounds(value: unknown): SceneBounds {
  const input = record(value, "bounds");
  return { minimum: scenePoint(input.minimum, "bounds.minimum"), maximum: scenePoint(input.maximum, "bounds.maximum") };
}

function scenePoint(value: unknown, field: string): ScenePoint {
  const input = record(value, field);
  return { x: finite(input.x, `${field}.x`), y: finite(input.y, `${field}.y`), z: finite(input.z, `${field}.z`) };
}

function record(value: unknown, field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new SceneContractError(`${field} must be an object`);
  return value as Record<string, unknown>;
}

function list(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) throw new SceneContractError(`${field} must be an array`);
  return value;
}

function text(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new SceneContractError(`${field} must be a non-empty string`);
  return value;
}

function flag(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") throw new SceneContractError(`${field} must be boolean`);
  return value;
}

function finite(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new SceneContractError(`${field} must be finite`);
  return value;
}
