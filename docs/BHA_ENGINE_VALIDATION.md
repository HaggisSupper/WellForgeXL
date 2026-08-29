# BHA Rust Engine Release 1 validation

## Implemented calculation boundary

Release 1 is a deterministic, small-deflection lateral Euler–Bernoulli beam engine. It assembles consistent stiffness/mass matrices, applies buoyed transverse gravity from the WITSML-aligned trajectory inclination, applies compressive geometric stiffness from WOB, solves static displacement, and projects OD/ID through the wellbore envelope. Negative clearance is an interference indication only; no contact reaction is reported.

The linearized dynamic lane returns undamped natural modes, direct complex unit-force receptance with 2% stiffness-proportional damping, critical synchronous RPM, and 1x/2x/3x Campbell order margins.

## Numerical ownership

| Capability | Library |
|---|---|
| Dense linear solve | `faer` |
| Cholesky/eigen/complex LU | `nalgebra` |
| Spatial frames/quaternions | `nalgebra` |
| Cylinder projection query | `parry3d-f64` |
| Nonlinear solver acceptance boundary | `levenberg-marquardt` |
| Dimension-safe SI conversion | `uom` |

WellForge code owns section properties, beam-element assembly, geometric stiffness, buoyancy/loading, result interpretation and drilling-specific contracts. It does not implement matrix factorizations, eigensolvers, nonlinear iteration, quaternions or geometry kernels.

## Automated evidence

- Cantilever tip displacement matches the closed form to relative tolerance `1e-12`.
- Centered hole clearance matches the dimensional radius difference to `1e-12 m`.
- A vertical BHA has zero transverse gravity sag.
- Compressive WOB lowers the first lateral natural frequency.
- Direct complex FRF peaks within 10% of the first eigenfrequency.
- Modes are positive and sorted.
- Static force residual is normalized and recorded in result evidence.
- Request/result hashes, compiler identity, target, dependency-lock hash and source references are returned with every result.
- Rust formatting, Clippy `-D warnings`, workspace tests, and `cargo-deny` licenses/bans/sources pass.

## Applicability limits

Release 1 is not the full Universal Mechanisms-style rigid-body/contact dynamics target. It does not yet solve six-degree rigid joints, nonlinear normal contact, frictional impact, whirl, bit/formation interaction, coupled torsion/axial response, transient integration or DAT-calibrated forcing. The immutable contract already distinguishes `rigid`, `beam` and `modal_flexible` representations so those capabilities can be added under a new compatible contract revision without re-keying WITSML sources.

Desktop Excel compilation, macro execution and screenshot acceptance remain Windows-only gates in `Build-WellForgeVbaSuite.ps1`.
