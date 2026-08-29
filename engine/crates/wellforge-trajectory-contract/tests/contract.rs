//! Trajectory request boundary and identity acceptance tests.

use uuid::Uuid;
use wellforge_trajectory_contract::{
    ApplicabilityStatement, AzimuthReference, CalculatedStation, CalculationEvidence,
    FormationEvaluation, FormationPick, InterpolationResult, InterpolationStatus, MdDatum,
    MdDatumKind, PlanSurveyResidual, PositionResidual, ProjectionAssessment, ProjectionRequest,
    SlideAssessment, SlideInterval, SpatialPosition, StationKind, Target, TargetAssessment,
    TargetBasis, TargetEvaluation, TargetKind, TrajectoryAnalysisRequest, TrajectoryAnalysisResult,
    TrajectoryAnalysisStatus, TrajectoryCalculation, TrajectorySourceSet, TrajectoryStation,
    validate_request,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

fn source(object_type: WitsmlObjectType, seed: u128) -> SourceObjectRef {
    SourceObjectRef {
        uuid: Uuid::from_u128(seed),
        uri: Some(format!("eml:///wellforge/trajectory/{seed}")),
        object_type,
        content_hash: format!("sha256:{seed:064x}"),
        citation_name: format!("fixture-{seed}"),
        source_system: "wellforge-test".to_owned(),
    }
}

fn station(seed: u128, md_m: f64, kind: StationKind) -> TrajectoryStation {
    TrajectoryStation {
        uid: Uuid::from_u128(seed),
        kind,
        md_m,
        inclination_rad: 0.1,
        azimuth_rad: 0.2,
    }
}

fn valid_request() -> TrajectoryAnalysisRequest {
    TrajectoryAnalysisRequest {
        contract_version: "1.0.0".to_owned(),
        analysis_id: Uuid::from_u128(100),
        sources: TrajectorySourceSet {
            well: source(WitsmlObjectType::Well, 1),
            wellbore: source(WitsmlObjectType::Wellbore, 2),
            plan_trajectory: source(WitsmlObjectType::Trajectory, 3),
            survey_trajectory: source(WitsmlObjectType::Trajectory, 4),
        },
        md_datum: MdDatum {
            uid: Uuid::from_u128(5),
            name: "Rotary Kelly Bushing".to_owned(),
            kind: MdDatumKind::RotaryKellyBushing,
        },
        azimuth_reference: AzimuthReference::TrueNorth,
        vertical_section_azimuth_rad: 0.0,
        plan: vec![
            station(10, 0.0, StationKind::TieIn),
            station(11, 100.0, StationKind::Plan),
        ],
        survey: vec![
            station(20, 0.0, StationKind::TieIn),
            station(21, 100.0, StationKind::Survey),
        ],
        targets: vec![Target {
            uid: Uuid::from_u128(30),
            name: "Target A".to_owned(),
            kind: TargetKind::Ellipse,
            md_m: 95.0,
            north_m: 10.0,
            east_m: 20.0,
            tvd_m: 90.0,
            major_m: 8.0,
            minor_m: 3.0,
            rotation_rad: 0.5,
            vertical_tolerance_m: 2.0,
        }],
        slides: vec![SlideInterval {
            uid: Uuid::from_u128(40),
            md_in_m: 0.0,
            md_out_m: 100.0,
            slide_length_m: 50.0,
            commanded_toolface_rad: 0.4,
            rotary_build_rad_per_m: 0.0002,
            rotary_effective_turn_rad_per_m: 0.0001,
            low_inclination_threshold_rad: 0.0,
        }],
        formations: vec![FormationPick {
            uid: Uuid::from_u128(50),
            name: "Formation A".to_owned(),
            prognosed_tvd_m: 2_000.0,
            actual_md_m: Some(95.0),
            tolerance_m: Some(3.0),
        }],
        projection: Some(ProjectionRequest {
            bit_md_m: 110.0,
            ahead_m: 40.0,
            build_tendency_rad_per_m: 0.001,
            effective_turn_tendency_rad_per_m: 0.0002,
            low_inclination_threshold_rad: 0.05,
        }),
    }
}

fn has_code(request: &TrajectoryAnalysisRequest, code: &str) -> bool {
    validate_request(request)
        .unwrap_err()
        .iter()
        .any(|error| error.code == code)
}

#[test]
fn accepts_complete_release_one_request() {
    assert_eq!(validate_request(&valid_request()), Ok(()));
}

#[test]
fn rejects_unknown_json_fields() {
    let mut value = serde_json::to_value(valid_request()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("mystery".to_owned(), serde_json::json!(true));
    assert!(serde_json::from_value::<TrajectoryAnalysisRequest>(value).is_err());
}

#[test]
fn rejects_unsupported_contract_major() {
    let mut request = valid_request();
    request.contract_version = "2.0.0".to_owned();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-001"));
}

#[test]
fn rejects_non_semver_contract_version() {
    let mut request = valid_request();
    request.contract_version = "1".to_owned();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-001"));
}

#[test]
fn requires_well_source_type() {
    let mut request = valid_request();
    request.sources.well.object_type = WitsmlObjectType::Wellbore;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-003"));
}

#[test]
fn requires_distinct_plan_and_survey_source_identities() {
    let mut request = valid_request();
    request.sources.survey_trajectory.uuid = request.sources.plan_trajectory.uuid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-004"));
}

#[test]
fn rejects_duplicate_source_identity() {
    let mut request = valid_request();
    request.sources.wellbore.uuid = request.sources.well.uuid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-004"));
}

#[test]
fn rejects_analysis_uuid_reused_by_source() {
    let mut request = valid_request();
    request.analysis_id = request.sources.well.uuid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-005"));
}

#[test]
fn rejects_nil_analysis_uuid() {
    let mut request = valid_request();
    request.analysis_id = Uuid::nil();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-008"));
}

#[test]
fn rejects_nil_source_uuid() {
    let mut request = valid_request();
    request.sources.well.uuid = Uuid::nil();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-008"));
}

#[test]
fn rejects_malformed_absolute_source_uri() {
    let mut request = valid_request();
    request.sources.plan_trajectory.uri = Some("https://[broken".to_owned());
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-006"));
}

#[test]
fn rejects_unsupported_source_uri_scheme_after_deserialization() {
    let mut value = serde_json::to_value(valid_request()).unwrap();
    value["sources"]["plan_trajectory"]["uri"] = serde_json::json!("ftp://example.test/plan");
    let request = serde_json::from_value::<TrajectoryAnalysisRequest>(value).unwrap();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-006"));
}

#[test]
fn allows_distinct_plan_and_survey_sources_without_uris() {
    let mut request = valid_request();
    request.sources.plan_trajectory.uri = None;
    request.sources.survey_trajectory.uri = None;
    assert_eq!(validate_request(&request), Ok(()));
}

#[test]
fn rejects_identical_nonempty_plan_and_survey_uris() {
    let mut request = valid_request();
    request.sources.survey_trajectory.uri = request.sources.plan_trajectory.uri.clone();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-004"));
}

#[test]
fn rejects_duplicate_source_uri_across_object_types() {
    let mut request = valid_request();
    request.sources.wellbore.uri = request.sources.well.uri.clone();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-004"));
}

#[test]
fn rejects_normalization_equivalent_source_uris() {
    let mut request = valid_request();
    request.sources.plan_trajectory.uri = Some("HTTPS://EXAMPLE.TEST/trajectory/one".to_owned());
    request.sources.survey_trajectory.uri = Some("https://example.test/trajectory/one".to_owned());
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-004"));
}

#[test]
fn rejects_source_with_blank_citation_name() {
    let mut request = valid_request();
    request.sources.well.citation_name = " \t".to_owned();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-002"));
}

#[test]
fn rejects_source_with_blank_source_system() {
    let mut request = valid_request();
    request.sources.well.source_system.clear();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-002"));
}

#[test]
fn rejects_source_with_non_sha256_content_hash() {
    let mut request = valid_request();
    request.sources.well.content_hash = "sha1:0123".to_owned();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-002"));
}

#[test]
fn rejects_empty_plan_or_survey() {
    let mut request = valid_request();
    request.survey.clear();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-007"));
}

#[test]
fn rejects_duplicate_station_uuid_across_plan_and_survey() {
    let mut request = valid_request();
    request.survey[1].uid = request.plan[1].uid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-008"));
}

#[test]
fn rejects_uuid_reuse_by_target_and_slide() {
    let mut request = valid_request();
    request.slides[0].uid = request.targets[0].uid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-008"));
}

#[test]
fn rejects_uuid_reuse_by_target_and_formation() {
    let mut request = valid_request();
    request.formations[0].uid = request.targets[0].uid;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-008"));
}

#[test]
fn rejects_negative_measured_depth() {
    let mut request = valid_request();
    request.plan[0].md_m = -1.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-009"));
}

#[test]
fn requires_strictly_increasing_measured_depth() {
    let mut request = valid_request();
    request.plan[1].md_m = request.plan[0].md_m;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-010"));
}

#[test]
fn rejects_nonfinite_station_inclination() {
    let mut request = valid_request();
    request.survey[1].inclination_rad = f64::NAN;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-011"));
}

#[test]
fn rejects_out_of_range_station_azimuth() {
    let mut request = valid_request();
    request.plan[1].azimuth_rad = std::f64::consts::TAU;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-011"));
}

#[test]
fn requires_plan_and_survey_station_kinds() {
    let mut request = valid_request();
    request.plan[1].kind = StationKind::Survey;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-012"));
}

#[test]
fn rejects_tie_in_after_the_first_plan_station() {
    let mut request = valid_request();
    request.plan[1].kind = StationKind::TieIn;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-012"));
}

#[test]
fn requires_a_zero_md_tie_in_at_the_start_of_each_collection() {
    let mut request = valid_request();
    request.plan[0].md_m = 1.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-018"));
}

#[test]
fn requires_the_first_station_of_each_collection_to_be_a_tie_in() {
    let mut request = valid_request();
    request.survey[0].kind = StationKind::Survey;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-018"));
}

#[test]
fn requires_plan_and_survey_tie_in_inclinations_to_match() {
    let mut request = valid_request();
    request.survey[0].inclination_rad = 0.2;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-019"));
}

#[test]
fn requires_plan_and_survey_tie_in_azimuths_to_match() {
    let mut request = valid_request();
    request.survey[0].azimuth_rad = 0.3;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-019"));
}

#[test]
fn accepts_vertical_tie_ins_with_different_azimuths() {
    let mut request = valid_request();
    request.plan[0].inclination_rad = 0.0;
    request.plan[0].azimuth_rad = 0.0;
    request.survey[0].inclination_rad = 0.0;
    request.survey[0].azimuth_rad = 1.5;
    assert_eq!(validate_request(&request), Ok(()));
}

#[test]
fn accepts_tie_in_direction_difference_within_normalization_tolerance() {
    let mut request = valid_request();
    request.survey[0].inclination_rad += 5.0e-11;
    assert_eq!(validate_request(&request), Ok(()));
}

#[test]
fn rejects_genuinely_different_tie_in_direction() {
    let mut request = valid_request();
    request.survey[0].inclination_rad = 0.3;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-019"));
}

#[test]
fn rejects_nil_md_datum_identity() {
    let mut request = valid_request();
    request.md_datum.uid = Uuid::nil();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-013"));
}

#[test]
fn rejects_blank_md_datum_name() {
    let mut request = valid_request();
    request.md_datum.name.clear();
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-013"));
}

#[test]
fn rejects_nonfinite_vertical_section_azimuth() {
    let mut request = valid_request();
    request.vertical_section_azimuth_rad = f64::INFINITY;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-014"));
}

#[test]
fn rejects_nonfinite_target_geometry() {
    let mut request = valid_request();
    request.targets[0].north_m = f64::NAN;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-015"));
}

#[test]
fn rejects_nonpositive_target_horizontal_radius() {
    let mut request = valid_request();
    request.targets[0].minor_m = 0.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-015"));
}

#[test]
fn target_rotation_uses_the_canonical_half_open_range() {
    let mut request = valid_request();
    request.targets[0].rotation_rad = 0.0;
    assert_eq!(validate_request(&request), Ok(()));

    request.targets[0].rotation_rad = f64::from_bits(std::f64::consts::TAU.to_bits() - 1);
    assert_eq!(validate_request(&request), Ok(()));

    for invalid in [-f64::EPSILON, std::f64::consts::TAU] {
        request.targets[0].rotation_rad = invalid;
        assert!(
            has_code(&request, "WF-TRAJECTORY-CONTRACT-015"),
            "accepted noncanonical target rotation {invalid}"
        );
    }
}

#[test]
fn request_json_carries_target_depth_simplified_slide_and_projection() {
    let json = serde_json::to_value(valid_request()).unwrap();
    assert_eq!(json["targets"][0]["md_m"], 95.0);
    assert!(json["slides"][0].get("start_inclination_rad").is_none());
    assert_eq!(json["projection"]["bit_md_m"], 110.0);
    assert_eq!(
        serde_json::from_value::<TrajectoryAnalysisRequest>(json).unwrap(),
        valid_request()
    );
}

#[test]
fn rejects_unknown_projection_fields() {
    let mut value = serde_json::to_value(valid_request()).unwrap();
    value["projection"]["mystery"] = serde_json::json!(true);
    assert!(serde_json::from_value::<TrajectoryAnalysisRequest>(value).is_err());
}

#[test]
fn rejects_nonfinite_or_negative_target_depth() {
    let mut request = valid_request();
    request.targets[0].md_m = f64::NAN;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-015"));
    request.targets[0].md_m = -1.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-015"));
}

#[test]
fn rejects_projection_bit_depth_behind_survey_td() {
    let mut request = valid_request();
    request.projection.as_mut().unwrap().bit_md_m = 99.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-020"));
}

#[test]
fn rejects_nonfinite_projection_depth_and_negative_ahead() {
    let mut request = valid_request();
    request.projection.as_mut().unwrap().bit_md_m = f64::INFINITY;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-020"));
    request.projection.as_mut().unwrap().bit_md_m = 110.0;
    request.projection.as_mut().unwrap().ahead_m = -1.0;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-020"));
}

#[test]
fn rejects_noncanonical_projection_low_inclination_threshold() {
    let mut request = valid_request();
    request
        .projection
        .as_mut()
        .unwrap()
        .low_inclination_threshold_rad = std::f64::consts::PI + 0.1;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-020"));
}

#[test]
fn rejects_nonfinite_slide_geometry() {
    let mut request = valid_request();
    request.slides[0].slide_length_m = f64::INFINITY;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-016"));
}

#[test]
fn rejects_zero_length_slide_course() {
    let mut request = valid_request();
    request.slides[0].md_out_m = request.slides[0].md_in_m;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-016"));
}

#[test]
fn rejects_nonfinite_formation_geometry() {
    let mut request = valid_request();
    request.formations[0].prognosed_tvd_m = f64::NEG_INFINITY;
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-017"));
}

#[test]
fn rejects_negative_formation_actual_md() {
    let mut request = valid_request();
    request.formations[0].actual_md_m = Some(-1.0);
    assert!(has_code(&request, "WF-TRAJECTORY-CONTRACT-017"));
}

fn calculated(seed: u128, md_m: f64) -> CalculatedStation {
    CalculatedStation {
        source_uid: Some(Uuid::from_u128(seed)),
        kind: StationKind::Survey,
        lower_source_uid: None,
        upper_source_uid: None,
        md_m,
        inclination_rad: 0.1,
        azimuth_rad: 0.2,
        north_m: 1.0,
        east_m: 2.0,
        tvd_m: 3.0,
        delta_md_m: md_m,
        dogleg_rad: 0.01,
        ratio_factor: 1.0,
        dls_rad_per_m: 0.001,
    }
}

fn assert_strict_round_trip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).unwrap();
    assert!(!json.contains("NaN"));
    assert!(!json.contains("Infinity"));
    let parsed = serde_json::from_str::<T>(&json).unwrap();
    assert_eq!(&parsed, value);

    let mut unknown = serde_json::to_value(value).unwrap();
    unknown["mystery"] = serde_json::json!(true);
    assert!(serde_json::from_value::<T>(unknown).is_err());
}

fn aggregate_result_fixture() -> TrajectoryAnalysisResult {
    let interpolation = InterpolationResult {
        md_m: Some(10.0),
        status: InterpolationStatus::Ok,
        station: Some(calculated(21, 10.0)),
    };
    let residual = PlanSurveyResidual {
        survey_uid: Uuid::from_u128(21),
        md_m: 10.0,
        plan: interpolation.clone(),
        residual: Some(PositionResidual {
            north_m: 1.0,
            east_m: 2.0,
            tvd_m: 3.0,
            along_track_m: 1.0,
            crossline_m: 2.0,
            horizontal_m: 2.0,
            error_3d_m: 4.0,
        }),
    };
    let target = TargetAssessment {
        target_uid: Uuid::from_u128(30),
        md_m: 10.0,
        basis: TargetBasis::Actual,
        position: Some(SpatialPosition {
            north_m: 1.0,
            east_m: 2.0,
            tvd_m: 3.0,
        }),
        evaluation: None::<TargetEvaluation>,
    };
    let slide = SlideAssessment {
        slide_uid: Uuid::from_u128(40),
        start: interpolation.clone(),
        end: interpolation.clone(),
        response: None,
    };
    let projection = ProjectionAssessment {
        bit: calculated(21, 110.0),
        projected: calculated(21, 150.0),
        low_inclination_turn_guard: true,
    };
    let calculation = TrajectoryCalculation {
        plan: vec![calculated(11, 10.0)],
        survey: vec![calculated(21, 10.0)],
        plan_survey_residuals: vec![residual.clone()],
        targets: vec![target.clone()],
        slides: vec![slide.clone()],
        formations: Vec::<FormationEvaluation>::new(),
        projection: Some(projection.clone()),
    };
    TrajectoryAnalysisResult {
        contract_version: "1.0.0".to_owned(),
        analysis_id: Uuid::from_u128(100),
        sources: valid_request().sources,
        status: TrajectoryAnalysisStatus::CompleteWithWarnings,
        applicability: ApplicabilityStatement {
            method: "minimum_curvature_closed_form".to_owned(),
            deterministic: true,
            limitations: vec!["projection used".to_owned()],
        },
        evidence: CalculationEvidence {
            engine_version: "0.1.0".to_owned(),
            compiler_version: "test compiler identity".to_owned(),
            target_triple: "x86_64-pc-windows-msvc".to_owned(),
            lockfile_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            request_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_owned(),
            result_hash: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .to_owned(),
        },
        calculation,
    }
}

#[test]
fn aggregate_result_records_are_strict_finite_json() {
    let result = aggregate_result_fixture();

    assert_strict_round_trip(&result.calculation.plan_survey_residuals[0]);
    assert_strict_round_trip(&result.calculation.targets[0]);
    assert_strict_round_trip(&result.calculation.slides[0]);
    assert_strict_round_trip(result.calculation.projection.as_ref().unwrap());
    assert_strict_round_trip(&result.calculation);
    assert_strict_round_trip(&result.applicability);
    assert_strict_round_trip(&result.evidence);
    assert_strict_round_trip(&result);
}

#[test]
fn top_level_result_json_rejects_nonfinite_required_component() {
    let mut result = aggregate_result_fixture();
    result.calculation.plan[0].north_m = f64::NAN;

    assert!(serde_json::to_string(&result).is_err());
    assert!(serde_json::to_value(&result).is_err());
}
