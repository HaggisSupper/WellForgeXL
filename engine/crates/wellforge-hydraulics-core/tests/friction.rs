//! Deterministic hydraulics sanity test against the canonical fixture.

use wellforge_hydraulics_contract::{
    ComputeBackend, FlowCorrelation, FlowLoop, HydraulicsSolverOptions, RheologyModel,
    validate_request,
};
use wellforge_hydraulics_core::{solve_hydraulics, solve_hydraulics_batch};
use wellforge_hydraulics_fixtures::{canonical_bingham_case, generalized_yield_power_law_case};

#[test]
fn bingham_case_produces_positive_losses_and_reasonable_ecd() {
    let request = canonical_bingham_case();
    let result = solve_hydraulics(&request).expect("solver must succeed");

    // For a positive flow rate we expect strictly positive pressure drops.
    assert!(result.total_pipe_pressure_loss_pa > 0.0);
    assert!(result.total_annulus_pressure_loss_pa > 0.0);
    assert!(result.bit_pressure_loss_pa > 0.0);

    // ECD must be within a physically sensible band for a 1200 kg/m^3 mud.
    let ecd = result.equivalent_circulating_density_kg_m3;
    assert!((1150.0..=1500.0).contains(&ecd), "ECD out of band: {ecd}");

    // Each section should emit exactly two records (pipe + annulus).
    assert_eq!(result.sections.len(), request.sections.len() * 2);
}

#[test]
fn generalized_newtonian_pipe_limit_matches_si_reynolds_number() {
    let mut request = generalized_yield_power_law_case();
    request.rheology.model = RheologyModel::Newtonian;
    request.rheology.dynamic_viscosity_pa_s = Some(0.020);
    request.rheology.yield_stress_pa = None;
    request.rheology.plastic_viscosity_pa_s = None;
    request.rheology.high_shear_flow_index = None;
    request.solver.as_mut().unwrap().flow_correlation = FlowCorrelation::GeneralizedYieldPowerLaw;

    let result = solve_hydraulics(&request).expect("solver must succeed");
    let pipe = &result.sections[0];
    let expected_re = request.operating.mud_density_kg_m3
        * pipe.bulk_velocity_m_s
        * request.sections[0].string_id_m
        / request.rheology.dynamic_viscosity_pa_s.unwrap();

    assert!((pipe.reynolds_number - expected_re).abs() <= expected_re * 1.0e-12);
    assert!(pipe.fanning_friction_factor.is_finite());
    assert!(pipe.fanning_friction_factor > 0.0);
}

#[test]
fn parallel_cpu_backend_preserves_order_and_numerical_results() {
    let mut serial_request = generalized_yield_power_law_case();
    let template = serial_request.sections[0].clone();
    serial_request.sections = (0_u32..64)
        .map(|index| {
            let mut section = template.clone();
            section.id = uuid::Uuid::from_u128(
                0x35b1_5a48_47c1_4d31_a92e_0000_0000_0000 + u128::from(index),
            );
            section.top_md_m = f64::from(index) * 50.0;
            section.bottom_md_m = section.top_md_m + 50.0;
            section.string_id_m += f64::from(index) * 1.0e-5;
            section.string_od_m += f64::from(index) * 1.0e-5;
            section
        })
        .collect();
    serial_request.solver.as_mut().unwrap().compute_backend = ComputeBackend::SerialCpu;
    let serial = solve_hydraulics(&serial_request).expect("serial solve");

    let mut parallel_request = serial_request;
    parallel_request.solver.as_mut().unwrap().compute_backend = ComputeBackend::ParallelCpu;
    let parallel = solve_hydraulics(&parallel_request).expect("parallel solve");

    assert_eq!(serial.sections, parallel.sections);
    assert_eq!(
        serial.total_pipe_pressure_loss_pa.to_bits(),
        parallel.total_pipe_pressure_loss_pa.to_bits()
    );
    assert_eq!(
        serial.total_annulus_pressure_loss_pa.to_bits(),
        parallel.total_annulus_pressure_loss_pa.to_bits()
    );
}

#[test]
fn nozzle_coefficient_obeys_inverse_square_pressure_scaling() {
    let mut reference_request = generalized_yield_power_law_case();
    reference_request.operating.nozzle_discharge_coefficient = Some(0.95);
    let reference = solve_hydraulics(&reference_request).expect("reference solve");

    let mut restricted_request = reference_request;
    restricted_request.operating.nozzle_discharge_coefficient = Some(0.50);
    let restricted = solve_hydraulics(&restricted_request).expect("restricted solve");

    let expected_ratio = (0.95_f64 / 0.50).powi(2);
    let actual_ratio = restricted.bit_pressure_loss_pa / reference.bit_pressure_loss_pa;
    assert!((actual_ratio - expected_ratio).abs() <= 1.0e-12);
}

#[test]
fn shared_state_nozzle_batch_matches_individual_solves() {
    let base = generalized_yield_power_law_case();
    let requests: Vec<_> = [0.0080, 0.0085, 0.0090, 0.0095, 0.0100]
        .into_iter()
        .map(|diameter_m| {
            let mut request = base.clone();
            for nozzle in &mut request.operating.nozzles {
                nozzle.diameter_m = diameter_m;
            }
            request
        })
        .collect();
    let independent: Vec<_> = requests
        .iter()
        .map(|request| solve_hydraulics(request).expect("independent solve"))
        .collect();

    let batched = solve_hydraulics_batch(&requests).expect("batched solve");

    assert_eq!(batched, independent);
}

#[test]
fn ecd_uses_true_vertical_depth_and_surface_backpressure() {
    let mut request = generalized_yield_power_law_case();
    request.operating.ecd_reference_tvd_m = Some(1500.0);
    request.operating.surface_backpressure_pa = Some(100_000.0);

    let result = solve_hydraulics(&request).expect("solver must succeed");
    let expected = request.operating.mud_density_kg_m3
        + (result.total_annulus_pressure_loss_pa
            + request.operating.surface_backpressure_pa.unwrap())
            / (9.80665 * 1500.0);

    assert!((result.reference_vertical_depth_m.unwrap() - 1500.0).abs() <= f64::EPSILON);
    assert!((result.equivalent_circulating_density_kg_m3 - expected).abs() <= 1.0e-10);
}

#[test]
fn validation_rejects_nonphysical_additive_controls() {
    let mut request = generalized_yield_power_law_case();
    request.operating.nozzle_discharge_coefficient = Some(0.0);
    request.operating.surface_backpressure_pa = Some(-1.0);
    request.sections[0].top_tvd_m = Some(1000.0);
    request.sections[0].bottom_tvd_m = Some(900.0);

    let errors = validate_request(&request).expect_err("request must be rejected");
    let codes: Vec<_> = errors.iter().map(|error| error.code).collect();

    assert!(codes.contains(&"WF-HYD-REQ-023"));
    assert!(codes.contains(&"WF-HYD-REQ-024"));
    assert!(codes.contains(&"WF-HYD-REQ-014"));
}

#[test]
fn validation_accepts_horizontal_tvd_interval() {
    let mut request = generalized_yield_power_law_case();
    request.sections[0].top_md_m = 1000.0;
    request.sections[0].bottom_md_m = 2000.0;
    request.sections[0].top_tvd_m = Some(900.0);
    request.sections[0].bottom_tvd_m = Some(900.0);

    validate_request(&request).expect("horizontal intervals have non-decreasing TVD");
}

#[test]
fn version_one_rejects_presence_of_version_two_controls() {
    let mut request = canonical_bingham_case();
    request.solver = Some(HydraulicsSolverOptions::default());

    let errors = validate_request(&request).expect_err("feature smuggling must be rejected");

    assert!(errors.iter().any(|error| error.code == "WF-HYD-REQ-040"));
}

#[test]
fn selected_flow_loop_controls_pressure_aggregation() {
    let mut both_request = generalized_yield_power_law_case();
    both_request.sections[0].active_flow_loop = None;
    let both = solve_hydraulics(&both_request).expect("both loops");

    let mut pipe_request = both_request.clone();
    pipe_request.sections[0].active_flow_loop = Some(FlowLoop::Pipe);
    let pipe = solve_hydraulics(&pipe_request).expect("pipe loop");

    let mut annulus_request = both_request;
    annulus_request.sections[0].active_flow_loop = Some(FlowLoop::Annulus);
    let annulus = solve_hydraulics(&annulus_request).expect("annulus loop");

    assert!(pipe.total_pipe_pressure_loss_pa > 0.0);
    assert!(pipe.total_annulus_pressure_loss_pa.abs() <= f64::EPSILON);
    assert!(annulus.total_pipe_pressure_loss_pa.abs() <= f64::EPSILON);
    assert!(annulus.total_annulus_pressure_loss_pa > 0.0);
    assert_eq!(
        both.total_pipe_pressure_loss_pa.to_bits(),
        pipe.total_pipe_pressure_loss_pa.to_bits()
    );
    assert_eq!(
        both.total_annulus_pressure_loss_pa.to_bits(),
        annulus.total_annulus_pressure_loss_pa.to_bits()
    );
}

#[test]
fn version_one_result_omits_version_two_evidence() {
    let result = solve_hydraulics(&canonical_bingham_case()).expect("compatibility solve");
    let json = serde_json::to_value(&result).unwrap();

    assert!(result.reference_vertical_depth_m.is_none());
    assert!(result.surface_backpressure_pa.is_none());
    assert!(result.circulating_system_pressure_pa.is_none());
    assert!(result.evidence.flow_correlation.is_none());
    assert!(result.evidence.compute_backend.is_none());
    assert!(
        result
            .sections
            .iter()
            .all(|section| section.flow_regime.is_none())
    );
    assert!(json.get("reference_vertical_depth_m").is_none());
    assert!(json["evidence"].get("flow_correlation").is_none());
    assert!(json["sections"][0].get("flow_regime").is_none());
}
