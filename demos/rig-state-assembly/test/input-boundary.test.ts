import assert from "node:assert/strict";
import test from "node:test";
import { assembleRigState, RIG_STATE_MANIFEST, validateRigStateManifest, type RigStateManifest } from "../src/contracts/rig-state.ts";
import { createRigStateArtifacts } from "../src/contracts/export.ts";
import { renderRigStateSvg } from "../src/glyph/render-svg.ts";
import { operatorScenarios } from "../src/preview/operator-scenarios.ts";

const valid = { stateId: "FLOW_CHECK", confidence: 0.94, telemetryAgeSeconds: 4, evidence: ["Simulated returns"] };

for (const confidence of [NaN, Infinity, -Infinity, -0.01, 1.01]) {
  test(`rejects invalid confidence ${confidence} before assembly/render`, () => {
    assert.throws(() => assembleRigState({ ...valid, confidence }), /confidence/);
    assert.throws(() => renderRigStateSvg({ ...valid, confidence }), /confidence/);
  });
}
for (const telemetryAgeSeconds of [NaN, Infinity, -Infinity, -1]) {
  test(`rejects invalid age ${telemetryAgeSeconds}`, () => {
    assert.throws(() => assembleRigState({ ...valid, telemetryAgeSeconds }), /telemetryAgeSeconds/);
  });
}
test("rejects unknown previous state even when current state is known", () => {
  for (const stateId of ["FLOW_CHECK", "UNKNOWN"]) {
    for (const previousConfirmedStateId of ["MISSING", "UNKNOWN", ""]) {
      assert.throws(() => assembleRigState({ ...valid, stateId, previousConfirmedStateId }), /previousConfirmedStateId/);
    }
  }
});
test("rejects malformed evidence instead of presenting it as engineering evidence", () => {
  for (const evidence of [null, "text", [42]]) {
    assert.throws(() => assembleRigState({ ...valid, evidence } as unknown as typeof valid), /evidence/);
  }
});

const malformed: [string, RigStateManifest][] = [
  ["missing layers", { ...RIG_STATE_MANIFEST, layers: undefined }],
  ["missing activities", { ...RIG_STATE_MANIFEST, activities: undefined }],
  ["missing dysfunctions", { ...RIG_STATE_MANIFEST, dysfunctions: undefined }],
  ["missing states", { ...RIG_STATE_MANIFEST, states: undefined } as unknown as RigStateManifest],
  ["empty states", { ...RIG_STATE_MANIFEST, states: [] }],
  ["empty layers", { ...RIG_STATE_MANIFEST, layers: [] }],
  ["empty activity layers", { ...RIG_STATE_MANIFEST, activities: RIG_STATE_MANIFEST.activities!.map(a => ({ ...a, layerIds: [] })) }],
  ["missing activity layers", { ...RIG_STATE_MANIFEST, activities: [{ id: "FLOW_CHECK", label: "Flow check" }] } as unknown as RigStateManifest],
  ["empty dysfunction layers", { ...RIG_STATE_MANIFEST, dysfunctions: RIG_STATE_MANIFEST.dysfunctions!.map(d => ({ ...d, layerIds: [] })) }],
  ["unknown activity", { ...RIG_STATE_MANIFEST, states: [{ id: "FLOW_CHECK", label: "Flow check", activityId: "MISSING" }] }],
  ["unknown activity layer", { ...RIG_STATE_MANIFEST, activities: RIG_STATE_MANIFEST.activities!.map(a => ({ ...a, layerIds: ["MISSING"] })) }],
  ["unknown dysfunction layer", { ...RIG_STATE_MANIFEST, dysfunctions: RIG_STATE_MANIFEST.dysfunctions!.map(d => ({ ...d, layerIds: ["MISSING"] })) }],
  ["unknown dysfunction", { ...RIG_STATE_MANIFEST, states: [{ id: "FLOW_CHECK", label: "Flow check", activityId: "HOLD_OBSERVE_RETURNS", dysfunctionIds: ["MISSING"] }] }],
];
for (const [name, manifest] of malformed) {
  test(`rejects incomplete manifest: ${name}`, () => {
    assert.ok(validateRigStateManifest(manifest).length > 0);
    assert.throws(() => assembleRigState(valid, manifest), /Invalid rig-state manifest/);
    assert.throws(() => createRigStateArtifacts(manifest), /Invalid rig-state manifest/);
  });
}

test("valid bounds and all five statuses retain their semantics", () => {
  for (const confidence of [0, 1]) {
    assert.equal(assembleRigState({ ...valid, confidence, telemetryAgeSeconds: 0 }).status.kind, "confirmed");
  }
  assert.equal(assembleRigState({ ...valid, telemetryAgeSeconds: 30 }).status.kind, "confirmed");
  assert.equal(assembleRigState({ ...valid, telemetryAgeSeconds: 30.01 }).status.kind, "stale");
  assert.equal(assembleRigState({ ...valid, telemetryAgeSeconds: 31, conflict: true }).status.kind, "conflict");
  assert.equal(assembleRigState({ ...valid, stateId: "UNKNOWN" }).status.kind, "unclassified");
  assert.equal(assembleRigState({ ...valid, stateId: "UNKNOWN", previousConfirmedStateId: "FLOW_CHECK" }).status.kind, "pending");
  assert.equal(assembleRigState({ ...valid, stateId: "UNRECOGNIZED" }).status.kind, "unclassified");
});
test("all 18 states and 17 scenarios assemble and render deterministically", () => {
  assert.equal(RIG_STATE_MANIFEST.states.length, 18);
  assert.equal(operatorScenarios.length, 17);
  for (const input of [...RIG_STATE_MANIFEST.states.map(s => ({ ...valid, stateId: s.id })), ...operatorScenarios.map(s => s.input)]) {
    const assembly = assembleRigState(input);
    assert.ok(assembly.layers.length > 0);
    assert.deepEqual(assembleRigState(input), assembly);
    const svg = renderRigStateSvg(input, { instanceId: "fixture" });
    assert.equal(renderRigStateSvg(input, { instanceId: "fixture" }), svg);
    assert.match(svg, /<title/);
  }
});
