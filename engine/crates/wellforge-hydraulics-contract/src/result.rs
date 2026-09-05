//! Hydraulics result types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::{ComputeBackend, FlowCorrelation, FlowLoop, ThermalAssumption};

/// Overall analysis status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    /// All checks passed within envelope.
    Ok,
    /// One or more checks issued warnings; consult `warnings`.
    Warning,
    /// One or more checks failed.
    Failed,
}

/// Hydraulic flow regime reported by the selected correlation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowRegime {
    /// Viscous forces govern the correlation.
    #[default]
    Laminar,
    /// The correlation blends laminar and turbulent responses.
    Transitional,
    /// Inertial forces govern the correlation.
    Turbulent,
}

/// Per-section pressure loss and regime indication.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SectionPressureLoss {
    /// Section identity.
    pub section_id: Uuid,
    /// Whether this loss is on the pipe side or the annulus side.
    pub flow_loop: FlowLoop,
    /// Bulk velocity in metres per second.
    pub bulk_velocity_m_s: f64,
    /// Generalised Reynolds number for the selected rheology.
    pub reynolds_number: f64,
    /// Fanning friction factor.
    pub fanning_friction_factor: f64,
    /// Flow regime determined by the selected correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_regime: Option<FlowRegime>,
    /// Section pressure loss in pascals.
    pub pressure_loss_pa: f64,
}

/// Solver evidence.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsSolverEvidence {
    /// Engine build version.
    pub engine_version: String,
    /// Normalized SHA-256 of the request JSON.
    pub request_hash: String,
    /// Normalized SHA-256 of the result JSON (excluding this field).
    pub result_hash: String,
    /// Reported standard profile identifier the results were computed against.
    pub profile_standard: String,
    /// Reported edition identifier.
    pub profile_edition: String,
    /// Correlation used to compute friction losses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_correlation: Option<FlowCorrelation>,
    /// Compute backend requested for independent section evaluations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_backend: Option<ComputeBackend>,
    /// Temperature treatment used while evaluating fluid properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal_assumption: Option<ThermalAssumption>,
}

/// Top-level hydraulics analysis result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsAnalysisResult {
    /// Semver contract version.
    pub contract_version: String,
    /// Echoed analysis identity.
    pub analysis_id: Uuid,
    /// Overall status.
    pub status: AnalysisStatus,
    /// Total pipe-side pressure drop in pascals.
    pub total_pipe_pressure_loss_pa: f64,
    /// Total annulus-side pressure drop in pascals.
    pub total_annulus_pressure_loss_pa: f64,
    /// Pressure drop across the bit nozzles in pascals.
    pub bit_pressure_loss_pa: f64,
    /// Total flow area of the nozzles in square metres.
    pub total_flow_area_m2: f64,
    /// Equivalent circulating density at total depth in kilograms per cubic metre.
    pub equivalent_circulating_density_kg_m3: f64,
    /// TVD used as the ECD hydrostatic reference in metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_vertical_depth_m: Option<f64>,
    /// Applied annulus surface backpressure in pascals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_backpressure_pa: Option<f64>,
    /// Nozzle discharge coefficient used in the bit pressure calculation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_discharge_coefficient: Option<f64>,
    /// Pipe, bit, annulus and surface-backpressure sum in pascals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circulating_system_pressure_pa: Option<f64>,
    /// Per-section losses in request order, with pipe then annulus for each section.
    pub sections: Vec<SectionPressureLoss>,
    /// Solver evidence.
    pub evidence: HydraulicsSolverEvidence,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}
