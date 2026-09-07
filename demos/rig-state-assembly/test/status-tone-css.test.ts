import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { statusPresentation } from "../src/ui/operator-format.ts";

const statusKinds = ["confirmed", "pending", "stale", "conflict", "unclassified"] as const;

function selectorsFromCss(css: string): Set<string> {
  const selectors = new Set<string>();
  const rulePattern = /(^|})\s*([^@{}]+)\{/g;
  for (const match of css.matchAll(rulePattern)) {
    for (const selector of match[2].split(",")) {
      selectors.add(selector.trim());
    }
  }
  return selectors;
}

test("status presentation tones have matching chip and dot CSS consumers", async () => {
  const css = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const selectors = selectorsFromCss(css);

  for (const statusKind of statusKinds) {
    const tone = statusPresentation(statusKind).tone;
    assert.ok(selectors.has(`.tone-${tone}`), `missing chip selector for ${statusKind}/${tone}`);
    assert.ok(selectors.has(`.tone-${tone} .status-dot`), `missing dot selector for ${statusKind}/${tone}`);
  }
});
