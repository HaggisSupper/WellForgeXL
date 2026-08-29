//! End-to-end CLI contract tests.

use std::{fs, process::Command};

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
    let status = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
        .args([
            "verify-result",
            "--input",
            second.to_str().unwrap(),
            "--request-hash",
            &result.evidence.request_hash,
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let bridge = directory.path().join("result.wfbridge");
    let status = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
        .args([
            "bridge",
            "--input",
            second.to_str().unwrap(),
            "--output",
            bridge.to_str().unwrap(),
            "--request-hash",
            &result.evidence.request_hash,
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let bridge_text = fs::read_to_string(bridge).unwrap();
    assert!(bridge_text.starts_with("H\t1.0.0\t"));
    assert!(bridge_text.lines().any(|line| line.starts_with("S\t")));
    assert!(bridge_text.lines().any(|line| line.starts_with("M\t")));
    assert!(bridge_text.lines().any(|line| line.starts_with("F\t")));
    assert!(bridge_text.lines().any(|line| line.starts_with("C\t")));
    assert!(!bridge_text.contains('{'));
}

#[test]
fn validate_rejects_unknown_fields() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let mut value = serde_json::to_value(wellforge_bha_fixtures::minimal_request()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), serde_json::json!(true));
    fs::write(&input, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_wellforge-bha"))
        .args(["validate", "--input", input.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(!status.success());
}
