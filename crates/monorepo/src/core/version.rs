//! Version management for monorepo packages
//!
//! This module provides comprehensive version management capabilities including
//! version bumping, dependency propagation, and impact analysis.

use crate::config::VersionBumpType;
use crate::core::{
    BreakingChangeAnalysis, ConflictType, DefaultVersioningStrategy, DependencyChainImpact,
    MonorepoProject, MonorepoPackageInfo, PackageImpactAnalysis, PackageVersionUpdate, PropagationResult,
    VersionConflict, VersionImpactAnalysis, VersionManager, VersioningPlan, VersioningPlanStep,
    VersioningResult, VersioningStrategy,
};
use crate::error::Result;
use std::collections::HashMap;
use sublime_package_tools::{DependencyRegistry, Version};
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
            repository: &project.repository,
            file_system: &project.file_system,
            root_path: &project.root_path,
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
            repository: &project.repository,
            file_system: &project.file_system,
            root_path: &project.root_path,
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
        let package_info = self.packages
            .iter()
            .find(|pkg| pkg.name() == package_name)
            .ok_or_else(|| crate::error::Error::versioning(format!("Package '{package_name}' not found")))?;

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
        let dependents: Vec<&MonorepoPackageInfo> = self.packages
            .iter()
            .filter(|pkg| {
                pkg.dependencies_external.iter().any(|dep| dep == updated_package)
            })
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
                if !conflicts.iter().any(|c| c.conflict_type == ConflictType::DirtyWorkingDirectory) {
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

    /// Propagate version changes with enhanced dependency analysis
    ///
    /// FASE 2 ASYNC ELIMINATION: Removed artificial async behavior and delays.
    /// This function performs the same work as the previous async version but
    /// without unnecessary async infection or artificial delays.
    pub fn propagate_version_changes_enhanced(
        &self,
        updated_package: &str,
    ) -> Result<PropagationResult> {
        // Use the robust synchronous implementation without artificial delays
        self.propagate_version_changes(updated_package)
    }

    /// Propagate version changes asynchronously (compatibility wrapper)
    ///
    /// FASE 2 ASYNC ELIMINATION: This async function now delegates to the sync version
    /// to eliminate async infection while maintaining backward compatibility.
    #[allow(clippy::unused_async)]
    pub async fn propagate_version_changes_async(
        &self,
        updated_package: &str,
    ) -> Result<PropagationResult> {
        // Delegate to sync version - no actual async work needed, no artificial delays
        self.propagate_version_changes_enhanced(updated_package)
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
                    affected_dependents: self.packages
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
        let dependents: Vec<&MonorepoPackageInfo> = self.packages
            .iter()
            .filter(|pkg| {
                pkg.dependencies_external.iter().any(|dep| dep == package_name)
            })
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
        let dependents: Vec<&MonorepoPackageInfo> = self.packages
            .iter()
            .filter(|pkg| {
                pkg.dependencies_external.iter().any(|dep| dep == package_name)
            })
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
        let dependents: Vec<&MonorepoPackageInfo> = self.packages
            .iter()
            .filter(|pkg| {
                pkg.dependencies_external.iter().any(|dep| dep == package_name)
            })
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
        let dependents: Vec<&MonorepoPackageInfo> = self.packages
            .iter()
            .filter(|pkg| {
                pkg.dependencies_external.iter().any(|dep| dep == package_name)
            })
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

        let max_analysis_depth =
            self.config.validation.dependency_analysis.max_analysis_depth;
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
                current_version: self.packages
                    .iter()
                    .find(|pkg| pkg.name() == &package_change.package_name)
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
        let result = match bump_type {
            VersionBumpType::Major => Version::bump_major(current_version),
            VersionBumpType::Minor => Version::bump_minor(current_version),
            VersionBumpType::Patch => Version::bump_patch(current_version),
            VersionBumpType::Snapshot => {
                let sha = commit_sha.unwrap_or("unknown");
                Version::bump_snapshot(current_version, sha)
            }
        }
        .map_err(|e| crate::error::Error::versioning(format!("Version bump failed: {e}")))?;

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
    pub fn execute_versioning_plan(&self, plan: &VersioningPlan) -> Result<VersioningResult> {
        let mut primary_updates = Vec::new();
        let mut propagated_updates = Vec::new();
        let mut all_conflicts = plan.conflicts.clone();

        for step in &plan.steps {
            // Execute the version bump
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

        Ok(VersioningResult {
            primary_updates,
            propagated_updates,
            conflicts: all_conflicts,
            dependency_updates,
        })
    }

    /// Execute a versioning plan with enhanced progress tracking
    ///
    /// FASE 2 ASYNC ELIMINATION: Removed artificial async behavior and delays.
    /// This function performs the same work as the previous async version but
    /// without unnecessary async infection or artificial delays.
    pub fn execute_versioning_plan_enhanced(
        &self,
        plan: &VersioningPlan,
    ) -> Result<VersioningResult> {
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

            // Execute the version bump synchronously - no artificial delays needed
            let result = self.bump_package_version(&step.package_name, step.planned_version_bump, None)?;

            // Collect results
            primary_updates.extend(result.primary_updates);
            propagated_updates.extend(result.propagated_updates);
            all_conflicts.extend(result.conflicts);

            // No artificial delay - let the system manage resources naturally
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

    /// Execute a versioning plan asynchronously (compatibility wrapper)
    ///
    /// FASE 2 ASYNC ELIMINATION: This async function now delegates to the sync version
    /// to eliminate async infection while maintaining backward compatibility.
    #[allow(clippy::unused_async)]
    pub async fn execute_versioning_plan_async(
        &self,
        plan: &VersioningPlan,
    ) -> Result<VersioningResult> {
        // Delegate to sync version - no actual async work needed, no artificial delays
        self.execute_versioning_plan_enhanced(plan)
    }

    /// Get dependency update strategy for a package (placeholder implementation)
    pub fn get_dependency_update_strategy(&self, package_name: &str) -> Result<Vec<PackageVersionUpdate>> {
        let _package = self.packages.iter().find(|pkg| pkg.name() == package_name)
            .ok_or_else(|| crate::error::Error::package_not_found(package_name))?;

        // Placeholder implementation - will be enhanced when dependency analysis is ready
        log::info!("Dependency update strategy analysis for package: {}", package_name);
        
        Ok(Vec::new())
    }

    /// Validate version compatibility across all packages (placeholder implementation)
    pub fn validate_version_compatibility(&self) -> Result<Vec<VersionConflict>> {
        // Placeholder implementation - will be enhanced when dependency analysis is ready
        log::info!("Version compatibility validation across packages");
        
        Ok(Vec::new())
    }
}
