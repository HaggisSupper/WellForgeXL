# WellForge numerical dependencies

Release 1 delegates general numerical operations to locked Rust crates:

| Capability | Dependency | Locked selection |
|---|---|---|
| Spatial vectors, transforms, quaternions and symmetric eigenpairs | `nalgebra` | 0.34.2 |
| Dense/sparse matrix storage, factorization and linear solves | `faer` | 0.24.4 |
| Collision and signed-distance queries | `parry3d-f64` | 0.30.2 |
| Nonlinear least-squares/root solution | `levenberg-marquardt` | 0.15.0 |

`levenberg-marquardt` replaces the plan's provisional `newton_rootfinder` candidate because it is maintained, uses the same `nalgebra` backend as the spatial model, exposes explicit residual/Jacobian contracts and convergence reports, and supports derivative checking. WellForge supplies drilling residuals and tangents; it does not reproduce the library iteration algorithm.

The audit policy blocks vulnerabilities, yanked crates, unknown registries and wildcard dependencies. It records one narrow unmaintained-only exception, `RUSTSEC-2024-0436`, for the transitive `paste` macro used by the current `faer`/`nalgebra` graph; the advisory reports no vulnerability and no safe upgrade. WITSML XML parsing uses `quick-xml >= 0.41.0`, which resolves `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195`.

The acceptance crate exercises each operation independently of drilling physics. `cargo-deny` remains the authoritative dependency-license gate; the runtime report only records that the packaged build passed that external policy.

Official references:

- https://docs.rs/nalgebra/
- https://docs.rs/faer/
- https://docs.rs/parry3d-f64/
- https://docs.rs/levenberg-marquardt/
