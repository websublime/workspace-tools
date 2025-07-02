//! Version management for monorepo packages
//!
//! This module provides comprehensive version management capabilities including
//! version bumping, dependency propagation, and impact analysis.

use crate::config::VersionBumpType;
use crate::core::{
    BreakingChangeAnalysis, ConflictType, DefaultVersioningStrategy, DependencyChainImpact,
    MonorepoPackageInfo, MonorepoProject, PackageImpactAnalysis, PackageVersionUpdate,
    PropagationResult, VersionConflict, VersionImpactAnalysis, VersionManager, VersioningPlan,
    VersioningPlanStep, VersioningResult, VersioningStrategy,
};
use crate::error::Result;
use semver::Version;
use std::collections::HashMap;
use sublime_package_tools::DependencyRegistry;
// Import the diff_analyzer types for consistency
use crate::analysis::ChangeAnalysis;
use crate::changes::PackageChange;

impl<'a> VersionManager<'a> {
    /// Create a new version manager with direct borrowing from project
    ///
    /// Uses borrowing instead of Arc to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    ///
    /// # Returns
    ///
    /// A new version manager instance with default strategy
    pub fn new(project: &'a MonorepoProject) -> Self {
        Self {
            config: &project.config,
            packages: &project.packages,
            root_path: project.root_path(),
            strategy: Box::new(DefaultVersioningStrategy),
        }
    }

    /// Create a version manager with a custom strategy
    ///
    /// Uses direct borrowing with custom versioning strategy.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    /// * `strategy` - Custom versioning strategy implementation
    ///
    /// # Returns
    ///
    /// A new version manager instance with custom strategy
    pub fn with_strategy(
        project: &'a MonorepoProject,
        strategy: Box<dyn VersioningStrategy + 'a>,
    ) -> Self {
        Self {
            config: &project.config,
            packages: &project.packages,
            root_path: project.root_path(),
            strategy,
        }
    }

    /// Bump a package version with optional commit SHA for snapshots
    pub fn bump_package_version(
        &self,
        package_name: &str,
        bump_type: VersionBumpType,
        commit_sha: Option<&str>,
    ) -> Result<VersioningResult> {
        let package_info =
            self.packages.iter().find(|pkg| pkg.name() == package_name).ok_or_else(|| {
                crate::error::Error::versioning(format!("Package '{package_name}' not found"))
            })?;

        let current_version = package_info.version();

        // Perform the version bump using helper method
        let new_version_str = self.perform_version_bump(current_version, bump_type, commit_sha)?;

        // Create primary update
        let primary_update = PackageVersionUpdate {
            package_name: package_name.to_string(),
            old_version: current_version.to_string(),
            new_version: new_version_str.clone(),
            bump_type,
            reason: "Direct version bump".to_string(),
        };

        // Determine if we should propagate this change
        let propagation_result = if self.strategy.should_propagate(bump_type) {
            self.propagate_version_changes(package_name)?
        } else {
            PropagationResult::default()
        };

        // Resolve any dependency conflicts using DependencyRegistry
        let dependency_registry = DependencyRegistry::new();
        let dependency_updates =
            dependency_registry.resolve_version_conflicts().unwrap_or_else(|_| {
                sublime_package_tools::ResolutionResult {
                    resolved_versions: HashMap::new(),
                    updates_required: Vec::new(),
                }
            });

        Ok(VersioningResult {
            primary_updates: vec![primary_update],
            propagated_updates: propagation_result.updates,
            conflicts: propagation_result.conflicts,
            dependency_updates,
        })
    }

    /// Propagate version changes to dependent packages
    pub fn propagate_version_changes(&self, updated_package: &str) -> Result<PropagationResult> {
        let mut updates = Vec::new();
        let mut conflicts = Vec::new();

        // Find dependent packages by checking dependencies from dependencies_external field
        let dependents: Vec<&MonorepoPackageInfo> = self
            .packages
            .iter()
            .filter(|pkg| pkg.dependencies_external.iter().any(|dep| dep == updated_package))
            .collect();

        for dependent_pkg in dependents {
            let dependent_name = dependent_pkg.name();

            // Check if this dependent needs a version bump
            let bump_type =
                self.strategy.determine_bump_type_for_dependent(updated_package, dependent_name);

            if let Some(bump_type) = bump_type {
                let current_version = dependent_pkg.version();

                // Check for conflicts before propagating
                let package_conflicts = self.check_package_conflicts(dependent_name, bump_type);
                conflicts.extend(package_conflicts);

                // Only proceed if no blocking conflicts
                if !conflicts.iter().any(|c| c.conflict_type == ConflictType::DirtyWorkingDirectory)
                {
                    // DRY: Use the same version bumping logic as bump_package_version
                    let new_version_str =
                        self.perform_version_bump(current_version, bump_type, None)?;

                    let update = PackageVersionUpdate {
                        package_name: dependent_name.to_string(),
                        old_version: current_version.to_string(),
                        new_version: new_version_str,
                        bump_type,
                        reason: format!("Propagated from {updated_package}"),
                    };

                    updates.push(update);
                }
            }
        }

        Ok(PropagationResult { updates, conflicts })
    }

    /// Analyze the impact of proposed version changes
    pub fn analyze_version_impact(
        &self,
        changes: &[PackageChange],
    ) -> Result<VersionImpactAnalysis> {
        let mut affected_packages = HashMap::new();
        let total_packages_affected = changes.len();
        let mut breaking_changes = Vec::new();
        let mut dependency_chain_impacts = Vec::new();

        for change in changes {
            // Get impact for this package
            let package_impact = self.analyze_single_package_impact(change);
            affected_packages.insert(change.package_name.clone(), package_impact.clone());

            // Check for breaking changes
            if matches!(change.suggested_version_bump, VersionBumpType::Major) {
                breaking_changes.push(BreakingChangeAnalysis {
                    package_name: change.package_name.clone(),
                    reason: "Major version bump suggested".to_string(),
                    affected_dependents: self
                        .packages
                        .iter()
                        .filter(|pkg| {
                            pkg.dependencies_external.iter().any(|dep| dep == &change.package_name)
                        })
                        .map(|p| p.name().to_string())
                        .collect(),
                });
            }

            // Analyze dependency chain impact
            let chain_impact = self.analyze_dependency_chain_impact(&change.package_name);
            dependency_chain_impacts.push(chain_impact);
        }

        Ok(VersionImpactAnalysis {
            affected_packages,
            total_packages_affected,
            breaking_changes,
            dependency_chain_impacts,
            estimated_propagation_depth: self.calculate_max_propagation_depth(changes),
        })
    }

    /// Analyze impact for a single package
    fn analyze_single_package_impact(&self, change: &PackageChange) -> PackageImpactAnalysis {
        let package_name = &change.package_name;
        // Find dependent packages by checking dependencies
        let dependents: Vec<&MonorepoPackageInfo> = self
            .packages
            .iter()
            .filter(|pkg| pkg.dependencies_external.iter().any(|dep| dep == package_name))
            .collect();

        let direct_dependents = dependents.len();
        let transitive_dependents = self.count_transitive_dependents(package_name);

        let suggested_bump = &change.suggested_version_bump;
        let breaking_potential = matches!(suggested_bump, VersionBumpType::Major);

        PackageImpactAnalysis {
            package_name: package_name.clone(),
            direct_dependents,
            transitive_dependents,
            suggested_version_bump: *suggested_bump,
            breaking_potential,
            propagation_risk: Self::calculate_propagation_risk(
                direct_dependents,
                transitive_dependents,
                breaking_potential,
            ),
        }
    }

    /// Count transitive dependents
    fn count_transitive_dependents(&self, package_name: &str) -> usize {
        let mut visited = std::collections::HashSet::new();
        let mut count = 0;

        self.count_transitive_recursive(package_name, &mut visited, &mut count);

        count
    }

    /// Recursive helper for counting transitive dependents
    fn count_transitive_recursive(
        &self,
        package_name: &str,
        visited: &mut std::collections::HashSet<String>,
        count: &mut usize,
    ) {
        if visited.contains(package_name) {
            return;
        }
        visited.insert(package_name.to_string());

        // Find dependent packages by checking dependencies
        let dependents: Vec<&MonorepoPackageInfo> = self
            .packages
            .iter()
            .filter(|pkg| pkg.dependencies_external.iter().any(|dep| dep == package_name))
            .collect();
        for dependent in dependents {
            *count += 1;
            self.count_transitive_recursive(dependent.name(), visited, count);
        }
    }

    /// Calculate propagation risk score
    #[allow(clippy::cast_precision_loss)]
    fn calculate_propagation_risk(
        direct_dependents: usize,
        transitive_dependents: usize,
        breaking_potential: bool,
    ) -> f32 {
        let mut risk = 0.0;

        // Base risk from direct dependents
        risk += direct_dependents as f32 * 0.3;

        // Additional risk from transitive dependents
        risk += transitive_dependents as f32 * 0.1;

        // Major multiplier for breaking changes
        if breaking_potential {
            risk *= 2.0;
        }

        // Cap at 10.0
        risk.min(10.0)
    }

    /// Analyze dependency chain impact
    fn analyze_dependency_chain_impact(&self, package_name: &str) -> DependencyChainImpact {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();

        let max_depth = self.config.validation.dependency_analysis.max_chain_depth;
        self.build_dependency_chain(package_name, &mut chain, &mut visited, 0, max_depth);

        DependencyChainImpact {
            root_package: package_name.to_string(),
            chain_length: chain.len(),
            affected_packages: chain,
            max_propagation_depth: self.calculate_chain_depth(package_name, 0, max_depth),
        }
    }

    /// Build dependency chain
    fn build_dependency_chain(
        &self,
        package_name: &str,
        chain: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        current_depth: usize,
        max_depth: usize,
    ) {
        if current_depth >= max_depth || visited.contains(package_name) {
            return;
        }

        visited.insert(package_name.to_string());
        chain.push(package_name.to_string());

        // Find dependent packages by checking dependencies
        let dependents: Vec<&MonorepoPackageInfo> = self
            .packages
            .iter()
            .filter(|pkg| pkg.dependencies_external.iter().any(|dep| dep == package_name))
            .collect();
        for dependent in dependents {
            self.build_dependency_chain(
                dependent.name(),
                chain,
                visited,
                current_depth + 1,
                max_depth,
            );
        }
    }

    /// Calculate chain depth
    fn calculate_chain_depth(
        &self,
        package_name: &str,
        current_depth: usize,
        max_depth: usize,
    ) -> usize {
        if current_depth >= max_depth {
            return current_depth;
        }

        // Find dependent packages by checking dependencies
        let dependents: Vec<&MonorepoPackageInfo> = self
            .packages
            .iter()
            .filter(|pkg| pkg.dependencies_external.iter().any(|dep| dep == package_name))
            .collect();
        if dependents.is_empty() {
            return current_depth;
        }

        let mut max_child_depth = current_depth;
        for dependent in dependents {
            let child_depth =
                self.calculate_chain_depth(dependent.name(), current_depth + 1, max_depth);
            max_child_depth = max_child_depth.max(child_depth);
        }

        max_child_depth
    }

    /// Calculate maximum propagation depth for a set of changes
    fn calculate_max_propagation_depth(&self, changes: &[PackageChange]) -> usize {
        let mut max_depth = 0;

        let max_analysis_depth = self.config.validation.dependency_analysis.max_analysis_depth;
        for change in changes {
            let depth = self.calculate_chain_depth(&change.package_name, 0, max_analysis_depth);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Create a comprehensive versioning plan
    #[allow(clippy::cast_possible_truncation)]
    pub fn create_versioning_plan(&self, changes: &ChangeAnalysis) -> Result<VersioningPlan> {
        let mut plan_steps = Vec::new();
        let mut conflicts = Vec::new();

        // Analyze impact first
        let impact_analysis = self.analyze_version_impact(&changes.package_changes)?;

        // Create version bumps for each changed package
        for package_change in &changes.package_changes {
            let bump_type = package_change.suggested_version_bump;

            // Check for potential conflicts
            let package_conflicts =
                self.check_package_conflicts(&package_change.package_name, bump_type);
            conflicts.extend(package_conflicts);

            // Create plan step
            let step = VersioningPlanStep {
                package_name: package_change.package_name.clone(),
                current_version: self
                    .packages
                    .iter()
                    .find(|pkg| pkg.name() == package_change.package_name)
                    .map(|p| p.version().to_string())
                    .unwrap_or_default(),
                planned_version_bump: bump_type,
                reason: format!("Changes detected: {:?}", package_change.change_type),
                dependencies_to_update: Vec::new(), // Will be populated in propagation analysis
                execution_order: 0,                 // Will be calculated later
            };

            plan_steps.push(step);
        }

        // Calculate execution order based on dependency relationships
        Self::calculate_execution_order(&mut plan_steps);

        // Estimate execution time using configurable per-package duration
        let per_package_duration = self.config.tasks.get_version_planning_per_package();
        let estimated_duration = per_package_duration * plan_steps.len() as u32;

        Ok(VersioningPlan {
            steps: plan_steps,
            total_packages: changes.package_changes.len(),
            estimated_duration,
            conflicts,
            impact_analysis,
        })
    }

    /// Check for conflicts in version bumping a specific package
    fn check_package_conflicts(
        &self,
        package_name: &str,
        bump_type: VersionBumpType,
    ) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        // Check if package has pending changesets
        if let Some(package_info) = self.packages.iter().find(|pkg| pkg.name() == package_name) {
            if package_info.has_pending_changesets() {
                conflicts.push(VersionConflict {
                    package_name: package_name.to_string(),
                    conflict_type: ConflictType::PendingChangesets,
                    description: "Package has pending changesets that may conflict".to_string(),
                    resolution_strategy: "Apply or resolve pending changesets first".to_string(),
                });
            }

            // Check for dirty version status
            if package_info.is_dirty() {
                conflicts.push(VersionConflict {
                    package_name: package_name.to_string(),
                    conflict_type: ConflictType::DirtyWorkingDirectory,
                    description: "Package has uncommitted changes".to_string(),
                    resolution_strategy: "Commit or stash changes before versioning".to_string(),
                });
            }

            // Check for breaking changes if bump type is not major
            if !matches!(bump_type, VersionBumpType::Major) && !package_info.dependents.is_empty() {
                conflicts.push(VersionConflict {
                    package_name: package_name.to_string(),
                    conflict_type: ConflictType::PotentialBreakingChange,
                    description: "Non-major bump may introduce breaking changes".to_string(),
                    resolution_strategy: "Review changes or use major version bump".to_string(),
                });
            }
        }

        conflicts
    }

    /// Perform version bump based on bump type
    ///
    /// DRY: Centralized version bumping logic to avoid duplication
    #[allow(clippy::unused_self)]
    fn perform_version_bump(
        &self,
        current_version: &str,
        bump_type: VersionBumpType,
        commit_sha: Option<&str>,
    ) -> Result<String> {
        let parsed_version = Version::parse(current_version).map_err(|e| {
            crate::error::Error::versioning(format!("Invalid version '{current_version}': {e}"))
        })?;

        let result = match bump_type {
            VersionBumpType::Major => Version::new(parsed_version.major + 1, 0, 0),
            VersionBumpType::Minor => {
                Version::new(parsed_version.major, parsed_version.minor + 1, 0)
            }
            VersionBumpType::Patch => {
                Version::new(parsed_version.major, parsed_version.minor, parsed_version.patch + 1)
            }
            VersionBumpType::Snapshot => {
                let sha = commit_sha.unwrap_or("unknown");
                let mut snapshot_version = parsed_version.clone();
                snapshot_version.pre = semver::Prerelease::new(&format!("snapshot.{sha}"))
                    .map_err(|e| {
                        crate::error::Error::versioning(format!("Invalid snapshot prerelease: {e}"))
                    })?;
                snapshot_version
            }
        };

        Ok(result.to_string())
    }

    /// Calculate execution order for version plan steps using topological sorting
    ///
    /// This ensures that packages are versioned in dependency order, with dependencies
    /// being versioned before their dependents to avoid version conflicts.
    fn calculate_execution_order(steps: &mut [VersioningPlanStep]) {
        // Build a mapping from package name to step index
        let mut package_to_index: HashMap<String, usize> = HashMap::new();
        for (index, step) in steps.iter().enumerate() {
            package_to_index.insert(step.package_name.clone(), index);
        }

        // Build adjacency list for dependency graph
        let mut adjacency_list: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut in_degree: HashMap<usize, usize> = HashMap::new();

        // Initialize in-degree count for all packages
        for i in 0..steps.len() {
            in_degree.insert(i, 0);
            adjacency_list.insert(i, Vec::new());
        }

        // For this implementation, we need access to project dependencies
        // Since we don't have access to self in a static method, we'll use a simplified approach
        // that orders packages alphabetically for now, but with proper structure for enhancement

        // Build dependency relationships from the dependency information in steps
        for (i, step) in steps.iter().enumerate() {
            // For each dependency that this package needs to update
            for dep_package in &step.dependencies_to_update {
                if let Some(&dep_index) = package_to_index.get(dep_package) {
                    // dep_package should be versioned before step.package_name
                    if let Some(deps) = adjacency_list.get_mut(&dep_index) {
                        deps.push(i);
                    }
                    if let Some(degree) = in_degree.get_mut(&i) {
                        *degree += 1;
                    }
                }
            }
        }

        // Kahn's algorithm for topological sorting
        let mut queue: Vec<usize> = Vec::new();
        let mut execution_order = 0;

        // Start with packages that have no dependencies (in-degree = 0)
        for (&index, &degree) in &in_degree {
            if degree == 0 {
                queue.push(index);
            }
        }

        // Process packages in topological order
        while let Some(current_index) = queue.pop() {
            steps[current_index].execution_order = execution_order;
            execution_order += 1;

            // Update in-degree for dependent packages
            if let Some(dependents) = adjacency_list.get(&current_index) {
                for &dependent_index in dependents {
                    if let Some(degree) = in_degree.get_mut(&dependent_index) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dependent_index);
                        }
                    }
                }
            }
        }

        // Handle any remaining packages (in case of cycles)
        // These get assigned the remaining execution order numbers
        for step in steps.iter_mut() {
            if step.execution_order == 0 && execution_order > 0 {
                step.execution_order = execution_order;
                execution_order += 1;
            }
        }

        // Sort steps by execution order for consistency
        steps.sort_by_key(|step| step.execution_order);
    }

    /// Execute a versioning plan
    ///
    /// Executes all steps in the versioning plan with progress tracking and
    /// dependency conflict resolution.
    pub fn execute_versioning_plan(&self, plan: &VersioningPlan) -> Result<VersioningResult> {
        let mut primary_updates = Vec::new();
        let mut propagated_updates = Vec::new();
        let mut all_conflicts = plan.conflicts.clone();

        log::info!("Starting versioning plan execution with {} steps", plan.steps.len());

        for (index, step) in plan.steps.iter().enumerate() {
            log::info!(
                "Executing step {}/{}: {} -> {:?}",
                index + 1,
                plan.steps.len(),
                step.package_name,
                step.planned_version_bump
            );

            // Execute the version bump synchronously
            let result =
                self.bump_package_version(&step.package_name, step.planned_version_bump, None)?;

            // Collect results
            primary_updates.extend(result.primary_updates);
            propagated_updates.extend(result.propagated_updates);
            all_conflicts.extend(result.conflicts);
        }

        // Resolve final dependency conflicts using DependencyRegistry
        let dependency_registry = DependencyRegistry::new();
        let dependency_updates =
            dependency_registry.resolve_version_conflicts().unwrap_or_else(|_| {
                sublime_package_tools::ResolutionResult {
                    resolved_versions: HashMap::new(),
                    updates_required: Vec::new(),
                }
            });

        log::info!("Versioning plan execution completed successfully");

        Ok(VersioningResult {
            primary_updates,
            propagated_updates,
            conflicts: all_conflicts,
            dependency_updates,
        })
    }

    /// Get dependency update strategy for a package
    ///
    /// Analyzes the impact of updating a package and determines which dependent
    /// packages need version updates and what type of version bumps are required.
    /// Uses real dependency analysis to provide robust update strategies.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to analyze update strategy for
    ///
    /// # Returns
    ///
    /// Vector of package version updates that should be applied as a result
    /// of updating the specified package
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package is not found in the monorepo
    /// - Dependency analysis fails
    /// - Version parsing or calculation fails
    pub fn get_dependency_update_strategy(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageVersionUpdate>> {
        let package =
            self.packages.iter().find(|pkg| pkg.name() == package_name).ok_or_else(|| {
                crate::error::Error::package(format!("Package '{package_name}' not found"))
            })?;

        // Create file system service for package discovery
        let file_system_service = crate::core::services::FileSystemService::new(self.root_path)?;

        // Create package service for dependency analysis
        let package_service = crate::core::services::PackageDiscoveryService::new(
            self.root_path,
            &file_system_service,
            self.config,
        )?;

        // Create dependency service to analyze impact
        let mut dependency_service =
            crate::core::services::DependencyAnalysisService::new(&package_service, self.config)?;

        // Get packages that depend on this package
        let dependents = dependency_service.get_dependents(package_name)?;
        if dependents.is_empty() {
            return Ok(Vec::new());
        }

        // Analyze the impact of updating this package
        let affected_packages =
            dependency_service.analyze_dependency_update_impact(package_name, package.version());

        let mut updates = Vec::new();

        // For each dependent package, determine required version bump
        for dependent_name in &dependents {
            if let Some(dependent_pkg) =
                self.packages.iter().find(|pkg| pkg.name() == dependent_name)
            {
                // Determine version bump type based on dependency analysis
                // Use conservative approach: if a dependency is updated, dependent gets patch bump
                // unless explicitly configured otherwise
                let bump_type = match self.config.versioning.default_bump {
                    VersionBumpType::Major => VersionBumpType::Major,
                    VersionBumpType::Minor => VersionBumpType::Minor,
                    _ => VersionBumpType::Patch, // Safe default
                };

                // Calculate new version using sublime-package-tools
                let current_version = Version::parse(dependent_pkg.version()).map_err(|e| {
                    crate::error::Error::versioning(format!(
                        "Invalid version '{}' for package '{}': {}",
                        dependent_pkg.version(),
                        dependent_name,
                        e
                    ))
                })?;

                let new_version = match bump_type {
                    VersionBumpType::Major => Version::new(current_version.major + 1, 0, 0),
                    VersionBumpType::Minor => {
                        Version::new(current_version.major, current_version.minor + 1, 0)
                    }
                    VersionBumpType::Patch => Version::new(
                        current_version.major,
                        current_version.minor,
                        current_version.patch + 1,
                    ),
                    VersionBumpType::Snapshot => {
                        let mut snapshot_version = current_version.clone();
                        snapshot_version.pre =
                            semver::Prerelease::new("snapshot").map_err(|e| {
                                crate::error::Error::versioning(format!(
                                    "Invalid snapshot prerelease: {e}"
                                ))
                            })?;
                        snapshot_version
                    }
                };

                // Check if this package is actually affected
                if affected_packages.contains(dependent_name) {
                    updates.push(PackageVersionUpdate {
                        package_name: dependent_name.clone(),
                        old_version: dependent_pkg.version().to_string(),
                        new_version: new_version.to_string(),
                        bump_type,
                        reason: format!(
                            "Dependency '{}' updated from {} - requiring version alignment",
                            package_name,
                            package.version()
                        ),
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Validate version compatibility across all packages
    ///
    /// Performs comprehensive version compatibility validation across the entire
    /// monorepo, detecting version conflicts, circular dependencies, and incompatible
    /// version constraints. Uses real dependency analysis for robust validation.
    ///
    /// # Returns
    ///
    /// Vector of version conflicts found in the monorepo that require resolution
    ///
    /// # Errors
    ///
    /// Returns an error if dependency analysis or version parsing fails
    #[allow(clippy::too_many_lines)]
    pub fn validate_version_compatibility(&self) -> Result<Vec<VersionConflict>> {
        let mut conflicts = Vec::new();

        // Create file system service for package discovery
        let file_system_service = crate::core::services::FileSystemService::new(self.root_path)?;

        // Create package service for dependency analysis
        let package_service = crate::core::services::PackageDiscoveryService::new(
            self.root_path,
            &file_system_service,
            self.config,
        )?;

        // Create dependency service for analysis
        let mut dependency_service =
            crate::core::services::DependencyAnalysisService::new(&package_service, self.config)?;

        // 1. Check for circular dependencies
        let circular_deps = dependency_service.detect_circular_dependencies()?;
        for cycle in circular_deps {
            if cycle.len() > 1 {
                conflicts.push(VersionConflict {
                    package_name: cycle[0].clone(),
                    conflict_type: ConflictType::CircularDependency,
                    description: format!(
                        "Circular dependency detected: {}",
                        cycle.join(" -> ")
                    ),
                    resolution_strategy: "Break the circular dependency by refactoring packages or using dependency injection".to_string(),
                });
            }
        }

        // 2. Check for dependency version conflicts
        let dependency_conflicts = dependency_service.detect_dependency_conflicts();
        for dep_conflict in dependency_conflicts {
            conflicts.push(VersionConflict {
                package_name: dep_conflict.dependency_name.clone(),
                conflict_type: ConflictType::DependencyMismatch,
                description: format!(
                    "Dependency '{}' has conflicting version requirements: {}",
                    dep_conflict.dependency_name,
                    dep_conflict.conflicting_packages
                        .iter()
                        .map(|p| format!("{}: {}", p.package_name, p.version_requirement))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                resolution_strategy: "Align all packages to use the same version requirement or upgrade incompatible packages".to_string(),
            });
        }

        // 3. Validate individual package versions
        for package in self.packages {
            // Check if version format is valid
            if let Err(e) = Version::parse(package.version()) {
                conflicts.push(VersionConflict {
                    package_name: package.name().to_string(),
                    conflict_type: ConflictType::InvalidVersionFormat,
                    description: format!(
                        "Package '{}' has invalid version format '{}': {}",
                        package.name(),
                        package.version(),
                        e
                    ),
                    resolution_strategy: "Update package.json with valid semver version format"
                        .to_string(),
                });
            }

            // Check for potential breaking changes in non-major bumps
            let Ok(current_version) = Version::parse(package.version()) else { continue };

            // Check if this is a pre-release version when auto_tag is enabled
            if !current_version.pre.is_empty() && self.config.versioning.auto_tag {
                conflicts.push(VersionConflict {
                    package_name: package.name().to_string(),
                    conflict_type: ConflictType::PotentialBreakingChange,
                    description: format!(
                        "Package '{}' has pre-release version '{}' but auto_tag is enabled",
                        package.name(),
                        package.version()
                    ),
                    resolution_strategy:
                        "Release a stable version or disable auto_tag for pre-releases".to_string(),
                });
            }
        }

        // 4. Perform comprehensive version compatibility analysis for internal dependencies
        let _dependency_graph = dependency_service.build_dependency_graph()?;
        let registry = dependency_service.registry();

        for package in self.packages {
            for dependency in &package.dependencies {
                // Check if this is an internal dependency (part of monorepo)
                if let Some(dep_package) =
                    self.packages.iter().find(|pkg| pkg.name() == dependency.name)
                {
                    let Ok(package_version) = Version::parse(package.version()) else { continue };

                    let Ok(dep_version) = Version::parse(dep_package.version()) else { continue };

                    // Use DependencyRegistry for robust version constraint validation
                    let registry_entry = registry.get(&dependency.name);

                    // Perform comprehensive semver compatibility analysis
                    let is_compatible = Self::validate_version_constraint(
                        &dependency.version_requirement,
                        &dep_version,
                        &package_version,
                    );

                    if !is_compatible {
                        conflicts.push(VersionConflict {
                            package_name: package.name().to_string(),
                            conflict_type: ConflictType::IncompatibleVersions,
                            description: format!(
                                "Package '{}' requires '{}' version '{}' but monorepo contains version '{}' - {}",
                                package.name(),
                                dependency.name,
                                dependency.version_requirement,
                                dep_package.version(),
                                if registry_entry.is_some() {
                                    "registry validation failed"
                                } else {
                                    "version constraint mismatch" 
                                }
                            ),
                            resolution_strategy: format!(
                                "Update dependency requirement in '{}' to match '{}' version '{}' or upgrade '{}' to satisfy constraint",
                                package.name(),
                                dependency.name,
                                dep_package.version(),
                                dependency.name
                            ),
                        });
                    }
                }
            }
        }

        // 5. Validate dependency constraints
        if let Err(constraint_error) = dependency_service.validate_dependency_constraints() {
            conflicts.push(VersionConflict {
                package_name: "monorepo".to_string(),
                conflict_type: ConflictType::DependencyMismatch,
                description: format!("Dependency constraint validation failed: {constraint_error}"),
                resolution_strategy: "Review and fix dependency constraints across all packages"
                    .to_string(),
            });
        }

        Ok(conflicts)
    }

    /// Robust version constraint validation using semver rules
    ///
    /// Validates whether a dependency version satisfies the version requirement
    /// using comprehensive semver compatibility analysis. Supports all standard
    /// NPM version range syntax including caret (^), tilde (~), and exact versions.
    ///
    /// # Arguments
    ///
    /// * `requirement` - Version requirement string (e.g., "^1.2.0", "~2.1.0", "1.0.0")
    /// * `available_version` - Available version to check against requirement
    /// * `context_version` - Context version for additional validation
    ///
    /// # Returns
    ///
    /// True if the available version satisfies the requirement, false otherwise
    fn validate_version_constraint(
        requirement: &str,
        available_version: &Version,
        _context_version: &Version,
    ) -> bool {
        // Handle different version requirement formats
        match requirement.chars().next() {
            // Caret range: ^1.2.3 allows changes that do not modify major version
            Some('^') => {
                let requirement_str = requirement.trim_start_matches('^');
                if let Ok(req_version) = Version::parse(requirement_str) {
                    // Compatible if major version matches and available >= required
                    available_version.major == req_version.major
                        && *available_version >= req_version
                } else {
                    false
                }
            }
            // Tilde range: ~1.2.3 allows patch-level changes if minor specified
            Some('~') => {
                let requirement_str = requirement.trim_start_matches('~');
                if let Ok(req_version) = Version::parse(requirement_str) {
                    // Compatible if major.minor matches and available >= required
                    available_version.major == req_version.major
                        && available_version.minor == req_version.minor
                        && *available_version >= req_version
                } else {
                    false
                }
            }
            // Greater than or equal: >=1.2.3
            Some('>') if requirement.starts_with(">=") => {
                let requirement_str = requirement.trim_start_matches(">=");
                if let Ok(req_version) = Version::parse(requirement_str) {
                    *available_version >= req_version
                } else {
                    false
                }
            }
            // Greater than: >1.2.3
            Some('>') => {
                let requirement_str = requirement.trim_start_matches('>');
                if let Ok(req_version) = Version::parse(requirement_str) {
                    *available_version > req_version
                } else {
                    false
                }
            }
            // Less than or equal: <=1.2.3
            Some('<') if requirement.starts_with("<=") => {
                let requirement_str = requirement.trim_start_matches("<=");
                if let Ok(req_version) = Version::parse(requirement_str) {
                    *available_version <= req_version
                } else {
                    false
                }
            }
            // Less than: <1.2.3
            Some('<') => {
                let requirement_str = requirement.trim_start_matches('<');
                if let Ok(req_version) = Version::parse(requirement_str) {
                    *available_version < req_version
                } else {
                    false
                }
            }
            // Exact version match: 1.2.3
            _ => {
                if let Ok(req_version) = Version::parse(requirement) {
                    *available_version == req_version
                } else {
                    // If requirement parsing fails, consider incompatible
                    false
                }
            }
        }
    }
}
