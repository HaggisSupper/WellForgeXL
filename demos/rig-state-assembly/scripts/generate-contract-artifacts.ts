import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { createRigStateArtifacts, validateRigStateArtifacts } from "../src/contracts/export.ts";
import { RIG_STATE_MANIFEST, validateRigStateManifest } from "../src/contracts/rig-state.ts";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const outputDirectory = resolve(packageRoot, "public/contracts");
const manifestErrors = validateRigStateManifest();
const artifacts = createRigStateArtifacts();
const artifactErrors = validateRigStateArtifacts(artifacts);

if (manifestErrors.length || artifactErrors.length) {
  throw new Error([...manifestErrors, ...artifactErrors].join("\n"));
}

await mkdir(outputDirectory, { recursive: true });
await Promise.all([
  writeFile(resolve(outputDirectory, "rig-state.manifest.json"), `${JSON.stringify(RIG_STATE_MANIFEST, null, 2)}\n`),
  writeFile(resolve(outputDirectory, "rig-state-catalogue.json"), `${JSON.stringify(artifacts.catalogue, null, 2)}\n`),
  writeFile(resolve(outputDirectory, "rig-state-compositions.json"), `${JSON.stringify({
    $schema: "./rig-state-composition.schema.json",
    version: "0.1.0",
    states: artifacts.compositions,
  }, null, 2)}\n`),
  copyFile(resolve(packageRoot, "contracts/rig-state-composition.schema.json"), resolve(outputDirectory, "rig-state-composition.schema.json")),
]);

process.stdout.write(`Generated ${artifacts.catalogue.length} rig-state catalogue entries and compositions.\n`);
