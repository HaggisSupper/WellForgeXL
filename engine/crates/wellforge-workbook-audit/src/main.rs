//! Command-line entry point for static workbook audits.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use wellforge_workbook_audit::audit_workbook;

#[derive(Debug, Parser)]
#[command(about = "Statically inventory workbook formulas without opening Excel")]
struct Arguments {
    /// Workbook to inspect.
    #[arg(long)]
    input: PathBuf,
    /// JSON file to create.
    #[arg(long)]
    output: PathBuf,
    /// Maximum accepted workbook size in bytes.
    #[arg(long, default_value_t = 536_870_912)]
    max_input_bytes: u64,
    /// Maximum emitted JSON size in bytes.
    #[arg(long, default_value_t = 536_870_912)]
    max_output_bytes: usize,
}

fn validate_distinct_paths(input: &Path, output: &Path) -> Result<()> {
    let canonical_input = fs::canonicalize(input)
        .with_context(|| format!("could not resolve input {}", input.display()))?;
    if output.exists() && same_file::is_same_file(&canonical_input, output)? {
        bail!("input and output resolve to the same file")
    }
    let output_directory = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let output_name = output
        .file_name()
        .context("output must identify a JSON file")?;
    let canonical_output = fs::canonicalize(output_directory)
        .with_context(|| format!("could not resolve {}", output_directory.display()))?
        .join(output_name);
    if canonical_input == canonical_output {
        bail!("input and output resolve to the same path")
    }
    Ok(())
}

fn main() -> Result<()> {
    let arguments = Arguments::parse();
    let output_directory = arguments
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_directory)
        .with_context(|| format!("could not create {}", output_directory.display()))?;
    validate_distinct_paths(&arguments.input, &arguments.output)?;
    let input_bytes = fs::metadata(&arguments.input)
        .with_context(|| format!("could not stat {}", arguments.input.display()))?
        .len();
    if input_bytes > arguments.max_input_bytes {
        bail!(
            "input contains {input_bytes} bytes, exceeding the {} byte limit",
            arguments.max_input_bytes
        )
    }
    let audit = audit_workbook(&arguments.input)?;
    let payload =
        serde_json::to_vec_pretty(&audit).context("could not serialize workbook audit")?;
    if payload.len() > arguments.max_output_bytes {
        bail!(
            "JSON contains {} bytes, exceeding the {} byte limit",
            payload.len(),
            arguments.max_output_bytes
        )
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".wellforge-workbook-audit-")
        .tempfile_in(output_directory)
        .with_context(|| {
            format!(
                "could not create temporary output in {}",
                output_directory.display()
            )
        })?;
    temporary
        .write_all(&payload)
        .with_context(|| format!("could not write {}", arguments.output.display()))?;
    temporary
        .as_file_mut()
        .sync_all()
        .with_context(|| format!("could not flush {}", arguments.output.display()))?;
    temporary
        .persist(&arguments.output)
        .with_context(|| format!("could not publish {}", arguments.output.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_distinct_paths;

    #[test]
    fn rejects_an_output_that_aliases_the_input() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let input = directory.path().join("book.xls");
        std::fs::write(&input, b"fixture").expect("fixture input");

        let error = validate_distinct_paths(&input, &input).expect_err("must reject alias");

        assert!(error.to_string().contains("same file"));
    }

    #[test]
    fn accepts_a_distinct_output_path() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let input = directory.path().join("book.xls");
        let output = directory.path().join("audit.json");
        std::fs::write(&input, b"fixture").expect("fixture input");

        validate_distinct_paths(&input, &output).expect("distinct output");
    }
}
