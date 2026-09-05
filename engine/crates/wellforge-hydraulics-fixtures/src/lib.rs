//! Deterministic fixtures for the hydraulics lane.

use uuid::Uuid;
use wellforge_hydraulics_contract::{
    ComputeBackend, FlowCorrelation, HydraulicsAnalysisRequest, HydraulicsOperatingPoint,
    HydraulicsSolverOptions, Nozzle, RheologyModel, RheologyParameters, StandardProfile,
    ThermalAssumption, TubularSection,
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
        top_tvd_m: None,
        bottom_tvd_m: None,
        active_flow_loop: None,
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
            content_hash: format!("sha256:{}", "0".repeat(64)),
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
            high_shear_flow_index: None,
        },
        solver: None,
        sections: vec![section],
        operating: HydraulicsOperatingPoint {
            mud_density_kg_m3: 1200.0,
            flow_rate_m3_s: 0.030,
            surface_temperature_k: 300.0,
            nozzle_discharge_coefficient: None,
            surface_backpressure_pa: None,
            ecd_reference_tvd_m: None,
            nozzles: vec![
                Nozzle {
                    diameter_m: 12.0 / 32.0 * 0.0254,
                },
                Nozzle {
                    diameter_m: 12.0 / 32.0 * 0.0254,
                },
                Nozzle {
                    diameter_m: 12.0 / 32.0 * 0.0254,
                },
            ],
        },
    }
}

/// Build the generalized SI-native case used to exercise the current correlation lane.
#[must_use]
pub fn generalized_yield_power_law_case() -> HydraulicsAnalysisRequest {
    let mut request = canonical_bingham_case();
    request.contract_version = "0.2.0".to_string();
    request.rheology = RheologyParameters {
        model: RheologyModel::HerschelBulkley,
        dynamic_viscosity_pa_s: None,
        yield_stress_pa: Some(5.0),
        plastic_viscosity_pa_s: None,
        consistency_k_pa_s_n: Some(0.012),
        flow_behavior_index: Some(0.70),
        high_shear_flow_index: Some(0.70),
    };
    request.solver = Some(HydraulicsSolverOptions {
        flow_correlation: FlowCorrelation::GeneralizedYieldPowerLaw,
        compute_backend: ComputeBackend::ParallelCpu,
        thermal_assumption: ThermalAssumption::ConstantProperties,
    });
    request.operating.nozzle_discharge_coefficient = Some(0.95);
    request.operating.surface_backpressure_pa = Some(0.0);
    request.operating.ecd_reference_tvd_m = Some(3000.0);
    request
}
