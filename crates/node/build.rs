//! Build script for napi-rs bindings.
//!
//! # What
//!
//! This build script configures napi-rs to generate Node.js bindings for the
//! `sublime_node_tools` crate. It sets up the necessary build environment and
//! generates the required artifacts for Node.js integration.
//!
//! # How
//!
//! Uses `napi_build::setup()` to configure the build environment. This function:
//! - Sets up the correct linker flags for the target platform
//! - Configures the output directory for generated bindings
//! - Ensures proper symbol visibility for the native module
//!
//! # Why
//!
//! napi-rs requires this build script to properly link the Rust library as a
//! Node.js native addon. Without this configuration, the compiled library would
//! not be loadable by Node.js. The setup handles platform-specific requirements
//! for macOS, Windows, and Linux.

/// Entry point for the build script.
///
/// Configures napi-rs to generate Node.js bindings. This function is called
/// automatically by Cargo during the build process.
///
/// # Panics
///
/// This function does not panic under normal circumstances. The napi-build
/// setup is designed to be infallible for supported platforms.
fn main() {
    napi_build::setup();
}
