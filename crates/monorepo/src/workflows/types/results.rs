//! Workflow result types
//!
//! Result types for different workflow executions including release,
//! development, and change analysis workflows.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::changesets::types::ChangesetApplication;
use crate::tasks::types::results::TaskExecutionResult;
use crate::{BranchComparisonResult, ChangeAnalysis, VersioningPlan, VersioningResult};

/// Result of a release workflow execution
///
/// Contains comprehensive information about what happened during a release,
/// including changes processed, tasks executed, and overall success status.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sublime_monorepo_tools::ReleaseResult;
///
/// let result = ReleaseResult {
///     changes: analysis,
///     tasks: vec![],
///     changesets_applied: vec![],
///     success: true,
///     duration: Duration::from_secs(30),
///     errors: vec![],
///     warnings: vec![],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ReleaseResult {
    /// Analysis of changes included in this release
    pub changes: ChangeAnalysis,

    /// Tasks that were executed during the release
    pub tasks: Vec<TaskExecutionResult>,

    /// Changesets that were applied
    pub changesets_applied: Vec<ChangesetApplication>,

    /// Whether the release was successful
    pub success: bool,

    /// Total duration of the release process
    pub duration: Duration,

    /// Any errors that occurred during the release
    pub errors: Vec<String>,

    /// Warnings generated during the release
    pub warnings: Vec<String>,
}

/// Result of a development workflow execution
///
/// Contains information about development operations like running tests
/// on affected packages and validating changesets.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sublime_monorepo_tools::{DevelopmentResult, ChangeAnalysis};
///
/// let result = DevelopmentResult {
///     changes: analysis,
///     affected_tasks: vec![],
///     recommendations: vec!["Run tests before committing".to_string()],
///     checks_passed: true,
///     duration: Duration::from_secs(15),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct DevelopmentResult {
    /// Analysis of current changes
    pub changes: ChangeAnalysis,

    /// Tasks executed for affected packages
    pub affected_tasks: Vec<TaskExecutionResult>,

    /// Recommendations for the developer
    pub recommendations: Vec<String>,

    /// Whether development checks passed
    pub checks_passed: bool,

    /// Duration of the development workflow
    pub duration: Duration,
}

/// Analysis result for changes workflow
///
/// Provides detailed analysis of changes between branches or commits,
/// including affected packages and recommended actions.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use sublime_monorepo_tools::ChangeAnalysisResult;
///
/// let result = ChangeAnalysisResult {
///     analysis: change_analysis,
///     affected_packages: vec![],
///     version_recommendations: vec![],
///     changesets_required: true,
///     duration: Duration::from_secs(5),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ChangeAnalysisResult {
    /// The change analysis data
    pub analysis: ChangeAnalysis,

    /// Affected packages with their impact level
    pub affected_packages: Vec<AffectedPackageInfo>,

    /// Recommended version bumps
    pub version_recommendations: Vec<VersionRecommendation>,

    /// Whether changesets are required
    pub changesets_required: bool,

    /// Analysis duration
    pub duration: Duration,
}

/// Information about an affected package
///
/// Provides details about how a package is affected by changes,
/// including impact level and dependent packages.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::{AffectedPackageInfo, ImpactLevel};
///
/// let package_info = AffectedPackageInfo {
///     name: "@test/core".to_string(),
///     impact_level: ImpactLevel::Medium,
///     changed_files: vec!["src/lib.rs".to_string()],
///     dependents: vec!["@test/ui".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedPackageInfo {
    /// Package name
    pub name: String,

    /// Impact level of changes
    pub impact_level: ImpactLevel,

    /// Files changed in this package
    pub changed_files: Vec<String>,

    /// Dependent packages that might be affected
    pub dependents: Vec<String>,
}

/// Level of impact for package changes
///
/// Used to categorize the severity of changes to help determine
/// appropriate version bumps and deployment strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Low impact - documentation, tests, or minor changes
    Low,

    /// Medium impact - feature additions or non-breaking changes
    Medium,

    /// High impact - breaking changes or major features
    High,

    /// Critical impact - major breaking changes or security fixes
    Critical,
}

/// Version bump recommendation for a package
///
/// Provides automated recommendations for version bumps based on
/// change analysis with confidence levels.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::{VersionRecommendation, ConfidenceLevel, VersionBumpType};
///
/// let recommendation = VersionRecommendation {
///     package: "@test/core".to_string(),
///     recommended_bump: VersionBumpType::Minor,
///     reason: "Added new API endpoint".to_string(),
///     confidence: ConfidenceLevel::High,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionRecommendation {
    /// Package name
    pub package: String,

    /// Recommended version bump type
    pub recommended_bump: crate::VersionBumpType,

    /// Reason for the recommendation
    pub reason: String,

    /// Confidence level of the recommendation
    pub confidence: ConfidenceLevel,
}

/// Confidence level for automated recommendations
///
/// Indicates how confident the automated system is about a recommendation,
/// helping users understand when manual review is needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Low confidence - manual review recommended
    Low,

    /// Medium confidence - probably correct but worth reviewing
    Medium,

    /// High confidence - very likely to be correct
    High,
}

/// Result of a change analysis workflow (Phase 2)
#[derive(Debug)]
pub enum ChangeAnalysisWorkflowResult {
    /// Result of branch comparison
    BranchComparison(BranchComparisonResult),
    /// Result of change analysis since a reference
    ChangeAnalysis(ChangeAnalysis),
}

/// Result of a versioning workflow execution (Phase 2)
#[derive(Debug)]
pub struct VersioningWorkflowResult {
    /// The versioning operation result
    pub versioning_result: VersioningResult,
    /// The plan that was executed
    pub plan_executed: Option<VersioningPlan>,
    /// Duration of the workflow execution
    pub duration: std::time::Duration,
}
