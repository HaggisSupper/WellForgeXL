//! Deterministic well-planning trajectory calculations.

use nalgebra::Vector3;
use thiserror::Error;
use wellforge_trajectory_contract::{
    CalculatedStation, FormationCoverage, FormationEvaluation, FormationPick, FormationSense,
    InterpolationResult, InterpolationStatus, PositionResidual, ProjectionAssessment,
    ProjectionRequest, SlideResponse, SlideStatus, SpatialPosition, StationKind, Target,
    TargetEvaluation, TargetKind, TargetStatus, TrajectoryStation,
};

const DOGLEG_SINGULARITY_RAD: f64 = 1.0e-9;

/// Errors returned while constructing or comparing trajectory geometry.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TrajectoryError {
    /// Measured depth must increase strictly between adjacent stations.
    #[error("minimum-curvature stations must have strictly increasing measured depth")]
    NonIncreasingMeasuredDepth,
    /// A station contains a NaN or infinite geometry value.
    #[error("trajectory station geometry must be finite")]
    NonFiniteStationGeometry,
    /// A station lies outside the canonical MD, inclination, or azimuth domains.
    #[error("trajectory station geometry is outside canonical bounds")]
    InvalidStationGeometry,
    /// A dogleg is antipodal or too close to antipodal for an unambiguous minimum-curvature arc.
    #[error("trajectory dogleg is antipodal or ambiguous")]
    AmbiguousDogleg,
    /// A position or vertical-section azimuth contains a NaN or infinite value.
    #[error("position geometry must be finite")]
    NonFinitePosition,
    /// Finite input values produced a non-finite calculation result.
    #[error("trajectory calculation overflowed")]
    NumericalOverflow,
    /// A tendency projection bit depth precedes survey total depth.
    #[error("projection bit measured depth cannot precede survey total depth")]
    ProjectionBehindSurvey,
    /// A tendency projection request or calculated survey is not usable geometry.
    #[error("projection geometry is invalid")]
    InvalidProjectionGeometry,
}

/// A slide interval whose endpoint directions have been resolved from the survey course.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSlideInterval {
    /// Stable interval identity.
    pub uid: uuid::Uuid,
    /// Interval start MD in metres.
    pub md_in_m: f64,
    /// Interval end MD in metres.
    pub md_out_m: f64,
    /// Sliding footage within the course in metres.
    pub slide_length_m: f64,
    /// Survey-derived start inclination in radians.
    pub start_inclination_rad: f64,
    /// Survey-derived end inclination in radians.
    pub end_inclination_rad: f64,
    /// Survey-derived start azimuth in radians.
    pub start_azimuth_rad: f64,
    /// Survey-derived end azimuth in radians.
    pub end_azimuth_rad: f64,
    /// Commanded toolface in radians.
    pub commanded_toolface_rad: f64,
    /// Rotary-only build baseline in radians per metre.
    pub rotary_build_rad_per_m: f64,
    /// Rotary-only effective-turn baseline in radians per metre.
    pub rotary_effective_turn_rad_per_m: f64,
    /// Inclination below which response direction is indeterminate.
    pub low_inclination_threshold_rad: f64,
}

fn direction(inclination_rad: f64, azimuth_rad: f64) -> Vector3<f64> {
    Vector3::new(
        inclination_rad.sin() * azimuth_rad.cos(),
        inclination_rad.sin() * azimuth_rad.sin(),
        inclination_rad.cos(),
    )
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

fn all_finite(values: &[f64]) -> bool {
    values.iter().copied().all(f64::is_finite)
}

fn valid_angle(angle_rad: f64, upper_bound_rad: f64) -> bool {
    (0.0..upper_bound_rad).contains(&angle_rad)
}

fn valid_inclination(inclination_rad: f64) -> bool {
    (0.0..=std::f64::consts::PI).contains(&inclination_rad)
}

fn dogleg_from_directions(
    first: Vector3<f64>,
    second: Vector3<f64>,
) -> Result<f64, TrajectoryError> {
    let beta = clamp_unit(first.dot(&second)).acos();
    if std::f64::consts::PI - beta <= DOGLEG_SINGULARITY_RAD {
        return Err(TrajectoryError::AmbiguousDogleg);
    }
    Ok(beta)
}

fn dogleg(first: &CalculatedStation, second: &CalculatedStation) -> Result<f64, TrajectoryError> {
    dogleg_from_directions(
        direction(first.inclination_rad, first.azimuth_rad),
        direction(second.inclination_rad, second.azimuth_rad),
    )
}

fn ratio_factor(beta: f64) -> f64 {
    if beta.abs() < 1.0e-9 {
        1.0 + beta.powi(2) / 12.0 + beta.powi(4) / 120.0
    } else {
        2.0 * (beta / 2.0).tan() / beta
    }
}

fn displacement(
    first: &CalculatedStation,
    second: &CalculatedStation,
    length_m: f64,
) -> Result<(f64, f64, f64, f64, f64), TrajectoryError> {
    let beta = dogleg(first, second)?;
    let ratio = ratio_factor(beta);
    let average_direction = (direction(first.inclination_rad, first.azimuth_rad)
        + direction(second.inclination_rad, second.azimuth_rad))
        / 2.0;
    let delta = average_direction * ratio * length_m;
    if !all_finite(&[delta.x, delta.y, delta.z, beta, ratio]) {
        return Err(TrajectoryError::NumericalOverflow);
    }
    Ok((delta.x, delta.y, delta.z, beta, ratio))
}

fn wrap_positive(angle_rad: f64) -> f64 {
    angle_rad.rem_euclid(std::f64::consts::TAU)
}

fn wrap_signed(angle_rad: f64) -> f64 {
    let wrapped = wrap_positive(angle_rad);
    if wrapped > std::f64::consts::PI {
        wrapped - std::f64::consts::TAU
    } else {
        wrapped
    }
}

fn valid_station(station: &TrajectoryStation) -> Result<(), TrajectoryError> {
    if ![station.md_m, station.inclination_rad, station.azimuth_rad]
        .into_iter()
        .all(f64::is_finite)
    {
        return Err(TrajectoryError::NonFiniteStationGeometry);
    }
    if station.md_m < 0.0
        || !valid_inclination(station.inclination_rad)
        || !valid_angle(station.azimuth_rad, std::f64::consts::TAU)
    {
        return Err(TrajectoryError::InvalidStationGeometry);
    }
    Ok(())
}

fn valid_calculated_station(station: &CalculatedStation) -> bool {
    [
        station.md_m,
        station.inclination_rad,
        station.azimuth_rad,
        station.north_m,
        station.east_m,
        station.tvd_m,
        station.delta_md_m,
        station.dogleg_rad,
        station.ratio_factor,
        station.dls_rad_per_m,
    ]
    .into_iter()
    .all(f64::is_finite)
        && station.md_m >= 0.0
        && valid_inclination(station.inclination_rad)
        && valid_angle(station.azimuth_rad, std::f64::consts::TAU)
}

fn valid_calculated_course(stations: &[CalculatedStation]) -> bool {
    stations.iter().all(valid_calculated_station)
        && stations.windows(2).all(|pair| {
            pair[1].md_m > pair[0].md_m
                && dogleg_from_directions(
                    direction(pair[0].inclination_rad, pair[0].azimuth_rad),
                    direction(pair[1].inclination_rad, pair[1].azimuth_rad),
                )
                .is_ok()
        })
}

fn interpolated_direction(
    lower: &CalculatedStation,
    upper: &CalculatedStation,
    fraction: f64,
) -> Result<Vector3<f64>, TrajectoryError> {
    let first = direction(lower.inclination_rad, lower.azimuth_rad);
    let second = direction(upper.inclination_rad, upper.azimuth_rad);
    let beta = dogleg_from_directions(first, second)?;
    if beta < 1.0e-9 {
        return Ok((first + (second - first) * fraction).normalize());
    }
    let sine = beta.sin();
    Ok(
        first * (((1.0 - fraction) * beta).sin() / sine)
            + second * ((fraction * beta).sin() / sine),
    )
}

fn interpolation_result(md_m: f64, status: InterpolationStatus) -> InterpolationResult {
    InterpolationResult {
        md_m: md_m.is_finite().then_some(md_m),
        status,
        station: None,
    }
}

#[allow(clippy::float_cmp)]
fn exact_md_match(left_m: f64, right_m: f64) -> bool {
    // Exact-source interpolation deliberately preserves the source station only for an exact MD.
    left_m == right_m
}

/// Calculates cumulative positions and interval metrics by minimum curvature.
///
/// # Errors
///
/// Returns a typed error for non-finite, non-canonical, non-increasing, or antipodal stations.
pub fn minimum_curvature(
    stations: &[TrajectoryStation],
) -> Result<Vec<CalculatedStation>, TrajectoryError> {
    for station in stations {
        valid_station(station)?;
    }
    let Some(first) = stations.first() else {
        return Ok(Vec::new());
    };
    let mut calculated = Vec::with_capacity(stations.len());
    calculated.push(CalculatedStation::from(first));
    for station in &stations[1..] {
        let Some(previous) = calculated.last() else {
            return Ok(calculated);
        };
        let delta_md_m = station.md_m - previous.md_m;
        if delta_md_m <= 0.0 {
            return Err(TrajectoryError::NonIncreasingMeasuredDepth);
        }
        let mut current = CalculatedStation::from(station);
        let (north_m, east_m, tvd_m, dogleg_rad, ratio_factor) =
            displacement(previous, &current, delta_md_m)?;
        current.north_m = previous.north_m + north_m;
        current.east_m = previous.east_m + east_m;
        current.tvd_m = previous.tvd_m + tvd_m;
        current.delta_md_m = delta_md_m;
        current.dogleg_rad = dogleg_rad;
        current.ratio_factor = ratio_factor;
        current.dls_rad_per_m = dogleg_rad / delta_md_m;
        if !all_finite(&[
            current.north_m,
            current.east_m,
            current.tvd_m,
            current.delta_md_m,
            current.dogleg_rad,
            current.ratio_factor,
            current.dls_rad_per_m,
        ]) {
            return Err(TrajectoryError::NumericalOverflow);
        }
        calculated.push(current);
    }
    Ok(calculated)
}

/// Interpolates a calculated course at a measured depth using a partial minimum-curvature arc.
#[must_use]
pub fn interpolate_minimum_curvature(
    stations: &[CalculatedStation],
    md_m: f64,
) -> InterpolationResult {
    if !md_m.is_finite() {
        return interpolation_result(md_m, InterpolationStatus::InvalidMeasuredDepth);
    }
    if stations.is_empty() {
        return interpolation_result(md_m, InterpolationStatus::NoStations);
    }
    if !valid_calculated_course(stations) {
        return interpolation_result(md_m, InterpolationStatus::InvalidCourse);
    }
    let first = &stations[0];
    let Some(last) = stations.last() else {
        return interpolation_result(md_m, InterpolationStatus::NoStations);
    };
    if md_m < first.md_m {
        return interpolation_result(md_m, InterpolationStatus::BeforeStart);
    }
    if md_m > last.md_m {
        return interpolation_result(md_m, InterpolationStatus::BeyondTd);
    }
    if let Some(exact) = stations
        .iter()
        .find(|station| exact_md_match(station.md_m, md_m))
    {
        return InterpolationResult {
            md_m: Some(md_m),
            status: InterpolationStatus::Ok,
            station: Some(exact.clone()),
        };
    }
    let upper_index = stations.partition_point(|station| station.md_m < md_m);
    let lower = &stations[upper_index - 1];
    let upper = &stations[upper_index];
    let delta_md_m = md_m - lower.md_m;
    let fraction = delta_md_m / (upper.md_m - lower.md_m);
    let Ok(vector) = interpolated_direction(lower, upper, fraction) else {
        return interpolation_result(md_m, InterpolationStatus::InvalidCourse);
    };
    let mut partial = lower.clone();
    partial.source_uid = None;
    partial.kind = StationKind::Interpolated;
    partial.lower_source_uid = lower.source_uid.or(lower.lower_source_uid);
    partial.upper_source_uid = upper.source_uid.or(upper.upper_source_uid);
    partial.md_m = md_m;
    partial.inclination_rad = clamp_unit(vector.z).acos();
    partial.azimuth_rad = wrap_positive(vector.y.atan2(vector.x));
    let (north_m, east_m, tvd_m, dogleg_rad, ratio_factor) =
        match displacement(lower, &partial, delta_md_m) {
            Ok(displacement) => displacement,
            Err(TrajectoryError::NumericalOverflow) => {
                return interpolation_result(md_m, InterpolationStatus::NumericalOverflow);
            }
            Err(_) => return interpolation_result(md_m, InterpolationStatus::InvalidCourse),
        };
    partial.north_m = lower.north_m + north_m;
    partial.east_m = lower.east_m + east_m;
    partial.tvd_m = lower.tvd_m + tvd_m;
    partial.delta_md_m = delta_md_m;
    partial.dogleg_rad = dogleg_rad;
    partial.ratio_factor = ratio_factor;
    partial.dls_rad_per_m = dogleg_rad / delta_md_m;
    if !all_finite(&[
        partial.north_m,
        partial.east_m,
        partial.tvd_m,
        partial.delta_md_m,
        partial.dogleg_rad,
        partial.ratio_factor,
        partial.dls_rad_per_m,
    ]) {
        return interpolation_result(md_m, InterpolationStatus::NumericalOverflow);
    }
    InterpolationResult {
        md_m: Some(md_m),
        status: InterpolationStatus::Ok,
        station: Some(partial),
    }
}

/// Computes actual-minus-plan residuals in north/east/TVD and vertical-section axes.
///
/// # Errors
///
/// Returns [`TrajectoryError::NonFinitePosition`] when a position or vertical-section azimuth is not finite.
pub fn position_residual(
    actual: &SpatialPosition,
    planned: &SpatialPosition,
    vertical_section_azimuth_rad: f64,
) -> Result<PositionResidual, TrajectoryError> {
    if ![
        actual.north_m,
        actual.east_m,
        actual.tvd_m,
        planned.north_m,
        planned.east_m,
        planned.tvd_m,
        vertical_section_azimuth_rad,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        return Err(TrajectoryError::NonFinitePosition);
    }
    let north_m = actual.north_m - planned.north_m;
    let east_m = actual.east_m - planned.east_m;
    let tvd_m = actual.tvd_m - planned.tvd_m;
    let along_track_m =
        north_m * vertical_section_azimuth_rad.cos() + east_m * vertical_section_azimuth_rad.sin();
    let crossline_m =
        -north_m * vertical_section_azimuth_rad.sin() + east_m * vertical_section_azimuth_rad.cos();
    let horizontal_m = north_m.hypot(east_m);
    let error_3d_m = horizontal_m.hypot(tvd_m);
    if !all_finite(&[
        north_m,
        east_m,
        tvd_m,
        along_track_m,
        crossline_m,
        horizontal_m,
        error_3d_m,
    ]) {
        return Err(TrajectoryError::NumericalOverflow);
    }
    Ok(PositionResidual {
        north_m,
        east_m,
        tvd_m,
        along_track_m,
        crossline_m,
        horizontal_m,
        error_3d_m,
    })
}

fn projection_leg(
    start: &CalculatedStation,
    end_md_m: f64,
    request: &ProjectionRequest,
) -> Result<(CalculatedStation, bool), TrajectoryError> {
    let delta_md_m = end_md_m - start.md_m;
    if !delta_md_m.is_finite() || delta_md_m < 0.0 {
        return Err(TrajectoryError::NumericalOverflow);
    }
    let inclination_rad = (start.inclination_rad + request.build_tendency_rad_per_m * delta_md_m)
        .clamp(0.0, std::f64::consts::PI);
    let mean_inclination_rad = f64::midpoint(start.inclination_rad, inclination_rad);
    let mean_sine = mean_inclination_rad.sin();
    let threshold_sine = request.low_inclination_threshold_rad.sin();
    let turn_denominator = mean_sine.max(threshold_sine).max(1.0e-9);
    let azimuth_delta_rad =
        request.effective_turn_tendency_rad_per_m * delta_md_m / turn_denominator;
    let azimuth_rad = wrap_positive(start.azimuth_rad + azimuth_delta_rad);
    if !all_finite(&[
        inclination_rad,
        mean_inclination_rad,
        turn_denominator,
        azimuth_delta_rad,
        azimuth_rad,
    ]) {
        return Err(TrajectoryError::NumericalOverflow);
    }
    let low_inclination_turn_guard = request.effective_turn_tendency_rad_per_m != 0.0
        && threshold_sine >= mean_sine
        && threshold_sine >= 1.0e-9;
    let mut projected = CalculatedStation {
        source_uid: None,
        kind: StationKind::Projection,
        lower_source_uid: start.source_uid.or(start.lower_source_uid),
        upper_source_uid: None,
        md_m: end_md_m,
        inclination_rad,
        azimuth_rad,
        north_m: start.north_m,
        east_m: start.east_m,
        tvd_m: start.tvd_m,
        delta_md_m,
        dogleg_rad: 0.0,
        ratio_factor: 1.0,
        dls_rad_per_m: 0.0,
    };
    if delta_md_m > 0.0 {
        let (north_m, east_m, tvd_m, dogleg_rad, ratio_factor) =
            displacement(start, &projected, delta_md_m)?;
        projected.north_m += north_m;
        projected.east_m += east_m;
        projected.tvd_m += tvd_m;
        projected.dogleg_rad = dogleg_rad;
        projected.ratio_factor = ratio_factor;
        projected.dls_rad_per_m = dogleg_rad / delta_md_m;
    }
    if !valid_calculated_station(&projected) {
        return Err(TrajectoryError::NumericalOverflow);
    }
    Ok((projected, low_inclination_turn_guard))
}

/// Projects the final survey station through bit MD and then ahead using workbook tendencies.
///
/// # Errors
///
/// Returns a typed error for malformed survey geometry, a bit behind survey TD, ambiguous
/// projected doglegs, or numerical overflow.
pub fn project_tendency(
    survey: &[CalculatedStation],
    request: &ProjectionRequest,
) -> Result<ProjectionAssessment, TrajectoryError> {
    if !valid_calculated_course(survey) || survey.is_empty() {
        return Err(TrajectoryError::InvalidProjectionGeometry);
    }
    if ![
        request.bit_md_m,
        request.ahead_m,
        request.build_tendency_rad_per_m,
        request.effective_turn_tendency_rad_per_m,
        request.low_inclination_threshold_rad,
    ]
    .into_iter()
    .all(f64::is_finite)
        || request.ahead_m < 0.0
        || !valid_inclination(request.low_inclination_threshold_rad)
    {
        return Err(TrajectoryError::InvalidProjectionGeometry);
    }
    let last = survey
        .last()
        .ok_or(TrajectoryError::InvalidProjectionGeometry)?;
    if request.bit_md_m < last.md_m {
        return Err(TrajectoryError::ProjectionBehindSurvey);
    }
    let (bit, bit_guard) = projection_leg(last, request.bit_md_m, request)?;
    let projected_md_m = request.bit_md_m + request.ahead_m;
    if !projected_md_m.is_finite() {
        return Err(TrajectoryError::NumericalOverflow);
    }
    let (projected, ahead_guard) = projection_leg(&bit, projected_md_m, request)?;
    Ok(ProjectionAssessment {
        bit,
        projected,
        low_inclination_turn_guard: bit_guard || ahead_guard,
    })
}

fn invalid_target(target_uid: uuid::Uuid) -> TargetEvaluation {
    TargetEvaluation {
        target_uid,
        status: TargetStatus::InvalidGeometry,
        horizontal_utilization: None,
        vertical_utilization: None,
        local_major_m: None,
        local_minor_m: None,
        vertical_difference_m: None,
    }
}

fn overflow_target(target_uid: uuid::Uuid) -> TargetEvaluation {
    TargetEvaluation {
        target_uid,
        status: TargetStatus::NumericalOverflow,
        horizontal_utilization: None,
        vertical_utilization: None,
        local_major_m: None,
        local_minor_m: None,
        vertical_difference_m: None,
    }
}

/// Evaluates a spatial position against a target envelope with inclusive boundaries.
#[must_use]
pub fn evaluate_target(target: &Target, position: &SpatialPosition) -> TargetEvaluation {
    let finite = [
        target.north_m,
        target.east_m,
        target.tvd_m,
        target.major_m,
        target.minor_m,
        target.rotation_rad,
        target.vertical_tolerance_m,
        position.north_m,
        position.east_m,
        position.tvd_m,
    ]
    .into_iter()
    .all(f64::is_finite);
    if !finite
        || target.major_m <= 0.0
        || target.minor_m <= 0.0
        || target.vertical_tolerance_m < 0.0
    {
        return invalid_target(target.uid);
    }
    let north_m = position.north_m - target.north_m;
    let east_m = position.east_m - target.east_m;
    let local_major_m = north_m * target.rotation_rad.cos() + east_m * target.rotation_rad.sin();
    let local_minor_m = -north_m * target.rotation_rad.sin() + east_m * target.rotation_rad.cos();
    let horizontal_utilization = match target.kind {
        TargetKind::Point | TargetKind::Circle => north_m.hypot(east_m) / target.major_m,
        TargetKind::Ellipse => {
            (local_major_m / target.major_m).hypot(local_minor_m / target.minor_m)
        }
        TargetKind::Box => {
            (local_major_m.abs() / target.major_m).max(local_minor_m.abs() / target.minor_m)
        }
    };
    let vertical_difference_m = position.tvd_m - target.tvd_m;
    let (vertical_utilization, vertical_within) = if target.vertical_tolerance_m == 0.0 {
        (
            (vertical_difference_m == 0.0).then_some(0.0),
            vertical_difference_m == 0.0,
        )
    } else {
        let utilization = vertical_difference_m.abs() / target.vertical_tolerance_m;
        (Some(utilization), utilization <= 1.0)
    };
    if !all_finite(&[
        north_m,
        east_m,
        local_major_m,
        local_minor_m,
        horizontal_utilization,
        vertical_difference_m,
    ]) || vertical_utilization.is_some_and(|utilization| !utilization.is_finite())
    {
        return overflow_target(target.uid);
    }
    TargetEvaluation {
        target_uid: target.uid,
        status: if horizontal_utilization <= 1.0 && vertical_within {
            TargetStatus::Hit
        } else {
            TargetStatus::Miss
        },
        horizontal_utilization: Some(horizontal_utilization),
        vertical_utilization,
        local_major_m: Some(local_major_m),
        local_minor_m: Some(local_minor_m),
        vertical_difference_m: Some(vertical_difference_m),
    }
}

fn slide_result(slide_uid: uuid::Uuid, status: SlideStatus) -> SlideResponse {
    SlideResponse {
        slide_uid,
        status,
        build_rad_per_m: None,
        effective_turn_rad_per_m: None,
        residual_build_rad_per_m: None,
        residual_turn_rad_per_m: None,
        yield_rad_per_m: None,
        response_toolface_rad: None,
        toolface_error_rad: None,
    }
}

fn valid_slide_geometry(interval: &ResolvedSlideInterval) -> bool {
    [
        interval.md_in_m,
        interval.md_out_m,
        interval.slide_length_m,
        interval.start_inclination_rad,
        interval.end_inclination_rad,
        interval.start_azimuth_rad,
        interval.end_azimuth_rad,
        interval.commanded_toolface_rad,
        interval.rotary_build_rad_per_m,
        interval.rotary_effective_turn_rad_per_m,
        interval.low_inclination_threshold_rad,
    ]
    .into_iter()
    .all(f64::is_finite)
        && interval.md_in_m >= 0.0
        && valid_inclination(interval.start_inclination_rad)
        && valid_inclination(interval.end_inclination_rad)
        && valid_angle(interval.start_azimuth_rad, std::f64::consts::TAU)
        && valid_angle(interval.end_azimuth_rad, std::f64::consts::TAU)
        && valid_angle(interval.commanded_toolface_rad, std::f64::consts::TAU)
        && valid_inclination(interval.low_inclination_threshold_rad)
}

/// Calculates a rotary-adjusted slide response and toolface error.
#[must_use]
pub fn slide_response(interval: &ResolvedSlideInterval) -> SlideResponse {
    if !valid_slide_geometry(interval) {
        return slide_result(interval.uid, SlideStatus::InvalidGeometry);
    }
    let course_length_m = interval.md_out_m - interval.md_in_m;
    if course_length_m <= 0.0 || interval.slide_length_m <= 0.0 {
        return slide_result(interval.uid, SlideStatus::InvalidSlideLength);
    }
    if interval.slide_length_m > course_length_m {
        return slide_result(interval.uid, SlideStatus::InvalidGeometry);
    }
    let average_inclination_rad =
        f64::midpoint(interval.start_inclination_rad, interval.end_inclination_rad);
    if average_inclination_rad < interval.low_inclination_threshold_rad {
        return slide_result(interval.uid, SlideStatus::LowInclination);
    }
    let build_rad_per_m =
        (interval.end_inclination_rad - interval.start_inclination_rad) / course_length_m;
    let effective_turn_rad_per_m =
        wrap_signed(interval.end_azimuth_rad - interval.start_azimuth_rad)
            * average_inclination_rad.sin()
            / course_length_m;
    let residual_build_rad_per_m = (build_rad_per_m - interval.rotary_build_rad_per_m)
        * course_length_m
        / interval.slide_length_m;
    let residual_turn_rad_per_m =
        (effective_turn_rad_per_m - interval.rotary_effective_turn_rad_per_m) * course_length_m
            / interval.slide_length_m;
    let yield_rad_per_m = residual_build_rad_per_m.hypot(residual_turn_rad_per_m);
    let response_toolface_rad =
        wrap_positive(residual_turn_rad_per_m.atan2(residual_build_rad_per_m));
    let toolface_error_rad = wrap_signed(response_toolface_rad - interval.commanded_toolface_rad);
    if !all_finite(&[
        build_rad_per_m,
        effective_turn_rad_per_m,
        residual_build_rad_per_m,
        residual_turn_rad_per_m,
        yield_rad_per_m,
        response_toolface_rad,
        toolface_error_rad,
    ]) {
        return slide_result(interval.uid, SlideStatus::NumericalOverflow);
    }
    SlideResponse {
        slide_uid: interval.uid,
        status: SlideStatus::Ok,
        build_rad_per_m: Some(build_rad_per_m),
        effective_turn_rad_per_m: Some(effective_turn_rad_per_m),
        residual_build_rad_per_m: Some(residual_build_rad_per_m),
        residual_turn_rad_per_m: Some(residual_turn_rad_per_m),
        yield_rad_per_m: Some(yield_rad_per_m),
        response_toolface_rad: Some(response_toolface_rad),
        toolface_error_rad: Some(toolface_error_rad),
    }
}

fn formation_result(formation_uid: uuid::Uuid, coverage: FormationCoverage) -> FormationEvaluation {
    FormationEvaluation {
        formation_uid,
        coverage,
        actual_tvd_m: None,
        high_low_m: None,
        sense: None,
        within_tolerance: None,
    }
}

/// Evaluates formation high/low from a prognosed TVD and an optional actual pick MD.
#[must_use]
pub fn evaluate_formation(
    formation: &FormationPick,
    survey: &[CalculatedStation],
) -> FormationEvaluation {
    if !formation.prognosed_tvd_m.is_finite()
        || formation.prognosed_tvd_m < 0.0
        || formation
            .tolerance_m
            .is_some_and(|tolerance_m| !tolerance_m.is_finite() || tolerance_m < 0.0)
    {
        return formation_result(formation.uid, FormationCoverage::InvalidGeometry);
    }
    let Some(actual_md_m) = formation.actual_md_m else {
        return formation_result(formation.uid, FormationCoverage::NoActualPick);
    };
    let interpolation = interpolate_minimum_curvature(survey, actual_md_m);
    let coverage = match interpolation.status {
        InterpolationStatus::Ok => FormationCoverage::Ok,
        InterpolationStatus::NoStations => FormationCoverage::NoStations,
        InterpolationStatus::BeforeStart => FormationCoverage::BeforeStart,
        InterpolationStatus::BeyondTd => FormationCoverage::BeyondTd,
        InterpolationStatus::InvalidMeasuredDepth => FormationCoverage::InvalidMeasuredDepth,
        InterpolationStatus::InvalidCourse => FormationCoverage::InvalidCourse,
        InterpolationStatus::NumericalOverflow => FormationCoverage::NumericalOverflow,
    };
    let Some(station) = interpolation.station else {
        return formation_result(formation.uid, coverage);
    };
    let high_low_m = formation.prognosed_tvd_m - station.tvd_m;
    if !all_finite(&[station.tvd_m, high_low_m]) {
        return formation_result(formation.uid, FormationCoverage::NumericalOverflow);
    }
    let sense = if high_low_m > 0.0 {
        FormationSense::High
    } else if high_low_m < 0.0 {
        FormationSense::Low
    } else {
        FormationSense::OnPrognosis
    };
    FormationEvaluation {
        formation_uid: formation.uid,
        coverage,
        actual_tvd_m: Some(station.tvd_m),
        high_low_m: Some(high_low_m),
        sense: Some(sense),
        within_tolerance: formation
            .tolerance_m
            .map(|tolerance_m| high_low_m.abs() <= tolerance_m),
    }
}
