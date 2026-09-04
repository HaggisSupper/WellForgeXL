use std::{
    f64::consts::{FRAC_PI_2, PI},
    path::Path,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use wellforge_3d::ScenePoint;
use wellforge_core::{Metres, Radians};
use wellforge_survey::SurveyPosition;
use wellforge_survey::{
    MagneticField, SurveyStation, Vector3, build_survey_scene,
    calculate_displacement_minimum_curvature, convert_true_to_grid_azimuth, nev_to_hla_dcm,
    root_sum_square, short_collar_corrected_bz,
};

const EPSILON: f64 = 1.0e-10;

#[test]
fn minimum_curvature_vertical_interval_has_only_tvd_displacement() {
    let start = SurveyStation::new(metres(0.0), radians(0.0), radians(0.0));
    let end = SurveyStation::new(metres(30.0), radians(0.0), radians(0.0));

    let result = calculate_displacement_minimum_curvature(&start, &end)
        .expect("a positive measured-depth interval is valid");

    assert!(result.north_m.get().abs() < EPSILON);
    assert!(result.east_m.get().abs() < EPSILON);
    assert!((result.tvd_m.get() - 30.0).abs() < EPSILON);
    assert!(result.dogleg_rad.get().abs() < EPSILON);
}

#[test]
fn minimum_curvature_quarter_turn_has_hand_checked_endpoint() {
    let start = SurveyStation::new(metres(0.0), radians(FRAC_PI_2), radians(0.0));
    let end = SurveyStation::new(metres(100.0), radians(FRAC_PI_2), radians(FRAC_PI_2));

    let result = calculate_displacement_minimum_curvature(&start, &end)
        .expect("quarter-turn interval is valid");
    let expected = 200.0 / PI;

    assert!((result.north_m.get() - expected).abs() < EPSILON);
    assert!((result.east_m.get() - expected).abs() < EPSILON);
    assert!(result.tvd_m.get().abs() < EPSILON);
    assert!((result.dogleg_rad.get() - FRAC_PI_2).abs() < EPSILON);
}

#[test]
fn minimum_curvature_rejects_a_finite_input_pair_that_overflows_derived_severity() {
    let start = SurveyStation::new(metres(0.0), radians(0.0), radians(0.0));
    let end = SurveyStation::new(metres(f64::from_bits(1)), radians(PI), radians(0.0));

    let error = calculate_displacement_minimum_curvature(&start, &end)
        .expect_err("derived dogleg severity must remain finite");

    assert_eq!(error.code(), "NON_FINITE_RESULT");
}

#[test]
fn typed_station_contract_preserves_existing_wire_field_names() {
    let station = SurveyStation::new(metres(12.5), radians(0.25), radians(1.5));

    let value = serde_json::to_value(station).expect("station serializes");

    assert_eq!(value["md_m"], 12.5);
    assert_eq!(value["inclination_rad"], 0.25);
    assert_eq!(value["azimuth_true_rad"], 1.5);
}

#[test]
fn short_collar_bz_preserves_sign_and_recovers_axial_field() {
    let corrected = short_collar_corrected_bz(MagneticField::new(3.0, 4.0, -1.0), 13.0)
        .expect("total field exceeds transverse field");

    assert!((corrected + 12.0).abs() < EPSILON);
}

#[test]
fn short_collar_bz_rejects_impossible_total_field() {
    let error = short_collar_corrected_bz(MagneticField::new(3.0, 4.0, 1.0), 4.9)
        .expect_err("total field cannot be smaller than transverse field");

    assert_eq!(error.code(), "INVALID_MAGNETIC_FIELD");
}

#[test]
fn true_to_grid_wraps_negative_angles() {
    let grid = convert_true_to_grid_azimuth(0.1, 0.2);

    assert!((grid - (2.0 * PI - 0.1)).abs() < EPSILON);
}

#[test]
fn hla_dcm_is_orthonormal_and_right_handed() {
    let dcm = nev_to_hla_dcm(FRAC_PI_2, 0.0).expect("horizontal north-facing well has a high side");
    let h = Vector3::new(dcm.rows[0][0], dcm.rows[0][1], dcm.rows[0][2]);
    let l = Vector3::new(dcm.rows[1][0], dcm.rows[1][1], dcm.rows[1][2]);
    let a = Vector3::new(dcm.rows[2][0], dcm.rows[2][1], dcm.rows[2][2]);

    assert!(h.dot(l).abs() < EPSILON);
    assert!(h.dot(a).abs() < EPSILON);
    assert!(l.dot(a).abs() < EPSILON);
    assert!((h.cross(l) - a).norm().abs() < EPSILON);
}

#[test]
fn root_sum_square_uses_vector_norm() {
    assert!((root_sum_square(&[3.0, 4.0]).expect("finite values") - 5.0).abs() < EPSILON);
}

#[test]
fn survey_scene_preserves_cumulative_ne_tvd_coordinates() {
    let scene = build_survey_scene(&[
        SurveyPosition {
            md_m: metres(0.0),
            north_m: metres(0.0),
            east_m: metres(0.0),
            tvd_m: metres(0.0),
        },
        SurveyPosition {
            md_m: metres(105.0),
            north_m: metres(20.0),
            east_m: metres(10.0),
            tvd_m: metres(100.0),
        },
    ])
    .expect("finite cumulative stations are renderable");

    let points = scene.layers[0].primitives[0].points();
    assert_eq!(points[1], ScenePoint::new(20.0, 10.0, 100.0));
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinimumCurvatureFixture {
    schema_version: String,
    stations: Vec<SurveyStation>,
    expected_displacement: FixtureDisplacement,
}

#[derive(Deserialize)]
struct FixtureDisplacement {
    north_m: Metres,
    east_m: Metres,
    tvd_m: Metres,
    dogleg_rad: Radians,
    dogleg_severity_rad_per_m: f64,
}

#[derive(Deserialize)]
struct FixtureManifest {
    fixtures: Vec<FixtureManifestEntry>,
}

#[derive(Deserialize)]
struct FixtureManifestEntry {
    path: String,
    sha256: String,
}

#[test]
fn minimum_curvature_fixture_is_hashed_and_conformant() {
    const FIXTURE: &str = include_str!("fixtures/minimum-curvature-synthetic.json");
    const MANIFEST: &str = include_str!("fixtures/manifest.json");

    let manifest: FixtureManifest = serde_json::from_str(MANIFEST).expect("manifest is valid JSON");
    let entry = manifest
        .fixtures
        .iter()
        .find(|entry| entry.path == Path::new("minimum-curvature-synthetic.json").to_string_lossy())
        .expect("fixture hash is listed");
    assert_eq!(
        entry.sha256,
        format!("sha256:{:x}", Sha256::digest(FIXTURE.as_bytes()))
    );

    let fixture: MinimumCurvatureFixture =
        serde_json::from_str(FIXTURE).expect("fixture is valid JSON");
    assert_eq!(fixture.schema_version, "wellforge.minimum-curvature.v1");
    let result =
        calculate_displacement_minimum_curvature(&fixture.stations[0], &fixture.stations[1])
            .expect("fixture stations are valid");
    assert_close(
        result.north_m.get(),
        fixture.expected_displacement.north_m.get(),
    );
    assert_close(
        result.east_m.get(),
        fixture.expected_displacement.east_m.get(),
    );
    assert_close(
        result.tvd_m.get(),
        fixture.expected_displacement.tvd_m.get(),
    );
    assert_close(
        result.dogleg_rad.get(),
        fixture.expected_displacement.dogleg_rad.get(),
    );
    assert_close(
        result.dogleg_severity_rad_per_m,
        fixture.expected_displacement.dogleg_severity_rad_per_m,
    );
}

fn metres(value: f64) -> Metres {
    Metres::try_new(value).expect("test value is finite")
}

fn radians(value: f64) -> Radians {
    Radians::try_new(value).expect("test value is finite")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {expected}, got {actual}"
    );
}
