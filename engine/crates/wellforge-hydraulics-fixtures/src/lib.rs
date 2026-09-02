//! Deterministic fixtures for the hydraulics lane.

use uuid::Uuid;
use wellforge_hydraulics_contract::{
    HydraulicsAnalysisRequest, HydraulicsOperatingPoint, Nozzle, RheologyModel, RheologyParameters,
    StandardProfile, TubularSection,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

/// Fixed section UUID for the canonical case.
pub const CANONICAL_SECTION_ID: Uuid = Uuid::from_u128(0x35b1_5a48_47c1_4d31_a92e_7c5b_8f20_0701);
/// Fixed analysis UUID.
pub const CANONICAL_ANALYSIS_ID: Uuid = Uuid::from_u128(0x35b1_5a48_47c1_4d31_a92e_7c5b_8f20_0702);
/// Fixed source UUID.
pub const CANONICAL_SOURCE_ID: Uuid = Uuid::from_u128(0x35b1_5a48_47c1_4d31_a92e_7c5b_8f20_0703);

/// Build a canonical Bingham-plastic steady-state case.
///
/// - Single 3000 m section, 5-inch drill pipe in a 9-7/8 inch hole.
/// - Bingham mud: yield 5 Pa, plastic viscosity 0.020 Pa*s, density 1200 kg/m^3.
/// - Flow 0.03 m^3/s (~ 476 gpm) through three 12/32 inch nozzles.
#[must_use]
pub fn canonical_bingham_case() -> HydraulicsAnalysisRequest {
    let section = TubularSection {
        id: CANONICAL_SECTION_ID,
        name: "5in DP in 9-7/8in hole".to_string(),
        top_md_m: 0.0,
        bottom_md_m: 3000.0,
        string_od_m: 0.1270,
        string_id_m: 0.1086,
        hole_id_m: 0.2508,
    };

    HydraulicsAnalysisRequest {
        contract_version: "0.1.0".to_string(),
        analysis_id: CANONICAL_ANALYSIS_ID,
        profile: StandardProfile {
            standard: "API RP 13D".to_string(),
            edition: "7th Edition, 2017 (reaffirmed 2023)".to_string(),
        },
        sources: vec![SourceObjectRef {
            uuid: CANONICAL_SOURCE_ID,
            uri: None,
            object_type: WitsmlObjectType::Tubular,
            content_hash: "0".repeat(64),
            citation_name: "canonical_bingham_case".to_string(),
            source_system: "wellforge-hydraulics-fixtures".to_string(),
        }],
        rheology: RheologyParameters {
            model: RheologyModel::Bingham,
            dynamic_viscosity_pa_s: None,
            yield_stress_pa: Some(5.0),
            plastic_viscosity_pa_s: Some(0.020),
            consistency_k_pa_s_n: None,
            flow_behavior_index: None,
        },
        sections: vec![section],
        operating: HydraulicsOperatingPoint {
            mud_density_kg_m3: 1200.0,
            flow_rate_m3_s: 0.030,
            surface_temperature_k: 300.0,
            nozzles: vec![
                Nozzle { diameter_m: 12.0 / 32.0 * 0.0254 },
                Nozzle { diameter_m: 12.0 / 32.0 * 0.0254 },
                Nozzle { diameter_m: 12.0 / 32.0 * 0.0254 },
            ],
        },
    }
}
