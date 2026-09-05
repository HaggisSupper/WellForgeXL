//! Steady-state hydraulics core: pipe and annulus friction, bit pressure loss, ECD.
//!
//! Reference basis (per `docs/REFERENCE_ARCHIVE.md`):
//! `Hydraulics Models\`, `^Technical Reference Tools/Drilling Practice Manual/Chapter 07 Hydraulics.pdf`,
//! `Drilling engineering PDFs/10 Hydraulics.pdf`. Standard profile default is API RP 13D 7th Ed (2017,
//! reaffirmed 2023) per `docs/RUST_ENGINE_ROADMAP.md` §3.
//!
//! All public dimensional values use SI base or SI-derived units.

mod correlation;

use rayon::prelude::*;
use wellforge_hydraulics_contract::{
    AnalysisStatus, ComputeBackend, FlowLoop, FlowRegime, HydraulicsAnalysisRequest,
    HydraulicsAnalysisResult, HydraulicsSolverEvidence, SectionPressureLoss, TubularSection,
};

use correlation::evaluate_flow_response;

/// Errors raised by the hydraulics solver.
#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    /// The rheology parameters are internally inconsistent.
    #[error("rheology parameters incomplete or inconsistent for the selected model")]
    Rheology,
    /// A correlation produced a non-finite or non-positive intermediate value.
    #[error("hydraulics correlation produced a non-physical numerical result")]
    Numerical,
}

const GRAVITY_M_S2: f64 = 9.80665;

#[derive(Clone, Debug)]
struct SectionResponse {
    pipe: SectionPressureLoss,
    annulus: SectionPressureLoss,
    pipe_is_transitional: bool,
    annulus_is_transitional: bool,
}

#[derive(Debug)]
struct PreparedFlowState {
    section_results: Vec<SectionPressureLoss>,
    total_pipe_pressure_loss_pa: f64,
    total_annulus_pressure_loss_pa: f64,
    geometry_reference_depth_m: f64,
    has_transitional_section: bool,
}

/// Solve the steady-state hydraulics pass.
///
/// # Errors
///
/// Returns [`SolveError::Rheology`] when the caller-supplied parameters are incomplete
/// (contract validation should prevent this).
pub fn solve_hydraulics(
    request: &HydraulicsAnalysisRequest,
) -> Result<HydraulicsAnalysisResult, SolveError> {
    let prepared = prepare_flow_state(request)?;
    Ok(complete_result(request, &prepared))
}

/// Solve a bounded request batch, reusing section flow state when only nozzle geometry differs.
///
/// Heterogeneous batches remain valid and fall back to independent solves. Result order always
/// matches request order.
///
/// # Errors
///
/// Returns the first solver error produced by a request in the batch.
pub fn solve_hydraulics_batch(
    requests: &[HydraulicsAnalysisRequest],
) -> Result<Vec<HydraulicsAnalysisResult>, SolveError> {
    let Some(first) = requests.first() else {
        return Ok(Vec::new());
    };
    if requests
        .iter()
        .skip(1)
        .all(|request| has_shared_flow_state(first, request))
    {
        let prepared = prepare_flow_state(first)?;
        return Ok(requests
            .iter()
            .map(|request| complete_result(request, &prepared))
            .collect());
    }

    requests.iter().map(solve_hydraulics).collect()
}

fn has_shared_flow_state(
    left: &HydraulicsAnalysisRequest,
    right: &HydraulicsAnalysisRequest,
) -> bool {
    let mut left_without_nozzles = left.clone();
    let mut right_without_nozzles = right.clone();
    left_without_nozzles.operating.nozzles.clear();
    right_without_nozzles.operating.nozzles.clear();
    left_without_nozzles == right_without_nozzles
}

fn prepare_flow_state(
    request: &HydraulicsAnalysisRequest,
) -> Result<PreparedFlowState, SolveError> {
    let rho = request.operating.mud_density_kg_m3;
    let q = request.operating.flow_rate_m3_s;
    let solver = request.solver.unwrap_or_default();
    let is_version_two = request.contract_version == "0.2.0";

    let evaluate = |section: &TubularSection| {
        evaluate_section(
            request,
            section,
            rho,
            q,
            solver.flow_correlation,
            is_version_two,
        )
    };
    let responses: Vec<SectionResponse> = match solver.compute_backend {
        ComputeBackend::SerialCpu => request
            .sections
            .iter()
            .map(evaluate)
            .collect::<Result<_, _>>()?,
        ComputeBackend::ParallelCpu => request
            .sections
            .par_iter()
            .map(evaluate)
            .collect::<Result<_, _>>()?,
    };

    let mut section_results = Vec::with_capacity(request.sections.len() * 2);
    let mut total_pipe = 0.0;
    let mut total_ann = 0.0;
    let mut has_transitional_section = false;
    for (section, response) in request.sections.iter().zip(responses) {
        match section.active_flow_loop {
            None => {
                total_pipe += response.pipe.pressure_loss_pa;
                total_ann += response.annulus.pressure_loss_pa;
                has_transitional_section |=
                    response.pipe_is_transitional || response.annulus_is_transitional;
            }
            Some(FlowLoop::Pipe) => {
                total_pipe += response.pipe.pressure_loss_pa;
                has_transitional_section |= response.pipe_is_transitional;
            }
            Some(FlowLoop::Annulus) => {
                total_ann += response.annulus.pressure_loss_pa;
                has_transitional_section |= response.annulus_is_transitional;
            }
        }
        section_results.push(response.pipe);
        section_results.push(response.annulus);
    }

    let geometry_reference_depth_m = request
        .sections
        .iter()
        .map(|section| section.bottom_tvd_m.unwrap_or(section.bottom_md_m))
        .fold(0.0_f64, f64::max);

    Ok(PreparedFlowState {
        section_results,
        total_pipe_pressure_loss_pa: total_pipe,
        total_annulus_pressure_loss_pa: total_ann,
        geometry_reference_depth_m,
        has_transitional_section,
    })
}

fn complete_result(
    request: &HydraulicsAnalysisRequest,
    prepared: &PreparedFlowState,
) -> HydraulicsAnalysisResult {
    let op = &request.operating;
    let rho = op.mud_density_kg_m3;
    let q = op.flow_rate_m3_s;
    let solver = request.solver.unwrap_or_default();
    let is_version_two = request.contract_version == "0.2.0";
    let nozzle_discharge_coefficient = op.nozzle_discharge_coefficient.unwrap_or(0.95);
    let surface_backpressure_pa = op.surface_backpressure_pa.unwrap_or(0.0);
    let total_flow_area_m2: f64 = op.nozzles.iter().map(|n| area_circle(n.diameter_m)).sum();
    let bit_dp = if total_flow_area_m2 > 0.0 {
        rho * q * q / (2.0 * (nozzle_discharge_coefficient * total_flow_area_m2).powi(2))
    } else {
        0.0
    };

    let reference_vertical_depth_m = op
        .ecd_reference_tvd_m
        .unwrap_or(prepared.geometry_reference_depth_m);
    let ecd = if reference_vertical_depth_m > 0.0 {
        let hydrostatic_pressure_pa = rho * GRAVITY_M_S2 * reference_vertical_depth_m;
        if is_version_two {
            rho + (prepared.total_annulus_pressure_loss_pa + surface_backpressure_pa)
                / (GRAVITY_M_S2 * reference_vertical_depth_m)
        } else {
            (hydrostatic_pressure_pa + prepared.total_annulus_pressure_loss_pa)
                / (GRAVITY_M_S2 * reference_vertical_depth_m)
        }
    } else {
        rho
    };
    let circulating_system_pressure_pa = prepared.total_pipe_pressure_loss_pa
        + bit_dp
        + prepared.total_annulus_pressure_loss_pa
        + surface_backpressure_pa;

    let mut warnings = Vec::new();
    if prepared.has_transitional_section {
        warnings.push(
            "WF-HYD-REGIME-001: at least one section is in the transitional Reynolds band; treat friction as indicative"
                .to_string(),
        );
    }

    let evidence = HydraulicsSolverEvidence {
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        request_hash: String::new(),
        result_hash: String::new(),
        profile_standard: request.profile.standard.clone(),
        profile_edition: request.profile.edition.clone(),
        flow_correlation: is_version_two.then_some(solver.flow_correlation),
        compute_backend: is_version_two.then_some(solver.compute_backend),
        thermal_assumption: is_version_two.then_some(solver.thermal_assumption),
    };

    HydraulicsAnalysisResult {
        contract_version: request.contract_version.clone(),
        analysis_id: request.analysis_id,
        status: if warnings.is_empty() {
            AnalysisStatus::Ok
        } else {
            AnalysisStatus::Warning
        },
        total_pipe_pressure_loss_pa: prepared.total_pipe_pressure_loss_pa,
        total_annulus_pressure_loss_pa: prepared.total_annulus_pressure_loss_pa,
        bit_pressure_loss_pa: bit_dp,
        total_flow_area_m2,
        equivalent_circulating_density_kg_m3: ecd,
        reference_vertical_depth_m: is_version_two.then_some(reference_vertical_depth_m),
        surface_backpressure_pa: is_version_two.then_some(surface_backpressure_pa),
        nozzle_discharge_coefficient: is_version_two.then_some(nozzle_discharge_coefficient),
        circulating_system_pressure_pa: is_version_two.then_some(circulating_system_pressure_pa),
        sections: prepared.section_results.clone(),
        evidence,
        warnings,
    }
}

fn evaluate_section(
    request: &HydraulicsAnalysisRequest,
    section: &TubularSection,
    density_kg_m3: f64,
    flow_rate_m3_s: f64,
    flow_correlation: wellforge_hydraulics_contract::FlowCorrelation,
    include_extended_result: bool,
) -> Result<SectionResponse, SolveError> {
    let length_m = (section.bottom_md_m - section.top_md_m).max(0.0);

    let pipe_area_m2 = area_circle(section.string_id_m);
    let pipe_velocity_m_s = flow_rate_m3_s / pipe_area_m2;
    let pipe_response = evaluate_flow_response(
        flow_correlation,
        &request.rheology,
        density_kg_m3,
        pipe_velocity_m_s,
        section.string_id_m,
        FlowLoop::Pipe,
    )?;
    let pipe_pressure_loss_pa = fanning_pressure_loss(
        pipe_response.fanning_friction_factor,
        density_kg_m3,
        pipe_velocity_m_s,
        length_m,
        section.string_id_m,
    );

    let annulus_area_m2 = area_circle(section.hole_id_m) - area_circle(section.string_od_m);
    let annulus_velocity_m_s = flow_rate_m3_s / annulus_area_m2;
    let annulus_hydraulic_diameter_m = section.hole_id_m - section.string_od_m;
    let annulus_response = evaluate_flow_response(
        flow_correlation,
        &request.rheology,
        density_kg_m3,
        annulus_velocity_m_s,
        annulus_hydraulic_diameter_m,
        FlowLoop::Annulus,
    )?;
    let annulus_pressure_loss_pa = fanning_pressure_loss(
        annulus_response.fanning_friction_factor,
        density_kg_m3,
        annulus_velocity_m_s,
        length_m,
        annulus_hydraulic_diameter_m,
    );

    Ok(SectionResponse {
        pipe: SectionPressureLoss {
            section_id: section.id,
            flow_loop: FlowLoop::Pipe,
            bulk_velocity_m_s: pipe_velocity_m_s,
            reynolds_number: pipe_response.reynolds_number,
            fanning_friction_factor: pipe_response.fanning_friction_factor,
            flow_regime: include_extended_result.then_some(pipe_response.flow_regime),
            pressure_loss_pa: pipe_pressure_loss_pa,
        },
        annulus: SectionPressureLoss {
            section_id: section.id,
            flow_loop: FlowLoop::Annulus,
            bulk_velocity_m_s: annulus_velocity_m_s,
            reynolds_number: annulus_response.reynolds_number,
            fanning_friction_factor: annulus_response.fanning_friction_factor,
            flow_regime: include_extended_result.then_some(annulus_response.flow_regime),
            pressure_loss_pa: annulus_pressure_loss_pa,
        },
        pipe_is_transitional: pipe_response.flow_regime == FlowRegime::Transitional,
        annulus_is_transitional: annulus_response.flow_regime == FlowRegime::Transitional,
    })
}

fn area_circle(diameter_m: f64) -> f64 {
    std::f64::consts::PI * diameter_m.powi(2) / 4.0
}

fn fanning_pressure_loss(
    fanning: f64,
    rho_kg_m3: f64,
    velocity_m_s: f64,
    length_m: f64,
    diameter_m: f64,
) -> f64 {
    if diameter_m <= 0.0 {
        return 0.0;
    }
    2.0 * fanning * rho_kg_m3 * velocity_m_s.powi(2) * length_m / diameter_m
}
