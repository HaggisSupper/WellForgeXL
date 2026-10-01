# Phase 1 — Canonical Contract Governance Addendum

## Status
Approved by the project owner on 2026-09-06 as part of Phase 1 — Foundational Numerical Method. This addendum extends the Phase 1 exit criteria; it does not resume new calculation-family work.

## Goal
Standardize versioned canonical contracts across every WellForge engine, process, persistence, and interchange boundary while keeping internal numerical primitives domain-independent.

## Design rule
Externally observable engineering data crosses boundaries only through a versioned canonical request/result contract:

`canonical request -> validation -> engine/domain adapter -> internal numerical primitives -> canonical result`

Internal numerical types such as `SolverDiagnostics`, `Interval`, `VolumeSegment`, sparse matrices, root solutions, and complementarity residuals are governed by the numerical crate API version and do not carry domain `contract_version` fields.

## Dependency architecture
Create two domain-independent governance layers to avoid contract dependency cycles:

1. `wellforge-contract-governance`
   - owns `ContractId`, SemVer compatibility rules, supported-version validation, canonical schema normalization, SHA-256 schema fingerprinting, and common governance errors;
   - depends only on generic serialization/schema/version/hash libraries;
   - does not depend on any trajectory/BHA/T&D/hydraulics contract crate.

2. `wellforge-contract-registry`
   - depends on the governance crate plus the engine contract crates;
   - aggregates the complete canonical descriptor set;
   - detects duplicate IDs/versions, conflicting fingerprints, missing request/result descriptors, and unsupported compatibility declarations;
   - is an audit/verification surface, not a dependency of the domain contracts.

Each engine contract crate depends on `wellforge-contract-governance` and exposes immutable descriptors for its request and result schemas.

## Canonical identifiers
Initial contract families:

- `wellforge.trajectory.analysis`
- `wellforge.bha.analysis`
- `wellforge.torque_drag.analysis`
- `wellforge.hydraulics.analysis`

A descriptor contains at minimum:

- canonical `ContractId`;
- semantic version;
- compatibility policy;
- request schema fingerprint;
- result schema fingerprint;
- request/result schema kind;
- stable human-readable title.

## SemVer policy
- Every boundary contract version must parse as valid SemVer.
- Supported versions are explicit per contract family.
- Major-version compatibility is never inferred merely from a dotted string prefix.
- Unsupported major versions are rejected before engine execution.
- Minor/patch compatibility must be declared by the owning contract crate rather than assumed globally.
- Hydraulics may continue to support both `0.1.0` and `0.2.0`, but both must be explicitly registered.
- Trajectory and BHA Release 1 accept registered major-1 versions only according to their descriptor policy.
- T&D must stop accepting arbitrary non-empty version strings and move to the same explicit SemVer policy.

## Schema fingerprinting
Canonical request/result schemas are generated with `schemars`, normalized deterministically, serialized to canonical JSON, and fingerprinted with SHA-256.

The fingerprint contract must ensure:

- object-key ordering does not change the fingerprint;
- semantically identical generated schemas produce the same fingerprint;
- any structural schema change changes the fingerprint unless the normalized representation is identical;
- a registered `(ContractId, semantic_version, schema_kind)` cannot map to two different fingerprints.

The registry verifies these invariants at test/build time.

## Compatibility rules
Compatibility is explicit metadata, not inferred solely from SemVer text. At minimum support:

- `Exact`: only the exact registered version is accepted;
- `SameMajor`: registered major version with compatible minor/patch policy;
- `ExplicitSet`: a fixed set of accepted versions, used where compatibility cannot safely be generalized.

The engine contract owns its chosen policy.

## Entry-point enforcement
Every CLI/service/Excel bridge entry point must perform, in order:

1. parse canonical `ContractId`/contract family from its own fixed engine context;
2. parse `contract_version` as SemVer;
3. resolve the corresponding registered descriptor/policy;
4. reject unsupported versions;
5. run domain contract validation;
6. execute engine physics;
7. emit a result carrying the exact accepted contract version and immutable evidence.

No caller-supplied version string may bypass registry/policy enforcement.

## Existing-engine stabilization requirements
### Trajectory
Retain current SemVer-major validation behavior but route it through governance primitives. Preserve existing `1.0.0` fixtures and outputs.

### BHA
Replace string-prefix major checking with real SemVer parsing and governance policy resolution. Preserve Release 1 behavior.

### Torque & Drag
Replace the current non-empty-only `contract_version` check with explicit SemVer + supported-version enforcement. Preserve the verified spatial-dogleg and request-contract changes already on the stabilization baseline.

### Hydraulics
Register and preserve `0.1.0` and `0.2.0`; route current explicit version checks through governance policy without changing calculation behavior.

## Acceptance
Phase 1 contract-governance acceptance requires:

- unit tests for `ContractId`, SemVer parsing, exact/same-major/explicit-set compatibility;
- deterministic schema-normalization/fingerprint tests;
- registry uniqueness/conflict tests;
- trajectory, BHA, T&D, and hydraulics contract tests proving supported versions pass and unsupported/malformed versions fail;
- CLI/bridge tests proving unsupported versions are rejected before calculation;
- schema fingerprints checked into deterministic evidence or generated fixtures where appropriate;
- full workspace formatting, Clippy `-D warnings`, and tests green.

## Exit criteria extension
Phase 1 is not complete until both the numerical-foundation design and this canonical-contract-governance addendum are implemented and verified. No new calculation-family development resumes before both are green.