//! Integration tests for the hydraulics CLI end-to-end path.

use std::{
    fs,
    process::{Command, Stdio},
};

use tempfile::tempdir;
use wellforge_hydraulics_fixtures::canonical_bingham_case;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wellforge-hydraulics"))
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
