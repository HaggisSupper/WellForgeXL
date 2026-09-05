fn main() {
    // DuckDB's Windows file-lock diagnostics call the supported Restart Manager APIs.
    // The bundled DuckDB build does not currently emit this linker dependency itself.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=rstrtmgr");
}
