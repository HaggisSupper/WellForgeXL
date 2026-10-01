import assert from "node:assert/strict";
import test from "node:test";

import { renderRigStateSvg } from "../src/glyph/render-svg.ts";

test("the SVG renderer gives every assembled glyph an accessible name and title", () => {
  const svg = renderRigStateSvg({
    stateId: "ROTATE_ON_BOTTOM",
    confidence: 0.97,
    telemetryAgeSeconds: 2,
    evidence: ["RPM 120", "WOB 28 klbf"],
  });

  assert.match(svg, /role="img"/);
  assert.match(svg, /aria-labelledby="glyph-title-/);
  assert.match(svg, /<title id="glyph-title-/);
  assert.match(svg, /ROTATION/);
});

test("flow check renders observation marks without the circulation loop", () => {
  const svg = renderRigStateSvg({
    stateId: "FLOW_CHECK",
    confidence: 0.91,
    telemetryAgeSeconds: 3,
    evidence: ["Pumps off", "Returns steady"],
  });

  assert.doesNotMatch(svg, /data-layer="FLOW_LOOP"/);
  assert.match(svg, /data-layer="RETURNS_OBSERVE"/);
});

test("critical loss is communicated with a dedicated patterned loss silhouette and severity rail", () => {
  const svg = renderRigStateSvg({
    stateId: "LOSS",
    confidence: 0.89,
    telemetryAgeSeconds: 4,
    evidence: ["Pit volume down", "Flow-out below flow-in"],
  });

  assert.match(svg, /data-severity="critical"/);
  assert.match(svg, /data-layer="LOSS"/);
  assert.match(svg, /id="loss-hatch-/);
  assert.match(svg, /data-severity-rail="critical"/);
  assert.match(svg, /id="arrow-red-/);
});
