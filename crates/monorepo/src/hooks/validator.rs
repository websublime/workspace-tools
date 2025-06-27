//! Hook validator for checking conditions and requirements
//!
//! The `HookValidator` evaluates hook conditions, validates changeset requirements,
//! and coordinates with other monorepo systems for comprehensive validation.

// Allow clippy lints during Phase 3 implementation - will be refined in Phase 4
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::unused_self)]

use super::{HookCondition, HookExecutionContext, HookValidationResult, ValidationCheck};
use super::types::{HookValidator, ChangesetValidationResult};
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use crate::{Environment};
use crate::changesets::Changeset;
use std::sync::Arc;

impl HookValidator {
    /// Create a new hook validator with injected dependencies
    #[must_use]
    pub fn new(
        git_provider: Box<dyn crate::core::GitProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
        config_provider: Box<dyn crate::core::ConfigProvider>,
    ) -> Self {
        Self { 
            git_provider,
            package_provider,
            config_provider,
        }
    }

    /// Create a new hook validator from project (convenience method)
    /// 
    /// NOTE: This convenience method creates provider instances from the project.
    /// For better performance and memory usage, prefer using the `new()` method with 
    /// pre-created providers when creating multiple components.
    #[must_use]
    pub fn from_project(project: Arc<MonorepoProject>) -> Self {
        use crate::core::interfaces::DependencyFactory;
        
        // Create providers efficiently
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider = DependencyFactory::config_provider(project);
        
        Self::new(
            git_provider,
            package_provider,
            config_provider,
        )
    }

    /// Check if all conditions are met for hook execution
    ///
    /// # Errors
    /// Returns an error if:
    /// - Git operations fail
    /// - File system operations fail
    /// - Condition evaluation encounters system errors
    pub fn check_conditions(
        &self,
        conditions: &[HookCondition],
        context: &HookExecutionContext,
    ) -> Result<bool> {
        for condition in conditions {
            if !self.evaluate_condition(condition, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Validate that changesets exist for affected packages
    ///
    /// # Errors
    /// Returns an error if:
    /// - Changeset storage cannot be accessed
    /// - Package information cannot be retrieved
    pub fn validate_changeset_exists(
        &self,
        affected_packages: &[String],
    ) -> Result<ChangesetValidationResult> {
        let mut result = ChangesetValidationResult::new();

        if affected_packages.is_empty() {
            return Ok(result.with_changeset_exists(true).with_validation_details(
                HookValidationResult::new()
                    .with_check("no_packages", ValidationCheck::passed("No packages affected")),
            ));
        }

        // Check if any of the affected packages have changesets
        // This would integrate with ChangesetManager when implemented
        let has_changeset = self.check_packages_have_changesets(affected_packages)?;

        result = result.with_changeset_exists(has_changeset);

        if has_changeset {
            // Try to find the specific changeset
            if let Ok(changeset) = self.find_changeset_for_packages(affected_packages) {
                result = result.with_changeset(changeset);
            }

            result = result.with_validation_details(HookValidationResult::new().with_check(
                "changeset_exists",
                ValidationCheck::passed("Changeset found for affected packages"),
            ));
        } else {
            let validation_details = HookValidationResult::new()
                .with_check(
                    "changeset_exists",
                    ValidationCheck::failed("No changeset found for affected packages"),
                )
                .with_required_action("Create a changeset for the affected packages");

            result = result.with_validation_details(validation_details);
        }

        Ok(result)
    }

    /// Validate branch naming conventions
    ///
    /// # Errors
    /// Returns an error if Git operations fail
    pub fn validate_branch_naming(&self, branch_name: &str) -> Result<ValidationCheck> {
        // This would check against configured branch naming patterns
        let patterns = self.get_branch_naming_patterns()?;

        for pattern in &patterns {
            if self.matches_pattern(branch_name, pattern) {
                return Ok(ValidationCheck::passed(format!(
                    "Branch name matches pattern: {pattern}"
                )));
            }
        }

        if patterns.is_empty() {
            return Ok(ValidationCheck::passed("No branch naming patterns configured"));
        }

        Ok(ValidationCheck::failed(format!(
            "Branch name '{branch_name}' does not match any configured patterns"
        )))
    }

    /// Validate commit message format
    ///
    /// # Errors
    /// Returns an error if commit information cannot be retrieved
    pub fn validate_commit_message(&self, commit_message: &str) -> Result<ValidationCheck> {
        // Check conventional commit format
        if self.is_conventional_commit(commit_message) {
            Ok(ValidationCheck::passed("Commit message follows conventional format"))
        } else {
            Ok(ValidationCheck::failed("Commit message does not follow conventional format")
                .with_details("Expected format: type(scope): description"))
        }
    }

    /// Validate file changes against patterns
    ///
    /// # Errors
    /// Returns an error if file pattern matching fails
    pub fn validate_file_changes(
        &self,
        changed_files: &[String],
        patterns: &[String],
    ) -> Result<ValidationCheck> {
        if patterns.is_empty() {
            return Ok(ValidationCheck::passed("No file patterns configured"));
        }

        let matched_files: Vec<&String> = changed_files
            .iter()
            .filter(|file| patterns.iter().any(|pattern| self.matches_pattern(file, pattern)))
            .collect();

        if matched_files.is_empty() {
            Ok(ValidationCheck::failed("No changed files match the required patterns"))
        } else {
            Ok(ValidationCheck::passed(format!("{} files match the patterns", matched_files.len())))
        }
    }

    /// Validate environment requirements
    ///
    /// # Errors
    /// Returns an error if environment variables cannot be accessed
    pub fn validate_environment(
        &self,
        required_env: &Environment,
        context: &HookExecutionContext,
    ) -> Result<ValidationCheck> {
        let current_env = self.detect_current_environment(context)?;

        if current_env == *required_env {
            Ok(ValidationCheck::passed(format!("Environment matches: {required_env:?}")))
        } else {
            Ok(ValidationCheck::failed(format!(
                "Environment mismatch. Expected: {required_env:?}, Current: {current_env:?}"
            )))
        }
    }

    // Private helper methods

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        condition: &HookCondition,
        context: &HookExecutionContext,
    ) -> Result<bool> {
        match condition {
            HookCondition::FilesChanged { patterns, match_any } => {
                self.evaluate_files_changed_condition(&context.changed_files, patterns, *match_any)
            }
            HookCondition::PackagesChanged { packages, match_any } => self
                .evaluate_packages_changed_condition(
                    &context.affected_packages,
                    packages,
                    *match_any,
                ),
            HookCondition::DependenciesChanged { dependency_types: _ } => {
                // Check if any dependency files have changed
                self.evaluate_dependencies_changed_condition(&context.changed_files)
            }
            HookCondition::OnBranch { pattern } => {
                Ok(self.matches_pattern(&context.current_branch, pattern))
            }
            HookCondition::Environment { env } => {
                let current_env = self.detect_current_environment(context)?;
                Ok(current_env == *env)
            }
            HookCondition::ChangesetExists { require_for_all: _ } => {
                self.check_packages_have_changesets(&context.affected_packages)
            }
            HookCondition::EnvironmentVariable { name, value } => {
                match context.get_env(name) {
                    Some(env_value) => {
                        if let Some(expected_value) = value {
                            Ok(env_value == expected_value)
                        } else {
                            Ok(true) // Just check if the variable exists
                        }
                    }
                    None => Ok(false),
                }
            }
            HookCondition::GitRefExists { ref_pattern } => self.check_git_ref_exists(ref_pattern),
        }
    }

    /// Evaluate files changed condition
    fn evaluate_files_changed_condition(
        &self,
        changed_files: &[String],
        patterns: &[String],
        match_any: bool,
    ) -> Result<bool> {
        if patterns.is_empty() {
            return Ok(true);
        }

        let matches: Vec<bool> = patterns
            .iter()
            .map(|pattern| changed_files.iter().any(|file| self.matches_pattern(file, pattern)))
            .collect();

        if match_any {
            Ok(matches.iter().any(|&m| m))
        } else {
            Ok(matches.iter().all(|&m| m))
        }
    }

    /// Evaluate packages changed condition
    fn evaluate_packages_changed_condition(
        &self,
        affected_packages: &[String],
        packages: &[String],
        match_any: bool,
    ) -> Result<bool> {
        if packages.is_empty() {
            return Ok(true);
        }

        let matches: Vec<bool> =
            packages.iter().map(|pkg| affected_packages.contains(pkg)).collect();

        if match_any {
            Ok(matches.iter().any(|&m| m))
        } else {
            Ok(matches.iter().all(|&m| m))
        }
    }

    /// Evaluate dependencies changed condition
    fn evaluate_dependencies_changed_condition(&self, changed_files: &[String]) -> Result<bool> {
        let dependency_files = ["package.json", "package-lock.json", "yarn.lock", "pnpm-lock.yaml"];

        Ok(changed_files
            .iter()
            .any(|file| dependency_files.iter().any(|dep_file| file.ends_with(dep_file))))
    }

    /// Check if packages have changesets
    fn check_packages_have_changesets(&self, packages: &[String]) -> Result<bool> {
        if packages.is_empty() {
            return Ok(true);
        }

        // Check configuration to see if changesets are required
        let config = self.config_provider.config();
        if !config.changesets.required {
            log::debug!("Changesets not required by configuration");
            return Ok(true);
        }

        // For now, this is a placeholder that checks if changeset requirement is disabled
        // Full changeset integration will be implemented in Phase 2
        log::warn!("Changeset validation requested but full integration not yet complete");
        log::warn!("To disable changeset requirements, set config.changesets.required = false");
        
        // Return false to indicate changesets are required but not found
        // This ensures the validation fails safely when changesets are expected
        Ok(false)
    }

    /// Find changeset for specific packages
    fn find_changeset_for_packages(&self, packages: &[String]) -> Result<Changeset> {
        if packages.is_empty() {
            return Err(Error::hook("No packages specified for changeset search"));
        }

        // This is a placeholder implementation for Phase 1
        // Full changeset integration will be implemented in Phase 2
        log::debug!("Changeset search requested for packages: {}", packages.join(", "));
        log::debug!("Full changeset integration pending - returning placeholder error");

        Err(Error::hook(format!(
            "Changeset search not yet fully implemented for packages: {}",
            packages.join(", ")
        )))
    }

    /// Get branch naming patterns from configuration
    fn get_branch_naming_patterns(&self) -> Result<Vec<String>> {
        let config = self.config_provider.config();
        let branch_config = &config.git.branches;
        
        let mut patterns = Vec::new();
        
        // Add feature branch patterns
        patterns.extend(branch_config.feature_prefixes.iter().map(|prefix| {
            if prefix.ends_with('/') {
                format!("{}*", prefix)
            } else {
                format!("{}/*", prefix)
            }
        }));
        
        // Add hotfix branch patterns  
        patterns.extend(branch_config.hotfix_prefixes.iter().map(|prefix| {
            if prefix.ends_with('/') {
                format!("{}*", prefix)
            } else {
                format!("{}/*", prefix)
            }
        }));
        
        // Add release branch patterns
        patterns.extend(branch_config.release_prefixes.iter().map(|prefix| {
            if prefix.ends_with('/') {
                format!("{}*", prefix)
            } else {
                format!("{}/*", prefix)
            }
        }));
        
        // Also include main and develop branches as valid patterns
        patterns.extend(branch_config.main_branches.clone());
        patterns.extend(branch_config.develop_branches.clone());
        
        Ok(patterns)
    }

    /// Check if a string matches a pattern using proper glob matching
    ///
    /// Uses the glob crate for standard glob pattern support including:
    /// - `*` matches any sequence of characters
    /// - `?` matches any single character
    /// - `[seq]` matches any character in seq
    /// - `[!seq]` matches any character not in seq
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        use glob::Pattern;
        
        // Create the glob pattern
        match Pattern::new(pattern) {
            Ok(glob_pattern) => glob_pattern.matches(text),
            Err(_) => {
                // If pattern is invalid, fall back to exact match
                text == pattern
            }
        }
    }

    /// Check if commit message follows conventional format
    fn is_conventional_commit(&self, message: &str) -> bool {
        let conventional_types = ["feat", "fix", "docs", "style", "refactor", "test", "chore"];

        for commit_type in &conventional_types {
            if message.starts_with(&format!("{commit_type}:"))
                || message.starts_with(&format!("{commit_type}("))
            {
                return true;
            }
        }

        false
    }

    /// Detect current environment from context
    fn detect_current_environment(&self, context: &HookExecutionContext) -> Result<Environment> {
        // Check environment variables and branch patterns to determine environment
        if let Some(env_var) = context.get_env("NODE_ENV") {
            match env_var.as_str() {
                "production" => return Ok(Environment::Production),
                "staging" => return Ok(Environment::Staging),
                "development" => return Ok(Environment::Development),
                _ => {}
            }
        }

        // Check branch patterns
        if context.current_branch.starts_with("main")
            || context.current_branch.starts_with("master")
        {
            Ok(Environment::Production)
        } else if context.current_branch.starts_with("staging")
            || context.current_branch.starts_with("stage")
        {
            Ok(Environment::Staging)
        } else {
            Ok(Environment::Development)
        }
    }

    /// Check if a Git reference exists
    fn check_git_ref_exists(&self, ref_pattern: &str) -> Result<bool> {
        let repository = self.git_provider.repository();
        
        // Handle glob patterns in references
        if ref_pattern.contains('*') || ref_pattern.contains('?') {
            // Get all branches and check if any match the pattern
            match repository.list_branches() {
                Ok(branches) => {
                    use glob::Pattern;
                    match Pattern::new(ref_pattern) {
                        Ok(pattern) => {
                            Ok(branches.iter().any(|branch_name| pattern.matches(branch_name)))
                        }
                        Err(_) => {
                            // Invalid pattern, fall back to exact match
                            Ok(branches.iter().any(|branch_name| branch_name == ref_pattern))
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to list Git branches: {}", e);
                    Ok(false)
                }
            }
        } else {
            // Check for exact branch/reference
            match repository.list_branches() {
                Ok(branches) => {
                    if branches.iter().any(|branch| branch == ref_pattern) {
                        return Ok(true);
                    }
                    
                    // If not a branch, check if it's a commit SHA by trying to get files changed
                    match repository.get_all_files_changed_since_sha(ref_pattern) {
                        Ok(_) => Ok(true),  // SHA exists
                        Err(_) => Ok(false), // SHA doesn't exist
                    }
                }
                Err(e) => {
                    log::warn!("Failed to check Git reference '{}': {}", ref_pattern, e);
                    Ok(false)
                }
            }
        }
    }

    /// Create a ChangesetManager instance for changeset operations
    ///
    /// This method creates a fully functional changeset manager with proper dependency injection.
    /// The changeset manager enables creation, management, and deployment of changesets
    /// within the hook validation context.
    ///
    /// # Returns
    ///
    /// A configured ChangesetManager instance ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if the changeset manager cannot be created due to:
    /// - Missing or invalid dependencies
    /// - File system or storage initialization issues
    /// - Configuration validation failures
    fn create_changeset_manager(&self) -> Result<crate::changesets::ChangesetManager> {
        use crate::changesets::ChangesetManager;
        use crate::core::MonorepoProject;
        use std::sync::Arc;

        // Get project root path from package provider
        let root_path = self.package_provider.root_path();
        
        // Create a new MonorepoProject instance from the root path
        // This is safe since the project structure is the same
        let project = Arc::new(MonorepoProject::new(root_path)
            .map_err(|e| Error::hook(format!("Failed to create project instance for changesets: {e}")))?);

        // Use the convenient from_project method which handles all dependency injection
        let changeset_manager = ChangesetManager::from_project(project)
            .map_err(|e| Error::hook(format!("Failed to create changeset manager from project: {e}")))?;

        log::debug!("Successfully created changeset manager for hook validation operations");
        Ok(changeset_manager)
    }
}

