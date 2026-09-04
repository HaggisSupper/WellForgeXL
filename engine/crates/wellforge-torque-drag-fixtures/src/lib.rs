//! Deterministic fixtures for the torque-and-drag lane.

use uuid::Uuid;
use wellforge_torque_drag_contract::{
    Api7gPipeSpec, OperationState, StringComponent, TnDAnalysisRequest, TnDOperatingPoint,
    TnDTrajectoryStation,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

/// Fixed component UUID for the single drill-pipe section in the canonical case.
pub const CANONICAL_DP_ID: Uuid = Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0601);

/// Fixed analysis UUID.
pub const CANONICAL_ANALYSIS_ID: Uuid = Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0602);

/// Fixed source object UUID.
pub const CANONICAL_SOURCE_ID: Uuid = Uuid::from_u128(0x2b7f_9d1c_44a1_4d31_a92e_7c5b_8f20_0603);

/// Build the canonical inclined-drill-pipe pickup case in canonical SI.
///
/// - Single 5-inch S135 drill-pipe section, 3000 m long.
/// - Constant inclination of 30 degrees along the whole string.
/// - Pickup with open-hole friction factor 0.30, mud density 1200 kg/m^3.
#[must_use]
pub fn canonical_pickup_case() -> TnDAnalysisRequest {
    let component = StringComponent {
        id: CANONICAL_DP_ID,
        name: "5in S135 drill pipe".to_string(),
        top_md_m: 0.0,
        bottom_md_m: 3000.0,
        od_m: 0.1270,
        id_m: 0.1086,
        linear_weight_kg_m: 29.79,
        youngs_modulus_pa: 2.07e11,
        density_kg_m3: 7850.0,
        api7g_spec: Some(Api7gPipeSpec {
            grade: "S135".to_string(),
            tensile_yield_pa: 9.31e8,
            torsional_yield_nm: 5.9e4,
            wear_class_derating: 0.80,
            safety_factor: 1.10,
        }),
    };

    let stations = (0..=30)
        .map(|i| {
            let md = 100.0 * f64::from(i);
            TnDTrajectoryStation {
                md_m: md,
                inclination_rad: 30.0_f64.to_radians(),
                azimuth_rad: 0.0,
                tvd_m: md * 30.0_f64.to_radians().cos(),
                cased: md < 500.0,
            }
        })
        .collect();

    TnDAnalysisRequest {
        contract_version: "0.1.0".to_string(),
        analysis_id: CANONICAL_ANALYSIS_ID,
        sources: vec![SourceObjectRef {
            uuid: CANONICAL_SOURCE_ID,
            uri: None,
            object_type: WitsmlObjectType::Tubular,
            content_hash: format!("sha256:{}", "0".repeat(64)),
            citation_name: "canonical_pickup_case".to_string(),
            source_system: "wellforge-torque-drag-fixtures".to_string(),
        }],
        components: vec![component],
        trajectory: stations,
        operating: TnDOperatingPoint {
            state: OperationState::Pickup,
            weight_on_bit_n: 0.0,
            torque_on_bit_nm: 0.0,
            surface_rpm_rad_s: 0.0,
            friction_factor_open_hole: 0.30,
            friction_factor_cased_hole: 0.20,
            mud_density_kg_m3: 1200.0,
        },
    }
}
