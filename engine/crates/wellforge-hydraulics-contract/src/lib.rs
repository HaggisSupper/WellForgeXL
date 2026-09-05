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
pub use validation::{ContractError, validate_request};
