//! Dependency analysis functionality for the analyzer plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::AnalyzerPlugin {
    /// Analyze package dependencies using real dependency analysis
    ///
    /// Performs comprehensive dependency analysis using the DependencyAnalysisService
    /// to provide accurate dependency relationships, external dependencies,
    /// and package statistics.
    ///
    /// # Arguments
    ///
    /// * `package_filter` - Optional package name to filter analysis
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Detailed dependency analysis including:
    /// - Total package count
    /// - External vs internal dependency breakdown
    /// - Dependency conflicts
    /// - Package-specific analysis if filtered
    pub(super) fn analyze_dependencies(
        package_filter: Option<&str>,
        context: &PluginContext,
    ) -> Result<PluginResult> {
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

        // Create dependency service for real analysis
        let mut dependency_service = crate::core::services::DependencyAnalysisService::new(
            &package_service,
            context.config_ref,
        )
        .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to create dependency service: {e}"))
        })?;

        // Build dependency graph for comprehensive analysis
        let dependency_graph = dependency_service.build_dependency_graph().map_err(|e| {
            crate::error::Error::plugin(format!("Failed to build dependency graph: {e}"))
        })?;

        // Clone the dependency graph data to avoid borrowing issues
        let dependencies_count = dependency_graph.dependencies.len();

        // Get external dependencies
        let external_dependencies =
            dependency_service.get_external_dependencies().map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get external dependencies: {e}"))
            })?;

        // Detect dependency conflicts
        let conflicts = dependency_service.detect_dependency_conflicts();

        // Prepare analysis result
        let mut analysis = serde_json::Map::new();
        analysis.insert(
            "total_packages".to_string(),
            serde_json::Value::Number(serde_json::Number::from(context.packages.len())),
        );
        analysis.insert(
            "external_dependencies".to_string(),
            serde_json::Value::Number(serde_json::Number::from(external_dependencies.len())),
        );
        analysis.insert(
            "internal_dependencies".to_string(),
            serde_json::Value::Number(serde_json::Number::from(dependencies_count)),
        );
        analysis.insert(
            "dependency_conflicts".to_string(),
            serde_json::Value::Number(serde_json::Number::from(conflicts.len())),
        );

        // Add external dependencies list
        let external_deps: Vec<serde_json::Value> =
            external_dependencies.into_iter().map(serde_json::Value::String).collect();
        analysis.insert(
            "external_dependency_list".to_string(),
            serde_json::Value::Array(external_deps),
        );

        // Add conflicts details
        let conflicts_json: Vec<serde_json::Value> = conflicts
            .into_iter()
            .map(|conflict| {
                serde_json::json!({
                    "dependency_name": conflict.dependency_name,
                    "conflicting_packages": conflict.conflicting_packages.into_iter()
                        .map(|pkg| serde_json::json!({
                            "package_name": pkg.package_name,
                            "version_requirement": pkg.version_requirement
                        }))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        analysis.insert("conflicts_detail".to_string(), serde_json::Value::Array(conflicts_json));

        // Package-specific analysis if filtered
        if let Some(package_name) = package_filter {
            analysis.insert(
                "analyzed_package".to_string(),
                serde_json::Value::String(package_name.to_string()),
            );

            // Get dependencies for specific package
            let package_deps = dependency_service.get_dependencies(package_name).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get package dependencies: {e}"))
            })?;

            let package_dependents =
                dependency_service.get_dependents(package_name).map_err(|e| {
                    crate::error::Error::plugin(format!("Failed to get package dependents: {e}"))
                })?;

            analysis.insert("package_dependencies".to_string(), serde_json::json!(package_deps));
            analysis
                .insert("package_dependents".to_string(), serde_json::json!(package_dependents));
            analysis.insert(
                "dependencies_count".to_string(),
                serde_json::Value::Number(serde_json::Number::from(package_deps.len())),
            );
            analysis.insert(
                "dependents_count".to_string(),
                serde_json::Value::Number(serde_json::Number::from(package_dependents.len())),
            );
        }

        Ok(success_with_timing(analysis, start_time)
            .with_metadata("command", "analyze-dependencies")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true))
    }
}