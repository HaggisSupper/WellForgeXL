import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("the portable composition schema declares its own schema field and strict state record", async () => {
  const schema = JSON.parse(await readFile(new URL("../contracts/rig-state-composition.schema.json", import.meta.url), "utf8"));

  assert.ok(schema.properties.$schema);
  assert.equal(schema.additionalProperties, false);
  assert.deepEqual(schema.required, ["$schema", "version", "states"]);
  assert.equal(schema.properties.states.items.required.includes("id"), true);
  assert.equal(schema.properties.states.items.required.includes("activityId"), true);
  assert.equal(schema.properties.states.items.required.includes("layerIds"), true);
});
