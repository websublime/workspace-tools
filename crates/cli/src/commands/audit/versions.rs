//! Version consistency audit command implementation.
//!
//! This module provides focused version consistency audit functionality for internal dependencies.
//!
//! # What
//!
//! Provides:
//! - `execute_version_consistency_audit` function - Main entry point for version consistency audits
//! - Detection of internal dependency version inconsistencies
//! - Identification of packages using different versions of the same internal dependency
//! - Recommendations for aligning versions (preferring workspace: protocol)
//! - Actionable insights for maintaining consistent internal dependency versions
//!
//! # How
//!
//! The execution flow:
//! 1. Initialize audit manager and configuration
//! 2. Execute version consistency analysis via `AuditManager`
//! 3. Detect internal packages with multiple version specifications
//! 4. Generate detailed report with inconsistency details and recommendations
//! 5. Display results via Output system or write to file
//!
//! # Why
//!
//! A dedicated version consistency audit provides:
//! - Early detection of version mismatches in internal dependencies
//! - Prevention of version conflicts that may cause runtime issues
//! - Encouragement of workspace protocol usage for better dependency management
//! - Clear visibility into inconsistent version specifications across packages

use crate::commands::audit::types::{MinSeverity, parse_verbosity};
use crate::error::{CliError, Result};
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::{AuditManager, IssueSeverity, Verbosity};

/// Executes a focused version consistency audit.
///
/// This function provides detailed version consistency analysis and reporting,
/// focusing specifically on internal dependencies that have inconsistent version
/// specifications across the workspace.
///
/// # Arguments
///
/// * `output` - The output context for formatting and display
/// * `workspace_root` - The workspace root directory
/// * `config_path` - Optional path to configuration file
/// * `min_severity` - Minimum severity level for filtering issues
/// * `verbosity` - Level of detail in the report
/// * `output_file` - Optional path to write report to file
///
/// # Returns
///
/// Returns `Ok(())` if the version consistency audit completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be loaded or is invalid
/// - Workspace root is invalid
/// - Audit manager initialization fails
/// - Version consistency analysis fails
/// - Report generation fails
/// - File I/O operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::versions::execute_version_consistency_audit;
/// use sublime_cli_tools::commands::audit::types::MinSeverity;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_version_consistency_audit(
///     &output,
///     workspace_root,
///     None,
///     MinSeverity::Info,
///     "normal",
///     None,
/// ).await?;
/// ```
pub async fn execute_version_consistency_audit(
    output: &Output,
    workspace_root: &Path,
    config_path: Option<&Path>,
    min_severity: MinSeverity,
    verbosity_str: &str,
    output_file: Option<&Path>,
) -> Result<()> {
    // Parse verbosity
    let verbosity = parse_verbosity(verbosity_str)?;

    // Load configuration
    let config = load_audit_config(config_path).await?;

    // Initialize audit manager
    output.info("Initializing version consistency audit...")?;
    let audit_manager = AuditManager::new(workspace_root.to_path_buf(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to initialize audit manager: {e}")))?;

    // Execute version consistency audit
    output.info("Analyzing internal dependency versions...")?;
    let version_consistency = audit_manager
        .audit_version_consistency()
        .await
        .map_err(|e| CliError::execution(format!("Version consistency audit failed: {e}")))?;

    // Display results
    format_version_consistency_report(&version_consistency, min_severity, verbosity, output)?;

    // Write to file if requested
    if let Some(file_path) = output_file {
        write_version_consistency_report_to_file(&version_consistency, file_path)?;
        output.success(&format!("Report written to {}", file_path.display()))?;
    }

    Ok(())
}

/// Formats and displays a version consistency audit report.
///
/// This function formats version consistency audit results with detailed analysis of
/// internal dependency version inconsistencies and actionable recommendations.
///
/// # Arguments
///
/// * `version_consistency` - The version consistency audit section results
/// * `min_severity` - Minimum severity level to display
/// * `verbosity` - Level of detail in the report
/// * `output` - Output context for formatting
///
/// # Returns
///
/// Returns `Ok(())` if the report was successfully formatted and displayed.
///
/// # Errors
///
/// Returns an error if output operations fail.
fn format_version_consistency_report(
    version_consistency: &sublime_pkg_tools::audit::VersionConsistencyAuditSection,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    // Display header
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("         VERSION CONSISTENCY AUDIT REPORT          ")?;
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("")?;

    // Filter issues by severity
    let filtered_issues: Vec<_> = version_consistency
        .issues
        .iter()
        .filter(|issue| {
            matches!(
                (min_severity, &issue.severity),
                (MinSeverity::Critical, IssueSeverity::Critical)
                    | (MinSeverity::Warning, IssueSeverity::Critical | IssueSeverity::Warning)
                    | (MinSeverity::Info, _)
            )
        })
        .collect();

    // Display summary
    display_version_consistency_summary(version_consistency, &filtered_issues, output)?;

    // Display inconsistencies (always shown if any exist)
    if !version_consistency.inconsistencies.is_empty() {
        output.info("")?;
        display_inconsistencies(version_consistency, verbosity, output)?;
    }

    // Display recommendations
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_version_consistency_recommendations(version_consistency, output)?;
    }

    Ok(())
}

/// Displays the version consistency audit summary.
///
/// # Arguments
///
/// * `version_consistency` - The version consistency audit section results
/// * `filtered_issues` - The filtered issues to count
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_version_consistency_summary(
    version_consistency: &sublime_pkg_tools::audit::VersionConsistencyAuditSection,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    output: &Output,
) -> Result<()> {
    output.info("━━━ Summary ━━━")?;

    // Overall health message
    if version_consistency.inconsistencies.is_empty() {
        output.success("✓ All internal dependencies have consistent versions!")?;
    } else {
        output.warning(&format!(
            "⚠️  {} version inconsistencies detected",
            version_consistency.inconsistencies.len()
        ))?;
    }

    output.info("")?;

    // Issue counts
    let critical_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
    let warning_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();
    let info_count =
        filtered_issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).count();

    output.info(&format!("Total Issues: {}", filtered_issues.len()))?;
    if critical_count > 0 {
        output.error(&format!("  Critical: {critical_count}"))?;
    }
    if warning_count > 0 {
        output.warning(&format!("  Warnings: {warning_count}"))?;
    }
    if info_count > 0 {
        output.info(&format!("  Info: {info_count}"))?;
    }

    // Inconsistency stats
    if !version_consistency.inconsistencies.is_empty() {
        output.info("")?;
        output.info("Inconsistency Details:")?;
        output.info(&format!(
            "  Internal packages affected: {}",
            version_consistency.inconsistencies.len()
        ))?;

        // Count total unique version specifications
        let total_unique_versions: usize =
            version_consistency.inconsistencies.iter().map(|inc| inc.unique_versions().len()).sum();

        output.info(&format!("  Total unique version specifications: {total_unique_versions}"))?;
    }

    Ok(())
}

/// Displays version inconsistencies section.
///
/// # Arguments
///
/// * `version_consistency` - The version consistency audit section results
/// * `verbosity` - Level of detail to show
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_inconsistencies(
    version_consistency: &sublime_pkg_tools::audit::VersionConsistencyAuditSection,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    output.info("━━━ Version Inconsistencies ━━━")?;
    output.info("")?;

    for inconsistency in &version_consistency.inconsistencies {
        output.warning(&format!("⚠️  {}", inconsistency.package_name))?;
        output
            .info(&format!("   Used with {} different versions:", inconsistency.version_count()))?;

        // Show all version usages in detailed mode, or just unique versions in normal mode
        if matches!(verbosity, Verbosity::Detailed) {
            for usage in &inconsistency.versions_used {
                output.info(&format!(
                    "   - {} uses version {}",
                    usage.package_name, usage.version_spec
                ))?;
            }
        } else {
            let unique_versions = inconsistency.unique_versions();
            for version in &unique_versions {
                // Count how many packages use this version
                let count = inconsistency
                    .versions_used
                    .iter()
                    .filter(|u| &u.version_spec == version)
                    .count();
                output.info(&format!(
                    "   - {} ({} package{})",
                    version,
                    count,
                    if count == 1 { "" } else { "s" }
                ))?;
            }
        }

        // Show recommended version
        output.info(&format!("   → Recommended: {}", inconsistency.recommended_version))?;
        output.info("")?;
    }

    output.info("💡 Version inconsistencies can lead to:")?;
    output.info("   - Confusion about which version is actually used at runtime")?;
    output.info("   - Potential runtime incompatibilities")?;
    output.info("   - Difficulty maintaining and updating dependencies")?;

    Ok(())
}

/// Displays actionable version consistency recommendations.
///
/// # Arguments
///
/// * `version_consistency` - The version consistency audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_version_consistency_recommendations(
    version_consistency: &sublime_pkg_tools::audit::VersionConsistencyAuditSection,
    output: &Output,
) -> Result<()> {
    let mut recommendations = Vec::new();

    // If inconsistencies exist
    if !version_consistency.inconsistencies.is_empty() {
        recommendations.push("📊 Align internal dependency versions for consistency");
        recommendations.push("   - Update package.json files to use the recommended versions");
        recommendations.push("   - Consider using the workspace: protocol (e.g., workspace:*)");
        recommendations
            .push("   - Run: npm install (or your package manager) to update lock files");

        // Check if any recommended version is workspace protocol
        let has_workspace_protocol = version_consistency
            .inconsistencies
            .iter()
            .any(|inc| inc.recommended_version.starts_with("workspace:"));

        if has_workspace_protocol {
            recommendations.push("");
            recommendations.push("✨ Workspace protocol benefits:");
            recommendations.push("   - Always uses the local version from the monorepo");
            recommendations.push("   - No version mismatches possible");
            recommendations.push("   - Simplifies version management in monorepos");
        } else {
            recommendations.push("");
            recommendations.push("💡 Consider migrating to workspace: protocol:");
            recommendations.push("   - Change version specs to 'workspace:*' in package.json");
            recommendations.push("   - This ensures all packages use the local monorepo versions");
            recommendations.push("   - Eliminates version inconsistency issues entirely");
        }
    }

    // Display recommendations if any
    if !recommendations.is_empty() {
        output.info("")?;
        output.info("━━━ Recommendations ━━━")?;
        output.info("")?;

        let mut rec_num = 1;
        for recommendation in &recommendations {
            if recommendation.is_empty() {
                output.info("")?;
            } else if recommendation.starts_with("   ") {
                output.info(recommendation)?;
            } else {
                output.info(&format!("{rec_num}. {recommendation}"))?;
                rec_num += 1;
            }
        }

        output.info("")?;
        output.info("💡 Tip: Use --verbosity detailed for more specific package information")?;
    }

    Ok(())
}

/// Writes the version consistency report to a file.
///
/// # Arguments
///
/// * `_version_consistency` - The version consistency audit section results
/// * `_file_path` - Path to write the report file
///
/// # Returns
///
/// Returns `Ok(())` if the file was written successfully.
///
/// # Errors
///
/// Returns an error if file I/O operations fail or JSON serialization fails.
///
/// # Note
///
/// Currently a placeholder. Full JSON output will be implemented based on
/// the OutputFormat from the global CLI args.
#[allow(clippy::todo)]
fn write_version_consistency_report_to_file(
    _version_consistency: &sublime_pkg_tools::audit::VersionConsistencyAuditSection,
    _file_path: &Path,
) -> Result<()> {
    // TODO: will be implemented in story 7.6 (JSON output format)
    todo!("File output will be implemented in story 7.6")
}

/// Loads audit configuration from workspace.
///
/// # Arguments
///
/// * `config_path` - Optional path to configuration file
///
/// # Returns
///
/// Returns the loaded `PackageToolsConfig`.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be found
/// - Configuration file cannot be parsed
/// - Configuration is invalid
async fn load_audit_config(
    config_path: Option<&Path>,
) -> Result<sublime_pkg_tools::config::PackageToolsConfig> {
    use sublime_pkg_tools::config::ConfigLoader;

    let config = if let Some(path) = config_path {
        ConfigLoader::load_from_file(path).await.map_err(|e| {
            CliError::configuration(format!("Failed to load config from {}: {e}", path.display()))
        })?
    } else {
        ConfigLoader::load_defaults()
            .await
            .map_err(|e| CliError::configuration(format!("Failed to load default config: {e}")))?
    };

    Ok(config)
}
