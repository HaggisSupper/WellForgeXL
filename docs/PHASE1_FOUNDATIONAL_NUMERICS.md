# Phase 1 — Foundational Numerical Method

Status: active stabilization; new drilling-calculation feature work is frozen until this phase clears workspace verification.

## Implemented production numerical primitives

- common solver diagnostics
- bracketed and safeguarded scalar root solving
- piecewise interval partitioning
- cumulative volume coordinate and inverse mapping
- monotone one-dimensional interpolation
- weighted circular/vector statistics
- Fischer–Burmeister complementarity residual and smoothed gradient
- reusable sparse linear systems with cached symbolic LU analysis and numerical refactorization
- independent production-numerics acceptance harness

## Existing-engine stabilization consumers

- BHA static reduced FE solve routes through `wellforge-numerics` sparse LU while retaining existing dense stiffness/mass result matrices for modal compatibility.
- BHA modal mass normalization uses Cholesky triangular solves and does not construct an explicit matrix inverse.
- Trajectory remains on its existing closed-form/vector numerical path.
- Hydraulics steady correlations remain unchanged pending later graph/segmented-geometry work.

## Canonical contract governance

- canonical dotted contract IDs
- semantic-version compatibility policies (`Exact`, `SameMajor`, `ExplicitSet`)
- malformed/unsupported version rejection before domain calculation
- deterministic JSON normalization
- SHA-256 request/result schema fingerprints
- centralized registry generated from the actual Rust request/result schema types
- registry conflict detection

Current registry families:

- `wellforge.trajectory.analysis` — canonical `1.0.0`, same-major compatibility
- `wellforge.bha.analysis` — canonical `1.0.0`, same-major compatibility
- `wellforge.torque_drag.analysis` — canonical `0.1.0`, exact compatibility
- `wellforge.hydraulics.analysis` — explicit `0.1.0` and `0.2.0`

## Consolidated substrate and remaining limits

The numerical branch now includes bounded damped Newton, conservative parcel bookkeeping,
flow continuity residuals, robust Huber regression and explicit residual/Jacobian interfaces.
These are numerical primitives, not completed thermal, contact or hydraulic-network models.
Robust regression uses normal equations rather than QR/SVD. Automatic differentiation is
not implemented by the residual/Jacobian interface alone. Sparse BHA retains dense matrices
for modal compatibility. Platform release acceptance remains separate from source checks.

See `docs/consolidation/README.md` for the integration and verification ledger.
