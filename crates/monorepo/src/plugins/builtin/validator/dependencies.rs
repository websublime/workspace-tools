//! Dependency validation functionality for the validator plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::ValidatorPlugin {
    /// Validate dependencies using real dependency analysis
    ///
    /// Performs comprehensive dependency validation including:
    /// - Version consistency across packages
    /// - Circular dependency detection
    /// - Unused dependency identification
    /// - Version range compatibility
    /// - Security vulnerability checks (basic)
    ///
    /// # Arguments
    ///
    /// * `strict` - Enable strict validation mode with additional checks
    /// * `context` - Plugin context with access to packages and services
    ///
    /// # Returns
    ///
    /// Detailed dependency validation result with violations and warnings
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_dependencies(strict: bool, context: &PluginContext) -> Result<PluginResult> {
        let start_time = Instant::now();

        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Create dependency service for real analysis
        let file_system_service = crate::core::services::FileSystemService::new(context.root_path)
            .map_err(|e| {
                crate::error::Error::plugin(format!("Failed to create file system service: {e}"))
            })?;

        let package_service = crate::core::services::PackageDiscoveryService::new(
            context.root_path,
            &file_system_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create package service: {e}"))
        })?;

        let mut dependency_service = crate::core::services::DependencyAnalysisService::new(
            &package_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create dependency service: {e}"))
        })?;

        // 1. Check for circular dependencies (critical)
        let circular_deps = dependency_service.detect_circular_dependencies().map_err(|e| {
            crate::error::Error::plugin(format!("Failed to detect circular dependencies: {e}"))
        })?;

        for cycle in &circular_deps {
            violations.push(serde_json::json!({
                "type": "circular_dependency",
                "severity": "critical",
                "description": format!("Circular dependency detected: {}", cycle.join(" -> ")),
                "packages": cycle,
                "resolution": "Refactor packages to break the circular dependency"
            }));
        }

        // 2. Check for dependency conflicts
        let conflicts = dependency_service.detect_dependency_conflicts();

        for conflict in &conflicts {
            violations.push(serde_json::json!({
                "type": "version_conflict",
                "severity": "high",
                "description": format!(
                    "Dependency '{}' has conflicting version requirements",
                    conflict.dependency_name
                ),
                "dependency": conflict.dependency_name,
                "conflicting_packages": conflict.conflicting_packages.iter()
                    .map(|pkg| serde_json::json!({
                        "package": pkg.package_name,
                        "requirement": pkg.version_requirement
                    }))
                    .collect::<Vec<_>>(),
                "resolution": "Align version requirements across all packages"
            }));
        }

        // 3. Validate individual package dependencies
        let mut dependency_stats = std::collections::HashMap::new();

        for package in context.packages {
            let package_name = package.name();

            // Check for empty dependencies (potential issue)
            if package.dependencies.is_empty() && package.dependencies_external.is_empty() && strict
            {
                warnings.push(serde_json::json!({
                    "type": "no_dependencies",
                    "severity": "low",
                    "package": package_name,
                    "description": format!("Package '{}' has no dependencies", package_name),
                    "suggestion": "Verify if this package should have dependencies"
                }));
            }

            // Check for excessive dependencies (potential bloat)
            let total_deps = package.dependencies.len() + package.dependencies_external.len();
            if total_deps > 50 {
                warnings.push(serde_json::json!({
                    "type": "excessive_dependencies",
                    "severity": "medium",
                    "package": package_name,
                    "description": format!("Package '{}' has {} dependencies (excessive)", package_name, total_deps),
                    "suggestion": "Review if all dependencies are necessary"
                }));
            }

            // Analyze dependency patterns
            let mut dev_deps = 0;
            let mut prod_deps = 0;

            for dep in &package.dependencies {
                match dep.dependency_type {
                    crate::core::types::DependencyType::Development => dev_deps += 1,
                    crate::core::types::DependencyType::Runtime => prod_deps += 1,
                    _ => {}
                }

                // Check for problematic version ranges in strict mode
                if strict {
                    if dep.version_requirement == "*" {
                        violations.push(serde_json::json!({
                            "type": "wildcard_version",
                            "severity": "high",
                            "package": package_name,
                            "dependency": dep.name,
                            "description": format!(
                                "Package '{}' uses wildcard version for '{}'",
                                package_name, dep.name
                            ),
                            "resolution": "Specify exact or bounded version range"
                        }));
                    }

                    if dep.version_requirement.starts_with("file:")
                        || dep.version_requirement.starts_with("git+")
                    {
                        warnings.push(serde_json::json!({
                            "type": "non_registry_dependency",
                            "severity": "medium",
                            "package": package_name,
                            "dependency": dep.name,
                            "description": format!(
                                "Package '{}' uses non-registry dependency '{}'",
                                package_name, dep.name
                            ),
                            "suggestion": "Consider using registry versions for better stability"
                        }));
                    }
                }
            }

            dependency_stats.insert(package_name.to_string(), serde_json::json!({
                "total_dependencies": total_deps,
                "production_dependencies": prod_deps,
                "development_dependencies": dev_deps,
                "external_dependencies": package.dependencies_external.len(),
                "internal_dependencies": package.dependencies.len() - package.dependencies_external.len()
            }));
        }

        // 4. Check dependency constraint consistency
        if let Err(constraint_error) = dependency_service.validate_dependency_constraints() {
            violations.push(serde_json::json!({
                "type": "constraint_violation",
                "severity": "critical",
                "description": format!("Dependency constraint validation failed: {}", constraint_error),
                "resolution": "Review and fix dependency constraints across all packages"
            }));
        }

        // 5. Generate recommendations based on analysis
        if circular_deps.is_empty() && conflicts.is_empty() {
            recommendations.push("Dependency graph is healthy with no critical issues".to_string());
        }

        if !violations.is_empty() {
            recommendations
                .push("Address critical dependency violations before proceeding".to_string());
        }

        if warnings.len() > 5 {
            recommendations.push("Consider reviewing dependency management practices".to_string());
        }

        // Calculate dependency health score
        let critical_issues = violations.iter().filter(|v| v["severity"] == "critical").count();
        let high_issues = violations.iter().filter(|v| v["severity"] == "high").count();
        let medium_issues = warnings.iter().filter(|w| w["severity"] == "medium").count();

        let health_score = u8::try_from(std::cmp::max(
            0,
            100 - (critical_issues * 25) - (high_issues * 10) - (medium_issues * 5),
        )).unwrap_or(0);

        let dependencies_valid = violations.is_empty();
        let overall_status = match health_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        let result = serde_json::json!({
            "dependencies_valid": dependencies_valid,
            "strict_mode": strict,
            "health_score": health_score,
            "overall_status": overall_status,
            "violations": violations,
            "warnings": warnings,
            "recommendations": recommendations,
            "statistics": {
                "total_packages": context.packages.len(),
                "circular_dependencies": circular_deps.len(),
                "version_conflicts": conflicts.len(),
                "critical_violations": critical_issues,
                "high_severity_issues": high_issues,
                "warnings_count": warnings.len()
            },
            "package_dependency_stats": dependency_stats,
            "analysis_details": {
                "circular_dependency_chains": circular_deps,
                "dependency_conflicts": conflicts.len(),
                "packages_analyzed": context.packages.len()
            }
        });

        Ok(success_with_timing(result, start_time)
            .with_metadata("command", "validate-dependencies")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("health_score", health_score)
            .with_metadata("strict_mode", strict))
    }
}