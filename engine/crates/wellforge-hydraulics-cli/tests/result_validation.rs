//! Rehashed invalid results exercise semantic guards independently of tampering.

use std::{fs, process::Command};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use wellforge_hydraulics_contract::HydraulicsAnalysisResult;

const REQUEST: &str = include_str!("../../wellforge-hydraulics-fixtures/data/v0_1_request.json");
const RESULT: &str = include_str!("../../wellforge-hydraulics-fixtures/data/v0_1_result.json");

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wellforge-hydraulics"))
}

fn rehash(value: Value) -> Value {
    let result: HydraulicsAnalysisResult = serde_json::from_value(value).unwrap();
    let mut normalized: Value =
        serde_json::from_slice(&serde_json::to_vec(&result).unwrap()).unwrap();
    normalized["evidence"]["result_hash"] = json!("");
    let digest = hex::encode(Sha256::digest(serde_json::to_vec(&normalized).unwrap()));
    normalized["evidence"]["result_hash"] = json!(digest);
    normalized
}

fn assert_rejected(mutate: impl FnOnce(&mut Value), batch: bool) {
    let dir = tempdir().unwrap();
    let result_path = dir.path().join("result.json");
    let mut result: Value = serde_json::from_str(RESULT).unwrap();
    mutate(&mut result);
    let result = rehash(result);
    let mut command = cli();
    if batch {
        let request_path = dir.path().join("request.json");
        let request: Value = serde_json::from_str(REQUEST).unwrap();
        fs::write(
            &request_path,
            serde_json::to_vec(&json!({"requests": [request]})).unwrap(),
        )
        .unwrap();
        fs::write(
            &result_path,
            serde_json::to_vec(&json!({"results": [result]})).unwrap(),
        )
        .unwrap();
        command
            .arg("verify-batch")
            .arg("--request")
            .arg(request_path)
            .arg("--result")
            .arg(result_path);
    } else {
        fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
        command
            .arg("verify-result")
            .arg("--input")
            .arg(result_path)
            .arg("--request-hash")
            .arg(result["evidence"]["request_hash"].as_str().unwrap());
    }
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(2),
        "accepted invalid result: {output:?}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("result hash mismatch"),
        "semantic case was rejected only as tampering: {output:?}"
    );
}

macro_rules! rejects_wire {
    ($name:ident, $mutate:expr) => {
        #[test]
        fn $name() {
            assert_rejected($mutate, false);
            assert_rejected($mutate, true);
        }
    };
}

rejects_wire!(
    negative_pressure,
    |r: &mut Value| r["total_pipe_pressure_loss_pa"] = json!(-1.0)
);
rejects_wire!(
    zero_ecd,
    |r: &mut Value| r["equivalent_circulating_density_kg_m3"] = json!(0.0)
);
rejects_wire!(negative_section_velocity, |r: &mut Value| r["sections"]
    [0]["bulk_velocity_m_s"] =
    json!(-1.0));
rejects_wire!(
    zero_section_reynolds,
    |r: &mut Value| r["sections"][0]["reynolds_number"] = json!(0.0)
);
rejects_wire!(negative_section_friction, |r: &mut Value| r["sections"]
    [0]["fanning_friction_factor"] =
    json!(-1.0));
rejects_wire!(negative_section_pressure, |r: &mut Value| r["sections"]
    [0]["pressure_loss_pa"] =
    json!(-1.0));
rejects_wire!(unsupported_version, |r: &mut Value| r["contract_version"] =
    json!("99.0.0"));
rejects_wire!(nil_analysis_identity, |r: &mut Value| r["analysis_id"] =
    json!("00000000-0000-0000-0000-000000000000"));
rejects_wire!(
    nil_section_identity,
    |r: &mut Value| r["sections"][0]["section_id"] = json!("00000000-0000-0000-0000-000000000000")
);
rejects_wire!(
    failed_status_with_correct_hash,
    |r: &mut Value| r["status"] = json!("failed")
);
rejects_wire!(empty_sections, |r: &mut Value| r["sections"] = json!([]));
rejects_wire!(duplicate_section_loop, |r: &mut Value| r["sections"][1] =
    r["sections"][0].clone());
rejects_wire!(
    engine_control_character,
    |r: &mut Value| r["evidence"]["engine_version"] = json!("0.1.0\n")
);
rejects_wire!(
    profile_control_character,
    |r: &mut Value| r["evidence"]["profile_standard"] = json!("API\r13D")
);
rejects_wire!(
    edition_control_character,
    |r: &mut Value| r["evidence"]["profile_edition"] = json!("7th\u{007f}")
);
rejects_wire!(warning_control_character, |r: &mut Value| r["warnings"] =
    json!(["unsafe\ntext"]));

#[test]
fn malformed_request_hash_cannot_be_endorsed_by_caller() {
    for hash in [
        String::new(),
        "a".repeat(63),
        "G".repeat(64),
        "A".repeat(64),
        format!("{}\n", "a".repeat(63)),
    ] {
        assert_rejected(|r| r["evidence"]["request_hash"] = json!(hash), false);
    }
}

#[test]
fn batch_binds_supported_version_to_request() {
    assert_rejected(|r| r["contract_version"] = json!("0.2.0"), true);
}

#[test]
fn batch_binds_analysis_identity_to_request() {
    assert_rejected(
        |r| r["analysis_id"] = json!("11111111-1111-4111-8111-111111111111"),
        true,
    );
}

#[test]
fn batch_binds_section_identity_to_request() {
    assert_rejected(
        |r| {
            for section in r["sections"].as_array_mut().unwrap() {
                section["section_id"] = json!("11111111-1111-4111-8111-111111111111");
            }
        },
        true,
    );
}

#[test]
fn batch_binds_section_order_and_count_to_request() {
    assert_rejected(|r| r["sections"].as_array_mut().unwrap().reverse(), true);
    assert_rejected(
        |r| {
            r["sections"].as_array_mut().unwrap().pop();
        },
        true,
    );
}

#[test]
fn batch_binds_profile_evidence_to_request() {
    assert_rejected(|r| r["evidence"]["profile_edition"] = json!("other"), true);
}

#[test]
fn frozen_result_verifies_without_rewriting_its_bytes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("frozen.json");
    fs::write(&path, RESULT).unwrap();
    let result: Value = serde_json::from_str(RESULT).unwrap();
    let output = cli()
        .arg("verify-result")
        .arg("--input")
        .arg(path)
        .arg("--request-hash")
        .arg(result["evidence"]["request_hash"].as_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn invalid_shared_batch_preserves_existing_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let first = wellforge_hydraulics_fixtures::generalized_yield_power_law_case();
    let mut later = first.clone();
    later.operating.nozzles[0].diameter_m = 1.0e200;
    fs::write(
        &input,
        serde_json::to_vec(&json!({"requests": [first, later]})).unwrap(),
    )
    .unwrap();
    fs::write(&output, b"previous accepted output").unwrap();
    let run = cli()
        .arg("run-batch")
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(2), "{run:?}");
    assert_eq!(fs::read(output).unwrap(), b"previous accepted output");
}

#[test]
fn numerically_valid_tampering_still_fails_hash_comparison() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("tampered.json");
    let mut result: Value = serde_json::from_str(RESULT).unwrap();
    result["bit_pressure_loss_pa"] = json!(1234.0);
    fs::write(&path, serde_json::to_vec(&result).unwrap()).unwrap();
    let output = cli()
        .arg("verify-result")
        .arg("--input")
        .arg(path)
        .arg("--request-hash")
        .arg(result["evidence"]["request_hash"].as_str().unwrap())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("result hash mismatch"));
}

fn assert_invalid_request_preserves_output(mutate: impl FnOnce(&mut Value), batch: bool) {
    let dir = tempdir().unwrap();
    let input = dir.path().join("request.json");
    let output = dir.path().join("result.json");
    let mut request: Value = serde_json::from_str(REQUEST).unwrap();
    mutate(&mut request);
    let payload = if batch {
        // A valid earlier result must not be committed when a later request is invalid.
        let first: Value = serde_json::from_str(REQUEST).unwrap();
        json!({"requests": [first, request]})
    } else {
        request
    };
    fs::write(&input, serde_json::to_vec(&payload).unwrap()).unwrap();
    let prior_output = b"previous accepted result bytes";
    fs::write(&output, prior_output).unwrap();

    let validated = cli()
        .arg(if batch { "validate-batch" } else { "validate" })
        .arg("--input")
        .arg(&input)
        .output()
        .unwrap();
    let produced = cli()
        .arg(if batch { "run-batch" } else { "run" })
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();

    assert_eq!(
        produced.status.code(),
        Some(2),
        "producer accepted invalid request: {produced:?}"
    );
    assert_eq!(fs::read(&output).unwrap(), prior_output);
    assert_eq!(
        validated.status.code(),
        Some(2),
        "validator accepted invalid request: {validated:?}"
    );
    let diagnostic: Value = serde_json::from_slice(&validated.stdout).unwrap();
    assert_eq!(diagnostic["error"], "request_validation_failure");
}

macro_rules! rejects_unverifiable_request {
    ($single:ident, $batch:ident, $mutate:expr) => {
        #[test]
        fn $single() {
            assert_invalid_request_preserves_output($mutate, false);
        }
        #[test]
        fn $batch() {
            assert_invalid_request_preserves_output($mutate, true);
        }
    };
}

rejects_unverifiable_request!(
    nil_section_request_preserves_output,
    later_nil_section_request_preserves_batch_output,
    |r: &mut Value| {
        r["sections"][0]["id"] = json!("00000000-0000-0000-0000-000000000000");
    }
);
rejects_unverifiable_request!(
    duplicate_section_request_preserves_output,
    later_duplicate_section_request_preserves_batch_output,
    |r: &mut Value| {
        let mut later = r["sections"][0].clone();
        later["top_md_m"] = later["bottom_md_m"].clone();
        later["bottom_md_m"] = json!(later["top_md_m"].as_f64().unwrap() + 100.0);
        r["sections"].as_array_mut().unwrap().push(later);
    }
);
rejects_unverifiable_request!(
    unsafe_standard_request_preserves_output,
    later_unsafe_standard_request_preserves_batch_output,
    |r: &mut Value| {
        r["profile"]["standard"] = json!("API\nRP 13D");
    }
);
rejects_unverifiable_request!(
    unsafe_edition_request_preserves_output,
    later_unsafe_edition_request_preserves_batch_output,
    |r: &mut Value| {
        r["profile"]["edition"] = json!("7th\nEdition");
    }
);
