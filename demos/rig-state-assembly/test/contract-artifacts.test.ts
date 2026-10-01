import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { RIG_STATE_MANIFEST } from "../src/contracts/rig-state.ts";
import { createRigStateArtifacts, validateRigStateArtifacts } from "../src/contracts/export.ts";

test("generated catalogue and portable composition have one-to-one state coverage", () => {
  const artifacts = createRigStateArtifacts();
  const canonicalIds = RIG_STATE_MANIFEST.states.map((state) => state.id).sort();

  assert.deepEqual(artifacts.catalogue.map((state) => state.id).sort(), canonicalIds);
  assert.deepEqual(artifacts.compositions.map((state) => state.id).sort(), canonicalIds);
  assert.deepEqual(validateRigStateArtifacts(artifacts), []);
});

test("the portable flow check composition emits the observer glyph and no flow-loop primitive", () => {
  const artifacts = createRigStateArtifacts();
  const flowCheck = artifacts.compositions.find((state) => state.id === "FLOW_CHECK")!;

  assert.equal(flowCheck.activityId, "HOLD_OBSERVE_RETURNS");
  assert.ok(flowCheck.layerIds.includes("RETURNS_OBSERVE"));
  assert.equal(flowCheck.layerIds.includes("FLOW_LOOP"), false);
});

test("the contract generator emits a self-describing portable composition document", async () => {
  const files = ["rig-state.manifest.json", "rig-state-catalogue.json", "rig-state-compositions.json", "rig-state-composition.schema.json"];
  const before = await Promise.all(files.map(name => readFile(new URL(`../public/contracts/${name}`, import.meta.url))));
  // A different, still bounded working directory must not redirect generation.
  execFileSync(process.execPath, ["--experimental-strip-types", fileURLToPath(new URL("../scripts/generate-contract-artifacts.ts", import.meta.url))], {
    cwd: fileURLToPath(new URL("./", import.meta.url)), stdio: "pipe",
  });
  const after = await Promise.all(files.map(name => readFile(new URL(`../public/contracts/${name}`, import.meta.url))));
  assert.deepEqual(after, before, "all four generated snapshots must retain byte parity");
  const document = JSON.parse(await readFile(new URL("../public/contracts/rig-state-compositions.json", import.meta.url), "utf8"));

  assert.equal(document.$schema, "./rig-state-composition.schema.json");
  assert.equal(document.version, "0.1.0");
  assert.equal(document.states.length, RIG_STATE_MANIFEST.states.length);
});
