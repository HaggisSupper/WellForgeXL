//! Discretized BHA beam model and projected hole geometry.

use nalgebra::{Isometry3, UnitQuaternion, Vector3};
use parry3d_f64::{
    math::{Pose, Vector},
    query::PointQuery,
    shape::Cylinder,
};
use petgraph::{
    algo::is_cyclic_directed,
    graph::{DiGraph, NodeIndex},
};
use thiserror::Error;
use wellforge_bha_contract::{BhaAnalysisRequest, validate_request};

/// One finite-element centerline node and its section properties.
#[derive(Clone, Debug)]
pub struct ModelNode {
    /// Node MD in metres.
    pub md_m: f64,
    /// Undeformed local-to-global pose.
    pub frame: Isometry3<f64>,
    /// OD in metres.
    pub od_m: f64,
    /// ID in metres.
    pub id_m: f64,
    /// Hole radius in metres.
    pub hole_radius_m: f64,
    /// Centered radial clearance in metres.
    pub radial_clearance_m: f64,
    /// Young's modulus in pascals.
    pub youngs_modulus_pa: f64,
    /// Material density in kilograms per cubic metre.
    pub density_kg_m3: f64,
}

/// Ordered discretized BHA model.
#[derive(Clone, Debug)]
pub struct BhaModel {
    /// Centerline nodes.
    pub nodes: Vec<ModelNode>,
    /// Total dry component mass.
    pub total_mass_kg: f64,
    /// Ordered mechanical component graph from top boundary to bit end.
    pub component_graph: DiGraph<uuid::Uuid, ()>,
    /// Graph node order corresponding to the request component path.
    pub component_path: Vec<NodeIndex>,
}

/// Model assembly failure.
#[derive(Debug, Error)]
pub enum ModelError {
    /// Contract validation failed.
    #[error("invalid request: {0}")]
    InvalidContract(String),
    /// No containing hole section was found.
    #[error("no hole geometry at MD {0}")]
    MissingHole(f64),
    /// Component graph is not a single acyclic path.
    #[error("component graph is not a single ordered path")]
    InvalidGraph,
}

/// Uses `parry3d-f64` cylinder projection to calculate indicated radial clearance.
#[must_use]
pub fn projected_clearance(hole_radius_m: f64, od_radius_m: f64, x_m: f64, y_m: f64) -> f64 {
    let cylinder = Cylinder::new(1.0, hole_radius_m);
    let point = Vector::new(x_m, 0.0, y_m);
    let projection = cylinder.project_point(&Pose::identity(), point, false);
    let distance_to_wall = (projection.point - point).length();
    let inside = x_m.hypot(y_m) <= hole_radius_m;
    if inside {
        distance_to_wall - od_radius_m
    } else {
        -distance_to_wall - od_radius_m
    }
}

/// Validates and discretizes ordered components without changing source identity.
///
/// # Errors
///
/// Returns [`ModelError`] when the request is invalid or its hole geometry does not cover a node.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
pub fn assemble_model(request: &BhaAnalysisRequest) -> Result<BhaModel, ModelError> {
    validate_request(request).map_err(|errors| {
        ModelError::InvalidContract(
            errors
                .into_iter()
                .map(|error| error.code)
                .collect::<Vec<_>>()
                .join(","),
        )
    })?;
    let mut nodes = Vec::new();
    let mut total_mass_kg = 0.0;
    let mut component_graph = DiGraph::new();
    let mut component_path = Vec::with_capacity(request.components.len());
    for component in &request.components {
        let node = component_graph.add_node(component.id);
        if let Some(previous) = component_path.last().copied() {
            component_graph.add_edge(previous, node, ());
        }
        component_path.push(node);
    }
    if is_cyclic_directed(&component_graph)
        || component_graph.edge_count() + 1 != component_graph.node_count()
    {
        return Err(ModelError::InvalidGraph);
    }
    for (component_index, component) in request.components.iter().enumerate() {
        let length = component.bottom_md_m - component.top_md_m;
        let element_count = (length / request.solver.max_element_length_m)
            .ceil()
            .max(1.0) as usize;
        let area = std::f64::consts::PI * (component.od_m.powi(2) - component.id_m.powi(2)) / 4.0;
        total_mass_kg += area * length * component.density_kg_m3;
        for index in 0..=element_count {
            if component_index > 0 && index == 0 {
                continue;
            }
            let fraction = index as f64 / element_count as f64;
            let md_m = component.top_md_m + fraction * length;
            let hole = request
                .hole
                .iter()
                .find(|section| {
                    md_m >= section.top_md_m - 1.0e-9 && md_m <= section.bottom_md_m + 1.0e-9
                })
                .ok_or(ModelError::MissingHole(md_m))?;
            let hole_radius_m = hole.diameter_m / 2.0;
            nodes.push(ModelNode {
                md_m,
                frame: Isometry3::from_parts(
                    Vector3::new(0.0, 0.0, md_m).into(),
                    UnitQuaternion::identity(),
                ),
                od_m: component.od_m,
                id_m: component.id_m,
                hole_radius_m,
                radial_clearance_m: projected_clearance(
                    hole_radius_m,
                    component.od_m / 2.0,
                    0.0,
                    0.0,
                ),
                youngs_modulus_pa: component.youngs_modulus_pa,
                density_kg_m3: component.density_kg_m3,
            });
        }
    }
    Ok(BhaModel {
        nodes,
        total_mass_kg,
        component_graph,
        component_path,
    })
}
