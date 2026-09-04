use serde::Deserialize;
use wellforge_3d::{
    SceneDocumentV1, SceneError, SceneLayerV1, SceneMarkerV1, ScenePoint, SceneProvenanceV1,
};

#[derive(Debug, Deserialize)]
pub(crate) struct CanonicalTrajectoryResult {
    calculation: CanonicalCalculation,
    evidence: CanonicalEvidence,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CanonicalCalculation {
    plan: Vec<CanonicalStation>,
    survey: Vec<CanonicalStation>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CanonicalStation {
    north_m: f64,
    east_m: f64,
    tvd_m: f64,
    md_m: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CanonicalEvidence {
    result_hash: String,
}

pub(crate) fn build_trajectory_scene(
    result: &CanonicalTrajectoryResult,
) -> Result<SceneDocumentV1, SceneError> {
    let plan_points = result.calculation.plan.iter().map(scene_point).collect();
    let survey_points = result.calculation.survey.iter().map(scene_point).collect();
    let station_markers = result
        .calculation
        .survey
        .iter()
        .enumerate()
        .map(|(index, station)| SceneMarkerV1 {
            id: format!("survey-station-{index}"),
            label: format!("MD {:.3} m", station.md_m),
            point: scene_point(station),
        })
        .collect();

    let mut provenance = SceneProvenanceV1::new("canonical-trajectory-adapter", "v1", "cpu");
    provenance.input_revision = Some(result.evidence.result_hash.clone());

    SceneDocumentV1::new(
        "trajectory-scene",
        "Canonical trajectory",
        vec![
            SceneLayerV1::polyline("plan-path", "Plan path", plan_points),
            SceneLayerV1::polyline("survey-path", "Survey path", survey_points),
            SceneLayerV1::markers("survey-stations", "Survey stations", station_markers),
        ],
        provenance,
    )
}

fn scene_point(station: &CanonicalStation) -> ScenePoint {
    ScenePoint::new(station.north_m, station.east_m, station.tvd_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_plan_and_survey_stations_become_ordered_3dmk_layers() {
        let result: CanonicalTrajectoryResult = serde_json::from_str(include_str!(
            "../../../engine/fixtures/expected/trajectory-release-one-minimal.result.json"
        ))
        .expect("canonical trajectory fixture parses");

        let scene = build_trajectory_scene(&result).expect("trajectory scene builds");

        assert_eq!(scene.layers[0].id, "plan-path");
        assert_eq!(scene.layers[1].id, "survey-path");
        assert_eq!(
            scene.layers[0].primitives[0].points()[0].x,
            result.calculation.plan[0].north_m
        );
        assert_eq!(
            scene.layers[1].primitives[0].points()[0].y,
            result.calculation.survey[0].east_m
        );
        assert_eq!(
            scene.provenance.input_revision.as_deref(),
            Some(result.evidence.result_hash.as_str())
        );
    }
}
