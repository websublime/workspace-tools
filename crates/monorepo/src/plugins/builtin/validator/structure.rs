//! Structure validation functionality for the validator plugin

use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::ValidatorPlugin {
    /// Validate monorepo structure using real filesystem analysis
    ///
    /// Performs comprehensive validation of the monorepo structure including:
    /// - Package discovery and validation
    /// - Directory structure compliance
    /// - Configuration file presence and validity
    /// - Workspace consistency
    ///
    /// # Arguments
    ///
    /// * `context` - Plugin context with access to file system and packages
    ///
    /// # Returns
    ///
    /// Detailed validation result with issues and recommendations
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_structure(context: &PluginContext) -> PluginResult {
        let start_time = Instant::now();

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();

        // 1. Validate root structure - only check package.json as it's essential for monorepos
        let package_json_path = context.root_path.join("package.json");
        if !package_json_path.exists() {
            issues.push("Missing required root file: package.json".to_string());
            recommendations.push("Create package.json in the root directory".to_string());
        }

        // 2. Validate package structure
        if context.packages.is_empty() {
            issues.push("No packages found in monorepo".to_string());
            recommendations.push(
                "Add packages to the packages/ directory or verify package discovery configuration"
                    .to_string(),
            );
        } else {
            for package in context.packages {
                // Check if package has required files
                let package_json_path = package.path().join("package.json");
                if !package_json_path.exists() {
                    issues.push(format!("Package '{}' missing package.json", package.name()));
                }

                // Check if package has source directory
                let src_path = package.path().join("src");
                if !src_path.exists() {
                    warnings.push(format!("Package '{}' has no src/ directory", package.name()));
                    recommendations.push(format!(
                        "Consider adding src/ directory to package '{}'",
                        package.name()
                    ));
                }

                // Validate package name convention
                let package_name = package.name();
                if !package_name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/')
                {
                    warnings.push(format!("Package '{package_name}' has non-standard name format"));
                }
            }
        }

        // 3. Check for workspace configuration consistency
        let workspace_config_path = context.root_path.join("package.json");
        if workspace_config_path.exists() {
            match std::fs::read_to_string(&workspace_config_path) {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json) => {
                            // Check for workspaces configuration
                            if json.get("workspaces").is_none() {
                                warnings.push(
                                    "Root package.json missing workspaces configuration"
                                        .to_string(),
                                );
                                recommendations.push(
                                    "Add workspaces configuration to root package.json".to_string(),
                                );
                            }

                            // Check for private flag
                            if json.get("private") != Some(&serde_json::Value::Bool(true)) {
                                warnings.push(
                                    "Root package.json should be marked as private".to_string(),
                                );
                                recommendations
                                    .push("Set \"private\": true in root package.json".to_string());
                            }
                        }
                        Err(_) => {
                            issues.push("Root package.json contains invalid JSON".to_string());
                        }
                    }
                }
                Err(_) => {
                    issues.push("Cannot read root package.json".to_string());
                }
            }
        }

        // 4. Check for monorepo configuration using the same logic as has_existing_config
        let config_base_names = ["monorepo.config", "monorepo"];
        let extensions = ["toml", "json", "yaml", "yml"];
        let legacy_configs = ["lerna.json", "nx.json", "rush.json", "workspace.json"];
        
        let mut has_monorepo_config = false;
        
        // Check modern config files with multiple extensions
        for base_name in &config_base_names {
            for ext in &extensions {
                let file_name = format!("{base_name}.{ext}");
                if context.root_path.join(&file_name).exists() {
                    has_monorepo_config = true;
                    break;
                }
            }
            if has_monorepo_config { break; }
        }
        
        // Check legacy config files
        if !has_monorepo_config {
            has_monorepo_config = legacy_configs.iter().any(|file| context.root_path.join(file).exists());
        }
        
        if !has_monorepo_config {
            recommendations.push("Consider adding a monorepo configuration file (monorepo.config.toml)".to_string());
        }

        // 5. Basic structure validation - just ensure packages have proper locations
        if !context.packages.is_empty() {
            for package in context.packages {
                if !package.path().exists() {
                    warnings.push(format!("Package '{}' path does not exist: {}", 
                        package.name(), package.path().display()));
                }
            }
        }

        // Calculate validation score
        let issues_count = issues.len();
        let warnings_count = warnings.len();

        let validation_score =
            u8::try_from(std::cmp::max(0, 100 - (issues_count * 10) - (warnings_count * 3))).unwrap_or(0);

        let structure_valid = issues.is_empty();
        let overall_status = match validation_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        let result = serde_json::json!({
            "structure_valid": structure_valid,
            "validation_score": validation_score,
            "overall_status": overall_status,
            "issues": issues,
            "warnings": warnings,
            "recommendations": recommendations,
            "statistics": {
                "total_packages": context.packages.len(),
                "issues_found": issues_count,
                "warnings_found": warnings_count,
                "recommendations_count": recommendations.len()
            },
            "checks_performed": {
                "root_files": true,
                "package_structure": true,
                "workspace_config": true,
                "directory_patterns": true
            }
        });

        success_with_timing(result, start_time)
            .with_metadata("command", "validate-structure")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("validation_score", validation_score)
    }
}
