use wellforge_core::{Metres, Radians};
use wellforge_iscwsa::{
    CovarianceStation, ErrorTerm, IscwsaError, ToolCode, WeightingFunction, load_toolcode_json,
    propagate_covariance,
};
use wellforge_survey::{Matrix3, SurveyStation};

fn metres(value: f64) -> Metres {
    Metres::try_new(value).expect("test value is finite")
}

fn radians(value: f64) -> Radians {
    Radians::try_new(value).expect("test value is finite")
}

fn station(md_m: f64) -> SurveyStation {
    SurveyStation::new(metres(md_m), radians(0.5), radians(1.0))
}

fn constant_toolcode(sigma_m: f64) -> ToolCode {
    ToolCode {
        id: "synthetic".to_owned(),
        revision: "v1".to_owned(),
        terms: vec![ErrorTerm {
            code: "constant".to_owned(),
            sigma_m,
            correlation_group: "independent".to_owned(),
            weighting: WeightingFunction::Constant,
        }],
    }
}

#[test]
fn toolcode_accepts_camel_case_wire_fields() {
    let toolcode = load_toolcode_json(
        r#"{
          "id":"synthetic",
          "revision":"v1",
          "terms":[{
            "code":"linear",
            "sigmaM":0.5,
            "correlationGroup":"shared",
            "weighting":{"kind":"linear_md","referenceMdM":100.0}
          }]
        }"#,
    )
    .expect("camel-case toolcode is valid");

    assert!(matches!(
        toolcode.terms[0].weighting,
        WeightingFunction::LinearMd {
            reference_md_m: 100.0
        }
    ));
}

#[test]
fn toolcode_rejects_invalid_numeric_weighting_inputs() {
    for reference_md_m in [0.0, -1.0] {
        let source = format!(
            r#"{{"id":"synthetic","revision":"v1","terms":[{{"code":"linear","sigmaM":0.5,"correlationGroup":"shared","weighting":{{"kind":"linear_md","referenceMdM":{reference_md_m}}}}}]}}"#
        );
        assert!(matches!(
            load_toolcode_json(&source),
            Err(IscwsaError::InvalidToolCode)
        ));
    }
    assert!(load_toolcode_json(
        r#"{"id":"synthetic","revision":"v1","terms":[{"code":"linear","sigmaM":1e999,"correlationGroup":"shared","weighting":{"kind":"constant"}}]}"#
    )
    .is_err());
}

#[test]
fn propagation_revalidates_programmatic_toolcodes() {
    let mut toolcode = constant_toolcode(0.5);
    toolcode.terms[0].weighting = WeightingFunction::LinearMd {
        reference_md_m: f64::NAN,
    };

    assert!(matches!(
        propagate_covariance(&[station(0.0)], &toolcode),
        Err(IscwsaError::InvalidToolCode)
    ));
}

#[test]
fn propagation_rejects_non_finite_derived_covariance() {
    let toolcode = constant_toolcode(f64::MAX);

    assert!(matches!(
        propagate_covariance(&[station(0.0)], &toolcode),
        Err(IscwsaError::NonFiniteCovariance)
    ));
}

#[test]
fn propagation_sums_matching_correlation_groups_before_the_outer_product() {
    let grouped = ToolCode {
        id: "synthetic".to_owned(),
        revision: "v1".to_owned(),
        terms: vec![
            ErrorTerm {
                code: "first".to_owned(),
                sigma_m: 3.0,
                correlation_group: "shared".to_owned(),
                weighting: WeightingFunction::Constant,
            },
            ErrorTerm {
                code: "second".to_owned(),
                sigma_m: 3.0,
                correlation_group: "shared".to_owned(),
                weighting: WeightingFunction::Constant,
            },
        ],
    };
    let independent = ToolCode {
        terms: vec![
            ErrorTerm {
                correlation_group: "left".to_owned(),
                ..grouped.terms[0].clone()
            },
            ErrorTerm {
                correlation_group: "right".to_owned(),
                ..grouped.terms[1].clone()
            },
        ],
        ..grouped.clone()
    };
    let horizontal_north = SurveyStation::new(
        metres(10.0),
        radians(std::f64::consts::FRAC_PI_2),
        radians(0.0),
    );

    let grouped_result = propagate_covariance(&[horizontal_north], &grouped)
        .expect("valid grouped toolcode propagates");
    let independent_result = propagate_covariance(&[horizontal_north], &independent)
        .expect("valid independent toolcode propagates");

    assert_eq!(grouped_result[0].nev.rows[0][0], 36.0);
    assert_eq!(independent_result[0].nev.rows[0][0], 18.0);
}

#[test]
fn covariance_station_uses_explicit_camel_case_wire_fields() {
    let station = CovarianceStation {
        md_m: metres(10.0),
        nev: Matrix3::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
        hla: Matrix3::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
        eou_radii_m: [1.0, 2.0, 3.0],
    };

    let value = serde_json::to_value(&station).expect("station serializes");
    assert_eq!(value["mdM"], 10.0);
    assert_eq!(value["eouRadiiM"], serde_json::json!([1.0, 2.0, 3.0]));
    assert!(value.get("md_m").is_none());
    assert!(serde_json::from_value::<CovarianceStation>(value).is_ok());
}
