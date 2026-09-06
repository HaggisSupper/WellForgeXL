# Phase 1 — Foundational Numerical Method Design

## Status
Approved by the project owner on 2026-09-06. New calculation-family development is frozen until this phase is stabilized.

## Goal
Create a narrow, reusable Rust numerical substrate that modernizes shared solver mechanics without changing validated engineering equations unnecessarily. Existing engines become acceptance consumers before new physics is added.

## Non-goals
- No new drilling calculation families during Phase 1.
- No linear/nonlinear BHA dynamic expansion and no 6-DOF BHA work.
- No CFD, PINNs, neural operators, distributed-memory solvers, or GPU-first solver architecture.
- No replacement of stable closed-form trajectory equations.
- No weakening of existing deterministic evidence or acceptance gates.

## Stabilization baseline
The stabilization branch starts from commit `3e3f2c60a7a90fc56e779b9b55af8e63a810c8e7`, which contains the verified 3-D T&D spatial-dogleg correction and backward-compatible T&D hole/solver request contract, but excludes the later unfinished hybrid-result experiment.

## Existing-engine assessment
### Trajectory
Retain the current minimum-curvature, spherical interpolation, target, and survey residual calculations. These are primarily closed-form/vector operations and do not benefit materially from a common nonlinear or sparse refactor. Expose reusable geometry helpers only where another engine needs them.

### BHA static
Refactor dense FE assembly/solve toward sparse assembly and reusable factorization through the common numerical crate. Preserve the Euler-Bernoulli element formulation, geometric stiffness, analytical cantilever oracle, and numerical residual evidence.

### BHA modal / FRF
Remove explicit triangular-matrix inversion from the generalized eigen transformation and prefer factored triangular solves. Assess modal-superposition FRF against the current direct complex-LU implementation while preserving the existing modal/FRF/Campbell scope.

### Torque & Drag
Preserve the verified full spatial dogleg correction. Keep the existing soft-string recurrence as the global baseline. Do not implement the stiff solver until the shared sparse/complementarity primitives exist.

### Hydraulics
Preserve current steady rheology/friction correlations. Do not rewrite them in Phase 1. The future split-flow/segmented-geometry work will consume Phase 1 interval, graph, root-solver, and diagnostic primitives.

## Production crate
Create `engine/crates/wellforge-numerics` as the canonical production numerical API. Keep `wellforge-numerics-acceptance` as an independent third-party-library acceptance harness.

The production crate is deliberately small and separated by responsibility:

- `diagnostics`: common convergence and residual evidence.
- `root`: bracketed scalar root solve and safeguarded Newton for small deterministic nonlinear problems.
- `interval`: deterministic piecewise interval partitioning and boundary intersection.
- `volume`: cumulative piecewise volume coordinate plus deterministic inverse mapping.
- `interpolation`: bounded monotone one-dimensional interpolation for calibration curves.
- `circular`: weighted circular/vector accumulation and coherence statistics.
- `regression`: numerically stable least-squares/robust-calibration primitives using library QR/SVD where practical.
- `sparse`: sparse system assembly/factorization boundary and reusable solve evidence.
- `complementarity`: Fischer-Burmeister residual and semismooth-contact primitives suitable for later stiff-string contact.
- `graph`: conservation-equation primitives for later hydraulic split-flow networks; Phase 1 provides representation/residual utilities, not drilling-domain hydraulics.

## Numerical contracts
### Solver diagnostics
Every iterative primitive returns a stable evidence object containing at minimum:
- convergence state;
- iteration count;
- initial residual norm;
- final residual norm;
- absolute tolerance;
- relative tolerance;
- fallback-used flag.

### Scalar root solve
For a continuous scalar residual `f(x)` with a valid sign-changing bracket `[a,b]`, the bracketed solver must preserve the bracket and converge deterministically or return a typed non-convergence error. Safeguarded Newton may accept a derivative callback but must fall back to the bracket when a Newton step leaves the admissible interval.

### Piecewise intervals
Given multiple ordered boundary sets, produce one sorted, deduplicated partition. No zero-length interval may be emitted. Exact source boundaries are preserved.

### Volume coordinate
For ordered intervals with non-negative cross-sectional area, define cumulative volume `V(x)` and inverse `x(V)`. The mapping must conserve volume to numerical tolerance and handle tapered/piecewise geometry without a spatial mesh or CFL timestep.

### Monotone interpolation
Calibration curves with strictly increasing x values use a monotonicity-preserving interpolation mode that cannot create overshoot outside local data bounds. Invalid/repeated x coordinates are rejected.

### Circular statistics
For weighted toolface vectors, return resultant vector, magnitude, total vector length, and coherence `|sum(v)| / sum(|v|)` with deterministic handling of zero total weight.

### Sparse linear algebra
Engine code should not materialize structurally sparse FE systems as dense matrices solely to solve them. The common boundary must support sparse assembly and residual verification, and allow factorization reuse when the sparsity pattern is unchanged.

### Complementarity
Represent unilateral contact by
`g >= 0`, `lambda >= 0`, `g * lambda = 0`.
Provide the Fischer-Burmeister residual
`phi(g, lambda) = sqrt(g^2 + lambda^2) - g - lambda`.
Phase 1 validates this primitive and its derivatives/finite-difference oracle. Domain contact solvers are deferred until the foundation is green.

## Acceptance strategy
1. Unit tests for each numerical primitive using closed-form or independently calculable cases.
2. `wellforge-numerics-acceptance` extended to consume the production crate rather than merely smoke-test third-party libraries.
3. BHA static migrated to sparse/common solve infrastructure with parity against existing reference fixtures and cantilever oracle.
4. BHA modal refactored to remove explicit inverse while preserving existing mode outputs within defined tolerance.
5. T&D full-spatial-dogleg tests remain green; no stiff physics added in this phase.
6. Full engine workspace `fmt`, Clippy with warnings denied, and tests must pass before Phase 1 is considered stable.

## Architecture rules
- Rust 2024, rust-version 1.98.
- No unsafe code.
- No Python numerical authority.
- No cloud/runtime dependency.
- Deterministic behavior and explicit convergence evidence are mandatory.
- Prefer established Rust numerical libraries (`faer`, `nalgebra`, existing approved dependencies) over bespoke matrix algorithms.
- Do not add a dependency merely to expose a one-line formula.
- Future automatic differentiation must plug into residual interfaces rather than require physics rewrites; AD is not a hard Phase 1 runtime dependency.

## Exit criteria
Phase 1 is complete only when the production numerical crate is implemented, its independent acceptance suite is green, BHA static/modal consume the relevant modernized primitives without regression, T&D spatial-curvature behavior remains green, and the full engine workspace passes formatting, Clippy, and tests.