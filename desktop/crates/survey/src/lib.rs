//! Deterministic wellbore survey geometry and correction primitives.
//!
//! Angles are radians and all lengths are metres. UI unit conversion belongs at
//! the application boundary; calculations in this crate always use SI.

use std::{f64::consts::TAU, ops::Sub};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wellforge_3d::{
    SceneDocumentV1, SceneError, SceneLayerV1, SceneMarkerV1, ScenePoint, SceneProvenanceV1,
};
use wellforge_core::{Metres, PlotPoint, PlotSpec, PlotTrace, Radians};

const SINGULARITY_EPSILON: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Result<Self, SurveyError> {
        let norm = self.norm();
        if !norm.is_finite() || norm < SINGULARITY_EPSILON {
            return Err(SurveyError::SingularVector);
        }
        Ok(Self::new(self.x / norm, self.y / norm, self.z / norm))
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Matrix3 {
    pub rows: [[f64; 3]; 3],
}

impl Matrix3 {
    pub const fn new(rows: [[f64; 3]; 3]) -> Self {
        Self { rows }
    }
    pub fn transpose(self) -> Self {
        Self::new([
            [self.rows[0][0], self.rows[1][0], self.rows[2][0]],
            [self.rows[0][1], self.rows[1][1], self.rows[2][1]],
            [self.rows[0][2], self.rows[1][2], self.rows[2][2]],
        ])
    }
    pub fn multiply(self, rhs: Self) -> Self {
        let mut rows = [[0.0; 3]; 3];
        for (row_index, row) in rows.iter_mut().enumerate() {
            for (column_index, value) in row.iter_mut().enumerate() {
                *value = (0..3)
                    .map(|index| self.rows[row_index][index] * rhs.rows[index][column_index])
                    .sum();
            }
        }
        Self::new(rows)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct SurveyStation {
    pub md_m: Metres,
    pub inclination_rad: Radians,
    pub azimuth_true_rad: Radians,
}
impl SurveyStation {
    pub const fn new(md_m: Metres, inclination_rad: Radians, azimuth_true_rad: Radians) -> Self {
        Self {
            md_m,
            inclination_rad,
            azimuth_true_rad,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Displacement {
    pub north_m: Metres,
    pub east_m: Metres,
    pub tvd_m: Metres,
    pub dogleg_rad: Radians,
    pub dogleg_severity_rad_per_m: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct MagneticField {
    pub bx: f64,
    pub by: f64,
    pub bz: f64,
}
impl MagneticField {
    pub const fn new(bx: f64, by: f64, bz: f64) -> Self {
        Self { bx, by, bz }
    }
    pub const fn vector(self) -> Vector3 {
        Vector3::new(self.bx, self.by, self.bz)
    }
}

/// Properties required for the simply-supported-beam sag correction.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct SagModel {
    pub span_m: f64,
    pub distributed_load_n_per_m: f64,
    pub youngs_modulus_pa: f64,
    pub second_moment_m4: f64,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SurveyError {
    #[error("Measured depth must increase between survey stations")]
    NonIncreasingMeasuredDepth,
    #[error("Survey inputs must be finite")]
    NonFiniteInput,
    #[error("Survey calculation produced a non-finite result")]
    NonFiniteResult,
    #[error("A direction vector is singular")]
    SingularVector,
    #[error("Azimuth is undefined for a vertical tool axis")]
    UndefinedAzimuth,
    #[error("High-side frame is undefined for a vertical wellbore")]
    UndefinedHighSide,
    #[error("Total magnetic field is smaller than the transverse magnetic field")]
    InvalidMagneticField,
    #[error("Crossover bounds must satisfy 0 <= low < high <= pi/2")]
    InvalidCrossoverRange,
    #[error("Beam sag properties must be finite and strictly positive")]
    InvalidSagModel,
    #[error("Root-sum-square requires at least one finite value")]
    InvalidRootSumSquareInput,
    #[error("3D scene construction failed: {0}")]
    SceneConstruction(#[from] SceneError),
}

impl SurveyError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NonIncreasingMeasuredDepth => "NON_INCREASING_MEASURED_DEPTH",
            Self::NonFiniteInput => "NON_FINITE_INPUT",
            Self::NonFiniteResult => "NON_FINITE_RESULT",
            Self::SingularVector => "SINGULAR_VECTOR",
            Self::UndefinedAzimuth => "UNDEFINED_AZIMUTH",
            Self::UndefinedHighSide => "UNDEFINED_HIGH_SIDE",
            Self::InvalidMagneticField => "INVALID_MAGNETIC_FIELD",
            Self::InvalidCrossoverRange => "INVALID_CROSSOVER_RANGE",
            Self::InvalidSagModel => "INVALID_SAG_MODEL",
            Self::InvalidRootSumSquareInput => "INVALID_ROOT_SUM_SQUARE_INPUT",
            Self::SceneConstruction(error) => error.code(),
        }
    }
}

/// Minimum-curvature displacement between two stations.
pub fn calculate_displacement_minimum_curvature(
    start: &SurveyStation,
    end: &SurveyStation,
) -> Result<Displacement, SurveyError> {
    validate_station(*start)?;
    validate_station(*end)?;
    let delta_md = end.md_m.get() - start.md_m.get();
    if delta_md <= 0.0 {
        return Err(SurveyError::NonIncreasingMeasuredDepth);
    }
    let cosine_dogleg = (start.inclination_rad.get().cos() * end.inclination_rad.get().cos()
        + start.inclination_rad.get().sin()
            * end.inclination_rad.get().sin()
            * (end.azimuth_true_rad.get() - start.azimuth_true_rad.get()).cos())
    .clamp(-1.0, 1.0);
    let dogleg_rad = cosine_dogleg.acos();
    let ratio_factor = if dogleg_rad < SINGULARITY_EPSILON {
        1.0
    } else {
        2.0 * (dogleg_rad / 2.0).tan() / dogleg_rad
    };
    let north_m = delta_md
        * 0.5
        * (start.inclination_rad.get().sin() * start.azimuth_true_rad.get().cos()
            + end.inclination_rad.get().sin() * end.azimuth_true_rad.get().cos())
        * ratio_factor;
    let east_m = delta_md
        * 0.5
        * (start.inclination_rad.get().sin() * start.azimuth_true_rad.get().sin()
            + end.inclination_rad.get().sin() * end.azimuth_true_rad.get().sin())
        * ratio_factor;
    let tvd_m = delta_md
        * 0.5
        * (start.inclination_rad.get().cos() + end.inclination_rad.get().cos())
        * ratio_factor;
    let dogleg_severity_rad_per_m = dogleg_rad / delta_md;
    if !dogleg_severity_rad_per_m.is_finite() {
        return Err(SurveyError::NonFiniteResult);
    }
    Ok(Displacement {
        north_m: Metres::try_new(north_m).map_err(|_| SurveyError::NonFiniteInput)?,
        east_m: Metres::try_new(east_m).map_err(|_| SurveyError::NonFiniteInput)?,
        tvd_m: Metres::try_new(tvd_m).map_err(|_| SurveyError::NonFiniteInput)?,
        dogleg_rad: Radians::try_new(dogleg_rad).map_err(|_| SurveyError::NonFiniteInput)?,
        dogleg_severity_rad_per_m,
    })
}

/// Inclination from three-axis gravity measurements in tool coordinates.
pub fn inclination_from_accelerometer(gravity: Vector3) -> Result<f64, SurveyError> {
    let gravity = gravity.normalized()?;
    Ok((gravity.x.hypot(gravity.y)).atan2(gravity.z))
}

/// True azimuth from accelerometer and magnetometer measurements.
/// Tool Z is axial; gravity points down; declination transforms magnetic to true north.
pub fn true_azimuth_from_six_axis(
    gravity: Vector3,
    magnetic: MagneticField,
    declination_rad: f64,
) -> Result<f64, SurveyError> {
    if !declination_rad.is_finite() {
        return Err(SurveyError::NonFiniteInput);
    }
    let down = gravity.normalized()?;
    let field = magnetic.vector().normalized()?;
    let north = (field - scale(down, field.dot(down))).normalized()?;
    let east = down.cross(north).normalized()?;
    let tool_axis = Vector3::new(0.0, 0.0, 1.0);
    let north_component = tool_axis.dot(north);
    let east_component = tool_axis.dot(east);
    if north_component.hypot(east_component) < SINGULARITY_EPSILON {
        return Err(SurveyError::UndefinedAzimuth);
    }
    Ok(normalize_azimuth(
        east_component.atan2(north_component) + declination_rad,
    ))
}

/// Magnetic toolface measured clockwise from tool X toward Y.
pub fn magnetic_toolface(magnetic: MagneticField) -> Result<f64, SurveyError> {
    if !magnetic.bx.is_finite() || !magnetic.by.is_finite() {
        return Err(SurveyError::NonFiniteInput);
    }
    if magnetic.bx.hypot(magnetic.by) < SINGULARITY_EPSILON {
        return Err(SurveyError::SingularVector);
    }
    Ok(normalize_azimuth(magnetic.by.atan2(magnetic.bx)))
}

/// High-side (gravity) toolface measured clockwise from tool X toward Y.
pub fn high_side_toolface(gravity: Vector3) -> Result<f64, SurveyError> {
    if !gravity.x.is_finite() || !gravity.y.is_finite() {
        return Err(SurveyError::NonFiniteInput);
    }
    if gravity.x.hypot(gravity.y) < SINGULARITY_EPSILON {
        return Err(SurveyError::SingularVector);
    }
    Ok(normalize_azimuth(gravity.y.atan2(gravity.x)))
}

/// Blends magnetic and high-side toolface across a defined inclination range.
pub fn get_toolface_with_crossover(
    inclination_rad: f64,
    magnetic_toolface_rad: f64,
    high_side_toolface_rad: f64,
    low_inclination_rad: f64,
    high_inclination_rad: f64,
) -> Result<f64, SurveyError> {
    if ![
        inclination_rad,
        magnetic_toolface_rad,
        high_side_toolface_rad,
    ]
    .iter()
    .all(|value| value.is_finite())
    {
        return Err(SurveyError::NonFiniteInput);
    }
    if !(0.0 <= low_inclination_rad
        && low_inclination_rad < high_inclination_rad
        && high_inclination_rad <= std::f64::consts::FRAC_PI_2)
    {
        return Err(SurveyError::InvalidCrossoverRange);
    }
    if inclination_rad <= low_inclination_rad {
        return Ok(normalize_azimuth(magnetic_toolface_rad));
    }
    if inclination_rad >= high_inclination_rad {
        return Ok(normalize_azimuth(high_side_toolface_rad));
    }
    let fraction =
        (inclination_rad - low_inclination_rad) / (high_inclination_rad - low_inclination_rad);
    Ok(circular_lerp(
        magnetic_toolface_rad,
        high_side_toolface_rad,
        fraction,
    ))
}

/// Reconstructs axial Bz from total and transverse fields, preserving measured sign.
pub fn short_collar_corrected_bz(
    measured: MagneticField,
    total_field: f64,
) -> Result<f64, SurveyError> {
    if ![measured.bx, measured.by, measured.bz, total_field]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(SurveyError::NonFiniteInput);
    }
    let axial_squared = total_field.mul_add(
        total_field,
        -(measured.bx.mul_add(measured.bx, measured.by * measured.by)),
    );
    if axial_squared < -SINGULARITY_EPSILON {
        return Err(SurveyError::InvalidMagneticField);
    }
    Ok(if measured.bz < 0.0 { -1.0 } else { 1.0 } * axial_squared.max(0.0).sqrt())
}

/// Simply-supported beam survey-inclination correction. The end slope is
/// `wL^3/(24EI)` and its projection approaches zero at vertical and horizontal.
pub fn sag_correction_inclination(
    inclination_rad: f64,
    model: SagModel,
) -> Result<f64, SurveyError> {
    if !inclination_rad.is_finite() {
        return Err(SurveyError::NonFiniteInput);
    }
    if ![
        model.span_m,
        model.distributed_load_n_per_m,
        model.youngs_modulus_pa,
        model.second_moment_m4,
    ]
    .iter()
    .all(|value| value.is_finite() && *value > 0.0)
    {
        return Err(SurveyError::InvalidSagModel);
    }
    let end_slope = model.distributed_load_n_per_m * model.span_m.powi(3)
        / (24.0 * model.youngs_modulus_pa * model.second_moment_m4);
    Ok(end_slope.atan() * inclination_rad.sin() * inclination_rad.cos())
}

/// Converts true azimuth to grid azimuth using positive-east grid convergence.
pub fn convert_true_to_grid_azimuth(true_azimuth_rad: f64, convergence_rad: f64) -> f64 {
    normalize_azimuth(true_azimuth_rad - convergence_rad)
}

/// Builds a NEV-to-HLA direction-cosine matrix. H, L, A rows are right handed.
pub fn nev_to_hla_dcm(inclination_rad: f64, azimuth_true_rad: f64) -> Result<Matrix3, SurveyError> {
    if !inclination_rad.is_finite() || !azimuth_true_rad.is_finite() {
        return Err(SurveyError::NonFiniteInput);
    }
    let axial = Vector3::new(
        inclination_rad.sin() * azimuth_true_rad.cos(),
        inclination_rad.sin() * azimuth_true_rad.sin(),
        inclination_rad.cos(),
    );
    let down = Vector3::new(0.0, 0.0, 1.0);
    let high_side = down - scale(axial, down.dot(axial));
    if high_side.norm() < SINGULARITY_EPSILON {
        return Err(SurveyError::UndefinedHighSide);
    }
    let high_side = high_side.normalized()?;
    let lateral = axial.cross(high_side).normalized()?;
    Ok(Matrix3::new([
        [high_side.x, high_side.y, high_side.z],
        [lateral.x, lateral.y, lateral.z],
        [axial.x, axial.y, axial.z],
    ]))
}

/// Applies `C_hla = R_hla_from_nev * C_nev * R_hla_from_nev^T`.
pub fn transform_covariance_nev_to_hla(
    covariance_nev: Matrix3,
    inclination_rad: f64,
    azimuth_true_rad: f64,
) -> Result<Matrix3, SurveyError> {
    let dcm = nev_to_hla_dcm(inclination_rad, azimuth_true_rad)?;
    Ok(dcm.multiply(covariance_nev).multiply(dcm.transpose()))
}

pub fn root_sum_square(values: &[f64]) -> Result<f64, SurveyError> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(SurveyError::InvalidRootSumSquareInput);
    }
    Ok(values.iter().map(|value| value * value).sum::<f64>().sqrt())
}

/// Produces plan and section traces from validated cumulative survey stations.
/// `north_m`, `east_m`, and `tvd_m` must be the cumulative coordinates at each
/// station, never recomputed in the UI.
pub fn build_plan_section_plot(stations: &[SurveyPosition]) -> Result<PlotSpec, SurveyError> {
    if stations.iter().any(|station| !station.is_finite()) {
        return Err(SurveyError::NonFiniteInput);
    }
    Ok(PlotSpec {
        title: "Survey plan and section".to_owned(),
        traces: vec![
            PlotTrace {
                id: "plan".to_owned(),
                name: "Plan".to_owned(),
                layer: "survey".to_owned(),
                points: stations
                    .iter()
                    .map(|s| PlotPoint {
                        x: s.east_m.get(),
                        y: s.north_m.get(),
                        z: Some(s.tvd_m.get()),
                    })
                    .collect(),
            },
            PlotTrace {
                id: "section".to_owned(),
                name: "Vertical section".to_owned(),
                layer: "survey".to_owned(),
                points: stations
                    .iter()
                    .map(|s| PlotPoint {
                        x: s.md_m.get(),
                        y: s.tvd_m.get(),
                        z: None,
                    })
                    .collect(),
            },
        ],
        bands: Vec::new(),
        annotations: Vec::new(),
    })
}

/// Adapts validated cumulative stations into a renderer-neutral 3Dmk scene.
/// This function deliberately copies coordinates without interpolation or
/// recalculation: engineering geometry remains owned by the survey workflow.
pub fn build_survey_scene(stations: &[SurveyPosition]) -> Result<SceneDocumentV1, SurveyError> {
    if stations.iter().any(|station| !station.is_finite()) {
        return Err(SurveyError::NonFiniteInput);
    }

    let path = stations
        .iter()
        .map(|station| {
            ScenePoint::new(
                station.north_m.get(),
                station.east_m.get(),
                station.tvd_m.get(),
            )
        })
        .collect();
    let station_markers = stations
        .iter()
        .enumerate()
        .map(|(index, station)| SceneMarkerV1 {
            id: format!("station-{index}"),
            label: format!("MD {:.3} m", station.md_m.get()),
            point: ScenePoint::new(
                station.north_m.get(),
                station.east_m.get(),
                station.tvd_m.get(),
            ),
        })
        .collect();

    SceneDocumentV1::new(
        "survey-scene",
        "Survey trajectory",
        vec![
            SceneLayerV1::polyline("survey-path", "Survey path", path),
            SceneLayerV1::markers("survey-stations", "Survey stations", station_markers),
        ],
        SceneProvenanceV1::new("survey-position-adapter", "v1", "cpu"),
    )
    .map_err(SurveyError::from)
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyPosition {
    pub md_m: Metres,
    pub north_m: Metres,
    pub east_m: Metres,
    pub tvd_m: Metres,
}
impl SurveyPosition {
    fn is_finite(self) -> bool {
        true
    }
}

fn validate_station(station: SurveyStation) -> Result<(), SurveyError> {
    let _ = station;
    Ok(())
}
fn scale(vector: Vector3, scalar: f64) -> Vector3 {
    Vector3::new(vector.x * scalar, vector.y * scalar, vector.z * scalar)
}
fn normalize_azimuth(angle_rad: f64) -> f64 {
    angle_rad.rem_euclid(TAU)
}
fn circular_lerp(start_rad: f64, end_rad: f64, fraction: f64) -> f64 {
    let delta = (end_rad - start_rad + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI;
    normalize_azimuth(start_rad + fraction * delta)
}
