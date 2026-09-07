import assert from "node:assert/strict";
import test from "node:test";

import { getScenario } from "../src/preview/operator-scenarios.ts";
import { createOperatorViewModel } from "../src/ui/operator-view-model.ts";

test("the pending view model retains confirmed activity while surfacing dwell rationale", () => {
  const view = createOperatorViewModel(getScenario("pending-slide"));

  assert.equal(view.activityLabel, "Rotating on bottom");
  assert.equal(view.status.kind, "pending");
  assert.match(view.status.reason, /Dwell timer/);
  assert.equal(view.confidenceLabel, "42%");
});

test("the loss view model carries critical severity, evidence, and assembly trace", () => {
  const view = createOperatorViewModel(getScenario("losses"));

  assert.equal(view.severity, "critical");
  assert.ok(view.evidence.includes("Pit volume −8 bbl"));
  assert.ok(view.assemblyTrace.some((layer) => layer.id === "LOSS"));
});
