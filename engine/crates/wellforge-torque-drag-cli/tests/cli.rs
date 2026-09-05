//! Integration tests for the torque-and-drag CLI end-to-end path.

use std::{
    fs,
    process::{Command, Stdio},
};

use tempfile::tempdir;
use wellforge_torque_drag_fixtures::canonical_pickup_case;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wellforge-torque-drag"))
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
        stdout.starts_with("wellforge-torque-drag "),
        "unexpected: {stdout}"
    );
}

#[test]
fn doctor_emits_json_metadata() {
    let output = cli().arg("doctor").output().expect("run doctor");
    assert!(output.status.success(), "doctor failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(value["engine"], "wellforge-torque-drag");
    assert!(value["dependency_lock_hash"].as_str().is_some());
}

#[test]
fn run_produces_result_with_populated_hashes() {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let request = canonical_pickup_case();
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

    let bytes = fs::read(&output).expect("read result");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");
    assert_eq!(value["contract_version"], "0.1.0");
    assert_eq!(
        value["evidence"]["request_hash"].as_str().unwrap().len(),
        64
    );
    assert_eq!(value["evidence"]["result_hash"].as_str().unwrap().len(), 64);
    let stations = value["stations"].as_array().unwrap();
    assert_eq!(stations.len(), request.trajectory.len());
    // For a pickup on an inclined string, the top-of-string tension must be positive.
    let top = &stations[0];
    let top_tension = top["effective_tension_n"].as_f64().unwrap();
    assert!(
        top_tension > 0.0,
        "top-of-string tension must be positive on pickup: {top_tension}"
    );
}

#[test]
fn validate_reports_normalized_request_hash_and_verify_result_rejects_tampering() {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let request = canonical_pickup_case();
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
    result["stations"][0]["torque_nm"] = serde_json::json!(1.0);
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
fn validate_rejects_reversed_trajectory() {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("bad.json");
    let mut request = canonical_pickup_case();
    // Reverse a middle station so md is not monotonic.
    let last = request.trajectory[1].md_m;
    request.trajectory[1].md_m = last - 500.0;
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .expect("validate");
    assert!(!output.status.success(), "validate must fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("WF-TND-REQ-"),
        "expected structured error: {stdout}"
    );
}

#[test]
fn validate_rejects_invalid_source_identity() {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("bad-source.json");
    let mut request = canonical_pickup_case();
    request.sources[0].content_hash = "sha1:0123".to_owned();
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = cli()
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .expect("validate");
    assert!(!output.status.success(), "validate must fail");
    assert!(String::from_utf8_lossy(&output.stdout).contains("WF-TND-REQ-005"));
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
    let request_bytes = fs::read(&request_path).unwrap();
    let result_bytes = fs::read(&result_path).unwrap();
    let request_json: serde_json::Value = serde_json::from_slice(&request_bytes).unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&result_bytes).unwrap();
    assert_eq!(request_json["title"], "TnDAnalysisRequest");
    assert_eq!(result_json["title"], "TnDAnalysisResult");
}
