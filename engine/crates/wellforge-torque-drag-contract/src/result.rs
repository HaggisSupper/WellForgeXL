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
    /// Solver evidence.
    pub evidence: TnDSolverEvidence,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}
