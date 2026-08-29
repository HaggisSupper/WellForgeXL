//! Pure orchestration for deterministic trajectory analysis.

use thiserror::Error;
use wellforge_trajectory_contract::{
    CalculatedStation, ContractError, InterpolationStatus, PlanSurveyResidual, SlideAssessment,
    SpatialPosition, TargetAssessment, TargetBasis, TrajectoryAnalysisRequest,
    TrajectoryCalculation, validate_request,
};
use wellforge_trajectory_core::{
    ResolvedSlideInterval, TrajectoryError, evaluate_formation, evaluate_target,
    interpolate_minimum_curvature, minimum_curvature, position_residual, project_tendency,
    slide_response,
};

/// Errors that prevent the pure trajectory calculation from completing.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrajectoryAnalysisError {
    /// The request failed strict contract validation.
    #[error("trajectory request contract validation failed")]
    InvalidRequest(Vec<ContractError>),
    /// Planned-course minimum-curvature calculation failed.
    #[error("planned trajectory calculation failed: {0}")]
    Plan(TrajectoryError),
    /// Surveyed-course minimum-curvature calculation failed.
    #[error("survey trajectory calculation failed: {0}")]
    Survey(TrajectoryError),
    /// A covered plan/survey position comparison overflowed.
    #[error("plan/survey residual calculation failed: {0}")]
    Residual(TrajectoryError),
    /// Optional tendency projection failed.
    #[error("trajectory tendency projection failed: {0}")]
    Projection(TrajectoryError),
    /// An internally calculated survey station lost its authoritative source identity.
    #[error("calculated survey station has no source identity")]
    MissingSurveySourceIdentity,
}

fn position(station: &CalculatedStation) -> SpatialPosition {
    SpatialPosition {
        north_m: station.north_m,
        east_m: station.east_m,
        tvd_m: station.tvd_m,
    }
}

fn projection_course(
    survey: &[CalculatedStation],
    projection: &wellforge_trajectory_contract::ProjectionAssessment,
) -> Vec<CalculatedStation> {
    let Some(last) = survey.last() else {
        return Vec::new();
    };
    let mut course = vec![last.clone()];
    if projection.bit.md_m > last.md_m {
        course.push(projection.bit.clone());
    }
    if course
        .last()
        .is_some_and(|station| projection.projected.md_m > station.md_m)
    {
        course.push(projection.projected.clone());
    }
    course
}

/// Composes one strict request into an ordered, pure numerical trajectory calculation.
///
/// # Errors
///
/// Returns a typed contract or numerical error when the authoritative plan, survey, residual, or
/// optional projection cannot be calculated. Typed MD coverage misses remain in the calculation.
#[allow(clippy::too_many_lines)]
pub fn analyze(
    request: &TrajectoryAnalysisRequest,
) -> Result<TrajectoryCalculation, TrajectoryAnalysisError> {
    validate_request(request).map_err(TrajectoryAnalysisError::InvalidRequest)?;
    let plan = minimum_curvature(&request.plan).map_err(TrajectoryAnalysisError::Plan)?;
    let survey = minimum_curvature(&request.survey).map_err(TrajectoryAnalysisError::Survey)?;

    let plan_survey_residuals = survey
        .iter()
        .map(|actual| {
            let planned = interpolate_minimum_curvature(&plan, actual.md_m);
            let residual = planned
                .station
                .as_ref()
                .map(|planned| {
                    position_residual(
                        &position(actual),
                        &position(planned),
                        request.vertical_section_azimuth_rad,
                    )
                    .map_err(TrajectoryAnalysisError::Residual)
                })
                .transpose()?;
            Ok(PlanSurveyResidual {
                survey_uid: actual
                    .source_uid
                    .ok_or(TrajectoryAnalysisError::MissingSurveySourceIdentity)?,
                md_m: actual.md_m,
                plan: planned,
                residual,
            })
        })
        .collect::<Result<Vec<_>, TrajectoryAnalysisError>>()?;

    let projection = request
        .projection
        .as_ref()
        .map(|projection_request| {
            project_tendency(&survey, projection_request)
                .map_err(TrajectoryAnalysisError::Projection)
        })
        .transpose()?;
    let projected_course = projection.as_ref().map_or_else(Vec::new, |assessment| {
        projection_course(&survey, assessment)
    });

    let targets = request
        .targets
        .iter()
        .map(|target| {
            let actual = interpolate_minimum_curvature(&survey, target.md_m);
            let (basis, station) = if actual.status == InterpolationStatus::Ok {
                (TargetBasis::Actual, actual.station)
            } else if actual.status == InterpolationStatus::BeyondTd && !projected_course.is_empty()
            {
                let projected = interpolate_minimum_curvature(&projected_course, target.md_m);
                if projected.status == InterpolationStatus::Ok {
                    (TargetBasis::Projected, projected.station)
                } else {
                    (TargetBasis::NotReached, None)
                }
            } else {
                (TargetBasis::NotReached, None)
            };
            let assessed_position = station.as_ref().map(position);
            let evaluation = assessed_position
                .as_ref()
                .map(|assessed_position| evaluate_target(target, assessed_position));
            TargetAssessment {
                target_uid: target.uid,
                md_m: target.md_m,
                basis,
                position: assessed_position,
                evaluation,
            }
        })
        .collect();

    let slides = request
        .slides
        .iter()
        .map(|slide| {
            let start = interpolate_minimum_curvature(&survey, slide.md_in_m);
            let end = interpolate_minimum_curvature(&survey, slide.md_out_m);
            let response = start
                .station
                .as_ref()
                .zip(end.station.as_ref())
                .map(|(start, end)| {
                    slide_response(&ResolvedSlideInterval {
                        uid: slide.uid,
                        md_in_m: slide.md_in_m,
                        md_out_m: slide.md_out_m,
                        slide_length_m: slide.slide_length_m,
                        start_inclination_rad: start.inclination_rad,
                        end_inclination_rad: end.inclination_rad,
                        start_azimuth_rad: start.azimuth_rad,
                        end_azimuth_rad: end.azimuth_rad,
                        commanded_toolface_rad: slide.commanded_toolface_rad,
                        rotary_build_rad_per_m: slide.rotary_build_rad_per_m,
                        rotary_effective_turn_rad_per_m: slide.rotary_effective_turn_rad_per_m,
                        low_inclination_threshold_rad: slide.low_inclination_threshold_rad,
                    })
                });
            SlideAssessment {
                slide_uid: slide.uid,
                start,
                end,
                response,
            }
        })
        .collect();

    let formations = request
        .formations
        .iter()
        .map(|formation| evaluate_formation(formation, &survey))
        .collect();

    Ok(TrajectoryCalculation {
        plan,
        survey,
        plan_survey_residuals,
        targets,
        slides,
        formations,
        projection,
    })
}
