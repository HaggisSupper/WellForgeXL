use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A uniform tubular section confined inside a wellbore.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct BucklingSection {
    pub length_m: f64,
    pub youngs_modulus_pa: f64,
    pub second_moment_m4: f64,
    pub radial_clearance_m: f64,
    pub buoyed_weight_n_per_m: f64,
    pub inclination_rad: f64,
    pub effective_length_factor: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct BucklingLoads {
    pub euler_load_n: f64,
    pub sinusoidal_onset_n: f64,
    pub helical_onset_n: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BucklingRegime {
    Stable,
    Sinusoidal,
    Helical,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum BucklingError {
    #[error("buckling section values must be finite and physically valid")]
    InvalidSection,
    #[error("compressive force must be finite and non-negative")]
    InvalidCompression,
    #[error("buckling calculation exceeds the finite numeric range")]
    NumericalOverflow,
}

/// Computes three explicit stability gates.
///
/// The Euler gate models finite-section column instability. The confined-hole
/// gates use the buoyed lateral loading and radial clearance of the section.
/// Keeping both values visible prevents the solver from silently substituting
/// one buckling criterion for another.
pub fn critical_buckling_loads(section: BucklingSection) -> Result<BucklingLoads, BucklingError> {
    validate(section)?;

    let lateral_weight_n_per_m =
        section.buoyed_weight_n_per_m * section.inclination_rad.sin().abs();
    if !lateral_weight_n_per_m.is_finite() || lateral_weight_n_per_m <= 0.0 {
        return Err(BucklingError::InvalidSection);
    }

    // Evaluate the final thresholds in the log domain so finite results do
    // not fail merely because intermediate stiffness or confinement products
    // overflow.
    let euler_log = 2.0 * std::f64::consts::PI.ln()
        + section.youngs_modulus_pa.ln()
        + section.second_moment_m4.ln()
        - 2.0 * section.effective_length_factor.ln()
        - 2.0 * section.length_m.ln();
    let confined_log = std::f64::consts::LN_2
        + 0.5
            * (section.youngs_modulus_pa.ln()
                + section.second_moment_m4.ln()
                + lateral_weight_n_per_m.ln()
                - section.radial_clearance_m.ln());
    let euler_load_n = euler_log.exp();
    let sinusoidal_onset_n = confined_log.exp();
    let helical_onset_n = (confined_log + 0.5 * std::f64::consts::LN_2).exp();

    if ![euler_load_n, sinusoidal_onset_n, helical_onset_n]
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
        || sinusoidal_onset_n >= helical_onset_n
    {
        return Err(BucklingError::NumericalOverflow);
    }

    Ok(BucklingLoads {
        euler_load_n,
        sinusoidal_onset_n,
        helical_onset_n,
    })
}

pub fn classify_buckling(
    compressive_force_n: f64,
    loads: BucklingLoads,
) -> Result<BucklingRegime, BucklingError> {
    if !compressive_force_n.is_finite()
        || compressive_force_n < 0.0
        || ![
            loads.euler_load_n,
            loads.sinusoidal_onset_n,
            loads.helical_onset_n,
        ]
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
        || loads.sinusoidal_onset_n >= loads.helical_onset_n
    {
        return Err(BucklingError::InvalidCompression);
    }

    let sinusoidal_gate = loads.euler_load_n.min(loads.sinusoidal_onset_n);
    Ok(if compressive_force_n < sinusoidal_gate {
        BucklingRegime::Stable
    } else if compressive_force_n < loads.helical_onset_n {
        BucklingRegime::Sinusoidal
    } else {
        BucklingRegime::Helical
    })
}

fn validate(section: BucklingSection) -> Result<(), BucklingError> {
    let positive = [
        section.length_m,
        section.youngs_modulus_pa,
        section.second_moment_m4,
        section.radial_clearance_m,
        section.effective_length_factor,
    ];
    if !positive
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
        || !section.buoyed_weight_n_per_m.is_finite()
        || section.buoyed_weight_n_per_m < 0.0
        || !section.inclination_rad.is_finite()
    {
        return Err(BucklingError::InvalidSection);
    }
    Ok(())
}
