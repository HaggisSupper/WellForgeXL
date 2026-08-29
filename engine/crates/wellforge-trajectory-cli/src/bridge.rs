//! Bounded, versioned UTF-8 table bridge for VBA consumption.

use std::fmt::Write;

use serde::Serialize;
use wellforge_trajectory_contract::{CalculatedStation, TrajectoryAnalysisResult};

const MAX_PLAN: usize = 500;
const MAX_SURVEY: usize = 500;
const MAX_TARGETS: usize = 100;
const MAX_SLIDES: usize = 200;
const MAX_FORMATIONS: usize = 100;

fn enum_text<T: Serialize>(value: &T) -> Result<String, String> {
    let encoded = serde_json::to_string(value).map_err(|error| error.to_string())?;
    let decoded = encoded
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(&encoded)
        .to_owned();
    validate_text(&decoded)?;
    Ok(decoded)
}

fn text(value: &str) -> Result<String, String> {
    validate_text(value)?;
    Ok(value.to_owned())
}

fn validate_text(value: &str) -> Result<(), String> {
    if value.contains(['\t', '\r', '\n', '\0']) {
        Err("bridge strings cannot contain tabs, newlines, or NUL bytes".to_owned())
    } else {
        Ok(())
    }
}

fn number(value: f64) -> Result<String, String> {
    if value.is_finite() {
        Ok(format!("{value:.17e}"))
    } else {
        Err("bridge numeric fields must be finite".to_owned())
    }
}

fn optional_number(value: Option<f64>) -> Result<String, String> {
    value.map_or_else(|| Ok(String::new()), number)
}

fn optional_bool(value: Option<bool>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn station_fields(station: &CalculatedStation) -> Result<String, String> {
    Ok(format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        station
            .source_uid
            .map_or_else(String::new, |uid| uid.to_string()),
        enum_text(&station.kind)?,
        number(station.md_m)?,
        number(station.inclination_rad)?,
        number(station.azimuth_rad)?,
        number(station.north_m)?,
        number(station.east_m)?,
        number(station.tvd_m)?,
        number(station.delta_md_m)?,
        number(station.dogleg_rad)?,
        number(station.ratio_factor)?,
        number(station.dls_rad_per_m)?,
    ))
}

fn check_capacity(result: &TrajectoryAnalysisResult) -> Result<(), String> {
    let calculation = &result.calculation;
    for (actual, maximum, name) in [
        (calculation.plan.len(), MAX_PLAN, "plan"),
        (calculation.survey.len(), MAX_SURVEY, "survey"),
        (calculation.targets.len(), MAX_TARGETS, "targets"),
        (calculation.slides.len(), MAX_SLIDES, "slides"),
        (calculation.formations.len(), MAX_FORMATIONS, "formations"),
    ] {
        if actual > maximum {
            return Err(format!(
                "bridge {name} capacity exceeded: {actual} > {maximum}"
            ));
        }
    }
    Ok(())
}

fn validate_complete_bridge(output: &str) -> Result<(), String> {
    if !output.ends_with('\n') {
        return Err("bridge must end with one newline".to_owned());
    }
    let mut headers = 0;
    for line in output.lines() {
        if line.contains(['\r', '\0']) {
            return Err("bridge contains a forbidden control character".to_owned());
        }
        let kind = line.split_once('\t').map_or(line, |(kind, _)| kind);
        if kind == "H" {
            headers += 1;
        }
        if !matches!(kind, "H" | "P" | "S" | "R" | "T" | "L" | "F" | "X") {
            return Err(format!("unsupported bridge record kind {kind}"));
        }
    }
    if headers != 1 {
        return Err(format!(
            "bridge requires exactly one header, found {headers}"
        ));
    }
    Ok(())
}

/// Builds and validates the complete bounded bridge before any output file is replaced.
#[allow(clippy::too_many_lines)]
pub(crate) fn build(result: &TrajectoryAnalysisResult) -> Result<Vec<u8>, String> {
    check_capacity(result)?;
    let calculation = &result.calculation;
    let mut output = String::new();
    writeln!(
        output,
        "H\t1.0.0\t{}\t{}\t{}\t{}\t{}\t{}",
        result.analysis_id,
        text(&result.evidence.request_hash)?,
        text(&result.evidence.result_hash)?,
        text(&result.evidence.engine_version)?,
        enum_text(&result.status)?,
        result.applicability.deterministic,
    )
    .expect("writing to a String cannot fail");

    for station in &calculation.plan {
        writeln!(output, "P\t{}", station_fields(station)?)
            .expect("writing to a String cannot fail");
    }
    for station in &calculation.survey {
        writeln!(output, "S\t{}", station_fields(station)?)
            .expect("writing to a String cannot fail");
    }
    for residual in &calculation.plan_survey_residuals {
        let position = residual.residual.as_ref();
        writeln!(
            output,
            "R\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            residual.survey_uid,
            number(residual.md_m)?,
            enum_text(&residual.plan.status)?,
            optional_number(position.map(|value| value.north_m))?,
            optional_number(position.map(|value| value.east_m))?,
            optional_number(position.map(|value| value.tvd_m))?,
            optional_number(position.map(|value| value.along_track_m))?,
            optional_number(position.map(|value| value.crossline_m))?,
            optional_number(position.map(|value| value.horizontal_m))?,
            optional_number(position.map(|value| value.error_3d_m))?,
        )
        .expect("writing to a String cannot fail");
    }
    for target in &calculation.targets {
        let position = target.position.as_ref();
        let evaluation = target.evaluation.as_ref();
        writeln!(
            output,
            "T\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            target.target_uid,
            number(target.md_m)?,
            enum_text(&target.basis)?,
            evaluation.map_or_else(|| Ok(String::new()), |value| enum_text(&value.status))?,
            optional_number(position.map(|value| value.north_m))?,
            optional_number(position.map(|value| value.east_m))?,
            optional_number(position.map(|value| value.tvd_m))?,
            optional_number(evaluation.and_then(|value| value.horizontal_utilization))?,
            optional_number(evaluation.and_then(|value| value.vertical_utilization))?,
            optional_number(evaluation.and_then(|value| value.local_major_m))?,
            optional_number(evaluation.and_then(|value| value.local_minor_m))?,
            optional_number(evaluation.and_then(|value| value.vertical_difference_m))?,
        )
        .expect("writing to a String cannot fail");
    }
    for slide in &calculation.slides {
        let response = slide.response.as_ref();
        writeln!(
            output,
            "L\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            slide.slide_uid,
            enum_text(&slide.start.status)?,
            enum_text(&slide.end.status)?,
            response.map_or_else(|| Ok(String::new()), |value| enum_text(&value.status))?,
            optional_number(response.and_then(|value| value.build_rad_per_m))?,
            optional_number(response.and_then(|value| value.effective_turn_rad_per_m))?,
            optional_number(response.and_then(|value| value.residual_build_rad_per_m))?,
            optional_number(response.and_then(|value| value.residual_turn_rad_per_m))?,
            optional_number(response.and_then(|value| value.yield_rad_per_m))?,
            optional_number(response.and_then(|value| value.response_toolface_rad))?,
            optional_number(response.and_then(|value| value.toolface_error_rad))?,
        )
        .expect("writing to a String cannot fail");
    }
    for formation in &calculation.formations {
        writeln!(
            output,
            "F\t{}\t{}\t{}\t{}\t{}\t{}",
            formation.formation_uid,
            enum_text(&formation.coverage)?,
            optional_number(formation.actual_tvd_m)?,
            optional_number(formation.high_low_m)?,
            formation
                .sense
                .map_or_else(|| Ok(String::new()), |value| enum_text(&value))?,
            optional_bool(formation.within_tolerance),
        )
        .expect("writing to a String cannot fail");
    }
    if let Some(projection) = &calculation.projection {
        writeln!(
            output,
            "X\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            number(projection.bit.md_m)?,
            number(projection.bit.inclination_rad)?,
            number(projection.bit.azimuth_rad)?,
            number(projection.bit.north_m)?,
            number(projection.bit.east_m)?,
            number(projection.bit.tvd_m)?,
            number(projection.projected.md_m)?,
            number(projection.projected.inclination_rad)?,
            number(projection.projected.azimuth_rad)?,
            number(projection.projected.north_m)?,
            number(projection.projected.east_m)?,
            number(projection.projected.tvd_m)?,
            projection.low_inclination_turn_guard,
        )
        .expect("writing to a String cannot fail");
    }
    validate_complete_bridge(&output)?;
    Ok(output.into_bytes())
}

#[cfg(test)]
mod tests {
    use wellforge_trajectory_analysis::analyze;
    use wellforge_trajectory_contract::{
        ApplicabilityStatement, CalculationEvidence, TrajectoryAnalysisResult,
        TrajectoryAnalysisStatus,
    };

    use super::{build, text};

    fn result() -> TrajectoryAnalysisResult {
        let request = wellforge_trajectory_fixtures::release_one_minimal_request();
        let calculation = analyze(&request).unwrap();
        TrajectoryAnalysisResult {
            contract_version: request.contract_version,
            analysis_id: request.analysis_id,
            sources: request.sources,
            status: TrajectoryAnalysisStatus::CompleteWithWarnings,
            applicability: ApplicabilityStatement {
                method: "minimum_curvature_closed_form".to_owned(),
                deterministic: true,
                limitations: Vec::new(),
            },
            evidence: CalculationEvidence {
                engine_version: "test-engine".to_owned(),
                compiler_version: "test-compiler".to_owned(),
                target_triple: "test-target".to_owned(),
                lockfile_hash: "test-lock".to_owned(),
                request_hash: "test-request".to_owned(),
                result_hash: "test-result".to_owned(),
            },
            calculation,
        }
    }

    #[test]
    fn bridge_text_rejects_raw_tabs_and_newlines_before_json_escaping() {
        for value in ["tab\tinside", "newline\ninside", "carriage\rinside"] {
            assert!(text(value).is_err(), "accepted forbidden text: {value:?}");
        }
    }

    #[test]
    fn bridge_accepts_every_collection_at_its_exact_capacity() {
        let mut result = result();
        result.calculation.plan = vec![result.calculation.plan[0].clone(); 500];
        result.calculation.survey = vec![result.calculation.survey[0].clone(); 500];
        result.calculation.targets = vec![result.calculation.targets[0].clone(); 100];
        result.calculation.slides = vec![result.calculation.slides[0].clone(); 200];
        result.calculation.formations = vec![result.calculation.formations[0].clone(); 100];

        let output = String::from_utf8(build(&result).unwrap()).unwrap();
        for (kind, expected) in [("P", 500), ("S", 500), ("T", 100), ("L", 200), ("F", 100)] {
            let prefix = format!("{kind}\t");
            assert_eq!(
                output
                    .lines()
                    .filter(|line| line.starts_with(&prefix))
                    .count(),
                expected,
                "wrong {kind} record count"
            );
        }
    }

    #[test]
    fn bridge_rejects_501_plan_records() {
        let mut result = result();
        result.calculation.plan = vec![result.calculation.plan[0].clone(); 501];
        assert_eq!(
            build(&result).unwrap_err(),
            "bridge plan capacity exceeded: 501 > 500"
        );
    }

    #[test]
    fn bridge_rejects_501_survey_records() {
        let mut result = result();
        result.calculation.survey = vec![result.calculation.survey[0].clone(); 501];
        assert_eq!(
            build(&result).unwrap_err(),
            "bridge survey capacity exceeded: 501 > 500"
        );
    }

    #[test]
    fn bridge_rejects_101_target_records() {
        let mut result = result();
        result.calculation.targets = vec![result.calculation.targets[0].clone(); 101];
        assert_eq!(
            build(&result).unwrap_err(),
            "bridge targets capacity exceeded: 101 > 100"
        );
    }

    #[test]
    fn bridge_rejects_201_slide_records() {
        let mut result = result();
        result.calculation.slides = vec![result.calculation.slides[0].clone(); 201];
        assert_eq!(
            build(&result).unwrap_err(),
            "bridge slides capacity exceeded: 201 > 200"
        );
    }

    #[test]
    fn bridge_rejects_101_formation_records() {
        let mut result = result();
        result.calculation.formations = vec![result.calculation.formations[0].clone(); 101];
        assert_eq!(
            build(&result).unwrap_err(),
            "bridge formations capacity exceeded: 101 > 100"
        );
    }
}
