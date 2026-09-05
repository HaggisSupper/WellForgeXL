//! Steady-state soft-string torque-and-drag calculation primitives.
//!
//! # Mathematical scope
//!
//! All inputs and outputs use SI units: metres, radians, newtons,
//! newton-metres, pascals, and metres to the fourth power. Inputs must be
//! finite. Length, stiffness, second moment, radial clearance, and effective
//! length factor are strictly positive where used; buoyed weight, friction,
//! and contact radius are non-negative. Negative zero is rejected for buoyed
//! weight, friction, and contact radius so reported torque never carries a
//! negative-zero sign.
//! Calculations that exceed the finite numeric range return an error rather
//! than reporting an infinite load or torque.
//!
//! The soft-string calculation treats each segment independently. For a
//! segment with buoyed weight per length `w`, length `L`, inclination `theta`,
//! and friction factor `mu`, it uses:
//!
//! - axial gravity contribution: `w L cos(theta)`;
//! - contact normal-force magnitude: `w L |sin(theta)|`; and
//! - drag magnitude: `mu w L |sin(theta)|`.
//!
//! Segment results accumulate from the first supplied segment toward the last.
//! `RunInHole` subtracts drag from gravity, `PullOutOfHole` and `Backream` add
//! drag, and `Rotate` retains only gravity. Torque is the non-negative
//! magnitude `drag * contact_radius` and is reported only for `Rotate` and
//! `Backream`.
//!
//! This is a steady-state screening model. It does not represent string
//! elasticity, trajectory-curvature contact forces, changing contact state,
//! hydraulic effects, transient dynamics, or coupled torque/axial response.
//!
//! # Buckling scope
//!
//! For a uniform finite section, the module reports the Euler threshold
//! `P_e = pi^2 E I / (K L)^2`, plus confined sinusoidal and helical onsets:
//! `P_s = 2 sqrt(E I q / c)` and `P_h = sqrt(2) P_s`, where
//! `q = w |sin(theta)|` is lateral buoyed loading and `c` is radial clearance.
//! A section must have positive lateral buoyed loading for this confined model;
//! a zero value has degenerate confined thresholds and is rejected.
//!
//! Classification is `Stable` below `min(P_e, P_s)`, `Sinusoidal` from that
//! gate up to (but excluding) `P_h`, and `Helical` at or above `P_h`.
//! These thresholds are a uniform-section stability screen, not a substitute
//! for a full contact, material, or deployment analysis.

mod buckling;

pub use buckling::{
    BucklingError, BucklingLoads, BucklingRegime, BucklingSection, classify_buckling,
    critical_buckling_loads,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct StringSegment {
    pub length_m: f64,
    pub inclination_rad: f64,
    pub buoyed_weight_n_per_m: f64,
    pub friction_factor: f64,
    pub contact_radius_m: f64,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    RunInHole,
    PullOutOfHole,
    Rotate,
    Backream,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SegmentResult {
    pub axial_force_n: f64,
    pub drag_n: f64,
    pub torque_nm: f64,
}
#[derive(Debug, Error)]
pub enum TndError {
    #[error("segment properties must be finite and physically valid")]
    InvalidSegment,
    #[error("at least one segment is required")]
    EmptyString,
    #[error("segment calculation exceeds the finite numeric range")]
    NumericalOverflow,
}

/// Integrates a deterministic soft-string load path from bit to surface.
pub fn solve_soft_string(
    segments: &[StringSegment],
    operation: Operation,
) -> Result<Vec<SegmentResult>, TndError> {
    if segments.is_empty() {
        return Err(TndError::EmptyString);
    }
    let mut axial_force_n = 0.0;
    let mut output = Vec::with_capacity(segments.len());
    for segment in segments {
        validate(*segment)?;
        let normal_force_n =
            segment.buoyed_weight_n_per_m * segment.length_m * segment.inclination_rad.sin().abs();
        let drag_n = segment.friction_factor * normal_force_n;
        let gravity_n =
            segment.buoyed_weight_n_per_m * segment.length_m * segment.inclination_rad.cos();
        let axial_increment_n = match operation {
            Operation::RunInHole => gravity_n - drag_n,
            Operation::PullOutOfHole => gravity_n + drag_n,
            Operation::Rotate => gravity_n,
            Operation::Backream => gravity_n + drag_n,
        };
        axial_force_n += axial_increment_n;
        let torque_nm = if matches!(operation, Operation::Rotate | Operation::Backream) {
            drag_n * segment.contact_radius_m
        } else {
            0.0
        };
        if ![
            normal_force_n,
            drag_n,
            gravity_n,
            axial_increment_n,
            axial_force_n,
            torque_nm,
        ]
        .iter()
        .all(|value| value.is_finite())
        {
            return Err(TndError::NumericalOverflow);
        }
        output.push(SegmentResult {
            axial_force_n,
            drag_n,
            torque_nm,
        });
    }
    Ok(output)
}
fn validate(segment: StringSegment) -> Result<(), TndError> {
    if [
        segment.length_m,
        segment.inclination_rad,
        segment.buoyed_weight_n_per_m,
        segment.friction_factor,
        segment.contact_radius_m,
    ]
    .iter()
    .all(|v| v.is_finite())
        && segment.length_m > 0.0
        && segment.buoyed_weight_n_per_m >= 0.0
        && segment.friction_factor >= 0.0
        && segment.contact_radius_m >= 0.0
        && !is_negative_zero(segment.buoyed_weight_n_per_m)
        && !is_negative_zero(segment.friction_factor)
        && !is_negative_zero(segment.contact_radius_m)
    {
        Ok(())
    } else {
        Err(TndError::InvalidSegment)
    }
}

fn is_negative_zero(value: f64) -> bool {
    value == 0.0 && value.is_sign_negative()
}
