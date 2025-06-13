//! Integration workflows that combine changesets with hooks
//!
//! This module provides workflows that integrate the changeset system
//! with Git hooks for seamless development experience.

use crate::changesets::{types::ChangesetFilter, ChangesetManager};
use crate::core::MonorepoProject;
use crate::error::Error;
use crate::hooks::HookManager;
use std::sync::Arc;
use sublime_standard_tools::filesystem::FileSystem;

/// Integration workflow that connects changesets with hooks
///
/// This workflow ensures that changesets are properly validated during
/// Git operations and provides seamless integration between the changeset
/// system and Git hooks.
// Import struct definition from types module
use crate::workflows::types::ChangesetHookIntegration;

impl ChangesetHookIntegration {
    /// Creates a new changeset-hook integration
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new integration instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required components cannot be initialized.
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self, Error> {
        let changeset_manager = ChangesetManager::new(Arc::clone(&project))?;
        let hook_manager = HookManager::new(Arc::clone(&project))?;
        let task_manager = crate::tasks::TaskManager::new(Arc::clone(&project))?;

        Ok(Self { project, changeset_manager, hook_manager, task_manager })
    }

    /// Validates that required changesets exist for the current changes
    ///
    /// This is called by pre-commit hooks to ensure that changes have
    /// appropriate changesets before being committed.
    ///
    /// # Returns
    ///
    /// True if changesets are valid or not required, false if changesets are missing.
    ///
    /// # Errors
    ///
    /// Returns an error if validation cannot be performed.
    pub fn validate_changesets_for_commit(&self) -> Result<bool, Error> {
        // Get current branch
        let current_branch = self
            .project
            .repository
            .get_current_branch()
            .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?;

        // Skip changeset validation for protected branches (main/develop)
        let branch_config = &self.project.config.git.branches;
        if branch_config.is_protected_branch(&current_branch) {
            return Ok(true);
        }

        // Check if changesets are required
        if !self.project.config.changesets.required {
            return Ok(true);
        }

        // Get staged files to determine affected packages
        let staged_files = self
            .project
            .repository
            .get_staged_files()
            .map_err(|e| Error::workflow(format!("Failed to get staged files: {e}")))?;

        if staged_files.is_empty() {
            return Ok(true);
        }

        // Map files to affected packages
        let affected_packages = self.map_files_to_packages(&staged_files);

        if affected_packages.is_empty() {
            return Ok(true);
        }

        // Check if changeset exists for this branch
        let filter = ChangesetFilter {
            branch: Some(current_branch.clone()),
            status: Some(crate::changesets::types::ChangesetStatus::Pending),
            ..Default::default()
        };

        let changesets = self.changeset_manager.list_changesets(&filter)?;

        if changesets.is_empty() {
            log::info!(
                "No changesets found for branch '{}' affecting packages: {:?}",
                current_branch,
                affected_packages
            );
            return Ok(false);
        }

        // Validate that changesets cover all affected packages
        let changeset_packages: std::collections::HashSet<String> =
            changesets.iter().map(|cs| cs.package.clone()).collect();

        let affected_packages_set: std::collections::HashSet<String> =
            affected_packages.into_iter().collect();

        let uncovered_packages: Vec<String> =
            affected_packages_set.difference(&changeset_packages).cloned().collect();

        if !uncovered_packages.is_empty() {
            log::warn!(
                "Packages affected by changes but not covered by changesets: {:?}",
                uncovered_packages
            );
            return Ok(false);
        }

        // Validate each changeset
        for changeset in &changesets {
            let validation = self.changeset_manager.validate_changeset(changeset)?;
            if !validation.is_valid {
                log::error!("Invalid changeset '{}': {:?}", changeset.id, validation.errors);
                return Ok(false);
            }
        }

        log::info!("All changesets validated successfully for branch '{}'", current_branch);
        Ok(true)
    }

    /// Prompts for changeset creation if needed
    ///
    /// This method is called when pre-commit validation fails due to
    /// missing changesets. It provides an interactive way to create
    /// the required changeset.
    ///
    /// # Returns
    ///
    /// True if a changeset was created or already exists, false if creation failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt process fails.
    pub fn prompt_for_changeset_if_needed(&self) -> Result<bool, Error> {
        // First check if changeset is actually needed
        let changeset_valid = self.validate_changesets_for_commit()?;

        if changeset_valid {
            return Ok(true);
        }

        // Get affected packages to inform the user
        let staged_files = self
            .project
            .repository
            .get_staged_files()
            .map_err(|e| Error::workflow(format!("Failed to get staged files: {e}")))?;
        let affected_packages = self.map_files_to_packages(&staged_files);

        log::info!("Changeset required for affected packages: {:?}", affected_packages);

        // Prompt for changeset creation
        match self.hook_manager.prompt_for_changeset() {
            Ok(changeset) => {
                log::info!(
                    "✅ Changeset '{}' created successfully for package '{}'",
                    changeset.id,
                    changeset.package
                );
                Ok(true)
            }
            Err(e) => {
                log::error!("❌ Failed to create changeset: {}", e);
                Ok(false)
            }
        }
    }

    /// Applies changesets when merging to main branches
    ///
    /// This method is called by post-merge hooks to automatically apply
    /// changesets when feature branches are merged to main branches.
    /// It also validates changesets before applying and handles dependency updates.
    ///
    /// # Arguments
    ///
    /// * `merged_branch` - The branch that was merged
    ///
    /// # Returns
    ///
    /// True if changesets were applied successfully or none were needed.
    ///
    /// # Errors
    ///
    /// Returns an error if changeset application fails.
    pub async fn apply_changesets_on_merge(&self, merged_branch: &str) -> Result<bool, Error> {
        // Get current branch (should be main/master after merge)
        let current_branch = self
            .project
            .repository
            .get_current_branch()
            .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?;

        // Only apply changesets when merging to protected branches
        let branch_config = &self.project.config.git.branches;
        if !branch_config.is_protected_branch(&current_branch) {
            log::info!(
                "Skipping changeset application - not on main branch (currently on '{}')",
                current_branch
            );
            return Ok(true);
        }

        // Check if there are any changesets for the merged branch
        let filter = crate::changesets::types::ChangesetFilter {
            branch: Some(merged_branch.to_string()),
            status: Some(crate::changesets::types::ChangesetStatus::Pending),
            ..Default::default()
        };

        let changesets = self.changeset_manager.list_changesets(&filter)?;

        if changesets.is_empty() {
            log::info!("No pending changesets found for merged branch '{}'", merged_branch);
            return Ok(true);
        }

        // Validate all changesets before applying any
        for changeset in &changesets {
            let validation = self.changeset_manager.validate_changeset(changeset)?;
            if !validation.is_valid {
                return Err(Error::workflow(format!(
                    "Cannot apply changeset '{}': validation failed with errors: {}",
                    changeset.id,
                    validation.errors.join(", ")
                )));
            }
        }

        // Apply changesets for the merged branch
        let applications = self.changeset_manager.apply_changesets_on_merge(merged_branch)?;

        if !applications.is_empty() {
            log::info!(
                "✅ Applied {} changeset(s) from branch '{}'",
                applications.len(),
                merged_branch
            );

            for application in &applications {
                if application.success {
                    log::info!(
                        "  ✅ {}: {} → {}",
                        application.package,
                        application.old_version,
                        application.new_version
                    );
                } else {
                    log::error!("  ❌ {}: failed to apply changeset", application.package);
                }
            }

            // Check if all applications were successful
            let failed_applications: Vec<_> =
                applications.iter().filter(|app| !app.success).collect();

            if !failed_applications.is_empty() {
                return Err(Error::workflow(format!(
                    "Failed to apply {} changeset(s)",
                    failed_applications.len()
                )));
            }

            // Run post-merge validation tasks if configured
            self.run_post_merge_validation(&applications).await?;
        }

        Ok(true)
    }

    /// Validates that all tests pass for affected packages before push
    ///
    /// This method is called by pre-push hooks to ensure that all
    /// affected packages have passing tests before pushing to remote.
    ///
    /// # Arguments
    ///
    /// * `commits` - List of commit hashes being pushed
    ///
    /// # Returns
    ///
    /// True if all tests pass or no packages are affected.
    ///
    /// # Errors
    ///
    /// Returns an error if test execution fails.
    pub async fn validate_tests_for_push(&self, commits: &[String]) -> Result<bool, Error> {
        if commits.is_empty() {
            return Ok(true);
        }

        // Get affected packages from commits
        let affected_packages = self.get_affected_packages_from_commits(commits)?;

        if affected_packages.is_empty() {
            return Ok(true);
        }

        // Run tests for affected packages using TaskManager
        let task_manager = crate::tasks::TaskManager::new(Arc::clone(&self.project))?;

        log::info!("🧪 Running tests for affected packages: {}", affected_packages.join(", "));

        // Execute test tasks for affected packages
        let test_results =
            task_manager.execute_tasks_for_affected_packages(&affected_packages).await?;

        // Check if all tests passed
        let failed_tests: Vec<_> = test_results
            .iter()
            .filter(|result| {
                !matches!(result.status, crate::tasks::types::results::TaskStatus::Success)
            })
            .collect();

        if !failed_tests.is_empty() {
            log::error!("❌ Tests failed for {} packages", failed_tests.len());
            for failed_test in &failed_tests {
                log::error!(
                    "  - {}: {}",
                    failed_test.task_name,
                    failed_test.errors.first().map_or("Unknown error", |e| &e.message)
                );
            }
            return Ok(false);
        }

        log::info!("✅ All tests passed!");
        Ok(true)
    }

    /// Sets up the complete integration between changesets and hooks
    ///
    /// This method installs all necessary Git hooks and configures them
    /// to work with the changeset system.
    ///
    /// # Returns
    ///
    /// True if setup completed successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if hook installation fails.
    #[allow(clippy::print_stdout)]
    pub fn setup_integration(&self) -> Result<bool, Error> {
        // Install Git hooks
        let installed_hooks = self.hook_manager.install_hooks()?;

        if installed_hooks.is_empty() {
            println!("⚠️  No hooks were installed");
            return Ok(false);
        }

        println!("✅ Installed {} Git hook(s)", installed_hooks.len());
        for hook_type in &installed_hooks {
            println!("  - {hook_type:?}");
        }

        // Verify changeset directory exists
        let changeset_dir =
            self.project.root_path().join(&self.project.config.changesets.changeset_dir);
        if !changeset_dir.exists() {
            self.project.file_system.create_dir_all(&changeset_dir).map_err(|e| {
                Error::workflow(format!("Failed to create changeset directory: {e}"))
            })?;
            println!("✅ Created changeset directory: {}", changeset_dir.display());
        }

        println!("🔗 Changeset-hook integration setup complete!");
        Ok(true)
    }

    /// Runs post-merge validation tasks after changesets are applied
    ///
    /// This ensures that applied changesets didn't break anything and that
    /// all packages are in a consistent state.
    async fn run_post_merge_validation(
        &self,
        applications: &[crate::changesets::types::ChangesetApplication],
    ) -> Result<(), Error> {
        log::info!("Running post-merge validation for {} applied changeset(s)", applications.len());

        // Get all affected packages
        let affected_packages: Vec<String> =
            applications.iter().map(|app| app.package.clone()).collect();

        if affected_packages.is_empty() {
            return Ok(());
        }

        // Create a temporary TaskManager for validation tasks
        let task_manager = crate::tasks::TaskManager::new(Arc::clone(&self.project))?;

        // Execute validation tasks for affected packages using TaskManager
        let validation_results =
            task_manager.execute_tasks_for_affected_packages(&affected_packages).await?;

        // Check if any validation tasks failed
        let failed_tasks: Vec<_> = validation_results
            .iter()
            .filter(|result| {
                !matches!(result.status, crate::tasks::types::results::TaskStatus::Success)
            })
            .collect();

        if !failed_tasks.is_empty() {
            let error_msg = failed_tasks
                .iter()
                .map(|task| {
                    format!(
                        "{}: {}",
                        task.task_name,
                        task.errors.first().map_or("Unknown error", |e| &e.message)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

            return Err(Error::workflow(format!("Post-merge validation failed: {error_msg}")));
        }

        // Run dependency graph validation
        self.validate_dependency_consistency(&affected_packages)?;

        log::info!("✅ Post-merge validation completed successfully");
        Ok(())
    }

    /// Validates that dependency versions are consistent across the monorepo
    ///
    /// Performs comprehensive dependency validation including:
    /// - Version compatibility checks between packages
    /// - Circular dependency detection
    /// - Missing dependency resolution
    /// - Version range consistency validation
    fn validate_dependency_consistency(&self, updated_packages: &[String]) -> Result<(), Error> {
        log::info!(
            "Validating dependency consistency for updated packages: {:?}",
            updated_packages
        );

        let mut validation_errors = Vec::new();

        // Check dependency consistency for each updated package
        for updated_package in updated_packages {
            log::debug!("Validating dependencies for package: {}", updated_package);

            // Find all packages that depend on this updated package
            let dependents = self.find_dependent_packages(updated_package)?;

            if dependents.is_empty() {
                log::debug!("Package '{}' has no dependents, skipping validation", updated_package);
                continue;
            }

            log::info!(
                "Package '{}' has {} dependent package(s): {:?}",
                updated_package,
                dependents.len(),
                dependents
            );

            // Get the current version of the updated package
            let updated_package_info =
                self.project.packages.iter().find(|pkg| pkg.name() == updated_package);

            if let Some(pkg_info) = updated_package_info {
                let current_version = pkg_info.version();

                // Validate that dependent packages can use this version
                for dependent in &dependents {
                    match self.validate_dependency_compatibility(
                        dependent,
                        updated_package,
                        current_version,
                    ) {
                        Ok(()) => {
                            log::debug!(
                                "✅ Dependency compatibility validated: {} -> {}",
                                dependent,
                                updated_package
                            );
                        }
                        Err(e) => {
                            validation_errors.push(format!(
                                "❌ Dependency incompatibility: {dependent} cannot use {updated_package} v{current_version}: {e}"
                            ));
                        }
                    }
                }
            } else {
                validation_errors.push(format!(
                    "❌ Updated package '{updated_package}' not found in project packages"
                ));
            }
        }

        // Check for circular dependencies in the entire project
        match self.detect_circular_dependencies() {
            Ok(()) => {
                log::debug!("✅ No circular dependencies detected");
            }
            Err(cycles) => {
                validation_errors.extend(cycles);
            }
        }

        // Report validation results
        if validation_errors.is_empty() {
            log::info!("✅ All dependency consistency checks passed");
            Ok(())
        } else {
            let error_message = format!(
                "Dependency validation failed with {} error(s):\n{}",
                validation_errors.len(),
                validation_errors.join("\n")
            );
            log::error!("{}", error_message);
            Err(Error::workflow(error_message))
        }
    }

    /// Finds packages that depend on the given package
    #[allow(clippy::unnecessary_wraps)]
    fn find_dependent_packages(&self, package: &str) -> Result<Vec<String>, Error> {
        let mut dependents = Vec::new();

        // Check all packages in the project
        for pkg in &self.project.packages {
            if pkg.name() == package {
                continue; // Skip self
            }

            // Read package.json to check dependencies
            let package_json_path = pkg.path().join("package.json");
            if let Ok(content) = self.project.file_system.read_file_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Check dependencies, devDependencies, and peerDependencies
                    let dep_sections = ["dependencies", "devDependencies", "peerDependencies"];

                    for section in &dep_sections {
                        if let Some(deps) = json[section].as_object() {
                            if deps.contains_key(package) {
                                dependents.push(pkg.name().to_string());
                                break; // No need to check other sections for this package
                            }
                        }
                    }
                }
            }
        }

        Ok(dependents)
    }

    /// Validates compatibility between a dependent package and its dependency
    fn validate_dependency_compatibility(
        &self,
        dependent_package: &str,
        dependency_package: &str,
        dependency_version: &str,
    ) -> Result<(), Error> {
        log::debug!(
            "Validating compatibility: {} depends on {} v{}",
            dependent_package,
            dependency_package,
            dependency_version
        );

        // Get the dependent package info
        let dependent_info =
            self.project.packages.iter().find(|pkg| pkg.name() == dependent_package).ok_or_else(
                || Error::workflow(format!("Dependent package '{dependent_package}' not found")),
            )?;

        // Read the dependent package's package.json to check version requirements
        let package_json_path = dependent_info.path().join("package.json");
        let content =
            self.project.file_system.read_file_string(&package_json_path).map_err(|e| {
                Error::workflow(format!("Failed to read package.json for {dependent_package}: {e}"))
            })?;

        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            Error::workflow(format!("Failed to parse package.json for {dependent_package}: {e}"))
        })?;

        // Check all dependency sections
        let dep_sections = ["dependencies", "devDependencies", "peerDependencies"];
        let mut required_version_range = None;

        for section in &dep_sections {
            if let Some(deps) = json[section].as_object() {
                if let Some(version_requirement) = deps.get(dependency_package) {
                    if let Some(version_str) = version_requirement.as_str() {
                        required_version_range = Some(version_str.to_string());
                        break;
                    }
                }
            }
        }

        let required_range = required_version_range.ok_or_else(|| {
            Error::workflow(format!(
                "Package '{dependent_package}' does not depend on '{dependency_package}'"
            ))
        })?;

        // Perform basic version compatibility checking
        // For a real implementation, you would use a proper semver library
        let is_compatible = self.check_version_compatibility(dependency_version, &required_range);

        if is_compatible {
            log::debug!(
                "✅ Version {} satisfies requirement {} for {} -> {}",
                dependency_version,
                required_range,
                dependent_package,
                dependency_package
            );
            Ok(())
        } else {
            Err(Error::workflow(format!(
                "Version {dependency_version} does not satisfy requirement {required_range} for dependency {dependent_package} -> {dependency_package}"
            )))
        }
    }

    /// Detects circular dependencies in the package dependency graph
    fn detect_circular_dependencies(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut visiting = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();

        // Check each package for cycles
        for pkg in &self.project.packages {
            let package_name = pkg.name().to_string();

            if !visited.contains(&package_name) {
                if let Err(cycle_errors) = self.detect_cycles_from_package(
                    &package_name,
                    &mut visiting,
                    &mut visited,
                    &mut Vec::new(),
                ) {
                    errors.extend(cycle_errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Recursive helper for cycle detection
    fn detect_cycles_from_package(
        &self,
        package_name: &str,
        visiting: &mut std::collections::HashSet<String>,
        visited: &mut std::collections::HashSet<String>,
        current_path: &mut Vec<String>,
    ) -> Result<(), Vec<String>> {
        if visiting.contains(package_name) {
            // Found a cycle
            let cycle_start = current_path.iter().position(|p| p == package_name).unwrap_or(0);
            let mut cycle: Vec<String> = current_path[cycle_start..].to_vec();
            cycle.push(package_name.to_string()); // Complete the cycle

            return Err(vec![format!("❌ Circular dependency detected: {}", cycle.join(" -> "))]);
        }

        if visited.contains(package_name) {
            return Ok(()); // Already processed
        }

        visiting.insert(package_name.to_string());
        current_path.push(package_name.to_string());

        // Get dependencies of this package
        let dependencies = self.get_package_dependencies(package_name);
        let mut all_errors = Vec::new();

        for dependency in dependencies {
            if let Err(cycle_errors) =
                self.detect_cycles_from_package(&dependency, visiting, visited, current_path)
            {
                all_errors.extend(cycle_errors);
            }
        }

        current_path.pop();
        visiting.remove(package_name);
        visited.insert(package_name.to_string());

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Gets the dependencies of a package
    fn get_package_dependencies(&self, package_name: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Find the package info
        if let Some(pkg_info) = self.project.packages.iter().find(|pkg| pkg.name() == package_name)
        {
            // Read package.json to get dependencies
            let package_json_path = pkg_info.path().join("package.json");
            if let Ok(content) = self.project.file_system.read_file_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Only check production dependencies for cycle detection
                    if let Some(deps) = json["dependencies"].as_object() {
                        for (dep_name, _) in deps {
                            // Only include internal monorepo dependencies
                            if self.project.packages.iter().any(|pkg| pkg.name() == dep_name) {
                                dependencies.push(dep_name.clone());
                            }
                        }
                    }
                }
            }
        }

        dependencies
    }

    /// Basic version compatibility checking
    ///
    /// This is a simplified implementation for demonstration.
    /// In a real scenario, you would use a proper semver library like `semver` crate.
    #[allow(clippy::manual_strip)]
    fn check_version_compatibility(&self, actual_version: &str, required_range: &str) -> bool {
        // Handle exact version matches
        if actual_version == required_range {
            return true;
        }

        // Handle common npm version range patterns
        if required_range.starts_with('^') {
            // Caret range: ^1.2.3 means >=1.2.3 and <2.0.0
            let required_version = &required_range[1..];
            return self.satisfies_caret_range(actual_version, required_version);
        }

        if required_range.starts_with('~') {
            // Tilde range: ~1.2.3 means >=1.2.3 and <1.3.0
            let required_version = &required_range[1..];
            return self.satisfies_tilde_range(actual_version, required_version);
        }

        if required_range.starts_with(">=") {
            let required_version = &required_range[2..];
            return self.compare_versions(actual_version, required_version) >= 0;
        }

        if required_range.starts_with('>') {
            let required_version = &required_range[1..];
            return self.compare_versions(actual_version, required_version) > 0;
        }

        if required_range.starts_with("<=") {
            let required_version = &required_range[2..];
            return self.compare_versions(actual_version, required_version) <= 0;
        }

        if required_range.starts_with('<') {
            let required_version = &required_range[1..];
            return self.compare_versions(actual_version, required_version) < 0;
        }

        // Default to false for unrecognized patterns
        log::warn!("Unrecognized version range pattern: {}", required_range);
        false
    }

    /// Check if version satisfies caret range (^1.2.3)
    fn satisfies_caret_range(&self, actual: &str, required: &str) -> bool {
        let actual_parts = self.parse_version(actual);
        let required_parts = self.parse_version(required);

        let Some((actual_major, actual_minor, actual_patch)) = actual_parts else { return false };
        let Some((req_major, req_minor, req_patch)) = required_parts else { return false };

        // Major version must match, minor and patch can be higher
        actual_major == req_major
            && (actual_minor > req_minor
                || (actual_minor == req_minor && actual_patch >= req_patch))
    }

    /// Check if version satisfies tilde range (~1.2.3)
    fn satisfies_tilde_range(&self, actual: &str, required: &str) -> bool {
        let actual_parts = self.parse_version(actual);
        let required_parts = self.parse_version(required);

        let Some((actual_major, actual_minor, actual_patch)) = actual_parts else { return false };
        let Some((req_major, req_minor, req_patch)) = required_parts else { return false };

        // Major and minor must match, patch can be higher
        actual_major == req_major && actual_minor == req_minor && actual_patch >= req_patch
    }

    /// Compare two versions (-1, 0, 1 for less, equal, greater)
    fn compare_versions(&self, version1: &str, version2: &str) -> i32 {
        let v1_parts = self.parse_version(version1);
        let v2_parts = self.parse_version(version2);

        let Some((v1_major, v1_minor, v1_patch)) = v1_parts else { return 0 };
        let Some((v2_major, v2_minor, v2_patch)) = v2_parts else { return 0 };

        if v1_major != v2_major {
            return if v1_major > v2_major { 1 } else { -1 };
        }
        if v1_minor != v2_minor {
            return if v1_minor > v2_minor { 1 } else { -1 };
        }
        if v1_patch != v2_patch {
            return if v1_patch > v2_patch { 1 } else { -1 };
        }

        0
    }

    /// Parse version string into (major, minor, patch)
    #[allow(clippy::unused_self)]
    fn parse_version(&self, version: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts[2].parse::<u32>().ok()?;

        Some((major, minor, patch))
    }

    /// Maps file paths to affected package names
    fn map_files_to_packages(&self, files: &[String]) -> Vec<String> {
        let mut affected_packages = Vec::new();

        for file_path in files {
            let full_path = self.project.root_path().join(file_path);

            if let Some(package) = self.project.descriptor.find_package_for_path(&full_path) {
                let package_name = package.name.clone();
                if !affected_packages.contains(&package_name) {
                    affected_packages.push(package_name);
                }
            }
        }

        affected_packages
    }

    /// Gets affected packages from commit hashes
    #[allow(clippy::unnecessary_wraps)]
    fn get_affected_packages_from_commits(&self, commits: &[String]) -> Result<Vec<String>, Error> {
        let mut all_affected_packages = Vec::new();

        for commit_hash in commits {
            match self.project.repository.get_all_files_changed_since_sha(commit_hash) {
                Ok(changed_files) => {
                    let affected_packages = self.map_files_to_packages(&changed_files);

                    for package in affected_packages {
                        if !all_affected_packages.contains(&package) {
                            all_affected_packages.push(package);
                        }
                    }
                }
                Err(_) => {
                    // Skip commits we can't analyze
                    continue;
                }
            }
        }

        Ok(all_affected_packages)
    }
}
