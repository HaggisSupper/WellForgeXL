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
    StringComponent, TnDAnalysisRequest, TnDAnalysisResult, TnDSolverEvidence,
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
            let dl = if dmd > 0.0 {
                ((s_lower.inclination_rad - s_upper.inclination_rad).abs()) / dmd
            } else {
                0.0
            };
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
