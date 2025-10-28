//! Health score calculation for audit reports.
//!
//! **What**: Provides a health score calculation algorithm that produces a 0-100
//! score based on audit findings, with configurable weights for different issue types.
//!
//! **How**: The health score is calculated by assigning point deductions for each
//! issue found during auditing, weighted by severity and category. The algorithm
//! starts at 100 and deducts points based on the number and severity of issues.
//!
//! **Why**: To provide a single, actionable metric that represents the overall health
//! of a project's dependency ecosystem, enabling quick assessment and tracking over time.
//!
//! # Algorithm
//!
//! The health score calculation follows these principles:
//!
//! 1. **Start at 100**: Perfect score with no issues
//! 2. **Deduct points per issue**: Each issue deducts points based on its severity
//! 3. **Category weights**: Different issue categories have different weights
//! 4. **Diminishing returns**: Multiple issues of the same type have less impact
//! 5. **Floor at 0**: Score cannot go below 0
//!
//! ## Base Deductions (per issue)
//!
//! - **Critical**: 15 points
//! - **Warning**: 5 points
//! - **Info**: 1 point
//!
//! ## Category Multipliers
//!
//! - **Security**: 1.5x (most important)
//! - **Breaking Changes**: 1.3x (high impact)
//! - **Dependencies**: 1.2x (structural issues)
//! - **Version Consistency**: 1.0x (quality issues)
//! - **Upgrades**: 0.8x (maintenance issues)
//! - **Other**: 1.0x (miscellaneous)
//!
//! ## Diminishing Returns
//!
//! To prevent a single category from dominating the score, each additional issue
//! in the same category has reduced impact:
//!
//! - 1st issue: 100% impact
//! - 2nd issue: 90% impact
//! - 3rd issue: 80% impact
//! - 4th+ issues: 70% impact
//!
//! # Examples
//!
//! ## Calculate score for a clean project
//!
//! ```rust
//! use sublime_pkg_tools::audit::{calculate_health_score, HealthScoreWeights, AuditIssue};
//!
//! let issues: Vec<AuditIssue> = vec![];
//! let weights = HealthScoreWeights::default();
//! let score = calculate_health_score(&issues, &weights);
//!
//! assert_eq!(score, 100);
//! ```
//!
//! ## Calculate score with issues
//!
//! ```rust
//! use sublime_pkg_tools::audit::{
//!     calculate_health_score, HealthScoreWeights, AuditIssue,
//!     IssueSeverity, IssueCategory
//! };
//!
//! let issues = vec![
//!     AuditIssue::new(
//!         IssueSeverity::Critical,
//!         IssueCategory::Security,
//!         "Security vulnerability".to_string(),
//!         "CVE detected".to_string(),
//!     ),
//!     AuditIssue::new(
//!         IssueSeverity::Warning,
//!         IssueCategory::Upgrades,
//!         "Upgrade available".to_string(),
//!         "Major version available".to_string(),
//!     ),
//! ];
//!
//! let weights = HealthScoreWeights::default();
//! let score = calculate_health_score(&issues, &weights);
//!
//! // Critical security issue (15 * 1.5 = 22.5) + Warning upgrade (5 * 0.8 = 4)
//! // 100 - 22.5 - 4 = 73.5 -> 73
//! assert!(score < 100);
//! assert!(score >= 70);
//! ```
//!
//! ## Custom weights
//!
//! ```rust
//! use sublime_pkg_tools::audit::{
//!     calculate_health_score, HealthScoreWeights, AuditIssue,
//!     IssueSeverity, IssueCategory
//! };
//!
//! let issues = vec![
//!     AuditIssue::new(
//!         IssueSeverity::Warning,
//!         IssueCategory::Upgrades,
//!         "Upgrade available".to_string(),
//!         "Update needed".to_string(),
//!     ),
//! ];
//!
//! // Custom weights that penalize upgrades more heavily
//! let weights = HealthScoreWeights {
//!     critical_weight: 15.0,
//!     warning_weight: 5.0,
//!     info_weight: 1.0,
//!     security_multiplier: 1.5,
//!     breaking_changes_multiplier: 1.3,
//!     dependencies_multiplier: 1.2,
//!     version_consistency_multiplier: 1.0,
//!     upgrades_multiplier: 1.5, // Increased from default 0.8
//!     other_multiplier: 1.0,
//! };
//!
//! let score = calculate_health_score(&issues, &weights);
//! assert!(score < 100);
//! ```

use crate::audit::{AuditIssue, IssueCategory, IssueSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Weights used for health score calculation.
///
/// These weights control how much each type of issue affects the overall health score.
/// All weights are positive numbers, where higher values mean more impact on the score.
///
/// # Examples
///
/// ## Using default weights
///
/// ```rust
/// use sublime_pkg_tools::audit::HealthScoreWeights;
///
/// let weights = HealthScoreWeights::default();
/// assert_eq!(weights.critical_weight, 15.0);
/// assert_eq!(weights.warning_weight, 5.0);
/// ```
///
/// ## Custom weights for CI/CD
///
/// ```rust
/// use sublime_pkg_tools::audit::HealthScoreWeights;
///
/// // More strict weights for production deployments
/// let strict_weights = HealthScoreWeights {
///     critical_weight: 20.0,  // Even more penalty for critical issues
///     warning_weight: 8.0,     // Warnings are more important
///     info_weight: 2.0,        // Even info matters
///     ..HealthScoreWeights::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthScoreWeights {
    /// Points deducted per critical issue.
    ///
    /// # Default: `15.0`
    pub critical_weight: f64,

    /// Points deducted per warning issue.
    ///
    /// # Default: `5.0`
    pub warning_weight: f64,

    /// Points deducted per info issue.
    ///
    /// # Default: `1.0`
    pub info_weight: f64,

    /// Multiplier for security issues.
    ///
    /// # Default: `1.5`
    pub security_multiplier: f64,

    /// Multiplier for breaking changes issues.
    ///
    /// # Default: `1.3`
    pub breaking_changes_multiplier: f64,

    /// Multiplier for dependency issues.
    ///
    /// # Default: `1.2`
    pub dependencies_multiplier: f64,

    /// Multiplier for version consistency issues.
    ///
    /// # Default: `1.0`
    pub version_consistency_multiplier: f64,

    /// Multiplier for upgrade issues.
    ///
    /// # Default: `0.8`
    pub upgrades_multiplier: f64,

    /// Multiplier for other issues.
    ///
    /// # Default: `1.0`
    pub other_multiplier: f64,
}

impl Default for HealthScoreWeights {
    fn default() -> Self {
        Self {
            critical_weight: 15.0,
            warning_weight: 5.0,
            info_weight: 1.0,
            security_multiplier: 1.5,
            breaking_changes_multiplier: 1.3,
            dependencies_multiplier: 1.2,
            version_consistency_multiplier: 1.0,
            upgrades_multiplier: 0.8,
            other_multiplier: 1.0,
        }
    }
}

impl HealthScoreWeights {
    /// Creates weights optimized for CI/CD pipelines.
    ///
    /// These weights are more strict and penalize issues more heavily,
    /// making them suitable for deployment gates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::HealthScoreWeights;
    ///
    /// let ci_weights = HealthScoreWeights::strict();
    /// assert!(ci_weights.critical_weight > 15.0);
    /// ```
    #[must_use]
    pub fn strict() -> Self {
        Self {
            critical_weight: 20.0,
            warning_weight: 8.0,
            info_weight: 2.0,
            security_multiplier: 2.0,
            breaking_changes_multiplier: 1.5,
            dependencies_multiplier: 1.3,
            version_consistency_multiplier: 1.2,
            upgrades_multiplier: 1.0,
            other_multiplier: 1.0,
        }
    }

    /// Creates weights optimized for development environments.
    ///
    /// These weights are more lenient and focus on critical issues,
    /// making them suitable for local development.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::HealthScoreWeights;
    ///
    /// let dev_weights = HealthScoreWeights::lenient();
    /// assert!(dev_weights.critical_weight < 15.0);
    /// ```
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            critical_weight: 10.0,
            warning_weight: 3.0,
            info_weight: 0.5,
            security_multiplier: 1.5,
            breaking_changes_multiplier: 1.2,
            dependencies_multiplier: 1.0,
            version_consistency_multiplier: 0.8,
            upgrades_multiplier: 0.5,
            other_multiplier: 0.8,
        }
    }

    /// Gets the category multiplier for a given issue category.
    ///
    /// # Arguments
    ///
    /// * `category` - The issue category to get the multiplier for
    ///
    /// # Returns
    ///
    /// The multiplier value for the category
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{HealthScoreWeights, IssueCategory};
    ///
    /// let weights = HealthScoreWeights::default();
    /// let security_mult = weights.category_multiplier(IssueCategory::Security);
    /// assert_eq!(security_mult, 1.5);
    /// ```
    #[must_use]
    pub fn category_multiplier(&self, category: IssueCategory) -> f64 {
        match category {
            IssueCategory::Security => self.security_multiplier,
            IssueCategory::BreakingChanges => self.breaking_changes_multiplier,
            IssueCategory::Dependencies => self.dependencies_multiplier,
            IssueCategory::VersionConsistency => self.version_consistency_multiplier,
            IssueCategory::Upgrades => self.upgrades_multiplier,
            IssueCategory::Other => self.other_multiplier,
        }
    }

    /// Gets the severity weight for a given issue severity.
    ///
    /// # Arguments
    ///
    /// * `severity` - The issue severity to get the weight for
    ///
    /// # Returns
    ///
    /// The weight value for the severity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{HealthScoreWeights, IssueSeverity};
    ///
    /// let weights = HealthScoreWeights::default();
    /// let critical_weight = weights.severity_weight(IssueSeverity::Critical);
    /// assert_eq!(critical_weight, 15.0);
    /// ```
    #[must_use]
    pub fn severity_weight(&self, severity: IssueSeverity) -> f64 {
        match severity {
            IssueSeverity::Critical => self.critical_weight,
            IssueSeverity::Warning => self.warning_weight,
            IssueSeverity::Info => self.info_weight,
        }
    }
}

/// Detailed breakdown of health score calculation.
///
/// Provides transparency into how the score was calculated, including
/// deductions per category and severity.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::{calculate_health_score_detailed, HealthScoreWeights, AuditIssue};
///
/// let issues: Vec<AuditIssue> = vec![];
/// let weights = HealthScoreWeights::default();
/// let breakdown = calculate_health_score_detailed(&issues, &weights);
///
/// assert_eq!(breakdown.score, 100);
/// assert_eq!(breakdown.total_issues, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthScoreBreakdown {
    /// The final health score (0-100).
    pub score: u8,

    /// Total number of issues analyzed.
    pub total_issues: usize,

    /// Total points deducted.
    pub total_deduction: f64,

    /// Deductions grouped by severity.
    pub deductions_by_severity: HashMap<String, f64>,

    /// Deductions grouped by category.
    pub deductions_by_category: HashMap<String, f64>,

    /// Number of issues by severity.
    pub issues_by_severity: HashMap<String, usize>,

    /// Number of issues by category.
    pub issues_by_category: HashMap<String, usize>,
}

impl HealthScoreBreakdown {
    /// Returns a human-readable summary of the score breakdown.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{calculate_health_score_detailed, HealthScoreWeights, AuditIssue};
    ///
    /// let issues: Vec<AuditIssue> = vec![];
    /// let weights = HealthScoreWeights::default();
    /// let breakdown = calculate_health_score_detailed(&issues, &weights);
    /// let summary = breakdown.summary();
    ///
    /// assert!(summary.contains("Health Score: 100/100"));
    /// ```
    #[must_use]
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Health Score: {}/100", self.score),
            format!("Total Issues: {}", self.total_issues),
            format!("Total Deduction: {:.1} points", self.total_deduction),
            String::new(),
        ];

        if !self.issues_by_severity.is_empty() {
            lines.push("Issues by Severity:".to_string());
            for severity in ["critical", "warning", "info"] {
                if let Some(count) = self.issues_by_severity.get(severity)
                    && *count > 0 {
                        let deduction =
                            self.deductions_by_severity.get(severity).copied().unwrap_or(0.0);
                        lines.push(format!("  {}: {} (-{:.1} points)", severity, count, deduction));
                    }
            }
            lines.push(String::new());
        }

        if !self.issues_by_category.is_empty() {
            lines.push("Issues by Category:".to_string());
            let mut categories: Vec<_> = self.issues_by_category.iter().collect();
            categories.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
            for (category, count) in categories {
                if *count > 0 {
                    let deduction =
                        self.deductions_by_category.get(category.as_str()).copied().unwrap_or(0.0);
                    lines.push(format!("  {}: {} (-{:.1} points)", category, count, deduction));
                }
            }
        }

        lines.join("\n")
    }
}

/// Calculates the health score for a set of audit issues.
///
/// The score ranges from 0 (worst) to 100 (best), with deductions based on
/// the number, severity, and category of issues found.
///
/// # Arguments
///
/// * `issues` - Slice of audit issues to analyze
/// * `weights` - Weights to use for score calculation
///
/// # Returns
///
/// A health score from 0 to 100, where 100 is perfect health.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::{
///     calculate_health_score, HealthScoreWeights, AuditIssue,
///     IssueSeverity, IssueCategory
/// };
///
/// let issues = vec![
///     AuditIssue::new(
///         IssueSeverity::Warning,
///         IssueCategory::Upgrades,
///         "Update available".to_string(),
///         "Package has update".to_string(),
///     ),
/// ];
///
/// let score = calculate_health_score(&issues, &HealthScoreWeights::default());
/// assert!(score < 100);
/// assert!(score > 90);
/// ```
#[must_use]
pub fn calculate_health_score(issues: &[AuditIssue], weights: &HealthScoreWeights) -> u8 {
    let breakdown = calculate_health_score_detailed(issues, weights);
    breakdown.score
}

/// Calculates the health score with detailed breakdown.
///
/// Provides the same score as `calculate_health_score` but includes detailed
/// information about how the score was calculated.
///
/// # Arguments
///
/// * `issues` - Slice of audit issues to analyze
/// * `weights` - Weights to use for score calculation
///
/// # Returns
///
/// A detailed breakdown of the health score calculation.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::{
///     calculate_health_score_detailed, HealthScoreWeights, AuditIssue,
///     IssueSeverity, IssueCategory
/// };
///
/// let issues = vec![
///     AuditIssue::new(
///         IssueSeverity::Critical,
///         IssueCategory::Security,
///         "Vulnerability".to_string(),
///         "CVE found".to_string(),
///     ),
/// ];
///
/// let breakdown = calculate_health_score_detailed(&issues, &HealthScoreWeights::default());
/// assert_eq!(breakdown.total_issues, 1);
/// assert!(breakdown.score < 100);
/// assert!(breakdown.total_deduction > 0.0);
/// ```
#[must_use]
pub fn calculate_health_score_detailed(
    issues: &[AuditIssue],
    weights: &HealthScoreWeights,
) -> HealthScoreBreakdown {
    // Initialize tracking structures
    let mut deductions_by_severity: HashMap<String, f64> = HashMap::new();
    let mut deductions_by_category: HashMap<String, f64> = HashMap::new();
    let mut issues_by_severity: HashMap<String, usize> = HashMap::new();
    let mut issues_by_category: HashMap<String, usize> = HashMap::new();
    let mut category_counts: HashMap<IssueCategory, usize> = HashMap::new();

    let mut total_deduction = 0.0;

    // Process each issue
    for issue in issues {
        // Count issues
        let severity_key = issue.severity.as_str().to_string();
        let category_key = issue.category.as_str().to_string();

        *issues_by_severity.entry(severity_key.clone()).or_insert(0) += 1;
        *issues_by_category.entry(category_key.clone()).or_insert(0) += 1;

        // Calculate base deduction for this issue
        let base_weight = weights.severity_weight(issue.severity);
        let category_multiplier = weights.category_multiplier(issue.category);

        // Apply diminishing returns based on category count
        let category_count = category_counts.entry(issue.category).or_insert(0);
        let diminishing_factor = calculate_diminishing_factor(*category_count);
        *category_count += 1;

        // Calculate final deduction
        let deduction = base_weight * category_multiplier * diminishing_factor;

        // Track deductions
        *deductions_by_severity.entry(severity_key).or_insert(0.0) += deduction;
        *deductions_by_category.entry(category_key).or_insert(0.0) += deduction;
        total_deduction += deduction;
    }

    // Calculate final score (floor at 0, cap at 100)
    let raw_score = 100.0 - total_deduction;
    let score = raw_score.clamp(0.0, 100.0).round() as u8;

    HealthScoreBreakdown {
        score,
        total_issues: issues.len(),
        total_deduction,
        deductions_by_severity,
        deductions_by_category,
        issues_by_severity,
        issues_by_category,
    }
}

/// Calculates the diminishing returns factor based on issue count.
///
/// Each additional issue in the same category has reduced impact:
/// - 1st issue: 100% impact (1.0)
/// - 2nd issue: 90% impact (0.9)
/// - 3rd issue: 80% impact (0.8)
/// - 4th+ issues: 70% impact (0.7)
///
/// # Arguments
///
/// * `count` - The current count of issues in this category (0-indexed)
///
/// # Returns
///
/// A multiplier between 0.7 and 1.0
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::calculate_diminishing_factor;
///
/// assert_eq!(calculate_diminishing_factor(0), 1.0);  // First issue
/// assert_eq!(calculate_diminishing_factor(1), 0.9);  // Second issue
/// assert_eq!(calculate_diminishing_factor(2), 0.8);  // Third issue
/// assert_eq!(calculate_diminishing_factor(3), 0.7);  // Fourth+ issue
/// ```
#[must_use]
pub fn calculate_diminishing_factor(count: usize) -> f64 {
    match count {
        0 => 1.0,
        1 => 0.9,
        2 => 0.8,
        _ => 0.7,
    }
}
