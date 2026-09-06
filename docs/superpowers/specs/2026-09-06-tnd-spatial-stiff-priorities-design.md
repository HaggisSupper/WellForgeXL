# T&D Spatial Curvature and Stiff-String Refinement Design

## Scope

This change closes the two highest-priority torque-and-drag physics gaps identified by the executable-engine audit:

1. Replace the current inclination-only curvature screen with full spatial dogleg curvature using inclination and azimuth.
2. Add a bounded stiff-string/contact refinement path that runs only where the accepted whole-well soft-string result indicates severe contact/buckling conditions and sufficient borehole geometry is supplied.

The existing whole-well soft-string calculation remains authoritative for broad interval coverage and API 7G utilization. Stiff-string results refine selected intervals; they never extrapolate into unrefined intervals and never replace soft-string values silently.

## Contract

### Spatial curvature

No request-schema change is required. `TnDTrajectoryStation` already carries measured depth, inclination and azimuth.

For adjacent stations, calculate spatial dogleg:

`beta = acos(cos(I1) cos(I2) + sin(I1) sin(I2) cos(A2-A1))`

and interval curvature:

`kappa = beta / delta_md`.

The result's existing `dogleg_rad_m` field becomes the full spatial curvature actually used by the solver.

### Hole geometry

Add optional hole sections to `TnDAnalysisRequest` using `#[serde(default)]` so existing 0.1 request fixtures remain readable and byte-compatible when the field is absent.

Each hole section contains stable identity, top/bottom MD and diameter. Validation requires increasing MD bounds, positive finite diameter and non-overlapping ordered coverage.

### Solver options

Add a defaulted solver-options object. Existing requests remain soft-string only when no hole geometry is supplied. The stiff refinement mode is `auto` by default but can run only when hole coverage exists.

Options include:

- `stiff_string_mode`: `disabled | auto`
- `severity_normal_load_n_m`
- `severity_buckling_margin_n`
- `transition_buffer_m`
- `max_element_length_m`
- `contact_penalty_n_m`

All numeric options are finite and positive except the buckling margin threshold, which may be zero.

## Severe-interval classifier

A station is severe when any of these is true:

- `normal_load_n_m >= severity_normal_load_n_m`
- `sinusoidal_margin_n <= severity_buckling_margin_n`
- `helical_margin_n <= severity_buckling_margin_n`

Adjacent severe stations are merged into one interval. Each interval expands by `transition_buffer_m` but is clamped to trajectory coverage. IDs are deterministic UUIDv5 values derived from analysis ID and interval MD bounds.

## Stiff-string solver

The Release-1 stiff refinement is a planar local flexible-beam/contact solve over each selected interval. It is intentionally bounded and does not claim six-degree rigid-body dynamics, impact or whirl.

For each interval:

1. Discretize the covered string into Euler-Bernoulli beam elements no longer than `max_element_length_m`.
2. Use section properties `A = pi(OD^2-ID^2)/4` and `I = pi(OD^4-ID^4)/64`.
3. Assemble elastic stiffness plus geometric stiffness from the inherited local axial compression.
4. Apply distributed transverse loading from buoyed gravity and the soft-string normal-load field.
5. Enforce borehole clearance with a unilateral penalty contact law:
   `F_contact = k_c * max(|x|-clearance, 0)` opposing penetration.
6. Iterate contact force to equilibrium using bounded fixed-point/Newton updates until residual tolerance or iteration limit is reached.
7. Report nodal displacement, contact force, bending moment, bending stress, clearance, residual, iteration count and convergence state.

The current approved numerical stack is reused (`nalgebra`/`faer`); no custom general-purpose matrix library is introduced.

## Result boundary

Add optional `stiff_string` output to `TnDAnalysisResult` with `skip_serializing_if = Option::is_none` so legacy soft-only result bytes do not change.

Each refined interval reports:

- deterministic interval ID
- start/end MD and parent severity reasons
- convergence state
- residual norm and iteration count
- peak contact force and peak bending stress
- ordered nodal states

If geometry is absent or refinement cannot converge, the soft-string result remains complete and a typed warning is emitted. No fabricated stiff values are written.

## Validation

Required gates:

- Exact 3-D dogleg oracle cases: pure build, pure turn at inclination, combined build/turn and wrap-around azimuth.
- Existing soft-string reference fixtures remain unchanged where azimuth is constant.
- Straight centered interval produces zero contact.
- Forced-penetration/contact fixture produces non-negative contact forces and positive clearance recovery.
- Stiff solver residual and convergence are deterministic.
- Missing hole geometry produces soft-only output without failure.
- `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- `cargo test --workspace --all-features --locked`.

## Non-goals

- No full 3-D Cosserat rod in this release.
- No frictional impact or whirl.
- No extrapolation of stiff contact loads outside selected intervals.
- No replacement of the whole-well soft-string pass.
