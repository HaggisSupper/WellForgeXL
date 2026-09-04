use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BhaComponentKind {
    Bit,
    PointTheBitRss,
    PushTheBitRss,
    Motor,
    Stabilizer,
    NonMagneticDrillCollar,
    HeavyWeightDrillPipe,
    CompactLwd,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BhaComponent {
    pub id: String,
    pub name: String,
    pub kind: BhaComponentKind,
    pub length_m: f64,
    pub outer_diameter_m: f64,
    pub inner_diameter_m: f64,
    pub youngs_modulus_pa: f64,
    pub density_kg_m3: f64,
}

impl BhaComponent {
    pub fn second_moment_m4(&self) -> Result<f64, CatalogError> {
        self.validate()?;
        let diameter_difference = self.outer_diameter_m - self.inner_diameter_m;
        let diameter_sum = self.outer_diameter_m + self.inner_diameter_m;
        let squared_diameter_sum = self.outer_diameter_m * self.outer_diameter_m
            + self.inner_diameter_m * self.inner_diameter_m;
        let second_moment = (std::f64::consts::PI / 64.0)
            * diameter_difference
            * diameter_sum
            * squared_diameter_sum;
        if second_moment.is_finite() && second_moment > 0.0 {
            Ok(second_moment)
        } else {
            Err(CatalogError::InvalidComponent(self.id.clone()))
        }
    }
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.id.trim().is_empty()
            || self.name.trim().is_empty()
            || ![
                self.length_m,
                self.outer_diameter_m,
                self.inner_diameter_m,
                self.youngs_modulus_pa,
                self.density_kg_m3,
            ]
            .iter()
            .all(|v| v.is_finite())
            || self.length_m <= 0.0
            || self.outer_diameter_m <= 0.0
            || self.inner_diameter_m < 0.0
            || self.inner_diameter_m >= self.outer_diameter_m
            || self.youngs_modulus_pa <= 0.0
            || self.density_kg_m3 <= 0.0
        {
            return Err(CatalogError::InvalidComponent(self.id.clone()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BhaAssembly {
    pub id: String,
    pub name: String,
    pub components_from_bit: Vec<BhaComponent>,
}
impl BhaAssembly {
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.id.trim().is_empty() || self.components_from_bit.is_empty() {
            return Err(CatalogError::InvalidAssembly);
        }
        for component in &self.components_from_bit {
            component.validate()?;
        }
        if self.components_from_bit[0].kind != BhaComponentKind::Bit {
            return Err(CatalogError::BitMustBeFirst);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CatalogError {
    #[error("invalid BHA component: {0}")]
    InvalidComponent(String),
    #[error("BHA assembly must have an identifier and at least one component")]
    InvalidAssembly,
    #[error("the first BHA component must be the bit")]
    BitMustBeFirst,
}
