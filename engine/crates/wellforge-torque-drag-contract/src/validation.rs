//! Strict pre-solver validation for `TnDAnalysisRequest`.

use crate::request::{OperationState, TnDAnalysisRequest};

/// A single contract validation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractError {
    /// Stable error code.
    pub code: &'static str,
    /// Human-readable message.
    pub message: String,
}

impl ContractError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Validate a request. Returns `Ok(())` when the request is admissible.
///
/// # Errors
///
/// Returns a vector of `ContractError` describing every violation found.
/// The caller may render them as-is; the CLI wraps them into the structured
/// JSON error envelope.
#[allow(clippy::too_many_lines)]
pub fn validate_request(request: &TnDAnalysisRequest) -> Result<(), Vec<ContractError>> {
    let mut errors = Vec::new();

    if request.contract_version.trim().is_empty() {
        errors.push(ContractError::new(
            "WF-TND-REQ-001",
            "contract_version must not be empty",
        ));
    }

    if request.components.is_empty() {
        errors.push(ContractError::new(
            "WF-TND-REQ-010",
            "components must contain at least one string section",
        ));
    }

    if request.trajectory.len() < 2 {
        errors.push(ContractError::new(
            "WF-TND-REQ-020",
            "trajectory must contain at least two stations to define a course",
        ));
    }

    for (i, station) in request.trajectory.iter().enumerate() {
        if !station.md_m.is_finite() || station.md_m < 0.0 {
            errors.push(ContractError::new(
                "WF-TND-REQ-021",
                format!("trajectory[{i}].md_m must be finite and non-negative"),
            ));
        }
        if !station.inclination_rad.is_finite() {
            errors.push(ContractError::new(
                "WF-TND-REQ-022",
                format!("trajectory[{i}].inclination_rad must be finite"),
            ));
        }
    }

    for pair in request.trajectory.windows(2) {
        if pair[1].md_m <= pair[0].md_m {
            errors.push(ContractError::new(
                "WF-TND-REQ-023",
                "trajectory stations must be strictly increasing in md_m",
            ));
            break;
        }
    }

    for (i, component) in request.components.iter().enumerate() {
        if component.bottom_md_m <= component.top_md_m {
            errors.push(ContractError::new(
                "WF-TND-REQ-011",
                format!("components[{i}] bottom_md_m must exceed top_md_m"),
            ));
        }
        if component.od_m <= component.id_m {
            errors.push(ContractError::new(
                "WF-TND-REQ-012",
                format!("components[{i}] od_m must exceed id_m"),
            ));
        }
        if component.linear_weight_kg_m <= 0.0 {
            errors.push(ContractError::new(
                "WF-TND-REQ-013",
                format!("components[{i}] linear_weight_kg_m must be positive"),
            ));
        }
        if let Some(spec) = &component.api7g_spec {
            if !(0.0..=1.0).contains(&spec.wear_class_derating) {
                errors.push(ContractError::new(
                    "WF-TND-REQ-030",
                    format!("components[{i}].api7g_spec.wear_class_derating must be within [0, 1]"),
                ));
            }
            if spec.safety_factor < 1.0 || !spec.safety_factor.is_finite() {
                errors.push(ContractError::new(
                    "WF-TND-REQ-031",
                    format!("components[{i}].api7g_spec.safety_factor must be >= 1.0"),
                ));
            }
        }
    }

    let op = &request.operating;
    if op.mud_density_kg_m3 <= 0.0 {
        errors.push(ContractError::new(
            "WF-TND-REQ-040",
            "operating.mud_density_kg_m3 must be positive",
        ));
    }
    if op.friction_factor_open_hole < 0.0 || op.friction_factor_open_hole > 1.0 {
        errors.push(ContractError::new(
            "WF-TND-REQ-041",
            "operating.friction_factor_open_hole must be within [0, 1]",
        ));
    }
    if op.friction_factor_cased_hole < 0.0 || op.friction_factor_cased_hole > 1.0 {
        errors.push(ContractError::new(
            "WF-TND-REQ-042",
            "operating.friction_factor_cased_hole must be within [0, 1]",
        ));
    }
    if matches!(
        op.state,
        OperationState::RotatingOffBottom | OperationState::Drilling | OperationState::Backreaming
    ) && op.surface_rpm_rad_s <= 0.0
    {
        errors.push(ContractError::new(
            "WF-TND-REQ-043",
            "operating.surface_rpm_rad_s must be positive for rotating states",
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
