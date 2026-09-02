//! Steady-state hydraulics core: pipe and annulus friction, bit pressure loss, ECD.
//!
//! Reference basis (per `docs/REFERENCE_ARCHIVE.md`):
//! `Hydraulics Models\`, `^Technical Reference Tools/Drilling Practice Manual/Chapter 07 Hydraulics.pdf`,
//! `Drilling engineering PDFs/10 Hydraulics.pdf`. Standard profile default is API RP 13D 7th Ed (2017,
//! reaffirmed 2023) per `docs/RUST_ENGINE_ROADMAP.md` §3.
//!
//! This first analytic pass supports:
//! - Newtonian, Bingham plastic, and power-law rheologies.
//! - Fanning friction from generalised Reynolds:
//!     - Laminar: f = 16 / Re
//!     - Turbulent: f = 0.0791 * Re^-0.25 (Blasius; smooth pipe)
//! - Bit pressure loss from a nozzle-area orifice: `dP = rho * Q^2 / (2 * (Cd * A_total)^2)`, `Cd = 0.95`.
//! - ECD as surface density plus annular hydrostatic pressure equivalent.

use wellforge_hydraulics_contract::{
    AnalysisStatus, FlowLoop, HydraulicsAnalysisRequest, HydraulicsAnalysisResult,
    HydraulicsSolverEvidence, RheologyModel, RheologyParameters, SectionPressureLoss,
};

/// Errors raised by the hydraulics solver.
#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    /// The rheology parameters are internally inconsistent.
    #[error("rheology parameters incomplete or inconsistent for the selected model")]
    Rheology,
}

const GRAVITY_M_S2: f64 = 9.80665;
const NOZZLE_DISCHARGE_COEFFICIENT: f64 = 0.95;

/// Solve the steady-state hydraulics pass.
///
/// # Errors
///
/// Returns [`SolveError::Rheology`] when the caller-supplied parameters are incomplete
/// (contract validation should prevent this).
pub fn solve_hydraulics(
    request: &HydraulicsAnalysisRequest,
) -> Result<HydraulicsAnalysisResult, SolveError> {
    let op = &request.operating;
    let rho = op.mud_density_kg_m3;
    let q = op.flow_rate_m3_s;

    let mut section_results = Vec::with_capacity(request.sections.len() * 2);
    let mut total_pipe = 0.0;
    let mut total_ann = 0.0;

    for section in &request.sections {
        let length_m = (section.bottom_md_m - section.top_md_m).max(0.0);

        // Pipe interior.
        let pipe_area = area_circle(section.string_id_m);
        let pipe_v = if pipe_area > 0.0 { q / pipe_area } else { 0.0 };
        let (pipe_re, pipe_f) =
            reynolds_and_friction(&request.rheology, rho, pipe_v, section.string_id_m)?;
        let pipe_dp = fanning_pressure_loss(pipe_f, rho, pipe_v, length_m, section.string_id_m);
        total_pipe += pipe_dp;
        section_results.push(SectionPressureLoss {
            section_id: section.id,
            flow_loop: FlowLoop::Pipe,
            bulk_velocity_m_s: pipe_v,
            reynolds_number: pipe_re,
            fanning_friction_factor: pipe_f,
            pressure_loss_pa: pipe_dp,
        });

        // Annulus.
        let ann_area = area_circle(section.hole_id_m) - area_circle(section.string_od_m);
        let ann_v = if ann_area > 0.0 { q / ann_area } else { 0.0 };
        let d_hyd = (section.hole_id_m - section.string_od_m).max(0.0);
        let (ann_re, ann_f) = reynolds_and_friction(&request.rheology, rho, ann_v, d_hyd)?;
        let ann_dp = fanning_pressure_loss(ann_f, rho, ann_v, length_m, d_hyd);
        total_ann += ann_dp;
        section_results.push(SectionPressureLoss {
            section_id: section.id,
            flow_loop: FlowLoop::Annulus,
            bulk_velocity_m_s: ann_v,
            reynolds_number: ann_re,
            fanning_friction_factor: ann_f,
            pressure_loss_pa: ann_dp,
        });
    }

    let total_flow_area_m2: f64 = op.nozzles.iter().map(|n| area_circle(n.diameter_m)).sum();
    let bit_dp = if total_flow_area_m2 > 0.0 {
        rho * q * q / (2.0 * (NOZZLE_DISCHARGE_COEFFICIENT * total_flow_area_m2).powi(2))
    } else {
        0.0
    };

    let deepest_md = request
        .sections
        .iter()
        .map(|s| s.bottom_md_m)
        .fold(0.0_f64, f64::max);
    let hydrostatic_pressure_pa = rho * GRAVITY_M_S2 * deepest_md;
    let ecd = if deepest_md > 0.0 {
        (hydrostatic_pressure_pa + total_ann) / (GRAVITY_M_S2 * deepest_md)
    } else {
        rho
    };

    let mut warnings = Vec::new();
    if section_results
        .iter()
        .any(|s| s.reynolds_number > 2100.0 && s.reynolds_number < 4000.0)
    {
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
    };

    Ok(HydraulicsAnalysisResult {
        contract_version: request.contract_version.clone(),
        analysis_id: request.analysis_id,
        status: if warnings.is_empty() {
            AnalysisStatus::Ok
        } else {
            AnalysisStatus::Warning
        },
        total_pipe_pressure_loss_pa: total_pipe,
        total_annulus_pressure_loss_pa: total_ann,
        bit_pressure_loss_pa: bit_dp,
        total_flow_area_m2,
        equivalent_circulating_density_kg_m3: ecd,
        sections: section_results,
        evidence,
        warnings,
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

fn reynolds_and_friction(
    rheology: &RheologyParameters,
    rho_kg_m3: f64,
    velocity_m_s: f64,
    hydraulic_diameter_m: f64,
) -> Result<(f64, f64), SolveError> {
    if velocity_m_s <= 0.0 || hydraulic_diameter_m <= 0.0 {
        return Ok((0.0, 0.0));
    }

    let apparent_viscosity = match rheology.model {
        RheologyModel::Newtonian => rheology.dynamic_viscosity_pa_s.ok_or(SolveError::Rheology)?,
        RheologyModel::Bingham => {
            let pv = rheology.plastic_viscosity_pa_s.ok_or(SolveError::Rheology)?;
            let ty = rheology.yield_stress_pa.ok_or(SolveError::Rheology)?;
            let gamma_wall = 8.0 * velocity_m_s / hydraulic_diameter_m;
            if gamma_wall > 0.0 {
                pv + ty / gamma_wall
            } else {
                pv
            }
        }
        RheologyModel::PowerLaw => {
            let k = rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?;
            let n = rheology.flow_behavior_index.ok_or(SolveError::Rheology)?;
            let gamma_wall = 8.0 * velocity_m_s / hydraulic_diameter_m;
            k * gamma_wall.powf(n - 1.0)
        }
        RheologyModel::HerschelBulkley => {
            let k = rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?;
            let n = rheology.flow_behavior_index.ok_or(SolveError::Rheology)?;
            let ty = rheology.yield_stress_pa.ok_or(SolveError::Rheology)?;
            let gamma_wall = 8.0 * velocity_m_s / hydraulic_diameter_m;
            let shear_stress = ty + k * gamma_wall.powf(n);
            if gamma_wall > 0.0 {
                shear_stress / gamma_wall
            } else {
                k
            }
        }
    };

    if apparent_viscosity <= 0.0 {
        return Ok((0.0, 0.0));
    }
    let re = rho_kg_m3 * velocity_m_s * hydraulic_diameter_m / apparent_viscosity;

    let f = if re < 2100.0 {
        16.0 / re.max(1.0)
    } else {
        0.0791 * re.powf(-0.25)
    };
    Ok((re, f))
}
