use super::suggest::suggest_version_bumps_with_options;
use crate::{
    suggest_version_bumps, ChangeTracker, PackageVersionChange, VersionBumpPreview,
    VersionBumpStrategy, VersionSuggestion, VersioningError, VersioningResult, Workspace,
};
use log::{debug, info};
/// Manages version operations across a workspace.
///
/// The VersionManager provides functionality for suggesting and applying version bumps
/// across packages in a workspace, with special handling for circular dependencies.
///
/// # Cycle Handling
///
/// The version manager handles circular dependencies in several ways:
///
/// - **Detection**: Cycles are detected and reported during version bumping
/// - **Harmonization**: By default, packages in the same cycle receive consistent version bumps
/// - **Visualization**: Tools are provided to visualize and understand cycles
/// - **Control**: Options allow customizing how cycles are handled during versioning
///
/// # Examples
///
/// ```
/// # use monorepo::{Workspace, ChangeTracker, VersionManager, VersionBumpStrategy};
/// # fn example(workspace: &Workspace, change_tracker: &ChangeTracker) {
/// let manager = VersionManager::new(workspace, Some(change_tracker));
///
/// // Check for cycles
/// if manager.has_cycles() {
///     println!("Cycles detected: {}", manager.visualize_cycles());
/// }
///
/// // Get suggested bumps with default cycle harmonization
/// let strategy = VersionBumpStrategy::default();
/// let suggestions = manager.suggest_bumps(&strategy).unwrap();
///
/// // Preview the changes
/// let preview = manager.preview_bumps(&strategy).unwrap();
/// # }
///
use std::collections::HashMap;

/// Manages version operations across a workspace.
///
/// The VersionManager provides functionality for suggesting and applying version bumps
/// across packages in a workspace, with special handling for circular dependencies.
///
/// # Cycle Handling
///
/// The version manager handles circular dependencies in several ways:
///
/// - **Detection**: Cycles are detected and reported during version bumping
/// - **Harmonization**: By default, packages in the same cycle receive consistent version bumps
/// - **Visualization**: Tools are provided to visualize and understand cycles
/// - **Control**: Options allow customizing how cycles are handled during versioning
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{
///     ChangeTracker, VersionBumpStrategy, VersionManager, Workspace
/// };
///
/// # fn example(workspace: &Workspace, tracker: &ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
/// // Create version manager
/// let manager = VersionManager::new(workspace, Some(tracker));
///
/// // Check for cycles
/// if manager.has_cycles() {
///     println!("Cycles detected: {}", manager.visualize_cycles());
/// }
///
/// // Create a version bump strategy
/// let strategy = VersionBumpStrategy::Independent {
///     major_if_breaking: true,
///     minor_if_feature: true,
///     patch_otherwise: true,
/// };
///
/// // Apply version bumps (dry run)
/// let dry_run = true;
/// let changes = manager.apply_bumps(&strategy, dry_run)?;
/// println!("Would update {} packages", changes.len());
/// # Ok(())
/// # }
/// ```
pub struct VersionManager<'a> {
    /// Reference to the workspace
    workspace: &'a Workspace,
    /// Optional reference to the change tracker
    change_tracker: Option<&'a ChangeTracker>,
}

impl<'a> VersionManager<'a> {
    /// Create a new version manager.
    ///
    /// # Arguments
    ///
    /// * `workspace` - Reference to the workspace containing package information
    /// * `change_tracker` - Optional reference to a change tracker for change-based version suggestions
    ///
    /// # Returns
    ///
    /// A new `VersionManager` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{ChangeTracker, VersionManager, Workspace};
    ///
    /// # fn example(workspace: &Workspace, tracker: &ChangeTracker) {
    /// // Create with change tracker
    /// let manager = VersionManager::new(workspace, Some(tracker));
    ///
    /// // Create without change tracker (limited functionality)
    /// let manager = VersionManager::new(workspace, None);
    /// # }
    /// ```
    pub fn new(workspace: &'a Workspace, change_tracker: Option<&'a ChangeTracker>) -> Self {
        Self { workspace, change_tracker }
    }

    /// Suggest version bumps based on changes.
    ///
    /// Analyzes changes and dependencies to suggest appropriate version bumps
    /// for packages in the workspace.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The version bump strategy to use
    ///
    /// # Returns
    ///
    /// A map of package names to version suggestions.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided or suggestion generation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionBumpStrategy, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Define strategy
    /// let strategy = VersionBumpStrategy::default();
    ///
    /// // Get suggestions
    /// let suggestions = manager.suggest_bumps(&strategy)?;
    ///
    /// // Process suggestions
    /// for (package, suggestion) in suggestions {
    ///     println!("Suggest bumping {} from {} to {}",
    ///         package,
    ///         suggestion.current_version,
    ///         suggestion.suggested_version
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Suggests version bumps with options for handling cycles.
    ///
    /// Similar to `suggest_bumps`, but with additional control over cycle harmonization.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The version bump strategy to use
    /// * `harmonize_cycles` - Whether to ensure packages in the same cycle get consistent version bumps
    ///
    /// # Returns
    ///
    /// A map of package names to version suggestions.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided or suggestion generation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionBumpStrategy, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let strategy = VersionBumpStrategy::default();
    ///
    /// // Get suggestions without cycle harmonization
    /// let suggestions = manager.suggest_bumps_with_options(&strategy, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn suggest_bumps_with_options(
        &self,
        strategy: &VersionBumpStrategy,
        harmonize_cycles: bool,
    ) -> VersioningResult<HashMap<String, VersionSuggestion>> {
        // Get change tracker
        let change_tracker = self.get_change_tracker()?;

        // Pass harmonize_cycles option to suggest_version_bumps
        suggest_version_bumps_with_options(
            self.workspace,
            change_tracker,
            strategy,
            harmonize_cycles,
        )
    }

    /// Preview version bumps without applying.
    ///
    /// Generates a preview of version bumps that would be applied based on the strategy,
    /// without actually modifying any files.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The version bump strategy to use
    ///
    /// # Returns
    ///
    /// A preview of version changes.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided or preview generation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionBumpStrategy, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let strategy = VersionBumpStrategy::default();
    ///
    /// // Generate preview
    /// let preview = manager.preview_bumps(&strategy)?;
    ///
    /// // Check for cycles
    /// if preview.cycle_detected {
    ///     println!("Cycle detected - harmonizing versions");
    ///     for group in &preview.cycle_groups {
    ///         println!("Cycle group: {}", group.join(" → "));
    ///     }
    /// }
    ///
    /// // Show changes
    /// for change in &preview.changes {
    ///     println!("{}: {} → {}",
    ///         change.package_name,
    ///         change.current_version,
    ///         change.suggested_version
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn preview_bumps(
        &self,
        strategy: &VersionBumpStrategy,
    ) -> VersioningResult<VersionBumpPreview> {
        // Get suggested bumps
        let suggestions = self.suggest_bumps(strategy)?;

        // Check for dependency cycles that might affect versioning
        let dependency_analysis = self.workspace.analyze_dependencies()?;
        let cycle_detected = dependency_analysis.cycles_detected;

        // Get detailed cycle information
        let sorted_with_cycles = self.workspace.get_sorted_packages_with_circulars();
        let cycle_groups = sorted_with_cycles
            .circular
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|p| p.borrow().package.borrow().name().to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        if cycle_detected {
            info!(
                    "Cycle detected in dependencies. Version bumps have been harmonized within cycle groups."
                );

            // Add more detailed info about the cycles
            if !sorted_with_cycles.circular.is_empty() {
                info!("Circular dependency groups found:");
                for (i, group) in cycle_groups.iter().enumerate() {
                    info!("  Group {}: {}", i + 1, group.join(" → "));
                }
            }
        }

        Ok(VersionBumpPreview {
            changes: suggestions.into_values().collect(),
            cycle_detected,
            cycle_groups, // Add cycle groups to preview
        })
    }

    /// Apply version bumps.
    ///
    /// Updates package versions according to the provided strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The version bump strategy to use
    /// * `dry_run` - If true, preview changes without applying them
    ///
    /// # Returns
    ///
    /// A list of version changes that were or would be applied.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided or version application fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionBumpStrategy, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let strategy = VersionBumpStrategy::default();
    ///
    /// // Dry run first
    /// let dry_run_changes = manager.apply_bumps(&strategy, true)?;
    /// println!("Would update {} packages", dry_run_changes.len());
    ///
    /// // Actually apply changes
    /// let changes = manager.apply_bumps(&strategy, false)?;
    /// println!("Updated {} packages", changes.len());
    /// # Ok(())
    /// # }
    /// ```
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

            // Also track if this is part of a cycle
            let is_cycle_update = suggestion
                    .reasons
                    .iter()
                    .any(|reason| matches!(reason, crate::BumpReason::Other(msg) if msg.contains("Part of dependency cycle")));

            // Create the change record
            let version_change = PackageVersionChange {
                package_name: package_name.clone(),
                previous_version: current_version.clone(),
                new_version: new_version.clone(),
                bump_type: suggestion.bump_type,
                is_dependency_update,
                is_cycle_update, // Add field to track cycle updates
                cycle_group: suggestion.cycle_group.clone(), // Add the cycle group information
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
    /// Updates the change tracker to mark changes as released with the specified versions.
    /// This should be called after `apply_bumps` if you want to mark changes as released.
    ///
    /// # Arguments
    ///
    /// * `change_tracker` - Mutable reference to the change tracker
    /// * `version_changes` - List of version changes to mark as released
    /// * `dry_run` - If true, preview changes without applying them
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if marking changes fails.
    ///
    /// # Errors
    ///
    /// Returns an error if marking changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionManager, VersionBumpStrategy};
    ///
    /// # fn example(manager: &VersionManager, mut tracker: &mut sublime_monorepo_tools::ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// let strategy = VersionBumpStrategy::default();
    ///
    /// // Apply version bumps
    /// let changes = manager.apply_bumps(&strategy, false)?;
    ///
    /// // Mark changes as released
    /// VersionManager::mark_changes_as_released(&mut tracker, &changes, false)?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// Checks for inconsistencies between package versions and their references
    /// in dependency declarations.
    ///
    /// # Returns
    ///
    /// Validation results containing cycle information and version inconsistencies.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::VersionManager;
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let validation = manager.validate_versions()?;
    ///
    /// if validation.has_cycles {
    ///     println!("Dependency cycles detected");
    /// }
    ///
    /// for inconsistency in validation.inconsistencies {
    ///     println!("{} requires {} @ {}, but actual version is {}",
    ///         inconsistency.package_name,
    ///         inconsistency.dependency_name,
    ///         inconsistency.required_version,
    ///         inconsistency.actual_version
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Returns
    ///
    /// Reference to the workspace.
    pub(crate) fn get_workspace(&self) -> &'a Workspace {
        self.workspace
    }

    /// Get the change tracker reference, returning an error if not available
    ///
    /// # Returns
    ///
    /// Reference to the change tracker.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided.
    pub(crate) fn get_change_tracker(&self) -> VersioningResult<&'a ChangeTracker> {
        self.change_tracker.ok_or_else(|| {
            VersioningError::InvalidBumpStrategy(
                "Change tracker required for this operation".to_string(),
            )
        })
    }

    /// Gets all cycle groups in the workspace dependency graph.
    ///
    /// # Returns
    ///
    /// List of cycle groups, where each group is a list of package names
    /// that form a circular dependency chain.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::VersionManager;
    ///
    /// # fn example(manager: &VersionManager) {
    /// let cycles = manager.get_cycle_groups();
    /// for (i, cycle) in cycles.iter().enumerate() {
    ///     println!("Cycle group {}: {}", i + 1, cycle.join(" → "));
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_cycle_groups(&self) -> Vec<Vec<String>> {
        self.workspace.get_circular_dependencies()
    }

    /// Checks if the workspace has cyclic dependencies.
    ///
    /// # Returns
    ///
    /// `true` if circular dependencies were detected, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::VersionManager;
    ///
    /// # fn example(manager: &VersionManager) {
    /// if manager.has_cycles() {
    ///     println!("Circular dependencies detected - special handling required");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn has_cycles(&self) -> bool {
        !self.get_cycle_groups().is_empty()
    }

    /// Visualizes cycles in the dependency graph.
    ///
    /// Creates a formatted string representation of all circular dependencies
    /// in the workspace, showing both simple cycle paths and detailed
    /// dependencies within each cycle.
    ///
    /// # Returns
    ///
    /// A formatted string showing all circular dependencies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::VersionManager;
    ///
    /// # fn example(manager: &VersionManager) {
    /// if manager.has_cycles() {
    ///     println!("{}", manager.visualize_cycles());
    /// }
    /// # }
    /// ```
    pub fn visualize_cycles(&self) -> String {
        let circles = self.workspace.get_circular_dependencies();
        if circles.is_empty() {
            return "No circular dependencies found.".to_string();
        }

        let mut output = String::from("Circular Dependencies:\n");

        for (i, group) in circles.iter().enumerate() {
            output.push_str(&format!("\nCycle Group {}:\n", i + 1));

            // Simple circular representation
            output.push_str(&format!(
                "  {} → {}\n",
                group.join(" → "),
                group.first().unwrap_or(&String::from("?"))
            ));

            // Also list each package with its direct dependencies in the cycle
            output.push_str("  Detailed dependencies:\n");

            for pkg_name in group {
                let pkg_info = self.workspace.get_package(pkg_name);
                let deps = pkg_info
                    .map(|p| {
                        let pkg = p.borrow();
                        let package_borrow = pkg.package.borrow();
                        let deps = package_borrow.dependencies();
                        deps.iter()
                            .map(|d| d.borrow().name().to_string())
                            .filter(|name| group.contains(name))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                output.push_str(&format!("    {} → {}\n", pkg_name, deps.join(", ")));
            }
        }

        output
    }

    /// Generate a detailed report of version bumps including cycle information
    ///
    /// Creates a formatted report showing all version changes, with special
    /// sections for direct changes, dependency-driven updates, and cycle-harmonized updates.
    ///
    /// # Arguments
    ///
    /// * `changes` - List of version changes to include in the report
    ///
    /// # Returns
    ///
    /// A formatted string report.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{VersionBumpStrategy, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let strategy = VersionBumpStrategy::default();
    /// let changes = manager.apply_bumps(&strategy, true)?;
    ///
    /// // Generate and print the report
    /// let report = manager.generate_version_report(&changes);
    /// println!("{}", report);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::too_many_lines)]
    pub fn generate_version_report(&self, changes: &[PackageVersionChange]) -> String {
        let mut report = String::new();

        // Get cycle information
        let cycle_groups = self.get_cycle_groups();
        let has_cycles = !cycle_groups.is_empty();

        // Create a cycle membership map for quick lookups
        let mut cycle_membership: HashMap<String, usize> = HashMap::new();
        for (i, group) in cycle_groups.iter().enumerate() {
            for pkg_name in group {
                cycle_membership.insert(pkg_name.clone(), i);
            }
        }

        // First display cycle information
        if has_cycles {
            report.push_str("Circular Dependency Groups:\n");
            for (i, group) in cycle_groups.iter().enumerate() {
                report.push_str(&format!("  Group {}: {}\n", i + 1, group.join(" → ")));
            }
            report.push_str("\nNote: Version bumps are harmonized within cycle groups.\n\n");
        }

        // Group changes by type
        let mut direct_changes = Vec::new();
        let mut dependency_updates = Vec::new();
        let mut cycle_updates = Vec::new();

        for change in changes {
            if change.is_cycle_update {
                cycle_updates.push(change);
            } else if change.is_dependency_update {
                dependency_updates.push(change);
            } else {
                direct_changes.push(change);
            }
        }

        // Display direct changes
        if !direct_changes.is_empty() {
            report.push_str("Direct Changes:\n");
            for &change in &direct_changes {
                // Changed to iterate over references
                let cycle_info =
                    if let Some(&group_idx) = cycle_membership.get(&change.package_name) {
                        format!(" (cycle group {})", group_idx + 1)
                    } else {
                        String::new()
                    };

                report.push_str(&format!(
                    "  {} {} → {} ({}){}\n",
                    change.package_name,
                    change.previous_version,
                    change.new_version,
                    change.bump_type,
                    cycle_info
                ));
            }
        }

        // Display dependency updates
        if !dependency_updates.is_empty() {
            report.push_str("\nDependency-Driven Updates:\n");
            for &change in &dependency_updates {
                // Changed to iterate over references
                let cycle_info =
                    if let Some(&group_idx) = cycle_membership.get(&change.package_name) {
                        format!(" (cycle group {})", group_idx + 1)
                    } else {
                        String::new()
                    };

                report.push_str(&format!(
                    "  {} {} → {} ({}){}\n",
                    change.package_name,
                    change.previous_version,
                    change.new_version,
                    change.bump_type,
                    cycle_info
                ));
            }
        }

        // Display cycle updates
        if !cycle_updates.is_empty() {
            report.push_str("\nCycle-Harmonized Updates:\n");

            // Group by cycle
            let mut by_cycle: HashMap<usize, Vec<&PackageVersionChange>> = HashMap::new();

            for &change in &cycle_updates {
                // Changed to iterate over references
                if let Some(&group_idx) = cycle_membership.get(&change.package_name) {
                    by_cycle.entry(group_idx).or_default().push(change);
                } else {
                    // Shouldn't happen but handle it anyway
                    report.push_str(&format!(
                        "  {} {} → {} ({}) [cycle group unknown]\n",
                        change.package_name,
                        change.previous_version,
                        change.new_version,
                        change.bump_type
                    ));
                }
            }

            // Display by cycle group
            let mut cycle_groups: Vec<_> = by_cycle.iter().collect();
            cycle_groups.sort_by_key(|&(k, _)| *k);
            for (group_idx, changes) in cycle_groups {
                report.push_str(&format!("  Cycle Group {}:\n", group_idx + 1));

                for change in changes {
                    report.push_str(&format!(
                        "    {} {} → {} ({})\n",
                        change.package_name,
                        change.previous_version,
                        change.new_version,
                        change.bump_type
                    ));
                }
            }
        }

        // Summary
        report.push_str(&format!(
                "\nSummary: {} packages updated ({} direct, {} dependency-driven, {} cycle-harmonized)\n",
                changes.len(),
                direct_changes.len(),
                dependency_updates.len(),
                cycle_updates.len()
            ));

        report
    }
}

/// Validation result for workspace version consistency.
///
/// Contains information about dependency cycles and version inconsistencies
/// between packages and their dependencies.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::VersionManager;
///
/// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
/// let validation = manager.validate_versions()?;
///
/// if validation.has_cycles {
///     println!("Workspace contains circular dependencies");
/// }
///
/// for issue in &validation.inconsistencies {
///     println!("{} requires {} @ {}, but actual version is {}",
///         issue.package_name,
///         issue.dependency_name,
///         issue.required_version,
///         issue.actual_version
///     );
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VersionValidation {
    /// Whether the dependency graph has cycles
    pub has_cycles: bool,
    /// List of version inconsistencies between packages and their dependencies
    pub inconsistencies: Vec<VersionInconsistency>,
}

/// Represents an inconsistency between a package's dependency and the actual package version.
///
/// This occurs when a package requires a specific version of a dependency,
/// but the actual version of that dependency in the workspace is different.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{VersionInconsistency, VersionManager};
///
/// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
/// let validation = manager.validate_versions()?;
///
/// // Process inconsistencies
/// for issue in validation.inconsistencies {
///     println!("Inconsistency: {} requires {} @ {}, but actual is {}",
///         issue.package_name,
///         issue.dependency_name,
///         issue.required_version,
///         issue.actual_version
///     );
/// }
/// # Ok(())
/// # }
/// ```
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
