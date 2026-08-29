//! Calculation-authoritative trajectory request types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::SourceObjectRef;

/// North reference used by every azimuth in one request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AzimuthReference {
    /// Geographic true north.
    TrueNorth,
    /// Projected grid north.
    GridNorth,
    /// Magnetic north at the named source epoch.
    MagneticNorth,
}

/// WITSML-aligned measured-depth reference kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MdDatumKind {
    /// Rotary Kelly bushing elevation.
    RotaryKellyBushing,
    /// Drill-floor elevation.
    DrillFloor,
    /// Mean sea level.
    MeanSeaLevel,
    /// Named project-specific datum.
    Other,
}

/// Stable measured-depth datum identity.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MdDatum {
    /// Stable datum identity.
    pub uid: Uuid,
    /// Human-readable datum name.
    pub name: String,
    /// Datum classification.
    pub kind: MdDatumKind,
}

/// Authoritative WITSML identities for the trajectory comparison.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrajectorySourceSet {
    /// Parent Well object.
    pub well: SourceObjectRef,
    /// Parent Wellbore object.
    pub wellbore: SourceObjectRef,
    /// Planned Trajectory object.
    pub plan_trajectory: SourceObjectRef,
    /// Surveyed Trajectory object.
    pub survey_trajectory: SourceObjectRef,
}

/// Semantic role of one trajectory station.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StationKind {
    /// Common tie-in station.
    TieIn,
    /// Planned station.
    Plan,
    /// Surveyed station.
    Survey,
    /// Project-ahead station.
    Projection,
    /// Exact partial-course interpolation station, generated only as a calculation result.
    Interpolated,
}

/// One WITSML-aligned trajectory station in canonical SI.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrajectoryStation {
    /// Stable station identity corresponding to WITSML station `uid`.
    pub uid: Uuid,
    /// Station semantic type.
    pub kind: StationKind,
    /// Measured depth in metres.
    pub md_m: f64,
    /// Inclination from vertical in radians.
    pub inclination_rad: f64,
    /// Azimuth in radians using the request reference.
    pub azimuth_rad: f64,
}

/// Supported horizontal target envelope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// Circular point tolerance.
    Point,
    /// Circular target.
    Circle,
    /// Rotated ellipse.
    Ellipse,
    /// Rotated rectangular box.
    Box,
}

/// One target envelope in north/east/TVD coordinates.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Stable target identity.
    pub uid: Uuid,
    /// Human-readable target name.
    pub name: String,
    /// Horizontal target shape.
    pub kind: TargetKind,
    /// Exact measured depth at which this target envelope is evaluated.
    pub md_m: f64,
    /// Target centre north coordinate in metres.
    pub north_m: f64,
    /// Target centre east coordinate in metres.
    pub east_m: f64,
    /// Target centre TVD in metres, positive down.
    pub tvd_m: f64,
    /// Major radius or box half-width in metres.
    pub major_m: f64,
    /// Minor radius or box half-height in metres.
    pub minor_m: f64,
    /// Clockwise rotation from north in radians.
    pub rotation_rad: f64,
    /// Permitted absolute TVD difference in metres.
    pub vertical_tolerance_m: f64,
}

/// One steering slide interval and its rotary baseline.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SlideInterval {
    /// Stable interval identity.
    pub uid: Uuid,
    /// Interval start MD in metres.
    pub md_in_m: f64,
    /// Interval end MD in metres.
    pub md_out_m: f64,
    /// Sliding footage within the course in metres.
    pub slide_length_m: f64,
    /// Commanded toolface in radians.
    pub commanded_toolface_rad: f64,
    /// Rotary-only build baseline in radians per metre.
    pub rotary_build_rad_per_m: f64,
    /// Rotary-only effective-turn baseline in radians per metre.
    pub rotary_effective_turn_rad_per_m: f64,
    /// Inclination below which response direction is indeterminate.
    pub low_inclination_threshold_rad: f64,
}

/// Optional tendency projection beyond the surveyed trajectory.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionRequest {
    /// Bit measured depth in metres; it cannot precede survey total depth.
    pub bit_md_m: f64,
    /// Projection distance ahead of the bit in metres.
    pub ahead_m: f64,
    /// Inclination build tendency in radians per metre.
    pub build_tendency_rad_per_m: f64,
    /// Effective turn tendency in radians per metre.
    pub effective_turn_tendency_rad_per_m: f64,
    /// Inclination threshold used to guard effective-turn conversion.
    pub low_inclination_threshold_rad: f64,
}

/// One formation prognosis and optional surveyed pick depth.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormationPick {
    /// Stable formation-pick identity.
    pub uid: Uuid,
    /// Human-readable formation name.
    pub name: String,
    /// Prognosed TVD in metres, positive down.
    pub prognosed_tvd_m: f64,
    /// Survey MD at the actual pick, when observed.
    pub actual_md_m: Option<f64>,
    /// Optional high/low absolute tolerance in metres.
    pub tolerance_m: Option<f64>,
}

/// Complete versioned trajectory analysis request.
///
/// These types are the already-normalized canonical-SI calculation boundary shared with the BHA
/// engine. They are not a future unit-preserving wire adapter.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrajectoryAnalysisRequest {
    /// Semantic contract version; Release 1 accepts major version 1 only.
    pub contract_version: String,
    /// Stable analysis identity.
    pub analysis_id: Uuid,
    /// Authoritative Well, Wellbore, plan and survey identities.
    pub sources: TrajectorySourceSet,
    /// Measured-depth datum shared by plan and survey.
    pub md_datum: MdDatum,
    /// Azimuth north reference.
    pub azimuth_reference: AzimuthReference,
    /// Vertical-section azimuth in radians.
    pub vertical_section_azimuth_rad: f64,
    /// Ordered planned stations.
    pub plan: Vec<TrajectoryStation>,
    /// Ordered surveyed stations.
    pub survey: Vec<TrajectoryStation>,
    /// Target envelopes.
    pub targets: Vec<Target>,
    /// Steering slide intervals.
    pub slides: Vec<SlideInterval>,
    /// Formation prognosis and pick records.
    pub formations: Vec<FormationPick>,
    /// Optional tendency projection from survey total depth through the bit and ahead.
    pub projection: Option<ProjectionRequest>,
}
