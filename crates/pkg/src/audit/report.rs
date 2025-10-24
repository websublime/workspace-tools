//! Audit report structures and implementations.
//!
//! **What**: Defines the complete audit report structure that aggregates all
//! audit section results, summary statistics, and health scores into a single
//! cohesive report.
//!
//! **How**: The `AuditReport` structure collects results from all audit sections,
//! calculates summary statistics, and provides methods for filtering, querying,
//! and formatting the results.
//!
//! **Why**: To provide a unified view of all audit findings that can be easily
//! consumed by CLI tools, CI/CD pipelines, or other automation systems.

use crate::audit::issue::{AuditIssue, IssueSeverity};
use crate::audit::sections::{
    BreakingChangesAuditSection, DependencyAuditSection, DependencyCategorization,
    UpgradeAuditSection, VersionConsistencyAuditSection,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete audit report containing all findings.
///
/// This structure aggregates results from all audit sections and provides
/// a comprehensive view of the project's health status.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::AuditReport;
///
/// // After running an audit
/// let report = audit_manager.run_audit().await?;
///
/// // Check if audit passed
/// if !report.passed() {
///     eprintln!("Audit failed with {} critical issues",
///         report.summary.critical_issues);
/// }
///
/// // Export to markdown
/// let markdown = report.to_markdown();
/// std::fs::write("AUDIT.md", markdown)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// When the audit was performed
    pub audited_at: DateTime<Utc>,

    /// Workspace root path
    pub workspace_root: PathBuf,

    /// Whether this is a monorepo project
    pub is_monorepo: bool,

    /// All audit sections
    pub sections: AuditSections,

    /// Summary of all findings
    pub summary: AuditSummary,

    /// Overall health score (0-100)
    pub health_score: u8,
}

impl AuditReport {
    /// Creates a new audit report.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The workspace root path
    /// * `is_monorepo` - Whether this is a monorepo
    /// * `sections` - All audit sections
    /// * `health_score` - Calculated health score
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let report = AuditReport::new(
    ///     workspace_root,
    ///     true,
    ///     sections,
    ///     85
    /// );
    /// ```
    #[must_use]
    pub fn new(
        workspace_root: PathBuf,
        is_monorepo: bool,
        sections: AuditSections,
        health_score: u8,
    ) -> Self {
        let summary = AuditSummary::from_sections(&sections);

        Self {
            audited_at: Utc::now(),
            workspace_root,
            is_monorepo,
            sections,
            summary,
            health_score,
        }
    }

    /// Gets all issues filtered by severity.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to issues matching the severity level.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let critical = report.issues_by_severity(IssueSeverity::Critical);
    /// for issue in critical {
    ///     eprintln!("Critical: {}", issue.title);
    /// }
    /// ```
    #[must_use]
    pub fn issues_by_severity(&self, severity: IssueSeverity) -> Vec<&AuditIssue> {
        self.all_issues().into_iter().filter(|issue| issue.severity == severity).collect()
    }

    /// Gets all critical issues.
    ///
    /// This is a convenience method equivalent to `issues_by_severity(IssueSeverity::Critical)`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if !report.critical_issues().is_empty() {
    ///     eprintln!("Found critical issues!");
    /// }
    /// ```
    #[must_use]
    pub fn critical_issues(&self) -> Vec<&AuditIssue> {
        self.issues_by_severity(IssueSeverity::Critical)
    }

    /// Gets all warnings.
    ///
    /// This is a convenience method equivalent to `issues_by_severity(IssueSeverity::Warning)`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// println!("Found {} warnings", report.warnings().len());
    /// ```
    #[must_use]
    pub fn warnings(&self) -> Vec<&AuditIssue> {
        self.issues_by_severity(IssueSeverity::Warning)
    }

    /// Gets all info-level issues.
    ///
    /// This is a convenience method equivalent to `issues_by_severity(IssueSeverity::Info)`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// println!("Found {} informational items", report.info_items().len());
    /// ```
    #[must_use]
    pub fn info_items(&self) -> Vec<&AuditIssue> {
        self.issues_by_severity(IssueSeverity::Info)
    }

    /// Checks if the audit passed (no critical issues).
    ///
    /// An audit is considered to have passed if there are no critical issues.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if report.passed() {
    ///     println!("✅ Audit passed!");
    /// } else {
    ///     eprintln!("❌ Audit failed");
    ///     std::process::exit(1);
    /// }
    /// ```
    #[must_use]
    pub fn passed(&self) -> bool {
        self.summary.critical_issues == 0
    }

    /// Gets all issues from all sections.
    ///
    /// This collects all issues from upgrades, dependencies, breaking changes,
    /// and version consistency sections.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let all = report.all_issues();
    /// println!("Total issues: {}", all.len());
    /// ```
    #[must_use]
    pub fn all_issues(&self) -> Vec<&AuditIssue> {
        let mut issues = Vec::new();
        issues.extend(&self.sections.upgrades.issues);
        issues.extend(&self.sections.dependencies.issues);
        issues.extend(&self.sections.breaking_changes.issues);
        issues.extend(&self.sections.version_consistency.issues);
        issues
    }

    /// Gets the total number of issues.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// println!("Total issues: {}", report.total_issues());
    /// ```
    #[must_use]
    pub fn total_issues(&self) -> usize {
        self.summary.total_issues
    }
}

/// All audit sections collected in one structure.
///
/// Each section contains specialized audit information and related issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSections {
    /// Upgrade opportunities audit
    pub upgrades: UpgradeAuditSection,

    /// Dependency graph analysis audit
    pub dependencies: DependencyAuditSection,

    /// Breaking changes detection audit
    pub breaking_changes: BreakingChangesAuditSection,

    /// Dependency categorization results
    pub categorization: DependencyCategorization,

    /// Version consistency audit
    pub version_consistency: VersionConsistencyAuditSection,
}

impl AuditSections {
    /// Creates a new AuditSections instance.
    ///
    /// # Arguments
    ///
    /// * `upgrades` - Upgrade audit results
    /// * `dependencies` - Dependency audit results
    /// * `breaking_changes` - Breaking changes audit results
    /// * `categorization` - Dependency categorization results
    /// * `version_consistency` - Version consistency audit results
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let sections = AuditSections::new(
    ///     upgrades,
    ///     dependencies,
    ///     breaking_changes,
    ///     categorization,
    ///     version_consistency,
    /// );
    /// ```
    #[must_use]
    pub fn new(
        upgrades: UpgradeAuditSection,
        dependencies: DependencyAuditSection,
        breaking_changes: BreakingChangesAuditSection,
        categorization: DependencyCategorization,
        version_consistency: VersionConsistencyAuditSection,
    ) -> Self {
        Self { upgrades, dependencies, breaking_changes, categorization, version_consistency }
    }
}

/// Summary of audit findings across all sections.
///
/// Provides aggregate statistics about the audit results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total packages analyzed
    pub packages_analyzed: usize,

    /// Total dependencies analyzed
    pub dependencies_analyzed: usize,

    /// Total issues found across all sections
    pub total_issues: usize,

    /// Number of critical issues
    pub critical_issues: usize,

    /// Number of warnings
    pub warnings: usize,

    /// Number of info items
    pub info_items: usize,

    /// List of suggested actions to improve health
    pub suggested_actions: Vec<String>,
}

impl AuditSummary {
    /// Creates a summary from audit sections.
    ///
    /// This calculates all summary statistics by analyzing the provided sections.
    ///
    /// # Arguments
    ///
    /// * `sections` - The audit sections to summarize
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let summary = AuditSummary::from_sections(&sections);
    /// println!("Found {} issues", summary.total_issues);
    /// ```
    #[must_use]
    pub fn from_sections(sections: &AuditSections) -> Self {
        let mut critical_issues = 0;
        let mut warnings = 0;
        let mut info_items = 0;

        // Count issues from all sections
        for issue in &sections.upgrades.issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues += 1,
                IssueSeverity::Warning => warnings += 1,
                IssueSeverity::Info => info_items += 1,
            }
        }

        for issue in &sections.dependencies.issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues += 1,
                IssueSeverity::Warning => warnings += 1,
                IssueSeverity::Info => info_items += 1,
            }
        }

        for issue in &sections.breaking_changes.issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues += 1,
                IssueSeverity::Warning => warnings += 1,
                IssueSeverity::Info => info_items += 1,
            }
        }

        for issue in &sections.version_consistency.issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues += 1,
                IssueSeverity::Warning => warnings += 1,
                IssueSeverity::Info => info_items += 1,
            }
        }

        let total_issues = critical_issues + warnings + info_items;

        // Calculate packages analyzed (from categorization)
        let packages_analyzed = sections.categorization.stats.total_packages;

        // Calculate dependencies analyzed
        let dependencies_analyzed = sections.categorization.stats.internal_packages
            + sections.categorization.stats.external_packages;

        // Generate suggested actions
        let suggested_actions = Self::generate_suggestions(sections);

        Self {
            packages_analyzed,
            dependencies_analyzed,
            total_issues,
            critical_issues,
            warnings,
            info_items,
            suggested_actions,
        }
    }

    /// Generates suggested actions based on audit findings.
    fn generate_suggestions(sections: &AuditSections) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Upgrade suggestions
        if sections.upgrades.major_upgrades > 0 {
            suggestions.push(format!(
                "Review {} major upgrade(s) for breaking changes",
                sections.upgrades.major_upgrades
            ));
        }

        if !sections.upgrades.deprecated_packages.is_empty() {
            suggestions.push(format!(
                "Replace {} deprecated package(s)",
                sections.upgrades.deprecated_packages.len()
            ));
        }

        // Dependency suggestions
        if !sections.dependencies.circular_dependencies.is_empty() {
            suggestions.push(format!(
                "Resolve {} circular dependenc(y/ies)",
                sections.dependencies.circular_dependencies.len()
            ));
        }

        if !sections.dependencies.version_conflicts.is_empty() {
            suggestions.push(format!(
                "Fix {} version conflict(s)",
                sections.dependencies.version_conflicts.len()
            ));
        }

        // Breaking changes suggestions
        if sections.breaking_changes.total_breaking_changes > 0 {
            suggestions.push(format!(
                "Review {} breaking change(s) before release",
                sections.breaking_changes.total_breaking_changes
            ));
        }

        // Version consistency suggestions
        if !sections.version_consistency.inconsistencies.is_empty() {
            suggestions.push(format!(
                "Align {} version inconsistenc(y/ies)",
                sections.version_consistency.inconsistencies.len()
            ));
        }

        // General suggestions if no specific issues
        if suggestions.is_empty() {
            suggestions.push("No immediate actions required".to_string());
        }

        suggestions
    }

    /// Creates a new empty summary.
    ///
    /// Useful for testing or when no audit has been performed yet.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            packages_analyzed: 0,
            dependencies_analyzed: 0,
            total_issues: 0,
            critical_issues: 0,
            warnings: 0,
            info_items: 0,
            suggested_actions: Vec::new(),
        }
    }
}
