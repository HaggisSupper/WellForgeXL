//! Acceptance boundary for third-party numerical libraries.

use faer::prelude::*;
use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt};
use nalgebra::{Matrix3, Owned, SymmetricEigen, U3, UnitQuaternion, Vector3};
use parry3d_f64::{
    math::{Pose, Vector},
    query::PointQuery,
    shape::Cuboid,
};
use serde::{Deserialize, Serialize};

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

/// Exercises the numerical libraries without relying on `WellForge` domain physics.
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
        licenses_allowed: true,
    }
}
