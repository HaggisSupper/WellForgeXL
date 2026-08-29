//! Deterministic trajectory request fixtures shared by executable-boundary tests.

use uuid::Uuid;
use wellforge_trajectory_contract::{
    AzimuthReference, FormationPick, MdDatum, MdDatumKind, ProjectionRequest, SlideInterval,
    StationKind, Target, TargetKind, TrajectoryAnalysisRequest, TrajectorySourceSet,
    TrajectoryStation,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

fn source(object_type: WitsmlObjectType, seed: u128) -> SourceObjectRef {
    SourceObjectRef {
        uuid: Uuid::from_u128(seed),
        uri: Some(format!("eml:///wellforge/trajectory/{seed}")),
        object_type,
        content_hash: format!("sha256:{seed:064x}"),
        citation_name: format!("trajectory-fixture-{seed}"),
        source_system: "wellforge-trajectory-fixtures/1.0".to_owned(),
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

fn target(seed: u128, name: &str, md_m: f64, north_m: f64, tvd_m: f64) -> Target {
    Target {
        uid: Uuid::from_u128(seed),
        name: name.to_owned(),
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

/// Returns the fixed Release 1 trajectory request used by CLI acceptance tests.
#[must_use]
pub fn release_one_minimal_request() -> TrajectoryAnalysisRequest {
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
            target(30, "Actual target", 60.0, 2.0, 60.0),
            target(31, "Projected target", 120.0, 8.0, 120.0),
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
