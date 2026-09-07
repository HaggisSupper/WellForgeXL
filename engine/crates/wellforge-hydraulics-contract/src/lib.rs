//! Versioned immutable hydraulics request and result contracts.
//!
//! Reference basis (per `docs/REFERENCE_ARCHIVE.md`):
//! - `Hydraulics Models\`, `^Technical Reference Tools/Drilling Practice Manual/Chapter 07 Hydraulics.pdf`.
//! - Standard profile identifier defaults to API RP 13D 7th Ed (2017, reaffirmed 2023) per `docs/RUST_ENGINE_ROADMAP.md` §3.

mod request;
mod result;
mod validation;

pub use request::{
    ComputeBackend, FlowCorrelation, FlowLoop, HydraulicsAnalysisRequest, HydraulicsOperatingPoint,
    HydraulicsSolverOptions, Nozzle, RheologyModel, RheologyParameters, StandardProfile,
    ThermalAssumption, TubularSection,
};
pub use result::{
    AnalysisStatus, FlowRegime, HydraulicsAnalysisResult, HydraulicsSolverEvidence,
    SectionPressureLoss,
};
pub use validation::ContractError;

/// Canonical registry identifier for hydraulics analysis.
pub const CONTRACT_ID: &str = "wellforge.hydraulics.analysis";
/// Latest canonical hydraulics contract version.
pub const CANONICAL_CONTRACT_VERSION: &str = "0.2.0";
/// Explicit hydraulics contract versions accepted by this engine family.
pub const SUPPORTED_CONTRACT_VERSIONS: &[&str] = &["0.1.0", "0.2.0"];

/// Validates canonical version policy before applying hydraulics-domain invariants.
///
/// # Errors
/// Returns stable contract diagnostics for unsupported versions or invalid domain content.
pub fn validate_request(request: &HydraulicsAnalysisRequest) -> Result<(), Vec<ContractError>> {
    if wellforge_contract_governance::validate_explicit(
        &request.contract_version,
        SUPPORTED_CONTRACT_VERSIONS,
    )
    .is_err()
    {
        return Err(vec![ContractError {
            code: "WF-HYD-REQ-001",
            message: "contract_version must be valid SemVer in the supported hydraulics set"
                .to_owned(),
        }]);
    }
    validation::validate_request(request)
}
