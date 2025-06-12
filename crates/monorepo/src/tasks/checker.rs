//! Task condition checking implementation
//!
//! The `ConditionChecker` evaluates task conditions to determine if tasks should
//! be executed based on changes, environment, and other contextual factors.

// TODO: Remove these allows after Phase 4 implementation - currently needed for incomplete features
#![allow(clippy::unused_async)] // Will be fixed when async is properly implemented
#![allow(clippy::manual_strip)] // Will be fixed with proper glob library in Phase 4
#![allow(clippy::unnecessary_wraps)] // Will be fixed when error handling is complete
#![allow(dead_code)] // Will be fixed when all condition types are implemented

use super::types::{ConditionChecker, DependencyFilter, ExecutionContext, VersionChangeThreshold};
use super::{
    BranchCondition, EnvironmentCondition, FilePattern, FilePatternType,
    TaskCondition, TaskDefinition,
};
use crate::analysis::ChangeAnalysis;
use crate::config::Environment;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use glob::Pattern;
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use VersionChangeThreshold::{Any, Major, MinorOrMajor, PatchOrHigher};

impl ConditionChecker {
    /// Create a new condition checker
    #[must_use]
    pub fn new(project: Arc<MonorepoProject>) -> Self {
        Self { project }
    }

    /// Check if all conditions are met for task execution
    pub async fn check_conditions(&self, conditions: &[TaskCondition]) -> Result<bool> {
        let context = ExecutionContext::default();
        self.check_conditions_with_context(conditions, &context).await
    }

    /// Check conditions with specific execution context
    pub async fn check_conditions_with_context(
        &self,
        conditions: &[TaskCondition],
        context: &ExecutionContext,
    ) -> Result<bool> {
        // If no conditions specified, task should run
        if conditions.is_empty() {
            return Ok(true);
        }

        // All conditions must be met
        for condition in conditions {
            if !self.evaluate_condition(condition, context).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a task matches the given changes
    pub async fn task_matches_changes(
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
        self.check_conditions_with_context(&task.conditions, &context).await
    }

    /// Evaluate a single condition
    fn evaluate_condition<'a>(
        &'a self,
        condition: &'a TaskCondition,
        context: &'a ExecutionContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + 'a>> {
        Box::pin(async move {
            match condition {
                TaskCondition::PackagesChanged { packages } => {
                    self.check_packages_changed(packages, context).await
                }

                TaskCondition::FilesChanged { patterns } => {
                    self.check_files_changed(patterns, context).await
                }

                TaskCondition::DependenciesChanged { filter } => {
                    self.check_dependencies_changed(filter.as_ref(), context).await
                }

                TaskCondition::OnBranch { pattern } => {
                    self.check_branch_condition(pattern, context).await
                }

                TaskCondition::Environment { env } => {
                    self.check_environment_condition(env, context).await
                }

                TaskCondition::All { conditions } => {
                    // All conditions must be true
                    for cond in conditions {
                        if !self.evaluate_condition(cond, context).await? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }

                TaskCondition::Any { conditions } => {
                    // At least one condition must be true
                    for cond in conditions {
                        if self.evaluate_condition(cond, context).await? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }

                TaskCondition::Not { condition } => {
                    let result = self.evaluate_condition(condition, context).await?;
                    Ok(!result)
                }

                TaskCondition::CustomScript { script, expected_output } => {
                    self.execute_custom_script(script, expected_output, context).await
                }
            }
        })
    }

    /// Check if specified packages have changed
    pub async fn check_packages_changed(
        &self,
        packages: &[String],
        context: &ExecutionContext,
    ) -> Result<bool> {
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
    pub async fn check_files_changed(
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
    pub async fn check_branch_condition(
        &self,
        condition: &BranchCondition,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Get current branch name - prefer context over repository
        let current_branch = if let Some(branch) = &context.current_branch {
            branch.clone()
        } else {
            self.project.repository.get_current_branch()?
        };

        match condition {
            BranchCondition::Equals(branch) => Ok(current_branch == *branch),

            BranchCondition::Matches(pattern) => {
                Ok(self.matches_glob_pattern(&current_branch, pattern).unwrap_or(false))
            }

            BranchCondition::OneOf(branches) => Ok(branches.contains(&current_branch)),

            BranchCondition::NoneOf(branches) => Ok(!branches.contains(&current_branch)),

            BranchCondition::IsMain => {
                // Common main branch names
                let main_branches = ["main", "master", "develop", "trunk"];
                Ok(main_branches.contains(&current_branch.as_str()))
            }

            BranchCondition::IsFeature => {
                Ok(current_branch.starts_with("feature/") || current_branch.starts_with("feat/"))
            }

            BranchCondition::IsRelease => {
                Ok(current_branch.starts_with("release/") || current_branch.starts_with("rel/"))
            }

            BranchCondition::IsHotfix => {
                Ok(current_branch.starts_with("hotfix/") || current_branch.starts_with("fix/"))
            }
        }
    }

    /// Check environment condition
    pub async fn check_environment_condition(
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

            EnvironmentCondition::Custom { checker } => {
                // Execute custom environment checker script/command
                self.execute_custom_environment_checker(checker, context).await
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

    /// Execute custom script and check output
    async fn execute_custom_script(
        &self,
        script: &str,
        expected_output: &Option<String>,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Determine working directory
        let working_dir = context.working_directory.as_deref().unwrap_or(self.project.root_path());

        // Execute script using shell
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(working_dir)
            .output();

        match output {
            Ok(result) => {
                if let Some(expected) = expected_output {
                    // Check if output matches expected value
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    Ok(stdout.trim() == expected.trim())
                } else {
                    // Check exit code only
                    Ok(result.status.success())
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Check dependencies changed condition
    async fn check_dependencies_changed(
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

        // Check version change threshold
        self.check_version_change_threshold(
            &packages_with_dep_changes,
            &filter.version_change,
            context,
        )
        .await
    }

    /// Get packages with dependency changes
    fn get_packages_with_dependency_changes(
        &self,
        context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        let mut packages_with_dep_changes = Vec::new();

        // Get packages from the project
        let packages = &self.project.packages;

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
        let root_package_json = self.project.root_path().join("package.json");
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

    /// Analyzes if a package.json file has actual dependency changes
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
        _context: &ExecutionContext,
    ) -> Result<bool> {
        use std::process::Command;

        // Get the git diff for this specific file
        let git_diff_output = Command::new("git")
            .args([
                "diff",
                "HEAD~1", // Compare with previous commit
                "--",
                &package_json_path.to_string_lossy(),
            ])
            .current_dir(self.project.root_path())
            .output();

        match git_diff_output {
            Ok(output) if output.status.success() => {
                let diff_content = String::from_utf8_lossy(&output.stdout);

                // Check if the diff contains dependency-related changes
                let has_dep_changes = diff_content.lines().any(|line| {
                    // Look for added or removed lines that contain dependency keys
                    (line.starts_with('+') || line.starts_with('-'))
                        && !line.starts_with("+++")
                        && !line.starts_with("---")
                        && (line.contains("\"dependencies\"")
                            || line.contains("\"devDependencies\"")
                            || line.contains("\"peerDependencies\"")
                            || line.contains("\"optionalDependencies\"")
                            || (line.contains(':')
                                && (
                                    line.contains('@') || // Package scopes
                         line.contains('^') || // Version ranges
                         line.contains('~') || // Version ranges
                         line.contains(">=") || // Version ranges
                         line.contains("<=") || // Version ranges
                         line.contains("file:") || // Local packages
                         line.contains("workspace:")
                                    // Workspace references
                                )))
                });

                Ok(has_dep_changes)
            }
            Ok(_) => {
                // Git command failed or file not tracked
                log::debug!(
                    "Could not get git diff for {}, assuming no dependency changes",
                    package_json_path.display()
                );
                Ok(false)
            }
            Err(e) => {
                log::warn!("Failed to execute git diff for {}: {}", package_json_path.display(), e);
                Ok(false)
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
    async fn check_version_change_threshold(
        &self,
        affected_packages: &[String],
        threshold: &VersionChangeThreshold,
        context: &ExecutionContext,
    ) -> Result<bool> {
        if affected_packages.is_empty() {
            return Ok(false);
        }

        // Analyze the changes to determine the highest version bump needed
        let mut highest_change_level = VersionChangeThreshold::Any;

        for package_name in affected_packages {
            let change_level = self.analyze_package_change_level(package_name, context).await?;

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
        Ok(self.meets_threshold(highest_change_level, *threshold))
    }

    /// Analyze the level of change for a specific package
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
    async fn analyze_package_change_level(
        &self,
        package_name: &str,
        context: &ExecutionContext,
    ) -> Result<VersionChangeThreshold> {
        use std::process::Command;

        // Find the package location from the project
        let packages = &self.project.packages;
        let package = packages.iter().find(|p| p.name() == package_name).ok_or_else(|| {
            crate::error::Error::task(format!("Package {package_name} not found"))
        })?;

        let package_path = &package.workspace_package.location;

        // Get the Git log for this package to analyze commit messages
        let git_log_output = Command::new("git")
            .args([
                "log",
                "--oneline",
                "--since=1 month ago", // Look at recent commits
                "--",
                &package_path.to_string_lossy(),
            ])
            .current_dir(self.project.root_path())
            .output();

        let mut has_breaking_changes = false;
        let mut has_new_features = false;

        // Analyze commit messages for conventional commit patterns
        if let Ok(output) = git_log_output {
            let log_content = String::from_utf8_lossy(&output.stdout);

            for line in log_content.lines() {
                // Check for breaking change indicators
                if line.contains("BREAKING CHANGE") ||
                   line.contains("!:") || // feat!: or fix!:
                   line.starts_with("* ") && line.to_lowercase().contains("breaking")
                {
                    has_breaking_changes = true;
                    break; // Breaking changes take priority
                }

                // Check for feature additions
                if line.contains("feat:")
                    || line.contains("feature:")
                    || line.to_lowercase().contains("add") && line.to_lowercase().contains("new")
                {
                    has_new_features = true;
                }
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
            {
                // Package metadata

                // If we see changes to main API files, assume potential breaking changes
                // unless we already detected explicit feature additions
                if !has_new_features {
                    has_breaking_changes = true;
                }
            }
        }

        // Determine the change level based on analysis
        if has_breaking_changes {
            Ok(VersionChangeThreshold::Major)
        } else if has_new_features {
            Ok(VersionChangeThreshold::MinorOrMajor)
        } else if !changed_files_in_package.is_empty() {
            Ok(VersionChangeThreshold::PatchOrHigher)
        } else {
            Ok(VersionChangeThreshold::Any)
        }
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

    /// Execute custom environment checker script
    async fn execute_custom_environment_checker(
        &self,
        checker_name: &str,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Look for checker script in project scripts directory
        let scripts_dir = self.project.root_path().join("scripts").join("checkers");
        let checker_path = scripts_dir.join(format!("{checker_name}.sh"));

        // Check if checker script exists
        if !checker_path.exists() {
            // Try as direct command if script file doesn't exist
            return self.execute_custom_script(checker_name, &None, context).await;
        }

        // Execute the checker script
        let working_dir = context.working_directory.as_deref().unwrap_or(self.project.root_path());

        let output =
            std::process::Command::new("sh").arg(checker_path).current_dir(working_dir).output();

        match output {
            Ok(result) => {
                // Check exit code (0 = condition met, non-zero = not met)
                Ok(result.status.success())
            }
            Err(_) => {
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
}
