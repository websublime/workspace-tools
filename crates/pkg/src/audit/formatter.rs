//! Audit report formatting implementations.
//!
//! **What**: Provides formatters for converting audit reports into human-readable
//! and machine-readable formats such as Markdown and JSON.
//!
//! **How**: Implements formatting traits and functions that convert `AuditReport`
//! structures into formatted strings with proper styling, structure, and color
//! support where applicable.
//!
//! **Why**: To enable audit reports to be displayed in terminals, documentation,
//! CI/CD logs, or consumed programmatically by other tools.

use crate::audit::issue::{AuditIssue, IssueSeverity};
use crate::audit::report::{AuditReport, AuditSections};
use crate::audit::sections::BreakingChangeSource;
use serde_json;
use std::fmt::Write as FmtWrite;

/// Formatting options for audit reports.
///
/// Controls how reports are formatted, including verbosity levels and color support.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::FormatOptions;
///
/// let options = FormatOptions::default()
///     .with_colors(true)
///     .with_verbosity(Verbosity::Detailed);
/// ```
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Enable color output (ANSI codes)
    pub colors: bool,

    /// Verbosity level for output
    pub verbosity: Verbosity,

    /// Include suggested actions in output
    pub include_suggestions: bool,

    /// Include metadata in output
    pub include_metadata: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            colors: false,
            verbosity: Verbosity::Normal,
            include_suggestions: true,
            include_metadata: false,
        }
    }
}

impl FormatOptions {
    /// Creates a new `FormatOptions` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to use colors in output.
    #[must_use]
    pub fn with_colors(mut self, colors: bool) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the verbosity level.
    #[must_use]
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Sets whether to include suggestions.
    #[must_use]
    pub fn with_suggestions(mut self, include: bool) -> Self {
        self.include_suggestions = include;
        self
    }

    /// Sets whether to include metadata.
    #[must_use]
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

/// Verbosity level for report formatting.
///
/// Controls how much detail is included in formatted output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Minimal output (summary only)
    Minimal,

    /// Normal output (summary + issues)
    Normal,

    /// Detailed output (all information)
    Detailed,
}

/// Formats an audit report as Markdown.
///
/// This function generates a well-structured Markdown document from an audit report,
/// including all sections, issues, and statistics.
///
/// # Arguments
///
/// * `report` - The audit report to format
/// * `options` - Formatting options
///
/// # Returns
///
/// A `String` containing the formatted Markdown document.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::{format_markdown, FormatOptions};
///
/// let markdown = format_markdown(&report, &FormatOptions::default());
/// std::fs::write("AUDIT.md", markdown)?;
/// ```
#[must_use]
pub fn format_markdown(report: &AuditReport, options: &FormatOptions) -> String {
    let mut output = String::new();

    // Header
    let _ = writeln!(output, "# Audit Report");
    let _ = writeln!(output);
    let _ =
        writeln!(output, "**Generated**: {}", report.audited_at.format("%Y-%m-%d %H:%M:%S UTC"));
    let _ = writeln!(output, "**Workspace**: `{}`", report.workspace_root.display());
    let _ = writeln!(
        output,
        "**Project Type**: {}",
        if report.is_monorepo { "Monorepo" } else { "Single Package" }
    );
    let _ = writeln!(output, "**Health Score**: {}/100", report.health_score);
    let _ = writeln!(output);

    // Summary section
    format_summary_markdown(&mut output, report, options);

    // Only include detailed sections if not minimal
    if options.verbosity != Verbosity::Minimal {
        // Upgrades section
        if report.sections.upgrades.total_upgrades > 0
            || !report.sections.upgrades.issues.is_empty()
        {
            format_upgrades_section_markdown(&mut output, &report.sections, options);
        }

        // Dependencies section
        if !report.sections.dependencies.circular_dependencies.is_empty()
            || !report.sections.dependencies.version_conflicts.is_empty()
            || !report.sections.dependencies.issues.is_empty()
        {
            format_dependencies_section_markdown(&mut output, &report.sections, options);
        }

        // Breaking changes section
        if report.sections.breaking_changes.total_breaking_changes > 0
            || !report.sections.breaking_changes.issues.is_empty()
        {
            format_breaking_changes_section_markdown(&mut output, &report.sections, options);
        }

        // Categorization section
        if options.verbosity == Verbosity::Detailed {
            format_categorization_section_markdown(&mut output, &report.sections, options);
        }

        // Version consistency section
        if !report.sections.version_consistency.inconsistencies.is_empty()
            || !report.sections.version_consistency.issues.is_empty()
        {
            format_version_consistency_section_markdown(&mut output, &report.sections, options);
        }

        // Issues section
        if !report.all_issues().is_empty() {
            format_issues_section_markdown(&mut output, report, options);
        }
    }

    // Suggested actions
    if options.include_suggestions && !report.summary.suggested_actions.is_empty() {
        format_suggestions_markdown(&mut output, report);
    }

    output
}

/// Formats the summary section in Markdown.
fn format_summary_markdown(output: &mut String, report: &AuditReport, _options: &FormatOptions) {
    let _ = writeln!(output, "## Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| Metric | Value |");
    let _ = writeln!(output, "|--------|-------|");
    let _ = writeln!(output, "| Packages Analyzed | {} |", report.summary.packages_analyzed);
    let _ =
        writeln!(output, "| Dependencies Analyzed | {} |", report.summary.dependencies_analyzed);
    let _ = writeln!(output, "| Total Issues | {} |", report.summary.total_issues);
    let _ = writeln!(output, "| Critical Issues | {} |", report.summary.critical_issues);
    let _ = writeln!(output, "| Warnings | {} |", report.summary.warnings);
    let _ = writeln!(output, "| Info Items | {} |", report.summary.info_items);
    let _ = writeln!(
        output,
        "| Status | {} |",
        if report.passed() { "✅ Passed" } else { "❌ Failed" }
    );
    let _ = writeln!(output);
}

/// Formats the upgrades section in Markdown.
fn format_upgrades_section_markdown(
    output: &mut String,
    sections: &AuditSections,
    _options: &FormatOptions,
) {
    let _ = writeln!(output, "## Upgrades Available");
    let _ = writeln!(output);
    let _ = writeln!(output, "- **Total Upgrades**: {}", sections.upgrades.total_upgrades);
    let _ = writeln!(output, "- **Major**: {} ⚠️", sections.upgrades.major_upgrades);
    let _ = writeln!(output, "- **Minor**: {}", sections.upgrades.minor_upgrades);
    let _ = writeln!(output, "- **Patch**: {}", sections.upgrades.patch_upgrades);
    let _ = writeln!(output);

    if !sections.upgrades.deprecated_packages.is_empty() {
        let _ = writeln!(output, "### Deprecated Packages");
        let _ = writeln!(output);
        for pkg in &sections.upgrades.deprecated_packages {
            let _ = writeln!(output, "- **{}** (v{})", pkg.name, pkg.current_version);
            let _ = writeln!(output, "  - {}", pkg.deprecation_message);
            if let Some(alt) = &pkg.alternative {
                let _ = writeln!(output, "  - Alternative: `{}`", alt);
            }
        }
        let _ = writeln!(output);
    }
}

/// Formats the dependencies section in Markdown.
fn format_dependencies_section_markdown(
    output: &mut String,
    sections: &AuditSections,
    _options: &FormatOptions,
) {
    let _ = writeln!(output, "## Dependency Analysis");
    let _ = writeln!(output);

    if !sections.dependencies.circular_dependencies.is_empty() {
        let _ = writeln!(output, "### Circular Dependencies");
        let _ = writeln!(output);
        for circular in &sections.dependencies.circular_dependencies {
            let _ = writeln!(output, "- {}", circular.cycle.join(" → "));
        }
        let _ = writeln!(output);
    }

    if !sections.dependencies.version_conflicts.is_empty() {
        let _ = writeln!(output, "### Version Conflicts");
        let _ = writeln!(output);
        for conflict in &sections.dependencies.version_conflicts {
            let _ = writeln!(output, "#### {}", conflict.dependency_name);
            let _ = writeln!(output);
            for usage in &conflict.versions {
                let _ =
                    writeln!(output, "- `{}` uses `{}`", usage.package_name, usage.version_spec);
            }
            let _ = writeln!(output);
        }
    }
}

/// Formats the breaking changes section in Markdown.
fn format_breaking_changes_section_markdown(
    output: &mut String,
    sections: &AuditSections,
    _options: &FormatOptions,
) {
    let _ = writeln!(output, "## Breaking Changes");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "- **Packages with Breaking Changes**: {}",
        sections.breaking_changes.packages_with_breaking.len()
    );
    let _ = writeln!(
        output,
        "- **Total Breaking Changes**: {}",
        sections.breaking_changes.total_breaking_changes
    );
    let _ = writeln!(output);

    if !sections.breaking_changes.packages_with_breaking.is_empty() {
        for pkg in &sections.breaking_changes.packages_with_breaking {
            let _ = writeln!(output, "### {}", pkg.package_name);
            let _ = writeln!(output);
            if let Some(current) = &pkg.current_version {
                if let Some(next) = &pkg.next_version {
                    let _ = writeln!(output, "**Version**: {} → {}", current, next);
                    let _ = writeln!(output);
                }
            }

            for change in &pkg.breaking_changes {
                let source_label = match change.source {
                    BreakingChangeSource::ConventionalCommit => "Commit",
                    BreakingChangeSource::Changelog => "Changelog",
                    BreakingChangeSource::Changeset => "Changeset",
                };
                let _ = writeln!(output, "- [{}] {}", source_label, change.description);
                if let Some(hash) = &change.commit_hash {
                    let _ = writeln!(output, "  - Commit: `{}`", hash);
                }
            }
            let _ = writeln!(output);
        }
    }
}

/// Formats the categorization section in Markdown.
fn format_categorization_section_markdown(
    output: &mut String,
    sections: &AuditSections,
    _options: &FormatOptions,
) {
    let _ = writeln!(output, "## Dependency Categorization");
    let _ = writeln!(output);

    let stats = &sections.categorization.stats;
    let _ = writeln!(output, "| Category | Count |");
    let _ = writeln!(output, "|----------|-------|");
    let _ = writeln!(output, "| Total Packages | {} |", stats.total_packages);
    let _ = writeln!(output, "| Internal Packages | {} |", stats.internal_packages);
    let _ = writeln!(output, "| External Packages | {} |", stats.external_packages);
    let _ = writeln!(output, "| Workspace Links | {} |", stats.workspace_links);
    let _ = writeln!(output, "| Local Links | {} |", stats.local_links);
    let _ = writeln!(output);

    if !sections.categorization.internal_packages.is_empty() {
        let _ = writeln!(output, "### Internal Packages");
        let _ = writeln!(output);
        for pkg in &sections.categorization.internal_packages {
            let version = pkg.version.as_ref().map_or("unknown".to_string(), |v| v.to_string());
            let _ = writeln!(output, "- **{}** (v{})", pkg.name, version);
            if !pkg.used_by.is_empty() {
                let _ = writeln!(output, "  - Used by: {}", pkg.used_by.join(", "));
            }
        }
        let _ = writeln!(output);
    }
}

/// Formats the version consistency section in Markdown.
fn format_version_consistency_section_markdown(
    output: &mut String,
    sections: &AuditSections,
    _options: &FormatOptions,
) {
    let _ = writeln!(output, "## Version Consistency");
    let _ = writeln!(output);

    if sections.version_consistency.inconsistencies.is_empty() {
        let _ = writeln!(output, "✅ No version inconsistencies found.");
        let _ = writeln!(output);
    } else {
        let _ = writeln!(
            output,
            "⚠️ Found {} version inconsistenc(y/ies):",
            sections.version_consistency.inconsistencies.len()
        );
        let _ = writeln!(output);

        for inconsistency in &sections.version_consistency.inconsistencies {
            let _ = writeln!(output, "### {}", inconsistency.package_name);
            let _ = writeln!(output);
            let _ = writeln!(output, "**Versions in use**:");
            for usage in &inconsistency.versions_used {
                let _ =
                    writeln!(output, "- `{}` uses `{}`", usage.package_name, usage.version_spec);
            }
            let _ = writeln!(output);
            let _ = writeln!(output, "**Recommended**: `{}`", inconsistency.recommended_version);
            let _ = writeln!(output);
        }
    }
}

/// Formats the issues section in Markdown.
fn format_issues_section_markdown(
    output: &mut String,
    report: &AuditReport,
    options: &FormatOptions,
) {
    let _ = writeln!(output, "## Issues");
    let _ = writeln!(output);

    // Group issues by severity
    let critical = report.critical_issues();
    let warnings = report.warnings();
    let info = report.info_items();

    if !critical.is_empty() {
        let _ = writeln!(output, "### Critical Issues ({})", critical.len());
        let _ = writeln!(output);
        format_issue_list_markdown(output, &critical, options);
    }

    if !warnings.is_empty() {
        let _ = writeln!(output, "### Warnings ({})", warnings.len());
        let _ = writeln!(output);
        format_issue_list_markdown(output, &warnings, options);
    }

    if !info.is_empty() && options.verbosity == Verbosity::Detailed {
        let _ = writeln!(output, "### Informational ({})", info.len());
        let _ = writeln!(output);
        format_issue_list_markdown(output, &info, options);
    }
}

/// Formats a list of issues in Markdown.
fn format_issue_list_markdown(
    output: &mut String,
    issues: &[&AuditIssue],
    options: &FormatOptions,
) {
    for issue in issues {
        let icon = match issue.severity {
            IssueSeverity::Critical => "❌",
            IssueSeverity::Warning => "⚠️",
            IssueSeverity::Info => "ℹ️",
        };

        let _ = writeln!(output, "{} **{}**", icon, issue.title);
        let _ = writeln!(output);
        let _ = writeln!(output, "{}", issue.description);
        let _ = writeln!(output);

        if !issue.affected_packages.is_empty() {
            let _ =
                writeln!(output, "**Affected packages**: {}", issue.affected_packages.join(", "));
            let _ = writeln!(output);
        }

        if let Some(suggestion) = &issue.suggestion {
            let _ = writeln!(output, "**Suggestion**: {}", suggestion);
            let _ = writeln!(output);
        }

        if options.include_metadata && !issue.metadata.is_empty() {
            let _ = writeln!(output, "**Metadata**:");
            for (key, value) in &issue.metadata {
                let _ = writeln!(output, "- {}: {}", key, value);
            }
            let _ = writeln!(output);
        }
    }
}

/// Formats suggested actions in Markdown.
fn format_suggestions_markdown(output: &mut String, report: &AuditReport) {
    let _ = writeln!(output, "## Suggested Actions");
    let _ = writeln!(output);

    for (i, action) in report.summary.suggested_actions.iter().enumerate() {
        let _ = writeln!(output, "{}. {}", i + 1, action);
    }
    let _ = writeln!(output);
}

/// Formats an audit report as JSON.
///
/// This function serializes the audit report into a JSON string that can be
/// consumed by other tools or stored for later analysis.
///
/// # Arguments
///
/// * `report` - The audit report to format
///
/// # Returns
///
/// A `Result` containing the JSON string or a serialization error.
///
/// # Errors
///
/// Returns an error if the report cannot be serialized to JSON.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::format_json;
///
/// let json = format_json(&report)?;
/// std::fs::write("audit-report.json", json)?;
/// ```
pub fn format_json(report: &AuditReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Formats an audit report as compact JSON (no pretty printing).
///
/// # Arguments
///
/// * `report` - The audit report to format
///
/// # Returns
///
/// A `Result` containing the JSON string or a serialization error.
///
/// # Errors
///
/// Returns an error if the report cannot be serialized to JSON.
pub fn format_json_compact(report: &AuditReport) -> Result<String, serde_json::Error> {
    serde_json::to_string(report)
}

/// Extension trait for `AuditReport` to add formatting methods.
///
/// This trait adds convenience methods directly on `AuditReport` for formatting.
pub trait AuditReportExt {
    /// Formats the report as Markdown with default options.
    fn to_markdown(&self) -> String;

    /// Formats the report as Markdown with custom options.
    fn to_markdown_with_options(&self, options: &FormatOptions) -> String;

    /// Formats the report as JSON.
    fn to_json(&self) -> Result<String, serde_json::Error>;

    /// Formats the report as compact JSON.
    fn to_json_compact(&self) -> Result<String, serde_json::Error>;
}

impl AuditReportExt for AuditReport {
    fn to_markdown(&self) -> String {
        format_markdown(self, &FormatOptions::default())
    }

    fn to_markdown_with_options(&self, options: &FormatOptions) -> String {
        format_markdown(self, options)
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        format_json(self)
    }

    fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        format_json_compact(self)
    }
}
