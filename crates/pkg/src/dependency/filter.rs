//! # Dependency Filter Module
//!
//! This module provides enumeration types for filtering and categorizing dependencies.
//!
//! In Node.js package ecosystems, dependencies can be categorized as:
//! - Production dependencies (regular dependencies)
//! - Development dependencies (devDependencies)
//! - Optional dependencies (optionalDependencies)
//!
//! The `DependencyFilter` enum allows configuring which types of dependencies
//! to include in various operations throughout the library.

/// Filter to control which types of dependencies are included in operations.
///
/// In Node.js packages, dependencies can be categorized in different sections
/// of package.json (dependencies, devDependencies, optionalDependencies).
/// This enum allows configuring which types to include.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{DependencyFilter, UpgradeConfig};
///
/// // Create a configuration that only includes production dependencies
/// let config = UpgradeConfig {
///     dependency_types: DependencyFilter::ProductionOnly,
///     ..UpgradeConfig::default()
/// };
///
/// // Use default (includes production and development dependencies)
/// let default_filter = DependencyFilter::default();
/// assert!(matches!(default_filter, DependencyFilter::WithDevelopment));
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DependencyFilter {
    /// Include only production dependencies
    ///
    /// Only includes dependencies from the main "dependencies" section of package.json.
    ProductionOnly,
    /// Include production and development dependencies
    ///
    /// Includes dependencies from both "dependencies" and "devDependencies" sections.
    WithDevelopment,
    /// Include production, development, and optional dependencies
    ///
    /// Includes all dependency types: "dependencies", "devDependencies", and "optionalDependencies".
    AllDependencies,
}

impl Default for DependencyFilter {
    /// Returns the default filter configuration.
    ///
    /// By default, both production and development dependencies are included.
    fn default() -> Self {
        Self::WithDevelopment
    }
}
