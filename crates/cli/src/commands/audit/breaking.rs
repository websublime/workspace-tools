//! Breaking changes audit command implementation.
//!
//! This module provides focused breaking changes audit functionality.
//!
//! # What
//!
//! Provides:
//! - `execute_breaking_changes_audit` function - Main entry point for breaking changes audits
//! - Detection of breaking changes from multiple sources
//! - Categorization by source (commits, changelogs, semver major bumps)
//! - Impact analysis across affected packages
//! - Actionable warnings and recommendations
//!
//! # How
//!
//! The execution flow:
//! 1. Initialize audit manager and configuration
//! 2. Execute breaking changes detection via AuditManager
//! 3. Analyze conventional commits for breaking change markers
//! 4. Parse changelogs for breaking change sections
//! 5. Detect major version bumps in dependencies
//! 6. Generate detailed report with impact analysis
//! 7. Display results via Output system or write to file
//!
//! # Why
//!
//! A dedicated breaking changes audit provides:
//! - Early detection of potential breaking changes before releases
//! - Clear visibility into sources of breaking changes
//! - Impact assessment for release planning
//! - Proactive warning system for downstream consumers
//! - Comprehensive tracking across multiple detection methods

use crate::commands::audit::types::{MinSeverity, parse_verbosity};
use crate::error::{CliError, Result};
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::{AuditManager, IssueSeverity, Verbosity};

/// Executes a focused breaking changes audit.
///
/// This function provides detailed breaking changes analysis and reporting,
/// focusing specifically on detecting potential breaking changes from commits,
/// changelogs, and semver analysis.
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
/// Returns `Ok(())` if the breaking changes audit completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be loaded or is invalid
/// - Workspace root is invalid
/// - Audit manager initialization fails
/// - Breaking changes detection fails
/// - Report generation fails
/// - File I/O operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::breaking::execute_breaking_changes_audit;
/// use sublime_cli_tools::commands::audit::types::MinSeverity;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_breaking_changes_audit(
///     &output,
///     workspace_root,
///     None,
///     MinSeverity::Info,
///     "normal",
///     None,
/// ).await?;
/// ```
pub async fn execute_breaking_changes_audit(
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
    output.info("Initializing breaking changes audit...")?;
    let audit_manager = AuditManager::new(workspace_root.to_path_buf(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to initialize audit manager: {e}")))?;

    // Execute breaking changes audit
    output.info("Detecting potential breaking changes...")?;
    let breaking_changes = audit_manager
        .audit_breaking_changes()
        .await
        .map_err(|e| CliError::execution(format!("Breaking changes audit failed: {e}")))?;

    // Display results
    format_breaking_changes_report(&breaking_changes, min_severity, verbosity, output)?;

    // Write to file if requested
    if let Some(file_path) = output_file {
        write_breaking_changes_report_to_file(&breaking_changes, file_path)?;
        output.success(&format!("Report written to {}", file_path.display()))?;
    }

    Ok(())
}

/// Formats and displays a breaking changes audit report.
///
/// This function formats breaking changes audit results with detailed analysis of
/// detected breaking changes, their sources, and impact on affected packages.
///
/// # Arguments
///
/// * `breaking_changes` - The breaking changes audit section results
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
fn format_breaking_changes_report(
    breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    // Display header
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("          BREAKING CHANGES AUDIT REPORT            ")?;
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("")?;

    // Filter issues by severity
    let filtered_issues: Vec<_> = breaking_changes
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
    display_breaking_changes_summary(breaking_changes, &filtered_issues, output)?;

    // Display breaking changes by package (always shown if any exist)
    if !breaking_changes.packages_with_breaking.is_empty() {
        output.info("")?;
        display_packages_with_breaking_changes(breaking_changes, verbosity, output)?;
    }

    // Display breaking changes by source
    if matches!(verbosity, Verbosity::Detailed) && breaking_changes.total_breaking_changes > 0 {
        display_breaking_changes_by_source(breaking_changes, output)?;
    }

    // Display recommendations
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_breaking_changes_recommendations(breaking_changes, output)?;
    }

    Ok(())
}

/// Displays the breaking changes audit summary.
///
/// # Arguments
///
/// * `breaking_changes` - The breaking changes audit section results
/// * `filtered_issues` - The filtered issues to count
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_breaking_changes_summary(
    breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    output: &Output,
) -> Result<()> {
    output.info("━━━ Summary ━━━")?;

    // Overall health message
    if breaking_changes.total_breaking_changes == 0 {
        output.success("✓ No breaking changes detected!")?;
    } else {
        output.warning(&format!(
            "⚠️  {} breaking change{} detected across {} package{}",
            breaking_changes.total_breaking_changes,
            if breaking_changes.total_breaking_changes == 1 { "" } else { "s" },
            breaking_changes.packages_with_breaking.len(),
            if breaking_changes.packages_with_breaking.len() == 1 { "" } else { "s" }
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

    // Breaking changes by source
    if breaking_changes.total_breaking_changes > 0 {
        output.info("")?;
        output.info("Breaking Changes by Source:")?;

        // Count by source type
        let mut from_commits = 0;
        let mut from_changelog = 0;
        let mut from_changeset = 0;

        for pkg in &breaking_changes.packages_with_breaking {
            for change in &pkg.breaking_changes {
                match change.source {
                    sublime_pkg_tools::audit::BreakingChangeSource::ConventionalCommit => {
                        from_commits += 1;
                    }
                    sublime_pkg_tools::audit::BreakingChangeSource::Changelog => {
                        from_changelog += 1;
                    }
                    sublime_pkg_tools::audit::BreakingChangeSource::Changeset => {
                        from_changeset += 1;
                    }
                }
            }
        }

        if from_commits > 0 {
            output.info(&format!("  Conventional commits: {from_commits}"))?;
        }
        if from_changelog > 0 {
            output.info(&format!("  Changelog entries: {from_changelog}"))?;
        }
        if from_changeset > 0 {
            output.info(&format!("  Changesets: {from_changeset}"))?;
        }
    }

    Ok(())
}

/// Displays packages with breaking changes section.
///
/// # Arguments
///
/// * `breaking_changes` - The breaking changes audit section results
/// * `verbosity` - Level of detail to show
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_packages_with_breaking_changes(
    breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    output.info("━━━ Packages with Breaking Changes ━━━")?;
    output.info("")?;

    for pkg in &breaking_changes.packages_with_breaking {
        output.warning(&format!("⚠️  {}", pkg.package_name))?;
        output.info(&format!(
            "   {} breaking change{}",
            pkg.breaking_changes.len(),
            if pkg.breaking_changes.len() == 1 { "" } else { "s" }
        ))?;

        // Display version info if available
        if let (Some(current), Some(next)) = (&pkg.current_version, &pkg.next_version) {
            output.info(&format!("   Version: {current} → {next}"))?;
        }

        // Display changes based on verbosity
        if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
            output.info("")?;
            for (idx, change) in pkg.breaking_changes.iter().enumerate() {
                let change_num = idx + 1;
                output.info(&format!("   {change_num}. {}", change.description))?;

                // Show commit hash and source in detailed mode
                if matches!(verbosity, Verbosity::Detailed) {
                    if let Some(ref commit_hash) = change.commit_hash {
                        output.info(&format!(
                            "      Commit: {}",
                            &commit_hash[..7.min(commit_hash.len())]
                        ))?;
                    }

                    let source_str = match change.source {
                        sublime_pkg_tools::audit::BreakingChangeSource::ConventionalCommit => {
                            "Conventional Commit"
                        }
                        sublime_pkg_tools::audit::BreakingChangeSource::Changelog => "Changelog",
                        sublime_pkg_tools::audit::BreakingChangeSource::Changeset => "Changeset",
                    };
                    output.info(&format!("      Source: {source_str}"))?;
                }
                output.info("")?;
            }
        } else {
            output.info("")?;
        }
    }

    output.info("💡 Breaking changes may require:")?;
    output.info("   - Updating consumer code and documentation")?;
    output.info("   - Releasing a new major version")?;
    output.info("   - Providing migration guides for users")?;
    output.info("   - Clear communication in release notes")?;

    Ok(())
}

/// Displays breaking changes categorized by source type.
///
/// # Arguments
///
/// * `breaking_changes` - The breaking changes audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_breaking_changes_by_source(
    breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
    output: &Output,
) -> Result<()> {
    output.info("")?;
    output.info("━━━ Breaking Changes by Source ━━━")?;
    output.info("")?;

    // Collect changes by source type
    let mut from_commits: Vec<(&str, &sublime_pkg_tools::audit::BreakingChange)> = Vec::new();
    let mut from_changelog: Vec<(&str, &sublime_pkg_tools::audit::BreakingChange)> = Vec::new();
    let mut from_changeset: Vec<(&str, &sublime_pkg_tools::audit::BreakingChange)> = Vec::new();

    for pkg in &breaking_changes.packages_with_breaking {
        for change in &pkg.breaking_changes {
            match change.source {
                sublime_pkg_tools::audit::BreakingChangeSource::ConventionalCommit => {
                    from_commits.push((&pkg.package_name, change));
                }
                sublime_pkg_tools::audit::BreakingChangeSource::Changelog => {
                    from_changelog.push((&pkg.package_name, change));
                }
                sublime_pkg_tools::audit::BreakingChangeSource::Changeset => {
                    from_changeset.push((&pkg.package_name, change));
                }
            }
        }
    }

    // Display conventional commits
    if !from_commits.is_empty() {
        output.info(&format!("Conventional Commits ({}):", from_commits.len()))?;
        for (pkg_name, change) in &from_commits {
            let commit_info = if let Some(ref hash) = change.commit_hash {
                format!(" [{}]", &hash[..7.min(hash.len())])
            } else {
                String::new()
            };
            output.info(&format!("  - {} ({}{})", change.description, pkg_name, commit_info))?;
        }
        output.info("")?;
    }

    // Display changelog entries
    if !from_changelog.is_empty() {
        output.info(&format!("Changelog Entries ({}):", from_changelog.len()))?;
        for (pkg_name, change) in &from_changelog {
            output.info(&format!("  - {} ({})", change.description, pkg_name))?;
        }
        output.info("")?;
    }

    // Display changeset entries
    if !from_changeset.is_empty() {
        output.info(&format!("Changesets ({}):", from_changeset.len()))?;
        for (pkg_name, change) in &from_changeset {
            output.info(&format!("  - {} ({})", change.description, pkg_name))?;
        }
        output.info("")?;
    }

    Ok(())
}

/// Displays actionable breaking changes recommendations.
///
/// # Arguments
///
/// * `breaking_changes` - The breaking changes audit section results
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_breaking_changes_recommendations(
    breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
    output: &Output,
) -> Result<()> {
    let mut recommendations = Vec::new();

    // If breaking changes exist
    if breaking_changes.total_breaking_changes > 0 {
        recommendations.push("📋 Review all breaking changes before release");
        recommendations.push("   - Verify that each breaking change is intentional");
        recommendations.push("   - Document breaking changes in CHANGELOG.md");
        recommendations.push("   - Ensure commit messages follow conventional commit format");

        recommendations.push("");
        recommendations.push("📚 Prepare migration documentation");
        recommendations.push("   - Create migration guides for users");
        recommendations.push("   - Provide code examples for common migration patterns");
        recommendations.push("   - Document deprecation paths where applicable");

        recommendations.push("");
        recommendations.push("🔢 Plan version bump strategy");
        recommendations.push("   - Breaking changes require a major version bump (x.0.0)");
        recommendations.push("   - Run: wnt bump --preview to see version impact");
        recommendations.push("   - Consider coordinating release across affected packages");

        recommendations.push("");
        recommendations.push("📣 Communicate with stakeholders");
        recommendations.push("   - Notify teams and users about upcoming breaking changes");
        recommendations.push("   - Consider a pre-release version (alpha/beta) for testing");
        recommendations.push("   - Update documentation before release");
    } else {
        recommendations.push("✅ No breaking changes detected");
        recommendations.push("   - Safe to release without major version bump");
        recommendations.push("   - Consider minor or patch version based on changes");
        recommendations.push("   - Run: wnt bump --preview to see recommended versions");
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
        output.info("💡 Tip: Use --verbosity detailed for source-specific information")?;
    }

    Ok(())
}

/// Writes the breaking changes report to a file.
///
/// # Arguments
///
/// * `_breaking_changes` - The breaking changes audit section results
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
fn write_breaking_changes_report_to_file(
    _breaking_changes: &sublime_pkg_tools::audit::BreakingChangesAuditSection,
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
