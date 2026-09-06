//! Monotonicity-preserving one-dimensional calibration curves.

/// Calibration curve construction or lookup failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpolationError {
    /// Fewer than two calibration points were supplied.
    TooFewPoints,
    /// A calibration coordinate or value is NaN or infinite.
    NonFinite,
    /// Calibration abscissae are not strictly increasing.
    NonIncreasingX,
    /// The requested abscissa lies outside the calibrated domain.
    OutOfRange,
}

/// A monotonicity-safe piecewise-linear calibration curve.
///
/// Piecewise-linear interpolation is deliberately used as the Phase 1 baseline because it cannot
/// overshoot the local calibration interval. A future higher-order interpolation may replace the
/// implementation only if it preserves this public contract and demonstrates parity.
#[derive(Clone, Debug, PartialEq)]
pub struct MonotoneCurve {
    points: Vec<(f64, f64)>,
}

impl MonotoneCurve {
    /// Builds a calibration curve from strictly increasing x coordinates.
    ///
    /// # Errors
    /// Returns [`InterpolationError::TooFewPoints`] for fewer than two points,
    /// [`InterpolationError::NonFinite`] for NaN/infinite values, and
    /// [`InterpolationError::NonIncreasingX`] when x coordinates do not increase strictly.
    pub fn new(points: &[(f64, f64)]) -> Result<Self, InterpolationError> {
        if points.len() < 2 {
            return Err(InterpolationError::TooFewPoints);
        }
        if points
            .iter()
            .flat_map(|(x, y)| [*x, *y])
            .any(|value| !value.is_finite())
        {
            return Err(InterpolationError::NonFinite);
        }
        if points.windows(2).any(|pair| pair[1].0 <= pair[0].0) {
            return Err(InterpolationError::NonIncreasingX);
        }
        Ok(Self {
            points: points.to_vec(),
        })
    }

    /// Evaluates the curve within its calibrated x domain.
    ///
    /// # Errors
    /// Returns [`InterpolationError::NonFinite`] for a non-finite query and
    /// [`InterpolationError::OutOfRange`] outside the calibration domain.
    pub fn evaluate(&self, x: f64) -> Result<f64, InterpolationError> {
        if !x.is_finite() {
            return Err(InterpolationError::NonFinite);
        }
        let first = self.points[0];
        let last = self.points[self.points.len() - 1];
        if x < first.0 || x > last.0 {
            return Err(InterpolationError::OutOfRange);
        }
        if x.to_bits() == first.0.to_bits() {
            return Ok(first.1);
        }
        if x.to_bits() == last.0.to_bits() {
            return Ok(last.1);
        }
        let upper = self.points.partition_point(|(point_x, _)| *point_x < x);
        let (x0, y0) = self.points[upper - 1];
        let (x1, y1) = self.points[upper];
        let fraction = (x - x0) / (x1 - x0);
        Ok(y0 + fraction * (y1 - y0))
    }
}
