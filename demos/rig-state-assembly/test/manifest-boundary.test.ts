import assert from "node:assert/strict";
import test from "node:test";
import { assembleRigState, RIG_STATE_MANIFEST, validateRigStateManifest, type RigStateManifest } from "../src/contracts/rig-state.ts";
import { createRigStateArtifacts, validateRigStateArtifacts } from "../src/contracts/export.ts";

type EditableManifest = Record<"layers" | "activities" | "dysfunctions" | "states", Record<string, unknown>[]>;
const copyManifest = (): EditableManifest => structuredClone(RIG_STATE_MANIFEST) as unknown as EditableManifest;
const input = { stateId: "LOSS", confidence: 0.94, telemetryAgeSeconds: 4, evidence: ["Simulated loss"] };

function assertRejected(value: EditableManifest, field: RegExp) {
  const manifest = value as unknown as RigStateManifest;
  assert.match(validateRigStateManifest(manifest).join("; "), field);
  assert.throws(() => assembleRigState(input, manifest), /Invalid rig-state manifest/);
  assert.throws(() => createRigStateArtifacts(manifest), /Invalid rig-state manifest/);
  assert.match(validateRigStateArtifacts(createRigStateArtifacts(), manifest).join("; "), field);
}

// Removing the required severity check must never turn malformed LOSS into normal/confirmed.
for (const severity of [undefined, null, "urgent", "", 0]) {
  test(`T4-01 rejects missing/invalid severity: ${String(severity)}`, () => {
    const manifest = copyManifest();
    manifest.dysfunctions.find(d => d.id === "LOSS")!.severity = severity;
    assertRejected(manifest, /severity/);
  });
}
for (const collection of ["layers", "activities", "dysfunctions", "states"] as const) {
  test(`T4-01 rejects missing/invalid ${collection} labels`, () => {
    for (const label of [undefined, null, "", "   ", 7, {}]) {
      const manifest = copyManifest();
      manifest[collection][0].label = label;
      assertRejected(manifest, /label/);
    }
  });
}
test("T4-01 rejects missing/invalid required layer slots", () => {
  for (const slot of [undefined, null, "", "unknown", 7]) {
    const manifest = copyManifest();
    manifest.layers[0].slot = slot;
    assertRejected(manifest, /slot/);
  }
});
test("T4-01 required activity references reject missing or invalid values", () => {
  for (const activityId of [undefined, null, "", 7, {}]) {
    const manifest = copyManifest();
    manifest.states[0].activityId = activityId;
    assertRejected(manifest, /activity/);
  }
});

const duplicateCases: [string, (manifest: EditableManifest) => void][] = [
  ["activity layers", m => { m.activities[0].layerIds = ["PIPE", "BIT", "ROTATION", "PIPE"]; }],
  ["dysfunction layers", m => { m.dysfunctions.find(d => d.id === "LOSS")!.layerIds = ["LOSS", "LOSS"]; }],
  ["state dysfunction references", m => { m.states.find(s => s.id === "LOSS")!.dysfunctionIds = ["LOSS", "LOSS"]; }],
  ["activity/dysfunction shared layer", m => { m.dysfunctions.find(d => d.id === "LOSS")!.layerIds = ["PIPE"]; }],
  ["two dysfunctions sharing a layer", m => {
    m.states.find(s => s.id === "LOSS")!.dysfunctionIds = ["LOSS", "INFLUX"];
    m.dysfunctions.find(d => d.id === "INFLUX")!.layerIds = ["LOSS"];
  }],
];
for (const [name, mutate] of duplicateCases) {
  test(`T4-02 rejects duplicate ${name} consistently`, () => {
    const manifest = copyManifest();
    mutate(manifest);
    assertRejected(manifest, /duplicate/);
  });
}
test("T4-02 rejects schema-invalid IDs even when references agree", () => {
  for (const id of ["lowercase", "BAD-ID", " PADDED "]) {
    const manifest: EditableManifest = {
      layers: [{ id, label: "Custom layer", slot: "structure" }],
      activities: [{ id, label: "Custom activity", layerIds: [id] }],
      dysfunctions: [],
      states: [{ id, label: "Custom state", activityId: id }],
    };
    assertRejected(manifest, /id/);
  }
});
test("T4-02 artifact validation rejects duplicate lists in independently supplied artifacts", () => {
  for (const field of ["layerIds", "dysfunctionIds"] as const) {
    const artifacts = createRigStateArtifacts();
    const composition = artifacts.compositions.find(c => c.id === "LOSS")!;
    composition[field] = [...composition[field], composition[field][0]];
    assert.match(validateRigStateArtifacts(artifacts).join("; "), /duplicate/);
  }
  const artifacts = createRigStateArtifacts();
  artifacts.catalogue.find(c => c.id === "LOSS")!.dysfunctionIds = ["LOSS", "LOSS"];
  assert.match(validateRigStateArtifacts(artifacts).join("; "), /duplicate/);
});

test("valid explicit severities and contributor order are preserved without mutating inputs", () => {
  for (const severity of ["normal", "advisory", "warning", "critical"] as const) {
    const manifest: RigStateManifest = {
      layers: [{ id: "BASE", label: "Base", slot: "structure" }, { id: "MARK", label: "Mark", slot: "dysfunction" }],
      activities: [{ id: "CUSTOM", label: "Custom activity", layerIds: ["BASE"] }],
      dysfunctions: [{ id: "CUSTOM_D", label: "Custom dysfunction", severity, layerIds: ["MARK"] }],
      states: [{ id: "CUSTOM", label: "Custom state", activityId: "CUSTOM", dysfunctionIds: ["CUSTOM_D"] }],
    };
    const before = structuredClone(manifest);
    const assembly = assembleRigState({ ...input, stateId: "CUSTOM" }, manifest);
    assert.equal(assembly.severity, severity);
    assert.equal(assembly.status.kind, "confirmed");
    assert.deepEqual(assembly.layers.map(l => l.id), ["BASE", "MARK"]);
    const artifacts = createRigStateArtifacts(manifest);
    assert.deepEqual(artifacts.compositions[0], { id: "CUSTOM", activityId: "CUSTOM", layerIds: ["BASE", "MARK"], dysfunctionIds: ["CUSTOM_D"] });
    assert.deepEqual(validateRigStateArtifacts(artifacts, manifest), []);
    assert.deepEqual(manifest, before);
  }
});
