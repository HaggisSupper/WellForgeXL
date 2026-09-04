use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct BeamElement {
    pub length_m: f64,
    pub youngs_modulus_pa: f64,
    pub second_moment_m4: f64,
    pub outer_radius_m: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct BeamResponse {
    pub end_shear_n: [f64; 2],
    pub end_moment_nm: [f64; 2],
    pub maximum_bending_stress_pa: f64,
    pub maximum_normal_stress_pa: f64,
    pub strain_energy_j: f64,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum BeamError {
    #[error("beam properties and degrees of freedom must be finite and physically valid")]
    InvalidElement,
}

impl BeamElement {
    /// Local 2-D Euler–Bernoulli `[y1, theta1, y2, theta2]` stiffness matrix.
    pub fn stiffness_matrix(self) -> Result<[[f64; 4]; 4], BeamError> {
        self.validate()?;
        let l = self.length_m;
        let flexural_rigidity = self.youngs_modulus_pa * self.second_moment_m4;
        let translation = 12.0 * flexural_rigidity / l / l / l;
        let translation_rotation = 6.0 * flexural_rigidity / l / l;
        let rotation = 4.0 * flexural_rigidity / l;
        let coupled_rotation = 2.0 * flexural_rigidity / l;
        let stiffness = [
            [
                translation,
                translation_rotation,
                -translation,
                translation_rotation,
            ],
            [
                translation_rotation,
                rotation,
                -translation_rotation,
                coupled_rotation,
            ],
            [
                -translation,
                -translation_rotation,
                translation,
                -translation_rotation,
            ],
            [
                translation_rotation,
                coupled_rotation,
                -translation_rotation,
                rotation,
            ],
        ];
        if stiffness.iter().flatten().all(|value| value.is_finite()) {
            Ok(stiffness)
        } else {
            Err(BeamError::InvalidElement)
        }
    }
    pub fn respond(
        self,
        displacement: [f64; 4],
        axial_stress_pa: f64,
    ) -> Result<BeamResponse, BeamError> {
        if !displacement.iter().all(|v| v.is_finite()) || !axial_stress_pa.is_finite() {
            return Err(BeamError::InvalidElement);
        }
        let stiffness = self.stiffness_matrix()?;
        let mut force = [0.0; 4];
        for row in 0..4 {
            force[row] = (0..4)
                .map(|col| stiffness[row][col] * displacement[col])
                .sum();
        }
        let maximum_moment_nm = force[1].abs().max(force[3].abs());
        let maximum_bending_stress_pa =
            maximum_moment_nm * self.outer_radius_m / self.second_moment_m4;
        let strain_energy_j = 0.5 * (0..4).map(|i| displacement[i] * force[i]).sum::<f64>();
        let response = BeamResponse {
            end_shear_n: [force[0], force[2]],
            end_moment_nm: [force[1], force[3]],
            maximum_bending_stress_pa,
            maximum_normal_stress_pa: axial_stress_pa.abs() + maximum_bending_stress_pa,
            strain_energy_j,
        };
        if response.end_shear_n.iter().all(|value| value.is_finite())
            && response.end_moment_nm.iter().all(|value| value.is_finite())
            && response.maximum_bending_stress_pa.is_finite()
            && response.maximum_normal_stress_pa.is_finite()
            && response.strain_energy_j.is_finite()
        {
            Ok(response)
        } else {
            Err(BeamError::InvalidElement)
        }
    }
    fn validate(self) -> Result<(), BeamError> {
        if ![
            self.length_m,
            self.youngs_modulus_pa,
            self.second_moment_m4,
            self.outer_radius_m,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        {
            return Err(BeamError::InvalidElement);
        }
        Ok(())
    }
}
