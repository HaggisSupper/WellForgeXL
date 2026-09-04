//! Steady-state drilling-hydraulics primitives in SI units.
//!
//! This module applies Darcy-Weisbach pressure loss with a caller-supplied
//! Darcy friction factor and derives equivalent circulating density from the
//! resulting pressure loss. It is applicable to a fixed-condition,
//! single-phase calculation with finite SI inputs. It does not include
//! transient or non-Newtonian effects, friction-factor determination, or
//! transport modeling.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
/// A flow section with a caller-supplied Darcy friction factor.
pub struct FlowSection {
    pub length_m: f64,
    pub hydraulic_diameter_m: f64,
    pub flow_area_m2: f64,
    /// Dimensionless Darcy friction factor.
    pub friction_factor: f64,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Fluid {
    pub density_kg_m3: f64,
    pub viscosity_pa_s: f64,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HydraulicsResult {
    pub flow_rate_m3_s: f64,
    pub velocity_m_s: f64,
    pub pressure_loss_pa: f64,
    pub ecd_kg_m3: f64,
}
#[derive(Debug, Error)]
pub enum HydraulicsError {
    #[error(
        "hydraulics inputs and calculated results must be finite and strictly positive where required"
    )]
    InvalidInput,
}

/// Calculates Darcy-Weisbach pressure loss and annular equivalent circulating density.
///
/// The supplied Darcy friction factor is used directly. Inputs and calculated
/// outputs must be finite; otherwise this function returns [`HydraulicsError::InvalidInput`].
pub fn calculate_steady_hydraulics(
    section: FlowSection,
    fluid: Fluid,
    flow_rate_m3_s: f64,
    true_vertical_depth_m: f64,
) -> Result<HydraulicsResult, HydraulicsError> {
    if ![
        section.length_m,
        section.hydraulic_diameter_m,
        section.flow_area_m2,
        section.friction_factor,
        fluid.density_kg_m3,
        fluid.viscosity_pa_s,
        flow_rate_m3_s,
        true_vertical_depth_m,
    ]
    .iter()
    .all(|value| value.is_finite())
        || section.length_m < 0.0
        || section.hydraulic_diameter_m <= 0.0
        || section.flow_area_m2 <= 0.0
        || section.friction_factor < 0.0
        || fluid.density_kg_m3 <= 0.0
        || fluid.viscosity_pa_s <= 0.0
        || flow_rate_m3_s < 0.0
        || true_vertical_depth_m <= 0.0
    {
        return Err(HydraulicsError::InvalidInput);
    }
    let velocity_m_s = flow_rate_m3_s / section.flow_area_m2;
    let pressure_loss_pa = section.friction_factor
        * (section.length_m / section.hydraulic_diameter_m)
        * (fluid.density_kg_m3 * velocity_m_s.powi(2) / 2.0);
    let ecd_kg_m3 = fluid.density_kg_m3 + pressure_loss_pa / (9.80665 * true_vertical_depth_m);
    if ![velocity_m_s, pressure_loss_pa, ecd_kg_m3]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(HydraulicsError::InvalidInput);
    }
    Ok(HydraulicsResult {
        flow_rate_m3_s,
        velocity_m_s,
        pressure_loss_pa,
        ecd_kg_m3,
    })
}
