# WellForge Rig State Assembly

An optional private web demo for supplied rig-state display inputs. All scenario telemetry is simulated: no live WITSML/ETP, sensor inference, production alarms, new physics or field qualification is implemented. Numerical Core remains the single numerical authority. This package does not connect to or replace the canonical Rust classifier or desktop application.

## What is demonstrable

- Rig-state glyphs for drilling, sliding, circulation, trips, ream/backream, coring, cement circulation, flow check, dysfunctions, and well control.
- Explicit `confirmed`, `pending`, `stale`, `conflict`, and `unclassified` status semantics with evidence and liveness.
- A flow check that displays hold-and-observe returns—not circulation.
- Distinct critical geometries for loss, influx, and well control, plus an accessible SVG label and title.
- Contract artefacts in `public/contracts/`, emitted from the same TypeScript manifest used by the renderer.
- 18 states, 17 scenarios, custom builder, comparison, evidence, browser SVG/JSON downloads and print presentation.

## Run locally

```powershell
npm ci --ignore-scripts --no-audit --no-fund
npm test
npm run typecheck
npm run generate:contracts
npm run dev
```

## Build the web prototype

```powershell
npm run generate:contracts
npm run build
```

The static prototype is emitted to ignored `dist/`. With Node 24, run all commands from this demo directory. `npm run preview` serves the build at `127.0.0.1:4173`; development also binds to loopback. No external fonts or telemetry services are required. Root package commands and dependencies are unchanged.

## Boundaries

The source dependency closure and lockfile are retained, including unused Tauri packages. Native commands and IPC export stubs are excluded from the web flow. Downloads use browser Blob URLs; JSON and reports identify simulated data. Print Report delegates to browser print support. DOM/CSS checks do not establish native preview, pagination or PDF-export acceptance.

Generation is anchored to the script's package location and writes only four files in `public/contracts/`. The artifact test regenerates those files and checks byte parity. It does not write to the caller's working directory.

Invalid confidence, age, previous state, evidence shape and incomplete/dangling manifests throw before assembly/export. Unknown current IDs remain unclassified. Conflict precedes stale (>30 seconds), then pending/unclassified; pending retains a known previous state. The looping countdown is illustrative, not a dwell engine, and audio intentionally remains silent.

The guide preserves unvalidated design ideas, not approved operational rules. Compound Boolean geometry, audible alarms, live telemetry and native export remain deferred. See [feature dispositions](../../docs/consolidation/rig-state-delta.md) and [display contract](docs/Glyph-Assembly-Contract.md).
