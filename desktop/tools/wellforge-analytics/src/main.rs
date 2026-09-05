use std::{env, fs, path::PathBuf, process::ExitCode};

use wellforge_analytics::reconcile_files;

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let (
        Some(manifest_flag),
        Some(manifest),
        Some(records_flag),
        Some(records),
        Some(report_flag),
        Some(report),
    ) = (
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
    )
    else {
        eprintln!(
            "usage: wellforge-analytics --manifest <manifest.json> --records <records.jsonl> --report <report.json>"
        );
        return ExitCode::from(2);
    };
    if arguments.next().is_some()
        || manifest_flag != "--manifest"
        || records_flag != "--records"
        || report_flag != "--report"
    {
        eprintln!(
            "only --manifest, --records, and --report are accepted; database access and promotion are unavailable"
        );
        return ExitCode::from(2);
    }

    match reconcile_files(PathBuf::from(manifest), PathBuf::from(records)) {
        Ok(report_value) => match serde_json::to_vec_pretty(&report_value) {
            Ok(json) => match fs::write(report, json) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("unable to write report: {error}");
                    ExitCode::from(1)
                }
            },
            Err(error) => {
                eprintln!("unable to encode report: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("reconciliation failed: {error}");
            ExitCode::from(1)
        }
    }
}
