//! Weighted circular/vector accumulation for angular calibration data.

/// Circular/vector accumulation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CircularError {
    /// Magnitude or angle is NaN or infinite.
    NonFinite,
    /// Vector magnitude is negative.
    NegativeMagnitude,
    /// No observations have been accumulated.
    Empty,
}

/// Resultant summary for weighted angular vectors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularSummary {
    /// Cartesian x component of the resultant vector.
    pub x: f64,
    /// Cartesian y component of the resultant vector.
    pub y: f64,
    /// Magnitude of the vector resultant.
    pub resultant: f64,
    /// Sum of individual vector magnitudes before cancellation.
    pub total_magnitude: f64,
    /// Resultant divided by total magnitude, bounded to `[0, 1]` apart from roundoff.
    pub coherence: f64,
}

/// Incremental weighted circular/vector accumulator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularAccumulator {
    x: f64,
    y: f64,
    total_magnitude: f64,
    count: usize,
}

impl CircularAccumulator {
    /// Creates an empty accumulator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            total_magnitude: 0.0,
            count: 0,
        }
    }

    /// Adds one vector represented by non-negative magnitude and angle in radians.
    ///
    /// # Errors
    /// Returns [`CircularError::NonFinite`] for NaN/infinite input and
    /// [`CircularError::NegativeMagnitude`] for negative magnitude.
    pub fn add(&mut self, magnitude: f64, angle_rad: f64) -> Result<(), CircularError> {
        if !magnitude.is_finite() || !angle_rad.is_finite() {
            return Err(CircularError::NonFinite);
        }
        if magnitude < 0.0 {
            return Err(CircularError::NegativeMagnitude);
        }
        self.x += magnitude * angle_rad.cos();
        self.y += magnitude * angle_rad.sin();
        self.total_magnitude += magnitude;
        self.count += 1;
        Ok(())
    }

    /// Returns the accumulated resultant and coherence.
    ///
    /// # Errors
    /// Returns [`CircularError::Empty`] when no observations have been added.
    pub fn summary(&self) -> Result<CircularSummary, CircularError> {
        if self.count == 0 {
            return Err(CircularError::Empty);
        }
        let resultant = self.x.hypot(self.y);
        let coherence = if self.total_magnitude > 0.0 {
            (resultant / self.total_magnitude).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Ok(CircularSummary {
            x: self.x,
            y: self.y,
            resultant,
            total_magnitude: self.total_magnitude,
            coherence,
        })
    }
}

impl Default for CircularAccumulator {
    fn default() -> Self {
        Self::new()
    }
}
