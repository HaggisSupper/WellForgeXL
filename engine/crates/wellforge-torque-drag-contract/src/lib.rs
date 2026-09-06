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
    StiffStringResult, TnDAnalysisResult, TnDSolverEvidence, derive_stiff_interval_id,
};
pub use validation::{ContractError, validate_request};
