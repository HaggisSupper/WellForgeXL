# Clean-Room BHA XML Interchange Design

## Objective

Add an independent Rust BHA interchange subsystem that converts neutral BHA XML into two JSON views without importing, linking to, copying, or depending on legacy implementation code. The subsystem preserves the non-proprietary engineering structure and removes restricted legacy identity metadata before any persisted output is produced.

## Scope

The first delivery covers the neutral XML shapes represented by independent BHA data examples:

- assembly metadata and ordered components;
- generic components and ordered tubular sections;
- bit, mud-motor, rotary-steerable-system, and stabilizer detail blocks;
- dimensions, mass/weight, material, wear, counts, and section classifications.

The first delivery does not reproduce legacy application behavior, graphical UI, project management, hydraulics, or solver algorithms. Existing WellForge BHA solver behavior remains authoritative and unchanged until a later adapter phase has an independently approved specification.

## Clean-Room Constraints

- Treat the source examples strictly as data-format observations, never as implementation source.
- Do not copy source code, comments, naming conventions, proprietary metadata, screenshots, or identifiers into WellForge.
- Do not add legacy-vendor, legacy-product, or legacy-version identifiers to tracked WellForge source, fixtures, JSON output, documentation, executable output, or test names.
- The importer must remove restricted identifiers from element names, attributes, scalar text values, and generated JSON keys before serialization.
- The restricted-identifier policy is supplied to the importer as SHA-256 token fingerprints from deployment configuration. The plaintext policy values are never committed to the repository.
- Test fixtures use invented neutral names and generic restricted-token cases only.

## Architecture

Add the workspace member `engine/crates/wellforge-bha-interchange`. It depends only on workspace `quick-xml`, `serde`, `serde_json`, `sha2`, `thiserror`, and `uuid` crates. It has no dependency on the legacy data location and initially has no dependency on the BHA solver crates.

The crate is divided by responsibility:

| Module | Responsibility |
|---|---|
| `xml_tree` | Safe, ordered XML parsing into a generic node tree. |
| `sanitize` | Removes nodes, attributes, and scalar values matching policy fingerprints. |
| `structural_json` | Serializes the sanitized generic tree without losing element order or nesting. |
| `model` | Defines neutral typed assembly, component, section, and tool-detail types. |
| `project` | Projects supported neutral nodes into canonical BHA engineering data. |
| `validate` | Enforces data quality and domain geometry constraints. |
| `error` | Defines typed parse, policy, projection, and validation errors. |

## Data Flow

```text
XML bytes
  -> safe XML tree parser
  -> restricted-identifier sanitizer
  -> structural JSON document
  -> typed neutral BHA projection
  -> canonical BHA JSON document
```

The XML parser disables DTD processing and external entity resolution. Every element preserves its original order, attributes, text, and child-node ordering in `StructuralNode`:

```rust
pub struct StructuralNode {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<StructuralNode>,
}
```

The structural JSON is a direct `serde_json` serialization of this tree. It deliberately retains source nesting rather than flattening repeated elements into inferred maps.

## Canonical Model

```rust
pub struct BhaAssembly {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub static_analysis_enabled: bool,
    pub vibration_analysis_enabled: bool,
    pub components: Vec<BhaComponentRecord>,
}

pub struct BhaComponentRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub quantity: u32,
    pub material: Option<String>,
    pub kind: ComponentKind,
    pub wear_count: u32,
    pub wear_od_m: Option<f64>,
    pub sections: Vec<TubularSection>,
    pub detail: Option<ComponentDetail>,
}

pub struct TubularSection {
    pub kind: String,
    pub od_m: f64,
    pub id_m: f64,
    pub length_m: f64,
    pub mass_kg: Option<f64>,
    pub material: Option<String>,
    pub classification: Option<String>,
}
```

`ComponentDetail` is a tagged enum for the three supported tool families. Motor data includes geometry, bend angle, lobe configuration, blade counts, and optional subassemblies. Rotary-steerable data includes collar geometry, pad geometry, and steering mode. Stabilizer data includes blade count, gauge diameter, and sub-lengths. Unknown but neutral XML remains available in structural JSON and produces an explicit projection warning instead of being silently reinterpreted.

## Sanitization Policy

`SanitizationPolicy` receives precomputed, lowercase SHA-256 fingerprints. The sanitizer tokenizes Unicode text at non-alphanumeric boundaries, hashes each token, and removes a node or attribute if either its name or any scalar value matches a configured fingerprint. A removal report records only counts by category: elements, attributes, values, and output paths. It never includes removed input text.

The same policy is applied before both structural serialization and typed projection. Therefore canonical JSON cannot reintroduce content excluded from structural JSON.

## Validation and Errors

Reject an input when:

- the document is malformed, uses a DTD, or has an unsupported root;
- a required assembly/component/section field is missing or unparseable;
- quantity is zero;
- section length is zero or negative;
- OD, ID, or mass is negative;
- OD is less than or equal to ID;
- a supported tool detail block is structurally inconsistent with its component kind.

Unknown neutral component kinds are retained structurally and projected as `ComponentKind::Other`; unknown fields do not cause data loss. Restricted nodes are removed, not surfaced as errors, unless removal empties a required field; that condition is a validation error.

## Test Strategy

Tests are crate-local and use newly authored neutral XML fixtures:

1. Parse an assembly containing ordered generic, bit, motor, rotary-steerable, stabilizer, and tubular components.
2. Assert exact structural node and repeated-component order in JSON.
3. Project each supported detail block into canonical JSON.
4. Verify a configured generic restricted token removes an element, an attribute, and a scalar value without exposing that value in the report.
5. Assert DTD, malformed XML, missing required fields, non-positive length, and `OD <= ID` fail with the intended typed error.
6. Assert the tracked crate fixtures and generated JSON contain no deployment-specific restricted identifier values.

## Integration Roadmap

1. Add the isolated interchange crate, parser, sanitizer, and structural JSON tests.
2. Add typed projection and canonical JSON tests.
3. Specify a separate adapter from `BhaComponentRecord` to `wellforge-bha-contract::BhaComponent`, including the existing workbook support-factor gap.
4. Specify any solver or calculation port as separate engineering work with independent acceptance tests.

## Acceptance Criteria

- A neutral BHA XML document converts deterministically to structural JSON and canonical BHA JSON.
- Component and section ordering are preserved.
- Supported tool-specific data is typed without flattening or loss.
- Invalid XML and invalid geometry are rejected with typed errors.
- Restricted identity metadata is absent from both JSON views and from all tracked WellForge artifacts.
- The crate compiles, its test suite passes, and the existing engine workspace tests remain green.
