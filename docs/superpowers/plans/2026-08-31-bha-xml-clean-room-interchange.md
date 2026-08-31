# BHA XML Clean-Room Interchange Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an independent Rust BHA XML interchange crate that produces sanitized structural JSON and canonical BHA component JSON from neutral XML examples.

**Architecture:** A new `wellforge-bha-interchange` workspace crate parses XML into an ordered generic tree, sanitizes restricted identity metadata using deployment-provided token fingerprints, and exposes both a lossless structural document and a typed BHA assembly projection. The crate remains independent from the BHA solver and receives only newly authored neutral fixtures.

**Tech Stack:** Rust 2024, `quick-xml`, `serde`, `serde_json`, `sha2`, `thiserror`, `uuid`, Cargo workspace tests.

**Spec:** `docs/superpowers/specs/2026-08-31-bha-xml-clean-room-interchange-design.md`

## Global Constraints

- Use only independently authored neutral XML fixtures; do not copy implementation code, comments, metadata, or identifiers from the legacy tree.
- Do not commit legacy-vendor, legacy-product, or legacy-version identifiers to source, fixtures, output, docs, or test names.
- Receive restricted-token SHA-256 fingerprints from callers; do not store plaintext restricted tokens in the repository.
- Disable DTD and external-entity processing for every XML parse.
- Preserve element, attribute, component, and section ordering in structural JSON.
- Use workspace dependency versions and inherit workspace Rust/clippy lints.
- Keep this crate independent of all BHA solver crates during this plan.

---

## File Structure

| Path | Responsibility |
|---|---|
| `engine/Cargo.toml` | Registers the new workspace member. |
| `engine/crates/wellforge-bha-interchange/Cargo.toml` | Declares the isolated crate and inherited dependencies/lints. |
| `engine/crates/wellforge-bha-interchange/src/lib.rs` | Exposes the stable XML-to-JSON public API. |
| `engine/crates/wellforge-bha-interchange/src/error.rs` | Typed parse, policy, projection, and validation errors. |
| `engine/crates/wellforge-bha-interchange/src/xml_tree.rs` | Safe XML parser and ordered structural tree. |
| `engine/crates/wellforge-bha-interchange/src/sanitize.rs` | Token-fingerprint policy and recursive structural sanitizer. |
| `engine/crates/wellforge-bha-interchange/src/structural_json.rs` | Structural JSON encoding helpers. |
| `engine/crates/wellforge-bha-interchange/src/model.rs` | Canonical assembly/component/section and tool-detail types. |
| `engine/crates/wellforge-bha-interchange/src/project.rs` | Converts supported structural nodes into canonical BHA data. |
| `engine/crates/wellforge-bha-interchange/src/validate.rs` | Applies model and geometry validation. |
| `engine/crates/wellforge-bha-interchange/tests/fixtures/neutral_bha.xml` | Independently authored neutral assembly input. |
| `engine/crates/wellforge-bha-interchange/tests/structural_json.rs` | Parser/order/JSON and sanitization integration tests. |
| `engine/crates/wellforge-bha-interchange/tests/projection.rs` | Canonical projection and validation integration tests. |

## Public Interfaces

```rust
pub struct StructuralNode {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<StructuralNode>,
}

pub struct SanitizationPolicy {
    pub blocked_token_digests: std::collections::BTreeSet<[u8; 32]>,
}

pub struct SanitizationReport {
    pub removed_elements: usize,
    pub removed_attributes: usize,
    pub removed_values: usize,
}

pub struct InterchangeOutput {
    pub structural: StructuralNode,
    pub canonical: BhaAssembly,
    pub report: SanitizationReport,
}

pub fn parse_xml(input: &str) -> Result<StructuralNode, InterchangeError>;
pub fn sanitize_tree(
    tree: StructuralNode,
    policy: &SanitizationPolicy,
) -> Result<(StructuralNode, SanitizationReport), InterchangeError>;
pub fn project_bha(tree: &StructuralNode) -> Result<BhaAssembly, InterchangeError>;
pub fn convert_xml(
    input: &str,
    policy: &SanitizationPolicy,
) -> Result<InterchangeOutput, InterchangeError>;
```

### Task 1: Register the isolated crate and establish the public error surface

**Files:**
- Modify: `engine/Cargo.toml`
- Create: `engine/crates/wellforge-bha-interchange/Cargo.toml`
- Create: `engine/crates/wellforge-bha-interchange/src/lib.rs`
- Create: `engine/crates/wellforge-bha-interchange/src/error.rs`
- Test: `engine/crates/wellforge-bha-interchange/tests/structural_json.rs`

**Interfaces:**
- Consumes: workspace dependencies and lints declared by `engine/Cargo.toml`.
- Produces: the `wellforge_bha_interchange` crate and `InterchangeError` used by every later task.

- [ ] **Step 1: Write the failing workspace-discovery test**

```rust
#[test]
fn crate_exposes_a_typed_error() {
    let error = wellforge_bha_interchange::InterchangeError::UnsupportedRoot("Other".into());
    assert_eq!(error.to_string(), "unsupported BHA XML root: Other");
}
```

- [ ] **Step 2: Run the test to verify it fails before the crate exists**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json crate_exposes_a_typed_error`

Expected: Cargo reports that package `wellforge-bha-interchange` does not exist.

- [ ] **Step 3: Register and implement the minimal crate surface**

Add `crates/wellforge-bha-interchange` to `engine/Cargo.toml`. Create a `Cargo.toml` that inherits the workspace version, edition, rust version, license, authors, lints, and required workspace dependencies. Define:

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum InterchangeError {
    #[error("invalid BHA XML: {0}")]
    InvalidXml(String),
    #[error("DTD and external entities are not permitted")]
    ProhibitedXmlConstruct,
    #[error("unsupported BHA XML root: {0}")]
    UnsupportedRoot(String),
    #[error("required field is missing: {0}")]
    MissingField(&'static str),
    #[error("invalid field {field}: {value}")]
    InvalidField { field: &'static str, value: String },
    #[error("invalid BHA geometry: {0}")]
    InvalidGeometry(String),
}
```

Re-export `InterchangeError` from `lib.rs`.

- [ ] **Step 4: Run the focused test**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json crate_exposes_a_typed_error`

Expected: PASS.

- [ ] **Step 5: Run formatting and commit the isolated crate shell**

Run: `cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

Commit:

```bash
git add engine/Cargo.toml engine/crates/wellforge-bha-interchange
git commit -m "feat: add BHA XML interchange crate"
```

### Task 2: Parse neutral XML into a safe ordered structural tree

**Files:**
- Create: `engine/crates/wellforge-bha-interchange/src/xml_tree.rs`
- Modify: `engine/crates/wellforge-bha-interchange/src/lib.rs`
- Create: `engine/crates/wellforge-bha-interchange/tests/fixtures/neutral_bha.xml`
- Modify: `engine/crates/wellforge-bha-interchange/tests/structural_json.rs`

**Interfaces:**
- Consumes: `InterchangeError` and the public `StructuralNode` type.
- Produces: `parse_xml(&str) -> Result<StructuralNode, InterchangeError>` for sanitization and projection.

- [ ] **Step 1: Write failing parser tests for ordering and XML hardening**

```rust
#[test]
fn parser_preserves_repeated_component_order() {
    let tree = parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap();
    let components = tree.children.iter().find(|node| node.name == "Components").unwrap();
    let captions: Vec<_> = components.children.iter()
        .map(|component| component.children.iter().find(|node| node.name == "Caption").unwrap().text.as_deref())
        .collect();
    assert_eq!(captions, vec![Some("Neutral bit"), Some("Neutral tubular")]);
}

#[test]
fn parser_rejects_a_doctype() {
    let xml = "<!DOCTYPE BHA [<!ENTITY sample SYSTEM 'file:///not-used'>]><BHA/>";
    assert_eq!(parse_xml(xml).unwrap_err(), InterchangeError::ProhibitedXmlConstruct);
}
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json parser_`

Expected: FAIL because `parse_xml` and `StructuralNode` are not exported.

- [ ] **Step 3: Implement the minimal safe parser**

Use `quick_xml::Reader` with text trimming disabled and a stack of `StructuralNode` values. Reject `DocType`, entity declaration, and processing-instruction events; accept only a single root named `BHA` or `Component`. Store attributes in reader order, decode names/text, preserve child order, and reject unbalanced input as `InvalidXml`.

Create `neutral_bha.xml` with an invented two-component assembly, no legacy metadata, and two ordered sections on the tubular component.

- [ ] **Step 4: Run focused parser tests and formatting**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json parser_ && cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit the safe parser**

```bash
git add engine/crates/wellforge-bha-interchange/src/xml_tree.rs engine/crates/wellforge-bha-interchange/src/lib.rs engine/crates/wellforge-bha-interchange/tests
git commit -m "feat: parse neutral BHA XML safely"
```

### Task 3: Sanitize restricted identity metadata and emit structural JSON

**Files:**
- Create: `engine/crates/wellforge-bha-interchange/src/sanitize.rs`
- Create: `engine/crates/wellforge-bha-interchange/src/structural_json.rs`
- Modify: `engine/crates/wellforge-bha-interchange/src/lib.rs`
- Modify: `engine/crates/wellforge-bha-interchange/tests/structural_json.rs`

**Interfaces:**
- Consumes: `StructuralNode`, `InterchangeError`, and caller-provided SHA-256 token fingerprints.
- Produces: `SanitizationPolicy`, `SanitizationReport`, `sanitize_tree`, and structural JSON serialization.

- [ ] **Step 1: Write failing policy and JSON tests**

```rust
#[test]
fn sanitizer_removes_matching_element_attribute_and_value() {
    let policy = SanitizationPolicy::new(BTreeSet::from([digest_for_test("restrictedtoken")]));
    let tree = parse_xml(
        "<BHA source=\"restrictedtoken\"><Caption>Neutral</Caption><restrictedtoken>restrictedtoken</restrictedtoken></BHA>",
    ).unwrap();
    let (sanitized, report) = sanitize_tree(tree, &policy).unwrap();
    assert!(sanitized.attributes.is_empty());
    assert_eq!(sanitized.children.len(), 1);
    assert_eq!(report.removed_elements, 1);
    assert_eq!(report.removed_attributes, 1);
    assert_eq!(report.removed_values, 0);
}

#[test]
fn structural_json_retains_child_order() {
    let tree = parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap();
    let json = structural_json::to_value(&tree);
    assert_eq!(json["children"][0]["name"], "Caption");
    assert_eq!(json["children"][1]["name"], "Components");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json sanitizer_ structural_json_`

Expected: FAIL because policy, sanitization, and JSON APIs do not exist.

- [ ] **Step 3: Implement recursive sanitization and JSON conversion**

Define `SanitizationPolicy::new(BTreeSet<[u8; 32]>)` and `Default` as an empty policy for production. In the integration test module, define `digest_for_test` locally with `Sha256`; production crate code never accepts or stores plaintext restricted tokens. Tokenize names and scalar values at non-alphanumeric boundaries, lowercase each token, hash it with SHA-256, and compare it to the supplied digest set.

Remove a matching element and its descendants. Remove matching attributes. Remove a matching scalar text value while retaining the neutral node. Count each removal category without preserving the removed text. Serialize `StructuralNode` directly with `serde` so `children` remains a vector.

- [ ] **Step 4: Run focused tests, clippy, and formatting**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test structural_json && cargo +1.98.0 clippy --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --all-targets -- -D warnings && cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit sanitizer and structural JSON support**

```bash
git add engine/crates/wellforge-bha-interchange/src engine/crates/wellforge-bha-interchange/tests/structural_json.rs
git commit -m "feat: sanitize BHA interchange metadata"
```

### Task 4: Define canonical neutral BHA models and project generic components

**Files:**
- Create: `engine/crates/wellforge-bha-interchange/src/model.rs`
- Create: `engine/crates/wellforge-bha-interchange/src/project.rs`
- Create: `engine/crates/wellforge-bha-interchange/src/validate.rs`
- Modify: `engine/crates/wellforge-bha-interchange/src/lib.rs`
- Create: `engine/crates/wellforge-bha-interchange/tests/projection.rs`

**Interfaces:**
- Consumes: sanitized `StructuralNode` values.
- Produces: `BhaAssembly`, `BhaComponentRecord`, `TubularSection`, `ComponentKind`, `ComponentDetail`, and `project_bha`.

- [ ] **Step 1: Write failing canonical projection tests**

```rust
#[test]
fn projection_retains_component_and_section_order() {
    let tree = parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap();
    let assembly = project_bha(&tree).unwrap();
    assert_eq!(assembly.components[0].name, "Neutral bit");
    assert_eq!(assembly.components[1].sections[0].kind, "Tool joint top");
    assert_eq!(assembly.components[1].sections[1].kind, "Tube");
}

#[test]
fn projection_rejects_section_with_od_not_greater_than_id() {
    let tree = parse_xml("<BHA><Caption>Neutral</Caption><Components><Component><Caption>Pipe</Caption><Count>1</Count><PartType>Common</PartType><Sections><Section><SectionType>Tube</SectionType><OD>0.1</OD><ID>0.1</ID><Length>1.0</Length></Section></Sections></Component></Components></BHA>").unwrap();
    assert!(matches!(project_bha(&tree), Err(InterchangeError::InvalidGeometry(_))));
}
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test projection`

Expected: FAIL because `project_bha` and canonical model types do not exist.

- [ ] **Step 3: Implement generic model and validation**

Define serializable types from the approved spec. Map `PartType` values `Common`, `MudMotor`, `RSS`, and `Stabilizer` into `ComponentKind`; map any other non-empty value to `ComponentKind::Other(String)`. Read the first direct child of each required name, generate deterministic v5 UUIDs from assembly/component order and neutral captions, and preserve component and section sequence.

Require assembly caption, component caption, positive count, section type, OD, ID, and length. Validate positive section length, non-negative dimensions/mass, and `od_m > id_m`.

- [ ] **Step 4: Run projection tests and quality checks**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test projection && cargo +1.98.0 clippy --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --all-targets -- -D warnings && cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit generic projection support**

```bash
git add engine/crates/wellforge-bha-interchange/src engine/crates/wellforge-bha-interchange/tests/projection.rs
git commit -m "feat: project neutral BHA components"
```

### Task 5: Project supported motor, rotary-steerable, and stabilizer detail blocks

**Files:**
- Modify: `engine/crates/wellforge-bha-interchange/src/model.rs`
- Modify: `engine/crates/wellforge-bha-interchange/src/project.rs`
- Modify: `engine/crates/wellforge-bha-interchange/src/validate.rs`
- Modify: `engine/crates/wellforge-bha-interchange/tests/fixtures/neutral_bha.xml`
- Modify: `engine/crates/wellforge-bha-interchange/tests/projection.rs`

**Interfaces:**
- Consumes: generic canonical projection types from Task 4.
- Produces: `ComponentDetail::{Motor, RotarySteerable, Stabilizer}` with validated, optional data fields.

- [ ] **Step 1: Write failing detail-projection tests**

```rust
#[test]
fn projection_maps_supported_tool_details() {
    let assembly = project_bha(&parse_xml(include_str!("fixtures/neutral_bha.xml")).unwrap()).unwrap();
    let motor = assembly.components.iter().find(|item| item.kind == ComponentKind::MudMotor).unwrap();
    assert!(matches!(motor.detail, Some(ComponentDetail::Motor(ref detail)) if detail.bend_angle_deg == Some(1.25)));
    let rss = assembly.components.iter().find(|item| item.kind == ComponentKind::RotarySteerable).unwrap();
    assert!(matches!(rss.detail, Some(ComponentDetail::RotarySteerable(ref detail)) if detail.push_the_bit));
}
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test projection projection_maps_supported_tool_details`

Expected: FAIL because detail variants and XML mappings do not exist.

- [ ] **Step 3: Implement only the supported neutral detail fields**

Implement motor geometry, bend angle, lobe configuration, blade counts, and nested subassembly sections. Implement rotary-steerable collar OD/ID, length, pad count, pad distance from bit, and steering mode. Implement stabilizer OD/ID, gauge diameter, blade count, and sub-length fields.

Treat an absent optional detail block as `None`. Reject a detail block whose component kind conflicts with the block. Keep unrecognized neutral detail elements only in structural JSON.

- [ ] **Step 4: Run detail tests and the entire crate suite**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange && cargo +1.98.0 clippy --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --all-targets -- -D warnings && cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit supported tool details**

```bash
git add engine/crates/wellforge-bha-interchange
git commit -m "feat: project BHA tool details"
```

### Task 6: Add the end-to-end converter and verify repository boundaries

**Files:**
- Modify: `engine/crates/wellforge-bha-interchange/src/lib.rs`
- Modify: `engine/crates/wellforge-bha-interchange/tests/structural_json.rs`
- Modify: `engine/crates/wellforge-bha-interchange/tests/projection.rs`
- Modify: `docs/superpowers/specs/2026-08-31-bha-xml-clean-room-interchange-design.md` only if implementation reveals a design contradiction

**Interfaces:**
- Consumes: parser, sanitizer, structural serializer, projection, and validation APIs from Tasks 1-5.
- Produces: `convert_xml(&str, &SanitizationPolicy) -> Result<InterchangeOutput, InterchangeError>`.

- [ ] **Step 1: Write the failing end-to-end conversion test**

```rust
#[test]
fn converter_emits_sanitized_structural_and_canonical_json() {
    let output = convert_xml(include_str!("fixtures/neutral_bha.xml"), &SanitizationPolicy::default()).unwrap();
    let structural = serde_json::to_value(&output.structural).unwrap();
    let canonical = serde_json::to_value(&output.canonical).unwrap();
    assert_eq!(structural["name"], "BHA");
    assert_eq!(canonical["components"].as_array().unwrap().len(), 5);
    assert_eq!(output.report.removed_elements, 0);
}
```

- [ ] **Step 2: Run the end-to-end test to verify it fails**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --test projection converter_emits_sanitized_structural_and_canonical_json`

Expected: FAIL because `convert_xml` and `InterchangeOutput` are not exported.

- [ ] **Step 3: Implement the public converter without solver coupling**

Compose parser, sanitizer, projection, and validation in `convert_xml`. Export the supported public types from `lib.rs`; do not introduce a dependency on `wellforge-bha-contract` or a CLI command in this task.

- [ ] **Step 4: Run full crate and workspace verification**

Run: `cargo +1.98.0 test --manifest-path engine/Cargo.toml -p wellforge-bha-interchange && cargo +1.98.0 clippy --manifest-path engine/Cargo.toml -p wellforge-bha-interchange --all-targets -- -D warnings && cargo +1.98.0 test --manifest-path engine/Cargo.toml && cargo +1.98.0 fmt --check --manifest-path engine/Cargo.toml`

Expected: PASS.

Then run a repository boundary inspection using deployment-provided restricted token values outside the repository. Confirm that no emitted fixture, JSON output, source, or documentation contains those plaintext values.

- [ ] **Step 5: Commit the end-to-end converter**

```bash
git add engine/crates/wellforge-bha-interchange docs/superpowers/specs/2026-08-31-bha-xml-clean-room-interchange-design.md
git commit -m "feat: convert clean BHA XML to JSON"
```

## Plan Self-Review

| Specification requirement | Planned task |
|---|---|
| Isolated Rust crate | Task 1 |
| Safe ordered XML parsing | Task 2 |
| Restricted-metadata removal | Task 3 |
| Structural JSON preservation | Tasks 2-3 |
| Canonical components and sections | Task 4 |
| Tool-specific details | Task 5 |
| Validation and typed errors | Tasks 1, 2, 4, and 5 |
| End-to-end API and full verification | Task 6 |
| Later solver adapter remains separately specified | Global constraints and Task 6 boundary |

The plan contains no placeholder work items. Public type and function names are introduced before a later task consumes them.
