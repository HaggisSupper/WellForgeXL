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
pub use validation::{ContractError, validate_request};
