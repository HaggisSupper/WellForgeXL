//! Value-only BHA result and evidence types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::SourceObjectRef;

/// Overall calculation state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    /// All requested analyses converged.
    Converged,
    /// Result is usable with explicit warnings.
    Warning,
    /// Calculation did not converge or validation failed.
    Failed,
}

/// Centerline and projected-envelope result at one MD.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StaticNodeResult {
    /// MD in metres.
    pub md_m: f64,
    /// Lateral displacement in local high-side direction.
    pub x_m: f64,
    /// Lateral displacement in local cross direction.
    pub y_m: f64,
    /// Tube outside radius in metres.
    pub od_radius_m: f64,
    /// Tube inside radius in metres.
    pub id_radius_m: f64,
    /// Hole radius in metres.
    pub hole_radius_m: f64,
    /// Signed indicated clearance; negative means the projected OD crosses the hole envelope.
    pub projected_clearance_m: f64,
    /// Bending moment magnitude in newton-metres.
    pub bending_moment_n_m: f64,
    /// Bending stress in pascals.
    pub bending_stress_pa: f64,
}

/// Active normal-contact result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPointResult {
    /// MD in metres.
    pub md_m: f64,
    /// Penetration before constraint recovery in metres.
    pub penetration_m: f64,
    /// Recovered normal force in newtons.
    pub normal_force_n: f64,
}

/// One undamped linearized mode.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModeResult {
    /// One-based mode number.
    pub mode_number: usize,
    /// Natural frequency in hertz.
    pub natural_frequency_hz: f64,
    /// Critical rotary speed in revolutions per minute for synchronous excitation.
    pub critical_speed_rpm: f64,
    /// Mass-normalized lateral amplitude by static node.
    pub normalized_shape: Vec<f64>,
}

/// One value-backed frequency-response sample at the lower BHA node.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrequencyResponsePoint {
    /// Excitation frequency in hertz.
    pub frequency_hz: f64,
    /// Response magnitude in metres per newton.
    pub receptance_m_n: f64,
    /// Response phase in degrees.
    pub phase_deg: f64,
}

/// One Campbell-diagram excitation line sample.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CampbellPoint {
    /// Excitation order, e.g. 1x or 3x.
    pub order: usize,
    /// Rotary speed in revolutions per minute.
    pub rpm: f64,
    /// Order frequency in hertz.
    pub excitation_frequency_hz: f64,
    /// Distance to the nearest natural mode in hertz.
    pub nearest_mode_margin_hz: f64,
}

/// Deterministic numerical evidence.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SolverEvidence {
    /// Engine package version.
    pub engine_version: String,
    /// Pinned compiler identity.
    pub compiler: String,
    /// Compilation target family and architecture.
    pub target: String,
    /// SHA-256 of the locked dependency graph.
    pub dependency_lock_hash: String,
    /// SHA-256 of normalized request JSON.
    pub request_hash: String,
    /// SHA-256 of normalized result payload before this field is assigned.
    pub result_hash: String,
    /// Nonlinear iteration count.
    pub iterations: usize,
    /// Final residual norm.
    pub residual_norm: f64,
    /// Whether all convergence criteria passed.
    pub converged: bool,
}

/// Complete value-only analysis result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BhaAnalysisResult {
    /// Contract version used to interpret the payload.
    pub contract_version: String,
    /// Request analysis identity.
    pub analysis_id: Uuid,
    /// Calculation state.
    pub status: AnalysisStatus,
    /// Authoritative source references copied from the request.
    pub sources: Vec<SourceObjectRef>,
    /// Static centerline and projection results.
    pub static_nodes: Vec<StaticNodeResult>,
    /// Active contacts.
    pub contacts: Vec<ContactPointResult>,
    /// Linearized natural modes.
    pub modes: Vec<ModeResult>,
    /// Unit-force lateral receptance frequency sweep.
    pub frequency_response: Vec<FrequencyResponsePoint>,
    /// Deterministic 1x, 2x and 3x Campbell order lines.
    pub campbell: Vec<CampbellPoint>,
    /// Stable warning codes/messages.
    pub warnings: Vec<String>,
    /// Solver evidence and hashes.
    pub evidence: SolverEvidence,
}
