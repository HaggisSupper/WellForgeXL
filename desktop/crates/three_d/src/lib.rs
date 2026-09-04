//! 3Dmk owns WellForge's renderer-neutral scene contracts.
//!
//! The crate validates display geometry only. It never derives engineering
//! positions or performs drilling calculations.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SCENE_SCHEMA_VERSION_V1: &str = "wellforge.scene/v1";
pub const NE_TVD_METRES: &str = "north-east-tvd-m";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ScenePoint {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn is_finite(self) -> bool {
        [self.x, self.y, self.z]
            .iter()
            .all(|coordinate| coordinate.is_finite())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBounds {
    pub minimum: ScenePoint,
    pub maximum: ScenePoint,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneProvenanceV1 {
    pub algorithm: String,
    pub profile_version: String,
    pub backend: String,
    pub input_revision: Option<String>,
    pub warnings: Vec<String>,
}

impl SceneProvenanceV1 {
    pub fn new(
        algorithm: impl Into<String>,
        profile_version: impl Into<String>,
        backend: impl Into<String>,
    ) -> Self {
        Self {
            algorithm: algorithm.into(),
            profile_version: profile_version.into(),
            backend: backend.into(),
            input_revision: None,
            warnings: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScenePrimitiveV1 {
    Polyline {
        points: Vec<ScenePoint>,
    },
    Marker {
        id: String,
        label: String,
        point: ScenePoint,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneMarkerV1 {
    pub id: String,
    pub label: String,
    pub point: ScenePoint,
}

impl ScenePrimitiveV1 {
    pub fn points(&self) -> Vec<ScenePoint> {
        match self {
            Self::Polyline { points } => points.clone(),
            Self::Marker { point, .. } => vec![*point],
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneLayerV1 {
    pub id: String,
    pub name: String,
    pub visible_by_default: bool,
    pub selectable: bool,
    pub color: String,
    pub primitives: Vec<ScenePrimitiveV1>,
}

impl SceneLayerV1 {
    pub fn polyline(
        id: impl Into<String>,
        name: impl Into<String>,
        points: Vec<ScenePoint>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible_by_default: true,
            selectable: false,
            color: "#bfc5ca".to_owned(),
            primitives: vec![ScenePrimitiveV1::Polyline { points }],
        }
    }

    pub fn markers(
        id: impl Into<String>,
        name: impl Into<String>,
        markers: Vec<SceneMarkerV1>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible_by_default: true,
            selectable: false,
            color: "#34d399".to_owned(),
            primitives: markers
                .into_iter()
                .map(|marker| ScenePrimitiveV1::Marker {
                    id: marker.id,
                    label: marker.label,
                    point: marker.point,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneDocumentV1 {
    pub schema_version: String,
    pub scene_id: String,
    pub title: String,
    pub coordinate_frame: String,
    pub layers: Vec<SceneLayerV1>,
    pub bounds: SceneBounds,
    pub provenance: SceneProvenanceV1,
}

impl SceneDocumentV1 {
    pub fn new(
        scene_id: impl Into<String>,
        title: impl Into<String>,
        layers: Vec<SceneLayerV1>,
        provenance: SceneProvenanceV1,
    ) -> Result<Self, SceneError> {
        let scene_id = scene_id.into();
        let title = title.into();
        if scene_id.trim().is_empty() || title.trim().is_empty() {
            return Err(SceneError::InvalidSceneIdentity);
        }
        if layers.is_empty() {
            return Err(SceneError::EmptyScene);
        }
        if provenance.algorithm.trim().is_empty()
            || provenance.profile_version.trim().is_empty()
            || provenance.backend.trim().is_empty()
        {
            return Err(SceneError::InvalidProvenance);
        }

        let mut layer_ids = HashSet::new();
        let mut points = Vec::new();
        for layer in &layers {
            if layer.id.trim().is_empty() || layer.name.trim().is_empty() {
                return Err(SceneError::InvalidLayerIdentity);
            }
            if !layer_ids.insert(layer.id.as_str()) {
                return Err(SceneError::DuplicateLayerId);
            }
            if !is_hex_color(&layer.color) {
                return Err(SceneError::InvalidLayerColor);
            }
            for primitive in &layer.primitives {
                match primitive {
                    ScenePrimitiveV1::Polyline {
                        points: primitive_points,
                    } if primitive_points.is_empty() => return Err(SceneError::EmptyPrimitive),
                    ScenePrimitiveV1::Marker { id, label, .. }
                        if id.trim().is_empty() || label.trim().is_empty() =>
                    {
                        return Err(SceneError::InvalidMarkerIdentity);
                    }
                    _ => {}
                }
                points.extend(primitive.points());
            }
        }
        if points.is_empty() {
            return Err(SceneError::EmptyScene);
        }
        if points.iter().any(|point| !point.is_finite()) {
            return Err(SceneError::NonFiniteCoordinate);
        }

        Ok(Self {
            schema_version: SCENE_SCHEMA_VERSION_V1.to_owned(),
            scene_id,
            title,
            coordinate_frame: NE_TVD_METRES.to_owned(),
            layers,
            bounds: bounds(&points),
            provenance,
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SceneError {
    #[error("scene ID and title must not be blank")]
    InvalidSceneIdentity,
    #[error("scene must contain at least one point")]
    EmptyScene,
    #[error("scene provenance algorithm, profile version, and backend must not be blank")]
    InvalidProvenance,
    #[error("scene layer ID and name must not be blank")]
    InvalidLayerIdentity,
    #[error("scene layer IDs must be unique")]
    DuplicateLayerId,
    #[error("scene layer colors must be six-digit hexadecimal values")]
    InvalidLayerColor,
    #[error("scene polyline must contain at least one point")]
    EmptyPrimitive,
    #[error("scene marker ID and label must not be blank")]
    InvalidMarkerIdentity,
    #[error("scene coordinates must be finite")]
    NonFiniteCoordinate,
}

impl SceneError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidSceneIdentity => "INVALID_SCENE_IDENTITY",
            Self::EmptyScene => "EMPTY_SCENE",
            Self::InvalidProvenance => "INVALID_SCENE_PROVENANCE",
            Self::InvalidLayerIdentity => "INVALID_SCENE_LAYER_IDENTITY",
            Self::DuplicateLayerId => "DUPLICATE_SCENE_LAYER_ID",
            Self::InvalidLayerColor => "INVALID_SCENE_LAYER_COLOR",
            Self::EmptyPrimitive => "EMPTY_SCENE_PRIMITIVE",
            Self::InvalidMarkerIdentity => "INVALID_SCENE_MARKER_IDENTITY",
            Self::NonFiniteCoordinate => "NON_FINITE_SCENE_COORDINATE",
        }
    }
}

fn bounds(points: &[ScenePoint]) -> SceneBounds {
    let first = points[0];
    let (mut minimum, mut maximum) = (first, first);
    for point in points.iter().copied().skip(1) {
        minimum.x = minimum.x.min(point.x);
        minimum.y = minimum.y.min(point.y);
        minimum.z = minimum.z.min(point.z);
        maximum.x = maximum.x.max(point.x);
        maximum.y = maximum.y.max(point.y);
        maximum.z = maximum.z.max(point.z);
    }
    SceneBounds { minimum, maximum }
}

fn is_hex_color(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit())
}
