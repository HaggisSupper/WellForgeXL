use wellforge_bha::{BeamElement, BhaAssembly, BhaComponent, BhaComponentKind, CatalogError};

const TOLERANCE: f64 = 1.0e-9;

fn component(kind: BhaComponentKind) -> BhaComponent {
    BhaComponent {
        id: "component-1".into(),
        name: "component".into(),
        kind,
        length_m: 8.0,
        outer_diameter_m: 0.2,
        inner_diameter_m: 0.1,
        youngs_modulus_pa: 207_000_000_000.0,
        density_kg_m3: 7_850.0,
    }
}

fn beam() -> BeamElement {
    BeamElement {
        length_m: 2.0,
        youngs_modulus_pa: 200_000_000_000.0,
        second_moment_m4: 0.000_05,
        outer_radius_m: 0.1,
    }
}

fn assert_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= TOLERANCE * scale,
        "expected {expected}, got {actual}"
    );
}

fn assert_relative_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOLERANCE * expected.abs(),
        "expected {expected}, got {actual}"
    );
}

#[test]
fn valid_component_second_moment_matches_the_hollow_tube_hand_formula() {
    let value = component(BhaComponentKind::Motor)
        .second_moment_m4()
        .expect("valid tubular component");

    assert_close(value, 0.000_073_631_077_818_510_78);
}

#[test]
fn second_moment_rejects_finite_dimensions_that_overflow_the_result() {
    let component = BhaComponent {
        outer_diameter_m: f64::MAX,
        inner_diameter_m: 0.0,
        ..component(BhaComponentKind::Motor)
    };

    assert!(component.second_moment_m4().is_err());
}

#[test]
fn second_moment_accepts_a_representable_thin_annulus_at_large_diameter() {
    let outer_diameter_m = 1.0e81;
    let component = BhaComponent {
        outer_diameter_m,
        inner_diameter_m: f64::from_bits(outer_diameter_m.to_bits() - 1),
        ..component(BhaComponentKind::Motor)
    };

    let second_moment = component
        .second_moment_m4()
        .expect("finite representable annular second moment");
    assert!(second_moment.is_finite());
    assert!(second_moment > 0.0);
}

#[test]
fn component_validation_rejects_invalid_identity_and_physical_properties() {
    let mut cases = Vec::new();

    let mut missing_id = component(BhaComponentKind::Motor);
    missing_id.id = " \t".into();
    cases.push(missing_id);
    let mut missing_name = component(BhaComponentKind::Motor);
    missing_name.name.clear();
    cases.push(missing_name);
    let mut zero_length = component(BhaComponentKind::Motor);
    zero_length.length_m = 0.0;
    cases.push(zero_length);
    let mut nan_diameter = component(BhaComponentKind::Motor);
    nan_diameter.outer_diameter_m = f64::NAN;
    cases.push(nan_diameter);
    let mut negative_bore = component(BhaComponentKind::Motor);
    negative_bore.inner_diameter_m = -0.01;
    cases.push(negative_bore);
    let mut solid_wall = component(BhaComponentKind::Motor);
    solid_wall.inner_diameter_m = solid_wall.outer_diameter_m;
    cases.push(solid_wall);
    let mut zero_modulus = component(BhaComponentKind::Motor);
    zero_modulus.youngs_modulus_pa = 0.0;
    cases.push(zero_modulus);
    let mut infinite_density = component(BhaComponentKind::Motor);
    infinite_density.density_kg_m3 = f64::INFINITY;
    cases.push(infinite_density);

    for invalid in cases {
        assert!(matches!(
            invalid.validate(),
            Err(CatalogError::InvalidComponent(_))
        ));
    }
}

#[test]
fn assembly_validation_accepts_a_valid_bit_first_component_sequence() {
    let assembly = BhaAssembly {
        id: "assembly-1".into(),
        name: "directional assembly".into(),
        components_from_bit: vec![
            component(BhaComponentKind::Bit),
            component(BhaComponentKind::Motor),
        ],
    };

    assert_eq!(assembly.validate(), Ok(()));
}

#[test]
fn assembly_validation_rejects_empty_and_non_bit_first_sequences() {
    let empty = BhaAssembly {
        id: "assembly-1".into(),
        name: "empty assembly".into(),
        components_from_bit: vec![],
    };
    assert_eq!(empty.validate(), Err(CatalogError::InvalidAssembly));

    let non_bit_first = BhaAssembly {
        id: "assembly-1".into(),
        name: "invalid order".into(),
        components_from_bit: vec![component(BhaComponentKind::Motor)],
    };
    assert_eq!(non_bit_first.validate(), Err(CatalogError::BitMustBeFirst));
}

#[test]
fn stiffness_matrix_is_symmetric() {
    let stiffness = beam().stiffness_matrix().expect("valid beam");

    for row in 0..4 {
        for column in 0..4 {
            assert_close(stiffness[row][column], stiffness[column][row]);
        }
    }
}

#[test]
fn stiffness_retains_representable_rotational_terms_for_a_very_long_element() {
    let stiffness = BeamElement {
        length_m: 1.0e150,
        youngs_modulus_pa: 1.0,
        second_moment_m4: 1.0,
        outer_radius_m: 0.1,
    }
    .stiffness_matrix()
    .expect("finite representable stiffness terms");

    assert_relative_close(stiffness[1][1], 4.0e-150);
    assert_relative_close(stiffness[1][3], 2.0e-150);
    assert_relative_close(stiffness[3][1], 2.0e-150);
    assert_relative_close(stiffness[3][3], 4.0e-150);
}

#[test]
fn stiffness_matrix_has_no_force_for_rigid_translation() {
    let response = beam()
        .respond([0.03, 0.0, 0.03, 0.0], 0.0)
        .expect("valid rigid translation");

    for value in response.end_shear_n {
        assert_close(value, 0.0);
    }
    for value in response.end_moment_nm {
        assert_close(value, 0.0);
    }
    assert_close(response.strain_energy_j, 0.0);
}

#[test]
fn stiffness_matrix_has_no_force_for_rigid_rotation() {
    let response = beam()
        .respond([0.0, 0.002, 0.004, 0.002], 0.0)
        .expect("valid rigid rotation");

    for value in response.end_shear_n {
        assert_close(value, 0.0);
    }
    for value in response.end_moment_nm {
        assert_close(value, 0.0);
    }
    assert_close(response.strain_energy_j, 0.0);
}

#[test]
fn zero_displacement_has_zero_bending_response_and_energy() {
    let response = beam()
        .respond([0.0; 4], 25_000_000.0)
        .expect("valid zero displacement");

    assert_eq!(response.end_shear_n, [0.0; 2]);
    assert_eq!(response.end_moment_nm, [0.0; 2]);
    assert_eq!(response.maximum_bending_stress_pa, 0.0);
    assert_eq!(response.strain_energy_j, 0.0);
    assert_eq!(response.maximum_normal_stress_pa, 25_000_000.0);
}

#[test]
fn non_rigid_displacement_matches_hand_calculated_force_stress_and_energy() {
    let response = beam()
        .respond([0.0, 0.0, 0.004, 0.0], -30_000_000.0)
        .expect("valid non-rigid displacement");

    assert_close(response.end_shear_n[0], -60_000.0);
    assert_close(response.end_shear_n[1], 60_000.0);
    assert_close(response.end_moment_nm[0], -60_000.0);
    assert_close(response.end_moment_nm[1], -60_000.0);
    assert_close(response.maximum_bending_stress_pa, 120_000_000.0);
    assert_close(response.maximum_normal_stress_pa, 150_000_000.0);
    assert_close(response.strain_energy_j, 120.0);
}

#[test]
fn valid_beam_response_contains_only_finite_outputs() {
    let response = beam()
        .respond([0.0, 0.001, 0.004, -0.001], 15_000_000.0)
        .expect("ordinary finite input");

    assert!(response.end_shear_n.iter().all(|value| value.is_finite()));
    assert!(response.end_moment_nm.iter().all(|value| value.is_finite()));
    assert!(response.maximum_bending_stress_pa.is_finite());
    assert!(response.maximum_normal_stress_pa.is_finite());
    assert!(response.strain_energy_j.is_finite());
}

#[test]
fn beam_rejects_invalid_and_non_finite_properties() {
    let cases = [
        BeamElement {
            length_m: 0.0,
            ..beam()
        },
        BeamElement {
            youngs_modulus_pa: f64::NAN,
            ..beam()
        },
        BeamElement {
            second_moment_m4: f64::INFINITY,
            ..beam()
        },
        BeamElement {
            outer_radius_m: -0.1,
            ..beam()
        },
    ];

    for invalid in cases {
        assert!(invalid.stiffness_matrix().is_err());
    }
}

#[test]
fn beam_response_rejects_non_finite_displacement_and_axial_stress() {
    assert!(beam().respond([0.0, f64::NAN, 0.0, 0.0], 0.0).is_err());
    assert!(beam().respond([0.0; 4], f64::NEG_INFINITY).is_err());
}

#[test]
fn stiffness_rejects_finite_properties_that_overflow_the_matrix() {
    let element = BeamElement {
        length_m: 1.0,
        youngs_modulus_pa: f64::MAX,
        second_moment_m4: f64::MAX,
        outer_radius_m: 0.1,
    };

    assert!(element.stiffness_matrix().is_err());
}

#[test]
fn response_rejects_finite_inputs_that_overflow_reported_stress() {
    let element = BeamElement {
        length_m: 1.0,
        youngs_modulus_pa: 200_000_000_000.0,
        second_moment_m4: 0.000_1,
        outer_radius_m: 0.1,
    };

    assert!(element.respond([0.0, 0.0, f64::MAX, 0.0], 0.0).is_err());
}
