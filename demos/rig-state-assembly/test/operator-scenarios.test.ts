import assert from "node:assert/strict";
import test from "node:test";

import { getScenario, operatorScenarios } from "../src/preview/operator-scenarios.ts";

test("operator scenarios expose an evidence-backed confirmed flow check", () => {
  const scenario = getScenario("flow-check");

  assert.equal(scenario.input.stateId, "FLOW_CHECK");
  assert.ok(scenario.input.evidence.includes("Pumps off"));
  assert.equal(scenario.expectedStatus, "confirmed");
});

test("the transition scenario declares a retained state and a pending reason", () => {
  const scenario = getScenario("pending-slide");

  assert.equal(scenario.input.stateId, "UNKNOWN");
  assert.equal(scenario.input.previousConfirmedStateId, "ROTATE_ON_BOTTOM");
  assert.match(scenario.input.reason ?? "", /Dwell timer/);
  assert.equal(scenario.expectedStatus, "pending");
});

test("every demonstration scenario has a unique key and an operator-facing label", () => {
  const keys = operatorScenarios.map((scenario) => scenario.key);
  assert.equal(new Set(keys).size, keys.length);
  assert.equal(operatorScenarios.length, 17);
  assert.ok(operatorScenarios.every((scenario) => scenario.label.length > 0));
});

test("all operator scenarios produce valid non-empty assemblies", () => {
  for (const scenario of operatorScenarios) {
    const sc = getScenario(scenario.key);
    assert.ok(sc.label);
    assert.ok(sc.input.evidence.length > 0);
  }
});
