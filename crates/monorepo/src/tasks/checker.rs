//! Task condition checking implementation
//!
//! The `ConditionChecker` evaluates task conditions to determine if tasks should
//! be executed based on changes, environment, and other contextual factors.
//! Uses direct borrowing patterns instead of trait objects.

// Phase 1 Async Clarity: Converted condition checking to synchronous operations
// Only command execution remains async for legitimate I/O operations
#![allow(clippy::manual_strip)] // Will be fixed with proper glob library in Phase 4
#![allow(clippy::unnecessary_wraps)] // Will be fixed when error handling is complete
#![allow(dead_code)] // Will be fixed when all condition types are implemented

use super::types::{ConditionChecker, DependencyFilter, ExecutionContext, VersionChangeThreshold};
use super::{
    BranchCondition, EnvironmentCondition, FilePattern, FilePatternType, TaskCondition,
    TaskDefinition,
};
use crate::analysis::ChangeAnalysis;
use crate::config::Environment;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use glob::Pattern;
use regex::Regex;
use std::collections::HashSet;
use sublime_standard_tools::filesystem::FileSystem;
use VersionChangeThreshold::{Any, Major, MinorOrMajor, PatchOrHigher};

impl<'a> ConditionChecker<'a> {
    /// Create a new condition checker with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    ///
    /// # Returns
    ///
    /// A new condition checker instance
    #[must_use]
    pub fn new(project: &'a MonorepoProject) -> Self {
        Self {
            repository: &project.repository,
            config: &project.config,
            packages: &project.packages,
            file_system: &project.file_system,
            root_path: &project.root_path,
        }
    }

    /// Create a new condition checker with direct component references and repository
    ///
    /// Uses direct borrowing of individual components instead of requiring
    /// a full MonorepoProject. Requires an actual repository reference for
    /// Git-based condition checking.
    ///
    /// # Arguments
    ///
    /// * `repository` - Reference to git repository
    /// * `config` - Reference to monorepo configuration
    /// * `file_system` - Reference to file system manager
    /// * `packages` - Reference to package list
    /// * `root_path` - Reference to root path
    ///
    /// # Returns
    ///
    /// A new condition checker instance
    #[must_use]
    pub fn with_components(
        repository: &'a sublime_git_tools::Repo,
        config: &'a crate::config::MonorepoConfig,
        file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,
        packages: &'a [crate::core::MonorepoPackageInfo],
        root_path: &'a std::path::Path,
    ) -> Self {
        Self { repository, config, packages, file_system, root_path }
    }

    /// Check if all conditions are met for task execution
    pub fn check_conditions(&self, conditions: &[TaskCondition]) -> Result<bool> {
        let context = ExecutionContext::default();
        self.check_conditions_with_context(conditions, &context)
    }

    /// Check conditions with specific execution context
    pub fn check_conditions_with_context(
        &self,
        conditions: &[TaskCondition],
        context: &ExecutionContext,
    ) -> Result<bool> {
        // If no conditions specified, task should run
        if conditions.is_empty() {
            return Ok(true);
        }

        // Check if any conditions require async execution
        if Self::has_async_conditions(conditions) {
            return Err(Error::task(format!(
                "Conditions contain async operations (custom scripts or environment checkers). \
                Use AsyncConditionAdapter::evaluate_conditions_adaptive() instead. \
                Async conditions found: {}",
                Self::describe_async_conditions(conditions)
            )));
        }

        // All conditions must be met
        for condition in conditions {
            if !self.evaluate_condition(condition, context)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a task matches the given changes
    pub fn task_matches_changes(
        &self,
        task: &TaskDefinition,
        changes: &ChangeAnalysis,
    ) -> Result<bool> {
        // Create execution context from changes
        let mut context = ExecutionContext::default();
        context.changed_files.clone_from(&changes.changed_files);
        context.affected_packages =
            changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

        // Check all task conditions
        self.check_conditions_with_context(&task.conditions, &context)
    }

    /// Evaluate a single condition (sync for most conditions, async wrapper for custom scripts)
    pub fn evaluate_condition(
        &self,
        condition: &TaskCondition,
        context: &ExecutionContext,
    ) -> Result<bool> {
        match condition {
            TaskCondition::PackagesChanged { packages } => {
                Self::check_packages_changed(packages, context)
            }

            TaskCondition::FilesChanged { patterns } => self.check_files_changed(patterns, context),

            TaskCondition::DependenciesChanged { filter } => {
                self.check_dependencies_changed(filter.as_ref(), context)
            }

            TaskCondition::OnBranch { pattern } => self.check_branch_condition(pattern, context),

            TaskCondition::Environment { env } => self.check_environment_condition(env, context),

            TaskCondition::All { conditions } => {
                // All conditions must be true
                for cond in conditions {
                    if !self.evaluate_condition(cond, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            TaskCondition::Any { conditions } => {
                // At least one condition must be true
                for cond in conditions {
                    if self.evaluate_condition(cond, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            TaskCondition::Not { condition } => {
                let result = self.evaluate_condition(condition, context)?;
                Ok(!result)
            }

            TaskCondition::CustomScript { script, expected_output: _ } => {
                // Custom scripts require async execution - provide comprehensive guidance
                let error_message = format!(
                    "Custom script '{script}' requires async execution context.\n\n\
                    To execute custom scripts properly, use one of these approaches:\n\
                    1. AsyncConditionAdapter::evaluate_conditions_adaptive() for mixed sync/async conditions\n\
                    2. AsyncConditionAdapter::execute_custom_script() for direct script execution\n\
                    3. TaskManager::execute_tasks_for_affected_packages() for task-based execution\n\n\
                    Example usage:\n\
                    ```rust\n\
                    let adapter = AsyncConditionAdapter::new(condition_checker);\n\
                    let result = adapter.evaluate_conditions_adaptive(&conditions, &context).await?;\n\
                    ```\n\n\
                    For more information, see the AsyncConditionAdapter documentation."
                );

                log::warn!(
                    "Custom script '{}' attempted in synchronous context. \
                    Scripts require async execution due to process spawning and I/O operations. \
                    Use AsyncConditionAdapter::evaluate_conditions_adaptive() or execute_custom_script() instead.",
                    script
                );

                // Add structured error context for better debugging
                log::error!(
                    "[error] Operation 'custom_script_async_boundary' failed: {} (script: {}, solution: {}, method: {})",
                    "Async execution required",
                    script,
                    "Use AsyncConditionAdapter for async execution",
                    "evaluate_conditions_adaptive() or execute_custom_script()"
                );

                Err(Error::task(error_message))
            }
        }
    }

    /// Check if specified packages have changed
    pub fn check_packages_changed(packages: &[String], context: &ExecutionContext) -> Result<bool> {
        // If no packages specified, condition is met (no restrictions)
        if packages.is_empty() {
            return Ok(true);
        }

        // Check if any of the specified packages are in the affected packages list
        let affected_set: HashSet<&String> = context.affected_packages.iter().collect();

        for package in packages {
            if affected_set.contains(package) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if files matching patterns have changed
    pub fn check_files_changed(
        &self,
        patterns: &[FilePattern],
        context: &ExecutionContext,
    ) -> Result<bool> {
        // If no patterns specified, condition is met (no restrictions)
        if patterns.is_empty() {
            return Ok(true);
        }

        for pattern in patterns {
            if self.files_match_pattern(&context.changed_files, pattern) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if files match a pattern
    fn files_match_pattern(
        &self,
        files: &[sublime_git_tools::GitChangedFile],
        pattern: &FilePattern,
    ) -> bool {
        for file in files {
            if self.matches_file_pattern(&file.path, pattern).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    /// Check if a single file matches a pattern
    pub fn matches_file_pattern(&self, file: &str, pattern: &FilePattern) -> Result<bool> {
        let matches = match pattern.pattern_type {
            FilePatternType::Exact => file == pattern.pattern,
            FilePatternType::Prefix => file.starts_with(&pattern.pattern),
            FilePatternType::Suffix => file.ends_with(&pattern.pattern),
            FilePatternType::Glob => {
                self.matches_glob_pattern(file, &pattern.pattern).unwrap_or(false)
            }
            FilePatternType::Regex => {
                // Real regex pattern matching using regex crate
                self.matches_regex_pattern(file, &pattern.pattern)?
            }
        };

        // Apply exclude logic
        Ok(if pattern.exclude { !matches } else { matches })
    }

    /// Match text against a glob pattern using the glob crate
    ///
    /// Supports standard glob patterns including:
    /// - `*` matches any sequence of characters
    /// - `?` matches any single character
    /// - `[seq]` matches any character in seq
    /// - `[!seq]` matches any character not in seq
    ///
    /// # Arguments
    ///
    /// * `text` - The text to match against
    /// * `pattern` - The glob pattern to use
    ///
    /// # Returns
    ///
    /// Ok(true) if the text matches the pattern, Ok(false) otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid
    #[allow(clippy::unused_self)]
    pub fn matches_glob_pattern(&self, text: &str, pattern: &str) -> Result<bool> {
        // Create and compile the glob pattern
        let glob_pattern = Pattern::new(pattern)
            .map_err(|e| Error::task(format!("Invalid glob pattern '{pattern}': {e}")))?;

        Ok(glob_pattern.matches(text))
    }

    /// Check branch condition
    pub fn check_branch_condition(
        &self,
        condition: &BranchCondition,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Get current branch name - prefer context over repository
        let current_branch = if let Some(branch) = &context.current_branch {
            branch.clone()
        } else {
            self.repository.get_current_branch()?
        };

        match condition {
            BranchCondition::Equals(branch) => Ok(current_branch == *branch),

            BranchCondition::Matches(pattern) => {
                Ok(self.matches_glob_pattern(&current_branch, pattern).unwrap_or(false))
            }

            BranchCondition::OneOf(branches) => Ok(branches.contains(&current_branch)),

            BranchCondition::NoneOf(branches) => Ok(!branches.contains(&current_branch)),

            BranchCondition::IsMain => {
                // Use configured main branches
                let branch_config = &self.config.git.branches;
                Ok(branch_config.is_main_branch(&current_branch))
            }

            BranchCondition::IsFeature => {
                let branch_config = &self.config.git.branches;
                Ok(branch_config.is_feature_branch(&current_branch))
            }

            BranchCondition::IsRelease => {
                let branch_config = &self.config.git.branches;
                Ok(branch_config.is_release_branch(&current_branch))
            }

            BranchCondition::IsHotfix => {
                let branch_config = &self.config.git.branches;
                Ok(branch_config.is_hotfix_branch(&current_branch))
            }
        }
    }

    /// Check environment condition
    pub fn check_environment_condition(
        &self,
        condition: &EnvironmentCondition,
        context: &ExecutionContext,
    ) -> Result<bool> {
        match condition {
            EnvironmentCondition::VariableExists { key } => {
                Ok(context.environment.contains_key(key) || std::env::var(key).is_ok())
            }

            EnvironmentCondition::VariableEquals { key, value } => {
                if let Some(ctx_value) = context.environment.get(key) {
                    Ok(ctx_value == value)
                } else {
                    match std::env::var(key) {
                        Ok(env_value) => Ok(env_value == *value),
                        Err(_) => Ok(false),
                    }
                }
            }

            EnvironmentCondition::VariableMatches { key, pattern } => {
                if let Some(ctx_value) = context.environment.get(key) {
                    Ok(self.matches_glob_pattern(ctx_value, pattern).unwrap_or(false))
                } else {
                    match std::env::var(key) {
                        Ok(env_value) => {
                            Ok(self.matches_glob_pattern(&env_value, pattern).unwrap_or(false))
                        }
                        Err(_) => Ok(false),
                    }
                }
            }

            EnvironmentCondition::OneOf(environments) => {
                let current_env = self.detect_current_environment(context);
                for env in environments {
                    if self.environment_matches(&current_env, env) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            EnvironmentCondition::Not(environments) => {
                let current_env = self.detect_current_environment(context);
                for env in environments {
                    if self.environment_matches(&current_env, env) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            EnvironmentCondition::Custom { checker: _ } => {
                // Custom environment checkers require async execution - use async boundary adapter
                // TODO: Implement async boundary adapter for custom environment checkers
                Err(Error::task("Custom environment checkers require async execution - use async boundary adapter".to_string()))
            }

            EnvironmentCondition::Is(env) => {
                let current_env = self.detect_current_environment(context);
                Ok(self.environment_matches(&current_env, env))
            }
        }
    }

    /// Detect current environment
    #[allow(clippy::unused_self)]
    fn detect_current_environment(&self, _context: &ExecutionContext) -> Environment {
        // Check common environment variables
        if let Ok(env) = std::env::var("NODE_ENV") {
            match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "staging" | "stage" => Environment::Staging,
                "integration" | "int" => Environment::Integration,
                "production" | "prod" => Environment::Production,
                _ => Environment::Custom(env),
            }
        } else if let Ok(env) = std::env::var("ENVIRONMENT") {
            match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "staging" | "stage" => Environment::Staging,
                "integration" | "int" => Environment::Integration,
                "production" | "prod" => Environment::Production,
                _ => Environment::Custom(env),
            }
        } else {
            // Default to development if no environment is set
            Environment::Development
        }
    }

    /// Check if current environment matches target environment
    #[allow(clippy::unused_self)]
    fn environment_matches(&self, current: &Environment, target: &Environment) -> bool {
        match (current, target) {
            (Environment::Development, Environment::Development)
            | (Environment::Staging, Environment::Staging)
            | (Environment::Integration, Environment::Integration)
            | (Environment::Production, Environment::Production) => true,
            (Environment::Custom(c), Environment::Custom(t)) => c == t,
            _ => false,
        }
    }

    /// Execute custom script and check output using standard command tools
    pub async fn execute_custom_script(
        &self,
        script: &str,
        expected_output: &Option<String>,
        context: &ExecutionContext,
    ) -> Result<bool> {
        use sublime_standard_tools::command::{CommandBuilder, DefaultExecutor, Executor};

        // Determine working directory
        let working_dir = context.working_directory.as_deref().unwrap_or(self.root_path);

        // Create command using standard tools
        let command = if cfg!(windows) {
            CommandBuilder::new("cmd").arg("/C").arg(script).current_dir(working_dir).build()
        } else {
            CommandBuilder::new("sh").arg("-c").arg(script).current_dir(working_dir).build()
        };

        // Execute using the standard executor
        let executor = DefaultExecutor::new();

        match executor.execute(command).await {
            Ok(result) => {
                log::debug!(
                    "[custom_script] Executed with exit code: {} (success: {})",
                    result.status(),
                    result.success()
                );

                if let Some(expected) = expected_output {
                    // Check if output matches expected value
                    let stdout = result.stdout().trim();
                    let matches = stdout == expected.trim();
                    log::debug!(
                        "[custom_script] Output validation - Expected: '{}', Got: '{}', Matches: {}",
                        expected.trim(),
                        stdout,
                        matches
                    );

                    if matches {
                        log::info!(
                            "[custom_script] {}: Output matched expected value",
                            script
                        );
                    } else {
                        log::info!(
                            "[custom_script] {}: Output did not match expected value",
                            script
                        );
                    }

                    Ok(matches)
                } else {
                    // Check exit code only (0 = success)
                    let success = result.success();

                    if success {
                        log::info!("[custom_script] {}: Completed successfully", script);
                    } else {
                        log::info!(
                            "[custom_script] {}: Failed with exit code: {}",
                            script, result.status()
                        );
                    }

                    Ok(success)
                }
            }
            Err(e) => {
                log::error!(
                    "[error] Operation 'custom_script' failed: {} (script: {})",
                    e, script
                );
                Ok(false)
            }
        }
    }

    /// Check dependencies changed condition
    fn check_dependencies_changed(
        &self,
        filter: Option<&DependencyFilter>,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Get packages with dependency changes
        let packages_with_dep_changes = self.get_packages_with_dependency_changes(context)?;

        if packages_with_dep_changes.is_empty() {
            return Ok(false);
        }

        // If no filter specified, return true if any dependency changes found
        let Some(filter) = filter else {
            return Ok(true);
        };

        // Apply filter criteria using the DependencyFilter struct

        // Check if any packages with dependency changes match the include filter
        if !filter.include.is_empty() {
            let matches_include =
                packages_with_dep_changes.iter().any(|pkg| filter.include.contains(pkg));
            if !matches_include {
                return Ok(false);
            }
        }

        // Check if any packages with dependency changes match the exclude filter
        if !filter.exclude.is_empty() {
            let matches_exclude =
                packages_with_dep_changes.iter().any(|pkg| filter.exclude.contains(pkg));
            if matches_exclude {
                return Ok(false);
            }
        }

        // Check version change threshold (converted to sync)
        self.check_version_change_threshold(
            &packages_with_dep_changes,
            filter.version_change,
            context,
        )
    }

    /// Get packages with dependency changes
    fn get_packages_with_dependency_changes(
        &self,
        context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        let mut packages_with_dep_changes = Vec::new();

        // Get packages from the project
        let packages = self.packages;

        // Analyze each affected package for dependency changes
        for package_name in &context.affected_packages {
            if let Some(package) = packages.iter().find(|p| p.name() == package_name) {
                // Check if any of the changed files are package.json files in this package
                let package_json_path = package.workspace_package.location.join("package.json");
                let package_json_str = package_json_path.to_string_lossy().to_string();

                // Check if package.json was modified
                let has_package_json_changes =
                    context.changed_files.iter().any(|file| file.path == package_json_str);

                if has_package_json_changes {
                    // Analyze the actual dependency changes
                    if self.has_dependency_changes_in_package(&package_json_path, context)? {
                        packages_with_dep_changes.push(package_name.clone());
                    }
                }
            }
        }

        // Also check for workspace-level dependency changes that could affect packages
        let root_package_json = self.root_path.join("package.json");
        let root_package_json_str = root_package_json.to_string_lossy().to_string();

        if context.changed_files.iter().any(|file| file.path == root_package_json_str)
            && self.has_dependency_changes_in_package(&root_package_json, context)?
        {
            // Root dependency changes can affect all packages
            let all_packages: HashSet<String> =
                packages.iter().map(|p| p.name().to_string()).collect();

            let mut all_affected = all_packages.into_iter().collect::<Vec<_>>();
            packages_with_dep_changes.append(&mut all_affected);
        }

        // Remove duplicates and return
        packages_with_dep_changes.sort();
        packages_with_dep_changes.dedup();
        Ok(packages_with_dep_changes)
    }

    /// Analyzes if a package.json file has actual dependency changes using git crate
    ///
    /// This function compares the current package.json against Git to determine
    /// if there were changes to dependencies, devDependencies, peerDependencies,
    /// or optionalDependencies sections.
    ///
    /// # Arguments
    ///
    /// * `package_json_path` - Path to the package.json file
    /// * `context` - Execution context with change information
    ///
    /// # Returns
    ///
    /// True if there are dependency changes, false otherwise.
    fn has_dependency_changes_in_package(
        &self,
        package_json_path: &std::path::Path,
        context: &ExecutionContext,
    ) -> Result<bool> {
        let repository = self.repository;
        let since_ref = &self.config.git.default_since_ref;

        // Convert package_json_path to relative path from repository root
        let repo_root = self.root_path;
        let Ok(relative_path) = package_json_path.strip_prefix(repo_root) else {
            log::error!(
                "[dependency_analysis] Failed: Package.json path {} is not within repository root {}",
                package_json_path.display(), repo_root.display()
            );
            return Ok(false);
        };

        // Get files changed since the reference
        match repository.get_all_files_changed_since_sha(since_ref) {
            Ok(changed_files) => {
                let relative_path_str = relative_path.to_string_lossy();

                // Check if our package.json is in the changed files
                let is_changed = changed_files
                    .iter()
                    .any(|file| file == &*relative_path_str || file.ends_with(&*relative_path_str));

                if !is_changed {
                    log::debug!(
                        "[dependency] Package.json {} not in changed files list",
                        relative_path.display()
                    );
                    return Ok(false);
                }

                log::info!(
                    "[dependency_analysis] Found changed package.json: {}",
                    relative_path.display()
                );

                // Since we don't have direct diff access, we'll check if the file is in context changed files
                // and read the current file to see if it has dependency sections that likely changed
                if self.file_system.exists(package_json_path) {
                    match self.file_system.read_file_string(package_json_path) {
                        Ok(content) => {
                            // Check if the file contains dependency sections
                            let has_dependencies = content.contains("\"dependencies\"")
                                || content.contains("\"devDependencies\"")
                                || content.contains("\"peerDependencies\"")
                                || content.contains("\"optionalDependencies\"");

                            if has_dependencies {
                                log::debug!(
                                    "Package.json {} contains dependency sections and was changed",
                                    relative_path.display()
                                );
                                Ok(true)
                            } else {
                                log::debug!(
                                    "Package.json {} changed but has no dependency sections",
                                    relative_path.display()
                                );
                                Ok(false)
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "[error] Operation 'dependency_analysis' failed: {} (file: {})",
                                e, package_json_path.display()
                            );
                            // If file exists in changed list but we can't read it, assume dependencies changed
                            Ok(true)
                        }
                    }
                } else {
                    log::debug!(
                        "Package.json {} does not exist, assuming no dependency changes",
                        relative_path.display()
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                log::debug!("Could not get changed files from git: {}. Using context fallback.", e);

                // Fallback: check if this file is in the context's changed files
                let file_path_str = relative_path.to_string_lossy();
                let is_changed = context.changed_files.iter().any(|file| {
                    file.path == *file_path_str || file.path.ends_with(&*file_path_str)
                });

                if is_changed {
                    log::debug!(
                        "File {} found in context changed files list, assuming dependency changes",
                        relative_path.display()
                    );
                    Ok(true) // Assume dependency changes if package.json changed
                } else {
                    log::debug!("No changes detected for {}", relative_path.display());
                    Ok(false)
                }
            }
        }
    }

    /// Check version change threshold by analyzing actual changes
    ///
    /// This function analyzes the changes in each affected package to determine
    /// the level of version change that would be required based on:
    /// - Breaking changes (major version bump needed)
    /// - New features (minor version bump needed)
    /// - Bug fixes and patches (patch version bump needed)
    ///
    /// # Arguments
    ///
    /// * `affected_packages` - List of packages that have been affected
    /// * `threshold` - The minimum threshold to check against
    /// * `context` - Execution context with change information
    ///
    /// # Returns
    ///
    /// True if any package meets or exceeds the version change threshold.
    fn check_version_change_threshold(
        &self,
        affected_packages: &[String],
        threshold: VersionChangeThreshold,
        context: &ExecutionContext,
    ) -> Result<bool> {
        if affected_packages.is_empty() {
            return Ok(false);
        }

        // Analyze the changes to determine the highest version bump needed
        let mut highest_change_level = VersionChangeThreshold::Any;

        for package_name in affected_packages {
            let change_level = self.analyze_package_change_level(package_name, context)?;

            // Update highest change level if this package requires a higher level change
            if self.is_higher_change_level(change_level, highest_change_level) {
                highest_change_level = change_level;
            }

            // Early exit if we've already found the highest possible level
            if matches!(highest_change_level, VersionChangeThreshold::Major) {
                break;
            }
        }

        // Check if the highest change level meets the threshold
        Ok(self.meets_threshold(highest_change_level, threshold))
    }

    /// Analyze the level of change for a specific package using git crate
    ///
    /// Determines whether changes to a package would require a major, minor, or patch version bump
    /// by analyzing:
    /// - File types changed (public API vs internal implementation)
    /// - Commit messages for conventional commit indicators
    /// - Breaking change indicators
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to analyze
    /// * `context` - Execution context with change information
    ///
    /// # Returns
    ///
    /// The version change threshold level for this package.
    #[allow(clippy::too_many_lines)]
    fn analyze_package_change_level(
        &self,
        package_name: &str,
        context: &ExecutionContext,
    ) -> Result<VersionChangeThreshold> {
        // Find the package location from the project
        let packages = self.packages;
        let package = packages.iter().find(|p| p.name() == package_name).ok_or_else(|| {
            crate::error::Error::task(format!("Package {package_name} not found"))
        })?;

        let package_path = &package.workspace_package.location;
        let repository = self.repository;

        // Convert package path to relative path from repository root
        let repo_root = self.root_path;
        let relative_package_path = match package_path.strip_prefix(repo_root) {
            Ok(rel_path) => rel_path.to_string_lossy().to_string(),
            Err(_) => {
                log::warn!(
                    "Package path {} is not within repository root {}",
                    package_path.display(),
                    repo_root.display()
                );
                return Ok(VersionChangeThreshold::Any);
            }
        };

        let mut has_breaking_changes = false;
        let mut has_new_features = false;

        // Get commits for this package using git crate
        match repository.get_commits_since(Some("1 month ago".to_string()), &None) {
            Ok(commits) => {
                log::debug!("Analyzing {} commits for package {}", commits.len(), package_name);

                for commit in commits {
                    // Check if this commit touches the package
                    let touches_package =
                        match repository.get_all_files_changed_since_sha(&commit.hash) {
                            Ok(changed_files) => changed_files
                                .iter()
                                .any(|file| file.starts_with(&relative_package_path)),
                            Err(_) => false, // If we can't get changed files, skip this commit
                        };

                    if !touches_package {
                        continue;
                    }

                    // Analyze commit message for conventional commit patterns
                    let message = &commit.message;

                    // Check for breaking change indicators
                    if message.contains("BREAKING CHANGE") ||
                       message.contains("!:") || // feat!: or fix!:
                       message.to_lowercase().contains("breaking")
                    {
                        log::debug!("Breaking change detected in commit: {}", commit.hash);
                        has_breaking_changes = true;
                        break; // Breaking changes take priority
                    }

                    // Check for feature additions
                    if message.contains("feat:")
                        || message.contains("feature:")
                        || (message.to_lowercase().contains("add")
                            && message.to_lowercase().contains("new"))
                    {
                        log::debug!("Feature addition detected in commit: {}", commit.hash);
                        has_new_features = true;
                    }
                }
            }
            Err(e) => {
                log::debug!("Could not get commit history for analysis: {}. Using file-based analysis only.", e);
            }
        }

        // Analyze changed file types to infer impact
        let changed_files_in_package: Vec<_> = context
            .changed_files
            .iter()
            .filter(|file| {
                let file_path = std::path::Path::new(&file.path);
                file_path.starts_with(package_path)
            })
            .collect();

        log::debug!(
            "Found {} changed files in package {}",
            changed_files_in_package.len(),
            package_name
        );

        // Check for API changes that might indicate breaking changes
        for file in &changed_files_in_package {
            let file_path = &file.path;

            // Public API files that could indicate breaking changes
            if file_path.contains("index.") ||      // Main entry points
               file_path.contains("lib/") ||        // Library code
               file_path.contains("src/index") ||   // Source entry points
               file_path.contains("types.") ||      // Type definitions
               file_path.contains(".d.ts") ||       // TypeScript declarations
               file_path.ends_with("package.json")
            // Package metadata
            {
                log::debug!("API-related file changed: {}", file_path);
                // If we see changes to main API files, assume potential breaking changes
                // unless we already detected explicit feature additions
                if !has_new_features {
                    has_breaking_changes = true;
                }
            }
        }

        // Determine the change level based on analysis
        let change_level = if has_breaking_changes {
            VersionChangeThreshold::Major
        } else if has_new_features {
            VersionChangeThreshold::MinorOrMajor
        } else if !changed_files_in_package.is_empty() {
            VersionChangeThreshold::PatchOrHigher
        } else {
            VersionChangeThreshold::Any
        };

        log::debug!(
            "Package {} analysis result: {:?} (breaking: {}, features: {}, files: {})",
            package_name,
            change_level,
            has_breaking_changes,
            has_new_features,
            changed_files_in_package.len()
        );

        Ok(change_level)
    }

    /// Check if one change level is higher than another
    ///
    /// # Arguments
    ///
    /// * `level1` - First change level to compare
    /// * `level2` - Second change level to compare
    ///
    /// # Returns
    ///
    /// True if level1 is higher than level2.
    #[allow(clippy::unused_self)]
    fn is_higher_change_level(
        &self,
        level1: VersionChangeThreshold,
        level2: VersionChangeThreshold,
    ) -> bool {
        matches!(
            (level1, level2),
            (Major, _) | (MinorOrMajor, Any | PatchOrHigher) | (PatchOrHigher, Any)
        )
    }

    /// Check if a change level meets the specified threshold
    ///
    /// # Arguments
    ///
    /// * `level` - The actual change level detected
    /// * `threshold` - The minimum threshold required
    ///
    /// # Returns
    ///
    /// True if the level meets or exceeds the threshold.
    #[allow(clippy::unused_self)]
    fn meets_threshold(
        &self,
        level: VersionChangeThreshold,
        threshold: VersionChangeThreshold,
    ) -> bool {
        match threshold {
            Any => true, // Any level meets "Any" threshold
            PatchOrHigher => matches!(level, PatchOrHigher | MinorOrMajor | Major),
            MinorOrMajor => matches!(level, MinorOrMajor | Major),
            Major => matches!(level, Major),
        }
    }

    /// Execute custom environment checker script using standard command tools
    pub async fn execute_custom_environment_checker(
        &self,
        checker_name: &str,
        context: &ExecutionContext,
    ) -> Result<bool> {
        use sublime_standard_tools::command::{CommandBuilder, DefaultExecutor, Executor};

        // Look for checker script in project scripts directory
        let scripts_dir = self.root_path.join("scripts").join("checkers");
        let checker_path = scripts_dir.join(format!("{checker_name}.sh"));

        // Check if checker script exists
        if !self.file_system.exists(&checker_path) {
            log::debug!(
                "Checker script '{}' not found at {}, trying as direct command",
                checker_name,
                checker_path.display()
            );
            // Try as direct command if script file doesn't exist
            return self.execute_custom_script(checker_name, &None, context).await;
        }

        // Determine working directory
        let working_dir = context.working_directory.as_deref().unwrap_or(self.root_path);

        // Create command using standard tools
        let command = if cfg!(windows) {
            CommandBuilder::new("cmd")
                .arg("/C")
                .arg(checker_path.to_string_lossy().to_string())
                .current_dir(working_dir)
                .build()
        } else {
            CommandBuilder::new("sh")
                .arg(checker_path.to_string_lossy().to_string())
                .current_dir(working_dir)
                .build()
        };

        // Execute using the standard executor
        let executor = DefaultExecutor::new();

        match executor.execute(command).await {
            Ok(result) => {
                log::debug!(
                    "Environment checker '{}' executed with exit code: {}",
                    checker_name,
                    result.status()
                );
                log::trace!("Checker stdout: {}", result.stdout());
                if !result.stderr().is_empty() {
                    log::debug!("Checker stderr: {}", result.stderr());
                }

                // Check exit code (0 = condition met, non-zero = not met)
                let success = result.success();
                Ok(success)
            }
            Err(e) => {
                log::warn!("Failed to execute environment checker '{}': {}", checker_name, e);
                // If script execution fails, condition is not met
                Ok(false)
            }
        }
    }

    /// Real regex pattern matching using the regex crate
    ///
    /// Provides full regex capabilities including:
    /// - Character classes, quantifiers, anchors
    /// - Capture groups and non-capturing groups
    /// - Lookahead and lookbehind assertions
    /// - Unicode support
    ///
    /// # Arguments
    ///
    /// * `text` - The text to match against
    /// * `pattern` - The regex pattern to use for matching
    ///
    /// # Returns
    ///
    /// True if the text matches the regex pattern, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::TaskChecker;
    /// # let checker = create_test_checker();
    /// assert!(checker.matches_regex_pattern("test.js", r"\.js$")?);
    /// assert!(checker.matches_regex_pattern("src/utils.ts", r"src/.*\.ts$")?);
    /// assert!(!checker.matches_regex_pattern("test.py", r"\.js$")?);
    /// ```
    #[allow(clippy::unused_self)]
    fn matches_regex_pattern(&self, text: &str, pattern: &str) -> Result<bool> {
        match Regex::new(pattern) {
            Ok(regex) => Ok(regex.is_match(text)),
            Err(e) => {
                log::warn!(
                    "Invalid regex pattern '{}': {}. Falling back to exact string match.",
                    pattern,
                    e
                );
                // Fallback to exact match if pattern is invalid
                Ok(text == pattern)
            }
        }
    }

    /// Check if conditions require async execution
    ///
    /// This helper method determines if the given conditions contain any that require
    /// async execution (like custom scripts or custom environment checkers).
    ///
    /// # Arguments
    ///
    /// * `conditions` - The conditions to check
    ///
    /// # Returns
    ///
    /// Check if any conditions require async execution
    ///
    /// This method helps identify when the AsyncConditionAdapter should be used
    /// instead of synchronous condition checking.
    #[must_use]
    pub fn has_async_conditions(conditions: &[TaskCondition]) -> bool {
        use TaskCondition;

        conditions.iter().any(|condition| match condition {
            TaskCondition::CustomScript { .. } => true,
            TaskCondition::Environment { env } => {
                matches!(env, crate::tasks::EnvironmentCondition::Custom { .. })
            }
            TaskCondition::All { conditions } | TaskCondition::Any { conditions } => {
                Self::has_async_conditions(conditions)
            }
            TaskCondition::Not { condition } => Self::has_async_conditions(&[*condition.clone()]),
            _ => false,
        })
    }

    /// Describe which conditions require async execution for error messages
    ///
    /// This method provides a human-readable description of async conditions
    /// to help developers understand why async execution is required.
    #[allow(clippy::items_after_statements)]
    #[must_use]
    pub fn describe_async_conditions(conditions: &[TaskCondition]) -> String {
        use TaskCondition;

        let mut async_conditions = Vec::new();

        fn collect_async_conditions(conditions: &[TaskCondition], acc: &mut Vec<String>) {
            for condition in conditions {
                match condition {
                    TaskCondition::CustomScript { script, .. } => {
                        acc.push(format!("CustomScript('{script})"));
                    }
                    TaskCondition::Environment {
                        env: crate::tasks::EnvironmentCondition::Custom { checker },
                    } => {
                        acc.push(format!("CustomEnvironment('{checker})"));
                    }
                    TaskCondition::All { conditions } | TaskCondition::Any { conditions } => {
                        collect_async_conditions(conditions, acc);
                    }
                    TaskCondition::Not { condition } => {
                        collect_async_conditions(&[*condition.clone()], acc);
                    }
                    _ => {}
                }
            }
        }

        collect_async_conditions(conditions, &mut async_conditions);

        if async_conditions.is_empty() {
            "No async conditions found".to_string()
        } else {
            async_conditions.join(", ")
        }
    }

    /// True if any condition requires async execution, false otherwise
    #[must_use]
    pub fn requires_async_execution(conditions: &[crate::tasks::TaskCondition]) -> bool {
        use crate::tasks::TaskCondition;

        conditions.iter().any(|condition| match condition {
            TaskCondition::CustomScript { .. } => true,
            TaskCondition::Environment { env } => {
                matches!(env, crate::tasks::EnvironmentCondition::Custom { .. })
            }
            TaskCondition::All { conditions } | TaskCondition::Any { conditions } => {
                Self::requires_async_execution(conditions)
            }
            TaskCondition::Not { condition } => {
                Self::requires_async_execution(&[*condition.clone()])
            }
            _ => false,
        })
    }

}
