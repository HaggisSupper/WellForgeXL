//! Data-driven ISCWSA Rev 5.13 covariance-propagation kernel.
//! Toolcode coefficients are loaded from versioned JSON; no UI layer may alter
//! term weights, correlation groups, or covariance arithmetic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wellforge_core::Metres;
use wellforge_survey::{Matrix3, SurveyStation, transform_covariance_nev_to_hla};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCode {
    pub id: String,
    pub revision: String,
    pub terms: Vec<ErrorTerm>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorTerm {
    pub code: String,
    pub sigma_m: f64,
    pub correlation_group: String,
    pub weighting: WeightingFunction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WeightingFunction {
    Constant,
    LinearMd {
        #[serde(rename = "referenceMdM")]
        reference_md_m: f64,
    },
    SinInclination,
    CosInclination,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CovarianceStation {
    pub md_m: Metres,
    pub nev: Matrix3,
    pub hla: Matrix3,
    pub eou_radii_m: [f64; 3],
}

#[derive(Debug, Error)]
pub enum IscwsaError {
    #[error("toolcode JSON is invalid: {0}")]
    ToolCodeJson(#[from] serde_json::Error),
    #[error("toolcode terms must have finite non-negative sigma values")]
    InvalidToolCode,
    #[error("survey stations must be ordered and non-empty")]
    InvalidSurvey,
    #[error("covariance propagation produced a non-finite value")]
    NonFiniteCovariance,
}

pub fn load_toolcode_json(json: &str) -> Result<ToolCode, IscwsaError> {
    let toolcode: ToolCode = serde_json::from_str(json)?;
    validate_toolcode(&toolcode)?;
    Ok(toolcode)
}

/// Propagates a conservative NEV covariance using independent term vectors.
/// Terms sharing a correlation group are summed before their outer product is
/// added, preserving fully correlated effects within a group.
pub fn propagate_covariance(
    stations: &[SurveyStation],
    toolcode: &ToolCode,
) -> Result<Vec<CovarianceStation>, IscwsaError> {
    validate_toolcode(toolcode)?;
    if stations.is_empty() || stations.windows(2).any(|pair| pair[1].md_m <= pair[0].md_m) {
        return Err(IscwsaError::InvalidSurvey);
    }
    let mut results = Vec::with_capacity(stations.len());
    for station in stations {
        let mut covariance = Matrix3::new([[0.0; 3]; 3]);
        let mut group_vectors: BTreeMap<&str, [f64; 3]> = BTreeMap::new();
        for term in &toolcode.terms {
            let sigma = term.sigma_m * weight(&term.weighting, station);
            let direction = [
                station.inclination_rad.get().sin() * station.azimuth_true_rad.get().cos(),
                station.inclination_rad.get().sin() * station.azimuth_true_rad.get().sin(),
                station.inclination_rad.get().cos(),
            ];
            let group_vector = group_vectors
                .entry(term.correlation_group.as_str())
                .or_insert([0.0; 3]);
            for index in 0..3 {
                group_vector[index] += sigma * direction[index];
            }
        }
        for group_vector in group_vectors.values() {
            for row in 0..3 {
                for col in 0..3 {
                    covariance.rows[row][col] += group_vector[row] * group_vector[col];
                }
            }
        }
        if !matrix_is_finite(covariance) {
            return Err(IscwsaError::NonFiniteCovariance);
        }
        let hla = transform_covariance_nev_to_hla(
            covariance,
            station.inclination_rad.get(),
            station.azimuth_true_rad.get(),
        )
        .map_err(|_| IscwsaError::InvalidSurvey)?;
        if !matrix_is_finite(hla) {
            return Err(IscwsaError::NonFiniteCovariance);
        }
        let eou_radii_m = [
            covariance.rows[0][0].max(0.0).sqrt(),
            covariance.rows[1][1].max(0.0).sqrt(),
            covariance.rows[2][2].max(0.0).sqrt(),
        ];
        if eou_radii_m.iter().any(|radius| !radius.is_finite()) {
            return Err(IscwsaError::NonFiniteCovariance);
        }
        results.push(CovarianceStation {
            md_m: station.md_m,
            nev: covariance,
            hla,
            eou_radii_m,
        });
    }
    Ok(results)
}

fn validate_toolcode(toolcode: &ToolCode) -> Result<(), IscwsaError> {
    if toolcode.id.trim().is_empty()
        || toolcode.revision.trim().is_empty()
        || toolcode.terms.iter().any(|term| {
            !term.sigma_m.is_finite() || term.sigma_m < 0.0 || !weighting_is_valid(&term.weighting)
        })
    {
        Err(IscwsaError::InvalidToolCode)
    } else {
        Ok(())
    }
}

fn weighting_is_valid(weighting: &WeightingFunction) -> bool {
    match weighting {
        WeightingFunction::LinearMd { reference_md_m } => {
            reference_md_m.is_finite() && *reference_md_m > 0.0
        }
        WeightingFunction::Constant
        | WeightingFunction::SinInclination
        | WeightingFunction::CosInclination => true,
    }
}

fn matrix_is_finite(matrix: Matrix3) -> bool {
    matrix.rows.iter().flatten().all(|value| value.is_finite())
}

fn weight(weighting: &WeightingFunction, station: &SurveyStation) -> f64 {
    match weighting {
        WeightingFunction::Constant => 1.0,
        WeightingFunction::LinearMd { reference_md_m } => station.md_m.get() / reference_md_m,
        WeightingFunction::SinInclination => station.inclination_rad.get().sin().abs(),
        WeightingFunction::CosInclination => station.inclination_rad.get().cos().abs(),
    }
}
