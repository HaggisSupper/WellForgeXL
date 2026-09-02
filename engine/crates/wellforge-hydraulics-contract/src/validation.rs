//! Strict pre-solver validation for `HydraulicsAnalysisRequest`.

use crate::request::{HydraulicsAnalysisRequest, RheologyModel};

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

/// Validate a hydraulics request.
///
/// # Errors
///
/// Returns a vector of `ContractError` describing every violation found.
#[allow(clippy::too_many_lines)]
pub fn validate_request(request: &HydraulicsAnalysisRequest) -> Result<(), Vec<ContractError>> {
    let mut errors = Vec::new();

    if request.contract_version.trim().is_empty() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-001",
            "contract_version must not be empty",
        ));
    }

    if request.profile.standard.trim().is_empty() || request.profile.edition.trim().is_empty() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-002",
            "profile.standard and profile.edition are both required",
        ));
    }

    if request.sections.is_empty() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-010",
            "sections must contain at least one tubular section",
        ));
    }

    for (i, s) in request.sections.iter().enumerate() {
        if s.bottom_md_m <= s.top_md_m {
            errors.push(ContractError::new(
                "WF-HYD-REQ-011",
                format!("sections[{i}] bottom_md_m must exceed top_md_m"),
            ));
        }
        if s.string_od_m <= s.string_id_m {
            errors.push(ContractError::new(
                "WF-HYD-REQ-012",
                format!("sections[{i}] string_od_m must exceed string_id_m"),
            ));
        }
        if s.hole_id_m <= s.string_od_m {
            errors.push(ContractError::new(
                "WF-HYD-REQ-013",
                format!("sections[{i}] hole_id_m must exceed string_od_m"),
            ));
        }
    }

    let op = &request.operating;
    if op.mud_density_kg_m3 <= 0.0 {
        errors.push(ContractError::new(
            "WF-HYD-REQ-020",
            "operating.mud_density_kg_m3 must be positive",
        ));
    }
    if op.flow_rate_m3_s <= 0.0 {
        errors.push(ContractError::new(
            "WF-HYD-REQ-021",
            "operating.flow_rate_m3_s must be positive",
        ));
    }
    for (i, n) in op.nozzles.iter().enumerate() {
        if n.diameter_m <= 0.0 {
            errors.push(ContractError::new(
                "WF-HYD-REQ-022",
                format!("operating.nozzles[{i}].diameter_m must be positive"),
            ));
        }
    }

    let r = &request.rheology;
    let missing = |field: &str| {
        ContractError::new(
            "WF-HYD-REQ-030",
            format!("rheology.{field} is required for the selected model"),
        )
    };
    match r.model {
        RheologyModel::Newtonian => {
            if r.dynamic_viscosity_pa_s.unwrap_or(0.0) <= 0.0 {
                errors.push(missing("dynamic_viscosity_pa_s"));
            }
        }
        RheologyModel::Bingham => {
            if r.yield_stress_pa.unwrap_or(-1.0) < 0.0 {
                errors.push(missing("yield_stress_pa"));
            }
            if r.plastic_viscosity_pa_s.unwrap_or(0.0) <= 0.0 {
                errors.push(missing("plastic_viscosity_pa_s"));
            }
        }
        RheologyModel::PowerLaw => {
            if r.consistency_k_pa_s_n.unwrap_or(0.0) <= 0.0 {
                errors.push(missing("consistency_k_pa_s_n"));
            }
            let n = r.flow_behavior_index.unwrap_or(0.0);
            if !(0.0..=2.0).contains(&n) || n == 0.0 {
                errors.push(missing("flow_behavior_index"));
            }
        }
        RheologyModel::HerschelBulkley => {
            if r.yield_stress_pa.unwrap_or(-1.0) < 0.0 {
                errors.push(missing("yield_stress_pa"));
            }
            if r.consistency_k_pa_s_n.unwrap_or(0.0) <= 0.0 {
                errors.push(missing("consistency_k_pa_s_n"));
            }
            let n = r.flow_behavior_index.unwrap_or(0.0);
            if !(0.0..=2.0).contains(&n) || n == 0.0 {
                errors.push(missing("flow_behavior_index"));
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
