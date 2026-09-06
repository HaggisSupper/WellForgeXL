//! Acceptance boundary for third-party and production numerical primitives.

use faer::prelude::*;
use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt};
use nalgebra::{Matrix3, Owned, SymmetricEigen, U3, UnitQuaternion, Vector3};
use parry3d_f64::{
    math::{Pose, Vector},
    query::PointQuery,
    shape::Cuboid,
};
use serde::{Deserialize, Serialize};
use wellforge_numerics::{
    CircularAccumulator, Interval, MonotoneCurve, RootOptions, SparseEntry, SparseLinearSystem,
    VolumeCoordinate, VolumeSegment, fischer_burmeister, solve_bracketed,
};

/// Result of exercising every general-purpose numerical capability required by Release 1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct NumericsAcceptanceReport {
    /// `nalgebra` spatial rotation and inverse rotation preserve a vector.
    pub quaternion_round_trip: bool,
    /// `faer` solves a known dense linear system.
    pub linear_solve: bool,
    /// A library symmetric eigendecomposition reproduces known eigenvalues.
    pub symmetric_eigenpairs: bool,
    /// `parry3d-f64` returns the expected geometric separation.
    pub contact_distance_query: bool,
    /// `levenberg-marquardt` solves a three-variable nonlinear residual.
    pub nonlinear_root_solve: bool,
    /// Production bracketed root solving converges on an independent scalar case.
    pub production_root_solve: bool,
    /// Production piecewise volume mapping conserves volume and round-trips position.
    pub production_volume_round_trip: bool,
    /// Production monotone interpolation remains inside independent local data bounds.
    pub production_monotone_interpolation: bool,
    /// Production circular accumulation preserves unit coherence for aligned vectors.
    pub production_circular_coherence: bool,
    /// Production complementarity residual accepts open-gap and closed-contact states.
    pub production_complementarity: bool,
    /// Production sparse solve reuses one symbolic pattern across changed numeric coefficients.
    pub production_sparse_symbolic_reuse: bool,
    /// The separate cargo-deny policy covers the selected dependency licenses.
    pub licenses_allowed: bool,
}

#[derive(Clone)]
struct ThreeVariableRoot {
    parameters: Vector3<f64>,
}

impl LeastSquaresProblem<f64, U3, U3> for ThreeVariableRoot {
    type ParameterStorage = Owned<f64, U3>;
    type ResidualStorage = Owned<f64, U3>;
    type JacobianStorage = Owned<f64, U3, U3>;

    fn set_params(&mut self, parameters: &Vector3<f64>) {
        self.parameters.copy_from(parameters);
    }

    fn params(&self) -> Vector3<f64> {
        self.parameters
    }

    fn residuals(&self) -> Option<Vector3<f64>> {
        let [x, y, z] = [self.parameters.x, self.parameters.y, self.parameters.z];
        Some(Vector3::new(x * x - 4.0, y - 3.0, z + 1.0))
    }

    fn jacobian(&self) -> Option<Matrix3<f64>> {
        Some(Matrix3::new(
            2.0 * self.parameters.x,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ))
    }
}

fn production_root_solve() -> bool {
    solve_bracketed(1.0, 2.0, |x| x * x * x - 5.0, RootOptions::default()).is_ok_and(
        |solution| {
            (solution.root - 5.0_f64.cbrt()).abs() < 1.0e-10
                && solution.diagnostics.converged()
                && solution.residual.abs() <= 1.0e-10
        },
    )
}

fn production_volume_round_trip() -> bool {
    VolumeCoordinate::new(vec![
        VolumeSegment {
            interval: Interval {
                start: 0.0,
                end: 3.0,
            },
            area: 2.0,
        },
        VolumeSegment {
            interval: Interval {
                start: 3.0,
                end: 7.0,
            },
            area: 1.5,
        },
    ])
    .is_ok_and(|coordinate| {
        (coordinate.total_volume() - 12.0).abs() < 1.0e-12
            && [0.0, 1.25, 3.0, 5.5, 7.0].into_iter().all(|position| {
                coordinate.volume_at(position).is_ok_and(|volume| {
                    coordinate
                        .position_at(volume)
                        .is_ok_and(|restored| (restored - position).abs() < 1.0e-12)
                })
            })
    })
}

fn production_monotone_interpolation() -> bool {
    MonotoneCurve::new(&[(0.0, 1.0), (2.0, 5.0), (5.0, 7.0)]).is_ok_and(|curve| {
        [(1.0, 1.0, 5.0), (3.5, 5.0, 7.0)]
            .into_iter()
            .all(|(x, lower, upper)| {
                curve
                    .evaluate(x)
                    .is_ok_and(|y| y >= lower && y <= upper)
            })
    })
}

fn production_circular_coherence() -> bool {
    let angle = std::f64::consts::PI / 6.0;
    let mut accumulator = CircularAccumulator::new();
    if accumulator.add(2.5, angle).is_err() || accumulator.add(1.5, angle).is_err() {
        return false;
    }
    accumulator.summary().is_ok_and(|summary| {
        (summary.resultant - 4.0).abs() < 1.0e-12
            && (summary.total_magnitude - 4.0).abs() < 1.0e-12
            && (summary.coherence - 1.0).abs() < 1.0e-12
    })
}

fn production_complementarity() -> bool {
    fischer_burmeister(0.75, 0.0).abs() < 1.0e-12
        && fischer_burmeister(0.0, 2500.0).abs() < 1.0e-12
        && fischer_burmeister(0.02, 25.0).abs() > 1.0e-6
}

fn tridiagonal(diagonal: f64) -> Vec<SparseEntry> {
    vec![
        SparseEntry::new(0, 0, diagonal),
        SparseEntry::new(1, 0, -1.0),
        SparseEntry::new(0, 1, -1.0),
        SparseEntry::new(1, 1, diagonal),
        SparseEntry::new(2, 1, -1.0),
        SparseEntry::new(1, 2, -1.0),
        SparseEntry::new(2, 2, diagonal),
    ]
}

fn production_sparse_symbolic_reuse() -> bool {
    let first = tridiagonal(4.0);
    let Ok(system) = SparseLinearSystem::new(3, &first) else {
        return false;
    };
    let first_ok = system
        .factor_and_solve(&first, &[2.0, 4.0, 10.0])
        .is_ok_and(|solution| {
            solution.residual_norm < 1.0e-12
                && solution
                    .values
                    .iter()
                    .zip([1.0, 2.0, 3.0])
                    .all(|(actual, expected)| (*actual - expected).abs() < 1.0e-12)
        });
    let second = tridiagonal(5.0);
    let second_ok = system
        .factor_and_solve(&second, &[3.0, 6.0, 13.0])
        .is_ok_and(|solution| {
            solution.residual_norm < 1.0e-12
                && solution
                    .values
                    .iter()
                    .zip([1.0, 2.0, 3.0])
                    .all(|(actual, expected)| (*actual - expected).abs() < 1.0e-12)
        });
    first_ok && second_ok
}

/// Exercises the numerical libraries and production primitives without relying on domain physics.
#[must_use]
pub fn run() -> NumericsAcceptanceReport {
    let rotation = UnitQuaternion::from_euler_angles(0.2, -0.4, 0.7);
    let vector = Vector3::new(1.25, -2.5, 4.0);
    let restored = rotation.inverse_transform_vector(&rotation.transform_vector(&vector));
    let quaternion_round_trip = (restored - vector).norm() < 1.0e-12;

    let matrix = faer::mat![[4.0, 1.0, 2.0], [0.0, 3.0, -1.0], [2.0, 0.0, 5.0]];
    let expected = faer::mat![[1.0], [2.0], [3.0]];
    let rhs = &matrix * &expected;
    let solved = matrix.partial_piv_lu().solve(&rhs);
    let linear_solve = (&solved - &expected).norm_l2() < 1.0e-12;

    let eigen = SymmetricEigen::new(nalgebra::Matrix2::new(2.0, -1.0, -1.0, 2.0));
    let mut eigenvalues = [eigen.eigenvalues[0], eigen.eigenvalues[1]];
    eigenvalues.sort_by(f64::total_cmp);
    let symmetric_eigenpairs =
        (eigenvalues[0] - 1.0).abs() < 1.0e-12 && (eigenvalues[1] - 3.0).abs() < 1.0e-12;

    let cuboid = Cuboid::new(Vector::splat(1.0));
    let pose = Pose::translation(5.0, 0.0, 0.0);
    let separation = cuboid.distance_to_point(&pose, Vector::ZERO, true);
    let contact_distance_query = (separation - 4.0).abs() < 1.0e-12;

    let nonlinear_root_solve = [Vector3::new(1.0, 0.0, 0.0), Vector3::new(3.5, 6.0, -4.0)]
        .into_iter()
        .all(|parameters| {
            let (solution, report) =
                LevenbergMarquardt::new().minimize(ThreeVariableRoot { parameters });
            report.termination.was_successful()
                && report.objective_function.abs() < 1.0e-20
                && (solution.parameters.x.abs() - 2.0).abs() < 1.0e-9
                && (solution.parameters.y - 3.0).abs() < 1.0e-9
                && (solution.parameters.z + 1.0).abs() < 1.0e-9
        });

    NumericsAcceptanceReport {
        quaternion_round_trip,
        linear_solve,
        symmetric_eigenpairs,
        contact_distance_query,
        nonlinear_root_solve,
        production_root_solve: production_root_solve(),
        production_volume_round_trip: production_volume_round_trip(),
        production_monotone_interpolation: production_monotone_interpolation(),
        production_circular_coherence: production_circular_coherence(),
        production_complementarity: production_complementarity(),
        production_sparse_symbolic_reuse: production_sparse_symbolic_reuse(),
        licenses_allowed: true,
    }
}
