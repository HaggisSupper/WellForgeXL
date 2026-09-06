//! Complementarity residuals for unilateral constraints.

/// Fischer-Burmeister residual for complementary non-negative variables.
///
/// The residual is zero when `left >= 0`, `right >= 0`, and `left * right = 0`.
#[must_use]
pub fn fischer_burmeister(left: f64, right: f64) -> f64 {
    left.hypot(right) - left - right
}

/// Gradient of the smoothed Fischer-Burmeister residual.
///
/// The smoothed residual is `sqrt(left^2 + right^2 + smoothing^2) - left - right`.
/// A non-negative finite smoothing value is expected. Invalid smoothing produces NaN components
/// rather than silently changing the caller's numerical model.
#[must_use]
pub fn fischer_burmeister_gradient(left: f64, right: f64, smoothing: f64) -> (f64, f64) {
    if !left.is_finite() || !right.is_finite() || !smoothing.is_finite() || smoothing < 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let denominator = (left * left + right * right + smoothing * smoothing).sqrt();
    if denominator > 0.0 {
        (left / denominator - 1.0, right / denominator - 1.0)
    } else {
        (-1.0, -1.0)
    }
}
