# Desktop source migration

The WellForge Tauri desktop source is maintained under `desktop/` in this
repository. It is intentionally contained as its own Cargo and Vite workspace
while the existing workbook suite and production Rust lanes remain under their
established paths.

## Migrated now

- React, TypeScript, Vite, Tailwind, and Zustand desktop UI
- Tauri host, commands, capabilities, and local state
- Desktop-specific Rust crates for core contracts, persistence, formats, 3D
  scene documents, anti-collision, and WITS support
- Desktop tests, fixtures, and port architecture documents

The native host exposes `build_trajectory_scene` as the first universal
calculation ingress. It accepts canonical trajectory-result JSON, validates
the result projection in Rust, and returns a `wellforge.scene/v1` document to
the renderer. Future BHA, hydraulics, and torque/drag commands should follow
the same result-JSON-to-scene pattern.

The host also contains a shared engine runner that invokes canonical CLI
lanes with explicit input and output paths. It rejects path aliasing before
launch, verifies packaged executable SHA-256 sidecars, and reports typed
discovery, launch, execution, and result-read failures.

Workspace-scoped execution additionally requires request and result paths to
remain inside the selected project workspace before the executable is
launched. React does not supply an executable path or an unrestricted output
location.

## Deliberately not duplicated

The source `survey`, `bha`, `hydra`, `tnd`, and `wits` crates overlap the
validated Rust lanes in `engine/crates`. They remain in `desktop/` for the
initial desktop workspace build, but the next convergence step should replace
those duplicate calculation implementations with adapters to the canonical
engine result boundaries.

Generated output, dependency caches, graph exports, and the legacy `work/`
reference dump are excluded from the migration and ignored when nested under
`desktop/`.
