use std::path::Path;
use std::{fs, process::Command};

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
}

impl EngineRunnerError {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::PathAlias => "ENGINE_PATH_ALIAS",
            Self::Launch(_) => "ENGINE_LAUNCH_FAILED",
            Self::Failed(_) => "ENGINE_EXECUTION_FAILED",
            Self::ReadResult(_) => "ENGINE_RESULT_READ_FAILED",
        }
    }
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
}
