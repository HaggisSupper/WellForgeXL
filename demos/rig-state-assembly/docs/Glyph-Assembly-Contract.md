# Rig-State Glyph Assembly Contract

## Authority

`src/contracts/rig-state.ts` defines layers, activities, dysfunctions, and display states for this independent demo only. Renderers consume its generated outputs; they do not maintain alternate state lists. Numerical Core remains the single numerical authority; no canonical classifier, engine or desktop schema is replaced.

`npm run generate:contracts` emits the portable contract under `public/contracts/`:

- `rig-state.manifest.json` — complete canonical source export.
- `rig-state-catalogue.json` — state IDs and operator labels.
- `rig-state-compositions.json` — renderer-neutral layer assemblies.
- `rig-state-composition.schema.json` — strict JSON schema for portable compositions.

## Non-negotiable semantics

| Condition | Display rule |
| --- | --- |
| Flow check | `HOLD_OBSERVE_RETURNS`; no circulation loop primitive. |
| Pending classification | Retain the previous confirmed activity and expose the dwell reason. |
| Stale telemetry | Retain activity while exposing the liveness failure. |
| Conflicting telemetry | Show conflict and evidence; do not manufacture a new assertion. |
| Unclassified | Show unclassified status and the reason; do not treat it as a confident idle state. |
| Critical losses, influx, well control | Use dedicated geometry plus the persistent critical rail. |

## Validation gates

The semantic validator rejects missing collections, empty required collections, missing/empty layer lists, duplicate IDs, dangling layer/activity/dysfunction references, and duplicate exclusive activity slots. Explicit manifests must supply layers, activities, dysfunctions (which may be empty) and states; they do not inherit omitted collections. Assembly and artifact creation validate before producing output.

The assembly boundary rejects non-finite confidence or values outside [0, 1], negative/non-finite age, malformed evidence arrays, and previous state IDs that are absent or UNKNOWN. Unknown current IDs fall back to an unclassified display. Valid supplied inputs retain conflict-before-stale-before-pending/unclassified precedence. The contract-artifact validator rejects a mismatch between state catalogue and portable compositions.

Every renderer must preserve the generated `id`, `activityId`, `layerIds`, and `dysfunctionIds` exactly. SVG renderers must also provide a readable name and `<title>` for each assembled glyph.

## Operational boundary

This prototype communicates supplied interpretation and simulated evidence. No WITSML/ETP service, telemetry inference, calibrated confidence, dwell engine, alarm authority or field qualification is implemented. Audio remains silent. Custom manifests are validated for assembly/export; the SVG renderer continues to use this demo's default vocabulary, not arbitrary alternate geometry. Compound Boolean geometry and native export are not implemented. It does not substitute for rig procedures, alarms, or human authority.
