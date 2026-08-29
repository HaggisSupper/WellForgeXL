//! Deterministic request validation.

use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::WitsmlObjectType;

use crate::{BhaAnalysisRequest, ComponentRepresentation};

/// Stable contract diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ContractError {
    /// Stable machine-readable code.
    pub code: String,
    /// Human-readable diagnostic.
    pub message: String,
}

fn push(errors: &mut Vec<ContractError>, code: &str, message: impl Into<String>) {
    errors.push(ContractError {
        code: code.to_owned(),
        message: message.into(),
    });
}

/// Validates all cross-field invariants required before model assembly.
///
/// # Errors
///
/// Returns all stable [`ContractError`] diagnostics when one or more invariants fail.
#[allow(clippy::too_many_lines)]
pub fn validate_request(request: &BhaAnalysisRequest) -> Result<(), Vec<ContractError>> {
    let mut errors = Vec::new();
    if request.contract_version.split('.').next() != Some("1") {
        push(
            &mut errors,
            "WF-BHA-CONTRACT-001",
            "unsupported contract major version",
        );
    }
    let required = [
        WitsmlObjectType::Well,
        WitsmlObjectType::Wellbore,
        WitsmlObjectType::Trajectory,
        WitsmlObjectType::WellboreGeometry,
        WitsmlObjectType::Tubular,
        WitsmlObjectType::BhaRun,
    ];
    for object_type in required {
        if !request
            .sources
            .iter()
            .any(|source| source.object_type == object_type)
        {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-004",
                format!("missing required {object_type:?} source"),
            );
        }
    }
    let mut ids = HashSet::<Uuid>::new();
    for component in &request.components {
        if !ids.insert(component.id) {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-007",
                "duplicate component UUID",
            );
        }
        if !component.bottom_md_m.is_finite()
            || !component.top_md_m.is_finite()
            || component.bottom_md_m <= component.top_md_m
        {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-009",
                "component depths must increase",
            );
        }
        if !(component.od_m > component.id_m && component.id_m >= 0.0) {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-012",
                "component OD must exceed nonnegative ID",
            );
        }
        if component.representation != ComponentRepresentation::Beam {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-021",
                "release 1 accepts only beam component representations",
            );
        }
    }
    for pair in request.components.windows(2) {
        if (pair[0].bottom_md_m - pair[1].top_md_m).abs() > 1.0e-9 {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-008",
                "components must form a contiguous path",
            );
        }
    }
    for section in &request.hole {
        if !(section.bottom_md_m > section.top_md_m && section.diameter_m > 0.0) {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-010",
                "invalid hole geometry section",
            );
        }
    }
    if request.trajectory.is_empty() {
        push(
            &mut errors,
            "WF-BHA-CONTRACT-005",
            "trajectory stations are required",
        );
    }
    for station in &request.trajectory {
        if !station.inclination_rad.is_finite()
            || !(0.0..=std::f64::consts::PI).contains(&station.inclination_rad)
        {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-022",
                "trajectory inclination must be finite and within [0, pi] radians",
            );
        }
    }
    for pair in request.trajectory.windows(2) {
        if pair[1].md_m <= pair[0].md_m {
            push(
                &mut errors,
                "WF-BHA-CONTRACT-006",
                "trajectory MD must increase",
            );
        }
    }
    if request.components.is_empty() || request.hole.is_empty() {
        push(
            &mut errors,
            "WF-BHA-CONTRACT-003",
            "components and hole geometry are required",
        );
    }
    if !request.solver.max_element_length_m.is_finite()
        || request.solver.max_element_length_m <= 0.0
    {
        push(
            &mut errors,
            "WF-BHA-CONTRACT-020",
            "maximum element length must be positive and finite",
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
