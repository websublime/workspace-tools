//! # Release error types and implementations
//!
//! ## What
//! This module provides error types specific to release management operations,
//! including release planning, execution, package releases, and rollback operations.
//!
//! ## How
//! Provides detailed error types for release-related failures with specific
//! context for different stages of the release lifecycle and strategy execution.
//!
//! ## Why
//! Release management is a critical operation that requires precise error
//! handling to provide clear feedback about planning failures, execution
//! problems, and rollback issues across multiple environments.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for release operations.
///
/// This is a convenience type alias for Results with `ReleaseError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{ReleaseResult, ReleaseError};
///
/// fn execute_release(environment: &str) -> ReleaseResult<()> {
///     if environment.is_empty() {
///         return Err(ReleaseError::ExecutionFailed {
///             environment: "unknown".to_string(),
///             reason: "Empty environment name".to_string(),
///         });
///     }
///     Ok(())
/// }
/// ```
pub type ReleaseResult<T> = StdResult<T, ReleaseError>;

/// Release-related error types.
///
/// Handles errors in release management including planning, execution,
/// package releases, and strategy validation.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::ReleaseError;
///
/// let error = ReleaseError::PlanningFailed {
///     reason: "No changesets found".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Release planning failed: No changesets found
/// ```
#[derive(Error, Debug, Clone)]
pub enum ReleaseError {
    /// Release planning failed
    #[error("Release planning failed: {reason}")]
    PlanningFailed {
        /// Reason for planning failure
        reason: String,
    },

    /// Release execution failed
    #[error("Release execution failed for environment '{environment}': {reason}")]
    ExecutionFailed {
        /// Target environment
        environment: String,
        /// Reason for execution failure
        reason: String,
    },

    /// Package release failed
    #[error("Failed to release package '{package}' to environment '{environment}': {reason}")]
    PackageReleaseFailed {
        /// Package name
        package: String,
        /// Target environment
        environment: String,
        /// Reason for package release failure
        reason: String,
    },

    /// Tag creation failed
    #[error("Failed to create tag '{tag}': {reason}")]
    TagCreationFailed {
        /// Tag name
        tag: String,
        /// Reason for tag creation failure
        reason: String,
    },

    /// Dry run failed
    #[error("Dry run failed: {reason}")]
    DryRunFailed {
        /// Reason for dry run failure
        reason: String,
    },

    /// Release strategy not supported
    #[error("Release strategy '{strategy}' is not supported")]
    StrategyNotSupported {
        /// Unsupported strategy name
        strategy: String,
    },

    /// Rollback failed
    #[error("Rollback failed: {reason}")]
    RollbackFailed {
        /// Reason for rollback failure
        reason: String,
    },
}

impl ReleaseError {
    /// Creates a planning failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why planning failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::planning_failed("No packages to release");
    /// assert!(error.to_string().contains("Release planning failed"));
    /// ```
    #[must_use]
    pub fn planning_failed(reason: impl Into<String>) -> Self {
        Self::PlanningFailed { reason: reason.into() }
    }

    /// Creates an execution failed error.
    ///
    /// # Arguments
    ///
    /// * `environment` - Target environment
    /// * `reason` - Why execution failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::execution_failed(
    ///     "production",
    ///     "Network timeout"
    /// );
    /// assert!(error.to_string().contains("Release execution failed"));
    /// ```
    #[must_use]
    pub fn execution_failed(environment: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ExecutionFailed { environment: environment.into(), reason: reason.into() }
    }

    /// Creates a package release failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `environment` - Target environment
    /// * `reason` - Why package release failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::package_release_failed(
    ///     "@myorg/my-package",
    ///     "staging",
    ///     "Build failed"
    /// );
    /// assert!(error.to_string().contains("Failed to release package"));
    /// ```
    #[must_use]
    pub fn package_release_failed(
        package: impl Into<String>,
        environment: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::PackageReleaseFailed {
            package: package.into(),
            environment: environment.into(),
            reason: reason.into(),
        }
    }

    /// Creates a tag creation failed error.
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag name
    /// * `reason` - Why tag creation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::tag_creation_failed(
    ///     "v1.0.0",
    ///     "Tag already exists"
    /// );
    /// assert!(error.to_string().contains("Failed to create tag"));
    /// ```
    #[must_use]
    pub fn tag_creation_failed(tag: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::TagCreationFailed { tag: tag.into(), reason: reason.into() }
    }

    /// Creates a dry run failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why dry run failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::dry_run_failed("Invalid configuration");
    /// assert!(error.to_string().contains("Dry run failed"));
    /// ```
    #[must_use]
    pub fn dry_run_failed(reason: impl Into<String>) -> Self {
        Self::DryRunFailed { reason: reason.into() }
    }

    /// Creates a strategy not supported error.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Unsupported strategy name
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::strategy_not_supported("custom-strategy");
    /// assert!(error.to_string().contains("not supported"));
    /// ```
    #[must_use]
    pub fn strategy_not_supported(strategy: impl Into<String>) -> Self {
        Self::StrategyNotSupported { strategy: strategy.into() }
    }

    /// Creates a rollback failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why rollback failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::rollback_failed("Snapshot not found");
    /// assert!(error.to_string().contains("Rollback failed"));
    /// ```
    #[must_use]
    pub fn rollback_failed(reason: impl Into<String>) -> Self {
        Self::RollbackFailed { reason: reason.into() }
    }

    /// Checks if this is a planning error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::planning_failed("reason");
    /// assert!(error.is_planning_error());
    ///
    /// let error = ReleaseError::execution_failed("env", "reason");
    /// assert!(!error.is_planning_error());
    /// ```
    #[must_use]
    pub fn is_planning_error(&self) -> bool {
        matches!(self, Self::PlanningFailed { .. } | Self::DryRunFailed { .. })
    }

    /// Checks if this is an execution error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::execution_failed("env", "reason");
    /// assert!(error.is_execution_error());
    ///
    /// let error = ReleaseError::package_release_failed("pkg", "env", "reason");
    /// assert!(error.is_execution_error());
    /// ```
    #[must_use]
    pub fn is_execution_error(&self) -> bool {
        matches!(
            self,
            Self::ExecutionFailed { .. }
                | Self::PackageReleaseFailed { .. }
                | Self::TagCreationFailed { .. }
        )
    }

    /// Checks if this is a configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::strategy_not_supported("strategy");
    /// assert!(error.is_configuration_error());
    ///
    /// let error = ReleaseError::execution_failed("env", "reason");
    /// assert!(!error.is_configuration_error());
    /// ```
    #[must_use]
    pub fn is_configuration_error(&self) -> bool {
        matches!(self, Self::StrategyNotSupported { .. })
    }

    /// Gets the environment name from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::execution_failed("production", "reason");
    /// assert_eq!(error.environment(), Some("production"));
    ///
    /// let error = ReleaseError::planning_failed("reason");
    /// assert_eq!(error.environment(), None);
    /// ```
    #[must_use]
    pub fn environment(&self) -> Option<&str> {
        match self {
            Self::ExecutionFailed { environment, .. }
            | Self::PackageReleaseFailed { environment, .. } => Some(environment),
            Self::PlanningFailed { .. }
            | Self::TagCreationFailed { .. }
            | Self::DryRunFailed { .. }
            | Self::StrategyNotSupported { .. }
            | Self::RollbackFailed { .. } => None,
        }
    }

    /// Gets the package name from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::package_release_failed("my-pkg", "env", "reason");
    /// assert_eq!(error.package_name(), Some("my-pkg"));
    ///
    /// let error = ReleaseError::execution_failed("env", "reason");
    /// assert_eq!(error.package_name(), None);
    /// ```
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::PackageReleaseFailed { package, .. } => Some(package),
            Self::PlanningFailed { .. }
            | Self::ExecutionFailed { .. }
            | Self::TagCreationFailed { .. }
            | Self::DryRunFailed { .. }
            | Self::StrategyNotSupported { .. }
            | Self::RollbackFailed { .. } => None,
        }
    }

    /// Gets the tag name from tag creation errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ReleaseError;
    ///
    /// let error = ReleaseError::tag_creation_failed("v1.0.0", "reason");
    /// assert_eq!(error.tag_name(), Some("v1.0.0"));
    ///
    /// let error = ReleaseError::planning_failed("reason");
    /// assert_eq!(error.tag_name(), None);
    /// ```
    #[must_use]
    pub fn tag_name(&self) -> Option<&str> {
        match self {
            Self::TagCreationFailed { tag, .. } => Some(tag),
            _ => None,
        }
    }
}

impl AsRef<str> for ReleaseError {
    fn as_ref(&self) -> &str {
        match self {
            ReleaseError::PlanningFailed { .. } => "ReleaseError::PlanningFailed",
            ReleaseError::ExecutionFailed { .. } => "ReleaseError::ExecutionFailed",
            ReleaseError::PackageReleaseFailed { .. } => "ReleaseError::PackageReleaseFailed",
            ReleaseError::TagCreationFailed { .. } => "ReleaseError::TagCreationFailed",
            ReleaseError::DryRunFailed { .. } => "ReleaseError::DryRunFailed",
            ReleaseError::StrategyNotSupported { .. } => "ReleaseError::StrategyNotSupported",
            ReleaseError::RollbackFailed { .. } => "ReleaseError::RollbackFailed",
        }
    }
}
