//! `WellForge` deterministic trajectory analysis command-line interface.

mod bridge;
mod canonical;
mod diagnostics;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use schemars::schema_for;
use serde::Serialize;
use sha2::Digest;
use thiserror::Error;
use wellforge_trajectory_analysis::{TrajectoryAnalysisError, analyze};
use wellforge_trajectory_contract::{
    ApplicabilityStatement, CalculationEvidence, FormationCoverage, InterpolationStatus,
    SlideStatus, TargetBasis, TargetStatus, TrajectoryAnalysisRequest, TrajectoryAnalysisResult,
    TrajectoryAnalysisStatus, validate_request,
};

const EXIT_INVALID_REQUEST: i32 = 10;
const EXIT_CALCULATION_FAILURE: i32 = 20;
const EXIT_INTEGRITY_FAILURE: i32 = 30;
const EXIT_IO_FAILURE: i32 = 40;
const COMPILER_VERSION: &str = env!("WELLFORGE_RUSTC_VERSION_VERBOSE");

#[derive(Parser)]
#[command(
    name = "wellforge-trajectory",
    version,
    about = "WellForge deterministic trajectory analysis engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a strict request without calculating a result.
    Validate {
        /// Explicit request JSON file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Calculate and atomically write a strict trajectory result.
    Run {
        /// Explicit request JSON file.
        #[arg(long)]
        input: PathBuf,
        /// Explicit result JSON file.
        #[arg(long)]
        output: PathBuf,
        /// Optional JSON Lines diagnostic file.
        #[arg(long)]
        diagnostics: Option<PathBuf>,
        /// Replace an existing result without preserving a timestamped backup.
        #[arg(long)]
        no_backup: bool,
    },
    /// Strictly parse and verify result and request hashes.
    VerifyResult {
        /// Explicit result JSON file.
        #[arg(long)]
        input: PathBuf,
        /// Expected canonical request SHA-256.
        #[arg(long)]
        request_hash: String,
    },
    /// Verify a result and atomically emit the versioned VBA bridge.
    Bridge {
        /// Explicit result JSON file.
        #[arg(long)]
        input: PathBuf,
        /// Explicit bridge output file.
        #[arg(long)]
        output: PathBuf,
        /// Expected canonical request SHA-256.
        #[arg(long)]
        request_hash: String,
    },
    /// Write deterministic request and result JSON Schemas.
    Schema {
        /// Explicit request-schema output file.
        #[arg(long)]
        request: PathBuf,
        /// Explicit result-schema output file.
        #[arg(long)]
        result: PathBuf,
    },
    /// Print engine build identity.
    Version {
        /// Emit a compact JSON identity object.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Error)]
enum CommandError {
    #[error("invalid trajectory request: {0}")]
    InvalidRequest(String),
    #[error("trajectory calculation failed: {0}")]
    Calculation(String),
    #[error("trajectory result integrity failure: {0}")]
    Integrity(String),
    #[error("file I/O failure: {0}")]
    Io(String),
}

impl CommandError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidRequest(_) => EXIT_INVALID_REQUEST,
            Self::Calculation(_) => EXIT_CALCULATION_FAILURE,
            Self::Integrity(_) => EXIT_INTEGRITY_FAILURE,
            Self::Io(_) => EXIT_IO_FAILURE,
        }
    }
}

fn io_error(action: &str, path: &Path, error: impl std::fmt::Display) -> CommandError {
    CommandError::Io(format!("{action} {}: {error}", path.display()))
}

fn read(path: &Path) -> Result<Vec<u8>, CommandError> {
    fs::read(path).map_err(|error| io_error("cannot read", path, error))
}

fn read_request(path: &Path) -> Result<TrajectoryAnalysisRequest, CommandError> {
    let request: TrajectoryAnalysisRequest = serde_json::from_slice(&read(path)?)
        .map_err(|error| CommandError::InvalidRequest(error.to_string()))?;
    validate_request(&request).map_err(|errors| {
        let diagnostic = errors
            .iter()
            .map(|error| format!("{}: {}", error.code, error.message))
            .collect::<Vec<_>>()
            .join("; ");
        CommandError::InvalidRequest(diagnostic)
    })?;
    Ok(request)
}

fn lexical_absolute(path: &Path) -> Result<PathBuf, CommandError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|error| {
                CommandError::Io(format!("cannot resolve current directory: {error}"))
            })?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

fn path_identity(path: &Path) -> Result<PathBuf, CommandError> {
    let normalized = lexical_absolute(path)?;
    let mut ancestor = normalized.as_path();
    let mut suffix = Vec::new();
    loop {
        match fs::canonicalize(ancestor) {
            Ok(mut identity) => {
                for component in suffix.iter().rev() {
                    identity.push(component);
                }
                return Ok(identity);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let component = ancestor.file_name().ok_or_else(|| {
                    io_error("cannot resolve an existing ancestor for", path, error)
                })?;
                suffix.push(component.to_os_string());
                ancestor = ancestor.parent().ok_or_else(|| {
                    CommandError::Io(format!(
                        "cannot resolve an existing ancestor for {}",
                        path.display()
                    ))
                })?;
            }
            Err(error) => {
                return Err(io_error("cannot resolve path identity for", path, error));
            }
        }
    }
}

fn path_identities_collide(left: &Path, right: &Path, case_insensitive: bool) -> bool {
    if case_insensitive {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    } else {
        left == right
    }
}

fn reject_path_collisions_with_case_policy(
    paths: &[(&str, &Path)],
    case_insensitive: bool,
) -> Result<(), CommandError> {
    let identities = paths
        .iter()
        .map(|(name, path)| Ok((*name, path_identity(path)?)))
        .collect::<Result<Vec<_>, CommandError>>()?;
    for (index, (left_name, left_identity)) in identities.iter().enumerate() {
        for (right_name, right_identity) in &identities[index + 1..] {
            if path_identities_collide(left_identity, right_identity, case_insensitive) {
                return Err(CommandError::Io(format!(
                    "{left_name} and {right_name} paths must not alias one another"
                )));
            }
        }
    }
    Ok(())
}

fn reject_path_collisions(paths: &[(&str, &Path)]) -> Result<(), CommandError> {
    reject_path_collisions_with_case_policy(paths, cfg!(windows))
}

fn backup_path(path: &Path) -> Result<PathBuf, CommandError> {
    let file_name = path.file_name().ok_or_else(|| {
        CommandError::Io(format!("output path has no file name: {}", path.display()))
    })?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CommandError::Io(format!("system clock precedes Unix epoch: {error}")))?
        .as_nanos();
    Ok(path.with_file_name(format!(
        "{}.backup.{timestamp}",
        file_name.to_string_lossy()
    )))
}

fn write_atomic(path: &Path, bytes: &[u8], preserve_backup: bool) -> Result<(), CommandError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| io_error("cannot create parent directory for", path, error))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| io_error("cannot create temporary file beside", path, error))?;
    temporary
        .write_all(bytes)
        .map_err(|error| io_error("cannot write temporary file for", path, error))?;
    temporary
        .as_file_mut()
        .sync_all()
        .map_err(|error| io_error("cannot flush temporary file for", path, error))?;

    if preserve_backup && path.exists() {
        let backup = backup_path(path)?;
        fs::copy(path, &backup)
            .map_err(|error| io_error("cannot preserve existing output", path, error))?;
    }
    if let Err(error) = temporary.persist(path) {
        return Err(io_error("cannot atomically replace", path, error.error));
    }
    Ok(())
}

fn write_pretty<T: Serialize>(
    path: &Path,
    value: &T,
    preserve_backup: bool,
) -> Result<(), CommandError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| CommandError::Calculation(error.to_string()))?;
    bytes.push(b'\n');
    write_atomic(path, &bytes, preserve_backup)
}

fn calculation_limitations(
    calculation: &wellforge_trajectory_contract::TrajectoryCalculation,
) -> Vec<String> {
    let mut limitations = Vec::new();
    if calculation
        .plan_survey_residuals
        .iter()
        .any(|value| value.plan.status != InterpolationStatus::Ok)
    {
        limitations.push("one or more survey stations are outside plan coverage".to_owned());
    }
    if calculation
        .targets
        .iter()
        .any(|value| value.basis == TargetBasis::NotReached)
    {
        limitations.push("one or more targets are not reached".to_owned());
    }
    if calculation.targets.iter().any(|value| {
        value
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.status == TargetStatus::InvalidGeometry)
    }) {
        limitations.push("one or more target evaluations have invalid geometry".to_owned());
    }
    if calculation.targets.iter().any(|value| {
        value
            .evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.status == TargetStatus::NumericalOverflow)
    }) {
        limitations
            .push("one or more target evaluations encountered numerical overflow".to_owned());
    }
    if calculation
        .slides
        .iter()
        .any(|value| value.response.is_none())
    {
        limitations.push("one or more slides are outside survey coverage".to_owned());
    }
    if calculation.slides.iter().any(|value| {
        value
            .response
            .as_ref()
            .is_some_and(|response| response.status == SlideStatus::LowInclination)
    }) {
        limitations
            .push("one or more slide responses are below the low-inclination threshold".to_owned());
    }
    if calculation.slides.iter().any(|value| {
        value
            .response
            .as_ref()
            .is_some_and(|response| response.status == SlideStatus::NumericalOverflow)
    }) {
        limitations.push("one or more slide responses encountered numerical overflow".to_owned());
    }
    if calculation
        .formations
        .iter()
        .any(|value| value.coverage != FormationCoverage::Ok)
    {
        limitations.push("one or more formation picks are outside survey coverage".to_owned());
    }
    limitations
}

fn target_triple() -> String {
    let vendor = if cfg!(target_vendor = "apple") {
        "apple"
    } else if cfg!(target_os = "windows") {
        "pc"
    } else {
        "unknown"
    };
    let operating_system = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        std::env::consts::OS
    };
    let environment = if cfg!(target_env = "gnu") {
        Some("gnu")
    } else if cfg!(target_env = "musl") {
        Some("musl")
    } else if cfg!(target_env = "msvc") {
        Some("msvc")
    } else {
        None
    };
    environment.map_or_else(
        || format!("{}-{vendor}-{operating_system}", std::env::consts::ARCH),
        |environment| {
            format!(
                "{}-{vendor}-{operating_system}-{environment}",
                std::env::consts::ARCH
            )
        },
    )
}

fn analyze_request(
    request: TrajectoryAnalysisRequest,
) -> Result<TrajectoryAnalysisResult, CommandError> {
    let request_hash = canonical::hash(&request)
        .map_err(|error| CommandError::InvalidRequest(error.to_string()))?;
    let calculation = analyze(&request).map_err(|error| match error {
        TrajectoryAnalysisError::InvalidRequest(errors) => CommandError::InvalidRequest(
            errors
                .into_iter()
                .map(|error| format!("{}: {}", error.code, error.message))
                .collect::<Vec<_>>()
                .join("; "),
        ),
        other => CommandError::Calculation(other.to_string()),
    })?;
    let limitations = calculation_limitations(&calculation);
    let mut result = TrajectoryAnalysisResult {
        contract_version: request.contract_version,
        analysis_id: request.analysis_id,
        sources: request.sources,
        status: if limitations.is_empty() {
            TrajectoryAnalysisStatus::Complete
        } else {
            TrajectoryAnalysisStatus::CompleteWithWarnings
        },
        applicability: ApplicabilityStatement {
            method: "minimum_curvature_closed_form".to_owned(),
            deterministic: true,
            limitations,
        },
        evidence: CalculationEvidence {
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            compiler_version: COMPILER_VERSION.to_owned(),
            target_triple: target_triple(),
            lockfile_hash: hex::encode(sha2::Sha256::digest(include_bytes!("../../../Cargo.lock"))),
            request_hash,
            result_hash: String::new(),
        },
        calculation,
    };
    let strict_round_trip: TrajectoryAnalysisResult = serde_json::from_slice(
        &serde_json::to_vec_pretty(&result)
            .map_err(|error| CommandError::Calculation(error.to_string()))?,
    )
    .map_err(|error| CommandError::Calculation(error.to_string()))?;
    result.evidence.result_hash = canonical::result_hash(&strict_round_trip)
        .map_err(|error| CommandError::Calculation(error.to_string()))?;
    Ok(result)
}

fn read_verified_result(
    path: &Path,
    expected_request_hash: &str,
) -> Result<TrajectoryAnalysisResult, CommandError> {
    let result: TrajectoryAnalysisResult = serde_json::from_slice(&read(path)?)
        .map_err(|error| CommandError::Integrity(error.to_string()))?;
    if result.evidence.request_hash != expected_request_hash {
        return Err(CommandError::Integrity("request hash mismatch".to_owned()));
    }
    let expected_result_hash = result.evidence.result_hash.clone();
    let actual_result_hash = canonical::result_hash(&result)
        .map_err(|error| CommandError::Integrity(error.to_string()))?;
    if expected_result_hash != actual_result_hash {
        return Err(CommandError::Integrity(format!(
            "result hash mismatch: expected {expected_result_hash}, actual {actual_result_hash}"
        )));
    }
    if !matches!(
        result.status,
        TrajectoryAnalysisStatus::Complete | TrajectoryAnalysisStatus::CompleteWithWarnings
    ) {
        return Err(CommandError::Integrity(
            "result does not have an accepted completed status".to_owned(),
        ));
    }
    Ok(result)
}

fn execute(command: Command) -> Result<(), CommandError> {
    match command {
        Command::Validate { input } => {
            read_request(&input)?;
        }
        Command::Run {
            input,
            output,
            diagnostics,
            no_backup,
        } => {
            let mut paths = vec![("input", input.as_path()), ("result", output.as_path())];
            if let Some(diagnostics) = diagnostics.as_deref() {
                paths.push(("diagnostics", diagnostics));
            }
            reject_path_collisions(&paths)?;
            let result = analyze_request(read_request(&input)?)?;
            let diagnostic_bytes = if diagnostics.is_some() {
                let analysis_id = result.analysis_id.to_string();
                let records = [
                    diagnostics::Diagnostic::info(
                        "request_validated",
                        "WF-TRAJECTORY-CLI-001",
                        &analysis_id,
                        &result.evidence.request_hash,
                        "strict request validation completed",
                    ),
                    diagnostics::Diagnostic::info(
                        "result_written",
                        "WF-TRAJECTORY-CLI-002",
                        &analysis_id,
                        &result.evidence.request_hash,
                        "deterministic trajectory result written atomically",
                    ),
                ];
                Some(
                    diagnostics::bytes(&records)
                        .map_err(|error| CommandError::Calculation(error.to_string()))?,
                )
            } else {
                None
            };
            write_pretty(&output, &result, !no_backup)?;
            if let Some((path, bytes)) = diagnostics.zip(diagnostic_bytes) {
                write_atomic(&path, &bytes, false)?;
            }
        }
        Command::VerifyResult {
            input,
            request_hash,
        } => {
            read_verified_result(&input, &request_hash)?;
        }
        Command::Bridge {
            input,
            output,
            request_hash,
        } => {
            reject_path_collisions(&[("input", input.as_path()), ("output", output.as_path())])?;
            let result = read_verified_result(&input, &request_hash)?;
            let bytes = bridge::build(&result).map_err(CommandError::Integrity)?;
            write_atomic(&output, &bytes, false)?;
        }
        Command::Schema { request, result } => {
            reject_path_collisions(&[
                ("request schema", request.as_path()),
                ("result schema", result.as_path()),
            ])?;
            write_pretty(&request, &schema_for!(TrajectoryAnalysisRequest), false)?;
            write_pretty(&result, &schema_for!(TrajectoryAnalysisResult), false)?;
        }
        Command::Version { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "name": "wellforge-trajectory",
                        "version": env!("CARGO_PKG_VERSION"),
                        "compiler_version": COMPILER_VERSION,
                    })
                );
            } else {
                println!("wellforge-trajectory {}", env!("CARGO_PKG_VERSION"));
            }
        }
    }
    Ok(())
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };
    if let Err(error) = execute(cli.command) {
        eprintln!("{error}");
        std::process::exit(error.exit_code());
    }
}

#[cfg(test)]
mod tests {
    use wellforge_trajectory_analysis::analyze;
    use wellforge_trajectory_contract::{SlideStatus, TargetStatus};

    use super::{calculation_limitations, reject_path_collisions_with_case_policy};

    #[test]
    fn windows_case_aliases_collide_for_run_bridge_and_schema_paths() {
        let directory = tempfile::tempdir().unwrap();

        let run_input = directory.path().join("request.json");
        let run_output = directory.path().join("RESULT.JSON");
        let run_diagnostics = directory.path().join("result.json");
        assert!(
            reject_path_collisions_with_case_policy(
                &[
                    ("input", run_input.as_path()),
                    ("result", run_output.as_path()),
                    ("diagnostics", run_diagnostics.as_path()),
                ],
                true,
            )
            .is_err()
        );

        let bridge_input = directory.path().join("BRIDGE.JSON");
        let bridge_output = directory.path().join("bridge.json");
        assert!(
            reject_path_collisions_with_case_policy(
                &[
                    ("input", bridge_input.as_path()),
                    ("output", bridge_output.as_path()),
                ],
                true,
            )
            .is_err()
        );

        let request_schema = directory.path().join("TRAJECTORY.SCHEMA.JSON");
        let result_schema = directory.path().join("trajectory.schema.json");
        assert!(
            reject_path_collisions_with_case_policy(
                &[
                    ("request schema", request_schema.as_path()),
                    ("result schema", result_schema.as_path()),
                ],
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn limitations_include_typed_target_and_slide_failures() {
        let request = wellforge_trajectory_fixtures::release_one_minimal_request();
        let mut calculation = analyze(&request).unwrap();
        calculation.targets[0].evaluation.as_mut().unwrap().status = TargetStatus::InvalidGeometry;
        calculation.targets[1].evaluation.as_mut().unwrap().status =
            TargetStatus::NumericalOverflow;
        calculation.slides[0].response.as_mut().unwrap().status = SlideStatus::LowInclination;
        let mut overflow_slide = calculation.slides[0].clone();
        overflow_slide.response.as_mut().unwrap().status = SlideStatus::NumericalOverflow;
        calculation.slides.push(overflow_slide);

        let limitations = calculation_limitations(&calculation);
        for expected in [
            "one or more survey stations are outside plan coverage",
            "one or more target evaluations have invalid geometry",
            "one or more target evaluations encountered numerical overflow",
            "one or more slides are outside survey coverage",
            "one or more slide responses are below the low-inclination threshold",
            "one or more slide responses encountered numerical overflow",
        ] {
            assert!(
                limitations.iter().any(|value| value == expected),
                "missing limitation {expected:?} from {limitations:?}"
            );
        }
    }
}
