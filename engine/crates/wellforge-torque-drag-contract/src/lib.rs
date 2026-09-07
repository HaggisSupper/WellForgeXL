//! Versioned immutable torque-and-drag request and result contracts.
//!
//! Reference basis (per `docs/REFERENCE_ARCHIVE.md`):
//! - Soft-string convention: `Torque and Drag\` and `Pipe Handbooks\`.
//! - API 7G string-strength derating folded into this lane per user directive.

mod request;
mod result;
mod validation;

pub use request::{
    Api7gPipeSpec, OperationState, StiffStringMode, StringComponent, TnDAnalysisRequest,
    TnDHoleSection, TnDOperatingPoint, TnDSolverOptions, TnDTrajectoryStation,
};
pub use result::{
    AnalysisStatus, ApiSevenGCheck, BucklingScreen, StationResult, StiffConvergence,
    StiffIntervalCandidate, StiffIntervalReason, StiffIntervalResult, StiffNodeResult,
    StiffPointKind, StiffStationRefinement, StiffStringResult, TnDAnalysisResult,
    TnDSolverEvidence, derive_stiff_interval_id,
};
pub use validation::ContractError;

/// Canonical registry identifier for torque-and-drag analysis.
pub const CONTRACT_ID: &str = "wellforge.torque_drag.analysis";
/// Canonical torque-and-drag contract version.
pub const CANONICAL_CONTRACT_VERSION: &str = "0.1.0";

/// Validates canonical version policy before applying torque-and-drag domain invariants.
///
/// # Errors
/// Returns stable contract diagnostics for unsupported versions or invalid domain content.
pub fn validate_request(request: &TnDAnalysisRequest) -> Result<(), Vec<ContractError>> {
    if wellforge_contract_governance::validate_exact(
        &request.contract_version,
        CANONICAL_CONTRACT_VERSION,
    )
    .is_err()
    {
        return Err(vec![ContractError {
            code: "WF-TND-REQ-001",
            message: "contract_version must be exactly supported canonical SemVer".to_owned(),
        }]);
    }
    validation::validate_request(request)
}
