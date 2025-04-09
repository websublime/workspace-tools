//! # Dependency Resolution Module
//!
//! This module provides structures for handling dependency resolution results.
//!
//! When resolving version conflicts or determining necessary updates,
//! `ResolutionResult` captures both the resolved versions for each package
//! and the specific updates that need to be applied.
use std::collections::HashMap;

use super::update::DependencyUpdate;

/// Result of a dependency resolution operation.
///
/// After analyzing dependencies and their version requirements,
/// this structure holds both:
/// - The resolved versions for each package name
/// - Specific updates needed to achieve consistency
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{ResolutionResult, DependencyUpdate};
/// use std::collections::HashMap;
///
/// // Create a resolution result
/// let mut resolved_versions = HashMap::new();
/// resolved_versions.insert("react".to_string(), "17.0.0".to_string());
/// resolved_versions.insert("lodash".to_string(), "4.17.21".to_string());
///
/// let updates = vec![
///     DependencyUpdate {
///         package_name: "my-app".to_string(),
///         dependency_name: "react".to_string(),
///         current_version: "^16.0.0".to_string(),
///         new_version: "^17.0.0".to_string(),
///     }
/// ];
///
/// let result = ResolutionResult {
///     resolved_versions,
///     updates_required: updates,
/// };
///
/// // Use the result
/// assert_eq!(result.resolved_versions.get("react").unwrap(), "17.0.0");
/// assert_eq!(result.updates_required.len(), 1);
/// ```
#[derive(Debug)]
pub struct ResolutionResult {
    /// Resolved versions for each package
    ///
    /// Maps package names to their resolved version strings.
    pub resolved_versions: HashMap<String, String>,
    /// Packages that need version updates
    ///
    /// Contains information about which specific packages need to be updated
    /// and what their new versions should be.
    pub updates_required: Vec<DependencyUpdate>,
}
