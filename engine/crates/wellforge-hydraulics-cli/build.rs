//! Captures the exact compiler identity used to build the torque-drag executable.

use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTC");

    let rustc = PathBuf::from(
        env::var_os("RUSTC").expect("Cargo must provide RUSTC to capture compiler identity"),
    );
    let output = Command::new(&rustc)
        .arg("-Vv")
        .output()
        .unwrap_or_else(|error| panic!("failed to execute {} -Vv: {error}", rustc.display()));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "{} -Vv failed with {}: {stderr}",
            rustc.display(),
            output.status
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .expect("successful RUSTC -Vv output must be valid UTF-8 compiler identity");
    let lines = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let valid = lines.first().is_some_and(|line| line.starts_with("rustc "))
        && ["commit-hash:", "host:", "release:"]
            .into_iter()
            .all(|field| lines.iter().any(|line| line.starts_with(field)))
        && !stdout.contains('\0');
    assert!(
        valid,
        "successful RUSTC -Vv output was not a valid compiler identity"
    );

    println!(
        "cargo::rustc-env=WELLFORGE_RUSTC_VERSION_VERBOSE={}",
        lines.join(" | ")
    );
}
