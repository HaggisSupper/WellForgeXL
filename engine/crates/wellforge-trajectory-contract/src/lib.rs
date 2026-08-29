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
pub use validation::{ContractError, validate_request};
