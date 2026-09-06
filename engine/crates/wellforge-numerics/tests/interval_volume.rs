//! Contract tests for deterministic piecewise intervals and volume coordinates.

use wellforge_numerics::{Interval, VolumeCoordinate, VolumeSegment, partition_boundaries};

fn assert_exact(value: f64, expected: f64) {
    assert_eq!(value.to_bits(), expected.to_bits());
}

#[test]
fn partition_merges_geometry_boundaries_without_zero_length_intervals() {
    let hole = [0.0, 100.0, 200.0];
    let string = [0.0, 50.0, 50.0, 150.0, 200.0];

    let intervals = partition_boundaries(&[&hole, &string]).expect("valid geometry boundaries");

    assert_eq!(intervals.len(), 4);
    let expected = [(0.0, 50.0), (50.0, 100.0), (100.0, 150.0), (150.0, 200.0)];
    for (interval, (start, end)) in intervals.iter().zip(expected) {
        assert_exact(interval.start, start);
        assert_exact(interval.end, end);
        assert!(interval.end > interval.start);
    }
}

#[test]
fn volume_coordinate_conserves_piecewise_volume_and_inverts_position() {
    let coordinate = VolumeCoordinate::new(vec![
        VolumeSegment {
            interval: Interval {
                start: 0.0,
                end: 10.0,
            },
            area: 2.0,
        },
        VolumeSegment {
            interval: Interval {
                start: 10.0,
                end: 20.0,
            },
            area: 4.0,
        },
    ])
    .expect("valid piecewise capacity");

    assert!((coordinate.total_volume() - 60.0).abs() < 1.0e-12);
    assert!((coordinate.volume_at(5.0).expect("inside coordinate") - 10.0).abs() < 1.0e-12);
    assert!((coordinate.volume_at(15.0).expect("inside coordinate") - 40.0).abs() < 1.0e-12);
    assert!((coordinate.position_at(40.0).expect("inside volume") - 15.0).abs() < 1.0e-12);

    for position in [0.0, 2.5, 10.0, 12.5, 20.0] {
        let volume = coordinate.volume_at(position).expect("volume lookup");
        let restored = coordinate.position_at(volume).expect("inverse lookup");
        assert!((restored - position).abs() < 1.0e-12);
    }
}
