//! Calculation-authoritative hydraulics request types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wellforge_witsml::SourceObjectRef;

/// Rheology model family selectable within a given `StandardProfile`.
///
/// Reference basis: `Hydraulics Models\` and API RP 13D 7th Ed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RheologyModel {
    /// Newtonian.
    Newtonian,
    /// Bingham plastic (yield stress + plastic viscosity).
    Bingham,
    /// Power law (K, n).
    PowerLaw,
    /// Herschel-Bulkley (`tau_y + K * gamma^n`).
    HerschelBulkley,
}

/// Which industry standard profile authored the accepted rheology and friction relations.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StandardProfile {
    /// Standard citation, e.g. "API RP 13D".
    pub standard: String,
    /// Edition citation, e.g. "7th Edition, 2017 (reaffirmed 2023)".
    pub edition: String,
}

/// Rheology parameter set. Unused fields for the selected model must be `None`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RheologyParameters {
    /// Selected model.
    pub model: RheologyModel,
    /// Newtonian dynamic viscosity in Pa*s (Newtonian only).
    pub dynamic_viscosity_pa_s: Option<f64>,
    /// Bingham yield stress in Pa (Bingham, Herschel-Bulkley).
    pub yield_stress_pa: Option<f64>,
    /// Bingham plastic viscosity in Pa*s (Bingham).
    pub plastic_viscosity_pa_s: Option<f64>,
    /// Power-law consistency K in Pa*s^n (Power law, Herschel-Bulkley).
    pub consistency_k_pa_s_n: Option<f64>,
    /// Power-law flow behaviour index n (Power law, Herschel-Bulkley).
    pub flow_behavior_index: Option<f64>,
}

/// One string/annular tubular section.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TubularSection {
    /// Stable section identity.
    pub id: Uuid,
    /// Citation name.
    pub name: String,
    /// Top MD in metres.
    pub top_md_m: f64,
    /// Bottom MD in metres.
    pub bottom_md_m: f64,
    /// String outside diameter in metres.
    pub string_od_m: f64,
    /// String inside diameter in metres.
    pub string_id_m: f64,
    /// Hole/casing inside diameter in metres (annulus outer boundary).
    pub hole_id_m: f64,
}

/// Bit nozzle.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Nozzle {
    /// Nozzle diameter in metres (32nds converted upstream).
    pub diameter_m: f64,
}

/// Operating point.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsOperatingPoint {
    /// Mud density at surface in kilograms per cubic metre.
    pub mud_density_kg_m3: f64,
    /// Flow rate in cubic metres per second.
    pub flow_rate_m3_s: f64,
    /// Surface temperature in kelvin.
    pub surface_temperature_k: f64,
    /// Bit nozzles.
    pub nozzles: Vec<Nozzle>,
}

/// Flow-loop selector: `Pipe` for the drill string interior; `Annulus` for the annulus.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowLoop {
    /// Drill-string interior.
    Pipe,
    /// Annulus between string and hole/casing.
    Annulus,
}

/// Top-level hydraulics analysis request.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydraulicsAnalysisRequest {
    /// Semver contract version this request conforms to.
    pub contract_version: String,
    /// Stable analysis identity.
    pub analysis_id: Uuid,
    /// Industry standard profile identifier.
    pub profile: StandardProfile,
    /// WITSML source object references for provenance.
    pub sources: Vec<SourceObjectRef>,
    /// Rheology parameters.
    pub rheology: RheologyParameters,
    /// Ordered tubular sections from surface to bit.
    pub sections: Vec<TubularSection>,
    /// Operating point.
    pub operating: HydraulicsOperatingPoint,
}
