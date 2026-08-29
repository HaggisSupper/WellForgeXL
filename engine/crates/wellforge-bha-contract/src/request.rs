//! Calculation-authoritative request types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::SourceObjectRef;

/// Mechanical representation selected for a component.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRepresentation {
    /// Six-degree rigid segment.
    Rigid,
    /// Flexible beam segment.
    Beam,
    /// Flexible segment backed by a supplied modal basis.
    ModalFlexible,
}

/// One ordered tubular/BHA component in canonical SI.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BhaComponent {
    /// Stable component identity.
    pub id: Uuid,
    /// Citation name only.
    pub name: String,
    /// Mechanical representation.
    pub representation: ComponentRepresentation,
    /// Component top measured depth in metres.
    pub top_md_m: f64,
    /// Component bottom measured depth in metres.
    pub bottom_md_m: f64,
    /// Outside diameter in metres.
    pub od_m: f64,
    /// Inside diameter in metres.
    pub id_m: f64,
    /// Young's modulus in pascals.
    pub youngs_modulus_pa: f64,
    /// Material density in kilograms per cubic metre.
    pub density_kg_m3: f64,
}

/// Piecewise-constant wellbore diameter section.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HoleSection {
    /// Top MD in metres.
    pub top_md_m: f64,
    /// Bottom MD in metres.
    pub bottom_md_m: f64,
    /// Hole diameter in metres.
    pub diameter_m: f64,
}

/// WITSML-aligned trajectory station used to orient gravity relative to the BHA.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrajectoryStation {
    /// Measured depth in metres.
    pub md_m: f64,
    /// Inclination from vertical in radians.
    pub inclination_rad: f64,
    /// Azimuth in radians.
    pub azimuth_rad: f64,
}

/// Operating point applied to a BHA analysis.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatingPoint {
    /// Weight on bit in newtons.
    pub wob_n: f64,
    /// Rotary speed in revolutions per minute, preserved for reporting.
    pub rpm: f64,
    /// Fluid density in kilograms per cubic metre.
    pub fluid_density_kg_m3: f64,
}

/// Deterministic solver controls.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SolverSettings {
    /// Maximum beam element length.
    pub max_element_length_m: f64,
    /// Maximum nonlinear iterations.
    pub max_iterations: usize,
    /// Residual convergence tolerance.
    pub residual_tolerance: f64,
    /// Contact penalty stiffness in newtons per metre.
    pub contact_penalty_n_m: f64,
    /// Number of requested modes.
    pub requested_modes: usize,
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self {
            max_element_length_m: 0.5,
            max_iterations: 80,
            residual_tolerance: 1.0e-8,
            contact_penalty_n_m: 1.0e9,
            requested_modes: 8,
        }
    }
}

/// Versioned BHA analysis request.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BhaAnalysisRequest {
    /// Semantic contract version; Release 1 accepts major version 1 only.
    pub contract_version: String,
    /// Stable analysis identity.
    pub analysis_id: Uuid,
    /// Authoritative WITSML source references.
    pub sources: Vec<SourceObjectRef>,
    /// Ordered trajectory stations projected from WITSML.
    pub trajectory: Vec<TrajectoryStation>,
    /// Ordered mechanical component path.
    pub components: Vec<BhaComponent>,
    /// Ordered wellbore geometry sections.
    pub hole: Vec<HoleSection>,
    /// Applied operating point.
    pub operating: OperatingPoint,
    /// Solver controls.
    pub solver: SolverSettings,
}
