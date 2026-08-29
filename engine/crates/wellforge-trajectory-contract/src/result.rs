//! Serializable trajectory calculation result types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

use crate::{StationKind, TrajectorySourceSet, TrajectoryStation};

/// A station with deterministic minimum-curvature coordinates and interval metrics.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CalculatedStation {
    /// Stable source-station identity, absent for a synthetic interpolated station.
    pub source_uid: Option<Uuid>,
    /// Source station semantic type.
    pub kind: StationKind,
    /// Lower source station identity for a synthetic interpolated station.
    pub lower_source_uid: Option<Uuid>,
    /// Upper source station identity for a synthetic interpolated station.
    pub upper_source_uid: Option<Uuid>,
    /// Measured depth in metres.
    pub md_m: f64,
    /// Inclination from vertical in radians.
    pub inclination_rad: f64,
    /// Azimuth in radians.
    pub azimuth_rad: f64,
    /// Northing in metres from the course origin.
    pub north_m: f64,
    /// Easting in metres from the course origin.
    pub east_m: f64,
    /// True vertical depth in metres from the course origin, positive down.
    pub tvd_m: f64,
    /// Measured-depth increment from the preceding station in metres.
    pub delta_md_m: f64,
    /// Dogleg over the preceding interval in radians.
    pub dogleg_rad: f64,
    /// Minimum-curvature ratio factor over the preceding interval.
    pub ratio_factor: f64,
    /// Dogleg severity in radians per metre over the preceding interval.
    pub dls_rad_per_m: f64,
}

impl From<&TrajectoryStation> for CalculatedStation {
    fn from(station: &TrajectoryStation) -> Self {
        Self {
            source_uid: Some(station.uid),
            kind: station.kind,
            lower_source_uid: None,
            upper_source_uid: None,
            md_m: station.md_m,
            inclination_rad: station.inclination_rad,
            azimuth_rad: station.azimuth_rad,
            north_m: 0.0,
            east_m: 0.0,
            tvd_m: 0.0,
            delta_md_m: 0.0,
            dogleg_rad: 0.0,
            ratio_factor: 1.0,
            dls_rad_per_m: 0.0,
        }
    }
}

/// Coverage state for a requested measured-depth interpolation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationStatus {
    /// The requested depth is represented by a calculated station.
    Ok,
    /// The course has no stations.
    NoStations,
    /// The requested depth precedes the first station.
    BeforeStart,
    /// The requested depth exceeds total depth.
    BeyondTd,
    /// The requested measured depth is non-finite.
    InvalidMeasuredDepth,
    /// The supplied calculated course is malformed or numerically invalid.
    InvalidCourse,
    /// Finite input values produced a non-finite calculation result.
    NumericalOverflow,
}

/// A calculated station when its requested measured depth is covered.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolationResult {
    /// Requested measured depth in metres, absent when the request was non-finite.
    pub md_m: Option<f64>,
    /// Course coverage state.
    pub status: InterpolationStatus,
    /// Calculated station, present only for [`InterpolationStatus::Ok`].
    pub station: Option<CalculatedStation>,
}

/// Cartesian residual between actual and planned positions.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PositionResidual {
    /// Actual-minus-plan north residual in metres.
    pub north_m: f64,
    /// Actual-minus-plan east residual in metres.
    pub east_m: f64,
    /// Actual-minus-plan TVD residual in metres.
    pub tvd_m: f64,
    /// Actual-minus-plan residual along the vertical section in metres.
    pub along_track_m: f64,
    /// Actual-minus-plan residual perpendicular to the vertical section in metres.
    pub crossline_m: f64,
    /// Horizontal residual magnitude in metres.
    pub horizontal_m: f64,
    /// Three-dimensional residual magnitude in metres.
    pub error_3d_m: f64,
}

/// North/east/TVD position in canonical SI coordinates.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialPosition {
    /// North coordinate in metres.
    pub north_m: f64,
    /// East coordinate in metres.
    pub east_m: f64,
    /// True vertical depth in metres, positive down.
    pub tvd_m: f64,
}

/// Target-envelope result state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetStatus {
    /// The position is within both target boundaries.
    Hit,
    /// The position is outside a target boundary.
    Miss,
    /// Target geometry is infeasible.
    InvalidGeometry,
    /// Finite geometry produced a non-finite evaluation result.
    NumericalOverflow,
}

/// Target-envelope evaluation in target-local coordinates.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetEvaluation {
    /// Stable target identity.
    pub target_uid: Uuid,
    /// Target-envelope result state.
    pub status: TargetStatus,
    /// Horizontal envelope utilization, present for valid geometry.
    pub horizontal_utilization: Option<f64>,
    /// Vertical envelope utilization for positive tolerances, or zero for an exact zero-tolerance
    /// match; absent when valid zero-tolerance geometry is exceeded or geometry is invalid.
    pub vertical_utilization: Option<f64>,
    /// North/east displacement projected onto the target major axis in metres.
    pub local_major_m: Option<f64>,
    /// North/east displacement projected onto the target minor axis in metres.
    pub local_minor_m: Option<f64>,
    /// Position-minus-target TVD in metres.
    pub vertical_difference_m: Option<f64>,
}

/// Slide-response result state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideStatus {
    /// Slide response is available.
    Ok,
    /// The course or slide length is not positive.
    InvalidSlideLength,
    /// The average inclination is below the configured threshold.
    LowInclination,
    /// Input geometry is non-finite or outside canonical bounds.
    InvalidGeometry,
    /// Finite geometry produced a non-finite response value.
    NumericalOverflow,
}

/// Slide build/turn response after removal of rotary baselines.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SlideResponse {
    /// Stable slide identity.
    pub slide_uid: Uuid,
    /// Slide response state.
    pub status: SlideStatus,
    /// Build rate in radians per metre, present only when status is OK.
    pub build_rad_per_m: Option<f64>,
    /// Effective-turn rate in radians per metre, present only when status is OK.
    pub effective_turn_rad_per_m: Option<f64>,
    /// Rotary-adjusted build rate in radians per metre, present only when status is OK.
    pub residual_build_rad_per_m: Option<f64>,
    /// Rotary-adjusted effective-turn rate in radians per metre, present only when status is OK.
    pub residual_turn_rad_per_m: Option<f64>,
    /// Rotary-adjusted response magnitude in radians per metre, present only when status is OK.
    pub yield_rad_per_m: Option<f64>,
    /// Response toolface in radians in [0, 2π), present only when status is OK.
    pub response_toolface_rad: Option<f64>,
    /// Signed response-minus-commanded toolface error in radians, present only when status is OK.
    pub toolface_error_rad: Option<f64>,
}

/// Formation-pick course coverage state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormationCoverage {
    /// The actual pick is covered by the survey course.
    Ok,
    /// No actual pick measured depth was supplied.
    NoActualPick,
    /// The actual pick precedes the first survey station.
    BeforeStart,
    /// The actual pick exceeds survey total depth.
    BeyondTd,
    /// The survey course contains no stations.
    NoStations,
    /// The actual pick measured depth is non-finite.
    InvalidMeasuredDepth,
    /// The calculated survey course is malformed or numerically invalid.
    InvalidCourse,
    /// Formation prognosis or tolerance geometry is invalid.
    InvalidGeometry,
    /// Finite input values produced a non-finite formation evaluation.
    NumericalOverflow,
}

/// Structural position of a formation pick relative to prognosis.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormationSense {
    /// Actual top is shallower than prognosis.
    High,
    /// Actual top is deeper than prognosis.
    Low,
    /// Actual and prognosed TVDs agree exactly.
    OnPrognosis,
}

/// Formation-pick evaluation using exact partial minimum-curvature interpolation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormationEvaluation {
    /// Stable formation-pick identity.
    pub formation_uid: Uuid,
    /// Survey coverage state at the actual pick MD.
    pub coverage: FormationCoverage,
    /// Interpolated actual TVD in metres, present only for covered picks.
    pub actual_tvd_m: Option<f64>,
    /// Prognosed-minus-actual TVD in metres, present only for covered picks.
    pub high_low_m: Option<f64>,
    /// Structural sense, present only for covered picks.
    pub sense: Option<FormationSense>,
    /// Whether absolute high/low is within an optional tolerance, present only when supplied and covered.
    pub within_tolerance: Option<bool>,
}

/// Survey position and its exact measured-depth plan comparison.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSurveyResidual {
    /// Stable surveyed-station identity.
    pub survey_uid: Uuid,
    /// Survey measured depth in metres.
    pub md_m: f64,
    /// Exact plan interpolation and its typed coverage state.
    pub plan: InterpolationResult,
    /// Actual-minus-plan residual, present only when the plan covers the survey MD.
    pub residual: Option<PositionResidual>,
}

/// Position basis used to evaluate one requested target depth.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetBasis {
    /// The survey covers the target measured depth.
    Actual,
    /// The optional projection covers a target beyond survey total depth.
    Projected,
    /// Neither actual survey nor projection reaches the target measured depth.
    NotReached,
}

/// Ordered target-envelope assessment at its requested measured depth.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetAssessment {
    /// Stable target identity.
    pub target_uid: Uuid,
    /// Requested target measured depth in metres.
    pub md_m: f64,
    /// Source used for the assessment position.
    pub basis: TargetBasis,
    /// Exact position used for evaluation, absent when not reached.
    pub position: Option<SpatialPosition>,
    /// Target evaluation, absent when not reached.
    pub evaluation: Option<TargetEvaluation>,
}

/// Ordered slide assessment with survey-derived endpoint directions.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SlideAssessment {
    /// Stable slide identity.
    pub slide_uid: Uuid,
    /// Exact survey interpolation at slide start MD.
    pub start: InterpolationResult,
    /// Exact survey interpolation at slide end MD.
    pub end: InterpolationResult,
    /// Slide response, present only when both endpoints are covered.
    pub response: Option<SlideResponse>,
}

/// Survey tendency projection through the bit and ahead endpoint.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionAssessment {
    /// Calculated station at bit measured depth.
    pub bit: CalculatedStation,
    /// Calculated station at bit MD plus requested ahead distance.
    pub projected: CalculatedStation,
    /// Whether nonzero effective turn used the low-inclination threshold denominator.
    pub low_inclination_turn_guard: bool,
}

/// Complete pure numerical trajectory calculation, preserving request order.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrajectoryCalculation {
    /// Calculated planned stations in request order.
    pub plan: Vec<CalculatedStation>,
    /// Calculated surveyed stations in request order.
    pub survey: Vec<CalculatedStation>,
    /// One plan comparison for every survey station in request order.
    pub plan_survey_residuals: Vec<PlanSurveyResidual>,
    /// Target assessments in request order.
    pub targets: Vec<TargetAssessment>,
    /// Slide assessments in request order.
    pub slides: Vec<SlideAssessment>,
    /// Formation evaluations in request order.
    pub formations: Vec<FormationEvaluation>,
    /// Optional projection assessment.
    pub projection: Option<ProjectionAssessment>,
}

/// Completion state of a serialized trajectory analysis.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrajectoryAnalysisStatus {
    /// Calculation completed without typed coverage or evaluation warnings.
    Complete,
    /// Calculation completed with one or more typed coverage or evaluation warnings.
    CompleteWithWarnings,
    /// Calculation did not complete and is not an accepted result.
    Failed,
}

/// Scope and deterministic method statement for an analysis result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicabilityStatement {
    /// Named calculation method.
    pub method: String,
    /// Whether identical canonical input deterministically produces identical calculation output.
    pub deterministic: bool,
    /// Explicit applicability limitations or warnings.
    pub limitations: Vec<String>,
}

/// Build and integrity evidence finalized by the executable boundary.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CalculationEvidence {
    /// Engine semantic version.
    pub engine_version: String,
    /// Compiler identity.
    pub compiler_version: String,
    /// Compilation target triple.
    pub target_triple: String,
    /// Cargo lockfile SHA-256 identity.
    pub lockfile_hash: String,
    /// Normalized request SHA-256 identity.
    pub request_hash: String,
    /// Normalized result SHA-256 identity.
    pub result_hash: String,
}

/// Strict aggregate trajectory analysis result.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TrajectoryAnalysisResult {
    /// Semantic contract version copied from the request.
    pub contract_version: String,
    /// Stable analysis identity copied from the request.
    pub analysis_id: Uuid,
    /// Immutable authoritative source references copied from the request.
    pub sources: TrajectorySourceSet,
    /// Overall completion state.
    pub status: TrajectoryAnalysisStatus,
    /// Deterministic method and applicability statement.
    pub applicability: ApplicabilityStatement,
    /// Build and content-integrity evidence.
    pub evidence: CalculationEvidence,
    /// Pure numerical calculation.
    pub calculation: TrajectoryCalculation,
}

fn option_is_finite(value: Option<f64>) -> bool {
    value.is_none_or(f64::is_finite)
}

fn station_is_finite(station: &CalculatedStation) -> bool {
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
}

fn interpolation_is_finite(interpolation: &InterpolationResult) -> bool {
    option_is_finite(interpolation.md_m)
        && interpolation.station.as_ref().is_none_or(station_is_finite)
}

fn residual_is_finite(residual: &PositionResidual) -> bool {
    [
        residual.north_m,
        residual.east_m,
        residual.tvd_m,
        residual.along_track_m,
        residual.crossline_m,
        residual.horizontal_m,
        residual.error_3d_m,
    ]
    .into_iter()
    .all(f64::is_finite)
}

fn position_is_finite(position: &SpatialPosition) -> bool {
    [position.north_m, position.east_m, position.tvd_m]
        .into_iter()
        .all(f64::is_finite)
}

fn target_evaluation_is_finite(evaluation: &TargetEvaluation) -> bool {
    [
        evaluation.horizontal_utilization,
        evaluation.vertical_utilization,
        evaluation.local_major_m,
        evaluation.local_minor_m,
        evaluation.vertical_difference_m,
    ]
    .into_iter()
    .all(option_is_finite)
}

fn slide_response_is_finite(response: &SlideResponse) -> bool {
    [
        response.build_rad_per_m,
        response.effective_turn_rad_per_m,
        response.residual_build_rad_per_m,
        response.residual_turn_rad_per_m,
        response.yield_rad_per_m,
        response.response_toolface_rad,
        response.toolface_error_rad,
    ]
    .into_iter()
    .all(option_is_finite)
}

fn calculation_is_finite(calculation: &TrajectoryCalculation) -> bool {
    calculation.plan.iter().all(station_is_finite)
        && calculation.survey.iter().all(station_is_finite)
        && calculation.plan_survey_residuals.iter().all(|item| {
            item.md_m.is_finite()
                && interpolation_is_finite(&item.plan)
                && item.residual.as_ref().is_none_or(residual_is_finite)
        })
        && calculation.targets.iter().all(|item| {
            item.md_m.is_finite()
                && item.position.as_ref().is_none_or(position_is_finite)
                && item
                    .evaluation
                    .as_ref()
                    .is_none_or(target_evaluation_is_finite)
        })
        && calculation.slides.iter().all(|item| {
            interpolation_is_finite(&item.start)
                && interpolation_is_finite(&item.end)
                && item.response.as_ref().is_none_or(slide_response_is_finite)
        })
        && calculation
            .formations
            .iter()
            .all(|item| option_is_finite(item.actual_tvd_m) && option_is_finite(item.high_low_m))
        && calculation
            .projection
            .as_ref()
            .is_none_or(|item| station_is_finite(&item.bit) && station_is_finite(&item.projected))
}

#[derive(Serialize)]
struct SerializableTrajectoryAnalysisResult<'a> {
    contract_version: &'a str,
    analysis_id: Uuid,
    sources: &'a TrajectorySourceSet,
    status: TrajectoryAnalysisStatus,
    applicability: &'a ApplicabilityStatement,
    evidence: &'a CalculationEvidence,
    calculation: &'a TrajectoryCalculation,
}

impl Serialize for TrajectoryAnalysisResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !calculation_is_finite(&self.calculation) {
            return Err(serde::ser::Error::custom(
                "trajectory analysis result contains a non-finite numerical component",
            ));
        }
        SerializableTrajectoryAnalysisResult {
            contract_version: &self.contract_version,
            analysis_id: self.analysis_id,
            sources: &self.sources,
            status: self.status,
            applicability: &self.applicability,
            evidence: &self.evidence,
            calculation: &self.calculation,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::{InterpolationResult, InterpolationStatus, TargetEvaluation, TargetStatus};
    use uuid::Uuid;

    #[test]
    fn target_evaluation_with_unbounded_vertical_utilization_round_trips_as_json() {
        let result = TargetEvaluation {
            target_uid: Uuid::from_u128(1),
            status: TargetStatus::Miss,
            horizontal_utilization: Some(0.0),
            vertical_utilization: None,
            local_major_m: Some(0.0),
            local_minor_m: Some(0.0),
            vertical_difference_m: Some(1.0),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("Infinity"));
        assert_eq!(
            serde_json::from_str::<TargetEvaluation>(&json).unwrap(),
            result
        );
    }

    #[test]
    fn invalid_nonfinite_depth_results_round_trip_without_nonfinite_json_values() {
        for status in [
            InterpolationStatus::InvalidMeasuredDepth,
            InterpolationStatus::NumericalOverflow,
        ] {
            let result = InterpolationResult {
                md_m: None,
                status,
                station: None,
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(!json.contains("NaN"));
            assert!(!json.contains("Infinity"));
            assert_eq!(
                serde_json::from_str::<InterpolationResult>(&json).unwrap(),
                result
            );
        }
    }
}
