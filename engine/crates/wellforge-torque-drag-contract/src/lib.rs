//! Versioned immutable torque-and-drag request and result contracts.
//!
//! Reference basis (per `docs/REFERENCE_ARCHIVE.md`):
//! - Soft-string convention: `Torque and Drag\` and `Pipe Handbooks\`.
//! - API 7G string-strength derating folded into this lane per user directive.

mod request;
mod result;
mod validation;

pub use request::{
    Api7gPipeSpec, OperationState, StringComponent, TnDAnalysisRequest, TnDOperatingPoint,
    TnDTrajectoryStation,
};
pub use result::{
    AnalysisStatus, ApiSevenGCheck, BucklingScreen, StationResult, TnDAnalysisResult,
    TnDSolverEvidence,
};
pub use validation::{ContractError, validate_request};
