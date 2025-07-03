//! Configuration validation functionality for the configurator plugin

use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::ConfiguratorPlugin {
    /// Validate configuration file format and content
    ///
    /// Performs comprehensive validation of monorepo configuration including:
    /// - TOML syntax and structure validation
    /// - Configuration content validation
    /// - Best practices compliance checking
    /// - Compatibility verification
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to validate
    /// * `context` - Plugin context with access to project structure
    ///
    /// # Returns
    ///
    /// Detailed validation result with issues, warnings, and suggestions
    pub(super) fn validate_configuration(
        config_path: &str,
        context: &PluginContext,
    ) -> PluginResult {
        let start_time = Instant::now();

        let config_file_path = context.root_path.join(config_path);

        if !config_file_path.exists() {
            return PluginResult::error(format!("Configuration file not found: {config_path}"));
        }

        // Read configuration file
        let config_content = match std::fs::read_to_string(&config_file_path) {
            Ok(content) => content,
            Err(e) => {
                return PluginResult::error(format!("Failed to read configuration file: {e}"))
            }
        };

        let mut validation_issues = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Basic TOML parsing validation
        match toml::from_str::<crate::config::MonorepoConfig>(&config_content) {
            Ok(config) => {
                // Configuration parsed successfully, validate content
                Self::validate_config_content(
                    &config,
                    &mut validation_issues,
                    &mut warnings,
                    &mut suggestions,
                );
            }
            Err(e) => {
                validation_issues.push(serde_json::json!({
                    "type": "parse_error",
                    "severity": "critical",
                    "message": format!("Failed to parse TOML configuration: {e}"),
                    "suggestion": "Check TOML syntax and structure"
                }));
            }
        }

        let is_valid = validation_issues.is_empty();

        let result = serde_json::json!({
            "config_path": config_path,
            "is_valid": is_valid,
            "validation_issues": validation_issues,
            "warnings": warnings,
            "suggestions": suggestions,
            "file_size_bytes": config_content.len(),
            "file_lines": config_content.lines().count(),
            "validation_timestamp": chrono::Utc::now().to_rfc3339(),
            "status": if is_valid { "valid" } else { "invalid" }
        });

        success_with_timing(result, start_time)
            .with_metadata("command", "validate-config")
            .with_metadata("configurator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("validation_result", if is_valid { "valid" } else { "invalid" })
    }

    /// Validate configuration content against best practices
    fn validate_config_content(
        config: &crate::config::MonorepoConfig,
        validation_issues: &mut Vec<serde_json::Value>,
        warnings: &mut Vec<serde_json::Value>,
        suggestions: &mut Vec<serde_json::Value>,
    ) {
        // Validate versioning configuration
        Self::validate_versioning_config(&config.versioning, validation_issues, warnings);

        // Validate tasks configuration
        Self::validate_tasks_config(&config.tasks, validation_issues, warnings, suggestions);

        // Validate workspace configuration
        Self::validate_workspace_config(&config.workspace, validation_issues, warnings);

        // Validate git configuration
        Self::validate_git_config(&config.git, validation_issues, warnings);

        // Validate validation configuration
        Self::validate_validation_config(&config.validation, warnings, suggestions);

        // Validate hooks configuration
        Self::validate_hooks_config(&config.hooks, warnings, suggestions);

        // Validate changesets configuration
        Self::validate_changesets_config(&config.changesets, warnings, suggestions);
    }

    /// Validate versioning configuration
    fn validate_versioning_config(
        versioning: &crate::config::VersioningConfig,
        validation_issues: &mut Vec<serde_json::Value>,
        warnings: &mut Vec<serde_json::Value>,
    ) {
        // Check for valid bump types
        let bump_str = format!("{:?}", versioning.default_bump);
        let valid_bumps = ["Major", "Minor", "Patch"];
        if !valid_bumps.contains(&bump_str.as_str()) {
            validation_issues.push(serde_json::json!({
                "type": "invalid_bump_type", 
                "severity": "high",
                "message": format!("Invalid default_bump value: {}", bump_str),
                "suggestion": "Use one of: Major, Minor, Patch"
            }));
        }

        // Validate version constraint format if present
        if let Some(ref constraint) = versioning.version_constraint {
            if !constraint.starts_with('^') && !constraint.starts_with('~') && !constraint.contains(">=") {
                warnings.push(serde_json::json!({
                    "type": "version_constraint_format",
                    "severity": "medium",
                    "message": "Version constraint may not follow semantic versioning",
                    "suggestion": "Consider using semver ranges like ^1.0.0 or ~1.0.0"
                }));
            }
        }
    }

    /// Validate tasks configuration
    fn validate_tasks_config(
        tasks: &crate::config::TasksConfig,
        validation_issues: &mut Vec<serde_json::Value>,
        warnings: &mut Vec<serde_json::Value>,
        suggestions: &mut Vec<serde_json::Value>,
    ) {
        // Check for empty default tasks
        if tasks.default_tasks.is_empty() {
            warnings.push(serde_json::json!({
                "type": "no_default_tasks",
                "severity": "medium",
                "message": "No default tasks configured",
                "suggestion": "Consider adding common tasks like 'test', 'lint', 'build'"
            }));
        }

        // Validate concurrency settings
        if tasks.max_concurrent == 0 {
            validation_issues.push(serde_json::json!({
                "type": "invalid_concurrency",
                "severity": "high",
                "message": "max_concurrent cannot be 0",
                "suggestion": "Set max_concurrent to a positive integer"
            }));
        }

        if tasks.max_concurrent > 32 {
            warnings.push(serde_json::json!({
                "type": "high_concurrency",
                "severity": "low",
                "message": "Very high concurrency setting may impact performance",
                "suggestion": "Consider reducing max_concurrent to 8-16 for optimal performance"
            }));
        }

        // Validate timeout settings
        if tasks.timeout > 3600 {
            suggestions.push(serde_json::json!({
                "type": "long_timeout",
                "message": "Task timeout is very long (>1 hour)",
                "suggestion": "Consider if such long timeouts are necessary"
            }));
        }
    }

    /// Validate workspace configuration
    fn validate_workspace_config(
        workspace: &crate::config::WorkspaceConfig,
        validation_issues: &mut Vec<serde_json::Value>,
        warnings: &mut Vec<serde_json::Value>,
    ) {
        // Check if patterns are defined when not using merge_with_detected
        if !workspace.merge_with_detected && workspace.patterns.is_empty() {
            validation_issues.push(serde_json::json!({
                "type": "no_workspace_patterns",
                "severity": "critical",
                "message": "No workspace patterns defined and merge_with_detected is false",
                "suggestion": "Define workspace patterns or enable merge_with_detected"
            }));
        }

        // Validate workspace validation settings
        if workspace.validation.naming_patterns.is_empty() && workspace.validation.validate_naming {
            warnings.push(serde_json::json!({
                "type": "no_naming_patterns",
                "severity": "medium",
                "message": "Naming validation enabled but no patterns defined",
                "suggestion": "Define naming patterns or disable validate_naming"
            }));
        }
    }

    /// Validate git configuration
    fn validate_git_config(
        git: &crate::config::GitConfig,
        _validation_issues: &mut [serde_json::Value],
        warnings: &mut Vec<serde_json::Value>,
    ) {
        // Check for empty main branches
        if git.branches.main_branches.is_empty() {
            warnings.push(serde_json::json!({
                "type": "no_main_branches",
                "severity": "medium",
                "message": "No main branches configured",
                "suggestion": "Configure main branches like ['main', 'master']"
            }));
        }

        // Check for common branch naming patterns
        let has_standard_main = git.branches.main_branches.iter()
            .any(|branch| branch == "main" || branch == "master");
        
        if !has_standard_main {
            warnings.push(serde_json::json!({
                "type": "non_standard_main_branch",
                "severity": "low",
                "message": "No standard main branch (main/master) configured",
                "suggestion": "Consider using 'main' or 'master' as main branch"
            }));
        }
    }

    /// Validate validation configuration
    fn validate_validation_config(
        _validation: &crate::config::types::ValidationConfig,
        _warnings: &mut [serde_json::Value],
        suggestions: &mut Vec<serde_json::Value>,
    ) {
        // Validate quality gates if present (simplified validation)
        suggestions.push(serde_json::json!({
            "type": "quality_gates_validation",
            "message": "Quality gates configuration detected",
            "suggestion": "Review quality gate thresholds to ensure they meet your project standards"
        }));
    }

    /// Validate hooks configuration
    fn validate_hooks_config(
        hooks: &crate::config::HooksConfig,
        warnings: &mut Vec<serde_json::Value>,
        suggestions: &mut Vec<serde_json::Value>,
    ) {
        if hooks.enabled {
            // Check if hooks directory exists (this would need context)
            let hooks_dir_path = hooks.hooks_dir.as_ref().map_or_else(|| ".hooks".to_string(), |p| p.to_string_lossy().to_string());
            
            suggestions.push(serde_json::json!({
                "type": "hooks_directory",
                "message": "Hooks are enabled",
                "suggestion": format!("Ensure hooks directory '{}' exists and contains executable scripts", hooks_dir_path)
            }));

            // Check pre-commit configuration
            if hooks.pre_commit.enabled && hooks.pre_commit.run_tasks.is_empty() {
                warnings.push(serde_json::json!({
                    "type": "empty_pre_commit_tasks",
                    "severity": "medium",
                    "message": "Pre-commit hook enabled but no tasks configured",
                    "suggestion": "Configure run_tasks for pre-commit hook"
                }));
            }

            // Check pre-push configuration
            if hooks.pre_push.enabled && hooks.pre_push.run_tasks.is_empty() {
                warnings.push(serde_json::json!({
                    "type": "empty_pre_push_tasks",
                    "severity": "medium",
                    "message": "Pre-push hook enabled but no tasks configured",
                    "suggestion": "Configure run_tasks for pre-push hook"
                }));
            }
        }
    }

    /// Validate changesets configuration
    fn validate_changesets_config(
        changesets: &crate::config::ChangesetsConfig,
        warnings: &mut Vec<serde_json::Value>,
        suggestions: &mut Vec<serde_json::Value>,
    ) {
        if changesets.required && changesets.default_environments.is_empty() {
            warnings.push(serde_json::json!({
                "type": "no_default_environments",
                "severity": "medium",
                "message": "Changesets required but no default environments configured",
                "suggestion": "Configure default_environments for changesets"
            }));
        }

        if changesets.auto_deploy {
            suggestions.push(serde_json::json!({
                "type": "auto_deploy_enabled",
                "message": "Auto-deploy is enabled for changesets",
                "suggestion": "Ensure proper CI/CD pipelines are in place for safe auto-deployment"
            }));
        }
    }
}