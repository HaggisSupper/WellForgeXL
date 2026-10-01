//! Numeric source-result checks, independent of wire evidence and hashing.

use crate::{ContractError, HydraulicsAnalysisResult};

/// Check every emitted scalar before serialization can turn non-finite floats into nulls.
///
/// Zero is allowed for inactive-loop totals, surface backpressure, reference depth and the
/// legacy empty-nozzle case. Nonzero subnormal values indicate numerical underflow.
/// Evidence hashes may be empty: the CLI attaches them after this check.
///
/// # Errors
/// Returns the first out-of-range numeric field; no evidence or identity validation is performed.
pub fn validate_result_numbers(result: &HydraulicsAnalysisResult) -> Result<(), ContractError> {
    let legacy_empty_nozzles =
        result.contract_version == "0.1.0" && result.total_flow_area_m2 == 0.0;
    for (name, value, allow_zero) in [
        (
            "total_pipe_pressure_loss_pa",
            result.total_pipe_pressure_loss_pa,
            true,
        ),
        (
            "total_annulus_pressure_loss_pa",
            result.total_annulus_pressure_loss_pa,
            true,
        ),
        (
            "bit_pressure_loss_pa",
            result.bit_pressure_loss_pa,
            legacy_empty_nozzles,
        ),
        (
            "total_flow_area_m2",
            result.total_flow_area_m2,
            legacy_empty_nozzles,
        ),
        (
            "equivalent_circulating_density_kg_m3",
            result.equivalent_circulating_density_kg_m3,
            false,
        ),
    ] {
        check_number(name, value, allow_zero)?;
    }
    for (name, value, allow_zero) in [
        (
            "reference_vertical_depth_m",
            result.reference_vertical_depth_m,
            true,
        ),
        (
            "surface_backpressure_pa",
            result.surface_backpressure_pa,
            true,
        ),
        (
            "nozzle_discharge_coefficient",
            result.nozzle_discharge_coefficient,
            false,
        ),
        (
            "circulating_system_pressure_pa",
            result.circulating_system_pressure_pa,
            false,
        ),
    ] {
        if let Some(value) = value {
            check_number(name, value, allow_zero)?;
        }
    }
    if result
        .nozzle_discharge_coefficient
        .is_some_and(|value| value > 1.0)
    {
        return Err(numeric_error("nozzle_discharge_coefficient"));
    }
    for (index, section) in result.sections.iter().enumerate() {
        for (name, value) in [
            ("bulk_velocity_m_s", section.bulk_velocity_m_s),
            ("reynolds_number", section.reynolds_number),
            ("fanning_friction_factor", section.fanning_friction_factor),
            ("pressure_loss_pa", section.pressure_loss_pa),
        ] {
            check_number(&format!("sections[{index}].{name}"), value, false)?;
        }
    }
    Ok(())
}

fn check_number(name: &str, value: f64, allow_zero: bool) -> Result<(), ContractError> {
    if (value.is_normal() && value > 0.0) || (allow_zero && value == 0.0) {
        Ok(())
    } else {
        Err(numeric_error(name))
    }
}

fn numeric_error(name: &str) -> ContractError {
    ContractError {
        code: "WF-HYD-RES-001",
        message: format!("{name} is outside the supported numeric range"),
    }
}
