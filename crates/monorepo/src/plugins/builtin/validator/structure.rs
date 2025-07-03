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

        // 1. Validate root structure
        let expected_root_files = vec!["package.json", ".gitignore", "README.md"];
        for expected_file in &expected_root_files {
            let file_path = context.root_path.join(expected_file);
            if !file_path.exists() {
                issues.push(format!("Missing required root file: {expected_file}"));
                recommendations.push(format!("Create {expected_file} in the root directory"));
            }
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

        // 4. Check for common configuration files
        let config_files = vec![
            ("tsconfig.json", "TypeScript configuration"),
            (".gitignore", "Git ignore patterns"),
        ];

        for (config_file, description) in &config_files {
            let config_path = context.root_path.join(config_file);
            if !config_path.exists() {
                recommendations.push(format!("Consider adding {config_file} for {description}"));
            }
        }

        // 5. Validate directory structure patterns
        let common_dirs = vec!["packages", "apps", "libs", "tools"];
        let mut has_package_dir = false;

        for dir in &common_dirs {
            let dir_path = context.root_path.join(dir);
            if dir_path.exists() && dir_path.is_dir() {
                has_package_dir = true;
                break;
            }
        }

        if !has_package_dir && !context.packages.is_empty() {
            warnings.push(
                "Packages found but no standard package directory structure detected".to_string(),
            );
            recommendations.push(
                "Consider organizing packages in packages/, apps/, or libs/ directories"
                    .to_string(),
            );
        }

        // Calculate validation score
        let _total_checks =
            expected_root_files.len() + context.packages.len() + config_files.len() + 3;
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
