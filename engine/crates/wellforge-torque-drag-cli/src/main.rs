//! `WellForge` torque-and-drag engine command-line interface.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use schemars::schema_for;
use serde::Serialize;
use sha2::{Digest, Sha256};
use wellforge_torque_drag_contract::{
    TnDAnalysisRequest, TnDAnalysisResult, validate_request,
};
use wellforge_torque_drag_core::solve_soft_string;

#[derive(Parser)]
#[command(
    name = "wellforge-torque-drag",
    version,
    about = "WellForge torque-and-drag engine (soft string + API 7G envelope)"
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
    /// Run the soft-string pass and API 7G governing check.
    Run {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
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

fn result_payload_hash(result: &TnDAnalysisResult) -> Result<String> {
    let mut normalized = result.clone();
    normalized.evidence.result_hash = String::new();
    let value = serde_json::to_value(normalized)?;
    Ok(hash(&serde_json::to_vec(&value)?))
}

fn read_request(path: &PathBuf) -> Result<TnDAnalysisRequest> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let request: TnDAnalysisRequest = serde_json::from_slice(&bytes)
        .context("request JSON does not match the strict contract")?;
    if let Err(errors) = validate_request(&request) {
        emit_json_error(
            "WF-TND-REQ-001",
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

fn doctor() -> Result<()> {
    let version = serde_json::json!({
        "engine": "wellforge-torque-drag",
        "version": env!("CARGO_PKG_VERSION"),
        "compiler": env!("WELLFORGE_RUSTC_VERSION_VERBOSE"),
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        "dependency_lock_hash": hash(include_bytes!("../../../Cargo.lock")),
    });
    if version["engine"] != "wellforge-torque-drag"
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
            let _ = read_request(&input)?;
            println!("{{\"status\":\"valid\"}}");
        }
        Command::Run { input, output } => {
            let request = read_request(&input)?;
            let request_bytes = serde_json::to_vec(&request)?;
            let mut result = solve_soft_string(&request).context("soft-string solver failed")?;
            result.evidence.request_hash = hash(&request_bytes);
            let payload_hash = result_payload_hash(&result)?;
            result.evidence.result_hash = payload_hash;
            let bytes = serde_json::to_vec_pretty(&result)?;
            fs::write(&output, &bytes)
                .with_context(|| format!("cannot write {}", output.display()))?;
            println!("{{\"status\":\"ok\",\"stations\":{}}}", result.stations.len());
        }
        Command::Schema { request, result } => {
            let request_schema = schema_for!(TnDAnalysisRequest);
            let result_schema = schema_for!(TnDAnalysisResult);
            fs::write(&request, serde_json::to_vec_pretty(&request_schema)?)?;
            fs::write(&result, serde_json::to_vec_pretty(&result_schema)?)?;
        }
        Command::Version { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "engine":"wellforge-torque-drag",
                        "version":env!("CARGO_PKG_VERSION"),
                        "compiler":env!("WELLFORGE_RUSTC_VERSION_VERBOSE"),
                        "target":format!("{}-{}",std::env::consts::ARCH,std::env::consts::OS),
                        "dependency_lock_hash":hash(include_bytes!("../../../Cargo.lock"))
                    })
                );
            } else {
                println!("wellforge-torque-drag {}", env!("CARGO_PKG_VERSION"));
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
