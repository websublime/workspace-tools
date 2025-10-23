//! Issue types and severity levels for audit reports.
//!
//! **What**: Defines the structure of audit issues, including severity levels,
//! categories, and metadata for programmatic access.
//!
//! **How**: Issues are created by audit sections during analysis and provide
//! detailed information about problems found in the project, along with
//! suggestions for resolution.
//!
//! **Why**: To provide a standardized way to represent and communicate audit
//! findings across different audit sections, enabling consistent reporting
//! and filtering.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single audit issue found during analysis.
///
/// Issues represent problems, warnings, or informational findings discovered
/// during the audit process. Each issue has a severity level, category, and
/// detailed information to help developers understand and resolve it.
///
/// # Examples
///
/// ## Creating a critical issue
///
/// ```rust
/// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
/// use std::collections::HashMap;
///
/// let issue = AuditIssue {
///     severity: IssueSeverity::Critical,
///     category: IssueCategory::Upgrades,
///     title: "Deprecated package detected".to_string(),
///     description: "Package 'old-lib' is deprecated and should be replaced".to_string(),
///     affected_packages: vec!["my-app".to_string()],
///     suggestion: Some("Consider migrating to 'new-lib'".to_string()),
///     metadata: HashMap::new(),
/// };
///
/// assert_eq!(issue.severity, IssueSeverity::Critical);
/// ```
///
/// ## With metadata
///
/// ```rust
/// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
/// use std::collections::HashMap;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("package".to_string(), "lodash".to_string());
/// metadata.insert("current_version".to_string(), "4.17.20".to_string());
/// metadata.insert("latest_version".to_string(), "4.17.21".to_string());
///
/// let issue = AuditIssue {
///     severity: IssueSeverity::Warning,
///     category: IssueCategory::Upgrades,
///     title: "Major upgrade available".to_string(),
///     description: "Package has a major version upgrade available".to_string(),
///     affected_packages: vec!["my-package".to_string()],
///     suggestion: Some("Review breaking changes before upgrading".to_string()),
///     metadata,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditIssue {
    /// Severity level of the issue.
    ///
    /// Determines how critical the issue is and affects filtering and reporting.
    pub severity: IssueSeverity,

    /// Category of the issue.
    ///
    /// Groups related issues together for better organization in reports.
    pub category: IssueCategory,

    /// Brief title describing the issue.
    ///
    /// Should be concise and descriptive, suitable for display in lists.
    pub title: String,

    /// Detailed description of the issue.
    ///
    /// Provides context and explains why this is a problem.
    pub description: String,

    /// List of package names affected by this issue.
    ///
    /// Package names should match the `name` field in package.json.
    pub affected_packages: Vec<String>,

    /// Optional suggested action to resolve the issue.
    ///
    /// Provides guidance on how to fix or mitigate the problem.
    pub suggestion: Option<String>,

    /// Additional metadata for programmatic access.
    ///
    /// Can contain any key-value pairs relevant to the issue, such as
    /// version numbers, dependency names, commit hashes, etc.
    pub metadata: HashMap<String, String>,
}

impl AuditIssue {
    /// Creates a new audit issue with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the issue
    /// * `category` - The category this issue belongs to
    /// * `title` - A brief title describing the issue
    /// * `description` - Detailed description of the issue
    ///
    /// # Returns
    ///
    /// A new `AuditIssue` with empty affected_packages, no suggestion, and empty metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let issue = AuditIssue::new(
    ///     IssueSeverity::Warning,
    ///     IssueCategory::Dependencies,
    ///     "Circular dependency detected".to_string(),
    ///     "Package A depends on B which depends on A".to_string(),
    /// );
    ///
    /// assert_eq!(issue.severity, IssueSeverity::Warning);
    /// assert!(issue.affected_packages.is_empty());
    /// ```
    #[must_use]
    pub fn new(
        severity: IssueSeverity,
        category: IssueCategory,
        title: String,
        description: String,
    ) -> Self {
        Self {
            severity,
            category,
            title,
            description,
            affected_packages: Vec::new(),
            suggestion: None,
            metadata: HashMap::new(),
        }
    }

    /// Adds an affected package to this issue.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package affected by this issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let mut issue = AuditIssue::new(
    ///     IssueSeverity::Info,
    ///     IssueCategory::Upgrades,
    ///     "Update available".to_string(),
    ///     "A new version is available".to_string(),
    /// );
    ///
    /// issue.add_affected_package("my-app".to_string());
    /// issue.add_affected_package("my-lib".to_string());
    ///
    /// assert_eq!(issue.affected_packages.len(), 2);
    /// ```
    pub fn add_affected_package(&mut self, package_name: String) {
        self.affected_packages.push(package_name);
    }

    /// Sets the suggestion for this issue.
    ///
    /// # Arguments
    ///
    /// * `suggestion` - Suggested action to resolve the issue
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let mut issue = AuditIssue::new(
    ///     IssueSeverity::Warning,
    ///     IssueCategory::BreakingChanges,
    ///     "Breaking change detected".to_string(),
    ///     "API has changed".to_string(),
    /// );
    ///
    /// issue.set_suggestion("Review migration guide".to_string());
    ///
    /// assert!(issue.suggestion.is_some());
    /// ```
    pub fn set_suggestion(&mut self, suggestion: String) {
        self.suggestion = Some(suggestion);
    }

    /// Adds a metadata entry to this issue.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let mut issue = AuditIssue::new(
    ///     IssueSeverity::Info,
    ///     IssueCategory::Upgrades,
    ///     "Patch available".to_string(),
    ///     "Security patch available".to_string(),
    /// );
    ///
    /// issue.add_metadata("package".to_string(), "express".to_string());
    /// issue.add_metadata("current".to_string(), "4.17.1".to_string());
    /// issue.add_metadata("latest".to_string(), "4.17.3".to_string());
    ///
    /// assert_eq!(issue.metadata.get("package"), Some(&"express".to_string()));
    /// ```
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Returns whether this is a critical issue.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let critical = AuditIssue::new(
    ///     IssueSeverity::Critical,
    ///     IssueCategory::Security,
    ///     "Security vulnerability".to_string(),
    ///     "CVE-2021-12345 detected".to_string(),
    /// );
    ///
    /// let warning = AuditIssue::new(
    ///     IssueSeverity::Warning,
    ///     IssueCategory::Upgrades,
    ///     "Update available".to_string(),
    ///     "New version available".to_string(),
    /// );
    ///
    /// assert!(critical.is_critical());
    /// assert!(!warning.is_critical());
    /// ```
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.severity == IssueSeverity::Critical
    }

    /// Returns whether this is a warning issue.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let warning = AuditIssue::new(
    ///     IssueSeverity::Warning,
    ///     IssueCategory::Dependencies,
    ///     "Unused dependency".to_string(),
    ///     "Package is not used".to_string(),
    /// );
    ///
    /// assert!(warning.is_warning());
    /// assert!(!warning.is_critical());
    /// ```
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.severity == IssueSeverity::Warning
    }

    /// Returns whether this is an informational issue.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
    ///
    /// let info = AuditIssue::new(
    ///     IssueSeverity::Info,
    ///     IssueCategory::Upgrades,
    ///     "Patch available".to_string(),
    ///     "Minor update available".to_string(),
    /// );
    ///
    /// assert!(info.is_info());
    /// assert!(!info.is_warning());
    /// ```
    #[must_use]
    pub fn is_info(&self) -> bool {
        self.severity == IssueSeverity::Info
    }
}

/// Severity level of an audit issue.
///
/// Determines the importance and urgency of addressing an issue.
/// Severities are ordered from most to least severe: Critical > Warning > Info.
///
/// # Examples
///
/// ## Comparing severities
///
/// ```rust
/// use sublime_pkg_tools::audit::IssueSeverity;
///
/// assert!(IssueSeverity::Critical > IssueSeverity::Warning);
/// assert!(IssueSeverity::Warning > IssueSeverity::Info);
/// ```
///
/// ## Filtering by severity
///
/// ```rust
/// use sublime_pkg_tools::audit::{AuditIssue, IssueSeverity, IssueCategory};
///
/// let issues = vec![
///     AuditIssue::new(
///         IssueSeverity::Critical,
///         IssueCategory::Security,
///         "Security issue".to_string(),
///         "Critical security vulnerability".to_string(),
///     ),
///     AuditIssue::new(
///         IssueSeverity::Info,
///         IssueCategory::Upgrades,
///         "Update available".to_string(),
///         "Patch update available".to_string(),
///     ),
/// ];
///
/// let critical_issues: Vec<_> = issues.iter()
///     .filter(|i| i.severity == IssueSeverity::Critical)
///     .collect();
///
/// assert_eq!(critical_issues.len(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum IssueSeverity {
    /// Informational severity for awareness.
    ///
    /// Used for issues that:
    /// - Provide useful information
    /// - Suggest minor or patch upgrades
    /// - Highlight opportunities for improvement
    /// - Can be addressed at convenience
    Info,

    /// Warning severity suggesting attention soon.
    ///
    /// Used for issues that:
    /// - May cause problems if not addressed
    /// - Represent potential breaking changes
    /// - Indicate major version upgrades
    /// - Should be reviewed before next release
    Warning,

    /// Critical severity requiring immediate attention.
    ///
    /// Used for issues that:
    /// - Represent security vulnerabilities
    /// - Block deployment or normal operation
    /// - Involve deprecated packages that must be replaced
    /// - Require urgent action
    Critical,
}

impl IssueSeverity {
    /// Returns the string representation of the severity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::IssueSeverity;
    ///
    /// assert_eq!(IssueSeverity::Critical.as_str(), "critical");
    /// assert_eq!(IssueSeverity::Warning.as_str(), "warning");
    /// assert_eq!(IssueSeverity::Info.as_str(), "info");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueSeverity::Info => "info",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Critical => "critical",
        }
    }
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Category of an audit issue.
///
/// Groups related issues together for better organization and filtering
/// in audit reports.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::IssueCategory;
///
/// let category = IssueCategory::Upgrades;
/// assert_eq!(category, IssueCategory::Upgrades);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum IssueCategory {
    /// Issues related to available package upgrades.
    Upgrades,

    /// Issues related to dependency graph and resolution.
    Dependencies,

    /// Issues related to breaking changes in upgrades.
    BreakingChanges,

    /// Issues related to version consistency across packages.
    VersionConsistency,

    /// Issues related to security vulnerabilities.
    Security,

    /// Other miscellaneous issues.
    Other,
}

impl IssueCategory {
    /// Returns the string representation of the category.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::IssueCategory;
    ///
    /// assert_eq!(IssueCategory::Upgrades.as_str(), "upgrades");
    /// assert_eq!(IssueCategory::Dependencies.as_str(), "dependencies");
    /// assert_eq!(IssueCategory::Security.as_str(), "security");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueCategory::Upgrades => "upgrades",
            IssueCategory::Dependencies => "dependencies",
            IssueCategory::BreakingChanges => "breaking_changes",
            IssueCategory::VersionConsistency => "version_consistency",
            IssueCategory::Security => "security",
            IssueCategory::Other => "other",
        }
    }
}

impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
