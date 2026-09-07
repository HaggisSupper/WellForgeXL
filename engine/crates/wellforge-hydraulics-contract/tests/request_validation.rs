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

#[test]
fn legacy_rejects_nil_section_identity() {
    let mut request = legacy();
    request.sections[0].id = uuid::Uuid::nil();
    let errors = validate_request(&request).expect_err("nil section ID cannot be verified");
    assert!(errors.iter().any(|error| error.code == "WF-HYD-REQ-016"));
}

#[test]
fn legacy_rejects_repeated_section_identity() {
    let mut request = legacy();
    let mut later = request.sections[0].clone();
    later.top_md_m = later.bottom_md_m;
    later.bottom_md_m += 100.0;
    request.sections.push(later);
    let errors = validate_request(&request).expect_err("repeated section ID cannot be verified");
    assert!(errors.iter().any(|error| error.code == "WF-HYD-REQ-016"));
}

#[test]
fn legacy_rejects_control_characters_in_profile_standard() {
    for control in ['\n', '\r', '\t', '\0', '\u{007f}', '\u{0085}'] {
        let mut request = legacy();
        request.profile.standard = format!("API{control}RP 13D");
        let errors = validate_request(&request).expect_err("unsafe standard cannot be verified");
        assert!(errors.iter().any(|error| error.code == "WF-HYD-REQ-002"));
    }
}

#[test]
fn legacy_rejects_control_characters_in_profile_edition() {
    for control in ['\n', '\r', '\t', '\0', '\u{007f}', '\u{0085}'] {
        let mut request = legacy();
        request.profile.edition = format!("7th{control}Edition");
        let errors = validate_request(&request).expect_err("unsafe edition cannot be verified");
        assert!(errors.iter().any(|error| error.code == "WF-HYD-REQ-002"));
    }
}
