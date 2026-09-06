//! Central canonical contract registry for `WellForge` calculation-engine boundaries.

use schemars::schema_for;
use semver::Version;
use thiserror::Error;
use wellforge_bha_contract::{BhaAnalysisRequest, BhaAnalysisResult};
use wellforge_contract_governance::{
    CompatibilityPolicy, ContractDescriptor, ContractRegistry, GovernanceError, RegistryError,
};
use wellforge_hydraulics_contract::{HydraulicsAnalysisRequest, HydraulicsAnalysisResult};
use wellforge_torque_drag_contract::{TnDAnalysisRequest, TnDAnalysisResult};
use wellforge_trajectory_contract::{TrajectoryAnalysisRequest, TrajectoryAnalysisResult};

/// Failure while constructing the canonical registry from Rust contract schemas.
#[derive(Debug, Error)]
pub enum RegistryBuildError {
    /// Contract identity/version metadata is invalid.
    #[error(transparent)]
    Governance(#[from] GovernanceError),
    /// The same canonical ID/version was registered with conflicting metadata.
    #[error(transparent)]
    Registration(#[from] RegistryError),
    /// A generated Rust JSON schema could not be represented as JSON.
    #[error(transparent)]
    SchemaSerialization(#[from] serde_json::Error),
}

fn descriptor(
    id: &str,
    version: &str,
    compatibility: CompatibilityPolicy,
    request_schema: serde_json::Value,
    result_schema: serde_json::Value,
) -> Result<ContractDescriptor, RegistryBuildError> {
    Ok(ContractDescriptor::new(
        id,
        version,
        compatibility,
        &request_schema,
        &result_schema,
    )?)
}

/// Builds the deterministic canonical contract registry directly from Rust request/result types.
///
/// # Errors
/// Returns [`RegistryBuildError`] if schema generation metadata is invalid or registrations
/// conflict.
pub fn build_registry() -> Result<ContractRegistry, RegistryBuildError> {
    let trajectory_request = serde_json::to_value(schema_for!(TrajectoryAnalysisRequest))?;
    let trajectory_result = serde_json::to_value(schema_for!(TrajectoryAnalysisResult))?;
    let bha_request = serde_json::to_value(schema_for!(BhaAnalysisRequest))?;
    let bha_result = serde_json::to_value(schema_for!(BhaAnalysisResult))?;
    let tnd_request = serde_json::to_value(schema_for!(TnDAnalysisRequest))?;
    let tnd_result = serde_json::to_value(schema_for!(TnDAnalysisResult))?;
    let hydraulics_request = serde_json::to_value(schema_for!(HydraulicsAnalysisRequest))?;
    let hydraulics_result = serde_json::to_value(schema_for!(HydraulicsAnalysisResult))?;

    let mut registry = ContractRegistry::new();
    registry.register(descriptor(
        wellforge_trajectory_contract::CONTRACT_ID,
        wellforge_trajectory_contract::CANONICAL_CONTRACT_VERSION,
        CompatibilityPolicy::SameMajor,
        trajectory_request,
        trajectory_result,
    )?)?;
    registry.register(descriptor(
        wellforge_bha_contract::CONTRACT_ID,
        wellforge_bha_contract::CANONICAL_CONTRACT_VERSION,
        CompatibilityPolicy::SameMajor,
        bha_request,
        bha_result,
    )?)?;
    registry.register(descriptor(
        wellforge_torque_drag_contract::CONTRACT_ID,
        wellforge_torque_drag_contract::CANONICAL_CONTRACT_VERSION,
        CompatibilityPolicy::Exact,
        tnd_request,
        tnd_result,
    )?)?;

    let hydraulics_compatibility =
        CompatibilityPolicy::ExplicitSet(vec![Version::new(0, 1, 0), Version::new(0, 2, 0)]);
    for version in wellforge_hydraulics_contract::SUPPORTED_CONTRACT_VERSIONS {
        registry.register(descriptor(
            wellforge_hydraulics_contract::CONTRACT_ID,
            version,
            hydraulics_compatibility.clone(),
            hydraulics_request.clone(),
            hydraulics_result.clone(),
        )?)?;
    }

    Ok(registry)
}
