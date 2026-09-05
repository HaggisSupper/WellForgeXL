//! SI-native pressure-loss correlations.

use wellforge_hydraulics_contract::{
    FlowCorrelation, FlowLoop, FlowRegime, RheologyModel, RheologyParameters,
};

use crate::SolveError;

const LAMINAR_FANNING_NUMERATOR: f64 = 16.0;
const LAMINAR_BLEND_EXPONENT: f64 = 12.0;
const TRANSITION_BLEND_EXPONENT: f64 = 8.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct FlowResponse {
    pub(crate) reynolds_number: f64,
    pub(crate) fanning_friction_factor: f64,
    pub(crate) flow_regime: FlowRegime,
}

#[derive(Clone, Copy, Debug)]
struct YieldPowerLawParameters {
    yield_stress_pa: f64,
    consistency_pa_s_n: f64,
    flow_index: f64,
    high_shear_flow_index: f64,
}

pub(crate) fn evaluate_flow_response(
    correlation: FlowCorrelation,
    rheology: &RheologyParameters,
    density_kg_m3: f64,
    velocity_m_s: f64,
    hydraulic_diameter_m: f64,
    flow_loop: FlowLoop,
) -> Result<FlowResponse, SolveError> {
    if velocity_m_s <= 0.0 || hydraulic_diameter_m <= 0.0 {
        return Ok(FlowResponse {
            reynolds_number: 0.0,
            fanning_friction_factor: 0.0,
            flow_regime: FlowRegime::Laminar,
        });
    }

    match correlation {
        FlowCorrelation::DarcyWeisbachScreening => {
            screening_response(rheology, density_kg_m3, velocity_m_s, hydraulic_diameter_m)
        }
        FlowCorrelation::GeneralizedYieldPowerLaw => generalized_response(
            rheology,
            density_kg_m3,
            velocity_m_s,
            hydraulic_diameter_m,
            flow_loop,
        ),
    }
}

fn screening_response(
    rheology: &RheologyParameters,
    density_kg_m3: f64,
    velocity_m_s: f64,
    hydraulic_diameter_m: f64,
) -> Result<FlowResponse, SolveError> {
    let wall_shear_rate_s = 8.0 * velocity_m_s / hydraulic_diameter_m;
    let apparent_viscosity_pa_s = match rheology.model {
        RheologyModel::Newtonian => rheology
            .dynamic_viscosity_pa_s
            .ok_or(SolveError::Rheology)?,
        RheologyModel::Bingham => {
            let plastic_viscosity = rheology
                .plastic_viscosity_pa_s
                .ok_or(SolveError::Rheology)?;
            let yield_stress = rheology.yield_stress_pa.ok_or(SolveError::Rheology)?;
            plastic_viscosity + yield_stress / wall_shear_rate_s
        }
        RheologyModel::PowerLaw => {
            let consistency = rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?;
            let flow_index = rheology.flow_behavior_index.ok_or(SolveError::Rheology)?;
            consistency * wall_shear_rate_s.powf(flow_index - 1.0)
        }
        RheologyModel::HerschelBulkley => {
            let consistency = rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?;
            let flow_index = rheology.flow_behavior_index.ok_or(SolveError::Rheology)?;
            let yield_stress = rheology.yield_stress_pa.ok_or(SolveError::Rheology)?;
            (yield_stress + consistency * wall_shear_rate_s.powf(flow_index)) / wall_shear_rate_s
        }
    };

    if !apparent_viscosity_pa_s.is_finite() || apparent_viscosity_pa_s <= 0.0 {
        return Err(SolveError::Numerical);
    }

    let reynolds_number =
        density_kg_m3 * velocity_m_s * hydraulic_diameter_m / apparent_viscosity_pa_s;
    let fanning_friction_factor = if reynolds_number < 2100.0 {
        LAMINAR_FANNING_NUMERATOR / reynolds_number.max(1.0)
    } else {
        0.0791 * reynolds_number.powf(-0.25)
    };
    let flow_regime = if reynolds_number < 2100.0 {
        FlowRegime::Laminar
    } else if reynolds_number < 4000.0 {
        FlowRegime::Transitional
    } else {
        FlowRegime::Turbulent
    };

    Ok(FlowResponse {
        reynolds_number,
        fanning_friction_factor,
        flow_regime,
    })
}

fn generalized_response(
    rheology: &RheologyParameters,
    density_kg_m3: f64,
    velocity_m_s: f64,
    hydraulic_diameter_m: f64,
    flow_loop: FlowLoop,
) -> Result<FlowResponse, SolveError> {
    let parameters = normalized_rheology(rheology)?;
    let geometry_code: f64 = match flow_loop {
        FlowLoop::Pipe => 0.0,
        FlowLoop::Annulus => 1.0,
    };
    let numerator = (3.0 - geometry_code).mul_add(parameters.flow_index, 1.0);
    let denominator = (4.0 - geometry_code) * parameters.flow_index;
    let geometry_factor = numerator / denominator * geometry_code.mul_add(0.5, 1.0);
    let wall_shear_rate_s = 8.0 * geometry_factor * velocity_m_s / hydraulic_diameter_m;
    let yield_geometry_factor =
        ((4.0 - geometry_code) / (3.0 - geometry_code)).powf(parameters.flow_index);
    let adjusted_yield_stress_pa = yield_geometry_factor * parameters.yield_stress_pa;
    let wall_shear_stress_pa = adjusted_yield_stress_pa
        + parameters.consistency_pa_s_n * wall_shear_rate_s.powf(parameters.flow_index);

    if !wall_shear_stress_pa.is_finite() || wall_shear_stress_pa <= 0.0 {
        return Err(SolveError::Numerical);
    }

    // The factor eight makes the n=1, zero-yield pipe limit equal conventional Reynolds.
    let reynolds_number = 8.0 * density_kg_m3 * velocity_m_s.powi(2) / wall_shear_stress_pa;
    let lower_transition = 3470.0 - 1370.0 * parameters.flow_index;
    let upper_transition = 4270.0 - 1370.0 * parameters.flow_index;
    let flow_regime = if reynolds_number < lower_transition {
        FlowRegime::Laminar
    } else if reynolds_number < upper_transition {
        FlowRegime::Transitional
    } else {
        FlowRegime::Turbulent
    };

    let fanning_friction_factor = blended_fanning_factor(
        reynolds_number,
        parameters.flow_index,
        parameters.high_shear_flow_index,
    );

    if !reynolds_number.is_finite()
        || !fanning_friction_factor.is_finite()
        || fanning_friction_factor <= 0.0
    {
        return Err(SolveError::Numerical);
    }

    Ok(FlowResponse {
        reynolds_number,
        fanning_friction_factor,
        flow_regime,
    })
}

fn normalized_rheology(
    rheology: &RheologyParameters,
) -> Result<YieldPowerLawParameters, SolveError> {
    let parameters = match rheology.model {
        RheologyModel::Newtonian => YieldPowerLawParameters {
            yield_stress_pa: 0.0,
            consistency_pa_s_n: rheology
                .dynamic_viscosity_pa_s
                .ok_or(SolveError::Rheology)?,
            flow_index: 1.0,
            high_shear_flow_index: 1.0,
        },
        RheologyModel::Bingham => YieldPowerLawParameters {
            yield_stress_pa: rheology.yield_stress_pa.ok_or(SolveError::Rheology)?,
            consistency_pa_s_n: rheology
                .plastic_viscosity_pa_s
                .ok_or(SolveError::Rheology)?,
            flow_index: 1.0,
            high_shear_flow_index: 1.0,
        },
        RheologyModel::PowerLaw => YieldPowerLawParameters {
            yield_stress_pa: 0.0,
            consistency_pa_s_n: rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?,
            flow_index: rheology.flow_behavior_index.ok_or(SolveError::Rheology)?,
            high_shear_flow_index: rheology.high_shear_flow_index.ok_or(SolveError::Rheology)?,
        },
        RheologyModel::HerschelBulkley => YieldPowerLawParameters {
            yield_stress_pa: rheology.yield_stress_pa.ok_or(SolveError::Rheology)?,
            consistency_pa_s_n: rheology.consistency_k_pa_s_n.ok_or(SolveError::Rheology)?,
            flow_index: rheology.flow_behavior_index.ok_or(SolveError::Rheology)?,
            high_shear_flow_index: rheology.high_shear_flow_index.ok_or(SolveError::Rheology)?,
        },
    };

    if !parameters.yield_stress_pa.is_finite()
        || parameters.yield_stress_pa < 0.0
        || !parameters.consistency_pa_s_n.is_finite()
        || parameters.consistency_pa_s_n <= 0.0
        || !parameters.flow_index.is_finite()
        || parameters.flow_index <= 0.0
        || !parameters.high_shear_flow_index.is_finite()
        || parameters.high_shear_flow_index <= 0.0
    {
        return Err(SolveError::Rheology);
    }

    Ok(parameters)
}

fn blended_fanning_factor(
    reynolds_number: f64,
    flow_index: f64,
    high_shear_flow_index: f64,
) -> f64 {
    let lower_transition = 3470.0 - 1370.0 * flow_index;
    let laminar_factor = LAMINAR_FANNING_NUMERATOR / reynolds_number.max(f64::MIN_POSITIVE);
    let transition_factor = LAMINAR_FANNING_NUMERATOR * reynolds_number / lower_transition.powi(2);
    let logarithmic_index = high_shear_flow_index.log10();
    let turbulent_scale = (logarithmic_index + 3.93) / 50.0;
    let turbulent_exponent = (1.75 - logarithmic_index) / 7.0;
    let turbulent_factor = turbulent_scale / reynolds_number.powf(turbulent_exponent);
    let transition_turbulent = smooth_minimum(
        transition_factor,
        turbulent_factor,
        TRANSITION_BLEND_EXPONENT,
    );
    smooth_maximum(transition_turbulent, laminar_factor, LAMINAR_BLEND_EXPONENT)
}

fn smooth_minimum(left: f64, right: f64, exponent: f64) -> f64 {
    let scale = left.min(right);
    scale * ((scale / left).powf(exponent) + (scale / right).powf(exponent)).powf(-1.0 / exponent)
}

fn smooth_maximum(left: f64, right: f64, exponent: f64) -> f64 {
    let scale = left.max(right);
    scale * ((left / scale).powf(exponent) + (right / scale).powf(exponent)).powf(1.0 / exponent)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn newtonian(viscosity_pa_s: f64) -> RheologyParameters {
        RheologyParameters {
            model: RheologyModel::Newtonian,
            dynamic_viscosity_pa_s: Some(viscosity_pa_s),
            yield_stress_pa: None,
            plastic_viscosity_pa_s: None,
            consistency_k_pa_s_n: None,
            flow_behavior_index: None,
            high_shear_flow_index: None,
        }
    }

    fn yield_power_law(
        yield_stress_pa: f64,
        consistency_pa_s_n: f64,
        flow_index: f64,
    ) -> RheologyParameters {
        RheologyParameters {
            model: RheologyModel::HerschelBulkley,
            dynamic_viscosity_pa_s: None,
            yield_stress_pa: Some(yield_stress_pa),
            plastic_viscosity_pa_s: None,
            consistency_k_pa_s_n: Some(consistency_pa_s_n),
            flow_behavior_index: Some(flow_index),
            high_shear_flow_index: Some(flow_index),
        }
    }

    #[test]
    fn newtonian_pipe_matches_si_oracle() {
        let response = generalized_response(&newtonian(0.01), 1000.0, 1.0, 0.1, FlowLoop::Pipe)
            .expect("valid response");

        assert!((response.reynolds_number - 10_000.0).abs() <= 1.0e-9);
        assert!((response.fanning_friction_factor - 0.007_859_995_236).abs() <= 1.0e-12);
        let pressure_gradient_pa_m = 2.0 * response.fanning_friction_factor * 1000.0 / 0.1;
        assert!((pressure_gradient_pa_m - 157.199_904_72).abs() <= 1.0e-8);
    }

    #[test]
    fn newtonian_annulus_matches_si_oracle() {
        let response = generalized_response(&newtonian(0.01), 1000.0, 1.0, 0.1, FlowLoop::Annulus)
            .expect("valid response");

        assert!((response.reynolds_number - 6_666.666_666_667).abs() <= 1.0e-9);
        assert!((response.fanning_friction_factor - 0.008_698_215_851).abs() <= 1.0e-12);
        let pressure_gradient_pa_m = 2.0 * response.fanning_friction_factor * 1000.0 / 0.1;
        assert!((pressure_gradient_pa_m - 173.964_317_02).abs() <= 1.0e-8);
    }

    #[test]
    fn generalized_transition_matches_power_law_oracle() {
        let rheology = yield_power_law(0.0, 0.175_366_512_91, 0.6);
        let response = generalized_response(&rheology, 1000.0, 1.0, 0.1, FlowLoop::Pipe)
            .expect("valid response");

        assert!((response.reynolds_number - 3000.0).abs() <= 1.0e-6);
        assert_eq!(response.flow_regime, FlowRegime::Transitional);
        assert!((response.fanning_friction_factor - 0.006_628_824_637).abs() <= 1.0e-12);
    }

    #[test]
    fn yield_power_law_geometry_matches_wall_response_oracle() {
        let rheology = yield_power_law(5.0, 0.2, 0.6);
        let pipe = generalized_response(&rheology, 1200.0, 1.5, 0.1, FlowLoop::Pipe)
            .expect("pipe response");
        let annulus = generalized_response(&rheology, 1200.0, 1.5, 0.1, FlowLoop::Annulus)
            .expect("annulus response");

        assert!((pipe.reynolds_number - 2199.393).abs() <= 1.0e-3);
        assert!((annulus.reynolds_number - 1884.097).abs() <= 1.0e-3);
    }

    #[test]
    fn bingham_pressure_gradient_has_finite_zero_flow_limit() {
        let rheology = RheologyParameters {
            model: RheologyModel::Bingham,
            dynamic_viscosity_pa_s: None,
            yield_stress_pa: Some(5.0),
            plastic_viscosity_pa_s: Some(0.02),
            consistency_k_pa_s_n: None,
            flow_behavior_index: None,
            high_shear_flow_index: None,
        };
        let velocity_m_s = 1.0e-9;
        let pipe = generalized_response(&rheology, 1200.0, velocity_m_s, 0.1, FlowLoop::Pipe)
            .expect("pipe response");
        let annulus = generalized_response(&rheology, 1200.0, velocity_m_s, 0.1, FlowLoop::Annulus)
            .expect("annulus response");
        let pipe_gradient =
            2.0 * pipe.fanning_friction_factor * 1200.0 * velocity_m_s.powi(2) / 0.1;
        let annulus_gradient =
            2.0 * annulus.fanning_friction_factor * 1200.0 * velocity_m_s.powi(2) / 0.1;

        assert!((pipe_gradient - 266.666_666_667).abs() <= 1.0e-5);
        assert!((annulus_gradient - 300.0).abs() <= 1.0e-5);
    }

    #[test]
    fn regime_blend_matches_boundary_oracles() {
        let lower = blended_fanning_factor(2648.0, 0.6, 0.6);
        let upper = blended_fanning_factor(3448.0, 0.6, 0.6);

        assert!((lower - 0.006_364_783_603).abs() <= 1.0e-12);
        assert!((upper - 0.007_018_857_688).abs() <= 1.0e-12);
        for boundary in [2648.0, 3448.0] {
            let below = blended_fanning_factor(boundary * (1.0 - 1.0e-9), 0.6, 0.6);
            let above = blended_fanning_factor(boundary * (1.0 + 1.0e-9), 0.6, 0.6);
            assert!((above - below).abs() <= 1.0e-9);
        }
    }
}
