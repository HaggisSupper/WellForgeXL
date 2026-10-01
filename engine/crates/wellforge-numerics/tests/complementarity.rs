//! Contract tests for contact complementarity primitives.

use wellforge_numerics::{fischer_burmeister, fischer_burmeister_gradient};

#[test]
fn open_gap_and_closed_contact_satisfy_complementarity() {
    assert!(fischer_burmeister(0.25, 0.0).abs() < 1.0e-12);
    assert!(fischer_burmeister(0.0, 1250.0).abs() < 1.0e-12);
}

#[test]
fn penetration_or_simultaneous_gap_and_reaction_are_nonzero() {
    assert!(fischer_burmeister(-0.01, 100.0).abs() > 1.0e-6);
    assert!(fischer_burmeister(0.01, 100.0).abs() > 1.0e-6);
}

#[test]
fn smoothed_gradient_matches_central_difference_away_from_origin() {
    let gap = 0.2;
    let reaction = 0.7;
    let smoothing = 1.0e-8;
    let step = 1.0e-6;
    let (dgap, dreaction) = fischer_burmeister_gradient(gap, reaction, smoothing);

    let residual = |g: f64, r: f64| (g * g + r * r + smoothing * smoothing).sqrt() - g - r;
    let numerical_gap =
        (residual(gap + step, reaction) - residual(gap - step, reaction)) / (2.0 * step);
    let numerical_reaction =
        (residual(gap, reaction + step) - residual(gap, reaction - step)) / (2.0 * step);

    assert!((dgap - numerical_gap).abs() < 1.0e-8);
    assert!((dreaction - numerical_reaction).abs() < 1.0e-8);
}

#[test]
fn smoothed_origin_gradient_is_finite() {
    let gradient = fischer_burmeister_gradient(0.0, 0.0, 1.0e-8);
    assert!(gradient.0.is_finite());
    assert!(gradient.1.is_finite());
}
