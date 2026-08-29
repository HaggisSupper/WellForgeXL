//! Independent-oracle trajectory-core acceptance tests.

use uuid::Uuid;
use wellforge_trajectory_contract::{
    CalculatedStation, FormationCoverage, FormationPick, FormationSense, InterpolationResult,
    InterpolationStatus, ProjectionRequest, SlideStatus, SpatialPosition, StationKind, Target,
    TargetKind, TargetStatus, TrajectoryStation,
};
use wellforge_trajectory_core::{
    ResolvedSlideInterval, TrajectoryError, evaluate_formation, evaluate_target,
    interpolate_minimum_curvature, minimum_curvature, position_residual, project_tendency,
    slide_response,
};

const EPS: f64 = 1.0e-12;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected:.16e}, got {actual:.16e}, tolerance {tolerance:.3e}"
    );
}

fn station(seed: u128, md_m: f64, inclination_rad: f64, azimuth_rad: f64) -> TrajectoryStation {
    TrajectoryStation {
        uid: Uuid::from_u128(seed),
        kind: StationKind::Plan,
        md_m,
        inclination_rad,
        azimuth_rad,
    }
}

fn right_angle_course() -> Vec<TrajectoryStation> {
    vec![
        station(1, 0.0, 0.0, 0.0),
        station(2, 100.0, std::f64::consts::FRAC_PI_2, 0.0),
        station(
            3,
            200.0,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2,
        ),
    ]
}

#[test]
fn minimum_curvature_matches_right_angle_build_and_turn() {
    let calculated = minimum_curvature(&right_angle_course()).unwrap();
    let build = &calculated[1];
    assert_close(build.north_m, 63.661_977_236_758_126, 1.0e-12);
    assert_close(build.east_m, 0.0, EPS);
    assert_close(build.tvd_m, 63.661_977_236_758_126, 1.0e-12);
    assert_close(build.dogleg_rad, std::f64::consts::FRAC_PI_2, EPS);
    assert_close(build.ratio_factor, 1.273_239_544_735_162_5, EPS);
    assert_close(build.dls_rad_per_m, 0.015_707_963_267_948_967, EPS);

    let turn = &calculated[2];
    assert_close(turn.north_m, 127.323_954_473_516_27, 1.0e-12);
    assert_close(turn.east_m, 63.661_977_236_758_126, 1.0e-12);
    assert_close(turn.tvd_m, 63.661_977_236_758_126, 1.0e-12);
}

#[test]
fn partial_course_interpolation_matches_spherical_midpoints() {
    let calculated = minimum_curvature(&right_angle_course()).unwrap();

    let first = interpolate_minimum_curvature(&calculated, 50.0);
    assert_eq!(first.status, InterpolationStatus::Ok);
    assert_eq!(first.md_m, Some(50.0));
    let first = first.station.unwrap();
    assert_close(first.inclination_rad, std::f64::consts::FRAC_PI_4, EPS);
    assert_close(first.azimuth_rad, 0.0, EPS);
    assert_close(first.north_m, 18.646_161_428_902_833, 1.0e-12);
    assert_close(first.east_m, 0.0, EPS);
    assert_close(first.tvd_m, 45.015_815_807_855_304, 1.0e-12);

    let second = interpolate_minimum_curvature(&calculated, 150.0);
    assert_eq!(second.status, InterpolationStatus::Ok);
    let second = second.station.unwrap();
    assert_close(second.inclination_rad, std::f64::consts::FRAC_PI_2, EPS);
    assert_close(second.azimuth_rad, std::f64::consts::FRAC_PI_4, EPS);
    assert_close(second.north_m, 108.677_793_044_613_42, 1.0e-12);
    assert_close(second.east_m, 18.646_161_428_902_83, 1.0e-12);
    assert_close(second.tvd_m, 63.661_977_236_758_126, 1.0e-12);
}

#[test]
fn partial_course_handles_exact_limits_and_near_zero_dogleg() {
    let calculated = minimum_curvature(&right_angle_course()).unwrap();
    assert_eq!(
        interpolate_minimum_curvature(&calculated, -1.0).status,
        InterpolationStatus::BeforeStart
    );
    assert_eq!(
        interpolate_minimum_curvature(&calculated, 201.0).status,
        InterpolationStatus::BeyondTd
    );
    let exact = interpolate_minimum_curvature(&calculated, 100.0);
    assert_eq!(exact.status, InterpolationStatus::Ok);
    assert_eq!(exact.station.unwrap(), calculated[1]);

    let nearly_straight = minimum_curvature(&[
        station(10, 0.0, 0.3, 0.2),
        station(11, 100.0, 0.300_000_000_000_1, 0.200_000_000_000_1),
    ])
    .unwrap();
    let midpoint = interpolate_minimum_curvature(&nearly_straight, 50.0);
    assert_eq!(midpoint.status, InterpolationStatus::Ok);
    assert!(midpoint.station.unwrap().north_m.is_finite());
}

#[test]
fn empty_and_nonincreasing_courses_report_typed_states() {
    assert_eq!(
        interpolate_minimum_curvature(&[], 10.0).status,
        InterpolationStatus::NoStations
    );
    let error =
        minimum_curvature(&[station(1, 10.0, 0.0, 0.0), station(2, 10.0, 0.1, 0.2)]).unwrap_err();
    assert_eq!(error, TrajectoryError::NonIncreasingMeasuredDepth);
}

#[test]
fn position_residual_uses_actual_minus_plan_signs() {
    let actual = SpatialPosition {
        north_m: 12.0,
        east_m: 8.0,
        tvd_m: 110.0,
    };
    let planned = SpatialPosition {
        north_m: 10.0,
        east_m: 5.0,
        tvd_m: 100.0,
    };
    let residual = position_residual(&actual, &planned, std::f64::consts::FRAC_PI_2).unwrap();
    assert_close(residual.north_m, 2.0, EPS);
    assert_close(residual.east_m, 3.0, EPS);
    assert_close(residual.tvd_m, 10.0, EPS);
    assert_close(residual.along_track_m, 3.0, EPS);
    assert_close(residual.crossline_m, -2.0, EPS);
    assert_close(residual.horizontal_m, 3.605_551_275_463_989_6, EPS);
    assert_close(residual.error_3d_m, 10.630_145_812_734_65, EPS);
}

fn target(kind: TargetKind) -> Target {
    Target {
        uid: Uuid::from_u128(30),
        name: "Target".to_owned(),
        kind,
        md_m: 100.0,
        north_m: 100.0,
        east_m: 200.0,
        tvd_m: 300.0,
        major_m: 5.0,
        minor_m: 5.0,
        rotation_rad: 0.0,
        vertical_tolerance_m: 2.0,
    }
}

#[test]
fn circular_target_includes_horizontal_and_vertical_boundaries() {
    let result = evaluate_target(
        &target(TargetKind::Point),
        &SpatialPosition {
            north_m: 103.0,
            east_m: 204.0,
            tvd_m: 301.0,
        },
    );
    assert_eq!(result.status, TargetStatus::Hit);
    assert_close(result.horizontal_utilization.unwrap(), 1.0, EPS);
    assert_close(result.vertical_utilization.unwrap(), 0.5, EPS);
}

#[test]
fn rotated_ellipse_and_box_preserve_major_minor_sign_convention() {
    let mut ellipse = target(TargetKind::Ellipse);
    ellipse.major_m = 10.0;
    ellipse.minor_m = 2.0;
    ellipse.rotation_rad = std::f64::consts::FRAC_PI_4;
    ellipse.vertical_tolerance_m = 3.0;
    let major = evaluate_target(
        &ellipse,
        &SpatialPosition {
            north_m: 100.0 + 6.0 * ellipse.rotation_rad.cos(),
            east_m: 200.0 + 6.0 * ellipse.rotation_rad.sin(),
            tvd_m: 301.0,
        },
    );
    assert_eq!(major.status, TargetStatus::Hit);
    assert_close(major.horizontal_utilization.unwrap(), 0.6, EPS);
    let minor = evaluate_target(
        &ellipse,
        &SpatialPosition {
            north_m: 100.0 - 3.0 * ellipse.rotation_rad.sin(),
            east_m: 200.0 + 3.0 * ellipse.rotation_rad.cos(),
            tvd_m: 300.0,
        },
    );
    assert_eq!(minor.status, TargetStatus::Miss);
    assert_close(minor.horizontal_utilization.unwrap(), 1.5, EPS);

    let mut box_target = target(TargetKind::Box);
    box_target.major_m = 8.0;
    box_target.minor_m = 3.0;
    box_target.rotation_rad = std::f64::consts::PI / 6.0;
    let local_major = 5.0;
    let local_minor = 2.0;
    let inside = evaluate_target(
        &box_target,
        &SpatialPosition {
            north_m: 100.0 + local_major * box_target.rotation_rad.cos()
                - local_minor * box_target.rotation_rad.sin(),
            east_m: 200.0
                + local_major * box_target.rotation_rad.sin()
                + local_minor * box_target.rotation_rad.cos(),
            tvd_m: 301.0,
        },
    );
    assert_eq!(inside.status, TargetStatus::Hit);
    assert_close(inside.horizontal_utilization.unwrap(), 2.0 / 3.0, EPS);
}

fn slide() -> ResolvedSlideInterval {
    ResolvedSlideInterval {
        uid: Uuid::from_u128(40),
        md_in_m: 0.0,
        md_out_m: 100.0,
        slide_length_m: 50.0,
        start_inclination_rad: 0.2,
        end_inclination_rad: 0.3,
        start_azimuth_rad: 0.1,
        end_azimuth_rad: 0.3,
        commanded_toolface_rad: 0.4,
        rotary_build_rad_per_m: 0.0002,
        rotary_effective_turn_rad_per_m: 0.0001,
        low_inclination_threshold_rad: 0.0,
    }
}

fn vertical_survey() -> Vec<CalculatedStation> {
    minimum_curvature(&[station(60, 0.0, 0.0, 0.0), station(61, 100.0, 0.0, 0.0)]).unwrap()
}

fn projection_request() -> ProjectionRequest {
    ProjectionRequest {
        bit_md_m: 110.0,
        ahead_m: 40.0,
        build_tendency_rad_per_m: 0.0,
        effective_turn_tendency_rad_per_m: 0.0,
        low_inclination_threshold_rad: 0.01,
    }
}

#[test]
fn straight_vertical_projection_extends_to_bit_and_ahead() {
    let projection = project_tendency(&vertical_survey(), &projection_request()).unwrap();
    assert_close(projection.bit.md_m, 110.0, EPS);
    assert_close(projection.bit.tvd_m, 110.0, EPS);
    assert_close(projection.projected.md_m, 150.0, EPS);
    assert_close(projection.projected.tvd_m, 150.0, EPS);
    assert_close(projection.projected.north_m, 0.0, EPS);
    assert_close(projection.projected.east_m, 0.0, EPS);
    assert!(!projection.low_inclination_turn_guard);
}

#[test]
fn pure_build_projection_matches_workbook_minimum_curvature_literal() {
    let mut request = projection_request();
    request.bit_md_m = 100.0;
    request.ahead_m = 100.0;
    request.build_tendency_rad_per_m = 0.001;
    let projection = project_tendency(&vertical_survey(), &request).unwrap();
    assert_close(projection.projected.inclination_rad, 0.1, EPS);
    assert_close(projection.projected.north_m, 4.995_834_721_974_234, 1.0e-12);
    assert_close(projection.projected.east_m, 0.0, EPS);
    assert_close(projection.projected.tvd_m, 199.833_416_646_828_14, 1.0e-12);
}

#[test]
fn effective_turn_projection_marks_low_inclination_guard() {
    let survey =
        minimum_curvature(&[station(60, 0.0, 0.001, 0.0), station(61, 100.0, 0.001, 0.0)]).unwrap();
    let mut request = projection_request();
    request.bit_md_m = 100.0;
    request.ahead_m = 10.0;
    request.effective_turn_tendency_rad_per_m = 0.001;
    let projection = project_tendency(&survey, &request).unwrap();
    assert!(projection.low_inclination_turn_guard);
    assert!(projection.projected.azimuth_rad.is_finite());
}

#[test]
fn projection_rejects_bit_behind_survey_and_numerical_overflow() {
    let mut request = projection_request();
    request.bit_md_m = 99.0;
    assert_eq!(
        project_tendency(&vertical_survey(), &request),
        Err(TrajectoryError::ProjectionBehindSurvey)
    );
    request.bit_md_m = f64::MAX;
    request.ahead_m = f64::MAX;
    assert_eq!(
        project_tendency(&vertical_survey(), &request),
        Err(TrajectoryError::NumericalOverflow)
    );
}

#[test]
fn resolved_slide_response_uses_survey_interpolated_endpoint_directions() {
    let survey = minimum_curvature(&right_angle_course()).unwrap();
    let start = interpolate_minimum_curvature(&survey, 50.0)
        .station
        .unwrap();
    let end = interpolate_minimum_curvature(&survey, 150.0)
        .station
        .unwrap();
    let resolved = ResolvedSlideInterval {
        uid: Uuid::from_u128(62),
        md_in_m: 50.0,
        md_out_m: 150.0,
        slide_length_m: 100.0,
        start_inclination_rad: start.inclination_rad,
        end_inclination_rad: end.inclination_rad,
        start_azimuth_rad: start.azimuth_rad,
        end_azimuth_rad: end.azimuth_rad,
        commanded_toolface_rad: 0.0,
        rotary_build_rad_per_m: 0.0,
        rotary_effective_turn_rad_per_m: 0.0,
        low_inclination_threshold_rad: 0.0,
    };
    let response = slide_response(&resolved);
    assert_eq!(response.status, SlideStatus::Ok);
    assert_close(
        response.build_rad_per_m.unwrap(),
        std::f64::consts::FRAC_PI_4 / 100.0,
        EPS,
    );
}

#[test]
fn slide_response_removes_rotary_baseline_and_reports_toolface_error() {
    let result = slide_response(&slide());
    assert_eq!(result.status, SlideStatus::Ok);
    assert_close(result.build_rad_per_m.unwrap(), 0.001, EPS);
    assert_close(
        result.effective_turn_rad_per_m.unwrap(),
        0.000_494_807_918_509_046_4,
        EPS,
    );
    assert_close(result.residual_build_rad_per_m.unwrap(), 0.0016, EPS);
    assert_close(
        result.residual_turn_rad_per_m.unwrap(),
        0.000_789_615_837_018_092_8,
        EPS,
    );
    assert_close(
        result.yield_rad_per_m.unwrap(),
        0.001_784_234_617_439_585_1,
        EPS,
    );
    assert_close(
        result.response_toolface_rad.unwrap(),
        0.458_442_060_592_046_73,
        EPS,
    );
    assert_close(
        result.toolface_error_rad.unwrap(),
        0.058_442_060_592_046_374,
        EPS,
    );
}

#[test]
fn slide_response_has_typed_invalid_and_low_inclination_states() {
    let mut interval = slide();
    interval.md_out_m = interval.md_in_m;
    assert_eq!(
        slide_response(&interval).status,
        SlideStatus::InvalidSlideLength
    );
    interval = slide();
    interval.low_inclination_threshold_rad = 0.3;
    assert_eq!(
        slide_response(&interval).status,
        SlideStatus::LowInclination
    );
}

#[test]
fn formation_high_low_uses_prognosis_minus_actual_tvd() {
    let survey =
        minimum_curvature(&[station(1, 0.0, 0.0, 0.0), station(2, 100.0, 0.0, 0.0)]).unwrap();
    let pick = FormationPick {
        uid: Uuid::from_u128(50),
        name: "Formation".to_owned(),
        prognosed_tvd_m: 55.5,
        actual_md_m: Some(50.0),
        tolerance_m: Some(3.0),
    };
    let result = evaluate_formation(&pick, &survey);
    assert_eq!(result.coverage, FormationCoverage::Ok);
    assert_eq!(result.sense, Some(FormationSense::High));
    assert_close(result.high_low_m.unwrap(), 5.5, EPS);
    assert_eq!(result.within_tolerance, Some(false));

    let mut on = pick.clone();
    on.prognosed_tvd_m = 50.0;
    let on = evaluate_formation(&on, &survey);
    assert_eq!(on.sense, Some(FormationSense::OnPrognosis));
}

#[test]
fn formation_evaluation_propagates_missing_and_outside_course_states() {
    let survey = minimum_curvature(&right_angle_course()).unwrap();
    let mut pick = FormationPick {
        uid: Uuid::from_u128(50),
        name: "Formation".to_owned(),
        prognosed_tvd_m: 100.0,
        actual_md_m: None,
        tolerance_m: None,
    };
    assert_eq!(
        evaluate_formation(&pick, &survey).coverage,
        FormationCoverage::NoActualPick
    );
    pick.actual_md_m = Some(-1.0);
    assert_eq!(
        evaluate_formation(&pick, &survey).coverage,
        FormationCoverage::BeforeStart
    );
    pick.actual_md_m = Some(201.0);
    assert_eq!(
        evaluate_formation(&pick, &survey).coverage,
        FormationCoverage::BeyondTd
    );
}

#[test]
fn interpolation_rejects_nonfinite_and_malformed_courses_with_typed_states() {
    let calculated = minimum_curvature(&right_angle_course()).unwrap();
    for query_md_m in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let result = interpolate_minimum_curvature(&calculated, query_md_m);
        assert_eq!(result.status, InterpolationStatus::InvalidMeasuredDepth);
        assert_eq!(result.md_m, None);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("NaN"));
        assert!(!json.contains("Infinity"));
        assert_eq!(
            serde_json::from_str::<InterpolationResult>(&json).unwrap(),
            result,
            "non-finite query {query_md_m:?} must round-trip as an absent MD"
        );
    }

    let mut unordered = calculated.clone();
    unordered[1].md_m = unordered[0].md_m;
    assert_eq!(
        interpolate_minimum_curvature(&unordered, 50.0).status,
        InterpolationStatus::InvalidCourse
    );
    let mut nonfinite = calculated;
    nonfinite[1].north_m = f64::INFINITY;
    assert_eq!(
        interpolate_minimum_curvature(&nonfinite, 50.0).status,
        InterpolationStatus::InvalidCourse
    );
}

#[test]
fn interpolation_preserves_exact_source_identity_and_marks_synthetic_station() {
    let calculated = minimum_curvature(&right_angle_course()).unwrap();
    let exact = interpolate_minimum_curvature(&calculated, 100.0)
        .station
        .unwrap();
    assert_eq!(exact.source_uid, Some(Uuid::from_u128(2)));
    assert_eq!(exact.kind, StationKind::Plan);
    assert_eq!(exact.lower_source_uid, None);
    assert_eq!(exact.upper_source_uid, None);

    let synthetic = interpolate_minimum_curvature(&calculated, 50.0)
        .station
        .unwrap();
    assert_eq!(synthetic.source_uid, None);
    assert_eq!(synthetic.kind, StationKind::Interpolated);
    assert_eq!(synthetic.lower_source_uid, Some(Uuid::from_u128(1)));
    assert_eq!(synthetic.upper_source_uid, Some(Uuid::from_u128(2)));
}

#[test]
fn zero_vertical_tolerance_miss_uses_empty_serializable_field() {
    let mut target = target(TargetKind::Point);
    target.vertical_tolerance_m = 0.0;
    let result = evaluate_target(
        &target,
        &SpatialPosition {
            north_m: target.north_m,
            east_m: target.east_m,
            tvd_m: target.tvd_m + 1.0,
        },
    );
    assert_eq!(result.status, TargetStatus::Miss);
    assert_eq!(result.vertical_utilization, None);
}

#[test]
fn zero_vertical_tolerance_exact_match_has_zero_utilization() {
    let mut target = target(TargetKind::Point);
    target.vertical_tolerance_m = 0.0;
    let result = evaluate_target(
        &target,
        &SpatialPosition {
            north_m: target.north_m,
            east_m: target.east_m,
            tvd_m: target.tvd_m,
        },
    );
    assert_eq!(result.status, TargetStatus::Hit);
    assert_eq!(result.vertical_utilization, Some(0.0));
}

#[test]
fn formation_invalid_geometry_has_no_calculated_values() {
    let survey = minimum_curvature(&right_angle_course()).unwrap();
    let mut formation = FormationPick {
        uid: Uuid::from_u128(80),
        name: "Invalid geometry".to_owned(),
        prognosed_tvd_m: f64::NAN,
        actual_md_m: Some(50.0),
        tolerance_m: Some(1.0),
    };
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        formation.prognosed_tvd_m = invalid;
        let result = evaluate_formation(&formation, &survey);
        assert_eq!(result.coverage, FormationCoverage::InvalidGeometry);
        assert_eq!(result.actual_tvd_m, None);
        assert_eq!(result.high_low_m, None);
        assert_eq!(result.sense, None);
        assert_eq!(result.within_tolerance, None);
    }
    formation.prognosed_tvd_m = 100.0;
    formation.tolerance_m = Some(-1.0);
    assert_eq!(
        evaluate_formation(&formation, &survey).coverage,
        FormationCoverage::InvalidGeometry
    );
}

#[test]
fn finite_extremes_report_overflow_states_without_nonfinite_results() {
    let extreme_position = SpatialPosition {
        north_m: f64::MAX,
        east_m: 0.0,
        tvd_m: 0.0,
    };
    let opposite_position = SpatialPosition {
        north_m: -f64::MAX,
        east_m: 0.0,
        tvd_m: 0.0,
    };
    assert_eq!(
        position_residual(&extreme_position, &opposite_position, 0.0).unwrap_err(),
        TrajectoryError::NumericalOverflow
    );

    let mut extreme_target = target(TargetKind::Point);
    extreme_target.north_m = -f64::MAX;
    assert_eq!(
        evaluate_target(&extreme_target, &extreme_position).status,
        TargetStatus::NumericalOverflow
    );

    let mut interval = slide();
    interval.md_out_m = f64::MAX;
    interval.slide_length_m = 1.0;
    interval.rotary_build_rad_per_m = -f64::MAX;
    assert_eq!(
        slide_response(&interval).status,
        SlideStatus::NumericalOverflow
    );

    let calculated = minimum_curvature(&right_angle_course()).unwrap();
    let mut extreme_course = calculated.clone();
    extreme_course.truncate(2);
    extreme_course[0].north_m = f64::MAX;
    extreme_course[1].md_m = f64::MAX;
    assert_eq!(
        interpolate_minimum_curvature(&extreme_course, f64::MAX / 2.0).status,
        InterpolationStatus::NumericalOverflow
    );

    let mut formation_course = calculated;
    formation_course[0].tvd_m = -f64::MAX;
    let formation = FormationPick {
        uid: Uuid::from_u128(81),
        name: "Overflow".to_owned(),
        prognosed_tvd_m: f64::MAX,
        actual_md_m: Some(0.0),
        tolerance_m: None,
    };
    assert_eq!(
        evaluate_formation(&formation, &formation_course).coverage,
        FormationCoverage::NumericalOverflow
    );
}

#[test]
fn core_rejects_nonfinite_invalid_and_antipodal_geometry() {
    let mut bad_station = station(60, 0.0, 0.0, 0.0);
    bad_station.inclination_rad = f64::NAN;
    assert_eq!(
        minimum_curvature(&[bad_station]).unwrap_err(),
        TrajectoryError::NonFiniteStationGeometry
    );
    assert_eq!(
        minimum_curvature(&[station(60, -1.0, 0.0, 0.0)]).unwrap_err(),
        TrajectoryError::InvalidStationGeometry
    );
    assert_eq!(
        minimum_curvature(&[
            station(61, 0.0, 0.0, 0.0),
            station(62, 100.0, std::f64::consts::PI, 0.0)
        ])
        .unwrap_err(),
        TrajectoryError::AmbiguousDogleg
    );

    let mut target = target(TargetKind::Circle);
    target.rotation_rad = f64::NAN;
    assert_eq!(
        evaluate_target(
            &target,
            &SpatialPosition {
                north_m: 0.0,
                east_m: 0.0,
                tvd_m: 0.0
            }
        )
        .status,
        TargetStatus::InvalidGeometry
    );
    assert_eq!(
        position_residual(
            &SpatialPosition {
                north_m: f64::INFINITY,
                east_m: 0.0,
                tvd_m: 0.0
            },
            &SpatialPosition {
                north_m: 0.0,
                east_m: 0.0,
                tvd_m: 0.0
            },
            0.0,
        )
        .unwrap_err(),
        TrajectoryError::NonFinitePosition
    );
}

#[test]
fn invalid_slide_and_ambiguous_manual_interpolation_are_typed() {
    let mut interval = slide();
    interval.start_azimuth_rad = f64::NAN;
    assert_eq!(
        slide_response(&interval).status,
        SlideStatus::InvalidGeometry
    );

    let ambiguous = vec![
        CalculatedStation {
            source_uid: Some(Uuid::from_u128(70)),
            kind: StationKind::Plan,
            lower_source_uid: None,
            upper_source_uid: None,
            md_m: 0.0,
            inclination_rad: 0.0,
            azimuth_rad: 0.0,
            north_m: 0.0,
            east_m: 0.0,
            tvd_m: 0.0,
            delta_md_m: 0.0,
            dogleg_rad: 0.0,
            ratio_factor: 1.0,
            dls_rad_per_m: 0.0,
        },
        CalculatedStation {
            source_uid: Some(Uuid::from_u128(71)),
            kind: StationKind::Plan,
            lower_source_uid: None,
            upper_source_uid: None,
            md_m: 100.0,
            inclination_rad: std::f64::consts::PI - 1.0e-10,
            azimuth_rad: 0.0,
            north_m: 0.0,
            east_m: 0.0,
            tvd_m: 0.0,
            delta_md_m: 100.0,
            dogleg_rad: std::f64::consts::PI,
            ratio_factor: 1.0,
            dls_rad_per_m: 0.0,
        },
    ];
    assert_eq!(
        interpolate_minimum_curvature(&ambiguous, 50.0).status,
        InterpolationStatus::InvalidCourse
    );

    let survey = minimum_curvature(&right_angle_course()).unwrap();
    let formation = FormationPick {
        uid: Uuid::from_u128(72),
        name: "Invalid".to_owned(),
        prognosed_tvd_m: 0.0,
        actual_md_m: Some(f64::INFINITY),
        tolerance_m: None,
    };
    assert_eq!(
        evaluate_formation(&formation, &survey).coverage,
        FormationCoverage::InvalidMeasuredDepth
    );
}
