//! Deterministic trajectory request validation.

use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use wellforge_witsml::{IdentityError, SourceObjectRef, WitsmlObjectType};

use crate::{SlideInterval, StationKind, TrajectoryAnalysisRequest, TrajectoryStation};

/// Stable trajectory contract diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractError {
    /// Stable machine-readable code.
    pub code: String,
    /// Human-readable diagnostic.
    pub message: String,
}

fn push(errors: &mut Vec<ContractError>, code: &str, message: impl Into<String>) {
    errors.push(ContractError {
        code: code.to_owned(),
        message: message.into(),
    });
}

fn source_refs(request: &TrajectoryAnalysisRequest) -> [(&SourceObjectRef, WitsmlObjectType); 4] {
    [
        (&request.sources.well, WitsmlObjectType::Well),
        (&request.sources.wellbore, WitsmlObjectType::Wellbore),
        (
            &request.sources.plan_trajectory,
            WitsmlObjectType::Trajectory,
        ),
        (
            &request.sources.survey_trajectory,
            WitsmlObjectType::Trajectory,
        ),
    ]
}

fn normalized_source_uri(source: &SourceObjectRef) -> Option<String> {
    source
        .uri
        .as_deref()
        .and_then(|uri| Url::parse(uri).ok())
        .map(|uri| uri.to_string())
}

/// Release 1 accepts tie-in unit vectors within this chord-distance tolerance (approximately the
/// same angular separation in radians). This avoids rejecting harmless f64 normalization drift.
const TIE_IN_DIRECTION_TOLERANCE: f64 = 1.0e-10;

fn tie_in_direction(station: &TrajectoryStation) -> [f64; 3] {
    let sin_inclination = station.inclination_rad.sin();
    [
        sin_inclination * station.azimuth_rad.cos(),
        sin_inclination * station.azimuth_rad.sin(),
        station.inclination_rad.cos(),
    ]
}

fn tie_in_directions_match(
    plan_tie_in: &TrajectoryStation,
    survey_tie_in: &TrajectoryStation,
) -> bool {
    let plan_direction = tie_in_direction(plan_tie_in);
    let survey_direction = tie_in_direction(survey_tie_in);
    let squared_distance = plan_direction
        .iter()
        .zip(survey_direction)
        .map(|(plan, survey)| (plan - survey).powi(2))
        .sum::<f64>();
    squared_distance <= TIE_IN_DIRECTION_TOLERANCE.powi(2)
}

fn validate_station_set(
    stations: &[TrajectoryStation],
    plan: bool,
    errors: &mut Vec<ContractError>,
) {
    for (index, station) in stations.iter().enumerate() {
        if !station.md_m.is_finite() || station.md_m < 0.0 {
            push(
                errors,
                "WF-TRAJECTORY-CONTRACT-009",
                "station measured depth must be finite and nonnegative",
            );
        }
        if !station.inclination_rad.is_finite()
            || !(0.0..=std::f64::consts::PI).contains(&station.inclination_rad)
            || !station.azimuth_rad.is_finite()
            || !(0.0..std::f64::consts::TAU).contains(&station.azimuth_rad)
        {
            push(
                errors,
                "WF-TRAJECTORY-CONTRACT-011",
                "station angles must be finite and in canonical ranges",
            );
        }
        let kind_valid = if index == 0 {
            matches!(station.kind, StationKind::TieIn)
        } else if plan {
            matches!(station.kind, StationKind::Plan)
        } else {
            matches!(station.kind, StationKind::Survey)
        };
        if !kind_valid {
            push(
                errors,
                if index == 0 {
                    "WF-TRAJECTORY-CONTRACT-018"
                } else {
                    "WF-TRAJECTORY-CONTRACT-012"
                },
                if index == 0 {
                    "each plan and survey collection must begin with a tie-in station"
                } else {
                    "later station type does not match its plan or survey collection"
                },
            );
        }
    }
    if stations.first().is_some_and(|station| station.md_m != 0.0) {
        push(
            errors,
            "WF-TRAJECTORY-CONTRACT-018",
            "each plan and survey collection must begin with a zero-MD tie-in station",
        );
    }
    for pair in stations.windows(2) {
        if pair[1].md_m <= pair[0].md_m {
            push(
                errors,
                "WF-TRAJECTORY-CONTRACT-010",
                "trajectory measured depths must increase strictly",
            );
        }
    }
}

fn valid_slide(slide: &SlideInterval) -> bool {
    let values = [
        slide.md_in_m,
        slide.md_out_m,
        slide.slide_length_m,
        slide.commanded_toolface_rad,
        slide.rotary_build_rad_per_m,
        slide.rotary_effective_turn_rad_per_m,
        slide.low_inclination_threshold_rad,
    ];
    let course_length = slide.md_out_m - slide.md_in_m;
    values.into_iter().all(f64::is_finite)
        && slide.md_in_m >= 0.0
        && course_length > 0.0
        && slide.slide_length_m > 0.0
        && slide.slide_length_m <= course_length
        && (0.0..std::f64::consts::TAU).contains(&slide.commanded_toolface_rad)
        && (0.0..=std::f64::consts::PI).contains(&slide.low_inclination_threshold_rad)
}

/// Validates every trajectory request invariant before calculation.
///
/// # Errors
///
/// Returns all stable [`ContractError`] diagnostics when one or more invariants fail.
#[allow(clippy::too_many_lines)]
pub fn validate_request(request: &TrajectoryAnalysisRequest) -> Result<(), Vec<ContractError>> {
    let mut errors = Vec::new();
    if !semver::Version::parse(&request.contract_version).is_ok_and(|version| version.major == 1) {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-001",
            "unsupported contract major version",
        );
    }

    let sources = source_refs(request);
    for (source, expected) in sources {
        if source.object_type != expected {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-003",
                format!("source must be {expected:?}"),
            );
        }
        match source.validate() {
            Ok(()) | Err(IdentityError::NilUuid) => {}
            Err(IdentityError::InvalidUri(_)) => push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-006",
                "source URI must use the eml, http or https scheme and be syntactically valid",
            ),
            Err(_) => push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-002",
                "source provenance requires a SHA-256 content hash, citation name and source system",
            ),
        }
    }

    let source_ids = source_refs(request).map(|(source, _)| source.uuid);
    if source_ids.contains(&request.analysis_id) {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-005",
            "analysis UUID must not reuse a source identity",
        );
    }
    let mut source_uris = HashSet::new();
    let duplicate_source_uri = source_refs(request)
        .into_iter()
        .filter_map(|(source, _)| normalized_source_uri(source))
        .any(|uri| !source_uris.insert(uri));
    if source_ids.into_iter().collect::<HashSet<_>>().len() != source_ids.len()
        || duplicate_source_uri
    {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-004",
            "source identities must be unique and plan/survey must be distinct",
        );
    }

    if request.plan.is_empty() || request.survey.is_empty() {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-007",
            "plan and survey station collections are required",
        );
    }
    validate_station_set(&request.plan, true, &mut errors);
    validate_station_set(&request.survey, false, &mut errors);
    if let (Some(plan_tie_in), Some(survey_tie_in)) = (request.plan.first(), request.survey.first())
        && !tie_in_directions_match(plan_tie_in, survey_tie_in)
    {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-019",
            "plan and survey zero-MD tie-in directions must match within tolerance",
        );
    }

    let mut record_ids = HashSet::<Uuid>::new();
    for uid in std::iter::once(request.analysis_id)
        .chain(source_ids)
        .chain(std::iter::once(request.md_datum.uid))
        .chain(request.plan.iter().map(|item| item.uid))
        .chain(request.survey.iter().map(|item| item.uid))
        .chain(request.targets.iter().map(|item| item.uid))
        .chain(request.slides.iter().map(|item| item.uid))
        .chain(request.formations.iter().map(|item| item.uid))
    {
        if uid.is_nil() || !record_ids.insert(uid) {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-008",
                "all identity-bearing records require unique non-nil UUIDs",
            );
        }
    }

    if request.md_datum.uid.is_nil() || request.md_datum.name.trim().is_empty() {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-013",
            "measured-depth datum requires a non-nil identity and name",
        );
    }
    if !request.vertical_section_azimuth_rad.is_finite()
        || !(0.0..std::f64::consts::TAU).contains(&request.vertical_section_azimuth_rad)
    {
        push(
            &mut errors,
            "WF-TRAJECTORY-CONTRACT-014",
            "vertical-section azimuth must be finite and canonical",
        );
    }

    for target in &request.targets {
        let values = [
            target.md_m,
            target.north_m,
            target.east_m,
            target.tvd_m,
            target.major_m,
            target.minor_m,
            target.rotation_rad,
            target.vertical_tolerance_m,
        ];
        if !values.into_iter().all(f64::is_finite)
            || target.name.trim().is_empty()
            || target.md_m < 0.0
            || target.major_m <= 0.0
            || target.minor_m <= 0.0
            || !(0.0..std::f64::consts::TAU).contains(&target.rotation_rad)
            || target.vertical_tolerance_m < 0.0
        {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-015",
                "target geometry must be finite with positive horizontal radii and canonical rotation",
            );
        }
    }
    for slide in &request.slides {
        if !valid_slide(slide) {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-016",
                "slide geometry and response inputs are invalid",
            );
        }
    }
    for formation in &request.formations {
        if formation.name.trim().is_empty()
            || !formation.prognosed_tvd_m.is_finite()
            || formation.prognosed_tvd_m < 0.0
            || formation
                .actual_md_m
                .is_some_and(|value| !value.is_finite() || value < 0.0)
            || formation
                .tolerance_m
                .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-017",
                "formation geometry must be finite and nonnegative",
            );
        }
    }

    if let Some(projection) = &request.projection {
        let survey_td_m = request.survey.last().map_or(0.0, |station| station.md_m);
        if ![
            projection.bit_md_m,
            projection.ahead_m,
            projection.build_tendency_rad_per_m,
            projection.effective_turn_tendency_rad_per_m,
            projection.low_inclination_threshold_rad,
        ]
        .into_iter()
        .all(f64::is_finite)
            || projection.bit_md_m < survey_td_m
            || projection.ahead_m < 0.0
            || !(0.0..=std::f64::consts::PI).contains(&projection.low_inclination_threshold_rad)
        {
            push(
                &mut errors,
                "WF-TRAJECTORY-CONTRACT-020",
                "projection geometry must be finite, canonical, and begin at or beyond survey TD",
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
