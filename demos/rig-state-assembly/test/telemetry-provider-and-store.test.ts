import assert from "node:assert/strict";
import test from "node:test";

import { MockScenarioTelemetryProvider, WebSocketTelemetryProvider } from "../src/telemetry/telemetry-provider.ts";

test("MockScenarioTelemetryProvider emits updates to subscribers", () => {
  const provider = new MockScenarioTelemetryProvider();
  assert.equal(provider.getStatus(), "connected");

  let receivedState = "";
  const unsubscribe = provider.subscribe((input) => {
    receivedState = input.stateId;
  });

  assert.equal(receivedState, "ROTATE_ON_BOTTOM");

  provider.selectScenario("flow-check");
  assert.equal(receivedState, "FLOW_CHECK");

  unsubscribe();
});

test("WebSocketTelemetryProvider instantiates with default URL and initial disconnected state", () => {
  const wsProvider = new WebSocketTelemetryProvider();
  assert.equal(wsProvider.getStatus(), "disconnected");
  assert.equal(wsProvider.name, "WITSML / ETP Stream");
});
