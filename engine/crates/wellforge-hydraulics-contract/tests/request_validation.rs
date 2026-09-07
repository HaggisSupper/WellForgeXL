//! Legacy JSON requests must reject invalid geometry without changing valid fixtures.

use wellforge_hydraulics_contract::{HydraulicsAnalysisRequest, validate_request};

fn legacy() -> HydraulicsAnalysisRequest {
    serde_json::from_str(include_str!(
        "../../wellforge-hydraulics-fixtures/data/v0_1_request.json"
    ))
    .unwrap()
}

#[test]
fn legacy_rejects_nonfinite_and_negative_md() {
    for top in [f64::NAN, f64::NEG_INFINITY, -1.0] {
        let mut request = legacy();
        request.sections[0].top_md_m = top;
        assert!(validate_request(&request).is_err(), "accepted {top}");
    }
}

#[test]
fn legacy_rejects_nonfinite_and_nonpositive_diameters() {
    for diameter in [f64::NAN, f64::NEG_INFINITY, 0.0, -0.1] {
        let mut request = legacy();
        request.sections[0].string_id_m = diameter;
        assert!(validate_request(&request).is_err(), "accepted {diameter}");
    }
}

#[test]
fn legacy_rejects_invalid_surface_temperature() {
    for temperature in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let mut request = legacy();
        request.operating.surface_temperature_k = temperature;
        assert!(
            validate_request(&request).is_err(),
            "accepted {temperature}"
        );
    }
}
