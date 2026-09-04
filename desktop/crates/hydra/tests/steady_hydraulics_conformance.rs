use wellforge_hydra::{FlowSection, Fluid, HydraulicsError, calculate_steady_hydraulics};

const REFERENCE_SECTION: FlowSection = FlowSection {
    length_m: 100.0,
    hydraulic_diameter_m: 0.1,
    flow_area_m2: 0.01,
    friction_factor: 0.02,
};

const REFERENCE_FLUID: Fluid = Fluid {
    density_kg_m3: 1_000.0,
    viscosity_pa_s: 0.001,
};

const REFERENCE_FLOW_RATE_M3_S: f64 = 0.02;
const REFERENCE_TRUE_VERTICAL_DEPTH_M: f64 = 1_000.0;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, got {actual}, tolerance {tolerance}"
    );
}

fn assert_invalid(
    section: FlowSection,
    fluid: Fluid,
    flow_rate_m3_s: f64,
    true_vertical_depth_m: f64,
) {
    assert!(matches!(
        calculate_steady_hydraulics(section, fluid, flow_rate_m3_s, true_vertical_depth_m),
        Err(HydraulicsError::InvalidInput)
    ));
}

#[test]
fn reference_case_matches_hand_calculated_darcy_weisbach_and_ecd_values() {
    let result = calculate_steady_hydraulics(
        REFERENCE_SECTION,
        REFERENCE_FLUID,
        REFERENCE_FLOW_RATE_M3_S,
        REFERENCE_TRUE_VERTICAL_DEPTH_M,
    )
    .expect("reference inputs are valid");

    assert_close(result.velocity_m_s, 2.0, 1e-12);
    assert_close(result.pressure_loss_pa, 40_000.0, 1e-9);
    assert_close(result.ecd_kg_m3, 1_004.078_864_851_911_7, 1e-9);
}

#[test]
fn zero_flow_has_zero_velocity_zero_loss_and_base_density_ecd() {
    let result = calculate_steady_hydraulics(
        REFERENCE_SECTION,
        REFERENCE_FLUID,
        0.0,
        REFERENCE_TRUE_VERTICAL_DEPTH_M,
    )
    .expect("zero flow is within the steady calculation domain");

    assert_close(result.velocity_m_s, 0.0, 0.0);
    assert_close(result.pressure_loss_pa, 0.0, 0.0);
    assert_close(result.ecd_kg_m3, REFERENCE_FLUID.density_kg_m3, 0.0);
}

#[test]
fn doubled_flow_doubles_velocity_and_quadruples_pressure_loss() {
    let baseline = calculate_steady_hydraulics(
        REFERENCE_SECTION,
        REFERENCE_FLUID,
        REFERENCE_FLOW_RATE_M3_S,
        REFERENCE_TRUE_VERTICAL_DEPTH_M,
    )
    .expect("baseline inputs are valid");
    let doubled = calculate_steady_hydraulics(
        REFERENCE_SECTION,
        REFERENCE_FLUID,
        REFERENCE_FLOW_RATE_M3_S * 2.0,
        REFERENCE_TRUE_VERTICAL_DEPTH_M,
    )
    .expect("doubled-flow inputs are valid");

    assert_close(doubled.velocity_m_s, baseline.velocity_m_s * 2.0, 1e-12);
    assert_close(
        doubled.pressure_loss_pa,
        baseline.pressure_loss_pa * 4.0,
        1e-8,
    );
}

#[test]
fn rejects_invalid_boundaries_for_every_input_field() {
    let invalid_cases = [
        (
            FlowSection {
                length_m: -0.01,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            FlowSection {
                hydraulic_diameter_m: 0.0,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            FlowSection {
                flow_area_m2: 0.0,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            FlowSection {
                friction_factor: -0.01,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            REFERENCE_SECTION,
            Fluid {
                density_kg_m3: 0.0,
                ..REFERENCE_FLUID
            },
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            REFERENCE_SECTION,
            Fluid {
                viscosity_pa_s: 0.0,
                ..REFERENCE_FLUID
            },
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            REFERENCE_SECTION,
            REFERENCE_FLUID,
            -0.01,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        ),
        (
            REFERENCE_SECTION,
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            0.0,
        ),
    ];

    for (section, fluid, flow_rate_m3_s, true_vertical_depth_m) in invalid_cases {
        assert_invalid(section, fluid, flow_rate_m3_s, true_vertical_depth_m);
    }
}

#[test]
fn rejects_nan_and_both_infinities_for_every_input_field() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_invalid(
            FlowSection {
                length_m: value,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            FlowSection {
                hydraulic_diameter_m: value,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            FlowSection {
                flow_area_m2: value,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            FlowSection {
                friction_factor: value,
                ..REFERENCE_SECTION
            },
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            REFERENCE_SECTION,
            Fluid {
                density_kg_m3: value,
                ..REFERENCE_FLUID
            },
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            REFERENCE_SECTION,
            Fluid {
                viscosity_pa_s: value,
                ..REFERENCE_FLUID
            },
            REFERENCE_FLOW_RATE_M3_S,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            REFERENCE_SECTION,
            REFERENCE_FLUID,
            value,
            REFERENCE_TRUE_VERTICAL_DEPTH_M,
        );
        assert_invalid(
            REFERENCE_SECTION,
            REFERENCE_FLUID,
            REFERENCE_FLOW_RATE_M3_S,
            value,
        );
    }
}

#[test]
fn rejects_finite_inputs_that_overflow_the_steady_calculation() {
    let result = calculate_steady_hydraulics(
        FlowSection {
            length_m: f64::MAX,
            hydraulic_diameter_m: 1.0,
            flow_area_m2: 1.0,
            friction_factor: 1.0,
        },
        Fluid {
            density_kg_m3: f64::MAX,
            viscosity_pa_s: 0.001,
        },
        1.0,
        1.0,
    );

    assert!(matches!(result, Err(HydraulicsError::InvalidInput)));
}

#[test]
fn valid_reference_output_is_finite_and_physically_meaningful() {
    let result = calculate_steady_hydraulics(
        REFERENCE_SECTION,
        REFERENCE_FLUID,
        REFERENCE_FLOW_RATE_M3_S,
        REFERENCE_TRUE_VERTICAL_DEPTH_M,
    )
    .expect("reference inputs are valid");

    assert!(result.velocity_m_s.is_finite() && result.velocity_m_s > 0.0);
    assert!(result.pressure_loss_pa.is_finite() && result.pressure_loss_pa > 0.0);
    assert!(result.ecd_kg_m3.is_finite() && result.ecd_kg_m3 > REFERENCE_FLUID.density_kg_m3);
}
