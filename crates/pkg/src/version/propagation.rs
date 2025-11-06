//! Dependency propagation logic for version changes.
//!
//! **What**: Provides the `DependencyPropagator` that propagates version changes through
//! the dependency graph, automatically updating dependent packages when their dependencies
//! change versions.
//!
//! **How**: Uses the dependency graph to find all packages that depend on changed packages,
//! respects configuration settings for which dependency types to propagate, skips special
//! protocols (workspace:, file:, link:, portal:), applies propagation bumps, and tracks
//! the propagation chain with depth information for transparency and debugging.
//!
//! **Why**: To automate the complex task of updating dependent packages when their
//! dependencies change, ensuring version consistency across the workspace while respecting
//! protocol constraints and avoiding infinite propagation loops.
//!
//! # Key Features
//!
//! - **Selective Propagation**: Configure which dependency types to propagate (dependencies,
//!   devDependencies, peerDependencies)
//! - **Protocol Skipping**: Automatically skip workspace:, file:, link:, and portal: protocols
//! - **Depth Control**: Limit propagation depth to prevent excessive cascading updates
//! - **Propagation Tracking**: Track the chain of propagation for transparency
//! - **Bump Control**: Configure the version bump type for propagated changes
//!
//! # Propagation Process
//!
//! The propagation follows these steps:
//!
//! 1. **Initialize**: Start with packages that have direct changes
//! 2. **Find Dependents**: For each changed package, find packages that depend on it
//! 3. **Filter**: Filter dependents by dependency type and protocol rules
//! 4. **Bump Version**: Apply propagation bump to dependent packages
//! 5. **Track Chain**: Record the propagation chain for transparency
//! 6. **Update Dependencies**: Calculate new dependency version specs
//! 7. **Recurse**: Continue propagation until max depth or no more dependents
//!
//! # Configuration
//!
//! Propagation behavior is controlled by `DependencyConfig`:
//!
//! ```toml
//! [package_tools.dependency]
//! propagation_bump = "patch"
//! propagate_dependencies = true
//! propagate_dev_dependencies = false
//! propagate_peer_dependencies = true
//! max_depth = 10
//! skip_workspace_protocol = true
//! skip_file_protocol = true
//! skip_link_protocol = true
//! skip_portal_protocol = true
//! ```
//!
//! # Examples
//!
//! ## Basic Propagation
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::propagation::DependencyPropagator;
//! use sublime_pkg_tools::version::{VersionResolution, DependencyGraph};
//! use sublime_pkg_tools::config::DependencyConfig;
//! use sublime_pkg_tools::types::PackageInfo;
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let packages: HashMap<String, PackageInfo> = HashMap::new(); // Load packages
//! let graph = DependencyGraph::from_packages(
//!     &packages.values().cloned().collect::<Vec<_>>()
//! )?;
//! let config = DependencyConfig::default();
//!
//! let propagator = DependencyPropagator::new(&graph, &packages, &config);
//!
//! let mut resolution = VersionResolution::new();
//! // Add initial updates...
//!
//! propagator.propagate(&mut resolution)?;
//!
//! // Now resolution contains both direct and propagated updates
//! for update in &resolution.updates {
//!     if update.is_propagated() {
//!         println!("Propagated: {}", update.name);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::config::DependencyConfig;
use crate::error::{VersionError, VersionResult};
use crate::types::dependency::{is_local_protocol, is_workspace_protocol};
use crate::types::{
    DependencyType, DependencyUpdate, PackageInfo, UpdateReason, Version, VersionBump,
};
use crate::version::DependencyGraph;
use crate::version::resolution::{PackageUpdate, VersionResolution};
use std::collections::{HashMap, HashSet};

/// Dependency propagator for version changes.
///
/// The `DependencyPropagator` handles the propagation of version changes through the
/// dependency graph. When a package's version changes, all packages that depend on it
/// are also updated according to the propagation rules in the configuration.
///
/// # Type Parameters
///
/// This struct is parameterized by lifetime `'a` for borrowed references to the graph,
/// packages, and configuration.
///
/// # Fields
///
/// * `graph` - The dependency graph for finding dependents
/// * `packages` - All packages in the workspace
/// * `config` - Dependency configuration for propagation rules
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::propagation::DependencyPropagator;
/// use sublime_pkg_tools::version::DependencyGraph;
/// use sublime_pkg_tools::config::DependencyConfig;
///
/// # fn example(
/// #     graph: &DependencyGraph,
/// #     packages: &HashMap<String, PackageInfo>,
/// #     resolution: &mut VersionResolution,
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let config = DependencyConfig::default();
/// let propagator = DependencyPropagator::new(graph, packages, &config);
///
/// propagator.propagate(resolution)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct DependencyPropagator<'a> {
    /// Dependency graph for finding dependents.
    graph: &'a DependencyGraph,
    /// All packages in the workspace.
    packages: &'a HashMap<String, PackageInfo>,
    /// Dependency configuration.
    config: &'a DependencyConfig,
}

impl<'a> DependencyPropagator<'a> {
    /// Creates a new `DependencyPropagator`.
    ///
    /// # Arguments
    ///
    /// * `graph` - The dependency graph for finding dependents
    /// * `packages` - All packages in the workspace
    /// * `config` - Dependency configuration for propagation rules
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::propagation::DependencyPropagator;
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// # fn example(
    /// #     graph: &DependencyGraph,
    /// #     packages: &HashMap<String, PackageInfo>,
    /// # ) {
    /// let config = DependencyConfig::default();
    /// let propagator = DependencyPropagator::new(graph, packages, &config);
    /// # }
    /// ```
    #[must_use]
    pub fn new(
        graph: &'a DependencyGraph,
        packages: &'a HashMap<String, PackageInfo>,
        config: &'a DependencyConfig,
    ) -> Self {
        Self { graph, packages, config }
    }

    /// Propagates version changes through the dependency graph.
    ///
    /// This method takes an initial resolution with direct changes and propagates those
    /// changes to dependent packages. It modifies the resolution in place, adding new
    /// `PackageUpdate` entries for propagated changes and populating `dependency_updates`
    /// for packages that need to update their dependency version specs.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The version resolution to propagate (modified in place)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if propagation succeeds, or an error if:
    /// - A package in the resolution is not found in the packages map
    /// - Version bump fails for any package
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageNotFound` if a package is not found.
    /// Returns `VersionError::InvalidBumpType` if the propagation bump type is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::propagation::DependencyPropagator;
    /// use sublime_pkg_tools::version::VersionResolution;
    ///
    /// # fn example(
    /// #     propagator: DependencyPropagator,
    /// #     mut resolution: VersionResolution,
    /// # ) -> Result<(), Box<dyn std::error::Error>> {
    /// propagator.propagate(&mut resolution)?;
    ///
    /// println!("Total updates after propagation: {}", resolution.updates.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn propagate(&self, resolution: &mut VersionResolution) -> VersionResult<()> {
        // Track packages that have been updated (by name -> new version)
        let mut updated_packages: HashMap<String, Version> = HashMap::new();

        // Initialize with direct changes
        for update in &resolution.updates {
            updated_packages.insert(update.name.clone(), update.next_version.clone());
        }

        // Track which packages we've already processed to avoid duplicates
        let mut processed: HashSet<String> = HashSet::new();
        for update in &resolution.updates {
            processed.insert(update.name.clone());
        }

        // Propagate changes level by level (breadth-first)
        let mut current_depth = 0;
        let mut current_level: Vec<String> =
            resolution.updates.iter().map(|u| u.name.clone()).collect();

        while !current_level.is_empty() && current_depth < self.config.max_depth {
            let mut next_level: Vec<String> = Vec::new();

            for package_name in &current_level {
                // Find all packages that depend on this package
                let dependents = self.graph.dependents(package_name);

                for dependent_name in dependents {
                    // Skip if already processed
                    if processed.contains(&dependent_name) {
                        continue;
                    }

                    // Get the dependent package info
                    let dependent_pkg = self.packages.get(&dependent_name).ok_or_else(|| {
                        VersionError::PackageNotFound {
                            name: dependent_name.clone(),
                            workspace_root: std::path::PathBuf::new(),
                        }
                    })?;

                    // Check if this dependency should trigger propagation
                    if !self.should_propagate(dependent_pkg, package_name) {
                        continue;
                    }

                    // Calculate next version for dependent
                    let current_version = dependent_pkg.version();
                    let propagation_bump = self.parse_propagation_bump()?;
                    let next_version = current_version.bump(propagation_bump)?;

                    // Create package update for this dependent
                    let update = PackageUpdate::new(
                        dependent_name.clone(),
                        dependent_pkg.path().to_path_buf(),
                        current_version,
                        next_version.clone(),
                        UpdateReason::DependencyPropagation {
                            triggered_by: package_name.clone(),
                            depth: current_depth + 1,
                        },
                    );

                    // Add to tracking
                    updated_packages.insert(dependent_name.clone(), next_version);
                    processed.insert(dependent_name.clone());
                    next_level.push(dependent_name);

                    // Add to resolution
                    resolution.add_update(update);
                }
            }

            current_level = next_level;
            current_depth += 1;
        }

        // Now update dependency_updates for all packages
        self.update_dependency_specs(resolution, &updated_packages)?;

        Ok(())
    }

    /// Checks if a dependency should trigger propagation.
    ///
    /// This method checks:
    /// 1. Whether the dependency type is enabled for propagation
    /// 2. Whether the dependency uses a protocol that should be skipped
    ///
    /// # Arguments
    ///
    /// * `dependent_pkg` - The package that depends on the changed package
    /// * `dependency_name` - Name of the dependency that changed
    ///
    /// # Returns
    ///
    /// Returns `true` if propagation should occur, `false` otherwise.
    fn should_propagate(&self, dependent_pkg: &PackageInfo, dependency_name: &str) -> bool {
        // Get all dependencies of this package
        let all_deps = dependent_pkg.all_dependencies();

        // Find the dependency and check its type
        for (dep_name, version_spec, dep_type) in all_deps {
            if dep_name != dependency_name {
                continue;
            }

            // Check if this dependency type should propagate
            let type_enabled = match dep_type {
                DependencyType::Regular => self.config.propagate_dependencies,
                DependencyType::Dev => self.config.propagate_dev_dependencies,
                DependencyType::Peer => self.config.propagate_peer_dependencies,
                DependencyType::Optional => false, // Optional deps don't propagate by default
            };

            if !type_enabled {
                return false;
            }

            // Check if protocol should be skipped
            if self.should_skip_version_spec(&version_spec) {
                return false;
            }

            return true;
        }

        false
    }

    /// Checks if a version spec should be skipped based on protocol.
    ///
    /// # Arguments
    ///
    /// * `version_spec` - The version specification to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the version spec should be skipped, `false` otherwise.
    fn should_skip_version_spec(&self, version_spec: &str) -> bool {
        if self.config.skip_workspace_protocol && is_workspace_protocol(version_spec) {
            return true;
        }

        if is_local_protocol(version_spec) {
            if self.config.skip_file_protocol && version_spec.starts_with("file:") {
                return true;
            }
            if self.config.skip_link_protocol && version_spec.starts_with("link:") {
                return true;
            }
            if self.config.skip_portal_protocol && version_spec.starts_with("portal:") {
                return true;
            }
        }

        false
    }

    /// Parses the propagation bump from configuration.
    ///
    /// # Returns
    ///
    /// Returns the `VersionBump` corresponding to the configuration value.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidBumpType` if the bump type is invalid.
    fn parse_propagation_bump(&self) -> VersionResult<VersionBump> {
        match self.config.propagation_bump.as_str() {
            "major" => Ok(VersionBump::Major),
            "minor" => Ok(VersionBump::Minor),
            "patch" => Ok(VersionBump::Patch),
            "none" => Ok(VersionBump::None),
            _ => Err(VersionError::InvalidBumpType {
                bump_type: self.config.propagation_bump.clone(),
            }),
        }
    }

    /// Updates dependency version specifications for all packages.
    ///
    /// This method goes through all packages in the resolution and updates their
    /// `dependency_updates` field with the new version specs for any dependencies
    /// that were updated.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The version resolution (modified in place)
    /// * `updated_packages` - Map of package names to their new versions
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageNotFound` if a package is not found.
    fn update_dependency_specs(
        &self,
        resolution: &mut VersionResolution,
        updated_packages: &HashMap<String, Version>,
    ) -> VersionResult<()> {
        // We need to update the updates in place
        // Since we can't mutate while iterating, collect indices first
        let update_count = resolution.updates.len();

        for i in 0..update_count {
            let update_name = resolution.updates[i].name.clone();

            // Get the package info
            let pkg =
                self.packages.get(&update_name).ok_or_else(|| VersionError::PackageNotFound {
                    name: update_name.clone(),
                    workspace_root: std::path::PathBuf::new(),
                })?;

            // Find dependencies that need updating
            let all_deps = pkg.all_dependencies();
            let mut dep_updates: Vec<DependencyUpdate> = Vec::new();

            for (dep_name, old_spec, dep_type) in all_deps {
                // Check if this dependency was updated
                if let Some(new_version) = updated_packages.get(&dep_name) {
                    // Skip if protocol should be skipped
                    if self.should_skip_version_spec(&old_spec) {
                        continue;
                    }

                    // Calculate new version spec
                    let new_spec = self.calculate_new_version_spec(&old_spec, new_version);

                    // Only add if the spec actually changes
                    if new_spec != old_spec {
                        dep_updates
                            .push(DependencyUpdate::new(dep_name, dep_type, old_spec, new_spec));
                    }
                }
            }

            // Add dependency updates to this package update
            for dep_update in dep_updates {
                resolution.updates[i].add_dependency_update(dep_update);
            }
        }

        Ok(())
    }

    /// Calculates the new version spec for a dependency.
    ///
    /// This method attempts to preserve the range operator from the old spec
    /// while updating to the new version.
    ///
    /// # Arguments
    ///
    /// * `old_spec` - The old version specification
    /// * `new_version` - The new version to use
    ///
    /// # Returns
    ///
    /// Returns the new version specification string.
    ///
    /// # Examples
    ///
    /// - `^1.0.0` with new version `2.0.0` -> `^2.0.0`
    /// - `~1.0.0` with new version `1.1.0` -> `~1.1.0`
    /// - `>=1.0.0` with new version `2.0.0` -> `>=2.0.0`
    /// - `1.0.0` with new version `2.0.0` -> `2.0.0`
    fn calculate_new_version_spec(&self, old_spec: &str, new_version: &Version) -> String {
        let trimmed = old_spec.trim();

        // Detect range operator
        if trimmed.starts_with('^') {
            format!("^{}", new_version)
        } else if trimmed.starts_with('~') {
            format!("~{}", new_version)
        } else if trimmed.starts_with(">=") {
            format!(">={}", new_version)
        } else if trimmed.starts_with('>') {
            format!(">{}", new_version)
        } else if trimmed.starts_with("<=") {
            format!("<={}", new_version)
        } else if trimmed.starts_with('<') {
            format!("<{}", new_version)
        } else if trimmed.starts_with('=') {
            format!("={}", new_version)
        } else {
            // No operator, just use the version
            new_version.to_string()
        }
    }
}
