//! Circular dependency detection functionality for the analyzer plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::AnalyzerPlugin {
    /// Detect circular dependencies using real dependency analysis
    ///
    /// Performs comprehensive circular dependency detection using the
    /// DependencyAnalysisService to identify actual dependency cycles
    /// that could cause build or runtime issues.
    ///
    /// # Arguments
    ///
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Detailed circular dependency analysis including:
    /// - Number of cycles found
    /// - Complete cycle chains
    /// - Affected packages
    /// - Severity assessment
    pub(super) fn detect_cycles(context: &PluginContext) -> Result<PluginResult> {
        let start_time = Instant::now();

        // Create file system service for package discovery
        let file_system_service = crate::core::services::FileSystemService::new(context.root_path)
            .map_err(|e| {
                crate::error::Error::plugin(format!("Failed to create file system service: {e}"))
            })?;

        // Create package service for dependency analysis
        let package_service = crate::core::services::PackageDiscoveryService::new(
            context.root_path,
            &file_system_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create package service: {e}"))
        })?;

        // Create dependency service for real circular dependency detection
        let mut dependency_service = crate::core::services::DependencyAnalysisService::new(
            &package_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create dependency service: {e}"))
        })?;

        // Detect circular dependencies using real analysis
        let circular_dependencies =
            dependency_service.detect_circular_dependencies().map_err(|e| {
                crate::error::Error::plugin(format!("Failed to detect circular dependencies: {e}"))
            })?;

        // Analyze cycle severity and create detailed report
        let mut affected_packages = std::collections::HashSet::new();
        let cycles_detail: Vec<serde_json::Value> = circular_dependencies.iter()
            .enumerate()
            .map(|(index, cycle)| {
                // Add all packages in this cycle to affected set
                for package in cycle {
                    affected_packages.insert(package.clone());
                }

                // Determine cycle severity based on length and package types
                let severity = if cycle.len() <= 2 {
                    "low"  // Simple bidirectional dependency
                } else if cycle.len() <= 4 {
                    "medium"  // Complex but manageable
                } else {
                    "high"  // Very complex cycle
                };

                serde_json::json!({
                    "cycle_id": index + 1,
                    "packages": cycle,
                    "length": cycle.len(),
                    "severity": severity,
                    "cycle_path": format!("{} -> {}", cycle.join(" -> "), cycle.first().unwrap_or(&"unknown".to_string())),
                    "recommendation": match severity {
                        "low" => "Consider refactoring to remove bidirectional dependency",
                        "medium" => "Refactor to break dependency cycle - use dependency injection or interfaces",
                        "high" => "CRITICAL: Complex dependency cycle requires immediate architectural refactoring",
                        _ => "Review dependency structure"
                    }
                })
            })
            .collect();

        // Calculate impact analysis
        let total_packages = context.packages.len();
        let affected_percentage = if total_packages > 0 {
            #[allow(clippy::cast_precision_loss)]
            let percentage = (affected_packages.len() as f64 / total_packages.max(1) as f64) * 100.0;
            percentage
        } else {
            0.0
        };

        // Determine overall status
        let overall_status = match circular_dependencies.len() {
            0 => "clean",
            1..=2 => "warning",
            3..=5 => "error",
            _ => "critical",
        };

        let cycles = serde_json::json!({
            "cycles_found": circular_dependencies.len(),
            "cycles": cycles_detail,
            "affected_packages": affected_packages.iter().cloned().collect::<Vec<_>>(),
            "affected_count": affected_packages.len(),
            "total_packages": total_packages,
            "affected_percentage": format!("{:.1}%", affected_percentage),
            "overall_status": overall_status,
            "health_score": u8::try_from(std::cmp::max(0, 100 - (circular_dependencies.len() * 10))).unwrap_or(0),
            "recommendations": match overall_status {
                "clean" => "No circular dependencies detected. Dependency graph is healthy.",
                "warning" => "Few circular dependencies detected. Consider refactoring for better maintainability.",
                "error" => "Multiple circular dependencies detected. Refactoring recommended to prevent build issues.",
                "critical" => "CRITICAL: Extensive circular dependencies detected. Immediate architectural review required.",
                _ => "Review dependency structure"
            }
        });

        Ok(success_with_timing(cycles, start_time)
            .with_metadata("command", "detect-cycles")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true)
            .with_metadata("dependency_health", overall_status))
    }
}