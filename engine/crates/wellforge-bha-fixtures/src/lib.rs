//! Deterministic fixture builders shared by engine tests.

use uuid::Uuid;
use wellforge_bha_contract::{
    BhaAnalysisRequest, BhaComponent, ComponentRepresentation, HoleSection, OperatingPoint,
    SolverSettings, TrajectoryStation,
};
use wellforge_witsml::{SourceObjectRef, WitsmlObjectType};

/// Returns the deterministic WITSML source set used by analytical fixtures.
#[must_use]
pub fn minimal_source_set() -> Vec<SourceObjectRef> {
    [
        WitsmlObjectType::Well,
        WitsmlObjectType::Wellbore,
        WitsmlObjectType::Trajectory,
        WitsmlObjectType::WellboreGeometry,
        WitsmlObjectType::Tubular,
        WitsmlObjectType::BhaRun,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, object_type)| {
        let seed = u128::try_from(index + 1).unwrap_or(1);
        SourceObjectRef {
            uuid: Uuid::from_u128(seed),
            uri: Some(format!("eml:///wellforge/fixture/{seed}")),
            object_type,
            content_hash: format!("sha256:{seed:064x}"),
            citation_name: format!("synthetic-{object_type:?}"),
            source_system: "wellforge-synthetic-1".to_owned(),
        }
    })
    .collect()
}

/// Returns a deterministic straight-hole BHA fixture with a flexible collar.
#[must_use]
pub fn minimal_request() -> BhaAnalysisRequest {
    BhaAnalysisRequest {
        contract_version: "1.0.0".to_owned(),
        analysis_id: Uuid::from_u128(100),
        sources: minimal_source_set(),
        trajectory: vec![
            TrajectoryStation {
                md_m: 0.0,
                inclination_rad: std::f64::consts::FRAC_PI_2,
                azimuth_rad: 0.0,
            },
            TrajectoryStation {
                md_m: 12.0,
                inclination_rad: std::f64::consts::FRAC_PI_2,
                azimuth_rad: 0.0,
            },
        ],
        components: vec![BhaComponent {
            id: Uuid::from_u128(200),
            name: "Synthetic drill collar".to_owned(),
            representation: ComponentRepresentation::Beam,
            top_md_m: 0.0,
            bottom_md_m: 12.0,
            od_m: 0.2032,
            id_m: 0.0714,
            youngs_modulus_pa: 206.84e9,
            density_kg_m3: 7850.0,
        }],
        hole: vec![HoleSection {
            top_md_m: 0.0,
            bottom_md_m: 12.0,
            diameter_m: 0.31115,
        }],
        operating: OperatingPoint {
            wob_n: 0.0,
            rpm: 120.0,
            fluid_density_kg_m3: 1200.0,
        },
        solver: SolverSettings {
            max_element_length_m: 0.5,
            requested_modes: 6,
            ..SolverSettings::default()
        },
    }
}

/// Returns a request missing one source type.
#[must_use]
pub fn minimal_request_without(object_type: WitsmlObjectType) -> BhaAnalysisRequest {
    let mut request = minimal_request();
    request
        .sources
        .retain(|source| source.object_type != object_type);
    request
}
