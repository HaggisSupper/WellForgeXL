import { describe, expect, it } from "vitest";
import { SceneContractError, parseSceneDocumentV1 } from "./scene";

const validScene = {
  schemaVersion: "wellforge.scene/v1",
  sceneId: "survey-scene",
  title: "Survey trajectory",
  coordinateFrame: "north-east-tvd-m",
  bounds: { minimum: { x: 0, y: 0, z: 0 }, maximum: { x: 20, y: 10, z: 100 } },
  provenance: { algorithm: "survey-position-adapter", profileVersion: "v1", backend: "cpu", inputRevision: null, warnings: [] },
  layers: [{ id: "survey-path", name: "Survey path", visibleByDefault: true, selectable: false, color: "#bfc5ca", primitives: [{ kind: "polyline", points: [{ x: 0, y: 0, z: 0 }, { x: 20, y: 10, z: 100 }] }] }],
};

describe("3Dmk scene ingress", () => {
  it("rejects a scene with a non-finite primitive coordinate", () => {
    const malformed = structuredClone(validScene);
    malformed.layers[0].primitives[0].points[1].z = Number.NaN;

    expect(() => parseSceneDocumentV1(malformed)).toThrow(SceneContractError);
  });

  it("retains an approved V1 scene without coercion", () => {
    expect(parseSceneDocumentV1(validScene)).toEqual(validScene);
  });
});
