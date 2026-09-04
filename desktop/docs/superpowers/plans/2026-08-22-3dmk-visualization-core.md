# 3Dmk Visualization Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish 3Dmk as WellForge's reusable typed 3D core and render a validated survey scene in the Tauri desktop UI.

**Architecture:** `wellforge-3d` owns a versioned renderer-neutral scene document. `wellforge-survey` adapts cumulative positions into that document; Tauri serializes it; React renders it on one native WebGL2 canvas while retaining only camera and layer-visibility state.

**Tech Stack:** Rust 2024, serde, thiserror, Tauri 2, React 18, TypeScript, Zustand, native WebGL2, Vitest.

**Spec:** `docs/superpowers/specs/2026-08-22-3dmk-visualization-core-design.md`

## Global Constraints

- `wellforge-3d` owns V1 3D scene contracts and validates all geometry in Rust.
- SI metres and the `north-east-tvd-m` coordinate frame are mandatory for the first slice.
- TypeScript must only render typed scene values and manage view state; no engineering math is permitted.
- Scene provenance is mandatory and is preserved through the IPC boundary.
- CUDA remains the preferred future accelerated backend and Vulkan/WebGPU the fallback; neither is introduced without CPU parity evidence.
- Do not introduce Docker, Electron, cloud dependencies, or silent operational fallback.

### Task 1: Create the V1 3Dmk Rust contract

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/three_d/Cargo.toml`
- Create: `crates/three_d/src/lib.rs`
- Create: `crates/three_d/tests/scene_contract.rs`

**Interfaces:**
- Produces: `SceneDocumentV1::new`, `SceneLayerV1`, `ScenePrimitiveV1`, `ScenePoint`, `SceneBounds`, `SceneProvenanceV1`, and `SceneError`.

- [ ] **Step 1: Write failing contract tests**

```rust
#[test]
fn scene_rejects_a_non_finite_vertex() {
    let error = SceneDocumentV1::new("scene", "title", vec![invalid_polyline(f64::NAN)], provenance()).unwrap_err();
    assert_eq!(error.code(), "NON_FINITE_SCENE_COORDINATE");
}

#[test]
fn scene_calculates_ne_tvd_bounds_in_rust() {
    let scene = SceneDocumentV1::new("scene", "title", vec![polyline(vec![(2.0, 3.0, 4.0), (-1.0, 7.0, 6.0)])], provenance()).unwrap();
    assert_eq!(scene.bounds.minimum, ScenePoint::new(-1.0, 3.0, 4.0));
    assert_eq!(scene.bounds.maximum, ScenePoint::new(2.0, 7.0, 6.0));
}
```

- [ ] **Step 2: Run focused test to verify it fails**

Run: `cargo test -p wellforge-3d --test scene_contract`

Expected: FAIL because package and types do not exist.

- [ ] **Step 3: Implement the smallest immutable serializable V1 scene contract**

Implement `SceneDocumentV1::new(scene_id: String, title: String, layers: Vec<SceneLayerV1>, provenance: SceneProvenanceV1) -> Result<SceneDocumentV1, SceneError>`. Validate non-empty IDs, distinct layer IDs, finite points, and at least one point. Compute `SceneBounds` from all primitive points. Serialize camelCase JSON with `schemaVersion: "wellforge.scene/v1"` and `coordinateFrame: "north-east-tvd-m"`.

- [ ] **Step 4: Run focused tests**

Run: `cargo test -p wellforge-3d --test scene_contract`

Expected: PASS.

### Task 2: Adapt survey results into a 3Dmk scene

**Files:**
- Modify: `crates/survey/Cargo.toml`
- Modify: `crates/survey/src/lib.rs`
- Modify: `crates/survey/tests/survey_math.rs`

**Interfaces:**
- Consumes: `SurveyPosition` and `wellforge_3d::SceneDocumentV1`.
- Produces: `build_survey_scene(stations: &[SurveyPosition]) -> Result<SceneDocumentV1, SurveyError>`.

- [ ] **Step 1: Write failing survey adapter test**

```rust
#[test]
fn survey_scene_preserves_cumulative_ne_tvd_coordinates() {
    let scene = build_survey_scene(&positions()).unwrap();
    let points = scene.layers[0].primitives[0].points();
    assert_eq!(points[1], ScenePoint::new(20.0, 10.0, 100.0));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p wellforge-survey survey_scene_preserves_cumulative_ne_tvd_coordinates`

Expected: FAIL because `build_survey_scene` does not exist.

- [ ] **Step 3: Implement the adapter without re-calculation**

Map `SurveyPosition { north_m, east_m, tvd_m }` directly to `ScenePoint { x: north_m, y: east_m, z: tvd_m }`, create a selectable `survey-path` polyline layer and `survey-stations` marker layer, and attach `algorithm: "survey-position-adapter"`, `profile_version: "v1"`, `backend: "cpu"`, and measured-depth labels as marker metadata. Preserve the existing `PlotSpec` API.

- [ ] **Step 4: Run focused tests**

Run: `cargo test -p wellforge-survey`

Expected: PASS.

### Task 3: Publish scenes through explicit Tauri IPC

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`
- Create or modify: `src-tauri/src/commands` tests

**Interfaces:**
- Produces: `build_survey_scene(request: SurveySceneRequest) -> Result<SceneDocumentV1, ApiError>`.

- [ ] **Step 1: Write a failing command test**

```rust
#[test]
fn scene_command_returns_structured_error_for_non_finite_station() {
    let result = build_survey_scene(SurveySceneRequest { stations: vec![non_finite_position()] });
    assert_eq!(result.unwrap_err().code, "NON_FINITE_INPUT");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p wellforge-app scene_command_returns_structured_error_for_non_finite_station`

Expected: FAIL because the command and request do not exist.

- [ ] **Step 3: Register the command and translate only typed errors**

Add `SurveySceneRequest { stations: Vec<SurveyPosition> }`, map `SurveyError` to `ApiError`, and register `build_survey_scene` exactly once in `tauri::generate_handler!`.

- [ ] **Step 4: Run focused commands tests**

Run: `cargo test -p wellforge-app`

Expected: PASS.

### Task 4: Add the 3Dmk desktop viewport

**Files:**
- Modify: `package.json`
- Modify: `src/lib/ipc.ts`
- Create: `src/lib/scene.ts`
- Create: `src/lib/scene.test.ts`
- Create: `src/stores/scene.ts`
- Create: `src/components/ThreeDViewport.tsx`
- Modify: `src/components/EmptyCanvas.tsx`
- Modify: `src/styles.css`
- Create: `src/components/ThreeDViewport.test.tsx`

**Interfaces:**
- Consumes: `wellforgeIpc.buildSurveyScene(stations)` returning `SceneDocumentV1`.
- Produces: one native WebGL2 viewport with orbit/pan/zoom, scene-layer toggles, and provenance/status display.

- [ ] **Step 1: Add a failing viewport test**

```tsx
it("shows typed scene provenance and provides a layer toggle", async () => {
  render(<ThreeDViewport scene={fixtureScene} />);
  expect(screen.getByText("survey-position-adapter")).toBeInTheDocument();
  expect(screen.getByRole("checkbox", { name: "survey-path" })).toBeChecked();
});
```

- [ ] **Step 2: Run it to verify failure**

Run: `npm test -- --run src/components/ThreeDViewport.test.tsx`

Expected: FAIL because the component does not exist.

- [ ] **Step 3: Render the schema primitives through native WebGL2**

Define scene types and `parseSceneDocumentV1(payload: unknown)` in `src/lib/scene.ts`; reject unsupported schemas, malformed primitives, and non-finite coordinates before data enters UI state. Initialize one `WebGL2RenderingContext` inside `ThreeDViewport.tsx`. Render only polyline and marker primitives, fit the camera using Rust-supplied bounds, and retain layer visibility only in `src/stores/scene.ts`. Do not calculate engineering geometry or bounds in TypeScript.

- [ ] **Step 4: Integrate the scene panel in the Surveys workspace**

When a typed scene exists, show the 3Dmk viewport beside `SurveyGrid`; otherwise show an explicit empty-scene panel and a test fixture action only in test/dev code. Do not claim to display a scene without an IPC response.

- [ ] **Step 5: Run frontend tests and static checks**

Run: `npm test -- --run && npm run typecheck && npm run build`

Expected: PASS.

### Task 5: Document ownership and execute full verification

**Files:**
- Modify: `README.md`
- Modify: `docs/wellforge-program-plan.md`
- Modify: `docs/superpowers/specs/2026-08-22-3dmk-visualization-core-design.md`

**Interfaces:**
- Produces: discoverable 3Dmk ownership, backend policy, and build/verification instructions.

- [ ] **Step 1: Add `wellforge-3d` to the crate map and master ownership table**

State that it owns scene schema/validation/provenance and must not own engineering equations or domain data persistence. Record CPU reference, CUDA preference, and Vulkan/WebGPU fallback policy.

- [ ] **Step 2: Execute complete verification**

Run: `cargo test --workspace && cargo check --workspace && npm test -- --run && npm run typecheck && npm run build`

Expected: all commands exit zero.

- [ ] **Step 3: Perform final hygiene check**

Run: `rg -n "TODO|TBD|placeholder" crates/three_d src/components/ThreeDViewport.tsx docs/superpowers/specs/2026-08-22-3dmk-visualization-core-design.md`

Expected: no incomplete implementation markers.
