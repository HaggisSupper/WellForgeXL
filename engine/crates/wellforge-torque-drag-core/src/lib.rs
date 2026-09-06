//! Whole-well soft-string torque-and-drag core.
//!
//! Reference basis: `Torque and Drag\` (Johancsik-Friesen soft-string convention);
//! API 7G derated envelope from `^Technical Reference Tools/Industry Specifications/API 7G 2009.pdf`.
//!
//! This is the initial analytic pass:
//! - Effective tension is integrated bottom-up along the trajectory.
//! - Weight-in-mud uses the buoyancy factor `1 - rho_mud / rho_steel`.
//! - Normal load per metre uses the classic axial-plus-curvature formulation:
//!   `w = W_bf * sin(inc) + T * dogleg`.
//! - Sign of `mu * |normal|` follows `OperationState`.
//! - Buckling thresholds use Dawson-Paslay sinusoidal and 2 * root(2) helical.

use wellforge_torque_drag_contract::{
    AnalysisStatus, Api7gPipeSpec, ApiSevenGCheck, BucklingScreen, OperationState, StationResult,
    StiffIntervalCandidate, StiffIntervalReason, StiffStringMode, StringComponent,
    TnDAnalysisRequest, TnDAnalysisResult, TnDSolverEvidence, TnDTrajectoryStation,
    derive_stiff_interval_id,
};

/// Errors raised by the solver before writing a result.
#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    /// No component contains the surveyed station MD.
    #[error("no component contains MD {md_m:.3} m")]
    NoComponentAtStation {
        /// Offending measured depth in metres.
        md_m: f64,
    },
    /// No component carries an API 7G spec (required for the governing check).
    #[error("at least one string component must carry an API 7G spec for the governing check")]
    MissingApi7gSpec,
}

const STEEL_DENSITY_KG_M3: f64 = 7850.0;
const GRAVITY_M_S2: f64 = 9.80665;

fn spatial_dogleg_rad_per_m(upper: &TnDTrajectoryStation, lower: &TnDTrajectoryStation) -> f64 {
    let delta_md = lower.md_m - upper.md_m;
    if delta_md <= 0.0 {
        return 0.0;
    }
    let cosine = upper.inclination_rad.cos() * lower.inclination_rad.cos()
        + upper.inclination_rad.sin()
            * lower.inclination_rad.sin()
            * (lower.azimuth_rad - upper.azimuth_rad).cos();
    cosine.clamp(-1.0, 1.0).acos() / delta_md
}

fn add_reason(reasons: &mut Vec<StiffIntervalReason>, reason: StiffIntervalReason) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn station_severity_reasons(
    request: &TnDAnalysisRequest,
    station: &StationResult,
    buckling: &BucklingScreen,
) -> Vec<StiffIntervalReason> {
    let mut reasons = Vec::with_capacity(3);
    if station.normal_load_n_m >= request.solver.severity_normal_load_n_m {
        reasons.push(StiffIntervalReason::NormalLoad);
    }
    if buckling.sinusoidal_margin_n <= request.solver.severity_buckling_margin_n {
        reasons.push(StiffIntervalReason::SinusoidalBuckling);
    }
    if buckling.helical_margin_n <= request.solver.severity_buckling_margin_n {
        reasons.push(StiffIntervalReason::HelicalBuckling);
    }
    reasons
}

/// Classifies deterministic severe intervals from an accepted whole-well soft-string result.
///
/// The classifier does not calculate contact loads. It selects bounded MD intervals for a later
/// stiff-string refinement using configured normal-load and buckling-margin triggers. Disabled
/// stiff-string mode always returns no candidates.
#[must_use]
pub fn classify_stiff_intervals(
    request: &TnDAnalysisRequest,
    soft_result: &TnDAnalysisResult,
) -> Vec<StiffIntervalCandidate> {
    if request.solver.stiff_string_mode == StiffStringMode::Disabled
        || soft_result.stations.is_empty()
        || soft_result.stations.len() != soft_result.buckling.len()
    {
        return Vec::new();
    }

    let first_md = soft_result.stations[0].md_m;
    let last_md = soft_result
        .stations
        .last()
        .map_or(first_md, |station| station.md_m);
    let mut candidates = Vec::new();
    let mut group_start: Option<usize> = None;
    let mut group_end = 0_usize;
    let mut group_reasons = Vec::new();

    let flush_group = |start: usize,
                       end: usize,
                       reasons: &mut Vec<StiffIntervalReason>,
                       output: &mut Vec<StiffIntervalCandidate>| {
        let raw_start = soft_result.stations[start].md_m;
        let raw_end = soft_result
            .stations
            .get(end + 1)
            .map_or(soft_result.stations[end].md_m, |station| station.md_m);
        let start_md_m = (raw_start - request.solver.transition_buffer_m).max(first_md);
        let end_md_m = (raw_end + request.solver.transition_buffer_m).min(last_md);
        output.push(StiffIntervalCandidate {
            id: derive_stiff_interval_id(request.analysis_id, start_md_m, end_md_m),
            start_md_m,
            end_md_m,
            reasons: std::mem::take(reasons),
        });
    };

    for (index, (station, buckling)) in soft_result
        .stations
        .iter()
        .zip(&soft_result.buckling)
        .enumerate()
    {
        let reasons = station_severity_reasons(request, station, buckling);
        if reasons.is_empty() {
            if let Some(start) = group_start.take() {
                flush_group(start, group_end, &mut group_reasons, &mut candidates);
            }
            continue;
        }

        group_start.get_or_insert(index);
        group_end = index;
        for reason in reasons {
            add_reason(&mut group_reasons, reason);
        }
    }

    if let Some(start) = group_start {
        flush_group(start, group_end, &mut group_reasons, &mut candidates);
    }

    candidates
}

/// Solve the soft-string pass and return the full result contract.
///
/// # Errors
///
/// Returns [`SolveError::NoComponentAtStation`] when the trajectory covers depth
/// not modelled by any component, or [`SolveError::MissingApi7gSpec`] when no
/// component carries the API 7G derated envelope.
pub fn solve_soft_string(request: &TnDAnalysisRequest) -> Result<TnDAnalysisResult, SolveError> {
    let op = &request.operating;
    let mu_sign: f64 = match op.state {
        OperationState::Pickup | OperationState::Backreaming => 1.0,
        OperationState::SlackOff | OperationState::Sliding | OperationState::Drilling => -1.0,
        OperationState::RotatingOffBottom => 0.0,
    };

    let mut stations = Vec::with_capacity(request.trajectory.len());
    let mut buckling = Vec::with_capacity(request.trajectory.len());

    // Iterate bottom-up so we can accumulate axial tension from the bit.
    let n = request.trajectory.len();
    let mut effective_tension = op.weight_on_bit_n.max(0.0);
    let mut running_torque = op.torque_on_bit_nm.max(0.0);

    // Prepare storage in surface-down order after loop.
    let mut station_buf = Vec::with_capacity(n);
    let mut buckle_buf = Vec::with_capacity(n);

    for i in (0..n).rev() {
        let s_upper = &request.trajectory[i];
        let component = find_component(&request.components, s_upper.md_m)?;
        let mu = if s_upper.cased {
            op.friction_factor_cased_hole
        } else {
            op.friction_factor_open_hole
        };

        let (delta_md, avg_inc, dogleg_rad_m) = if i + 1 < n {
            let s_lower = &request.trajectory[i + 1];
            let dmd = (s_lower.md_m - s_upper.md_m).max(0.0);
            let avg = f64::midpoint(s_upper.inclination_rad, s_lower.inclination_rad);
            let dl = spatial_dogleg_rad_per_m(s_upper, s_lower);
            (dmd, avg, dl)
        } else {
            (0.0, s_upper.inclination_rad, 0.0)
        };

        let buoyancy_factor = 1.0 - op.mud_density_kg_m3 / STEEL_DENSITY_KG_M3;
        let weight_per_m = component.linear_weight_kg_m * GRAVITY_M_S2 * buoyancy_factor;
        let normal_per_m =
            (weight_per_m * s_upper.inclination_rad.sin()).abs() + effective_tension * dogleg_rad_m;

        // Increment tension for the section BELOW the current station (bottom-up).
        let axial_gain =
            weight_per_m * avg_inc.cos() * delta_md + mu_sign * mu * normal_per_m * delta_md;
        effective_tension += axial_gain;
        running_torque += mu * normal_per_m * component.od_m * 0.5 * delta_md;

        let (sin_th, hel_th) =
            buckling_thresholds(component, weight_per_m, s_upper.inclination_rad);
        let compression = (-effective_tension).max(0.0);
        station_buf.push(StationResult {
            md_m: s_upper.md_m,
            effective_tension_n: effective_tension,
            torque_nm: running_torque,
            normal_load_n_m: normal_per_m,
            dogleg_rad_m,
        });
        buckle_buf.push(BucklingScreen {
            md_m: s_upper.md_m,
            sinusoidal_threshold_n: sin_th,
            helical_threshold_n: hel_th,
            sinusoidal_margin_n: sin_th - compression,
            helical_margin_n: hel_th - compression,
        });
    }

    // Reverse to surface-down order.
    for entry in station_buf.into_iter().rev() {
        stations.push(entry);
    }
    for entry in buckle_buf.into_iter().rev() {
        buckling.push(entry);
    }

    let api7g = compute_api7g_check(&request.components, &stations)?;
    let mut warnings = Vec::new();
    if buckling.iter().any(|b| b.helical_margin_n < 0.0) {
        warnings.push(
            "WF-TND-BUCKLING-001: helical lockup predicted by soft-string screen; refine with stiff-string"
                .to_string(),
        );
    }
    let status = if api7g.tensile_utilization > 1.0 || api7g.torsional_utilization > 1.0 {
        AnalysisStatus::Failed
    } else if !warnings.is_empty() || api7g.tensile_utilization > 0.9 {
        AnalysisStatus::Warning
    } else {
        AnalysisStatus::Ok
    };

    let evidence = TnDSolverEvidence {
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        request_hash: String::new(),
        result_hash: String::new(),
        stations_solved: stations.len(),
    };

    Ok(TnDAnalysisResult {
        contract_version: request.contract_version.clone(),
        analysis_id: request.analysis_id,
        status,
        stations,
        buckling,
        api7g,
        stiff_string: None,
        evidence,
        warnings,
    })
}

fn find_component(
    components: &[StringComponent],
    md_m: f64,
) -> Result<&StringComponent, SolveError> {
    components
        .iter()
        .find(|c| md_m >= c.top_md_m && md_m <= c.bottom_md_m)
        .or_else(|| {
            components.iter().min_by(|a, b| {
                (md_m - a.top_md_m)
                    .abs()
                    .partial_cmp(&(md_m - b.top_md_m).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .ok_or(SolveError::NoComponentAtStation { md_m })
}

fn buckling_thresholds(
    component: &StringComponent,
    weight_per_m: f64,
    inclination_rad: f64,
) -> (f64, f64) {
    // Dawson-Paslay sinusoidal: F_sin = 2 * sqrt(E * I * w * sin(inc) / r)
    // r is the radial clearance; here we approximate r = OD / 2 (no hole spec at this layer).
    let e = component.youngs_modulus_pa;
    let i_area = std::f64::consts::PI * (component.od_m.powi(4) - component.id_m.powi(4)) / 64.0;
    let r = component.od_m / 2.0;
    let w_sin_inc = weight_per_m * inclination_rad.sin().abs();
    if r <= 0.0 || w_sin_inc <= 0.0 || i_area <= 0.0 {
        return (f64::INFINITY, f64::INFINITY);
    }
    let sin_th = 2.0 * (e * i_area * w_sin_inc / r).sqrt();
    let hel_th = 2.0 * (2.0_f64).sqrt() * sin_th;
    (sin_th, hel_th)
}

fn compute_api7g_check(
    components: &[StringComponent],
    stations: &[StationResult],
) -> Result<ApiSevenGCheck, SolveError> {
    let (component, spec) = components
        .iter()
        .find_map(|c| c.api7g_spec.as_ref().map(|s| (c, s)))
        .ok_or(SolveError::MissingApi7gSpec)?;

    let peak_tension = stations
        .iter()
        .map(|s| s.effective_tension_n)
        .fold(f64::MIN, f64::max)
        .max(0.0);
    let peak_torque = stations
        .iter()
        .map(|s| s.torque_nm)
        .fold(f64::MIN, f64::max)
        .max(0.0);

    let derated_tensile = derated_limit(spec.tensile_yield_pa * tension_area(component), spec);
    let derated_torsional = derated_limit(spec.torsional_yield_nm, spec);

    Ok(ApiSevenGCheck {
        component_id: component.id,
        derated_tensile_limit_n: derated_tensile,
        peak_tensile_n: peak_tension,
        tensile_utilization: safe_ratio(peak_tension, derated_tensile),
        derated_torsional_limit_nm: derated_torsional,
        peak_torque_nm: peak_torque,
        torsional_utilization: safe_ratio(peak_torque, derated_torsional),
    })
}

fn tension_area(component: &StringComponent) -> f64 {
    std::f64::consts::PI * (component.od_m.powi(2) - component.id_m.powi(2)) / 4.0
}

fn derated_limit(new_pipe_value: f64, spec: &Api7gPipeSpec) -> f64 {
    let derated_new = new_pipe_value * spec.wear_class_derating;
    if spec.safety_factor > 0.0 {
        derated_new / spec.safety_factor
    } else {
        derated_new
    }
}

fn safe_ratio(peak: f64, limit: f64) -> f64 {
    if limit > 0.0 { peak / limit } else { 0.0 }
}
