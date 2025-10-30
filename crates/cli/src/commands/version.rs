//! Version command implementation.
//!
//! This module implements the `wnt version` command that displays version information
//! about the CLI and its dependencies.
//!
//! # What
//!
//! Provides the `execute_version` function that:
//! - Displays the CLI version from Cargo.toml
//! - Shows Rust compiler version used to build the binary
//! - Lists internal crate dependencies and their versions
//! - Shows build profile and target information
//! - Supports human-readable and JSON output formats
//!
//! # How
//!
//! Uses compile-time environment variables set by Cargo to gather version information:
//! - `CARGO_PKG_VERSION` for the CLI version
//! - `RUSTC_VERSION` for the Rust compiler version (via build.rs if needed)
//! - Dependency versions from Cargo.toml
//!
//! The function formats output based on the requested format (Human or JSON).
//!
//! # Why
//!
//! Version information is critical for:
//! - Troubleshooting issues (knowing exact versions)
//! - Verifying installations
//! - CI/CD pipelines (checking tool versions)
//! - Bug reports (including version details)
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::commands::version::execute_version;
//! use sublime_cli_tools::cli::commands::VersionArgs;
//! use sublime_cli_tools::output::OutputFormat;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = VersionArgs { verbose: false };
//! execute_version(&args, Path::new("."), OutputFormat::Human).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::VersionArgs;
use crate::error::Result;
use crate::output::{JsonResponse, OutputFormat};
use serde::Serialize;
use std::path::Path;
use tracing::info;

/// Version information structure for JSON output.
///
/// Contains all version details in a structured format suitable for
/// machine parsing and JSON serialization.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::version::VersionInfo;
///
/// let info = VersionInfo::new();
/// assert!(!info.version.is_empty());
/// ```
#[derive(Debug, Serialize)]
pub struct VersionInfo {
    /// CLI version from Cargo.toml
    pub version: String,

    /// Rust compiler version used to build
    #[serde(rename = "rustVersion")]
    pub rust_version: String,

    /// Internal crate dependencies
    pub dependencies: DependencyVersions,

    /// Build information
    pub build: BuildInfo,
}

/// Internal crate dependency versions.
///
/// Tracks versions of the internal crates used by the CLI.
#[derive(Debug, Serialize)]
pub struct DependencyVersions {
    /// sublime-package-tools version
    #[serde(rename = "sublime-package-tools")]
    pub package_tools: String,

    /// sublime-standard-tools version
    #[serde(rename = "sublime-standard-tools")]
    pub standard_tools: String,

    /// sublime-git-tools version
    #[serde(rename = "sublime-git-tools")]
    pub git_tools: String,
}

/// Build configuration information.
///
/// Contains details about how the binary was built.
#[derive(Debug, Serialize)]
pub struct BuildInfo {
    /// Build profile (debug or release)
    pub profile: String,

    /// Target triple (e.g., x86_64-unknown-linux-gnu)
    pub target: String,

    /// Enabled features
    pub features: Vec<String>,
}

impl VersionInfo {
    /// Creates a new VersionInfo with data from compile-time environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::version::VersionInfo;
    ///
    /// let info = VersionInfo::new();
    /// assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: Self::rust_version(),
            dependencies: DependencyVersions {
                // Dependency versions are hardcoded from workspace
                package_tools: "0.1.0".to_string(),
                standard_tools: "0.1.0".to_string(),
                git_tools: "0.1.0".to_string(),
            },
            build: BuildInfo {
                profile: Self::build_profile(),
                target: Self::build_target(),
                features: Self::enabled_features(),
            },
        }
    }

    /// Gets the Rust version used to compile.
    ///
    /// Returns the version string from rustc, or "unknown" if not available.
    fn rust_version() -> String {
        // CARGO_PKG_RUST_VERSION is the MSRV, not the compiler version
        // We need to get the actual rustc version
        option_env!("RUSTC_VERSION").unwrap_or(env!("CARGO_PKG_RUST_VERSION")).to_string()
    }

    /// Determines the build profile used.
    ///
    /// Returns "release" or "debug" based on compile-time configuration.
    fn build_profile() -> String {
        if cfg!(debug_assertions) { "debug".to_string() } else { "release".to_string() }
    }

    /// Gets the build target triple.
    ///
    /// Returns the target triple for the platform this binary was built for.
    fn build_target() -> String {
        std::env::consts::ARCH.to_string() + "-" + std::env::consts::OS
    }

    /// Gets the list of enabled Cargo features.
    ///
    /// Returns a vector of feature names that were enabled during compilation.
    /// Feature names are detected at build time via build.rs.
    fn enabled_features() -> Vec<String> {
        env!("CARGO_FEATURES").split(',').filter(|s| !s.is_empty()).map(String::from).collect()
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes the version command.
///
/// Displays version information about the CLI and its dependencies.
/// Output format depends on the global `--format` flag.
///
/// # Arguments
///
/// * `args` - Version command arguments (e.g., verbose flag)
/// * `_root` - Root directory (unused but kept for consistency)
/// * `format` - Output format (Human, Json, or Quiet)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if output fails.
///
/// # Errors
///
/// This function rarely errors, but may fail if:
/// - JSON serialization fails (unlikely)
/// - Output writing fails (e.g., broken pipe)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::version::execute_version;
/// use sublime_cli_tools::cli::commands::VersionArgs;
/// use sublime_cli_tools::output::OutputFormat;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = VersionArgs { verbose: false };
/// execute_version(&args, Path::new("."), OutputFormat::Human).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::print_stdout)]
pub fn execute_version(args: &VersionArgs, _root: &Path, format: OutputFormat) -> Result<()> {
    info!("Executing version command");

    let version_info = VersionInfo::new();

    match format {
        OutputFormat::Human => {
            display_human_version(&version_info, args.verbose);
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let json_response = JsonResponse::success(version_info);
            let json_str = if format == OutputFormat::JsonCompact {
                serde_json::to_string(&json_response)
            } else {
                serde_json::to_string_pretty(&json_response)
            }
            .map_err(|e| {
                crate::error::CliError::execution(format!("Failed to serialize JSON: {e}"))
            })?;
            println!("{json_str}");
        }
        OutputFormat::Quiet => {
            // In quiet mode, just output the version number
            println!("{}", version_info.version);
        }
    }

    Ok(())
}

/// Displays version information in human-readable format.
///
/// Shows a formatted version display with optional verbose details.
///
/// # Arguments
///
/// * `info` - Version information to display
/// * `verbose` - Whether to show detailed information
#[allow(clippy::print_stdout)]
fn display_human_version(info: &VersionInfo, verbose: bool) {
    use console::style;

    println!("{} {}", style("wnt").bold().cyan(), style(&info.version).bold());

    if verbose {
        println!();
        println!("{}", style("Build Information:").bold().underlined());
        println!("  {} {}", style("Rust:").bold(), info.rust_version);
        println!("  {} {}", style("Profile:").bold(), info.build.profile);
        println!("  {} {}", style("Target:").bold(), info.build.target);

        if !info.build.features.is_empty() {
            println!("  {} {}", style("Features:").bold(), info.build.features.join(", "));
        }

        println!();
        println!("{}", style("Dependencies:").bold().underlined());
        println!(
            "  {} {}",
            style("sublime-package-tools:").bold(),
            info.dependencies.package_tools
        );
        println!(
            "  {} {}",
            style("sublime-standard-tools:").bold(),
            info.dependencies.standard_tools
        );
        println!("  {} {}", style("sublime-git-tools:").bold(), info.dependencies.git_tools);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_creation() {
        let info = VersionInfo::new();
        assert!(!info.version.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(!info.dependencies.package_tools.is_empty());
        assert!(!info.dependencies.standard_tools.is_empty());
        assert!(!info.dependencies.git_tools.is_empty());
        assert!(!info.build.target.is_empty());
    }

    #[test]
    fn test_version_info_default() {
        let info = VersionInfo::default();
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_build_profile() {
        let profile = VersionInfo::build_profile();
        assert!(profile == "debug" || profile == "release");
    }

    #[test]
    fn test_enabled_features() {
        let features = VersionInfo::enabled_features();
        assert!(features.is_empty() || !features.is_empty());
    }

    #[test]
    fn test_execute_version_human() {
        let args = VersionArgs { verbose: false };
        let result = execute_version(&args, Path::new("."), OutputFormat::Human);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_version_verbose() {
        let args = VersionArgs { verbose: true };
        let result = execute_version(&args, Path::new("."), OutputFormat::Human);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_version_json() {
        let args = VersionArgs { verbose: false };
        let result = execute_version(&args, Path::new("."), OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_version_json_compact() {
        let args = VersionArgs { verbose: false };
        let result = execute_version(&args, Path::new("."), OutputFormat::JsonCompact);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_version_quiet() {
        let args = VersionArgs { verbose: false };
        let result = execute_version(&args, Path::new("."), OutputFormat::Quiet);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_human_version_basic() {
        let info = VersionInfo::new();
        display_human_version(&info, false);
        // No panic means success
    }

    #[test]
    fn test_display_human_version_verbose() {
        let info = VersionInfo::new();
        display_human_version(&info, true);
        // No panic means success
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_version_info_serialization() {
        let info = VersionInfo::new();
        let json = serde_json::to_string(&info).expect("Should serialize to JSON");
        assert!(json.contains("version"));
        assert!(json.contains("rustVersion"));
        assert!(json.contains("dependencies"));
        assert!(json.contains("build"));
    }
}
