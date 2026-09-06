//! Contract tests for monotone curve evaluation and circular/vector accumulation.

use wellforge_numerics::{CircularAccumulator, InterpolationError, MonotoneCurve};

#[test]
fn monotone_curve_stays_inside_local_data_bounds() {
    let curve = MonotoneCurve::new(&[(0.0, 0.0), (1.0, 2.0), (2.0, 3.0)])
        .expect("strictly increasing x data");

    for (x, lower, upper) in [(0.25, 0.0, 2.0), (0.75, 0.0, 2.0), (1.25, 2.0, 3.0), (1.75, 2.0, 3.0)] {
        let y = curve.evaluate(x).expect("inside curve domain");
        assert!(y >= lower && y <= upper);
    }
}

#[test]
fn monotone_curve_rejects_repeated_x() {
    let error = MonotoneCurve::new(&[(0.0, 1.0), (1.0, 2.0), (1.0, 3.0)])
        .expect_err("repeated x is ambiguous");

    assert_eq!(error, InterpolationError::NonIncreasingX);
}

#[test]
fn aligned_vectors_have_unit_coherence() {
    let mut accumulator = CircularAccumulator::new();
    accumulator.add(2.0, 0.0).expect("valid vector");
    accumulator.add(3.0, 0.0).expect("valid vector");

    let summary = accumulator.summary().expect("non-empty vector set");
    assert!((summary.x - 5.0).abs() < 1.0e-12);
    assert!(summary.y.abs() < 1.0e-12);
    assert!((summary.resultant - 5.0).abs() < 1.0e-12);
    assert!((summary.total_magnitude - 5.0).abs() < 1.0e-12);
    assert!((summary.coherence - 1.0).abs() < 1.0e-12);
}

#[test]
fn opposing_vectors_cancel() {
    let mut accumulator = CircularAccumulator::new();
    accumulator.add(1.0, 0.0).expect("valid vector");
    accumulator
        .add(1.0, std::f64::consts::PI)
        .expect("valid vector");

    let summary = accumulator.summary().expect("non-empty vector set");
    assert!(summary.x.abs() < 1.0e-12);
    assert!(summary.y.abs() < 1.0e-12);
    assert!(summary.resultant < 1.0e-12);
    assert!(summary.coherence < 1.0e-12);
}
