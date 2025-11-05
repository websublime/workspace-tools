//! Audit report formatting and output.
//!
//! This module handles formatting and displaying audit results.
//!
//! # What
//!
//! Provides:
//! - Report formatting for different output modes
//! - Severity filtering
//! - Health score display
//! - Issue categorization and display
//! - File output handling
//!
//! # How
//!
//! The report formatter:
//! 1. Filters issues by minimum severity
//! 2. Groups issues by section
//! 3. Formats based on output mode (Human, Json, etc.)
//! 4. Displays or writes to file
//!
//! # Why
//!
//! Centralizing report formatting:
//! - Ensures consistent output across all audit types
//! - Supports multiple output formats
//! - Provides clear, actionable information
//! - Enables programmatic consumption via JSON

use crate::commands::audit::comprehensive::AuditResults;
use crate::commands::audit::types::MinSeverity;
use crate::error::Result;
use crate::output::Output;
use std::path::Path;
use sublime_pkg_tools::audit::{IssueSeverity, Verbosity};

/// Formats and displays an audit report.
///
/// This function takes audit results, applies filtering, formats the output,
/// and either displays it or writes it to a file.
///
/// # Arguments
///
/// * `results` - The aggregated audit results
/// * `health_score` - Optional overall health score (0-100)
/// * `min_severity` - Minimum severity level to display
/// * `verbosity` - Level of detail in the report
/// * `output` - Output context for formatting
/// * `output_file` - Optional path to write report to file
///
/// # Returns
///
/// Returns `Ok(())` if the report was successfully formatted and displayed.
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail
/// - JSON serialization fails
/// - Output formatting fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::report::format_audit_report;
///
/// let results = AuditResults { /* ... */ };
/// let health_score = Some(85);
/// let min_severity = MinSeverity::Info;
/// let verbosity = Verbosity::Normal;
///
/// format_audit_report(
///     &results,
///     health_score,
///     min_severity,
///     verbosity,
///     &output,
///     None,
/// ).await?;
/// ```
pub async fn format_audit_report(
    results: &AuditResults,
    health_score: Option<u8>,
    min_severity: MinSeverity,
    verbosity: Verbosity,
    output: &Output,
    output_file: Option<&Path>,
) -> Result<()> {
    // Filter issues by severity
    let filtered_issues = filter_issues_by_severity(results, min_severity);

    // Display header
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("                  AUDIT REPORT                    ")?;
    output.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.info("")?; // Empty line for spacing

    // Display health score if available
    if let Some(score) = health_score {
        display_health_score(score, output)?;
    }

    // Display summary
    display_summary(results, &filtered_issues, output)?;

    // Display detailed results by section
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_section_results(results, &filtered_issues, verbosity, output)?;
    }

    // Display actionable recommendations
    if matches!(verbosity, Verbosity::Normal | Verbosity::Detailed) {
        display_recommendations(results, health_score, output)?;
    }

    // Write to file if requested
    if let Some(file_path) = output_file {
        write_report_to_file(results, health_score, file_path)?;
        output.success(&format!("Report written to {}", file_path.display()))?;
    }

    Ok(())
}

/// Filters audit issues by minimum severity level.
///
/// # Arguments
///
/// * `results` - The audit results containing all issues
/// * `min_severity` - The minimum severity level to include
///
/// # Returns
///
/// A vector of filtered issues matching or exceeding the minimum severity.
pub(crate) fn filter_issues_by_severity(
    results: &AuditResults,
    min_severity: MinSeverity,
) -> Vec<&sublime_pkg_tools::audit::AuditIssue> {
    results
        .all_issues()
        .into_iter()
        .filter(|issue| {
            matches!(
                (min_severity, &issue.severity),
                (MinSeverity::Critical, IssueSeverity::Critical)
                    | (MinSeverity::Warning, IssueSeverity::Critical | IssueSeverity::Warning)
                    | (MinSeverity::Info, _)
            )
        })
        .collect()
}

/// Displays the health score with color coding.
///
/// # Arguments
///
/// * `score` - The health score (0-100)
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
pub(crate) fn display_health_score(score: u8, output: &Output) -> Result<()> {
    let message = format!("Health Score: {score}/100");

    if score >= 80 {
        output.success(&message)?;
    } else if score >= 60 {
        output.warning(&message)?;
    } else {
        output.error(&message)?;
    }

    output.info("")?; // Empty line for spacing
    Ok(())
}

/// Displays a summary of audit results.
///
/// # Arguments
///
/// * `results` - The audit results
/// * `filtered_issues` - The filtered issues to display
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_summary(
    results: &AuditResults,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    output: &Output,
) -> Result<()> {
    output.info("━━━ Summary ━━━")?;

    // Count issues by severity
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

    // Display section-specific summaries
    if let Some(ref upgrades) = results.upgrades {
        output.info("")?;
        output.info(&format!("Upgrades Available: {}", upgrades.total_upgrades))?;
        if upgrades.major_upgrades > 0 {
            output.warning(&format!("  Major: {}", upgrades.major_upgrades))?;
        }
        if upgrades.minor_upgrades > 0 {
            output.info(&format!("  Minor: {}", upgrades.minor_upgrades))?;
        }
        if upgrades.patch_upgrades > 0 {
            output.info(&format!("  Patch: {}", upgrades.patch_upgrades))?;
        }
        if !upgrades.deprecated_packages.is_empty() {
            output.error(&format!("  Deprecated: {}", upgrades.deprecated_packages.len()))?;
        }
    }

    if let Some(ref dependencies) = results.dependencies
        && (!dependencies.circular_dependencies.is_empty()
            || !dependencies.version_conflicts.is_empty())
    {
        output.info("")?;
        if !dependencies.circular_dependencies.is_empty() {
            output.error(&format!(
                "Circular Dependencies: {}",
                dependencies.circular_dependencies.len()
            ))?;
        }
        if !dependencies.version_conflicts.is_empty() {
            output
                .warning(&format!("Version Conflicts: {}", dependencies.version_conflicts.len()))?;
        }
    }

    if let Some(ref version_consistency) = results.version_consistency
        && !version_consistency.inconsistencies.is_empty()
    {
        output.info("")?;
        output.warning(&format!(
            "Version Inconsistencies: {}",
            version_consistency.inconsistencies.len()
        ))?;
    }

    if let Some(ref breaking_changes) = results.breaking_changes
        && breaking_changes.total_breaking_changes > 0
    {
        output.info("")?;
        output.warning(&format!(
            "Breaking Changes: {} detected across {} package{}",
            breaking_changes.total_breaking_changes,
            breaking_changes.packages_with_breaking.len(),
            if breaking_changes.packages_with_breaking.len() == 1 { "" } else { "s" }
        ))?;
    }

    output.info("")?; // Empty line for spacing
    Ok(())
}

/// Displays actionable recommendations based on audit results.
///
/// # Arguments
///
/// * `results` - The audit results
/// * `health_score` - The overall health score
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
pub(crate) fn display_recommendations(
    results: &AuditResults,
    health_score: Option<u8>,
    output: &Output,
) -> Result<()> {
    let mut recommendations = Vec::new();

    // Health score based recommendations
    if let Some(score) = health_score {
        if score < 60 {
            recommendations
                .push("🚨 Project health is critical - address high-severity issues immediately");
        } else if score < 80 {
            recommendations.push("⚠️  Project health needs attention - review and fix warnings");
        }
    }

    // Upgrade recommendations
    if let Some(ref upgrades) = results.upgrades {
        if !upgrades.deprecated_packages.is_empty() {
            recommendations
                .push("📦 Replace deprecated packages to avoid future compatibility issues");
            recommendations.push("   Run: wnt upgrade check --show-deprecated");
        }
        if upgrades.major_upgrades > 0 {
            recommendations.push("🔄 Review major version upgrades for breaking changes");
            recommendations.push("   Run: wnt upgrade check --filter major");
        }
    }

    // Dependency recommendations - circular dependencies
    if let Some(ref dependencies) = results.dependencies
        && !dependencies.circular_dependencies.is_empty()
    {
        recommendations.push("🔁 Resolve circular dependencies to improve maintainability");
        recommendations.push("   Consider extracting shared code into a separate package");
    }

    // Dependency recommendations - version conflicts
    if let Some(ref dependencies) = results.dependencies
        && !dependencies.version_conflicts.is_empty()
    {
        recommendations.push("⚡ Fix version conflicts to ensure consistent dependency resolution");
    }

    // Version consistency recommendations
    if let Some(ref version_consistency) = results.version_consistency
        && !version_consistency.inconsistencies.is_empty()
    {
        recommendations.push("📊 Align internal dependency versions across packages");
        recommendations.push("   Run: wnt upgrade apply --internal-only");
    }

    // Breaking changes recommendations
    if let Some(ref breaking_changes) = results.breaking_changes
        && breaking_changes.total_breaking_changes > 0
    {
        recommendations.push("⚠️  Review breaking changes before release");
        recommendations.push("   Run: wnt audit breaking-changes --verbosity detailed");
        recommendations.push("   Ensure major version bump for packages with breaking changes");
    }

    // Display recommendations if any
    if !recommendations.is_empty() {
        output.info("")?;
        output.info("━━━ Recommendations ━━━")?;
        output.info("")?;

        for (index, recommendation) in recommendations.iter().enumerate() {
            if recommendation.starts_with("   ") {
                // Indented command - show as code
                output.info(recommendation)?;
            } else {
                // Main recommendation - show with number
                output.info(&format!("{}. {}", index / 2 + 1, recommendation))?;
            }
        }

        output.info("")?;
        output.info("💡 Tip: Use --verbosity detailed for more specific guidance")?;
    }

    Ok(())
}

/// Displays detailed section results.
///
/// # Arguments
///
/// * `results` - The audit results
/// * `filtered_issues` - The filtered issues to display
/// * `verbosity` - Level of detail to show
/// * `output` - The output context
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_section_results(
    results: &AuditResults,
    filtered_issues: &[&sublime_pkg_tools::audit::AuditIssue],
    verbosity: Verbosity,
    output: &Output,
) -> Result<()> {
    if filtered_issues.is_empty() {
        output.success("No issues found!")?;
        return Ok(());
    }

    output.info("━━━ Issues ━━━")?;

    for issue in filtered_issues {
        let severity_label = match issue.severity {
            IssueSeverity::Critical => "CRITICAL",
            IssueSeverity::Warning => "WARNING",
            IssueSeverity::Info => "INFO",
        };

        let message = format!("[{}] {}", severity_label, issue.title);

        match issue.severity {
            IssueSeverity::Critical => output.error(&message)?,
            IssueSeverity::Warning => output.warning(&message)?,
            IssueSeverity::Info => output.info(&message)?,
        }

        if matches!(verbosity, Verbosity::Detailed) {
            if !issue.description.is_empty() {
                output.info(&format!("  {}", issue.description))?;
            }
            if let Some(ref suggestion) = issue.suggestion {
                output.info(&format!("  → {suggestion}"))?;
            }
        }
    }

    // Show count of sections audited
    let mut sections_run = Vec::new();
    if results.upgrades.is_some() {
        sections_run.push("Upgrades");
    }
    if results.dependencies.is_some() {
        sections_run.push("Dependencies");
    }
    if results.version_consistency.is_some() {
        sections_run.push("Version Consistency");
    }
    if results.breaking_changes.is_some() {
        sections_run.push("Breaking Changes");
    }

    output.info("")?;
    output.info(&format!("Sections audited: {}", sections_run.join(", ")))?;

    Ok(())
}

/// Writes the audit report to a file in the specified format.
///
/// The format is determined automatically from the file extension:
/// - `.html`, `.htm` → HTML format with embedded styling
/// - `.md`, `.markdown` → Markdown format
/// - Other extensions → HTML format (default)
///
/// # Arguments
///
/// * `results` - The audit results to export
/// * `health_score` - The overall health score
/// * `file_path` - Path to write the report file
///
/// # Returns
///
/// Returns `Ok(())` if the file was written successfully.
///
/// # Errors
///
/// Returns an error if:
/// - File I/O operations fail
/// - Serialization fails
/// - The parent directory doesn't exist
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::audit::report::write_report_to_file;
/// use std::path::Path;
///
/// let results = AuditResults { /* ... */ };
/// write_report_to_file(&results, Some(85), Path::new("report.html"))?;
/// ```
pub(crate) fn write_report_to_file(
    results: &AuditResults,
    health_score: Option<u8>,
    file_path: &Path,
) -> Result<()> {
    use crate::commands::audit::comprehensive::create_exportable_data;
    use crate::output::export::{ExportFormat, export_data};

    // Create exportable data structure
    let data = create_exportable_data(results, health_score);

    // Determine format from file extension
    let format = match file_path.extension().and_then(|s| s.to_str()) {
        Some("md" | "markdown") => ExportFormat::Markdown,
        _ => ExportFormat::Html, // default to HTML for .html, .htm, and unknown extensions
    };

    // Export the data to the file
    export_data(&data, format, file_path)?;

    Ok(())
}
