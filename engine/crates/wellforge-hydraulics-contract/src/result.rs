//! Hydraulics result types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::FlowLoop;

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
    /// Per-section losses (pipe entries first, annulus entries second).
    pub sections: Vec<SectionPressureLoss>,
    /// Solver evidence.
    pub evidence: HydraulicsSolverEvidence,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}
