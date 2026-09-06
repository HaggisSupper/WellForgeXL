use uuid::Uuid;
use wellforge_torque_drag_contract::{
    StiffStringMode, TnDAnalysisRequest, TnDHoleSection, TnDSolverOptions, validate_request,
};

const LEGACY_REQUEST: &str = r#"
{
  "contract_version": "0.1.0",
  "analysis_id": "2b7f9d1c-44a1-4d31-a92e-7c5b8f200602",
  "sources": [
    {
      "uuid": "2b7f9d1c-44a1-4d31-a92e-7c5b8f200603",
      "uri": null,
      "object_type": "tubular",
      "content_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "citation_name": "legacy-contract-test",
      "source_system": "wellforge-test"
    }
  ],
  "components": [
    {
      "id": "2b7f9d1c-44a1-4d31-a92e-7c5b8f200601",
      "name": "test pipe",
      "top_md_m": 0.0,
      "bottom_md_m": 100.0,
      "od_m": 0.127,
      "id_m": 0.1086,
      "linear_weight_kg_m": 29.79,
      "youngs_modulus_pa": 207000000000.0,
      "density_kg_m3": 7850.0,
      "api7g_spec": {
        "grade": "S135",
        "tensile_yield_pa": 931000000.0,
        "torsional_yield_nm": 59000.0,
        "wear_class_derating": 0.8,
        "safety_factor": 1.1
      }
    }
  ],
  "trajectory": [
    {"md_m": 0.0, "inclination_rad": 0.5, "azimuth_rad": 0.0, "tvd_m": 0.0, "cased": false},
    {"md_m": 100.0, "inclination_rad": 0.5, "azimuth_rad": 0.1, "tvd_m": 87.758256, "cased": false}
  ],
  "operating": {
    "state": "pickup",
    "weight_on_bit_n": 0.0,
    "torque_on_bit_nm": 0.0,
    "surface_rpm_rad_s": 0.0,
    "friction_factor_open_hole": 0.3,
    "friction_factor_cased_hole": 0.2,
    "mud_density_kg_m3": 1200.0
  }
}
"#;

#[test]
fn legacy_request_without_hole_or_solver_uses_compatible_defaults() {
    let request: TnDAnalysisRequest = serde_json::from_str(LEGACY_REQUEST).expect("legacy request");
    assert!(request.hole.is_empty());
    assert_eq!(request.solver.stiff_string_mode, StiffStringMode::Auto);
    assert!(request.solver.max_element_length_m > 0.0);
    assert!(validate_request(&request).is_ok());
}

#[test]
fn default_stiff_options_are_positive_and_bounded() {
    let options = TnDSolverOptions::default();
    assert_eq!(options.stiff_string_mode, StiffStringMode::Auto);
    assert!(options.severity_normal_load_n_m > 0.0);
    assert!(options.severity_buckling_margin_n >= 0.0);
    assert!(options.transition_buffer_m >= 0.0);
    assert!(options.max_element_length_m > 0.0);
    assert!(options.contact_penalty_n_m > 0.0);
}

#[test]
fn invalid_hole_geometry_is_rejected() {
    let mut request: TnDAnalysisRequest = serde_json::from_str(LEGACY_REQUEST).expect("legacy request");
    request.hole.push(TnDHoleSection {
        id: Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0610),
        top_md_m: 0.0,
        bottom_md_m: 100.0,
        diameter_m: -0.216,
    });
    let errors = validate_request(&request).expect_err("negative diameter must fail");
    assert!(errors.iter().any(|error| error.code == "WF-TND-REQ-050"));
}

#[test]
fn overlapping_hole_sections_are_rejected() {
    let mut request: TnDAnalysisRequest = serde_json::from_str(LEGACY_REQUEST).expect("legacy request");
    request.hole = vec![
        TnDHoleSection {
            id: Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0611),
            top_md_m: 0.0,
            bottom_md_m: 70.0,
            diameter_m: 0.216,
        },
        TnDHoleSection {
            id: Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0612),
            top_md_m: 60.0,
            bottom_md_m: 100.0,
            diameter_m: 0.216,
        },
    ];
    let errors = validate_request(&request).expect_err("overlap must fail");
    assert!(errors.iter().any(|error| error.code == "WF-TND-REQ-051"));
}
