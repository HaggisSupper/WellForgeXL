//! `WellForge` hydraulics engine command-line interface.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wellforge_hydraulics_contract::{
    AnalysisStatus, HydraulicsAnalysisRequest, HydraulicsAnalysisResult, validate_request,
};
use wellforge_hydraulics_core::{solve_hydraulics, solve_hydraulics_batch};

const MAX_BATCH_ANALYSES: usize = 128;

#[derive(Parser)]
#[command(
    name = "wellforge-hydraulics",
    version,
    about = "WellForge steady-state hydraulics engine (API RP 13D 7th Ed profile)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a request without running analysis.
    Validate {
        #[arg(long)]
        input: PathBuf,
    },
    /// Validate a bounded batch of requests without running analysis.
    ValidateBatch {
        #[arg(long)]
        input: PathBuf,
    },
    /// Run the steady-state pass and emit the result contract.
    Run {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run a bounded request batch and emit results in request order.
    RunBatch {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify a result against its request hash and embedded result hash.
    VerifyResult {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        request_hash: String,
    },
    /// Verify a result batch against its normalized request batch.
    VerifyBatch {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        result: PathBuf,
    },
    /// Write deterministic request and result JSON Schemas.
    Schema {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        result: PathBuf,
    },
    /// Print engine build identity.
    Version {
        #[arg(long)]
        json: bool,
    },
    /// Verify local engine health and embedded build metadata.
    Doctor,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HydraulicsRequestBatch {
    requests: Vec<HydraulicsAnalysisRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HydraulicsResultBatch {
    results: Vec<HydraulicsAnalysisResult>,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn result_payload_hash(result: &HydraulicsAnalysisResult) -> Result<String> {
    let bytes = serde_json::to_vec(result)?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    value["evidence"]["result_hash"] = serde_json::Value::String(String::new());
    Ok(hash(&serde_json::to_vec(&value)?))
}

fn read_request(path: &PathBuf) -> Result<HydraulicsAnalysisRequest> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let request: HydraulicsAnalysisRequest = serde_json::from_slice(&bytes)
        .context("request JSON does not match the strict contract")?;
    if let Err(errors) = validate_request(&request) {
        emit_json_error(
            "WF-HYD-REQ-001",
            errors
                .iter()
                .map(|error| format!("{}: {}", error.code, error.message))
                .collect::<Vec<_>>()
                .join("; "),
        );
        bail!("request validation failed with {} error(s)", errors.len());
    }
    Ok(request)
}

fn read_request_batch(path: &PathBuf) -> Result<HydraulicsRequestBatch> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let batch: HydraulicsRequestBatch = serde_json::from_slice(&bytes)
        .context("batch request JSON does not match the strict contract")?;
    if batch.requests.is_empty() || batch.requests.len() > MAX_BATCH_ANALYSES {
        emit_json_error(
            "WF-HYD-BATCH-001",
            format!("batch requests must contain between 1 and {MAX_BATCH_ANALYSES} analyses"),
        );
        bail!("invalid batch size: {}", batch.requests.len());
    }
    for (index, request) in batch.requests.iter().enumerate() {
        if let Err(errors) = validate_request(request) {
            emit_json_error(
                "WF-HYD-BATCH-002",
                errors
                    .iter()
                    .map(|error| format!("requests[{index}] {}: {}", error.code, error.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            );
            bail!(
                "batch request {index} failed validation with {} error(s)",
                errors.len()
            );
        }
    }
    Ok(batch)
}

fn normalized_request_hash(request: &HydraulicsAnalysisRequest) -> Result<String> {
    Ok(hash(&serde_json::to_vec(request)?))
}

fn attach_hashes(
    request: &HydraulicsAnalysisRequest,
    result: &mut HydraulicsAnalysisResult,
) -> Result<()> {
    result.evidence.request_hash = normalized_request_hash(request)?;
    result.evidence.result_hash = result_payload_hash(result)?;
    Ok(())
}

fn verify_result_value(result: &HydraulicsAnalysisResult, request_hash: &str) -> Result<()> {
    if result.evidence.request_hash != request_hash {
        bail!("result request hash does not match the validated request");
    }
    let expected_hash = result_payload_hash(result)?;
    if result.evidence.result_hash != expected_hash {
        bail!("result hash mismatch");
    }
    if matches!(result.status, AnalysisStatus::Failed) {
        bail!("hydraulics result is failed");
    }
    Ok(())
}

fn verify_result(path: &PathBuf, request_hash: &str) -> Result<()> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let result: HydraulicsAnalysisResult =
        serde_json::from_slice(&bytes).context("result JSON does not match the strict contract")?;
    verify_result_value(&result, request_hash)?;
    println!("{{\"status\":\"valid\"}}");
    Ok(())
}

fn verify_batch(request_path: &PathBuf, result_path: &PathBuf) -> Result<()> {
    let request_batch = read_request_batch(request_path)?;
    let result_bytes =
        fs::read(result_path).with_context(|| format!("cannot read {}", result_path.display()))?;
    let result_batch: HydraulicsResultBatch = serde_json::from_slice(&result_bytes)
        .context("batch result JSON does not match the strict contract")?;
    if result_batch.results.len() != request_batch.requests.len() {
        bail!("batch request and result counts do not match");
    }
    for (index, (request, result)) in request_batch
        .requests
        .iter()
        .zip(&result_batch.results)
        .enumerate()
    {
        if result.analysis_id != request.analysis_id {
            bail!("batch result {index} analysis ID mismatch");
        }
        let request_hash = normalized_request_hash(request)?;
        verify_result_value(result, &request_hash)
            .with_context(|| format!("batch result {index} failed verification"))?;
    }
    println!(
        "{{\"status\":\"valid\",\"analyses\":{}}}",
        result_batch.results.len()
    );
    Ok(())
}

fn doctor() -> Result<()> {
    let version = serde_json::json!({
        "engine": "wellforge-hydraulics",
        "version": env!("CARGO_PKG_VERSION"),
        "compiler": env!("WELLFORGE_RUSTC_VERSION_VERBOSE"),
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        "dependency_lock_hash": hash(include_bytes!("../../../Cargo.lock")),
    });
    if version["engine"] != "wellforge-hydraulics"
        || version["version"] != env!("CARGO_PKG_VERSION")
    {
        bail!("build metadata validation failed");
    }
    println!("{}", serde_json::to_string_pretty(&version)?);
    Ok(())
}

fn execute() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Validate { input } => {
            let request = read_request(&input)?;
            println!(
                "{{\"status\":\"valid\",\"request_hash\":\"{}\"}}",
                normalized_request_hash(&request)?
            );
        }
        Command::ValidateBatch { input } => {
            let batch = read_request_batch(&input)?;
            let request_hashes = batch
                .requests
                .iter()
                .map(normalized_request_hash)
                .collect::<Result<Vec<_>>>()?;
            println!(
                "{}",
                serde_json::json!({
                    "status": "valid",
                    "request_hashes": request_hashes,
                })
            );
        }
        Command::Run { input, output } => {
            let request = read_request(&input)?;
            let mut result = solve_hydraulics(&request).context("hydraulics solver failed")?;
            attach_hashes(&request, &mut result)?;
            let bytes = serde_json::to_vec_pretty(&result)?;
            fs::write(&output, &bytes)
                .with_context(|| format!("cannot write {}", output.display()))?;
            println!(
                "{{\"status\":\"ok\",\"sections\":{}}}",
                result.sections.len()
            );
        }
        Command::RunBatch { input, output } => {
            let batch = read_request_batch(&input)?;
            let mut results = solve_hydraulics_batch(&batch.requests)
                .context("hydraulics batch solver failed")?;
            for (request, result) in batch.requests.iter().zip(&mut results) {
                attach_hashes(request, result)?;
            }
            let result_batch = HydraulicsResultBatch { results };
            fs::write(&output, serde_json::to_vec_pretty(&result_batch)?)
                .with_context(|| format!("cannot write {}", output.display()))?;
            println!(
                "{{\"status\":\"ok\",\"analyses\":{}}}",
                result_batch.results.len()
            );
        }
        Command::VerifyResult {
            input,
            request_hash,
        } => verify_result(&input, &request_hash)?,
        Command::VerifyBatch { request, result } => verify_batch(&request, &result)?,
        Command::Schema { request, result } => {
            let request_schema = schema_for!(HydraulicsAnalysisRequest);
            let result_schema = schema_for!(HydraulicsAnalysisResult);
            fs::write(&request, serde_json::to_vec_pretty(&request_schema)?)?;
            fs::write(&result, serde_json::to_vec_pretty(&result_schema)?)?;
        }
        Command::Version { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "engine":"wellforge-hydraulics",
                        "version":env!("CARGO_PKG_VERSION"),
                        "compiler":env!("WELLFORGE_RUSTC_VERSION_VERBOSE"),
                        "target":format!("{}-{}",std::env::consts::ARCH,std::env::consts::OS),
                        "dependency_lock_hash":hash(include_bytes!("../../../Cargo.lock"))
                    })
                );
            } else {
                println!("wellforge-hydraulics {}", env!("CARGO_PKG_VERSION"));
            }
        }
        Command::Doctor => doctor()?,
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("ERROR: {error:#}");
        std::process::exit(2);
    }
}

#[derive(Serialize)]
struct CliError<'a> {
    error: &'a str,
    code: &'a str,
    message: String,
}

fn emit_json_error(code: &'static str, message: impl Into<String>) {
    let payload = CliError {
        error: "request_validation_failure",
        code,
        message: message.into(),
    };
    println!(
        "{}",
        serde_json::to_string(&payload).expect("serializing cli error")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use wellforge_hydraulics_fixtures::canonical_bingham_case;

    #[test]
    fn result_hash_survives_json_round_trip() {
        let request = canonical_bingham_case();
        let mut result = solve_hydraulics(&request).unwrap();
        result.evidence.request_hash = hash(&serde_json::to_vec(&request).unwrap());
        let before = result_payload_hash(&result).unwrap();
        let parsed: HydraulicsAnalysisResult =
            serde_json::from_slice(&serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        let after = result_payload_hash(&parsed).unwrap();
        assert_eq!(before, after);
    }
}
