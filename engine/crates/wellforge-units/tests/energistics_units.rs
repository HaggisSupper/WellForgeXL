//! Acceptance tests for Energistics-style unit symbols.

use wellforge_units::{Quantity, QuantityClass, UnitError};

#[test]
fn rejects_force_unit_for_pressure_quantity() {
    let result = Quantity::parse(10.0, "kN", QuantityClass::Pressure);
    assert!(matches!(result, Err(UnitError::WrongQuantityClass { .. })));
}

#[test]
fn preserves_wire_value_and_converts_to_si() {
    let quantity = Quantity::parse(10.0, "in", QuantityClass::Length).unwrap();
    assert!((quantity.value - 10.0).abs() < f64::EPSILON);
    assert_eq!(quantity.unit, "in");
    assert!((quantity.si_value - 0.254).abs() < 1.0e-12);
}
