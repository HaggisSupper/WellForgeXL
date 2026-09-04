//! `WellForge` hydraulics engine command-line interface.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use schemars::schema_for;
use serde::Serialize;
use sha2::{Digest, Sha256};
use wellforge_hydraulics_contract::{
    AnalysisStatus, HydraulicsAnalysisRequest, HydraulicsAnalysisResult, validate_request,
};
use wellforge_hydraulics_core::solve_hydraulics;

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
    /// Run the steady-state pass and emit the result contract.
    Run {
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

fn verify_result(path: &PathBuf, request_hash: &str) -> Result<()> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let result: HydraulicsAnalysisResult =
        serde_json::from_slice(&bytes).context("result JSON does not match the strict contract")?;
    if result.evidence.request_hash != request_hash {
        bail!("result request hash does not match the validated request");
    }
    let expected_hash = result_payload_hash(&result)?;
    if result.evidence.result_hash != expected_hash {
        bail!("result hash mismatch");
    }
    if matches!(result.status, AnalysisStatus::Failed) {
        bail!("hydraulics result is failed");
    }
    println!("{{\"status\":\"valid\"}}");
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
            let request_bytes = serde_json::to_vec(&request)?;
            println!(
                "{{\"status\":\"valid\",\"request_hash\":\"{}\"}}",
                hash(&request_bytes)
            );
        }
        Command::Run { input, output } => {
            let request = read_request(&input)?;
            let request_bytes = serde_json::to_vec(&request)?;
            let mut result = solve_hydraulics(&request).context("hydraulics solver failed")?;
            result.evidence.request_hash = hash(&request_bytes);
            let payload_hash = result_payload_hash(&result)?;
            result.evidence.result_hash = payload_hash;
            let bytes = serde_json::to_vec_pretty(&result)?;
            fs::write(&output, &bytes)
                .with_context(|| format!("cannot write {}", output.display()))?;
            println!(
                "{{\"status\":\"ok\",\"sections\":{}}}",
                result.sections.len()
            );
        }
        Command::VerifyResult {
            input,
            request_hash,
        } => verify_result(&input, &request_hash)?,
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
