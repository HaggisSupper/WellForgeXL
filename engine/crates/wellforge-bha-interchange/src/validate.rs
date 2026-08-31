use crate::{InterchangeError, TubularSection};

pub(crate) fn section(s: &TubularSection) -> Result<(), InterchangeError> {
    if s.length_m <= 0.0 {
        return Err(InterchangeError::InvalidGeometry(
            "section length must be positive".into(),
        ));
    }
    if s.od_m < 0.0 || s.id_m < 0.0 || s.mass_kg.is_some_and(|m| m < 0.0) {
        return Err(InterchangeError::InvalidGeometry(
            "section dimensions and mass must be non-negative".into(),
        ));
    }
    if s.od_m <= s.id_m {
        return Err(InterchangeError::InvalidGeometry(
            "section OD must be greater than ID".into(),
        ));
    }
    Ok(())
}
