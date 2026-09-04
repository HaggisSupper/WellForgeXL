//! Anti-collision geometry and risk contracts. This crate consumes survey
//! positions and uncertainty envelopes; it never owns survey calculation.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wellforge_core::{PlotAnnotation, PlotPoint, PlotSpec, PlotTrace};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpatialStation {
    pub md_m: f64,
    pub north_m: f64,
    pub east_m: f64,
    pub tvd_m: f64,
    pub eou_radius_m: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ClosestApproach {
    pub reference_md_m: f64,
    pub offset_md_m: f64,
    pub separation_m: f64,
    pub combined_eou_m: f64,
    pub separation_factor: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WarningRules {
    pub no_go_sf: f64,
    pub warning_sf: f64,
}
impl Default for WarningRules {
    fn default() -> Self {
        Self {
            no_go_sf: 1.0,
            warning_sf: 1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum RiskLevel {
    Clear,
    Warning,
    NoGo,
}

#[derive(Debug, Error)]
pub enum AcError {
    #[error("AC input must contain finite non-negative values")]
    InvalidInput,
    #[error("at least one reference and offset station is required")]
    EmptyScan,
}

/// Exhaustive station-pair scan. Later progressive interpolation refines this
/// same contract; it cannot silently replace or understate a reported minimum.
pub fn closest_approach_scan(
    reference: &[SpatialStation],
    offsets: &[SpatialStation],
) -> Result<ClosestApproach, AcError> {
    if reference.is_empty() || offsets.is_empty() {
        return Err(AcError::EmptyScan);
    }
    let mut result: Option<ClosestApproach> = None;
    for left in reference {
        for right in offsets {
            validate(*left)?;
            validate(*right)?;
            let separation_m = ((left.north_m - right.north_m).powi(2)
                + (left.east_m - right.east_m).powi(2)
                + (left.tvd_m - right.tvd_m).powi(2))
            .sqrt();
            let combined_eou_m = (left.eou_radius_m.powi(2) + right.eou_radius_m.powi(2)).sqrt();
            let candidate = ClosestApproach {
                reference_md_m: left.md_m,
                offset_md_m: right.md_m,
                separation_m,
                combined_eou_m,
                separation_factor: if combined_eou_m == 0.0 {
                    f64::INFINITY
                } else {
                    separation_m / combined_eou_m
                },
            };
            if result
                .as_ref()
                .is_none_or(|current| candidate.separation_factor < current.separation_factor)
            {
                result = Some(candidate);
            }
        }
    }
    Ok(result.expect("non-empty validated scan has a result"))
}

pub fn classify_risk(
    approach: &ClosestApproach,
    rules: WarningRules,
) -> Result<RiskLevel, AcError> {
    if !rules.no_go_sf.is_finite()
        || !rules.warning_sf.is_finite()
        || rules.no_go_sf <= 0.0
        || rules.warning_sf < rules.no_go_sf
    {
        return Err(AcError::InvalidInput);
    }
    Ok(if approach.separation_factor <= rules.no_go_sf {
        RiskLevel::NoGo
    } else if approach.separation_factor <= rules.warning_sf {
        RiskLevel::Warning
    } else {
        RiskLevel::Clear
    })
}

pub fn separation_factor_plot(
    approaches: &[ClosestApproach],
    rules: WarningRules,
) -> Result<PlotSpec, AcError> {
    if approaches
        .iter()
        .any(|a| !a.reference_md_m.is_finite() || !a.separation_factor.is_finite())
    {
        return Err(AcError::InvalidInput);
    }
    Ok(PlotSpec {
        title: "Anti-collision separation factor".into(),
        traces: vec![PlotTrace {
            id: "sf".into(),
            name: "SF".into(),
            layer: "risk".into(),
            points: approaches
                .iter()
                .map(|a| PlotPoint {
                    x: a.reference_md_m,
                    y: a.separation_factor,
                    z: None,
                })
                .collect(),
        }],
        bands: vec![],
        annotations: vec![PlotAnnotation {
            text: format!("Warning SF {:.2}", rules.warning_sf),
            point: PlotPoint {
                x: 0.0,
                y: rules.warning_sf,
                z: None,
            },
        }],
    })
}

fn validate(station: SpatialStation) -> Result<(), AcError> {
    if [
        station.md_m,
        station.north_m,
        station.east_m,
        station.tvd_m,
        station.eou_radius_m,
    ]
    .iter()
    .all(|v| v.is_finite())
        && station.md_m >= 0.0
        && station.eou_radius_m >= 0.0
    {
        Ok(())
    } else {
        Err(AcError::InvalidInput)
    }
}
