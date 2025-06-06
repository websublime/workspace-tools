//! Task condition checking implementation
//!
//! The ConditionChecker evaluates task conditions to determine if tasks should
//! be executed based on changes, environment, and other contextual factors.

// TODO: Remove these allows after Phase 4 implementation - currently needed for incomplete features
#![allow(clippy::unused_async)] // Will be fixed when async is properly implemented
#![allow(clippy::manual_strip)] // Will be fixed with proper glob library in Phase 4
#![allow(clippy::unnecessary_wraps)] // Will be fixed when error handling is complete
#![allow(dead_code)] // Will be fixed when all condition types are implemented

use crate::core::MonorepoProject;
use crate::analysis::ChangeAnalysis;
use crate::error::Result;
use crate::config::Environment;
use super::{
    TaskCondition, TaskDefinition, FilePattern, FilePatternType,
    EnvironmentCondition, BranchCondition,
    manager::ExecutionContext,
};
use super::types::{DependencyFilter, VersionChangeThreshold};
use std::collections::HashSet;
use std::sync::Arc;

/// Checker for evaluating task execution conditions
pub struct ConditionChecker {
    /// Reference to the monorepo project
    project: Arc<MonorepoProject>,
}

impl ConditionChecker {
    /// Create a new condition checker
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
        context.affected_packages = changes.package_changes
            .iter()
            .map(|pc| pc.package_name.clone())
            .collect();
        
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
    fn files_match_pattern(&self, files: &[sublime_git_tools::GitChangedFile], pattern: &FilePattern) -> bool {
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
                // Simple glob matching (could be enhanced with proper glob library)
                self.matches_glob_pattern(file, &pattern.pattern).unwrap_or(false)
            }
            FilePatternType::Regex => {
                // Would use regex crate in real implementation
                file.contains(&pattern.pattern)
            }
        };
        
        // Apply exclude logic
        Ok(if pattern.exclude { !matches } else { matches })
    }
    
    /// Simple glob pattern matching
    pub fn matches_glob_pattern(&self, text: &str, pattern: &str) -> Result<bool> {
        let result = if pattern.contains('*') {
            if let Some(prefix) = pattern.strip_suffix('*') {
                text.starts_with(prefix)
            } else if let Some(suffix) = pattern.strip_prefix('*') {
                text.ends_with(suffix)
            } else {
                // Handle patterns with * in the middle
                if let Some(star_pos) = pattern.find('*') {
                    let prefix = &pattern[..star_pos];
                    let suffix = &pattern[star_pos + 1..];
                    
                    // Check if text starts with prefix and ends with suffix
                    // and is long enough to contain both
                    text.starts_with(prefix) && 
                    text.ends_with(suffix) && 
                    text.len() >= prefix.len() + suffix.len()
                } else {
                    // Fallback - shouldn't reach here
                    text == pattern
                }
            }
        } else if pattern.contains('?') {
            // Handle ? wildcards (single character)
            if text.len() != pattern.len() {
                false
            } else {
                text.chars().zip(pattern.chars()).all(|(t_char, p_char)| {
                    p_char == '?' || p_char == t_char
                })
            }
        } else {
            text == pattern
        };
        Ok(result)
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
            BranchCondition::Equals(branch) => {
                Ok(current_branch == *branch)
            }
            
            BranchCondition::Matches(pattern) => {
                Ok(self.matches_glob_pattern(&current_branch, pattern).unwrap_or(false))
            }
            
            BranchCondition::OneOf(branches) => {
                Ok(branches.contains(&current_branch))
            }
            
            BranchCondition::NoneOf(branches) => {
                Ok(!branches.contains(&current_branch))
            }
            
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
                        Ok(env_value) => Ok(self.matches_glob_pattern(&env_value, pattern).unwrap_or(false)),
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
        let working_dir = context.working_directory
            .as_deref()
            .unwrap_or(self.project.root_path());
        
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
            let matches_include = packages_with_dep_changes.iter()
                .any(|pkg| filter.include.contains(pkg));
            if !matches_include {
                return Ok(false);
            }
        }
        
        // Check if any packages with dependency changes match the exclude filter
        if !filter.exclude.is_empty() {
            let matches_exclude = packages_with_dep_changes.iter()
                .any(|pkg| filter.exclude.contains(pkg));
            if matches_exclude {
                return Ok(false);
            }
        }
        
        // Check version change threshold
        self.check_version_change_threshold(&packages_with_dep_changes, &filter.version_change, context).await
    }
    
    /// Get packages with dependency changes
    fn get_packages_with_dependency_changes(&self, context: &ExecutionContext) -> Result<Vec<String>> {
        // In a real implementation, this would analyze package.json changes
        // For now, return affected packages as they likely have dependency changes
        Ok(context.affected_packages.clone())
    }
    
    /// Check version change threshold
    async fn check_version_change_threshold(
        &self,
        affected_packages: &[String],
        threshold: &VersionChangeThreshold,
        _context: &ExecutionContext,
    ) -> Result<bool> {
        // In a real implementation, this would check the actual version changes
        // For now, we'll implement basic logic
        
        match threshold {
            VersionChangeThreshold::Any => {
                Ok(!affected_packages.is_empty())
            }
            VersionChangeThreshold::Major => {
                // Would check for major version changes
                Ok(!affected_packages.is_empty())
            }
            VersionChangeThreshold::MinorOrMajor => {
                // Would check for minor or major changes
                Ok(!affected_packages.is_empty())
            }
            VersionChangeThreshold::PatchOrHigher => {
                // Would check for any version changes
                Ok(!affected_packages.is_empty())
            }
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
        let working_dir = context.working_directory
            .as_deref()
            .unwrap_or(self.project.root_path());
        
        let output = std::process::Command::new("sh")
            .arg(checker_path)
            .current_dir(working_dir)
            .output();
        
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
}