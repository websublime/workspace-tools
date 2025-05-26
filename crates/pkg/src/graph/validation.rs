//! # Dependency Graph Validation Module
//!
//! This module provides functionality for validating dependency graphs and reporting issues.
//!
//! The main structures are `ValidationIssue`, which represents different types of
//! dependency issues (circular dependencies, unresolved dependencies, version conflicts),
//! and `ValidationReport`, which collects these issues.
//!
//! ## Key Features
//!
//! - Detection of circular dependencies
//! - Identification of unresolved dependencies
//! - Discovery of version conflicts
//! - Customizable validation with `ValidationOptions`
//! - Classification of issues as critical or warnings
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{build_dependency_graph_from_packages, ValidationOptions};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let packages = vec![];
//! // Build a dependency graph
//! let graph = build_dependency_graph_from_packages(&packages);
//!
//! // Create custom validation options
//! let options = ValidationOptions::new()
//!     .treat_unresolved_as_external(true)
//!     .with_internal_packages(vec!["@company/ui", "@company/core"]);
//!
//! // Validate with custom options
//! let report = graph.validate_with_options(&options)?;
//!
//! if report.has_critical_issues() {
//!     println!("Found critical issues:");
//!     for issue in report.critical_issues() {
//!         println!("  - {}", issue.message());
//!     }
//! }
//!
//! if report.has_warnings() {
//!     println!("Found warnings:");
//!     for warning in report.warnings() {
//!         println!("  - {}", warning.message());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#[derive(Debug)]
pub enum ValidationIssue {
    /// Circular dependency detected - now just an issue, not a blocker
    CircularDependency { path: Vec<String> },

    /// Unresolved dependency
    UnresolvedDependency { name: String, version_req: String },

    /// Version conflict
    VersionConflict { name: String, versions: Vec<String> },
}

impl ValidationIssue {
    /// Returns true if this is a critical issue that should be fixed
    pub fn is_critical(&self) -> bool {
        match self {
            // Circular dependencies are now marked as warnings, not critical errors
            Self::UnresolvedDependency { .. } => true,
            Self::VersionConflict { .. } | Self::CircularDependency { .. } => false, // Consider version conflicts as warnings
        }
    }

    /// Returns a descriptive message for this issue
    pub fn message(&self) -> String {
        match self {
            Self::CircularDependency { path } => {
                format!("Circular dependency detected: {}", path.join(" -> "))
            }
            Self::UnresolvedDependency { name, version_req } => {
                format!("Unresolved dependency: {name} {version_req}")
            }
            Self::VersionConflict { name, versions } => {
                format!("Version conflict for {}: {}", name, versions.join(", "))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(ValidationIssue::is_critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| !issue.is_critical())
    }

    pub fn critical_issues(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_critical()).collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| !issue.is_critical()).collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// If true, unresolved dependencies are treated as external and not flagged as errors
    pub treat_unresolved_as_external: bool,

    /// Optional list of specific packages to consider internal (only used when treat_unresolved_as_external is true)
    /// Any unresolved dependency in this list will still be flagged as an error
    pub internal_packages: Vec<String>,
}

impl ValidationOptions {
    /// Create new validation options with default settings (flag all unresolved dependencies)
    pub fn new() -> Self {
        Self::default()
    }

    /// Treat unresolved dependencies as external (don't flag them as errors)
    #[must_use]
    pub fn treat_unresolved_as_external(mut self, value: bool) -> Self {
        self.treat_unresolved_as_external = value;
        self
    }

    /// Set list of packages that should be considered internal
    #[must_use]
    pub fn with_internal_packages<I, S>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.internal_packages = packages.into_iter().map(Into::into).collect();
        self
    }

    /// Check if a dependency should be treated as internal
    pub fn is_internal_dependency(&self, name: &str) -> bool {
        self.internal_packages.iter().any(|pkg| pkg == name)
    }
}
