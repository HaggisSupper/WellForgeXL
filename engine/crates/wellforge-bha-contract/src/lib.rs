//! Versioned immutable BHA request and result contracts.

mod request;
mod result;
mod validation;

pub use request::{
    BhaAnalysisRequest, BhaComponent, ComponentRepresentation, HoleSection, OperatingPoint,
    SolverSettings, TrajectoryStation,
};
pub use result::{
    AnalysisStatus, BhaAnalysisResult, CampbellPoint, ContactPointResult, FrequencyResponsePoint,
    ModeResult, SolverEvidence, StaticNodeResult,
};
pub use validation::ContractError;

/// Canonical registry identifier for BHA analysis.
pub const CONTRACT_ID: &str = "wellforge.bha.analysis";
/// Canonical Release 1 contract version.
pub const CANONICAL_CONTRACT_VERSION: &str = "1.0.0";

/// Validates canonical version policy before applying BHA-domain invariants.
///
/// # Errors
/// Returns stable contract diagnostics for unsupported versions or invalid domain content.
pub fn validate_request(request: &BhaAnalysisRequest) -> Result<(), Vec<ContractError>> {
    if wellforge_contract_governance::validate_same_major(
        &request.contract_version,
        CANONICAL_CONTRACT_VERSION,
    )
    .is_err()
    {
        return Err(vec![ContractError {
            code: "WF-BHA-CONTRACT-001".to_owned(),
            message: "contract_version must be valid SemVer with supported major version"
                .to_owned(),
        }]);
    }
    validation::validate_request(request)
}
