//! Build script for the CLI crate.
//!
//! This script generates shell completion files at build time for bash, zsh, fish, and PowerShell.
//! The completions are generated from the Clap CLI definition and placed in the OUT_DIR.
//!
//! # What
//! Generates shell completion scripts for the `wnt` CLI tool.
//!
//! # How
//! Uses clap_complete to generate completion files from the CLI definition at build time.
//!
//! # Why
//! Shell completions improve user experience by providing command and argument completion
//! in various shells. Generating at build time ensures completions are always in sync
//! with the CLI definition.

use std::env;
use std::io::Error;
use std::process::Command;

fn main() -> Result<(), Error> {
    // Get the output directory for build artifacts
    let out_dir = env::var("OUT_DIR").map_err(|e| {
        Error::new(
            std::io::ErrorKind::NotFound,
            format!("OUT_DIR environment variable not set: {e}"),
        )
    })?;

    println!("cargo:rerun-if-changed=src/cli/mod.rs");
    println!("cargo:rerun-if-changed=src/cli/commands.rs");
    println!("cargo:rerun-if-changed=src/cli/args.rs");

    // Capture rustc version at build time
    if let Ok(output) = Command::new("rustc").arg("--version").output()
        && let Ok(version) = String::from_utf8(output.stdout)
    {
        let version = version.trim();
        println!("cargo:rustc-env=RUSTC_VERSION={version}");
    }

    // Capture enabled cargo features
    // CARGO_FEATURE_<name> environment variables are set by Cargo for each enabled feature
    let features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            if key.starts_with("CARGO_FEATURE_") {
                // Extract feature name and convert back to kebab-case
                let feature = key
                    .strip_prefix("CARGO_FEATURE_")
                    .map(|s| s.to_lowercase().replace('_', "-"))?;
                Some(feature)
            } else {
                None
            }
        })
        .collect();

    // Set as comma-separated list
    let features_list = features.join(",");
    println!("cargo:rustc-env=CARGO_FEATURES={features_list}");

    // Note: Actual completion generation will be implemented when CLI structure is defined
    // This is a placeholder that compiles successfully
    println!("cargo:warning=Shell completions will be generated in OUT_DIR: {out_dir}");

    Ok(())
}
