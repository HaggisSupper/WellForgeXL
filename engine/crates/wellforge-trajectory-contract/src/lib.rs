//! Strict, versioned trajectory analysis contracts.

mod request;
mod result;
mod validation;

pub use request::{
    AzimuthReference, FormationPick, MdDatum, MdDatumKind, ProjectionRequest, SlideInterval,
    StationKind, Target, TargetKind, TrajectoryAnalysisRequest, TrajectorySourceSet,
    TrajectoryStation,
};
pub use result::{
    ApplicabilityStatement, CalculatedStation, CalculationEvidence, FormationCoverage,
    FormationEvaluation, FormationSense, InterpolationResult, InterpolationStatus,
    PlanSurveyResidual, PositionResidual, ProjectionAssessment, SlideAssessment, SlideResponse,
    SlideStatus, SpatialPosition, TargetAssessment, TargetBasis, TargetEvaluation, TargetStatus,
    TrajectoryAnalysisResult, TrajectoryAnalysisStatus, TrajectoryCalculation,
};
pub use validation::ContractError;

/// Canonical registry identifier for trajectory analysis.
pub const CONTRACT_ID: &str = "wellforge.trajectory.analysis";
/// Canonical Release 1 contract version.
pub const CANONICAL_CONTRACT_VERSION: &str = "1.0.0";

/// Validates canonical version policy before applying trajectory-domain invariants.
///
/// # Errors
/// Returns stable contract diagnostics for unsupported versions or invalid domain content.
pub fn validate_request(request: &TrajectoryAnalysisRequest) -> Result<(), Vec<ContractError>> {
    if wellforge_contract_governance::validate_same_major(
        &request.contract_version,
        CANONICAL_CONTRACT_VERSION,
    )
    .is_err()
    {
        return Err(vec![ContractError {
            code: "WF-TRAJECTORY-CONTRACT-001".to_owned(),
            message: "contract_version must be valid SemVer with supported major version"
                .to_owned(),
        }]);
    }
    validation::validate_request(request)
}
