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
- bounded small-system Newton solving with projected bounds and a backtracking line search (`solve_newton_system`)
- residual/Jacobian provider interfaces (`ResidualModel`, `JacobianProvider`); callers supply Jacobian values, with no automatic differentiation backend
- conservative fixed-capacity volume-space parcel displacement (`ParcelQueue`)
- directed-flow node continuity residuals (`FlowEdge`, `continuity_residuals`)
- robust weighted regression with a deterministic resistant seed and Huber IRLS (`robust_weighted_least_squares`); weighted normal equations use sparse LU
- independent production-numerics acceptance harness

These primitives and interfaces are exported from `wellforge-numerics`; `tests/foundational_phase1.rs` exercises the bounded Newton solve, parcel conservation, flow continuity, and robust regression. Numerical Core remains the shared numerical authority, while domain physics remains in its consuming engines.

## Existing-engine stabilization consumers

- BHA static reduced FE solve routes through `wellforge-numerics` sparse LU while retaining existing dense stiffness/mass result matrices for modal compatibility.
- BHA modal mass normalization uses Cholesky triangular solves and does not construct an explicit matrix inverse.
- Trajectory remains on its existing closed-form/vector numerical path.
- Hydraulics steady correlations remain unchanged pending later graph/segmented-geometry work.
- T&D retains its soft/stiff hybrid path: converged stiff values substitute existing soft stations, and additional stations are inserted only at contact points.

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

## Remaining Phase 1 substrate before feature work resumes

The following approved domain-independent primitives still require implementation or an explicit defer ruling before Phase 1 closes:

- QR/SVD calibration support; the implemented robust regression currently solves weighted normal equations through sparse LU
- final full-workspace formatting, Clippy `-D warnings`, tests, and lockfile gate

These items are numerical infrastructure only; they must not introduce new drilling calculation physics during Phase 1.
