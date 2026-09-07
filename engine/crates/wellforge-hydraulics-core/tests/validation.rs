//! Regressions for invalid direct solver inputs and numerical range failures.

use wellforge_hydraulics_contract::{HydraulicsAnalysisRequest, RheologyModel, validate_request};
use wellforge_hydraulics_core::{solve_hydraulics, solve_hydraulics_batch};
use wellforge_hydraulics_fixtures::{canonical_bingham_case, generalized_yield_power_law_case};

// Removing a public-boundary guard must make its corresponding case accept bad input.
macro_rules! rejects_request {
    ($name:ident, $mutate:expr) => {
        #[test]
        fn $name() {
            for mut request in [canonical_bingham_case(), generalized_yield_power_law_case()] {
                ($mutate)(&mut request);
                assert!(
                    solve_hydraulics(&request).is_err(),
                    "accepted {}",
                    request.contract_version
                );
            }
        }
    };
}

rejects_request!(zero_flow, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .flow_rate_m3_s =
    0.0);
rejects_request!(negative_flow, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .flow_rate_m3_s =
    -0.03);
rejects_request!(nan_flow, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .flow_rate_m3_s =
    f64::NAN);
rejects_request!(infinite_flow, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .flow_rate_m3_s =
    f64::INFINITY);
rejects_request!(zero_density, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .mud_density_kg_m3 =
    0.0);
rejects_request!(negative_density, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .mud_density_kg_m3 =
    -1200.0);
rejects_request!(nan_density, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .mud_density_kg_m3 =
    f64::NAN);
rejects_request!(infinite_density, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .mud_density_kg_m3 =
    f64::INFINITY);
rejects_request!(zero_temperature, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .surface_temperature_k =
    0.0);
rejects_request!(nan_temperature, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .surface_temperature_k =
    f64::NAN);
rejects_request!(reversed_md, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .top_md_m =
    4000.0);
rejects_request!(zero_length, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .bottom_md_m =
    0.0);
rejects_request!(negative_md, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .top_md_m =
    -1.0);
rejects_request!(nan_md, |r: &mut HydraulicsAnalysisRequest| r.sections[0]
    .top_md_m =
    f64::NAN);
rejects_request!(infinite_md, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .bottom_md_m =
    f64::INFINITY);
rejects_request!(zero_pipe_id, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .string_id_m =
    0.0);
rejects_request!(
    reversed_string_diameters,
    |r: &mut HydraulicsAnalysisRequest| r.sections[0].string_od_m = 0.05
);
rejects_request!(closed_annulus, |r: &mut HydraulicsAnalysisRequest| r
    .sections[0]
    .hole_id_m =
    0.127);
rejects_request!(nan_hole, |r: &mut HydraulicsAnalysisRequest| r.sections
    [0]
.hole_id_m =
    f64::NAN);
rejects_request!(overflowed_geometry, |r: &mut HydraulicsAnalysisRequest| {
    r.rheology.model = RheologyModel::Newtonian;
    r.rheology.dynamic_viscosity_pa_s = Some(0.02);
    r.sections[0].string_id_m = 1.0e155;
    r.sections[0].string_od_m = 2.0e155;
    r.sections[0].hole_id_m = 3.0e155;
});
rejects_request!(underflowed_geometry, |r: &mut HydraulicsAnalysisRequest| {
    r.sections[0].string_id_m = 1.0e-200;
    r.sections[0].string_od_m = 2.0e-200;
    r.sections[0].hole_id_m = 3.0e-200;
});
rejects_request!(tiny_nozzle, |r: &mut HydraulicsAnalysisRequest| {
    r.operating.nozzles.truncate(1);
    r.operating.nozzles[0].diameter_m = 1.0e-160;
});
rejects_request!(underflowed_nozzle, |r: &mut HydraulicsAnalysisRequest| {
    r.operating.nozzles.truncate(1);
    r.operating.nozzles[0].diameter_m = 1.0e-200;
});
rejects_request!(huge_nozzle, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .nozzles[0]
    .diameter_m =
    1.0e200);
rejects_request!(
    overflowed_nozzle_sum,
    |r: &mut HydraulicsAnalysisRequest| {
        // Each area is finite (PI * 7e153^2 / 4); five areas overflow their sum.
        r.operating.nozzles = vec![
            wellforge_hydraulics_contract::Nozzle {
                diameter_m: 7.0e153
            };
            5
        ];
    }
);
rejects_request!(
    underflowed_bit_pressure,
    |r: &mut HydraulicsAnalysisRequest| {
        r.operating.nozzles[0].diameter_m = 1.0e100;
    }
);
rejects_request!(tiny_later_nozzle, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .nozzles[2]
    .diameter_m =
    1.0e-200);
rejects_request!(negative_nozzle, |r: &mut HydraulicsAnalysisRequest| r
    .operating
    .nozzles[0]
    .diameter_m =
    -0.01);
rejects_request!(
    negative_bingham_yield,
    |r: &mut HydraulicsAnalysisRequest| {
        r.rheology.model = RheologyModel::Bingham;
        r.rheology.plastic_viscosity_pa_s = Some(0.02);
        r.rheology.yield_stress_pa = Some(-0.001);
    }
);
rejects_request!(missing_rheology, |r: &mut HydraulicsAnalysisRequest| r
    .rheology
    .yield_stress_pa =
    None);
rejects_request!(
    overflowed_section_pressure,
    |r: &mut HydraulicsAnalysisRequest| r.sections[0].bottom_md_m = f64::MAX
);
rejects_request!(
    underflowed_section_pressure,
    |r: &mut HydraulicsAnalysisRequest| r.sections[0].bottom_md_m = f64::from_bits(1)
);

#[test]
fn later_shared_candidate_is_validated_and_numerically_checked() {
    let mut accepted = Vec::new();
    for diameter_m in [-0.01, 0.0, f64::NAN, 1.0e-160, 1.0e200] {
        let first = generalized_yield_power_law_case();
        let mut later = first.clone();
        later.operating.nozzles = vec![wellforge_hydraulics_contract::Nozzle { diameter_m }];
        if solve_hydraulics_batch(&[first, later]).is_ok() {
            accepted.push(diameter_m);
        }
    }
    assert!(
        accepted.is_empty(),
        "accepted later candidates: {accepted:?}"
    );
}

#[test]
fn heterogeneous_batch_rejects_invalid_later_result() {
    let first = generalized_yield_power_law_case();
    let mut later = canonical_bingham_case();
    later.operating.nozzles[0].diameter_m = 1.0e200;
    assert!(solve_hydraulics_batch(&[first, later]).is_err());
}

#[test]
fn empty_nozzles_retain_versioned_policy_in_single_and_shared_solves() {
    let mut legacy = canonical_bingham_case();
    legacy.operating.nozzles.clear();
    validate_request(&legacy).unwrap();
    let result = solve_hydraulics(&legacy).unwrap();
    // Preserve the existing iterator sum's negative-zero identity exactly.
    assert_eq!(result.total_flow_area_m2.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(result.bit_pressure_loss_pa.to_bits(), 0.0_f64.to_bits());
    assert!(solve_hydraulics_batch(&[canonical_bingham_case(), legacy]).is_ok());

    let current = generalized_yield_power_law_case();
    let mut empty = current.clone();
    empty.operating.nozzles.clear();
    assert!(solve_hydraulics(&empty).is_err());
    assert!(solve_hydraulics_batch(&[current, empty]).is_err());
}

#[test]
fn horizontal_tvd_solves_with_unattached_evidence() {
    let mut request = generalized_yield_power_law_case();
    request.sections[0].top_md_m = 1000.0;
    request.sections[0].top_tvd_m = Some(900.0);
    request.sections[0].bottom_tvd_m = Some(900.0);
    request.operating.ecd_reference_tvd_m = None;
    let result = solve_hydraulics(&request).unwrap();
    assert_eq!(result.reference_vertical_depth_m, Some(900.0));
    assert!(result.evidence.request_hash.is_empty());
    assert!(result.evidence.result_hash.is_empty());
}

#[test]
fn overflowed_ecd_denominator_is_rejected() {
    let mut request = generalized_yield_power_law_case();
    request.operating.ecd_reference_tvd_m = Some(f64::MAX);
    assert!(solve_hydraulics(&request).is_err());
}

#[test]
fn overflowed_ecd_is_rejected() {
    let mut request = generalized_yield_power_law_case();
    request.operating.ecd_reference_tvd_m = Some(1.0e-300);
    request.operating.surface_backpressure_pa = Some(1.0e100);
    assert!(solve_hydraulics(&request).is_err());
}
