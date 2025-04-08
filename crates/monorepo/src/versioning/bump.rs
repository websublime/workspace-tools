//! Functionality for applying version bumps to packages in a workspace.

use std::collections::HashMap;

use log::{debug, info};

use crate::{
    suggest_version_bumps, ChangeTracker, PackageVersionChange, VersionBumpPreview,
    VersionBumpStrategy, VersionSuggestion, VersioningError, VersioningResult, Workspace,
};

/// Manages version operations across a workspace.
pub struct VersionManager<'a> {
    /// Reference to the workspace
    workspace: &'a Workspace,
    /// Optional reference to the change tracker
    change_tracker: Option<&'a ChangeTracker>,
}

impl<'a> VersionManager<'a> {
    /// Create a new version manager.
    pub fn new(workspace: &'a Workspace, change_tracker: Option<&'a ChangeTracker>) -> Self {
        Self { workspace, change_tracker }
    }

    /// Suggest version bumps based on changes.
    pub fn suggest_bumps(
        &self,
        strategy: &VersionBumpStrategy,
    ) -> VersioningResult<HashMap<String, VersionSuggestion>> {
        // Ensure we have a change tracker
        let change_tracker = self.change_tracker.ok_or_else(|| {
            VersioningError::InvalidBumpStrategy(
                "Change tracker required for version suggestions".to_string(),
            )
        })?;

        // Use the suggest_version_bumps function
        suggest_version_bumps(self.workspace, change_tracker, strategy)
    }

    /// Preview version bumps without applying.
    pub fn preview_bumps(
        &self,
        strategy: &VersionBumpStrategy,
    ) -> VersioningResult<VersionBumpPreview> {
        // Get suggested bumps
        let suggestions = self.suggest_bumps(strategy)?;

        // Check for dependency cycles that might prevent proper versioning
        let dependency_analysis = self.workspace.analyze_dependencies()?;
        let cycle_detected = dependency_analysis.cycles_detected;

        if cycle_detected && matches!(strategy, VersionBumpStrategy::Synchronized { .. }) {
            info!(
                "Cycle detected in dependencies. Synchronized versioning may not work correctly."
            );

            // Add more detailed info about the cycles
            let sorted_with_cycles = self.workspace.get_sorted_packages_with_circulars();
            if !sorted_with_cycles.circular.is_empty() {
                info!("Circular dependency groups found:");
                for (i, group) in sorted_with_cycles.circular.iter().enumerate() {
                    let names: Vec<String> = group
                        .iter()
                        .map(|p| p.borrow().package.borrow().name().to_string())
                        .collect();
                    info!("  Group {}: {}", i + 1, names.join(" → "));
                }
            }
        }

        Ok(VersionBumpPreview { changes: suggestions.into_values().collect(), cycle_detected })
    }

    /// Apply version bumps.
    ///
    /// Note: If `mark_as_released` is true, you must separately call
    /// `mark_changes_as_released` on the change tracker with the results
    /// of this function, as the change tracker is not mutable here.
    pub fn apply_bumps(
        &self,
        strategy: &VersionBumpStrategy,
        dry_run: bool,
    ) -> VersioningResult<Vec<PackageVersionChange>> {
        let suggestions = self.suggest_bumps(strategy)?;

        if suggestions.is_empty() {
            debug!("No version bumps to apply");
            return Ok(Vec::new());
        }

        let mut changes = Vec::new();

        // Apply each version change
        for (package_name, suggestion) in suggestions {
            let package_info = self
                .workspace
                .get_package(&package_name)
                .ok_or_else(|| VersioningError::PackageNotFound(package_name.clone()))?;

            let current_version = package_info.borrow().package.borrow().version_str();
            let new_version = suggestion.suggested_version;

            // Track if this is a dependency-only update
            let is_dependency_update = suggestion
                .reasons
                .iter()
                .all(|reason| matches!(reason, crate::BumpReason::DependencyUpdate(_)));

            // Create the change record
            let version_change = PackageVersionChange {
                package_name: package_name.clone(),
                previous_version: current_version.clone(),
                new_version: new_version.clone(),
                bump_type: suggestion.bump_type,
                is_dependency_update,
            };

            // Actually apply the change if not a dry run
            if !dry_run {
                // Update the version in the package.json
                package_info.borrow().update_version(&new_version)?;
            }

            changes.push(version_change);
        }

        // Write changes to disk if not a dry run
        if !dry_run {
            self.workspace.write_changes()?;
        }

        Ok(changes)
    }

    /// Mark changes as released in the change tracker.
    ///
    /// This should be called after `apply_bumps` if you want to mark changes as released.
    pub fn mark_changes_as_released(
        change_tracker: &mut ChangeTracker,
        version_changes: &[PackageVersionChange],
        dry_run: bool,
    ) -> VersioningResult<()> {
        for change in version_changes {
            change_tracker.mark_released(&change.package_name, &change.new_version, dry_run)?;
        }
        Ok(())
    }

    /// Validate version consistency in the workspace.
    pub fn validate_versions(&self) -> VersioningResult<VersionValidation> {
        // First, check for cycles in the dependency graph
        let dependency_analysis = self.workspace.analyze_dependencies()?;

        // Get all packages
        let packages = self.workspace.sorted_packages();

        // Map to store dependency consistency issues
        let mut inconsistencies = Vec::new();

        // Check that each internal dependency references the correct version
        for pkg_info in &packages {
            let pkg = pkg_info.borrow();
            let package = pkg.package.borrow();
            let pkg_name = package.name().to_string();

            // Check each dependency
            for dep in package.dependencies() {
                let dep_name = dep.borrow().name().to_string();

                // Skip external dependencies
                if let Some(dep_pkg) = self.workspace.get_package(&dep_name) {
                    let dep_actual_version = dep_pkg.borrow().package.borrow().version_str();

                    // Try to get the fixed version from the dependency
                    if let Ok(dep_required_version) = dep.borrow().fixed_version() {
                        let dep_required_str = dep_required_version.to_string();

                        // Check if the required version matches the actual version
                        if dep_required_str != dep_actual_version {
                            inconsistencies.push(VersionInconsistency {
                                package_name: pkg_name.clone(),
                                dependency_name: dep_name,
                                required_version: dep_required_str,
                                actual_version: dep_actual_version,
                            });
                        }
                    }
                }
            }
        }

        Ok(VersionValidation { has_cycles: dependency_analysis.cycles_detected, inconsistencies })
    }

    /// Get the workspace reference
    pub(crate) fn get_workspace(&self) -> &'a Workspace {
        self.workspace
    }

    /// Get the change tracker reference, returning an error if not available
    pub(crate) fn get_change_tracker(&self) -> VersioningResult<&'a ChangeTracker> {
        self.change_tracker.ok_or_else(|| {
            VersioningError::InvalidBumpStrategy(
                "Change tracker required for this operation".to_string(),
            )
        })
    }
}

/// Validation result for workspace version consistency.
#[derive(Debug, Clone)]
pub struct VersionValidation {
    /// Whether the dependency graph has cycles
    pub has_cycles: bool,
    /// List of version inconsistencies between packages and their dependencies
    pub inconsistencies: Vec<VersionInconsistency>,
}

/// Represents an inconsistency between a package's dependency and the actual package version.
#[derive(Debug, Clone)]
pub struct VersionInconsistency {
    /// The package with the inconsistent dependency
    pub package_name: String,
    /// The dependency package name
    pub dependency_name: String,
    /// The version required by the package
    pub required_version: String,
    /// The actual version of the dependency package
    pub actual_version: String,
}
