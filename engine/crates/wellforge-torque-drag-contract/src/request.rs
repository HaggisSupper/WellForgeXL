//! Calculation-authoritative torque-and-drag request types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::SourceObjectRef;

/// Soft-string operation state that selects sign conventions for friction.
///
/// Reference basis: `Torque and Drag\` (soft-string operational states).
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    /// Pickup: upward motion, drag opposes uphole travel.
    Pickup,
    /// Slack-off: downward motion, drag opposes downhole travel.
    SlackOff,
    /// Rotating off-bottom: torque only, minimal axial friction.
    RotatingOffBottom,
    /// Drilling: rotation with weight on bit and downward motion.
    Drilling,
    /// Sliding: no rotation, downward motion with maximum axial friction.
    Sliding,
    /// Backreaming: rotation with upward motion.
    Backreaming,
}

/// One tubular string section in canonical SI, with API 7G spec.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StringComponent {
    /// Stable component identity.
    pub id: Uuid,
    /// Citation name (drill pipe grade, HWDP, drill collar, etc.).
    pub name: String,
    /// Section top measured depth in metres.
    pub top_md_m: f64,
    /// Section bottom measured depth in metres.
    pub bottom_md_m: f64,
    /// Outside diameter in metres.
    pub od_m: f64,
    /// Inside diameter in metres.
    pub id_m: f64,
    /// Linear weight in air in kilograms per metre.
    pub linear_weight_kg_m: f64,
    /// Young's modulus in pascals.
    pub youngs_modulus_pa: f64,
    /// Material density in kilograms per cubic metre.
    pub density_kg_m3: f64,
    /// API 7G derated strength envelope, or `None` for non-drill-pipe sections.
    pub api7g_spec: Option<Api7gPipeSpec>,
}

/// API 7G pipe strength envelope in canonical SI.
///
/// Reference basis: archive `^Technical Reference Tools/Industry Specifications/API 7G 2009.pdf`
/// (values authored independently; PDF used only as convention reference).
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Api7gPipeSpec {
    /// Pipe grade citation (E75, X95, G105, S135, Z140, V150).
    pub grade: String,
    /// New-pipe tensile yield strength in pascals.
    pub tensile_yield_pa: f64,
    /// New-pipe torsional yield strength in newton-metres.
    pub torsional_yield_nm: f64,
    /// Wear-class multiplier applied to new-pipe strengths (0.0..=1.0).
    pub wear_class_derating: f64,
    /// Safety factor applied to derated envelope (>= 1.0).
    pub safety_factor: f64,
}

/// WITSML-aligned trajectory station used to orient gravity and compute dogleg.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TnDTrajectoryStation {
    /// Measured depth in metres.
    pub md_m: f64,
    /// Inclination in radians.
    pub inclination_rad: f64,
    /// Azimuth in radians.
    pub azimuth_rad: f64,
    /// True vertical depth in metres.
    pub tvd_m: f64,
    /// Flag: true when this section is cased (uses cased-hole friction factor).
    pub cased: bool,
}

/// Operating point that selects the state and provides boundary loads.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TnDOperatingPoint {
    /// Selected operation state.
    pub state: OperationState,
    /// Weight on bit at the string bottom in newtons (positive downhole).
    pub weight_on_bit_n: f64,
    /// Torque at bit in newton-metres.
    pub torque_on_bit_nm: f64,
    /// Rotary speed at surface in radians per second (0 for pure sliding).
    pub surface_rpm_rad_s: f64,
    /// Open-hole friction factor (dimensionless).
    pub friction_factor_open_hole: f64,
    /// Cased-hole friction factor (dimensionless).
    pub friction_factor_cased_hole: f64,
    /// Mud density in kilograms per cubic metre for buoyancy.
    pub mud_density_kg_m3: f64,
}

/// Top-level torque-and-drag analysis request.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TnDAnalysisRequest {
    /// Semver contract version this request conforms to.
    pub contract_version: String,
    /// Stable analysis identity.
    pub analysis_id: Uuid,
    /// WITSML source object references for provenance.
    pub sources: Vec<SourceObjectRef>,
    /// Ordered string components from top to bit.
    pub components: Vec<StringComponent>,
    /// Trajectory stations covering the full modelled string.
    pub trajectory: Vec<TnDTrajectoryStation>,
    /// Operating point.
    pub operating: TnDOperatingPoint,
}
