//! Upgrade audit command implementation.
//!
//! This module provides focused upgrade audit functionality.
//!
//! # What
//!
//! Provides:
//! - `execute_upgrade_audit` function - Main entry point for upgrade-specific audits
//! - Detailed upgrade categorization and reporting
//! - Deprecated package detection and highlighting
//! - Upgrade recommendations by severity
//!
//! # How
//!
//! The execution flow:
//! 1. Initialize audit manager and configuration
//! 2. Execute upgrade detection via AuditManager
//! 3. Categorize upgrades by type (major, minor, patch)
//! 4. Identify and highlight deprecated packages
//! 5. Generate detailed report with recommendations
//! 6. Display results via Output system or write to file
//!
//! # Why
//!
//! A dedicated upgrade audit provides:
//! - Focused visibility into dependency freshness
//! - Early detection of deprecated dependencies
//! - Clear categorization for upgrade planning
//! - Actionable recommendations for update strategy

use crate::commands::audit::types::{MinSeverity, parse_verbosity};
use crate::error::{CliError, Result};
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::{AuditManager, IssueSeverity, Verbosity};

/// Executes a focused upgrade audit.
///
/// This function provides detailed upgrade analysis and reporting, focusing
/// specifically on available dependency updates and deprecated packages.
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
/// Returns `Ok(())` if the upgrade audit completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be loaded or is invalid
/// - Workspace root is invalid
/// - Audit manager initialization fails
/// - Upgrade detection fails
/// - Report generation fails
/// - File I/O operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::upgrades::execute_upgrade_audit;
/// use sublime_cli_tools::commands::audit::types::MinSeverity;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_upgrade_audit(
///     &output,
///     workspace_root,
///     None,
///     MinSeverity::Info,
///     "normal",
///     None,
/// ).await?;
/// ```
pub async fn execute_upgrade_audit(
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
    output.info("Initializing upgrade audit...")?;
    let audit_manager = AuditManager::new(workspace_root.to_path_buf(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to initialize audit manager: {e}")))?;

    // Execute upgrade audit
    output.info("Detecting available upgrades...")?;
    let upgrades = audit_manager
        .audit_upgrades()
        .await
        .map_err(|e| CliError::execution(format!("Upgrade audit failed: {e}")))?;

    // Display results
    format_upgrade_report(&upgrades, min_severity, verbosity, output)?;

    // Write to file if requested
    if let Some(file_path) = output_file {
        write_upgrade_report_to_file(&upgrades, file_path)?;
        output.success(&format!("Report written to {}", file_path.display()))?;
    }

    Ok(())
}

/// Formats and displays an upgrade audit report.
///
/// This function formats upgrade results with detailed categorization
/// and actionable recommendations.
///
/// # Arguments
///
/// * `upgrades` - The upgrade audit section results
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
fn format_upgrade_report(
    upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    // Display header
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("              UPGRADE AUDIT REPORT                 ")?;
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("")?;

    // Filter issues by severity
    let filtered_issues: Vec<_> = upgrades
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
    display_upgrade_summary(upgrades, &filtered_issues, output)?;

    // Display deprecated packages (always shown if any exist)
    if !upgrades.deprecated_packages.is_empty() {
        output.info("")?;
        display_deprecated_packages(upgrades, output)?;
    }

    // Display upgrades by type
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_upgrades_by_type(upgrades, min_severity, verbosity, output)?;
    }

    // Display recommendations
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_upgrade_recommendations(upgrades, output)?;
    }

    Ok(())
}

/// Displays the upgrade audit summary.
///
/// # Arguments
///
/// * `upgrades` - The upgrade audit section results
/// * `filtered_issues` - The filtered issues to count
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_upgrade_summary(
    upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    output: &Output,
) -> Result<()> {
    output.info("━━━ Summary ━━━")?;

    if upgrades.total_upgrades == 0 {
        output.success("✓ All dependencies are up to date!")?;
        return Ok(());
    }

    output.info(&format!("Total Upgrades Available: {}", upgrades.total_upgrades))?;

    if upgrades.major_upgrades > 0 {
        output.warning(&format!(
            "  Major: {} (may include breaking changes)",
            upgrades.major_upgrades
        ))?;
    }
    if upgrades.minor_upgrades > 0 {
        output.info(&format!(
            "  Minor: {} (new features, backward compatible)",
            upgrades.minor_upgrades
        ))?;
    }
    if upgrades.patch_upgrades > 0 {
        output.info(&format!("  Patch: {} (bug fixes)", upgrades.patch_upgrades))?;
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

    Ok(())
}

/// Displays deprecated packages section.
///
/// # Arguments
///
/// * `upgrades` - The upgrade audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_deprecated_packages(
    upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    output: &Output,
) -> Result<()> {
    output.info("━━━ Deprecated Packages (CRITICAL) ━━━")?;
    output.info("")?;

    for deprecated in &upgrades.deprecated_packages {
        output.error(&format!("⚠️  {} (v{})", deprecated.name, deprecated.current_version))?;
        output.info(&format!("   {}", deprecated.deprecation_message))?;

        if let Some(alternative) = &deprecated.alternative {
            output.info(&format!("   → Consider migrating to: {alternative}"))?;
        }
        output.info("")?;
    }

    Ok(())
}

/// Displays upgrades categorized by type.
///
/// # Arguments
///
/// * `upgrades` - The upgrade audit section results
/// * `min_severity` - Minimum severity level to display
/// * `verbosity` - Level of detail to show
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_upgrades_by_type(
    upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    // Collect all upgrades
    let mut all_upgrades: Vec<_> = upgrades
        .upgrades_by_package
        .iter()
        .flat_map(|(pkg_name, pkg_upgrades)| {
            pkg_upgrades.iter().map(move |upgrade| (pkg_name, upgrade))
        })
        .collect();

    // Sort by severity (major first, then minor, then patch)
    all_upgrades.sort_by_key(|(_, upgrade)| match upgrade.upgrade_type {
        sublime_pkg_tools::upgrade::UpgradeType::Major => 0,
        sublime_pkg_tools::upgrade::UpgradeType::Minor => 1,
        sublime_pkg_tools::upgrade::UpgradeType::Patch => 2,
    });

    // Display major upgrades
    if upgrades.major_upgrades > 0
        && matches!(min_severity, MinSeverity::Info | MinSeverity::Warning)
    {
        output.info("")?;
        output.info("━━━ Major Upgrades (Breaking Changes Possible) ━━━")?;
        output.info("")?;

        for (pkg_name, upgrade) in &all_upgrades {
            if matches!(upgrade.upgrade_type, sublime_pkg_tools::upgrade::UpgradeType::Major) {
                output.warning(&format!("📦 {} → {}", upgrade.name, upgrade.latest_version))?;
                output.info(&format!("   Current: v{}", upgrade.current_version))?;
                output.info(&format!("   Package: {pkg_name}"))?;

                if matches!(verbosity, Verbosity::Detailed)
                    && upgrade.version_info.deprecated.is_none()
                {
                    output.info("   → Review changelog for breaking changes before upgrading")?;
                }
                output.info("")?;
            }
        }
    }

    // Display minor upgrades
    if upgrades.minor_upgrades > 0
        && matches!(min_severity, MinSeverity::Info)
        && matches!(verbosity, Verbosity::Detailed)
    {
        output.info("━━━ Minor Upgrades (New Features) ━━━")?;
        output.info("")?;

        for (pkg_name, upgrade) in &all_upgrades {
            if matches!(upgrade.upgrade_type, sublime_pkg_tools::upgrade::UpgradeType::Minor) {
                output.info(&format!("📦 {} → {}", upgrade.name, upgrade.latest_version))?;
                output.info(&format!("   Current: v{}", upgrade.current_version))?;
                output.info(&format!("   Package: {pkg_name}"))?;
                output.info("")?;
            }
        }
    }

    // Display patch upgrades (only in detailed mode)
    if upgrades.patch_upgrades > 0
        && matches!(min_severity, MinSeverity::Info)
        && matches!(verbosity, Verbosity::Detailed)
    {
        output.info("━━━ Patch Upgrades (Bug Fixes) ━━━")?;
        output.info("")?;

        for (pkg_name, upgrade) in &all_upgrades {
            if matches!(upgrade.upgrade_type, sublime_pkg_tools::upgrade::UpgradeType::Patch) {
                output.info(&format!("📦 {} → {}", upgrade.name, upgrade.latest_version))?;
                output.info(&format!("   Current: v{}", upgrade.current_version))?;
                output.info(&format!("   Package: {pkg_name}"))?;
                output.info("")?;
            }
        }
    }

    Ok(())
}

/// Displays actionable upgrade recommendations.
///
/// # Arguments
///
/// * `upgrades` - The upgrade audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_upgrade_recommendations(
    upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    output: &Output,
) -> Result<()> {
    let mut recommendations = Vec::new();

    // Deprecated packages - highest priority
    if !upgrades.deprecated_packages.is_empty() {
        recommendations.push("🚨 Replace deprecated packages immediately");
        recommendations.push("   Run: wnt upgrade check --show-deprecated");
    }

    // Major upgrades - review needed
    if upgrades.major_upgrades > 0 {
        recommendations.push("📋 Review major upgrades for breaking changes");
        recommendations.push("   Run: wnt upgrade check --filter major");
        recommendations.push("   Review changelogs before applying");
    }

    // Minor/Patch upgrades - safe to apply
    if upgrades.minor_upgrades > 0 || upgrades.patch_upgrades > 0 {
        recommendations.push("✅ Minor and patch upgrades are generally safe");
        recommendations.push("   Run: wnt upgrade apply --filter minor,patch");
    }

    // Display recommendations if any
    if !recommendations.is_empty() {
        output.info("")?;
        output.info("━━━ Recommendations ━━━")?;
        output.info("")?;

        for (index, recommendation) in recommendations.iter().enumerate() {
            if recommendation.starts_with("   ") {
                // Indented command - show as-is
                output.info(recommendation)?;
            } else {
                // Main recommendation - show with number
                let rec_num = (index / 2) + 1;
                if rec_num <= recommendations.len() / 2 {
                    output.info(&format!("{rec_num}. {recommendation}"))?;
                }
            }
        }

        output.info("")?;
        output.info("💡 Tip: Always test upgrades in a development environment first")?;
    }

    Ok(())
}

/// Writes the upgrade report to a file.
///
/// # Arguments
///
/// * `_upgrades` - The upgrade audit section results
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
fn write_upgrade_report_to_file(
    _upgrades: &sublime_pkg_tools::audit::UpgradeAuditSection,
    _file_path: &Path,
) -> Result<()> {
    // TODO: will be implemented in story 8.3 (Export Formats)
    todo!("File output will be implemented in story 8.3")
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
