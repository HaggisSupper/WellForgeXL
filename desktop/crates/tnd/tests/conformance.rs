use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};

use wellforge_tnd::{
    BucklingError, BucklingLoads, BucklingRegime, BucklingSection, Operation, StringSegment,
    TndError, classify_buckling, critical_buckling_loads, solve_soft_string,
};

const EPSILON: f64 = 1.0e-9;

fn assert_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= EPSILON * scale,
        "expected {expected}, got {actual}"
    );
}

fn inclined_segment(friction_factor: f64) -> StringSegment {
    StringSegment {
        length_m: 10.0,
        inclination_rad: FRAC_PI_3,
        buoyed_weight_n_per_m: 100.0,
        friction_factor,
        contact_radius_m: 0.1,
    }
}

fn short_confined_section() -> BucklingSection {
    BucklingSection {
        length_m: 1.0,
        youngs_modulus_pa: 200.0e9,
        second_moment_m4: 1.0e-6,
        radial_clearance_m: 0.02,
        buoyed_weight_n_per_m: 1_000.0,
        inclination_rad: FRAC_PI_2,
        effective_length_factor: 1.0,
    }
}

#[test]
fn zero_friction_reduces_every_operation_to_gravity_only() {
    let segments = [
        inclined_segment(0.0),
        StringSegment {
            length_m: 5.0,
            inclination_rad: 0.0,
            buoyed_weight_n_per_m: 80.0,
            friction_factor: 0.0,
            contact_radius_m: 0.2,
        },
    ];
    let expected_first = 500.0;
    let expected_second = 900.0;

    for operation in [
        Operation::RunInHole,
        Operation::PullOutOfHole,
        Operation::Rotate,
        Operation::Backream,
    ] {
        let results = solve_soft_string(&segments, operation).expect("finite SI inputs solve");
        assert_eq!(results.len(), 2);
        assert_close(results[0].axial_force_n, expected_first);
        assert_close(results[1].axial_force_n, expected_second);
        assert_close(results[0].drag_n, 0.0);
        assert_close(results[1].drag_n, 0.0);
        assert_close(results[0].torque_nm, 0.0);
        assert_close(results[1].torque_nm, 0.0);
    }
}

#[test]
fn operations_apply_drag_signs_and_torque_only_when_rotating() {
    let segment = inclined_segment(0.2);
    let gravity_n = 500.0;
    let drag_n = 173.205_080_756_887_72;
    let torque_nm = 17.320_508_075_688_77;

    for (operation, expected_axial_force_n, expected_torque_nm) in [
        (Operation::RunInHole, gravity_n - drag_n, 0.0),
        (Operation::PullOutOfHole, gravity_n + drag_n, 0.0),
        (Operation::Rotate, gravity_n, torque_nm),
        (Operation::Backream, gravity_n + drag_n, torque_nm),
    ] {
        let result = solve_soft_string(&[segment], operation)
            .expect("finite SI inputs solve")
            .pop()
            .expect("one segment gives one result");
        assert_close(result.axial_force_n, expected_axial_force_n);
        assert_close(result.drag_n, drag_n);
        assert_close(result.torque_nm, expected_torque_nm);
    }
}

#[test]
fn rejects_non_finite_or_non_physical_si_inputs() {
    assert!(matches!(
        solve_soft_string(&[], Operation::RunInHole),
        Err(TndError::EmptyString)
    ));
    assert!(matches!(
        solve_soft_string(
            &[StringSegment {
                length_m: f64::NAN,
                ..inclined_segment(0.2)
            }],
            Operation::RunInHole,
        ),
        Err(TndError::InvalidSegment)
    ));
    assert!(matches!(
        critical_buckling_loads(BucklingSection {
            youngs_modulus_pa: f64::INFINITY,
            ..short_confined_section()
        }),
        Err(BucklingError::InvalidSection)
    ));
    assert_eq!(
        classify_buckling(
            f64::NAN,
            BucklingLoads {
                euler_load_n: 1.0,
                sinusoidal_onset_n: 2.0,
                helical_onset_n: 3.0,
            }
        ),
        Err(BucklingError::InvalidCompression)
    );
}

#[test]
fn rejects_finite_si_inputs_that_overflow_the_model_equations() {
    assert!(matches!(
        solve_soft_string(
            &[StringSegment {
                length_m: f64::MAX,
                inclination_rad: FRAC_PI_2,
                buoyed_weight_n_per_m: 2.0,
                friction_factor: 1.0,
                contact_radius_m: 1.0,
            }],
            Operation::Backream,
        ),
        Err(TndError::NumericalOverflow)
    ));
    assert_eq!(
        critical_buckling_loads(BucklingSection {
            youngs_modulus_pa: f64::MAX,
            second_moment_m4: 1.0,
            ..short_confined_section()
        }),
        Err(BucklingError::NumericalOverflow)
    );
}

#[test]
fn preserves_representable_buckling_thresholds_when_intermediate_products_overflow() {
    for section in [
        BucklingSection {
            youngs_modulus_pa: f64::MAX,
            second_moment_m4: 2.0,
            length_m: 1.0e10,
            effective_length_factor: 1.0,
            radial_clearance_m: 1.0,
            buoyed_weight_n_per_m: 1.0,
            inclination_rad: FRAC_PI_2,
        },
        BucklingSection {
            youngs_modulus_pa: 1.0e307,
            second_moment_m4: 1.0,
            length_m: 1.0,
            effective_length_factor: 1.0,
            radial_clearance_m: f64::MAX,
            buoyed_weight_n_per_m: f64::MAX,
            inclination_rad: FRAC_PI_2,
        },
    ] {
        let loads = critical_buckling_loads(section)
            .expect("representable buckling thresholds must not fail on intermediate overflow");
        for threshold in [
            loads.euler_load_n,
            loads.sinusoidal_onset_n,
            loads.helical_onset_n,
        ] {
            assert!(threshold.is_finite() && threshold > 0.0);
        }
    }
}

#[test]
fn rejects_negative_zero_inputs_that_could_produce_signed_torque() {
    for operation in [Operation::Rotate, Operation::Backream] {
        assert!(matches!(
            solve_soft_string(
                &[StringSegment {
                    buoyed_weight_n_per_m: -0.0,
                    ..inclined_segment(0.2)
                }],
                operation,
            ),
            Err(TndError::InvalidSegment)
        ));
    }
    assert!(matches!(
        solve_soft_string(
            &[StringSegment {
                friction_factor: -0.0,
                ..inclined_segment(0.2)
            }],
            Operation::Rotate,
        ),
        Err(TndError::InvalidSegment)
    ));
    assert!(matches!(
        solve_soft_string(
            &[StringSegment {
                contact_radius_m: -0.0,
                ..inclined_segment(0.2)
            }],
            Operation::Backream,
        ),
        Err(TndError::InvalidSegment)
    ));
}

#[test]
fn rejects_confined_buckling_sections_without_lateral_buoyed_loading() {
    assert_eq!(
        critical_buckling_loads(BucklingSection {
            inclination_rad: 0.0,
            ..short_confined_section()
        }),
        Err(BucklingError::InvalidSection)
    );
    assert_eq!(
        critical_buckling_loads(BucklingSection {
            buoyed_weight_n_per_m: 0.0,
            ..short_confined_section()
        }),
        Err(BucklingError::InvalidSection)
    );
}

#[test]
fn classifies_euler_and_confined_thresholds_at_their_boundaries() {
    let euler_limited = BucklingSection {
        length_m: 10.0,
        ..short_confined_section()
    };
    let euler_loads = critical_buckling_loads(euler_limited).expect("valid euler-limited section");
    assert!(euler_loads.euler_load_n < euler_loads.sinusoidal_onset_n);
    assert_eq!(
        classify_buckling(euler_loads.euler_load_n - 1.0, euler_loads),
        Ok(BucklingRegime::Stable)
    );
    assert_eq!(
        classify_buckling(euler_loads.euler_load_n, euler_loads),
        Ok(BucklingRegime::Sinusoidal)
    );

    let confined_loads =
        critical_buckling_loads(short_confined_section()).expect("valid confined section");
    assert!(confined_loads.sinusoidal_onset_n < confined_loads.euler_load_n);
    assert_eq!(
        classify_buckling(confined_loads.sinusoidal_onset_n - 1.0, confined_loads),
        Ok(BucklingRegime::Stable)
    );
    assert_eq!(
        classify_buckling(confined_loads.sinusoidal_onset_n, confined_loads),
        Ok(BucklingRegime::Sinusoidal)
    );
    assert_eq!(
        classify_buckling(confined_loads.helical_onset_n, confined_loads),
        Ok(BucklingRegime::Helical)
    );
}

#[test]
fn rejects_buckling_loads_with_inconsistent_threshold_ordering() {
    let inconsistent = BucklingLoads {
        euler_load_n: 1_000.0,
        sinusoidal_onset_n: 200.0,
        helical_onset_n: 100.0,
    };

    assert_eq!(
        classify_buckling(150.0, inconsistent),
        Err(BucklingError::InvalidCompression)
    );
}

#[test]
fn published_euler_formula_matches_the_finite_section_model() {
    let section = short_confined_section();
    let loads = critical_buckling_loads(section).expect("valid finite SI section");
    let expected_euler = 1_973_920.880_217_871_6;

    assert_close(loads.euler_load_n, expected_euler);
}
