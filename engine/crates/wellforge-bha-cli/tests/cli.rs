//! End-to-end CLI contract tests.

use std::{fs, process::Command};

use serde_json::Value;

#[test]
fn run_writes_deterministic_value_only_result() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let first = directory.path().join("first.json");
    let second = directory.path().join("second.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&wellforge_bha_fixtures::minimal_request()).unwrap(),
    )
    .unwrap();
    for output in [&first, &second] {
        let status = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
            .args([
                "run",
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
    }
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    let result: wellforge_bha_contract::BhaAnalysisResult =
        serde_json::from_slice(&fs::read(first).unwrap()).unwrap();
    assert!(result.evidence.converged);
    assert!(!result.static_nodes.is_empty());
    assert!(!result.modes.is_empty());
}

#[test]
fn validate_rejects_unknown_fields_and_doctor_emits_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let mut value = serde_json::to_value(wellforge_bha_fixtures::minimal_request()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), serde_json::json!(true));
    fs::write(&input, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let validate = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!validate.status.success());
    assert!(!validate.stdout.is_empty() || !validate.stderr.is_empty());

    let doctor = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
        .args(["doctor"])
        .output()
        .unwrap();
    assert!(doctor.status.success());
    let identity: Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert_eq!(identity["engine"], "wellforge-bha");
    assert_eq!(identity["version"], env!("CARGO_PKG_VERSION"));
}
