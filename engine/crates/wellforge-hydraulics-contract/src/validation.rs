//! Strict pre-solver validation for `HydraulicsAnalysisRequest`.

use std::collections::HashSet;

use crate::request::{FlowCorrelation, HydraulicsAnalysisRequest, RheologyModel};

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

    if request.analysis_id.is_nil() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-003",
            "analysis_id must not be nil",
        ));
    }

    if request.sources.is_empty() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-004",
            "sources must contain at least one WITSML source reference",
        ));
    }
    let mut source_ids = HashSet::new();
    for (i, source) in request.sources.iter().enumerate() {
        if !source_ids.insert(source.uuid) {
            errors.push(ContractError::new(
                "WF-HYD-REQ-005",
                format!("sources[{i}] reuses a source UUID"),
            ));
        }
        if let Err(identity_error) = source.validate() {
            errors.push(ContractError::new(
                "WF-HYD-REQ-006",
                format!("sources[{i}] is invalid: {identity_error}"),
            ));
        }
    }

    if !matches!(request.contract_version.as_str(), "0.1.0" | "0.2.0") {
        errors.push(ContractError::new(
            "WF-HYD-REQ-001",
            "contract_version must be a supported hydraulics contract version",
        ));
    }
    let is_version_two = request.contract_version == "0.2.0";
    let uses_version_two_fields = request.solver.is_some()
        || request.rheology.high_shear_flow_index.is_some()
        || request.operating.nozzle_discharge_coefficient.is_some()
        || request.operating.surface_backpressure_pa.is_some()
        || request.operating.ecd_reference_tvd_m.is_some()
        || request.sections.iter().any(|section| {
            section.top_tvd_m.is_some()
                || section.bottom_tvd_m.is_some()
                || section.active_flow_loop.is_some()
        });
    if request.contract_version == "0.1.0" && uses_version_two_fields {
        errors.push(ContractError::new(
            "WF-HYD-REQ-040",
            "contract 0.1.0 cannot contain 0.2.0 solver, TVD, active-loop, nozzle or backpressure controls",
        ));
    }
    if is_version_two
        && (request.solver.is_none()
            || request.operating.nozzle_discharge_coefficient.is_none()
            || request.operating.surface_backpressure_pa.is_none())
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-041",
            "contract 0.2.0 requires solver, nozzle_discharge_coefficient and surface_backpressure_pa",
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

    let mut section_ids = HashSet::new();
    let any_tvd = request
        .sections
        .iter()
        .any(|section| section.top_tvd_m.is_some() || section.bottom_tvd_m.is_some());
    let all_tvd = request
        .sections
        .iter()
        .all(|section| section.top_tvd_m.is_some() && section.bottom_tvd_m.is_some());
    if is_version_two && any_tvd && !all_tvd {
        errors.push(ContractError::new(
            "WF-HYD-REQ-015",
            "TVD must be supplied for every section or omitted from every section",
        ));
    }

    for (i, s) in request.sections.iter().enumerate() {
        if is_version_two
            && (s.id.is_nil() || !section_ids.insert(s.id) || s.name.trim().is_empty())
        {
            errors.push(ContractError::new(
                "WF-HYD-REQ-016",
                format!("sections[{i}] must have a unique non-nil ID and non-empty name"),
            ));
        }
        let invalid_md = if is_version_two {
            !s.top_md_m.is_finite()
                || !s.bottom_md_m.is_finite()
                || s.top_md_m < 0.0
                || s.bottom_md_m <= s.top_md_m
        } else {
            s.bottom_md_m <= s.top_md_m
        };
        if invalid_md {
            errors.push(ContractError::new(
                "WF-HYD-REQ-011",
                format!(
                    "sections[{i}] MD values must be finite, non-negative, and increase downward"
                ),
            ));
        }
        let invalid_string_geometry = if is_version_two {
            !s.string_id_m.is_finite()
                || !s.string_od_m.is_finite()
                || s.string_id_m <= 0.0
                || s.string_od_m <= s.string_id_m
        } else {
            s.string_od_m <= s.string_id_m
        };
        if invalid_string_geometry {
            errors.push(ContractError::new(
                "WF-HYD-REQ-012",
                format!(
                    "sections[{i}] string diameters must be finite and string_od_m must exceed a positive string_id_m"
                ),
            ));
        }
        if (is_version_two && !s.hole_id_m.is_finite()) || s.hole_id_m <= s.string_od_m {
            errors.push(ContractError::new(
                "WF-HYD-REQ-013",
                format!("sections[{i}] hole_id_m must exceed string_od_m"),
            ));
        }
        if is_version_two {
            match (s.top_tvd_m, s.bottom_tvd_m) {
                (None, None) => {}
                (Some(top_tvd), Some(bottom_tvd))
                    if top_tvd.is_finite()
                        && bottom_tvd.is_finite()
                        && top_tvd >= 0.0
                        && bottom_tvd >= top_tvd
                        && top_tvd <= s.top_md_m
                        && bottom_tvd <= s.bottom_md_m => {}
                _ => errors.push(ContractError::new(
                    "WF-HYD-REQ-014",
                    format!(
                        "sections[{i}] TVD values must be supplied as a finite non-decreasing pair bounded by MD"
                    ),
                )),
            }
        }
    }

    let op = &request.operating;
    if !op.mud_density_kg_m3.is_finite() || op.mud_density_kg_m3 <= 0.0 {
        errors.push(ContractError::new(
            "WF-HYD-REQ-020",
            "operating.mud_density_kg_m3 must be positive",
        ));
    }
    if !op.flow_rate_m3_s.is_finite() || op.flow_rate_m3_s <= 0.0 {
        errors.push(ContractError::new(
            "WF-HYD-REQ-021",
            "operating.flow_rate_m3_s must be positive",
        ));
    }
    if is_version_two && (!op.surface_temperature_k.is_finite() || op.surface_temperature_k <= 0.0)
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-025",
            "operating.surface_temperature_k must be finite and positive",
        ));
    }
    if let Some(discharge_coefficient) = op.nozzle_discharge_coefficient
        && (!discharge_coefficient.is_finite()
            || discharge_coefficient <= 0.0
            || discharge_coefficient > 1.0)
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-023",
            "operating.nozzle_discharge_coefficient must be finite and in (0, 1]",
        ));
    }
    if let Some(surface_backpressure_pa) = op.surface_backpressure_pa
        && (!surface_backpressure_pa.is_finite() || surface_backpressure_pa < 0.0)
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-024",
            "operating.surface_backpressure_pa must be finite and non-negative",
        ));
    }
    if let Some(ecd_reference_tvd_m) = op.ecd_reference_tvd_m
        && (!ecd_reference_tvd_m.is_finite() || ecd_reference_tvd_m <= 0.0)
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-027",
            "operating.ecd_reference_tvd_m must be finite and positive",
        ));
    }
    if is_version_two && op.nozzles.is_empty() {
        errors.push(ContractError::new(
            "WF-HYD-REQ-026",
            "operating.nozzles must contain at least one nozzle",
        ));
    }
    for (i, n) in op.nozzles.iter().enumerate() {
        if !n.diameter_m.is_finite() || n.diameter_m <= 0.0 {
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
            if !r
                .dynamic_viscosity_pa_s
                .is_some_and(|value| value.is_finite() && value > 0.0)
            {
                errors.push(missing("dynamic_viscosity_pa_s"));
            }
        }
        RheologyModel::Bingham => {
            if !r
                .yield_stress_pa
                .is_some_and(|value| value.is_finite() && value >= 0.0)
            {
                errors.push(missing("yield_stress_pa"));
            }
            if !r
                .plastic_viscosity_pa_s
                .is_some_and(|value| value.is_finite() && value > 0.0)
            {
                errors.push(missing("plastic_viscosity_pa_s"));
            }
        }
        RheologyModel::PowerLaw => {
            if !r
                .consistency_k_pa_s_n
                .is_some_and(|value| value.is_finite() && value > 0.0)
            {
                errors.push(missing("consistency_k_pa_s_n"));
            }
            let n = r.flow_behavior_index.unwrap_or(0.0);
            if !n.is_finite() || !(0.0..=2.0).contains(&n) || n == 0.0 {
                errors.push(missing("flow_behavior_index"));
            }
            validate_high_shear_index(request, &mut errors);
        }
        RheologyModel::HerschelBulkley => {
            if !r
                .yield_stress_pa
                .is_some_and(|value| value.is_finite() && value >= 0.0)
            {
                errors.push(missing("yield_stress_pa"));
            }
            if !r
                .consistency_k_pa_s_n
                .is_some_and(|value| value.is_finite() && value > 0.0)
            {
                errors.push(missing("consistency_k_pa_s_n"));
            }
            let n = r.flow_behavior_index.unwrap_or(0.0);
            if !n.is_finite() || !(0.0..=2.0).contains(&n) || n == 0.0 {
                errors.push(missing("flow_behavior_index"));
            }
            validate_high_shear_index(request, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_high_shear_index(request: &HydraulicsAnalysisRequest, errors: &mut Vec<ContractError>) {
    if !request
        .solver
        .is_some_and(|solver| solver.flow_correlation == FlowCorrelation::GeneralizedYieldPowerLaw)
    {
        return;
    }
    let flow_index = request.rheology.flow_behavior_index.unwrap_or(0.0);
    let high_shear_index = request.rheology.high_shear_flow_index.unwrap_or(0.0);
    if !flow_index.is_finite()
        || !(0.05..=2.0).contains(&flow_index)
        || !high_shear_index.is_finite()
        || !(0.05..=2.0).contains(&high_shear_index)
    {
        errors.push(ContractError::new(
            "WF-HYD-REQ-031",
            "generalized power-law rheology requires flow_behavior_index and high_shear_flow_index in [0.05, 2]",
        ));
    }
}
