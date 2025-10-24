//! Audit configuration for health checks and dependency audits.
//!
//! **What**: Defines configuration for audit operations, including which audit sections
//! to run, severity thresholds, specific checks for each audit type, and health score weights.
//!
//! **How**: This module provides the `AuditConfig` structure that controls how dependency
//! audits, health checks, and issue detection are performed, along with customizable weights
//! for health score calculation.
//!
//! **Why**: To enable comprehensive project health monitoring with configurable checks,
//! severity levels, and scoring weights that can be tailored to project needs and deployment
//! environments.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for audit and health check operations.
///
/// This structure controls all aspects of project auditing, including which
/// sections to audit, minimum severity levels, and specific check configurations.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::AuditConfig;
///
/// let config = AuditConfig::default();
/// assert!(config.enabled);
/// assert_eq!(config.min_severity, "warning");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditConfig {
    /// Whether auditing is enabled.
    ///
    /// # Default: `true`
    pub enabled: bool,

    /// Minimum severity level for reporting issues.
    ///
    /// Valid values: "critical", "warning", "info"
    ///
    /// # Default: `"warning"`
    pub min_severity: String,

    /// Configuration for which audit sections to run.
    pub sections: AuditSectionsConfig,

    /// Configuration for upgrade audits.
    pub upgrades: UpgradeAuditConfig,

    /// Configuration for dependency audits.
    pub dependencies: DependencyAuditConfig,

    /// Configuration for breaking changes audits.
    pub breaking_changes: BreakingChangesAuditConfig,

    /// Configuration for version consistency audits.
    pub version_consistency: VersionConsistencyAuditConfig,

    /// Configuration for health score calculation weights.
    pub health_score_weights: HealthScoreWeightsConfig,
}

/// Configuration for which audit sections to execute.
///
/// Each section can be independently enabled or disabled.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::AuditSectionsConfig;
///
/// let config = AuditSectionsConfig::default();
/// assert!(config.upgrades);
/// assert!(config.dependencies);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditSectionsConfig {
    /// Whether to run upgrade availability audits.
    ///
    /// # Default: `true`
    pub upgrades: bool,

    /// Whether to run dependency health audits.
    ///
    /// # Default: `true`
    pub dependencies: bool,

    /// Whether to check for breaking changes.
    ///
    /// # Default: `true`
    pub breaking_changes: bool,

    /// Whether to categorize dependencies.
    ///
    /// # Default: `true`
    pub categorization: bool,

    /// Whether to check version consistency.
    ///
    /// # Default: `true`
    pub version_consistency: bool,
}

/// Configuration for upgrade audits.
///
/// Controls which types of upgrades to include in audit reports.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::UpgradeAuditConfig;
///
/// let config = UpgradeAuditConfig::default();
/// assert!(config.include_major);
/// assert!(config.deprecated_as_critical);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpgradeAuditConfig {
    /// Whether to include patch version upgrades.
    ///
    /// # Default: `true`
    pub include_patch: bool,

    /// Whether to include minor version upgrades.
    ///
    /// # Default: `true`
    pub include_minor: bool,

    /// Whether to include major version upgrades.
    ///
    /// # Default: `true`
    pub include_major: bool,

    /// Whether to treat deprecated packages as critical issues.
    ///
    /// # Default: `true`
    pub deprecated_as_critical: bool,
}

/// Configuration for dependency audits.
///
/// Controls which dependency checks to perform.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::DependencyAuditConfig;
///
/// let config = DependencyAuditConfig::default();
/// assert!(config.check_circular);
/// assert!(config.check_version_conflicts);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyAuditConfig {
    /// Whether to detect circular dependencies.
    ///
    /// # Default: `true`
    pub check_circular: bool,

    /// Whether to check for missing dependencies.
    ///
    /// # Default: `false`
    pub check_missing: bool,

    /// Whether to check for unused dependencies.
    ///
    /// # Default: `false`
    pub check_unused: bool,

    /// Whether to check for version conflicts.
    ///
    /// # Default: `true`
    pub check_version_conflicts: bool,
}

/// Configuration for breaking changes audits.
///
/// Controls how breaking changes are detected.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::BreakingChangesAuditConfig;
///
/// let config = BreakingChangesAuditConfig::default();
/// assert!(config.check_conventional_commits);
/// assert!(config.check_changelog);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreakingChangesAuditConfig {
    /// Whether to check for breaking changes in conventional commits.
    ///
    /// # Default: `true`
    pub check_conventional_commits: bool,

    /// Whether to check for breaking changes in changelogs.
    ///
    /// # Default: `true`
    pub check_changelog: bool,
}

/// Configuration for version consistency audits.
///
/// Controls how version inconsistencies are handled.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::VersionConsistencyAuditConfig;
///
/// let config = VersionConsistencyAuditConfig::default();
/// assert!(!config.fail_on_inconsistency);
/// assert!(config.warn_on_inconsistency);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionConsistencyAuditConfig {
    /// Whether to fail when version inconsistencies are detected.
    ///
    /// # Default: `false`
    pub fail_on_inconsistency: bool,

    /// Whether to warn when version inconsistencies are detected.
    ///
    /// # Default: `true`
    pub warn_on_inconsistency: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: "warning".to_string(),
            sections: AuditSectionsConfig::default(),
            upgrades: UpgradeAuditConfig::default(),
            dependencies: DependencyAuditConfig::default(),
            breaking_changes: BreakingChangesAuditConfig::default(),
            version_consistency: VersionConsistencyAuditConfig::default(),
            health_score_weights: HealthScoreWeightsConfig::default(),
        }
    }
}

impl Default for AuditSectionsConfig {
    fn default() -> Self {
        Self {
            upgrades: true,
            dependencies: true,
            breaking_changes: true,
            categorization: true,
            version_consistency: true,
        }
    }
}

impl Default for UpgradeAuditConfig {
    fn default() -> Self {
        Self {
            include_patch: true,
            include_minor: true,
            include_major: true,
            deprecated_as_critical: true,
        }
    }
}

impl Default for DependencyAuditConfig {
    fn default() -> Self {
        Self {
            check_circular: true,
            check_missing: false,
            check_unused: false,
            check_version_conflicts: true,
        }
    }
}

impl Default for BreakingChangesAuditConfig {
    fn default() -> Self {
        Self { check_conventional_commits: true, check_changelog: true }
    }
}

impl Default for VersionConsistencyAuditConfig {
    fn default() -> Self {
        Self { fail_on_inconsistency: false, warn_on_inconsistency: true }
    }
}

/// Configuration for health score calculation weights.
///
/// These weights control how much each type of issue affects the overall health score.
/// All weights should be positive numbers where higher values mean more impact.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::HealthScoreWeightsConfig;
///
/// let config = HealthScoreWeightsConfig::default();
/// assert_eq!(config.critical_weight, 15.0);
/// assert_eq!(config.security_multiplier, 1.5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthScoreWeightsConfig {
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

impl Default for HealthScoreWeightsConfig {
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

impl Configurable for AuditConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate min_severity
        match self.min_severity.as_str() {
            "critical" | "warning" | "info" => {}
            _ => {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "audit.min_severity: Invalid severity '{}'. Must be one of: critical, warning, info",
                        self.min_severity
                    ),
                });
            }
        }

        self.sections.validate()?;
        self.upgrades.validate()?;
        self.dependencies.validate()?;
        self.breaking_changes.validate()?;
        self.version_consistency.validate()?;
        self.health_score_weights.validate()?;

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enabled = other.enabled;
        self.min_severity = other.min_severity;
        self.sections.merge_with(other.sections)?;
        self.upgrades.merge_with(other.upgrades)?;
        self.dependencies.merge_with(other.dependencies)?;
        self.breaking_changes.merge_with(other.breaking_changes)?;
        self.version_consistency.merge_with(other.version_consistency)?;
        self.health_score_weights.merge_with(other.health_score_weights)?;
        Ok(())
    }
}

impl Configurable for AuditSectionsConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.upgrades = other.upgrades;
        self.dependencies = other.dependencies;
        self.breaking_changes = other.breaking_changes;
        self.categorization = other.categorization;
        self.version_consistency = other.version_consistency;
        Ok(())
    }
}

impl Configurable for UpgradeAuditConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.include_patch = other.include_patch;
        self.include_minor = other.include_minor;
        self.include_major = other.include_major;
        self.deprecated_as_critical = other.deprecated_as_critical;
        Ok(())
    }
}

impl Configurable for DependencyAuditConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.check_circular = other.check_circular;
        self.check_missing = other.check_missing;
        self.check_unused = other.check_unused;
        self.check_version_conflicts = other.check_version_conflicts;
        Ok(())
    }
}

impl Configurable for BreakingChangesAuditConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.check_conventional_commits = other.check_conventional_commits;
        self.check_changelog = other.check_changelog;
        Ok(())
    }
}

impl Configurable for VersionConsistencyAuditConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.fail_on_inconsistency = other.fail_on_inconsistency;
        self.warn_on_inconsistency = other.warn_on_inconsistency;
        Ok(())
    }
}

impl Configurable for HealthScoreWeightsConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate that all weights are positive
        let weights = [
            ("critical_weight", self.critical_weight),
            ("warning_weight", self.warning_weight),
            ("info_weight", self.info_weight),
            ("security_multiplier", self.security_multiplier),
            ("breaking_changes_multiplier", self.breaking_changes_multiplier),
            ("dependencies_multiplier", self.dependencies_multiplier),
            ("version_consistency_multiplier", self.version_consistency_multiplier),
            ("upgrades_multiplier", self.upgrades_multiplier),
            ("other_multiplier", self.other_multiplier),
        ];

        for (name, value) in &weights {
            if *value < 0.0 {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "audit.health_score_weights.{}: Must be non-negative, got {}",
                        name, value
                    ),
                });
            }
        }

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.critical_weight = other.critical_weight;
        self.warning_weight = other.warning_weight;
        self.info_weight = other.info_weight;
        self.security_multiplier = other.security_multiplier;
        self.breaking_changes_multiplier = other.breaking_changes_multiplier;
        self.dependencies_multiplier = other.dependencies_multiplier;
        self.version_consistency_multiplier = other.version_consistency_multiplier;
        self.upgrades_multiplier = other.upgrades_multiplier;
        self.other_multiplier = other.other_multiplier;
        Ok(())
    }
}
