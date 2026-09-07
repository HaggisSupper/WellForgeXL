import assert from "node:assert/strict";
import test from "node:test";

import { formatTelemetryAge, statusPresentation } from "../src/ui/operator-format.ts";

test("telemetry age is expressed compactly for the operator", () => {
  assert.equal(formatTelemetryAge(4), "4 s ago");
  assert.equal(formatTelemetryAge(65), "1 min 5 s ago");
  assert.equal(formatTelemetryAge(NaN), "0 s ago");
  assert.equal(formatTelemetryAge(-10), "0 s ago");
  assert.equal(formatTelemetryAge(Infinity), "0 s ago");
});

test("pending and conflict states use direct operational language", () => {
  assert.deepEqual(statusPresentation("pending"), { label: "Pending", tone: "caution" });
  assert.deepEqual(statusPresentation("conflict"), { label: "Conflict", tone: "critical" });
});
