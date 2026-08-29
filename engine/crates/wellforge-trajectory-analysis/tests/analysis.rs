//! Pure trajectory-analysis orchestration acceptance tests.

use uuid::Uuid;
use wellforge_trajectory_analysis::{TrajectoryAnalysisError, analyze};
use wellforge_trajectory_contract::{
    AzimuthReference, FormationPick, InterpolationStatus, MdDatum, MdDatumKind, ProjectionRequest,
    SlideInterval, StationKind, Target, TargetBasis, TargetKind, TrajectoryAnalysisRequest,
    TrajectorySourceSet, TrajectoryStation,
};
use wellforge_trajectory_core::TrajectoryError;
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

fn source(object_type: WitsmlObjectType, seed: u128) -> SourceObjectRef {
    SourceObjectRef {
        uuid: Uuid::from_u128(seed),
        uri: Some(format!("eml:///wellforge/analysis/{seed}")),
        object_type,
        content_hash: format!("sha256:{seed:064x}"),
        citation_name: format!("fixture-{seed}"),
        source_system: "analysis-test".to_owned(),
    }
}

fn station(
    seed: u128,
    kind: StationKind,
    md_m: f64,
    inclination_rad: f64,
    azimuth_rad: f64,
) -> TrajectoryStation {
    TrajectoryStation {
        uid: Uuid::from_u128(seed),
        kind,
        md_m,
        inclination_rad,
        azimuth_rad,
    }
}

fn target(seed: u128, md_m: f64, north_m: f64, tvd_m: f64) -> Target {
    Target {
        uid: Uuid::from_u128(seed),
        name: format!("Target {seed}"),
        kind: TargetKind::Circle,
        md_m,
        north_m,
        east_m: 0.0,
        tvd_m,
        major_m: 20.0,
        minor_m: 20.0,
        rotation_rad: 0.0,
        vertical_tolerance_m: 20.0,
    }
}

fn request() -> TrajectoryAnalysisRequest {
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
            name: "RKB".to_owned(),
            kind: MdDatumKind::RotaryKellyBushing,
        },
        azimuth_reference: AzimuthReference::TrueNorth,
        vertical_section_azimuth_rad: 0.0,
        plan: vec![
            station(10, StationKind::TieIn, 0.0, 0.0, 0.0),
            station(11, StationKind::Plan, 40.0, 0.04, 0.0),
            station(12, StationKind::Plan, 80.0, 0.08, 0.0),
        ],
        survey: vec![
            station(20, StationKind::TieIn, 0.0, 0.0, 0.0),
            station(21, StationKind::Survey, 50.0, 0.05, 0.0),
            station(22, StationKind::Survey, 100.0, 0.1, 0.0),
        ],
        targets: vec![
            target(30, 60.0, 2.0, 60.0),
            target(31, 120.0, 8.0, 120.0),
            target(32, 160.0, 20.0, 155.0),
        ],
        slides: vec![
            SlideInterval {
                uid: Uuid::from_u128(40),
                md_in_m: 20.0,
                md_out_m: 80.0,
                slide_length_m: 30.0,
                commanded_toolface_rad: 0.0,
                rotary_build_rad_per_m: 0.0,
                rotary_effective_turn_rad_per_m: 0.0,
                low_inclination_threshold_rad: 0.0,
            },
            SlideInterval {
                uid: Uuid::from_u128(41),
                md_in_m: 90.0,
                md_out_m: 120.0,
                slide_length_m: 20.0,
                commanded_toolface_rad: 0.0,
                rotary_build_rad_per_m: 0.0,
                rotary_effective_turn_rad_per_m: 0.0,
                low_inclination_threshold_rad: 0.0,
            },
        ],
        formations: vec![FormationPick {
            uid: Uuid::from_u128(50),
            name: "Formation A".to_owned(),
            prognosed_tvd_m: 94.0,
            actual_md_m: Some(95.0),
            tolerance_m: Some(5.0),
        }],
        projection: Some(ProjectionRequest {
            bit_md_m: 110.0,
            ahead_m: 40.0,
            build_tendency_rad_per_m: 0.001,
            effective_turn_tendency_rad_per_m: 0.0,
            low_inclination_threshold_rad: 0.01,
        }),
    }
}

#[test]
fn analyze_composes_ordered_course_residual_target_slide_and_formation_results() {
    let request = request();
    let result = analyze(&request).unwrap();
    assert_eq!(result.plan.len(), request.plan.len());
    assert_eq!(result.survey.len(), request.survey.len());
    assert_eq!(result.plan_survey_residuals.len(), request.survey.len());
    assert_eq!(result.targets[0].basis, TargetBasis::Actual);
    assert_eq!(result.targets[1].basis, TargetBasis::Projected);
    assert_eq!(result.targets[2].basis, TargetBasis::NotReached);
    assert!(result.slides[0].response.is_some());
    assert!(result.slides[1].response.is_none());
    assert_eq!(
        result.formations[0].formation_uid,
        request.formations[0].uid
    );

    assert_eq!(
        result.plan_survey_residuals[2].plan.status,
        InterpolationStatus::BeyondTd
    );
    assert!(result.plan_survey_residuals[2].residual.is_none());
    assert_eq!(result.slides[1].end.status, InterpolationStatus::BeyondTd);
    assert_eq!(
        result
            .plan
            .iter()
            .map(|item| item.source_uid)
            .collect::<Vec<_>>(),
        request
            .plan
            .iter()
            .map(|item| Some(item.uid))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        result
            .targets
            .iter()
            .map(|item| item.target_uid)
            .collect::<Vec<_>>(),
        request
            .targets
            .iter()
            .map(|item| item.uid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn projection_low_inclination_guard_propagates_to_analysis() {
    let mut request = request();
    for station in &mut request.plan {
        station.inclination_rad = 0.001;
    }
    for station in &mut request.survey {
        station.inclination_rad = 0.001;
    }
    request
        .projection
        .as_mut()
        .unwrap()
        .effective_turn_tendency_rad_per_m = 0.001;
    assert!(
        analyze(&request)
            .unwrap()
            .projection
            .unwrap()
            .low_inclination_turn_guard
    );
}

#[test]
fn analyze_rejects_ambiguous_and_overflowing_plan_courses() {
    let mut ambiguous = request();
    ambiguous.plan[1].inclination_rad = std::f64::consts::PI;
    ambiguous.plan[1].azimuth_rad = 0.0;
    assert!(matches!(
        analyze(&ambiguous),
        Err(TrajectoryAnalysisError::Plan(
            TrajectoryError::AmbiguousDogleg
        ))
    ));

    let mut overflow = request();
    overflow.projection = None;
    for station in [&mut overflow.plan[1], &mut overflow.survey[1]] {
        station.md_m = f64::MAX / 2.0;
        station.inclination_rad = std::f64::consts::FRAC_PI_2;
    }
    for station in [&mut overflow.plan[2], &mut overflow.survey[2]] {
        station.md_m = f64::MAX;
        station.inclination_rad = std::f64::consts::FRAC_PI_2;
    }
    overflow.survey[1].azimuth_rad = std::f64::consts::PI;
    overflow.survey[2].azimuth_rad = std::f64::consts::PI;
    assert_eq!(
        analyze(&overflow),
        Err(TrajectoryAnalysisError::Residual(
            TrajectoryError::NumericalOverflow
        ))
    );
}
