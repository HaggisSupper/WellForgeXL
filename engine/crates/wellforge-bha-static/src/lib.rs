//! Static flexible-beam analysis for BHA Release 1.

use faer::prelude::*;
use nalgebra::DMatrix;
use thiserror::Error;
use wellforge_bha_contract::{BhaAnalysisRequest, StaticNodeResult};
use wellforge_bha_model::{BhaModel, projected_clearance};

/// Static calculation output and matrices reused by modal analysis.
#[derive(Clone, Debug)]
pub struct StaticSolution {
    /// Value-only centerline/projection output.
    pub nodes: Vec<StaticNodeResult>,
    /// Assembled reduced stiffness matrix.
    pub stiffness: DMatrix<f64>,
    /// Assembled reduced consistent mass matrix.
    pub mass: DMatrix<f64>,
    /// Reduced displacement vector.
    pub displacement: Vec<f64>,
    /// Linear-solve residual norm.
    pub residual_norm: f64,
}

/// Static solve failure.
#[derive(Debug, Error)]
pub enum StaticSolveError {
    /// Model has fewer than two nodes.
    #[error("model requires at least two nodes")]
    TooFewNodes,
    /// Matrix factorization did not produce finite results.
    #[error("static matrix solve failed")]
    LinearSolve,
}

/// Euler-Bernoulli cantilever tip deflection used as an analytical oracle.
#[must_use]
pub fn cantilever_tip_deflection(
    length_m: f64,
    youngs_modulus_pa: f64,
    inertia_m4: f64,
    force_n: f64,
) -> f64 {
    force_n * length_m.powi(3) / (3.0 * youngs_modulus_pa * inertia_m4)
}

fn element_matrices(
    length: f64,
    flexural_rigidity: f64,
    mass_per_length: f64,
) -> ([[f64; 4]; 4], [[f64; 4]; 4]) {
    let l2 = length * length;
    let l3 = l2 * length;
    let k0 = flexural_rigidity / l3;
    let k = [
        [12.0 * k0, 6.0 * length * k0, -12.0 * k0, 6.0 * length * k0],
        [
            6.0 * length * k0,
            4.0 * l2 * k0,
            -6.0 * length * k0,
            2.0 * l2 * k0,
        ],
        [
            -12.0 * k0,
            -6.0 * length * k0,
            12.0 * k0,
            -6.0 * length * k0,
        ],
        [
            6.0 * length * k0,
            2.0 * l2 * k0,
            -6.0 * length * k0,
            4.0 * l2 * k0,
        ],
    ];
    let m0 = mass_per_length * length / 420.0;
    let m = [
        [
            156.0 * m0,
            22.0 * length * m0,
            54.0 * m0,
            -13.0 * length * m0,
        ],
        [
            22.0 * length * m0,
            4.0 * l2 * m0,
            13.0 * length * m0,
            -3.0 * l2 * m0,
        ],
        [
            54.0 * m0,
            13.0 * length * m0,
            156.0 * m0,
            -22.0 * length * m0,
        ],
        [
            -13.0 * length * m0,
            -3.0 * l2 * m0,
            -22.0 * length * m0,
            4.0 * l2 * m0,
        ],
    ];
    (k, m)
}

fn geometric_stiffness(length: f64, compression_n: f64) -> [[f64; 4]; 4] {
    let scale = compression_n / (30.0 * length);
    let l2 = length * length;
    [
        [
            36.0 * scale,
            3.0 * length * scale,
            -36.0 * scale,
            3.0 * length * scale,
        ],
        [
            3.0 * length * scale,
            4.0 * l2 * scale,
            -3.0 * length * scale,
            -l2 * scale,
        ],
        [
            -36.0 * scale,
            -3.0 * length * scale,
            36.0 * scale,
            -3.0 * length * scale,
        ],
        [
            3.0 * length * scale,
            -l2 * scale,
            -3.0 * length * scale,
            4.0 * l2 * scale,
        ],
    ]
}

/// Solves a small-deflection, buoyed-weight static beam and calculates OD/hole projection indication.
///
/// # Errors
///
/// Returns [`StaticSolveError`] when the mesh is too small or the library linear solve is non-finite.
#[allow(clippy::too_many_lines)]
pub fn solve_static(
    model: &BhaModel,
    request: &BhaAnalysisRequest,
) -> Result<StaticSolution, StaticSolveError> {
    if model.nodes.len() < 2 {
        return Err(StaticSolveError::TooFewNodes);
    }
    let full_dofs = model.nodes.len() * 2;
    let mut k_full = DMatrix::<f64>::zeros(full_dofs, full_dofs);
    let mut m_full = DMatrix::<f64>::zeros(full_dofs, full_dofs);
    let mut f_full = vec![0.0; full_dofs];
    for element in 0..model.nodes.len() - 1 {
        let first = &model.nodes[element];
        let second = &model.nodes[element + 1];
        let length = second.md_m - first.md_m;
        let od = first.od_m.midpoint(second.od_m);
        let id = first.id_m.midpoint(second.id_m);
        let area = std::f64::consts::PI * (od.powi(2) - id.powi(2)) / 4.0;
        let inertia = std::f64::consts::PI * (od.powi(4) - id.powi(4)) / 64.0;
        let density = first.density_kg_m3.midpoint(second.density_kg_m3);
        let young = first.youngs_modulus_pa.midpoint(second.youngs_modulus_pa);
        let (ke, me) = element_matrices(length, young * inertia, density * area);
        let kg = geometric_stiffness(length, request.operating.wob_n);
        let map = [
            2 * element,
            2 * element + 1,
            2 * element + 2,
            2 * element + 3,
        ];
        for row in 0..4 {
            for col in 0..4 {
                k_full[(map[row], map[col])] += ke[row][col] - kg[row][col];
                m_full[(map[row], map[col])] += me[row][col];
            }
        }
        let buoyed_mass_per_length =
            (density - request.operating.fluid_density_kg_m3).max(0.0) * area;
        let mid_md = first.md_m.midpoint(second.md_m);
        let inclination = request
            .trajectory
            .iter()
            .min_by(|left, right| {
                (left.md_m - mid_md)
                    .abs()
                    .total_cmp(&(right.md_m - mid_md).abs())
            })
            .map_or(0.0, |station| station.inclination_rad);
        let load = buoyed_mass_per_length * 9.80665 * inclination.sin().abs();
        f_full[2 * element] += load * length / 2.0;
        f_full[2 * element + 2] += load * length / 2.0;
    }
    let reduced = full_dofs - 2;
    let stiffness = k_full.view((2, 2), (reduced, reduced)).into_owned();
    let mass = m_full.view((2, 2), (reduced, reduced)).into_owned();
    let rhs_values = &f_full[2..];
    let k_faer = faer::Mat::from_fn(reduced, reduced, |row, col| stiffness[(row, col)]);
    let rhs = faer::Mat::from_fn(reduced, 1, |row, _| rhs_values[row]);
    let solved = k_faer.partial_piv_lu().solve(&rhs);
    let displacement: Vec<f64> = (0..reduced).map(|row| solved[(row, 0)]).collect();
    if displacement.iter().any(|value| !value.is_finite()) {
        return Err(StaticSolveError::LinearSolve);
    }
    let residual_norm = (&k_faer * &solved - &rhs).norm_l2() / rhs.norm_l2().max(1.0);
    let mut full_displacement = vec![0.0; full_dofs];
    full_displacement[2..].copy_from_slice(&displacement);
    let mut nodes = Vec::with_capacity(model.nodes.len());
    for (index, node) in model.nodes.iter().enumerate() {
        let x = full_displacement[2 * index];
        let moment = if index == 0 || index + 1 == model.nodes.len() {
            0.0
        } else {
            let h1 = node.md_m - model.nodes[index - 1].md_m;
            let h2 = model.nodes[index + 1].md_m - node.md_m;
            let curvature = 2.0
                * ((full_displacement[2 * (index + 1)] - x) / h2
                    - (x - full_displacement[2 * (index - 1)]) / h1)
                / (h1 + h2);
            let inertia = std::f64::consts::PI * (node.od_m.powi(4) - node.id_m.powi(4)) / 64.0;
            node.youngs_modulus_pa * inertia * curvature.abs()
        };
        let inertia = std::f64::consts::PI * (node.od_m.powi(4) - node.id_m.powi(4)) / 64.0;
        let stress = if inertia > 0.0 {
            moment * (node.od_m / 2.0) / inertia
        } else {
            0.0
        };
        nodes.push(StaticNodeResult {
            md_m: node.md_m,
            x_m: x,
            y_m: 0.0,
            od_radius_m: node.od_m / 2.0,
            id_radius_m: node.id_m / 2.0,
            hole_radius_m: node.hole_radius_m,
            projected_clearance_m: projected_clearance(node.hole_radius_m, node.od_m / 2.0, x, 0.0),
            bending_moment_n_m: moment,
            bending_stress_pa: stress,
        });
    }
    Ok(StaticSolution {
        nodes,
        stiffness,
        mass,
        displacement,
        residual_norm,
    })
}
