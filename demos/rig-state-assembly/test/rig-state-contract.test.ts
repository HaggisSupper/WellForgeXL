import assert from "node:assert/strict";
import test from "node:test";

import {
  assembleRigState,
  validateRigStateManifest,
  type RigStateManifest,
} from "../src/contracts/rig-state.ts";

test("the canonical manifest is internally referentially complete", () => {
  const errors = validateRigStateManifest();
  assert.deepEqual(errors, []);
});

test("a flow check uses a dedicated hold-and-observe glyph instead of circulation", () => {
  const glyph = assembleRigState({
    stateId: "FLOW_CHECK",
    confidence: 0.94,
    telemetryAgeSeconds: 4,
    evidence: ["Pumps off", "Return flow observed"],
  });

  assert.equal(glyph.activity.id, "HOLD_OBSERVE_RETURNS");
  assert.equal(glyph.layers.some((layer) => layer.id === "FLOW_LOOP"), false);
});

test("an in-progress classification retains the last confirmed activity without hiding its pending status", () => {
  const glyph = assembleRigState({
    stateId: "UNKNOWN",
    previousConfirmedStateId: "ROTATE_ON_BOTTOM",
    confidence: 0.42,
    telemetryAgeSeconds: 6,
    reason: "Dwell timer: 19 s remaining",
    evidence: ["RPM present", "Block velocity near zero"],
  });

  assert.equal(glyph.activity.id, "ROTATE_ON_BOTTOM");
  assert.equal(glyph.status.kind, "pending");
  assert.equal(glyph.status.reason, "Dwell timer: 19 s remaining");
});

test("severity is resolved once by the canonical assembly for critical and warning states", () => {
  const losses = assembleRigState({
    stateId: "LOSS",
    confidence: 0.89,
    telemetryAgeSeconds: 4,
    evidence: ["Pit volume down"],
  });
  const packOff = assembleRigState({
    stateId: "PACK_OFF",
    confidence: 0.84,
    telemetryAgeSeconds: 3,
    evidence: ["SPP increasing"],
  });

  assert.equal(losses.severity, "critical");
  assert.equal(packOff.severity, "warning");
});

test("the semantic validator detects dangling layers and duplicate state ids", () => {
  const malformed = {
    ...({ states: [{ id: "X", activityId: "MISSING" }, { id: "X", activityId: "MISSING" }] } as RigStateManifest),
  };

  const errors = validateRigStateManifest(malformed);
  assert.ok(errors.some((error) => error.includes("duplicate state id: X")));
  assert.ok(errors.some((error) => error.includes("unknown activity: MISSING")));
});
