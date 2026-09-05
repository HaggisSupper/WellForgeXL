//! Integration tests for the hydraulics CLI end-to-end path.

use std::{
    fs,
    process::{Command, Stdio},
};

use tempfile::tempdir;
use wellforge_hydraulics_contract::HydraulicsAnalysisRequest;
use wellforge_hydraulics_fixtures::{canonical_bingham_case, generalized_yield_power_law_case};

const FROZEN_VERSION_ONE_REQUEST: &str =
    include_str!("../../wellforge-hydraulics-fixtures/data/v0_1_request.json");
const FROZEN_VERSION_ONE_RESULT: &str =
    include_str!("../../wellforge-hydraulics-fixtures/data/v0_1_result.json");

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wellforge-hydraulics"))
}

#[test]
fn version_one_frozen_fixture_matches_pre_migration_result() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    fs::write(&input, FROZEN_VERSION_ONE_REQUEST).unwrap();

    let run = cli()
        .args([
            "run",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(run.status.success(), "run failed: {run:?}");

    let actual: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(FROZEN_VERSION_ONE_RESULT).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn version_reports_engine_name() {
    let output = cli().arg("version").output().expect("run version");
    assert!(
        output.status.success(),
        "version subcommand failed: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("wellforge-hydraulics "),
        "unexpected: {stdout}"
    );
}

#[test]
fn doctor_emits_json_metadata() {
    let output = cli().arg("doctor").output().expect("run doctor");
    assert!(output.status.success(), "doctor failed: {output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(value["engine"], "wellforge-hydraulics");
    assert!(value["dependency_lock_hash"].as_str().is_some());
}

#[test]
fn run_produces_result_with_populated_hashes() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let request = canonical_bingham_case();
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let status = cli()
        .args([
            "run",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .status()
        .expect("run subcommand");
    assert!(status.success(), "run failed with status {status:?}");

    let bytes = fs::read(&output).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["contract_version"], "0.1.0");
    assert_eq!(
        value["evidence"]["request_hash"].as_str().unwrap().len(),
        64
    );
    assert_eq!(value["evidence"]["result_hash"].as_str().unwrap().len(), 64);
    assert_eq!(value["evidence"]["profile_standard"], "API RP 13D");
    // Bit dP must be strictly positive for a flow rate through non-zero nozzles.
    assert!(value["bit_pressure_loss_pa"].as_f64().unwrap() > 0.0);
    // ECD must exceed the surface mud density because we added annular losses.
    let ecd = value["equivalent_circulating_density_kg_m3"]
        .as_f64()
        .unwrap();
    assert!(ecd >= 1200.0, "ECD must be at least mud density: {ecd}");
}

#[test]
fn validate_rejects_missing_bingham_parameters() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bad.json");
    let mut request = canonical_bingham_case();
    request.rheology.plastic_viscosity_pa_s = None;
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .expect("validate");
    assert!(!output.status.success(), "validate must fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("WF-HYD-REQ-"),
        "expected structured error: {stdout}"
    );
}

#[test]
fn validate_rejects_invalid_source_identity() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bad-source.json");
    let mut request = canonical_bingham_case();
    request.sources[0].content_hash = "sha1:0123".to_owned();
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .expect("validate");
    assert!(!output.status.success(), "validate must fail");
    assert!(String::from_utf8_lossy(&output.stdout).contains("WF-HYD-REQ-006"));
}

#[test]
fn validate_rejects_unsupported_contract_version() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("unsupported-version.json");
    let mut request = canonical_bingham_case();
    request.contract_version = "0.3.0".to_string();
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .expect("validate");

    assert!(!output.status.success(), "validate must fail");
    assert!(String::from_utf8_lossy(&output.stdout).contains("WF-HYD-REQ-001"));
}

#[test]
fn generalized_run_reports_selected_correlation_and_backend() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let request = generalized_yield_power_law_case();
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let status = cli()
        .args([
            "run",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("run subcommand");
    assert!(status.success(), "run failed with status {status:?}");

    let value: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert_eq!(value["contract_version"], "0.2.0");
    assert_eq!(
        value["evidence"]["flow_correlation"],
        "generalized_yield_power_law"
    );
    assert_eq!(value["evidence"]["compute_backend"], "parallel_cpu");
}

#[test]
fn batch_run_matches_individual_results_and_verifies() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("batch-request.json");
    let output = dir.path().join("batch-result.json");
    let base = generalized_yield_power_law_case();
    let requests: Vec<_> = [0.0080, 0.0085, 0.0090, 0.0095, 0.0100]
        .into_iter()
        .map(|diameter_m| {
            let mut request = base.clone();
            for nozzle in &mut request.operating.nozzles {
                nozzle.diameter_m = diameter_m;
            }
            request
        })
        .collect();
    fs::write(
        &input,
        serde_json::to_vec(&serde_json::json!({ "requests": requests })).unwrap(),
    )
    .unwrap();

    let validation = cli()
        .args(["validate-batch", "--input", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        validation.status.success(),
        "validation failed: {validation:?}"
    );
    let validation_json: serde_json::Value = serde_json::from_slice(&validation.stdout).unwrap();
    assert_eq!(
        validation_json["request_hashes"].as_array().unwrap().len(),
        5
    );

    let run = cli()
        .args([
            "run-batch",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(run.status.success(), "batch run failed: {run:?}");

    let batch: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    let results = batch["results"].as_array().unwrap();
    assert_eq!(results.len(), 5);
    for (index, request) in requests.iter().enumerate() {
        let single_input = dir.path().join(format!("single-request-{index}.json"));
        let single_output = dir.path().join(format!("single-result-{index}.json"));
        fs::write(&single_input, serde_json::to_vec(request).unwrap()).unwrap();
        let status = cli()
            .args([
                "run",
                "--input",
                single_input.to_str().unwrap(),
                "--output",
                single_output.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        let individual: serde_json::Value =
            serde_json::from_slice(&fs::read(single_output).unwrap()).unwrap();
        assert_eq!(results[index], individual);
    }

    let verified = cli()
        .args([
            "verify-batch",
            "--request",
            input.to_str().unwrap(),
            "--result",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "batch verify failed: {verified:?}"
    );

    let mut tampered = batch;
    tampered["results"][0]["bit_pressure_loss_pa"] = serde_json::json!(0.0);
    fs::write(&output, serde_json::to_vec(&tampered).unwrap()).unwrap();
    let rejected = cli()
        .args([
            "verify-batch",
            "--request",
            input.to_str().unwrap(),
            "--result",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "tampered batch must be rejected"
    );
}

#[test]
fn validate_batch_rejects_empty_request_array() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("empty-batch.json");
    fs::write(&input, br#"{"requests":[]}"#).unwrap();

    let validation = cli()
        .args(["validate-batch", "--input", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!validation.status.success());
    assert!(String::from_utf8_lossy(&validation.stdout).contains("WF-HYD-BATCH-001"));
}

#[test]
fn validate_reports_normalized_request_hash_and_verify_result_rejects_tampering() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let request = canonical_bingham_case();
    fs::write(&input, serde_json::to_vec_pretty(&request).unwrap()).unwrap();

    let validation = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(validation.status.success());
    let validation_json: serde_json::Value = serde_json::from_slice(&validation.stdout).unwrap();
    let request_hash = validation_json["request_hash"].as_str().unwrap();
    assert_eq!(request_hash.len(), 64);

    let run = cli()
        .args([
            "run",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(run.status.success());

    let verified = cli()
        .args([
            "verify-result",
            "--input",
            output.to_str().unwrap(),
            "--request-hash",
            request_hash,
        ])
        .output()
        .unwrap();
    assert!(verified.status.success(), "verify failed: {verified:?}");

    let mut result: serde_json::Value =
        serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    result["bit_pressure_loss_pa"] = serde_json::json!(0.0);
    fs::write(&output, serde_json::to_vec(&result).unwrap()).unwrap();
    let tampered = cli()
        .args([
            "verify-result",
            "--input",
            output.to_str().unwrap(),
            "--request-hash",
            request_hash,
        ])
        .output()
        .unwrap();
    assert!(!tampered.status.success());
}

#[test]
fn schema_writes_deterministic_documents() {
    let dir = tempdir().unwrap();
    let request_path = dir.path().join("req.schema.json");
    let result_path = dir.path().join("res.schema.json");
    let status = cli()
        .args([
            "schema",
            "--request",
            request_path.to_str().unwrap(),
            "--result",
            result_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let request_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&request_path).unwrap()).unwrap();
    let result_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&result_path).unwrap()).unwrap();
    assert_eq!(request_json["title"], "HydraulicsAnalysisRequest");
    assert_eq!(result_json["title"], "HydraulicsAnalysisResult");
}

#[test]
fn request_without_additive_controls_retains_screening_defaults() {
    let request = canonical_bingham_case();
    let value = serde_json::to_value(&request).unwrap();
    let input_bytes = serde_json::to_vec(&request).unwrap();
    let parsed: HydraulicsAnalysisRequest = serde_json::from_slice(&input_bytes).unwrap();
    let canonical_bytes = serde_json::to_vec(&parsed).unwrap();
    let reparsed: HydraulicsAnalysisRequest = serde_json::from_slice(&canonical_bytes).unwrap();

    assert_eq!(parsed.solver, None);
    assert_eq!(parsed.operating.nozzle_discharge_coefficient, None);
    assert_eq!(parsed.operating.surface_backpressure_pa, None);
    assert_eq!(parsed.operating.ecd_reference_tvd_m, None);
    assert_eq!(parsed.sections[0].top_tvd_m, None);
    assert_eq!(parsed.sections[0].bottom_tvd_m, None);
    assert_eq!(parsed.sections[0].active_flow_loop, None);
    assert_eq!(canonical_bytes, serde_json::to_vec(&reparsed).unwrap());
    assert!(value.get("solver").is_none());
    assert!(value["rheology"].get("high_shear_flow_index").is_none());
    assert!(
        value["operating"]
            .get("nozzle_discharge_coefficient")
            .is_none()
    );
    assert!(value["sections"][0].get("top_tvd_m").is_none());
}
