//! `WellForge` BHA engine command-line interface.

use std::{
    fmt::Write as FmtWrite,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use schemars::schema_for;
use sha2::{Digest, Sha256};
use wellforge_bha_contract::{
    AnalysisStatus, BhaAnalysisRequest, BhaAnalysisResult, SolverEvidence, validate_request,
};

#[derive(Parser)]
#[command(
    name = "wellforge-bha",
    version,
    about = "WellForge BHA static and modal engine"
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
    /// Run static projection and modal analysis.
    Run {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run only the static stage and omit modal/FRF arrays from the output.
    SolveStatic {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Run static preload plus modal, FRF and Campbell stages.
    SolveModal {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify request and result hashes without writing workbook values.
    VerifyResult {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        request_hash: String,
    },
    /// Parse and verify result JSON in Rust, then emit the versioned VBA table bridge.
    Bridge {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
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
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn result_payload_hash(result: &BhaAnalysisResult) -> Result<String> {
    let mut normalized = result.clone();
    normalized.evidence.result_hash = String::new();
    let value = serde_json::to_value(normalized)?;
    Ok(hash(&serde_json::to_vec(&value)?))
}

fn read_request(path: &Path) -> Result<BhaAnalysisRequest> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let request: BhaAnalysisRequest = serde_json::from_slice(&bytes)
        .context("request JSON does not match the strict contract")?;
    if let Err(errors) = validate_request(&request) {
        for error in &errors {
            eprintln!("{}: {}", error.code, error.message);
        }
        bail!("request validation failed with {} error(s)", errors.len());
    }
    Ok(request)
}

fn run_analysis(request: BhaAnalysisRequest) -> Result<BhaAnalysisResult> {
    let request_bytes = serde_json::to_vec(&request)?;
    let bha_model = wellforge_bha_model::assemble_model(&request)?;
    let static_solution = wellforge_bha_static::solve_static(&bha_model, &request)?;
    let mode_results = wellforge_bha_modal::solve_modes(&bha_model, &request, &static_solution)?;
    let stop_hz = mode_results
        .last()
        .map_or(100.0, |mode| (mode.natural_frequency_hz * 1.25).max(1.0));
    let frequency_response =
        wellforge_bha_modal::solve_frequency_response(&static_solution, 0.1, stop_hz, 160)?;
    let campbell = wellforge_bha_modal::build_campbell_map(
        &mode_results,
        (request.operating.rpm * 2.0).max(300.0),
        61,
    );
    let mut warnings = Vec::new();
    if static_solution
        .nodes
        .iter()
        .any(|node| node.projected_clearance_m < 0.0)
    {
        warnings.push("WF-BHA-PROJECTION-001: projected OD crosses the hole envelope; this is an indication, not a solved contact force".to_owned());
    }
    let result = BhaAnalysisResult {
        contract_version: request.contract_version,
        analysis_id: request.analysis_id,
        status: if warnings.is_empty() {
            AnalysisStatus::Converged
        } else {
            AnalysisStatus::Warning
        },
        sources: request.sources,
        static_nodes: static_solution.nodes,
        contacts: Vec::new(),
        modes: mode_results,
        frequency_response,
        campbell,
        warnings,
        evidence: SolverEvidence {
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            compiler: "rustc 1.98.0 (pinned)".to_owned(),
            target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            dependency_lock_hash: hash(include_bytes!("../../../Cargo.lock")),
            request_hash: hash(&request_bytes),
            result_hash: String::new(),
            iterations: 1,
            residual_norm: static_solution.residual_norm,
            converged: true,
        },
    };
    Ok(result)
}

fn write_pretty(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.flush()?;
    let backup = path.with_extension("wellforge-backup");
    if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(path, &backup)?;
    }
    if let Err(error) = temporary.persist(path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(error.error)
            .with_context(|| format!("cannot atomically write {}", path.display()));
    }
    if backup.exists() {
        fs::remove_file(backup)?;
    }
    Ok(())
}

fn read_verified_result(path: &Path, request_hash: &str) -> Result<BhaAnalysisResult> {
    let result: BhaAnalysisResult = serde_json::from_slice(&fs::read(path)?)
        .context("result JSON does not match the strict contract")?;
    if result.evidence.request_hash != request_hash {
        bail!("result request hash mismatch");
    }
    let expected = result.evidence.result_hash.clone();
    let actual = result_payload_hash(&result)?;
    if actual != expected {
        bail!("result payload hash mismatch: expected {expected}, actual {actual}");
    }
    if !result.evidence.converged {
        bail!("result evidence is non-converged");
    }
    Ok(result)
}

fn bridge_bytes(result: &BhaAnalysisResult) -> Vec<u8> {
    let mut output = String::new();
    writeln!(
        output,
        "H\t1.0.0\t{}\t{}\t{}\t{}\t{:?}\t{}",
        result.analysis_id,
        result.evidence.request_hash,
        result.evidence.result_hash,
        result.evidence.engine_version,
        result.status,
        result.evidence.converged
    )
    .expect("writing to a String cannot fail");
    for node in &result.static_nodes {
        writeln!(
            output,
            "S\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}\t{:.17e}",
            node.md_m,
            node.x_m,
            node.y_m,
            node.od_radius_m,
            node.id_radius_m,
            node.hole_radius_m,
            node.projected_clearance_m,
            node.bending_moment_n_m,
            node.bending_stress_pa
        )
        .expect("writing to a String cannot fail");
    }
    for mode in &result.modes {
        writeln!(
            output,
            "M\t{}\t{:.17e}\t{:.17e}",
            mode.mode_number, mode.natural_frequency_hz, mode.critical_speed_rpm
        )
        .expect("writing to a String cannot fail");
        for (node, shape) in result.static_nodes.iter().zip(&mode.normalized_shape) {
            writeln!(
                output,
                "P\t{}\t{:.17e}\t{:.17e}",
                mode.mode_number, node.md_m, shape
            )
            .expect("writing to a String cannot fail");
        }
    }
    for point in &result.frequency_response {
        writeln!(
            output,
            "F\t{:.17e}\t{:.17e}\t{:.17e}",
            point.frequency_hz, point.receptance_m_n, point.phase_deg
        )
        .expect("writing to a String cannot fail");
    }
    for point in &result.campbell {
        writeln!(
            output,
            "C\t{}\t{:.17e}\t{:.17e}\t{:.17e}",
            point.order, point.rpm, point.excitation_frequency_hz, point.nearest_mode_margin_hz
        )
        .expect("writing to a String cannot fail");
    }
    output.into_bytes()
}

fn write_result(path: &Path, mut result: BhaAnalysisResult) -> Result<()> {
    result.evidence.result_hash = String::new();
    write_pretty(path, &result)?;
    let parsed: BhaAnalysisResult = serde_json::from_slice(&fs::read(path)?)?;
    result.evidence.result_hash = result_payload_hash(&parsed)?;
    write_pretty(path, &result)
}

fn execute() -> Result<()> {
    match Cli::parse().command {
        Command::Validate { input } => {
            let request = read_request(&input)?;
            println!("{}", hash(&serde_json::to_vec(&request)?));
        }
        Command::Run { input, output } | Command::SolveModal { input, output } => {
            let result = run_analysis(read_request(&input)?)?;
            write_result(&output, result)?;
        }
        Command::SolveStatic { input, output } => {
            let mut result = run_analysis(read_request(&input)?)?;
            result.modes.clear();
            result.frequency_response.clear();
            result.campbell.clear();
            write_result(&output, result)?;
        }
        Command::VerifyResult {
            input,
            request_hash,
        } => {
            read_verified_result(&input, &request_hash)?;
            println!("valid");
        }
        Command::Bridge {
            input,
            output,
            request_hash,
        } => {
            let result = read_verified_result(&input, &request_hash)?;
            write_atomic(&output, &bridge_bytes(&result))?;
        }
        Command::Schema { request, result } => {
            write_pretty(&request, &schema_for!(BhaAnalysisRequest))?;
            write_pretty(&result, &schema_for!(BhaAnalysisResult))?;
        }
        Command::Version { json } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"engine":"wellforge-bha","version":env!("CARGO_PKG_VERSION"),"compiler":"rustc 1.98.0 (pinned)","target":format!("{}-{}",std::env::consts::ARCH,std::env::consts::OS),"dependency_lock_hash":hash(include_bytes!("../../../Cargo.lock"))})
                );
            } else {
                println!("wellforge-bha {}", env!("CARGO_PKG_VERSION"));
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("ERROR: {error:#}");
        std::process::exit(2);
    }
}
