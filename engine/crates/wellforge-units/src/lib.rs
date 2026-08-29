//! Dimension-safe wire quantities for `WellForge` engine contracts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uom::si::f64::{Angle, Force, Length, MassDensity, Pressure, Torque};
use uom::si::{
    angle::{degree, radian},
    force::{kilonewton, newton, pound_force},
    length::{foot, inch, meter, millimeter},
    mass_density::{kilogram_per_cubic_meter, pound_per_cubic_foot},
    pressure::{gigapascal, megapascal, pascal, pound_force_per_square_inch},
    torque::{newton_meter, pound_force_foot},
};

/// Physical dimension associated with a wire unit.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityClass {
    /// Length, depth, diameter or displacement.
    Length,
    /// Force.
    Force,
    /// Pressure or stress.
    Pressure,
    /// Torque.
    Torque,
    /// Mass density.
    Density,
    /// Plane angle.
    Angle,
    /// Cycles per second.
    Frequency,
    /// Revolutions per minute.
    RotationalSpeed,
    /// Dimensionless value.
    Dimensionless,
}

/// Quantity preserved in its source unit and normalized to canonical SI.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Quantity {
    /// Original numeric wire value.
    pub value: f64,
    /// Original Energistics-compatible symbol.
    pub unit: String,
    /// Physical quantity class.
    pub class: QuantityClass,
    /// Canonical SI value used by solvers.
    pub si_value: f64,
}

/// Strict unit-registry failure.
#[derive(Debug, Error, PartialEq)]
pub enum UnitError {
    /// Unit symbol is not in the checked-in registry.
    #[error("unsupported unit symbol: {0}")]
    UnsupportedSymbol(String),
    /// Symbol belongs to a different physical dimension.
    #[error("unit {unit} belongs to {actual:?}, not {requested:?}")]
    WrongQuantityClass {
        /// Rejected wire symbol.
        unit: String,
        /// Requested class.
        requested: QuantityClass,
        /// Registry class.
        actual: QuantityClass,
    },
    /// Value is NaN or infinite.
    #[error("quantity value must be finite")]
    NonFinite,
}

impl Quantity {
    /// Parses a wire value, validates its dimension and converts it to SI using `uom`.
    ///
    /// # Errors
    ///
    /// Returns [`UnitError`] when the value is non-finite, the symbol is unsupported or its dimension is wrong.
    pub fn parse(value: f64, unit: &str, requested: QuantityClass) -> Result<Self, UnitError> {
        if !value.is_finite() {
            return Err(UnitError::NonFinite);
        }
        let (actual, si_value) = convert(value, unit)?;
        if actual != requested {
            return Err(UnitError::WrongQuantityClass {
                unit: unit.to_owned(),
                requested,
                actual,
            });
        }
        Ok(Self {
            value,
            unit: unit.to_owned(),
            class: requested,
            si_value,
        })
    }
}

#[allow(clippy::too_many_lines)]
fn convert(value: f64, unit: &str) -> Result<(QuantityClass, f64), UnitError> {
    let item = match unit {
        "m" => (
            QuantityClass::Length,
            Length::new::<meter>(value).get::<meter>(),
        ),
        "mm" => (
            QuantityClass::Length,
            Length::new::<millimeter>(value).get::<meter>(),
        ),
        "ft" => (
            QuantityClass::Length,
            Length::new::<foot>(value).get::<meter>(),
        ),
        "in" => (
            QuantityClass::Length,
            Length::new::<inch>(value).get::<meter>(),
        ),
        "N" => (
            QuantityClass::Force,
            Force::new::<newton>(value).get::<newton>(),
        ),
        "kN" => (
            QuantityClass::Force,
            Force::new::<kilonewton>(value).get::<newton>(),
        ),
        "lbf" => (
            QuantityClass::Force,
            Force::new::<pound_force>(value).get::<newton>(),
        ),
        "Pa" => (
            QuantityClass::Pressure,
            Pressure::new::<pascal>(value).get::<pascal>(),
        ),
        "MPa" => (
            QuantityClass::Pressure,
            Pressure::new::<megapascal>(value).get::<pascal>(),
        ),
        "GPa" => (
            QuantityClass::Pressure,
            Pressure::new::<gigapascal>(value).get::<pascal>(),
        ),
        "psi" => (
            QuantityClass::Pressure,
            Pressure::new::<pound_force_per_square_inch>(value).get::<pascal>(),
        ),
        "N.m" => (
            QuantityClass::Torque,
            Torque::new::<newton_meter>(value).get::<newton_meter>(),
        ),
        "lbf.ft" => (
            QuantityClass::Torque,
            Torque::new::<pound_force_foot>(value).get::<newton_meter>(),
        ),
        "kg/m3" => (
            QuantityClass::Density,
            MassDensity::new::<kilogram_per_cubic_meter>(value).get::<kilogram_per_cubic_meter>(),
        ),
        "lbm/ft3" => (
            QuantityClass::Density,
            MassDensity::new::<pound_per_cubic_foot>(value).get::<kilogram_per_cubic_meter>(),
        ),
        "rad" => (
            QuantityClass::Angle,
            Angle::new::<radian>(value).get::<radian>(),
        ),
        "deg" => (
            QuantityClass::Angle,
            Angle::new::<degree>(value).get::<radian>(),
        ),
        "Hz" => (QuantityClass::Frequency, value),
        "rpm" => (
            QuantityClass::RotationalSpeed,
            value * std::f64::consts::TAU / 60.0,
        ),
        "1" => (QuantityClass::Dimensionless, value),
        _ => return Err(UnitError::UnsupportedSymbol(unit.to_owned())),
    };
    Ok(item)
}
