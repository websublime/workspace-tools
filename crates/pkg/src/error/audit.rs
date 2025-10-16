//! Audit error types for package tools.
//!
//! **What**: Defines error types specific to audit operations, health checks, dependency analysis,
//! and report generation.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! package names, section identifiers, and analysis details. Implements `AsRef<str>` for
//! string conversion.
//!
//! **Why**: To provide clear, actionable error messages for audit operations, enabling
//! users to quickly identify and fix issues with dependency audits, health checks, and
//! report generation.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{AuditError, AuditResult};
//!
//! fn run_audit(package: &str) -> AuditResult<String> {
//!     if package.is_empty() {
//!         return Err(AuditError::InvalidPackage {
//!             package: package.to_string(),
//!             reason: "Package name cannot be empty".to_string(),
//!         });
//!     }
//!     Ok("audit-report".to_string())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for audit operations.
///
/// This type alias simplifies error handling in audit-related functions
/// by defaulting to `AuditError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{AuditError, AuditResult};
///
/// fn analyze_dependencies() -> AuditResult<Vec<String>> {
///     Ok(vec!["issue1".to_string(), "issue2".to_string()])
/// }
/// ```
pub type AuditResult<T> = Result<T, AuditError>;

/// Errors that can occur during audit and health check operations.
///
/// This enum covers all possible error scenarios when running audits,
/// analyzing dependencies, detecting issues, and generating reports.
///
/// # Examples
///
/// ## Handling audit errors
///
/// ```rust
/// use sublime_pkg_tools::error::AuditError;
///
/// fn handle_audit_error(error: AuditError) {
///     match error {
///         AuditError::SectionDisabled { section } => {
///             eprintln!("Audit section '{}' is disabled", section);
///         }
///         AuditError::AnalysisFailed { section, reason } => {
///             eprintln!("Analysis failed for {}: {}", section, reason);
///         }
///         _ => eprintln!("Audit error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::AuditError;
///
/// let error = AuditError::SectionDisabled {
///     section: "upgrades".to_string(),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("section disabled"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum AuditError {
    /// Audit section is disabled in configuration.
    ///
    /// This error occurs when attempting to run an audit section that
    /// has been explicitly disabled in the configuration.
    #[error("Audit section '{section}' is disabled in configuration")]
    SectionDisabled {
        /// Name of the disabled audit section.
        section: String,
    },

    /// Audit analysis failed for a specific section.
    ///
    /// This error occurs when an audit section encounters an error during
    /// analysis, such as dependency resolution or data collection failures.
    #[error("Audit analysis failed for section '{section}': {reason}")]
    AnalysisFailed {
        /// Name of the audit section that failed.
        section: String,
        /// Description of why the analysis failed.
        reason: String,
    },

    /// Report generation failed.
    ///
    /// This error occurs when generating the final audit report fails,
    /// possibly due to formatting errors or missing data.
    #[error("Failed to generate audit report: {reason}")]
    ReportGenerationFailed {
        /// Description of why report generation failed.
        reason: String,
    },

    /// Invalid audit configuration.
    ///
    /// This error occurs when the audit configuration is invalid,
    /// incomplete, or contains conflicting settings.
    #[error("Invalid audit configuration: {reason}")]
    InvalidConfig {
        /// Description of the configuration problem.
        reason: String,
    },

    /// Package not found during audit.
    ///
    /// This error occurs when attempting to audit a package that does
    /// not exist in the workspace.
    #[error("Package '{package}' not found in workspace")]
    PackageNotFound {
        /// Name of the package that was not found.
        package: String,
    },

    /// Dependency graph construction failed.
    ///
    /// This error occurs when building the dependency graph for analysis
    /// fails due to circular dependencies or parsing errors.
    #[error("Failed to construct dependency graph: {reason}")]
    DependencyGraphFailed {
        /// Description of why graph construction failed.
        reason: String,
    },

    /// Circular dependency detection failed.
    ///
    /// This error occurs when the circular dependency detection algorithm
    /// encounters an error or fails to complete.
    #[error("Circular dependency detection failed: {reason}")]
    CircularDependencyDetectionFailed {
        /// Description of the detection error.
        reason: String,
    },

    /// Missing dependency analysis failed.
    ///
    /// This error occurs when checking for missing dependencies encounters
    /// errors in file scanning or import analysis.
    #[error("Missing dependency analysis failed: {reason}")]
    MissingDependencyAnalysisFailed {
        /// Description of the analysis error.
        reason: String,
    },

    /// Unused dependency analysis failed.
    ///
    /// This error occurs when checking for unused dependencies encounters
    /// errors in usage scanning or analysis.
    #[error("Unused dependency analysis failed: {reason}")]
    UnusedDependencyAnalysisFailed {
        /// Description of the analysis error.
        reason: String,
    },

    /// Version conflict detection failed.
    ///
    /// This error occurs when detecting version conflicts across packages
    /// fails due to resolution errors.
    #[error("Version conflict detection failed: {reason}")]
    VersionConflictDetectionFailed {
        /// Description of the detection error.
        reason: String,
    },

    /// Breaking changes detection failed.
    ///
    /// This error occurs when analyzing commits or changelogs for breaking
    /// changes fails.
    #[error("Breaking changes detection failed: {reason}")]
    BreakingChangesDetectionFailed {
        /// Description of the detection error.
        reason: String,
    },

    /// Upgrade detection failed during audit.
    ///
    /// This error occurs when checking for available upgrades as part of
    /// the audit fails.
    #[error("Upgrade detection failed: {reason}")]
    UpgradeDetectionFailed {
        /// Description of the detection error.
        reason: String,
    },

    /// Dependency categorization failed.
    ///
    /// This error occurs when categorizing dependencies into internal,
    /// external, and local links fails.
    #[error("Dependency categorization failed: {reason}")]
    CategorizationFailed {
        /// Description of the categorization error.
        reason: String,
    },

    /// Health score calculation failed.
    ///
    /// This error occurs when computing the overall health score based
    /// on audit findings fails.
    #[error("Health score calculation failed: {reason}")]
    HealthScoreCalculationFailed {
        /// Description of the calculation error.
        reason: String,
    },

    /// Invalid severity level specified.
    ///
    /// This error occurs when an invalid severity level is specified
    /// for filtering or reporting.
    #[error("Invalid severity level '{severity}': expected 'critical', 'warning', or 'info'")]
    InvalidSeverity {
        /// The invalid severity level.
        severity: String,
    },

    /// No issues found but audit was expected to find some.
    ///
    /// This error occurs in strict mode when an audit is expected to
    /// find issues but none are detected.
    #[error("No issues found in audit (expected at least one in strict mode)")]
    NoIssuesFound,

    /// Workspace analysis failed.
    ///
    /// This error occurs when analyzing the workspace structure or
    /// detecting packages fails.
    #[error("Workspace analysis failed: {reason}")]
    WorkspaceAnalysisFailed {
        /// Description of the analysis error.
        reason: String,
    },

    /// File system error during audit operations.
    ///
    /// This error occurs when filesystem operations fail during audit
    /// execution, such as reading package files or writing reports.
    #[error("Filesystem error at '{path}': {reason}")]
    FileSystemError {
        /// Path where the error occurred.
        path: PathBuf,
        /// Description of the filesystem error.
        reason: String,
    },

    /// Git operation failed during audit.
    ///
    /// This error occurs when git operations required for audit analysis
    /// (e.g., checking for breaking changes) fail.
    #[error("Git operation failed during audit: {operation} - {reason}")]
    GitError {
        /// Description of the git operation that failed.
        operation: String,
        /// Detailed error message from git.
        reason: String,
    },

    /// Report format is not supported.
    ///
    /// This error occurs when attempting to generate a report in an
    /// unsupported format.
    #[error("Unsupported report format: {format}")]
    UnsupportedFormat {
        /// The unsupported format name.
        format: String,
    },

    /// Report export failed.
    ///
    /// This error occurs when exporting the audit report to a file fails,
    /// possibly due to permission issues or disk space.
    #[error("Failed to export report to '{path}': {reason}")]
    ExportFailed {
        /// Path where export was attempted.
        path: PathBuf,
        /// Description of why export failed.
        reason: String,
    },

    /// Timeout exceeded during audit operations.
    ///
    /// This error occurs when audit operations take longer than the
    /// configured timeout period.
    #[error("Audit operation timed out after {duration_secs} seconds")]
    Timeout {
        /// Duration in seconds before timeout.
        duration_secs: u64,
    },

    /// Registry communication failed during audit.
    ///
    /// This error occurs when checking for upgrades or deprecated packages
    /// requires registry access but communication fails.
    #[error("Registry communication failed during audit: {reason}")]
    RegistryError {
        /// Description of the registry error.
        reason: String,
    },

    /// Data inconsistency detected in audit results.
    ///
    /// This error occurs when audit data is internally inconsistent,
    /// possibly indicating a bug or corrupted state.
    #[error("Data inconsistency detected: {reason}")]
    DataInconsistency {
        /// Description of the inconsistency.
        reason: String,
    },

    /// Audit threshold exceeded.
    ///
    /// This error occurs when the number of issues exceeds configured
    /// thresholds, causing the audit to fail.
    #[error(
        "Audit threshold exceeded: {threshold_type} limit of {limit} exceeded with {actual} issues"
    )]
    ThresholdExceeded {
        /// Type of threshold (e.g., "critical", "warning").
        threshold_type: String,
        /// Maximum allowed issues.
        limit: usize,
        /// Actual number of issues found.
        actual: usize,
    },

    /// Invalid workspace root for audit.
    ///
    /// This error occurs when the workspace root is invalid or cannot
    /// be determined for audit operations.
    #[error("Invalid workspace root '{path}': {reason}")]
    InvalidWorkspaceRoot {
        /// Path to the invalid workspace root.
        path: PathBuf,
        /// Description of why it's invalid.
        reason: String,
    },
}

impl AsRef<str> for AuditError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::AuditError;
    ///
    /// let error = AuditError::SectionDisabled {
    ///     section: "upgrades".to_string(),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("section disabled"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::SectionDisabled { .. } => "audit section disabled",
            Self::AnalysisFailed { .. } => "audit analysis failed",
            Self::ReportGenerationFailed { .. } => "report generation failed",
            Self::InvalidConfig { .. } => "invalid configuration",
            Self::PackageNotFound { .. } => "package not found",
            Self::DependencyGraphFailed { .. } => "dependency graph failed",
            Self::CircularDependencyDetectionFailed { .. } => {
                "circular dependency detection failed"
            }
            Self::MissingDependencyAnalysisFailed { .. } => "missing dependency analysis failed",
            Self::UnusedDependencyAnalysisFailed { .. } => "unused dependency analysis failed",
            Self::VersionConflictDetectionFailed { .. } => "version conflict detection failed",
            Self::BreakingChangesDetectionFailed { .. } => "breaking changes detection failed",
            Self::UpgradeDetectionFailed { .. } => "upgrade detection failed",
            Self::CategorizationFailed { .. } => "categorization failed",
            Self::HealthScoreCalculationFailed { .. } => "health score calculation failed",
            Self::InvalidSeverity { .. } => "invalid severity",
            Self::NoIssuesFound => "no issues found",
            Self::WorkspaceAnalysisFailed { .. } => "workspace analysis failed",
            Self::FileSystemError { .. } => "filesystem error",
            Self::GitError { .. } => "git error",
            Self::UnsupportedFormat { .. } => "unsupported format",
            Self::ExportFailed { .. } => "export failed",
            Self::Timeout { .. } => "timeout",
            Self::RegistryError { .. } => "registry error",
            Self::DataInconsistency { .. } => "data inconsistency",
            Self::ThresholdExceeded { .. } => "threshold exceeded",
            Self::InvalidWorkspaceRoot { .. } => "invalid workspace root",
        }
    }
}

impl AuditError {
    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some audit errors (like filesystem errors or registry errors) might be
    /// recoverable through retry, while others (like invalid configuration) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::AuditError;
    /// use std::path::PathBuf;
    ///
    /// let fs_error = AuditError::FileSystemError {
    ///     path: PathBuf::from("package.json"),
    ///     reason: "temporary lock".to_string(),
    /// };
    /// assert!(fs_error.is_transient());
    ///
    /// let config_error = AuditError::InvalidConfig {
    ///     reason: "missing field".to_string(),
    /// };
    /// assert!(!config_error.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::FileSystemError { .. }
                | Self::GitError { .. }
                | Self::RegistryError { .. }
                | Self::Timeout { .. }
        )
    }

    /// Returns whether this error is fatal and should stop the audit.
    ///
    /// Some errors are fatal and prevent the audit from completing, while
    /// others can be treated as warnings or partial failures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::AuditError;
    ///
    /// let invalid_config = AuditError::InvalidConfig {
    ///     reason: "missing required field".to_string(),
    /// };
    /// assert!(invalid_config.is_fatal());
    ///
    /// let section_disabled = AuditError::SectionDisabled {
    ///     section: "upgrades".to_string(),
    /// };
    /// assert!(!section_disabled.is_fatal());
    /// ```
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::InvalidConfig { .. }
                | Self::InvalidWorkspaceRoot { .. }
                | Self::WorkspaceAnalysisFailed { .. }
                | Self::DataInconsistency { .. }
        )
    }

    /// Returns whether this error is related to dependency analysis.
    ///
    /// This helper method identifies errors that occur during dependency
    /// analysis operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::AuditError;
    ///
    /// let circular_error = AuditError::CircularDependencyDetectionFailed {
    ///     reason: "cycle detected".to_string(),
    /// };
    /// assert!(circular_error.is_dependency_related());
    ///
    /// let report_error = AuditError::ReportGenerationFailed {
    ///     reason: "formatting error".to_string(),
    /// };
    /// assert!(!report_error.is_dependency_related());
    /// ```
    #[must_use]
    pub fn is_dependency_related(&self) -> bool {
        matches!(
            self,
            Self::DependencyGraphFailed { .. }
                | Self::CircularDependencyDetectionFailed { .. }
                | Self::MissingDependencyAnalysisFailed { .. }
                | Self::UnusedDependencyAnalysisFailed { .. }
                | Self::VersionConflictDetectionFailed { .. }
                | Self::CategorizationFailed { .. }
        )
    }
}

