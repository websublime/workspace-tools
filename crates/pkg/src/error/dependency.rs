//! # Dependency error types and implementations
//!
//! ## What
//! This module provides error types specific to dependency management operations,
//! including circular dependency detection, resolution failures, and graph construction.
//!
//! ## How
//! Provides detailed error types for dependency-related failures with specific
//! context for dependency resolution, graph analysis, and propagation operations.
//!
//! ## Why
//! Dependency management is complex in monorepo environments and requires
//! precise error handling to provide clear feedback about circular dependencies,
//! resolution conflicts, and propagation issues.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for dependency operations.
///
/// This is a convenience type alias for Results with `DependencyError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{DependencyResult, DependencyError};
///
/// fn resolve_dependencies(package: &str) -> DependencyResult<Vec<String>> {
///     if package.is_empty() {
///         return Err(DependencyError::InvalidSpecification {
///             package: "unknown".to_string(),
///             spec: "".to_string(),
///             reason: "Empty package name".to_string(),
///         });
///     }
///     Ok(vec![])
/// }
/// ```
pub type DependencyResult<T> = StdResult<T, DependencyError>;

/// Dependency-related error types.
///
/// Handles errors in dependency analysis including circular dependencies,
/// resolution failures, and graph construction problems.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::DependencyError;
///
/// let error = DependencyError::CircularDependency {
///     cycle: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()],
/// };
///
/// println!("Error: {}", error);
/// // Output: Circular dependency detected: pkg-a -> pkg-b -> pkg-a
/// ```
#[derive(Error, Debug, Clone)]
pub enum DependencyError {
    /// Circular dependency detected
    #[error("Circular dependency detected: {}", cycle.join(" -> "))]
    CircularDependency {
        /// Dependency cycle path
        cycle: Vec<String>,
    },

    /// Dependency resolution failed
    #[error("Failed to resolve dependencies for package '{package}': {reason}")]
    ResolutionFailed {
        /// Package name
        package: String,
        /// Reason for resolution failure
        reason: String,
    },

    /// Missing dependency
    #[error("Missing dependency '{dependency}' for package '{package}'")]
    MissingDependency {
        /// Package that has missing dependency
        package: String,
        /// Missing dependency name
        dependency: String,
    },

    /// Invalid dependency specification
    #[error("Invalid dependency specification for package '{package}': '{spec}' - {reason}")]
    InvalidSpecification {
        /// Package name
        package: String,
        /// Invalid dependency specification
        spec: String,
        /// Reason why specification is invalid
        reason: String,
    },

    /// Dependency graph construction failed
    #[error("Failed to construct dependency graph: {reason}")]
    GraphConstructionFailed {
        /// Reason for graph construction failure
        reason: String,
    },

    /// Dependency propagation failed
    #[error("Failed to propagate dependency updates: {reason}")]
    PropagationFailed {
        /// Reason for propagation failure
        reason: String,
    },

    /// Maximum dependency depth exceeded
    #[error("Maximum dependency depth exceeded: {max_depth}")]
    MaxDepthExceeded {
        /// Maximum allowed depth
        max_depth: usize,
    },
}

impl DependencyError {
    /// Creates a circular dependency error.
    ///
    /// # Arguments
    ///
    /// * `cycle` - The dependency cycle path
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::circular_dependency(vec![
    ///     "pkg-a".to_string(),
    ///     "pkg-b".to_string(),
    ///     "pkg-a".to_string()
    /// ]);
    /// assert!(error.to_string().contains("Circular dependency"));
    /// ```
    #[must_use]
    pub fn circular_dependency(cycle: Vec<String>) -> Self {
        Self::CircularDependency { cycle }
    }

    /// Creates a resolution failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `reason` - Why resolution failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::resolution_failed(
    ///     "@myorg/my-package",
    ///     "Version conflict detected"
    /// );
    /// assert!(error.to_string().contains("Failed to resolve dependencies"));
    /// ```
    #[must_use]
    pub fn resolution_failed(package: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ResolutionFailed { package: package.into(), reason: reason.into() }
    }

    /// Creates a missing dependency error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package that has missing dependency
    /// * `dependency` - Missing dependency name
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::missing_dependency(
    ///     "@myorg/consumer",
    ///     "@myorg/missing-dep"
    /// );
    /// assert!(error.to_string().contains("Missing dependency"));
    /// ```
    #[must_use]
    pub fn missing_dependency(package: impl Into<String>, dependency: impl Into<String>) -> Self {
        Self::MissingDependency { package: package.into(), dependency: dependency.into() }
    }

    /// Creates an invalid specification error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `spec` - Invalid dependency specification
    /// * `reason` - Why specification is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::invalid_specification(
    ///     "@myorg/my-package",
    ///     "invalid-version-range",
    ///     "Malformed semver range"
    /// );
    /// assert!(error.to_string().contains("Invalid dependency specification"));
    /// ```
    #[must_use]
    pub fn invalid_specification(
        package: impl Into<String>,
        spec: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidSpecification {
            package: package.into(),
            spec: spec.into(),
            reason: reason.into(),
        }
    }

    /// Creates a graph construction failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why graph construction failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::graph_construction_failed("Duplicate package names detected");
    /// assert!(error.to_string().contains("Failed to construct dependency graph"));
    /// ```
    #[must_use]
    pub fn graph_construction_failed(reason: impl Into<String>) -> Self {
        Self::GraphConstructionFailed { reason: reason.into() }
    }

    /// Creates a propagation failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why propagation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::propagation_failed("Circular propagation detected");
    /// assert!(error.to_string().contains("Failed to propagate dependency updates"));
    /// ```
    #[must_use]
    pub fn propagation_failed(reason: impl Into<String>) -> Self {
        Self::PropagationFailed { reason: reason.into() }
    }

    /// Creates a max depth exceeded error.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum allowed depth
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::max_depth_exceeded(100);
    /// assert!(error.to_string().contains("Maximum dependency depth exceeded"));
    /// ```
    #[must_use]
    pub fn max_depth_exceeded(max_depth: usize) -> Self {
        Self::MaxDepthExceeded { max_depth }
    }

    /// Checks if this is a circular dependency error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::circular_dependency(vec!["a".to_string(), "b".to_string()]);
    /// assert!(error.is_circular_dependency());
    ///
    /// let error = DependencyError::resolution_failed("pkg", "reason");
    /// assert!(!error.is_circular_dependency());
    /// ```
    #[must_use]
    pub fn is_circular_dependency(&self) -> bool {
        matches!(self, Self::CircularDependency { .. })
    }

    /// Checks if this is a resolution error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::resolution_failed("pkg", "reason");
    /// assert!(error.is_resolution_error());
    ///
    /// let error = DependencyError::missing_dependency("pkg", "dep");
    /// assert!(error.is_resolution_error());
    /// ```
    #[must_use]
    pub fn is_resolution_error(&self) -> bool {
        matches!(
            self,
            Self::ResolutionFailed { .. }
                | Self::MissingDependency { .. }
                | Self::InvalidSpecification { .. }
        )
    }

    /// Checks if this is a graph-related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::graph_construction_failed("reason");
    /// assert!(error.is_graph_error());
    ///
    /// let error = DependencyError::circular_dependency(vec![]);
    /// assert!(error.is_graph_error());
    /// ```
    #[must_use]
    pub fn is_graph_error(&self) -> bool {
        matches!(
            self,
            Self::GraphConstructionFailed { .. }
                | Self::CircularDependency { .. }
                | Self::MaxDepthExceeded { .. }
        )
    }

    /// Gets the package name from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let error = DependencyError::resolution_failed("my-package", "reason");
    /// assert_eq!(error.package_name(), Some("my-package"));
    ///
    /// let error = DependencyError::graph_construction_failed("reason");
    /// assert_eq!(error.package_name(), None);
    /// ```
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::ResolutionFailed { package, .. }
            | Self::MissingDependency { package, .. }
            | Self::InvalidSpecification { package, .. } => Some(package),
            Self::CircularDependency { .. }
            | Self::GraphConstructionFailed { .. }
            | Self::PropagationFailed { .. }
            | Self::MaxDepthExceeded { .. } => None,
        }
    }

    /// Gets the dependency cycle from circular dependency errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::DependencyError;
    ///
    /// let cycle = vec!["a".to_string(), "b".to_string(), "a".to_string()];
    /// let error = DependencyError::circular_dependency(cycle.clone());
    /// assert_eq!(error.dependency_cycle(), Some(&cycle));
    ///
    /// let error = DependencyError::resolution_failed("pkg", "reason");
    /// assert_eq!(error.dependency_cycle(), None);
    /// ```
    #[must_use]
    pub fn dependency_cycle(&self) -> Option<&Vec<String>> {
        match self {
            Self::CircularDependency { cycle } => Some(cycle),
            _ => None,
        }
    }
}

impl AsRef<str> for DependencyError {
    fn as_ref(&self) -> &str {
        match self {
            DependencyError::CircularDependency { .. } => "DependencyError::CircularDependency",
            DependencyError::ResolutionFailed { .. } => "DependencyError::ResolutionFailed",
            DependencyError::MissingDependency { .. } => "DependencyError::MissingDependency",
            DependencyError::InvalidSpecification { .. } => "DependencyError::InvalidSpecification",
            DependencyError::GraphConstructionFailed { .. } => {
                "DependencyError::GraphConstructionFailed"
            }
            DependencyError::PropagationFailed { .. } => "DependencyError::PropagationFailed",
            DependencyError::MaxDepthExceeded { .. } => "DependencyError::MaxDepthExceeded",
        }
    }
}
