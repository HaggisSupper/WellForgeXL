//! Contract validation acceptance tests.

use uuid::Uuid;
use wellforge_bha_contract::{
    BhaAnalysisRequest, BhaComponent, ComponentRepresentation, HoleSection, OperatingPoint,
    SolverSettings, TrajectoryStation, validate_request,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

fn source(object_type: WitsmlObjectType, seed: u128) -> SourceObjectRef {
    SourceObjectRef {
        uuid: Uuid::from_u128(seed),
        uri: Some(format!("eml:///wellforge/{seed}")),
        object_type,
        content_hash: format!("sha256:{seed:064x}"),
        citation_name: format!("fixture-{seed}"),
        source_system: "wellforge-test".to_owned(),
    }
}

fn valid_request() -> BhaAnalysisRequest {
    BhaAnalysisRequest {
        contract_version: "1.0.0".to_owned(),
        analysis_id: Uuid::from_u128(99),
        sources: vec![
            source(WitsmlObjectType::Well, 1),
            source(WitsmlObjectType::Wellbore, 2),
            source(WitsmlObjectType::Trajectory, 3),
            source(WitsmlObjectType::WellboreGeometry, 4),
            source(WitsmlObjectType::Tubular, 5),
            source(WitsmlObjectType::BhaRun, 6),
        ],
        trajectory: vec![
            TrajectoryStation {
                md_m: 0.0,
                inclination_rad: std::f64::consts::FRAC_PI_2,
                azimuth_rad: 0.0,
            },
            TrajectoryStation {
                md_m: 30.0,
                inclination_rad: std::f64::consts::FRAC_PI_2,
                azimuth_rad: 0.0,
            },
        ],
        components: vec![BhaComponent {
            id: Uuid::from_u128(10),
            name: "Drill collar".to_owned(),
            representation: ComponentRepresentation::Beam,
            top_md_m: 0.0,
            bottom_md_m: 30.0,
            od_m: 0.2032,
            id_m: 0.0714,
            youngs_modulus_pa: 206.84e9,
            density_kg_m3: 7850.0,
        }],
        hole: vec![HoleSection {
            top_md_m: 0.0,
            bottom_md_m: 30.0,
            diameter_m: 0.31115,
        }],
        operating: OperatingPoint {
            wob_n: 100_000.0,
            rpm: 120.0,
            fluid_density_kg_m3: 1200.0,
        },
        solver: SolverSettings::default(),
    }
}

#[test]
fn request_requires_complete_witsml_source_set() {
    let mut request = valid_request();
    request
        .sources
        .retain(|source| source.object_type != WitsmlObjectType::Tubular);
    let errors = validate_request(&request).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == "WF-BHA-CONTRACT-004")
    );
}

#[test]
fn request_rejects_impossible_component_geometry() {
    let mut request = valid_request();
    request.components[0].id_m = request.components[0].od_m;
    let errors = validate_request(&request).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == "WF-BHA-CONTRACT-012")
    );
}

#[test]
fn release_one_rejects_unimplemented_component_representations() {
    let mut request = valid_request();
    request.components[0].representation = ComponentRepresentation::Rigid;
    let errors = validate_request(&request).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == "WF-BHA-CONTRACT-021")
    );
}

#[test]
fn request_rejects_unknown_json_fields() {
    let mut value = serde_json::to_value(valid_request()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("mystery".to_owned(), serde_json::json!(1));
    assert!(serde_json::from_value::<BhaAnalysisRequest>(value).is_err());
}
