//! Version management for monorepo packages
//!
//! This module provides comprehensive version management capabilities including
//! version bumping, dependency propagation, and impact analysis.

use crate::changes::ChangeSignificance;
use crate::config::VersionBumpType;
use crate::core::{
    AggressiveVersioningStrategy, BreakingChangeAnalysis, ConflictType,
    ConservativeVersioningStrategy, DefaultVersioningStrategy, DependencyChainImpact,
    MonorepoProject, PackageImpactAnalysis, PackageVersionUpdate, PropagationResult,
    VersionConflict, VersionImpactAnalysis, VersioningPlan, VersioningPlanStep, VersioningResult,
    VersionManager, VersioningStrategy,
};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use sublime_package_tools::{Version, DependencyRegistry};
// Import the diff_analyzer types for consistency
use crate::analysis::{ChangeAnalysis, PackageChange};

impl VersionManager {
    /// Create a new version manager with the default strategy
    #[must_use]
    pub fn new(project: Arc<MonorepoProject>) -> Self {
        Self { project, strategy: Box::new(DefaultVersioningStrategy) }
    }

    /// Create a version manager with a custom strategy
    #[must_use]
    pub fn with_strategy(
        project: Arc<MonorepoProject>,
        strategy: Box<dyn VersioningStrategy>,
    ) -> Self {
        Self { project, strategy }
    }

    /// Bump a package version with optional commit SHA for snapshots
    pub fn bump_package_version(
        &self,
        package_name: &str,
        bump_type: VersionBumpType,
        commit_sha: Option<&str>,
    ) -> Result<VersioningResult> {
        let package_info = self
            .project
            .get_package(package_name)
            .ok_or_else(|| crate::error::Error::package_not_found(package_name))?;

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
        let dependency_updates = dependency_registry.resolve_version_conflicts()
            .unwrap_or_else(|_| sublime_package_tools::ResolutionResult {
                resolved_versions: HashMap::new(),
                updates_required: Vec::new(),
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
        let conflicts = Vec::new();

        let dependents = self.project.get_dependents(updated_package);

        for dependent_pkg in dependents {
            let dependent_name = dependent_pkg.name();

            // Check if this dependent needs a version bump
            let bump_type =
                self.strategy.determine_bump_type_for_dependent(updated_package, dependent_name);

            if let Some(bump_type) = bump_type {
                let current_version = dependent_pkg.version();
                
                // DRY: Use the same version bumping logic as bump_package_version
                let new_version_str = self.perform_version_bump(current_version, bump_type, None)?;

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
                        .project
                        .get_dependents(&change.package_name)
                        .into_iter()
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
        let dependents = self.project.get_dependents(package_name);

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

        let dependents = self.project.get_dependents(package_name);
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

        self.build_dependency_chain(package_name, &mut chain, &mut visited, 0, 5); // Max depth 5

        DependencyChainImpact {
            root_package: package_name.to_string(),
            chain_length: chain.len(),
            affected_packages: chain,
            max_propagation_depth: self.calculate_chain_depth(package_name, 0, 5),
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

        let dependents = self.project.get_dependents(package_name);
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

        let dependents = self.project.get_dependents(package_name);
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

        for change in changes {
            let depth = self.calculate_chain_depth(&change.package_name, 0, 10);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Create a comprehensive versioning plan
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
                    .project
                    .get_package(&package_change.package_name)
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
        let per_package_duration = self.project.config.tasks.get_version_planning_per_package();
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
        if let Some(package_info) = self.project.get_package(package_name) {
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

    /// Calculate execution order for version plan steps
    fn calculate_execution_order(steps: &mut [VersioningPlanStep]) {
        // Simple ordering for now - can be enhanced with topological sort
        for (index, step) in steps.iter_mut().enumerate() {
            step.execution_order = index;
        }
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
        let dependency_updates = dependency_registry.resolve_version_conflicts()
            .unwrap_or_else(|_| sublime_package_tools::ResolutionResult {
                resolved_versions: HashMap::new(),
                updates_required: Vec::new(),
            });

        Ok(VersioningResult {
            primary_updates,
            propagated_updates,
            conflicts: all_conflicts,
            dependency_updates,
        })
    }
}


