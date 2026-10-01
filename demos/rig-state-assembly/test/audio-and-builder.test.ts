import assert from "node:assert/strict";
import test from "node:test";

import { isSoundEnabled } from "../src/utils/audio.ts";
import { assembleRigState } from "../src/contracts/rig-state.ts";

test("audio remains muted by default per design choice", () => {
  assert.equal(isSoundEnabled(), false);
});

test("custom simulation input maps correctly to assembly view model", () => {
  const custom = assembleRigState({
    stateId: "PACK_OFF",
    confidence: 0.88,
    telemetryAgeSeconds: 5,
    evidence: ["SPP spike +400 psi"],
  });

  assert.equal(custom.state.id, "PACK_OFF");
  assert.equal(custom.severity, "warning");
  assert.ok(custom.layers.some((l) => l.id === "PACK_OFF"));
});
