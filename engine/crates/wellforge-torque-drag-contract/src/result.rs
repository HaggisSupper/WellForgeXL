//! Torque-and-drag result types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Overall analysis status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    /// All checks passed within envelope.
    Ok,
    /// One or more checks issued warnings; consult `checks`.
    Warning,
    /// One or more checks failed.
    Failed,
}

/// Reason a soft-string station or interval requires stiff-string refinement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StiffIntervalReason {
    /// Soft-string normal load meets or exceeds the configured severity threshold.
    NormalLoad,
    /// Sinusoidal buckling margin meets or falls below the configured threshold.
    SinusoidalBuckling,
    /// Helical buckling margin meets or falls below the configured threshold.
    HelicalBuckling,
}

/// Deterministic severe interval selected from the accepted soft-string result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StiffIntervalCandidate {
    /// Stable `UUIDv5` derived from analysis identity and interval bounds.
    pub id: Uuid,
    /// Buffered interval start measured depth in metres.
    pub start_md_m: f64,
    /// Buffered interval end measured depth in metres.
    pub end_md_m: f64,
    /// Stable ordered reasons that caused selection.
    pub reasons: Vec<StiffIntervalReason>,
}

/// Stiff-string equilibrium convergence state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StiffConvergence {
    /// The bounded contact equilibrium reached its residual tolerance.
    Converged,
    /// The solver exhausted its iteration bound without satisfying tolerance.
    NotConverged,
}

/// One nodal state from a bounded stiff-string interval refinement.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StiffNodeResult {
    /// Measured depth in metres.
    pub md_m: f64,
    /// Effective axial tension inherited by this stiff node, in newtons.
    pub effective_tension_n: f64,
    /// Torque inherited by this stiff node, in newton-metres.
    pub torque_nm: f64,
    /// Local normal load per unit length, in newtons per metre.
    pub normal_load_n_m: f64,
    /// Local spatial dogleg curvature, in radians per metre.
    pub dogleg_rad_m: f64,
    /// Local transverse string-center displacement in metres.
    pub displacement_m: f64,
    /// Signed remaining radial clearance in metres; negative indicates penetration before penalty correction.
    pub radial_clearance_m: f64,
    /// Unilateral borehole contact reaction magnitude in newtons.
    pub contact_force_n: f64,
    /// Local bending moment magnitude in newton-metres.
    pub bending_moment_nm: f64,
    /// Extreme-fibre bending stress in pascals.
    pub bending_stress_pa: f64,
}

/// One bounded stiff-string interval result linked to its soft-string candidate.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StiffIntervalResult {
    /// Parent severe-interval identity.
    pub interval_id: Uuid,
    /// Refined interval start measured depth in metres.
    pub start_md_m: f64,
    /// Refined interval end measured depth in metres.
    pub end_md_m: f64,
    /// Reasons inherited from the soft-string severity classifier.
    pub reasons: Vec<StiffIntervalReason>,
    /// Solver convergence state.
    pub convergence: StiffConvergence,
    /// Normalized final equilibrium residual.
    pub residual_norm: f64,
    /// Number of contact-equilibrium iterations executed.
    pub iterations: usize,
    /// Peak nodal contact force in newtons.
    pub peak_contact_force_n: f64,
    /// Peak absolute bending stress in pascals.
    pub peak_bending_stress_pa: f64,
    /// Ordered nodal states through the refined interval.
    pub nodes: Vec<StiffNodeResult>,
}

/// Optional collection of bounded stiff-string interval refinements.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StiffStringResult {
    /// Refined severe intervals in ascending measured-depth order.
    pub intervals: Vec<StiffIntervalResult>,
}

/// Derives a deterministic severe-interval UUID from analysis identity and exact IEEE-754 MD bounds.
#[must_use]
pub fn derive_stiff_interval_id(analysis_id: Uuid, start_md_m: f64, end_md_m: f64) -> Uuid {
    let mut name = [0_u8; 16];
    name[..8].copy_from_slice(&start_md_m.to_bits().to_be_bytes());
    name[8..].copy_from_slice(&end_md_m.to_bits().to_be_bytes());
    Uuid::new_v5(&analysis_id, &name)
}

/// How a final station entered the hybrid soft/stiff result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StiffPointKind {
    /// An existing soft-string station was replaced by a stiff-string nodal state.
    Substituted,
    /// A new station was inserted because the stiff solver reported borehole contact.
    InsertedContact,
}

/// Optional stiff-string detail attached to a final station.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StiffStationRefinement {
    /// How this point entered the final station sequence.
    pub kind: StiffPointKind,
    /// Local transverse displacement in metres.
    pub displacement_m: f64,
    /// Signed remaining radial clearance in metres.
    pub radial_clearance_m: f64,
    /// Borehole contact reaction magnitude in newtons.
    pub contact_force_n: f64,
    /// Local bending moment magnitude in newton-metres.
    pub bending_moment_nm: f64,
    /// Extreme-fibre bending stress in pascals.
    pub bending_stress_pa: f64,
}

/// Per-station soft-string result in canonical SI.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StationResult {
    /// Measured depth in metres.
    pub md_m: f64,
    /// Effective axial tension in newtons (positive = tension).
    pub effective_tension_n: f64,
    /// Torque in newton-metres.
    pub torque_nm: f64,
    /// Normal contact force per metre in newtons per metre (soft-string indication).
    pub normal_load_n_m: f64,
    /// Dogleg severity used at this station in radians per metre.
    pub dogleg_rad_m: f64,
    /// Optional stiff-string detail; absent for untouched soft-string stations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refinement: Option<StiffStationRefinement>,
}

impl StationResult {
    /// Borehole contact reaction for this station, or zero for an untouched soft station.
    #[must_use]
    pub fn contact_force_n(&self) -> f64 {
        self.refinement
            .as_ref()
            .map_or(0.0, |refinement| refinement.contact_force_n)
    }
}

/// Buckling screen result for a station.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BucklingScreen {
    /// Measured depth in metres.
    pub md_m: f64,
    /// Sinusoidal buckling load threshold in newtons (compression positive).
    pub sinusoidal_threshold_n: f64,
    /// Helical buckling load threshold in newtons (compression positive).
    pub helical_threshold_n: f64,
    /// Margin against sinusoidal onset: threshold minus compression.
    /// Negative indicates onset predicted.
    pub sinusoidal_margin_n: f64,
    /// Margin against helical lockup.
    pub helical_margin_n: f64,
}

/// API 7G derated envelope utilization at the most critical station.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSevenGCheck {
    /// Component identity of the governing pipe section.
    pub component_id: Uuid,
    /// Derated tensile envelope in newtons.
    pub derated_tensile_limit_n: f64,
    /// Peak tensile load in newtons.
    pub peak_tensile_n: f64,
    /// Utilization ratio: peak / derated (0.0..=1.0 nominal).
    pub tensile_utilization: f64,
    /// Derated torsional envelope in newton-metres.
    pub derated_torsional_limit_nm: f64,
    /// Peak torque in newton-metres.
    pub peak_torque_nm: f64,
    /// Torsional utilization ratio.
    pub torsional_utilization: f64,
}

/// Solver evidence including dependency lock and normalized hashes.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TnDSolverEvidence {
    /// Engine build version (Cargo package version).
    pub engine_version: String,
    /// Normalized SHA-256 of the request JSON.
    pub request_hash: String,
    /// Normalized SHA-256 of the result JSON (excluding this field).
    pub result_hash: String,
    /// Number of stations solved.
    pub stations_solved: usize,
}

/// Top-level torque-and-drag analysis result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TnDAnalysisResult {
    /// Semver contract version.
    pub contract_version: String,
    /// Echoed analysis identity.
    pub analysis_id: Uuid,
    /// Overall status.
    pub status: AnalysisStatus,
    /// Per-station soft-string states along the string.
    pub stations: Vec<StationResult>,
    /// Buckling screen per station.
    pub buckling: Vec<BucklingScreen>,
    /// API 7G governing check.
    pub api7g: ApiSevenGCheck,
    /// Optional bounded stiff-string refinement. Absent preserves the soft-only result surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stiff_string: Option<StiffStringResult>,
    /// Solver evidence.
    pub evidence: TnDSolverEvidence,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}
