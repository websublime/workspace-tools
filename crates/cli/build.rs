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

    // Note: Actual completion generation will be implemented when CLI structure is defined
    // This is a placeholder that compiles successfully
    println!("cargo:warning=Shell completions will be generated in OUT_DIR: {out_dir}");

    Ok(())
}
