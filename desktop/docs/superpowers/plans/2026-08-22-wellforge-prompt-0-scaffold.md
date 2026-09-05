# WellForge Prompt 0 Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the initial local-first WellForge Tauri application shell with explicit state, IPC, frontend stores, crate boundaries, and documentation.

**Architecture:** A Rust 2024 Cargo workspace holds domain crate boundaries. Tauri owns the state and JSON IPC seam while React owns shell rendering and local display preferences. The scaffold intentionally excludes engineering math and persistence behavior.

**Tech Stack:** Rust 2024, Tauri 2, React 18, TypeScript, Vite, Zustand, Tailwind CSS, thiserror, serde.

**Spec:** `docs/superpowers/specs/2026-08-22-wellforge-prompt-0-design.md`

## Global Constraints

- Use Rust 2024, Tauri 2, React 18, TypeScript, Vite, Zustand, and Tailwind.
- Core application workflows are local-only; do not add a cloud service, Docker, Electron, or Go.
- Rust owns all future engineering math; TypeScript contains no survey, AC, BHA, T&D, or hydraulics formulas.
- Translate Rust failures using `thiserror` into `{ code, message, details }` JSON contracts.
- Preserve explicit crate ownership; do not fabricate dummy physics or database behavior.

---

### Task 1: Initialize reproducible workspace metadata

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `tailwind.config.ts`
- Create: `postcss.config.js`

**Interfaces:**
- Produces: a Rust 2024 workspace and Vite application build commands.

- [ ] **Step 1: Write a failing build invocation**

Run: `cargo check --workspace`

Expected: failure because the Cargo workspace does not exist.

- [ ] **Step 2: Add minimal workspace and frontend build metadata**

Use workspace members `src-tauri` and the nine `crates/*` libraries. Add npm scripts `dev`, `build`, `typecheck`, and `test`.

- [ ] **Step 3: Verify workspace metadata resolves**

Run: `cargo metadata --no-deps --format-version 1`

Expected: workspace members are listed.

### Task 2: Define shared contracts and Tauri state

**Files:**
- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`
- Create: `crates/core/tests/contracts.rs`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/commands/mod.rs`

**Interfaces:**
- Produces: `wellforge_core::{ApiError, ProjectSummary, UnitPreferences, PlotPreferences}`.
- Produces: `AppState::new()`, `AppState::open_project(path)`, and `AppState::save_project()`.

- [ ] **Step 1: Write failing contract tests**

```rust
#[test]
fn no_project_error_serializes_to_structured_json() {
    let error = ApiError::no_open_project();
    let value = serde_json::to_value(error).unwrap();
    assert_eq!(value["code"], "NO_OPEN_PROJECT");
}
```

- [ ] **Step 2: Run the test and verify it fails for missing symbols**

Run: `cargo test -p wellforge-core --test contracts`

Expected: failure because `ApiError` has not been implemented.

- [ ] **Step 3: Implement the minimal serializable contract types**

Define typed unit/plot preferences, `ApiError` using `thiserror`, and project summary. Protect mutable `AppState` values with `RwLock`.

- [ ] **Step 4: Run the focused tests**

Run: `cargo test -p wellforge-core --test contracts`

Expected: PASS.

### Task 3: Add the requested domain crate boundaries

**Files:**
- Create: `crates/{survey,iscwsa,ac,bha,tnd,hydra,wits,db}/Cargo.toml`
- Create: `crates/{survey,iscwsa,ac,bha,tnd,hydra,wits,db}/src/lib.rs`
- Test: `cargo metadata --no-deps --format-version 1`

**Interfaces:**
- Consumes: root Cargo workspace.
- Produces: named library packages `wellforge-survey`, `wellforge-iscwsa`, `wellforge-ac`, `wellforge-bha`, `wellforge-tnd`, `wellforge-hydra`, `wellforge-wits`, and `wellforge-db`.

- [ ] **Step 1: Write a failing metadata assertion**

Run: `cargo metadata --no-deps --format-version 1 | rg 'wellforge-survey'`

Expected: no match because the package is absent.

- [ ] **Step 2: Create minimal documented library boundaries**

Each crate exports only a crate-level ownership document; it must not export dummy engineering calculations.

- [ ] **Step 3: Re-run workspace metadata and compilation**

Run: `cargo check --workspace`

Expected: PASS.

### Task 4: Wire the Tauri application and command contracts

**Files:**
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/tauri.conf.json`
- Test: `src-tauri/src/commands/mod.rs` unit tests

**Interfaces:**
- Consumes: `AppState`, `ProjectSummary`, `UnitPreferences`, and `ApiError`.
- Produces: IPC commands `ping`, `open_project`, `save_project`, and `get_units`.

- [ ] **Step 1: Write failing command tests**

```rust
#[test]
fn save_project_fails_without_an_open_project() {
    let state = AppState::new();
    let error = state.save_project().unwrap_err();
    assert_eq!(error.code(), "NO_OPEN_PROJECT");
}
```

- [ ] **Step 2: Verify the test fails because the state behavior is missing**

Run: `cargo test -p wellforge-app save_project_fails_without_an_open_project`

Expected: failure before implementation.

- [ ] **Step 3: Register commands and expose state-backed behavior**

Return typed JSON-compatible results. `open_project` validates that a non-empty path was supplied; `save_project` records a deterministic local save timestamp only after a project is active.

- [ ] **Step 4: Verify focused and workspace tests**

Run: `cargo test --workspace`

Expected: PASS.

### Task 5: Build the frontend navigation shell and Zustand stores

**Files:**
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src/lib/ipc.ts`
- Create: `src/stores/project.ts`
- Create: `src/stores/plotPrefs.ts`
- Create: `src/stores/ui.ts`
- Create: `src/components/Sidebar.tsx`
- Create: `src/components/EmptyCanvas.tsx`
- Create: `src/**/*.test.tsx`

**Interfaces:**
- Consumes: `invoke` calls declared in `src/lib/ipc.ts` and typed frontend state.
- Produces: the sidebar with eight named module entries and empty content canvas.

- [ ] **Step 1: Write a failing UI test**

```tsx
it('renders every WellForge module in the navigation', () => {
  render(<App />);
  expect(screen.getByRole('navigation')).toHaveTextContent('Hydraulics');
  expect(screen.getByRole('navigation')).toHaveTextContent('Reports');
});
```

- [ ] **Step 2: Run the test and verify it fails because App is absent**

Run: `npm test -- --run src/App.test.tsx`

Expected: failure for a missing module.

- [ ] **Step 3: Implement minimal shell, styling, and isolated stores**

Use Tailwind directives and a concise desktop shell. The canvas may show state text only; it must contain no computations.

- [ ] **Step 4: Run frontend tests and type-check**

Run: `npm test -- --run && npm run typecheck && npm run build`

Expected: PASS.

### Task 6: Document and validate the scaffold

**Files:**
- Create: `README.md`
- Modify: `docs/superpowers/specs/2026-08-22-wellforge-prompt-0-design.md`
- Test: workspace and frontend verification commands

**Interfaces:**
- Produces: build/run instructions and a crate responsibility map.

- [ ] **Step 1: Write README acceptance checks**

Run: `rg -n 'cargo tauri dev|wellforge-survey|React 18' README.md`

Expected: initially no matches.

- [ ] **Step 2: Document prerequisites, commands, crate map, and architecture boundary**

Include how to run `npm install`, `npm run tauri dev`, frontend checks, and Cargo tests. State that physics implementation begins in later prompts.

- [ ] **Step 3: Execute the complete verification matrix**

Run: `cargo test --workspace && cargo check --workspace && npm test -- --run && npm run typecheck && npm run build`

Expected: all commands exit zero.

- [ ] **Step 4: Review the diff**

Run: `git diff --check && git status --short`

Expected: no whitespace errors and only Prompt 0 files changed.
