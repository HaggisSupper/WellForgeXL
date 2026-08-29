//! End-to-end tests for the deterministic trajectory executable boundary.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{Value, json};
use wellforge_trajectory_contract::{TrajectoryAnalysisResult, TrajectoryAnalysisStatus};

const REQUEST_HASH: &str = "87b7e1a6b6ef981b4cbb1a24c67124ae479aa19336df41617b7f489b3a116771";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_wellforge-trajectory")
}

fn write_request(path: &Path) {
    let bytes =
        serde_json::to_vec_pretty(&wellforge_trajectory_fixtures::release_one_minimal_request())
            .unwrap();
    fs::write(path, bytes).unwrap();
}

fn run(args: &[&str]) -> Output {
    Command::new(bin()).args(args).output().unwrap()
}

fn exit(output: &Output) -> i32 {
    output.status.code().unwrap()
}

fn run_result(input: &Path, output: &Path, extra: &[&str]) -> Output {
    let mut arguments = vec![
        "run",
        "--input",
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ];
    arguments.extend_from_slice(extra);
    run(&arguments)
}

fn parse_result(path: &Path) -> TrajectoryAnalysisResult {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn stable_fixture_value(bytes: &[u8]) -> Value {
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    let evidence = value["evidence"].as_object_mut().unwrap();
    for build_specific in ["compiler_version", "target_triple", "result_hash"] {
        evidence.remove(build_specific);
    }
    value
}

fn backup_paths(directory: &Path, output: &Path) -> Vec<PathBuf> {
    let prefix = format!("{}.backup.", output.file_name().unwrap().to_string_lossy());
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
        })
        .collect()
}

#[test]
fn run_is_byte_deterministic_and_strictly_round_trippable() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let first = directory.path().join("first.json");
    let second = directory.path().join("second.json");
    write_request(&input);

    assert_eq!(exit(&run_result(&input, &first, &["--no-backup"])), 0);
    assert_eq!(exit(&run_result(&input, &second, &["--no-backup"])), 0);
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    let expected = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/expected/trajectory-release-one-minimal.result.json");
    assert_eq!(
        stable_fixture_value(&fs::read(&first).unwrap()),
        stable_fixture_value(&fs::read(expected).unwrap())
    );
    assert!(fs::read(&first).unwrap().ends_with(b"\n"));

    let result = parse_result(&first);
    assert_eq!(
        result.analysis_id.to_string(),
        "00000000-0000-0000-0000-000000000064"
    );
    assert_eq!(
        result.status,
        TrajectoryAnalysisStatus::CompleteWithWarnings
    );
    assert!(result.applicability.deterministic);
    assert_eq!(result.calculation.plan.len(), 3);
    assert_eq!(result.calculation.survey.len(), 3);
    assert_eq!(result.calculation.targets.len(), 2);
    assert_eq!(result.calculation.slides.len(), 2);
    assert_eq!(result.calculation.formations.len(), 1);
    assert!(result.calculation.projection.is_some());
    assert_eq!(result.evidence.request_hash, REQUEST_HASH);
    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    assert_eq!(result.evidence.target_triple, "x86_64-unknown-linux-gnu");

    let mut value: Value = serde_json::from_slice(&fs::read(&first).unwrap()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), json!(true));
    assert!(serde_json::from_value::<TrajectoryAnalysisResult>(value).is_err());
}

#[test]
fn run_reports_typed_target_and_slide_warning_states() {
    let directory = tempfile::tempdir().unwrap();

    let mut low_inclination = wellforge_trajectory_fixtures::release_one_minimal_request();
    low_inclination.plan[2].md_m = 100.0;
    low_inclination.slides.truncate(1);
    low_inclination.slides[0].low_inclination_threshold_rad = 0.2;
    let low_input = directory.path().join("low-inclination.request.json");
    let low_output = directory.path().join("low-inclination.result.json");
    fs::write(
        &low_input,
        serde_json::to_vec_pretty(&low_inclination).unwrap(),
    )
    .unwrap();

    assert_eq!(
        exit(&run_result(&low_input, &low_output, &["--no-backup"])),
        0
    );
    let low_result = parse_result(&low_output);
    assert_eq!(
        low_result.status,
        TrajectoryAnalysisStatus::CompleteWithWarnings
    );
    assert!(low_result.applicability.limitations.iter().any(|value| {
        value == "one or more slide responses are below the low-inclination threshold"
    }));

    let mut overflow = wellforge_trajectory_fixtures::release_one_minimal_request();
    overflow.plan[2].md_m = 100.0;
    overflow.slides.truncate(1);
    overflow.targets[0].major_m = f64::from_bits(1);
    overflow.targets[0].minor_m = f64::from_bits(1);
    overflow.slides[0].rotary_build_rad_per_m = f64::MAX;
    let overflow_input = directory.path().join("overflow.request.json");
    let overflow_output = directory.path().join("overflow.result.json");
    fs::write(
        &overflow_input,
        serde_json::to_vec_pretty(&overflow).unwrap(),
    )
    .unwrap();

    assert_eq!(
        exit(&run_result(
            &overflow_input,
            &overflow_output,
            &["--no-backup"]
        )),
        0
    );
    let overflow_result = parse_result(&overflow_output);
    assert_eq!(
        overflow_result.status,
        TrajectoryAnalysisStatus::CompleteWithWarnings
    );
    assert!(
        overflow_result
            .applicability
            .limitations
            .iter()
            .any(|value| {
                value == "one or more target evaluations encountered numerical overflow"
            })
    );
    assert!(
        overflow_result
            .applicability
            .limitations
            .iter()
            .any(|value| { value == "one or more slide responses encountered numerical overflow" })
    );
}

#[test]
fn canonical_request_hash_normalizes_layout_and_negative_zero() {
    let directory = tempfile::tempdir().unwrap();
    let first_input = directory.path().join("first.json");
    let second_input = directory.path().join("second.json");
    let first_output = directory.path().join("first.result.json");
    let second_output = directory.path().join("second.result.json");
    let request = wellforge_trajectory_fixtures::release_one_minimal_request();
    fs::write(&first_input, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    let mut value = serde_json::to_value(request).unwrap();
    value["vertical_section_azimuth_rad"] = json!(-0.0);
    fs::write(&second_input, serde_json::to_vec(&value).unwrap()).unwrap();

    assert_eq!(
        exit(&run_result(&first_input, &first_output, &["--no-backup"])),
        0
    );
    assert_eq!(
        exit(&run_result(&second_input, &second_output, &["--no-backup"])),
        0
    );
    assert_eq!(
        parse_result(&first_output).evidence.request_hash,
        REQUEST_HASH
    );
    assert_eq!(
        parse_result(&first_output).evidence.request_hash,
        parse_result(&second_output).evidence.request_hash
    );
}

#[test]
fn command_exit_codes_are_stable_and_validation_is_strict() {
    let directory = tempfile::tempdir().unwrap();
    let valid = directory.path().join("valid.json");
    let invalid = directory.path().join("invalid.json");
    let calculation_failure = directory.path().join("calculation-failure.json");
    let result = directory.path().join("result.json");
    write_request(&valid);

    let mut unknown =
        serde_json::to_value(wellforge_trajectory_fixtures::release_one_minimal_request()).unwrap();
    unknown["unknown"] = json!(true);
    fs::write(&invalid, serde_json::to_vec(&unknown).unwrap()).unwrap();

    let mut ambiguous = wellforge_trajectory_fixtures::release_one_minimal_request();
    ambiguous.plan[1].inclination_rad = std::f64::consts::PI;
    ambiguous.plan[1].azimuth_rad = 0.0;
    fs::write(
        &calculation_failure,
        serde_json::to_vec(&ambiguous).unwrap(),
    )
    .unwrap();

    assert_eq!(exit(&run(&["not-a-command"])), 2);
    assert_eq!(
        exit(&run(&["validate", "--input", invalid.to_str().unwrap()])),
        10
    );
    assert_eq!(
        exit(&run_result(&calculation_failure, &result, &["--no-backup"])),
        20
    );
    assert!(!result.exists());
    assert_eq!(
        exit(&run(&[
            "validate",
            "--input",
            directory.path().join("missing.json").to_str().unwrap(),
        ])),
        40
    );
    assert_eq!(
        exit(&run(&["validate", "--input", valid.to_str().unwrap()])),
        0
    );
}

#[test]
fn verify_result_rejects_hash_mismatch_tampering_and_failed_status() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let result = directory.path().join("result.json");
    write_request(&input);
    assert_eq!(exit(&run_result(&input, &result, &["--no-backup"])), 0);
    let parsed = parse_result(&result);
    let request_hash = parsed.evidence.request_hash.clone();

    assert_eq!(
        exit(&run(&[
            "verify-result",
            "--input",
            result.to_str().unwrap(),
            "--request-hash",
            &request_hash,
        ])),
        0
    );
    assert_eq!(
        exit(&run(&[
            "verify-result",
            "--input",
            result.to_str().unwrap(),
            "--request-hash",
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ])),
        30
    );

    let original = fs::read(&result).unwrap();
    let mut tampered: Value = serde_json::from_slice(&original).unwrap();
    tampered["calculation"]["plan"][0]["north_m"] = json!(1.0);
    fs::write(&result, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
    assert_eq!(
        exit(&run(&[
            "verify-result",
            "--input",
            result.to_str().unwrap(),
            "--request-hash",
            &request_hash,
        ])),
        30
    );

    let mut failed: Value = serde_json::from_slice(&original).unwrap();
    failed["status"] = json!("failed");
    failed["evidence"]["result_hash"] = json!("");
    fs::write(&result, serde_json::to_vec_pretty(&failed).unwrap()).unwrap();
    assert_eq!(
        exit(&run(&[
            "verify-result",
            "--input",
            result.to_str().unwrap(),
            "--request-hash",
            &request_hash,
        ])),
        30
    );
}

#[test]
fn run_preserves_timestamped_backup_unless_disabled() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    write_request(&input);
    fs::write(&output, b"previous result\n").unwrap();

    assert_eq!(exit(&run_result(&input, &output, &[])), 0);
    let backups = backup_paths(directory.path(), &output);
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(&backups[0]).unwrap(), b"previous result\n");
    assert!(parse_result(&output).applicability.deterministic);

    for backup in backups {
        fs::remove_file(backup).unwrap();
    }
    fs::write(&output, b"replace without backup\n").unwrap();
    assert_eq!(exit(&run_result(&input, &output, &["--no-backup"])), 0);
    assert!(backup_paths(directory.path(), &output).is_empty());
}

#[test]
fn backup_is_copied_before_atomic_destination_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    let old_output_witness = directory.path().join("old-output-witness.json");
    write_request(&input);
    fs::write(&output, b"previous result\n").unwrap();
    fs::hard_link(&output, &old_output_witness).unwrap();

    assert_eq!(exit(&run_result(&input, &output, &[])), 0);
    let backups = backup_paths(directory.path(), &output);
    assert_eq!(backups.len(), 1);
    fs::write(&old_output_witness, b"mutated witness\n").unwrap();
    assert_eq!(fs::read(&backups[0]).unwrap(), b"previous result\n");
    assert!(parse_result(&output).applicability.deterministic);
}

#[test]
fn diagnostics_reject_output_path_aliases_before_writing_result() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    let nested = directory.path().join("nested");
    write_request(&input);
    fs::create_dir(&nested).unwrap();

    for diagnostics in [output.clone(), nested.join("../result.json")] {
        fs::write(&output, b"untouched output\n").unwrap();
        let process = run_result(
            &input,
            &output,
            &[
                "--diagnostics",
                diagnostics.to_str().unwrap(),
                "--no-backup",
            ],
        );
        assert_eq!(exit(&process), 40);
        assert_eq!(fs::read(&output).unwrap(), b"untouched output\n");
    }
}

#[cfg(unix)]
#[test]
fn diagnostics_reject_nonexistent_output_through_symlinked_parent_alias() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let real_parent = directory.path().join("real-dir");
    let alias_parent = directory.path().join("alias-dir");
    let output = alias_parent.join("result.json");
    let diagnostics = real_parent.join("result.json");
    write_request(&input);
    fs::create_dir(&real_parent).unwrap();
    symlink(&real_parent, &alias_parent).unwrap();
    assert!(!output.exists());
    assert!(!diagnostics.exists());

    let process = run_result(
        &input,
        &output,
        &[
            "--diagnostics",
            diagnostics.to_str().unwrap(),
            "--no-backup",
        ],
    );
    assert_eq!(exit(&process), 40);
    assert!(!output.exists());
    assert!(!diagnostics.exists());
}

#[test]
fn run_rejects_input_output_alias_without_diagnostics_before_reading_or_writing() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let nested = directory.path().join("nested");
    let output = nested.join("../request.json");
    write_request(&input);
    fs::create_dir(&nested).unwrap();
    let original = fs::read(&input).unwrap();

    let process = run_result(&input, &output, &["--no-backup"]);
    assert_eq!(exit(&process), 40);
    assert_eq!(fs::read(&input).unwrap(), original);
}

#[test]
fn bridge_rejects_input_output_alias_before_reading_or_writing() {
    let directory = tempfile::tempdir().unwrap();
    let request = directory.path().join("request.json");
    let result = directory.path().join("result.json");
    write_request(&request);
    assert_eq!(exit(&run_result(&request, &result, &["--no-backup"])), 0);
    let parsed = parse_result(&result);
    let original = fs::read(&result).unwrap();

    let process = run(&[
        "bridge",
        "--input",
        result.to_str().unwrap(),
        "--output",
        result.to_str().unwrap(),
        "--request-hash",
        &parsed.evidence.request_hash,
    ]);
    assert_eq!(exit(&process), 40);
    assert_eq!(fs::read(&result).unwrap(), original);
}

#[test]
fn schema_rejects_identical_outputs_before_writing_either_file() {
    let directory = tempfile::tempdir().unwrap();
    let schema = directory.path().join("trajectory.schema.json");

    let process = run(&[
        "schema",
        "--request",
        schema.to_str().unwrap(),
        "--result",
        schema.to_str().unwrap(),
    ]);
    assert_eq!(exit(&process), 40);
    assert!(!schema.exists());
}

#[cfg(unix)]
#[test]
fn bridge_and_schema_reject_nearest_parent_symlink_aliases() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let real_parent = directory.path().join("real-dir");
    let alias_parent = directory.path().join("alias-dir");
    fs::create_dir(&real_parent).unwrap();
    symlink(&real_parent, &alias_parent).unwrap();

    let request = directory.path().join("request.json");
    let result = real_parent.join("result.json");
    let bridge_alias = alias_parent.join("result.json");
    write_request(&request);
    assert_eq!(exit(&run_result(&request, &result, &["--no-backup"])), 0);
    let parsed = parse_result(&result);
    let original = fs::read(&result).unwrap();
    let bridge = run(&[
        "bridge",
        "--input",
        result.to_str().unwrap(),
        "--output",
        bridge_alias.to_str().unwrap(),
        "--request-hash",
        &parsed.evidence.request_hash,
    ]);
    assert_eq!(exit(&bridge), 40);
    assert_eq!(fs::read(&result).unwrap(), original);

    let request_schema = alias_parent.join("schema.json");
    let result_schema = real_parent.join("schema.json");
    assert!(!request_schema.exists());
    assert!(!result_schema.exists());
    let schema = run(&[
        "schema",
        "--request",
        request_schema.to_str().unwrap(),
        "--result",
        result_schema.to_str().unwrap(),
    ]);
    assert_eq!(exit(&schema), 40);
    assert!(!request_schema.exists());
    assert!(!result_schema.exists());
}

#[test]
fn diagnostics_replace_atomically_instead_of_truncating_existing_file() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    let diagnostics = directory.path().join("diagnostics.jsonl");
    let old_diagnostics_witness = directory.path().join("old-diagnostics-witness.jsonl");
    write_request(&input);
    fs::write(&diagnostics, b"previous diagnostics\n").unwrap();
    fs::hard_link(&diagnostics, &old_diagnostics_witness).unwrap();

    let process = run_result(
        &input,
        &output,
        &[
            "--diagnostics",
            diagnostics.to_str().unwrap(),
            "--no-backup",
        ],
    );
    assert_eq!(exit(&process), 0);
    let new_diagnostics = fs::read(&diagnostics).unwrap();
    fs::write(&old_diagnostics_witness, b"mutated witness\n").unwrap();
    assert_eq!(fs::read(&diagnostics).unwrap(), new_diagnostics);
    assert_ne!(new_diagnostics, b"previous diagnostics\n");
}

#[test]
fn diagnostics_are_json_lines_and_stdout_is_bounded() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    let diagnostics = directory.path().join("diagnostics.jsonl");
    write_request(&input);

    let process = run_result(
        &input,
        &output,
        &[
            "--diagnostics",
            diagnostics.to_str().unwrap(),
            "--no-backup",
        ],
    );
    assert_eq!(exit(&process), 0);
    assert!(process.stdout.len() <= 1024);
    assert!(!process.stdout.windows(2).any(|pair| pair == b"[{"));
    let lines = fs::read_to_string(diagnostics).unwrap();
    assert!(!lines.is_empty());
    for line in lines.lines() {
        let record: Value = serde_json::from_str(line).unwrap();
        for field in [
            "level",
            "event",
            "code",
            "analysis_id",
            "request_hash",
            "message",
        ] {
            assert!(record.get(field).is_some(), "missing {field}: {line}");
        }
    }
}

#[test]
fn schema_and_version_commands_emit_bounded_file_only_contracts() {
    let directory = tempfile::tempdir().unwrap();
    let request_schema = directory.path().join("request.schema.json");
    let result_schema = directory.path().join("result.schema.json");
    let schema = run(&[
        "schema",
        "--request",
        request_schema.to_str().unwrap(),
        "--result",
        result_schema.to_str().unwrap(),
    ]);
    assert_eq!(exit(&schema), 0);
    assert!(schema.stdout.len() <= 1024);
    for path in [&request_schema, &result_schema] {
        let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(value["title"].as_str().is_some());
    }

    let version = run(&["version", "--json"]);
    assert_eq!(exit(&version), 0);
    let value: Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(value["name"], "wellforge-trajectory");
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn result_and_version_share_the_build_captured_rustc_identity() {
    let injected = env!("WELLFORGE_RUSTC_VERSION_VERBOSE");
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let output = directory.path().join("result.json");
    write_request(&input);
    assert_eq!(exit(&run_result(&input, &output, &["--no-backup"])), 0);

    let result = parse_result(&output);
    let version = run(&["version", "--json"]);
    assert_eq!(exit(&version), 0);
    let version: Value = serde_json::from_slice(&version.stdout).unwrap();
    assert_eq!(result.evidence.compiler_version, injected);
    assert_eq!(version["compiler_version"], injected);
    for field in ["rustc ", "commit-hash:", "host:", "release:"] {
        assert!(injected.contains(field), "missing {field} in {injected}");
    }
}

#[test]
fn bridge_has_one_header_and_ordered_bounded_record_counts() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("request.json");
    let result = directory.path().join("result.json");
    let bridge = directory.path().join("result.wfbridge");
    write_request(&input);
    assert_eq!(exit(&run_result(&input, &result, &["--no-backup"])), 0);
    let parsed = parse_result(&result);
    let process = run(&[
        "bridge",
        "--input",
        result.to_str().unwrap(),
        "--output",
        bridge.to_str().unwrap(),
        "--request-hash",
        &parsed.evidence.request_hash,
    ]);
    assert_eq!(exit(&process), 0);
    assert!(process.stdout.len() <= 1024);

    let text = fs::read_to_string(&bridge).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("H\t")).count(),
        1
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("P\t")).count(),
        3
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("S\t")).count(),
        3
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("R\t")).count(),
        3
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("T\t")).count(),
        2
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("L\t")).count(),
        2
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("F\t")).count(),
        1
    );
    assert_eq!(
        lines.iter().filter(|line| line.starts_with("X\t")).count(),
        1
    );
    let header: Vec<_> = lines[0].split('\t').collect();
    assert_eq!(header[0], "H");
    assert_eq!(header[1], "1.0.0");
    assert_eq!(header[2], parsed.analysis_id.to_string());
    assert_eq!(header[3], parsed.evidence.request_hash);
    assert_eq!(header[4], parsed.evidence.result_hash);
    assert_eq!(header[6], "complete_with_warnings");
    assert_eq!(header[7], "true");
    assert!(lines.iter().all(|line| !line.contains(['\r', '\0'])));
}
