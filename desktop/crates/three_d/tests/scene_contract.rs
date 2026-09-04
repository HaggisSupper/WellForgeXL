use wellforge_3d::{
    SceneDocumentV1, SceneLayerV1, ScenePoint, ScenePrimitiveV1, SceneProvenanceV1,
};

fn provenance() -> SceneProvenanceV1 {
    SceneProvenanceV1::new("survey-position-adapter", "v1", "cpu")
}

fn polyline(points: Vec<ScenePoint>) -> SceneLayerV1 {
    SceneLayerV1::polyline("survey-path", "Survey path", points)
}

#[test]
fn scene_rejects_a_non_finite_vertex() {
    let error = SceneDocumentV1::new(
        "well-a",
        "Well A",
        vec![polyline(vec![ScenePoint::new(f64::NAN, 0.0, 0.0)])],
        provenance(),
    )
    .expect_err("scene coordinates must be finite");

    assert_eq!(error.code(), "NON_FINITE_SCENE_COORDINATE");
}

#[test]
fn scene_calculates_ne_tvd_bounds_in_rust() {
    let scene = SceneDocumentV1::new(
        "well-a",
        "Well A",
        vec![polyline(vec![
            ScenePoint::new(2.0, 3.0, 4.0),
            ScenePoint::new(-1.0, 7.0, 6.0),
        ])],
        provenance(),
    )
    .expect("finite unique scene is valid");

    assert_eq!(scene.schema_version, "wellforge.scene/v1");
    assert_eq!(scene.bounds.minimum, ScenePoint::new(-1.0, 3.0, 4.0));
    assert_eq!(scene.bounds.maximum, ScenePoint::new(2.0, 7.0, 6.0));
    assert!(matches!(
        scene.layers[0].primitives[0],
        ScenePrimitiveV1::Polyline { .. }
    ));
}

#[test]
fn scene_rejects_duplicate_layer_identifiers() {
    let error = SceneDocumentV1::new(
        "well-a",
        "Well A",
        vec![
            polyline(vec![ScenePoint::new(0.0, 0.0, 0.0)]),
            polyline(vec![ScenePoint::new(1.0, 1.0, 1.0)]),
        ],
        provenance(),
    )
    .expect_err("layer IDs must be unique");

    assert_eq!(error.code(), "DUPLICATE_SCENE_LAYER_ID");
}
