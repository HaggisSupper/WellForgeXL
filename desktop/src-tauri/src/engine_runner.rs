use std::path::{Path, PathBuf};
use std::{fs, process::Command};

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EngineRunnerError {
    #[error("engine input and output paths must differ")]
    PathAlias,
    #[error("engine could not be launched: {0}")]
    Launch(String),
    #[error("engine exited unsuccessfully with status {0}")]
    Failed(String),
    #[error("engine result could not be read: {0}")]
    ReadResult(String),
    #[error("verified trajectory executable was not found")]
    ExecutableNotFound,
    #[error("trajectory executable checksum sidecar is missing")]
    ChecksumMissing,
    #[error("trajectory executable checksum does not match")]
    ChecksumMismatch,
    #[error("trajectory executable could not be inspected: {0}")]
    Inspect(String),
    #[error("engine request and result paths must remain inside the project workspace")]
    PathOutsideWorkspace,
}

impl EngineRunnerError {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::PathAlias => "ENGINE_PATH_ALIAS",
            Self::Launch(_) => "ENGINE_LAUNCH_FAILED",
            Self::Failed(_) => "ENGINE_EXECUTION_FAILED",
            Self::ReadResult(_) => "ENGINE_RESULT_READ_FAILED",
            Self::ExecutableNotFound => "ENGINE_EXECUTABLE_NOT_FOUND",
            Self::ChecksumMissing => "ENGINE_CHECKSUM_MISSING",
            Self::ChecksumMismatch => "ENGINE_CHECKSUM_MISMATCH",
            Self::Inspect(_) => "ENGINE_EXECUTABLE_INSPECTION_FAILED",
            Self::PathOutsideWorkspace => "ENGINE_PATH_OUTSIDE_WORKSPACE",
        }
    }
}

pub(crate) fn run_trajectory_engine_in_workspace(
    executable: &Path,
    workspace: &Path,
    input: &Path,
    output: &Path,
) -> Result<Vec<u8>, EngineRunnerError> {
    let workspace = normalized_path(workspace);
    if !is_within(&workspace, input) || !is_within(&workspace, output) {
        return Err(EngineRunnerError::PathOutsideWorkspace);
    }
    run_trajectory_engine(executable, input, output)
}

pub(crate) fn discover_trajectory_executable(
    roots: &[&Path],
) -> Result<PathBuf, EngineRunnerError> {
    for root in roots {
        for relative in [
            Path::new("wellforge-trajectory.exe"),
            Path::new("engines/wellforge-trajectory.exe"),
            Path::new("outputs/vba-engine/wellforge-trajectory.exe"),
        ] {
            let executable = root.join(relative);
            if executable.is_file() {
                verify_sidecar(&executable)?;
                return Ok(executable);
            }
        }
    }
    Err(EngineRunnerError::ExecutableNotFound)
}

fn verify_sidecar(executable: &Path) -> Result<(), EngineRunnerError> {
    let sidecar = executable.with_extension("exe.sha256");
    let expected = fs::read_to_string(&sidecar)
        .map_err(|_| EngineRunnerError::ChecksumMissing)?
        .trim()
        .to_ascii_lowercase();
    if expected.len() != 64
        || !expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(EngineRunnerError::ChecksumMissing);
    }
    let bytes =
        fs::read(executable).map_err(|error| EngineRunnerError::Inspect(error.to_string()))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err(EngineRunnerError::ChecksumMismatch);
    }
    Ok(())
}

pub(crate) fn run_trajectory_engine(
    executable: &Path,
    input: &Path,
    output: &Path,
) -> Result<Vec<u8>, EngineRunnerError> {
    if normalized_path(input) == normalized_path(output) {
        return Err(EngineRunnerError::PathAlias);
    }

    let status = Command::new(executable)
        .args(["run", "--input"])
        .arg(input)
        .args(["--output"])
        .arg(output)
        .arg("--no-backup")
        .status()
        .map_err(|error| EngineRunnerError::Launch(error.to_string()))?;
    if !status.success() {
        return Err(EngineRunnerError::Failed(status.to_string()));
    }
    fs::read(output).map_err(|error| EngineRunnerError::ReadResult(error.to_string()))
}

fn normalized_path(path: &Path) -> std::path::PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_owned()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    })
}

fn is_within(root: &Path, child: &Path) -> bool {
    normalized_path(child).starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_identical_input_and_output_paths_before_launch() {
        let error = run_trajectory_engine(
            Path::new("engine.exe"),
            Path::new("case.json"),
            Path::new("case.json"),
        )
        .expect_err("input/output alias must be rejected");

        assert_eq!(error.code(), "ENGINE_PATH_ALIAS");
    }

    #[test]
    fn discovers_only_a_checksum_verified_packaged_engine() {
        let root = tempfile::tempdir().expect("temporary root creates");
        let executable = root.path().join("wellforge-trajectory.exe");
        let bytes = b"verified engine";
        fs::write(&executable, bytes).expect("engine fixture writes");
        fs::write(
            executable.with_extension("exe.sha256"),
            format!("{:x}\n", Sha256::digest(bytes)),
        )
        .expect("checksum sidecar writes");

        let found = discover_trajectory_executable(&[root.path()]).expect("engine discovers");

        assert_eq!(found, executable);
    }

    #[test]
    fn rejects_request_and_result_paths_outside_the_project_workspace() {
        let root = tempfile::tempdir().expect("temporary root creates");
        let outside = tempfile::tempdir().expect("outside root creates");

        let error = run_trajectory_engine_in_workspace(
            Path::new("engine.exe"),
            root.path(),
            &outside.path().join("request.json"),
            &root.path().join("result.json"),
        )
        .expect_err("request outside workspace must be rejected");

        assert_eq!(error.code(), "ENGINE_PATH_OUTSIDE_WORKSPACE");
    }
}
