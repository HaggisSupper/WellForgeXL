# WellForge Prompt 0: Scaffold Design

## Purpose

Create the local-first WellForge desktop application shell. This tranche establishes stable Rust, IPC, state, frontend, and crate boundaries only. It deliberately contains no survey, anti-collision, BHA, T&D, hydraulics, or realtime calculation implementations.

## Architecture

WellForge is a Tauri 2 desktop application using a Rust 2024 workspace for domain capabilities and a React 18 + TypeScript frontend built by Vite. Rust owns all future engineering computation. The frontend owns display state and sends typed request/response commands through Tauri IPC.

`src-tauri/src/state.rs` holds an `AppState` managed by Tauri. Its project handle, unit preferences, and plot preferences are protected with `std::sync::RwLock`, so later commands may read concurrently and make writes explicit. `src-tauri/src/commands/mod.rs` is the sole command registration boundary. Commands return structured `ApiError` values serialized to the UI.

The Cargo workspace contains independent Rust library crates under `crates/`. Each crate has an explicit responsibility and must depend only on lower-level boundaries as later prompts require. The initial crates are empty capability boundaries, rather than fake calculations or premature shared abstractions.

## IPC Contract v0

| Command | Input | Output | Effect |
|---|---|---|---|
| `ping` | none | `{ message: "wellforge-ready" }` | confirms command bridge availability |
| `open_project` | `{ path: string }` | `ProjectSummary` | records the selected local project path |
| `save_project` | none | `ProjectSummary` | returns the active project or an error when none is open |
| `get_units` | none | `UnitPreferences` | returns current unit preferences |

`ProjectSummary`, `UnitPreferences`, `PlotPreferences`, and `ApiError` are serializable Rust contracts. All errors have machine-readable `code`, human-readable `message`, and optional `details` fields.

## UI

The UI has a fixed sidebar containing Project, Surveys, Plans, AC, BHA, T&D, Hydraulics, and Reports. The initial content canvas communicates that a module has not yet been implemented. Zustand stores isolate `project`, `plotPrefs`, and `ui` state. No engineering values or computational logic are in TypeScript.

## Constraints

- Tauri 2, Rust edition 2024, React 18, TypeScript, Vite, Zustand, and Tailwind are mandatory.
- WellForge is local-first; core workflows have no cloud dependency.
- Never use Electron, Docker, Go, or TypeScript for survey/AC/BHA mathematics.
- SI is the eventual internal system; UI preferences support oilfield, SI, and custom modes.
- Rust errors use `thiserror` and serialize as structured JSON to the frontend.
- PostgreSQL access/migrations and synchronization contracts are reserved for
  the `wellforge-db` crate. Prompt 0 includes no database implementation;
  offline saves use managed XML/artifact files and synchronization envelopes.

## Acceptance Criteria

1. Cargo workspace exposes the nine requested named crates.
2. Tauri compiles with state and four registered commands.
3. Frontend builds and renders the requested navigation shell.
4. The frontend stores are typed and separate by responsibility.
5. README explains build/run commands and crate ownership.
6. Rust tests and frontend checks pass without requiring a network connection at runtime.
