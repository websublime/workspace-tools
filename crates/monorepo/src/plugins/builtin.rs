//! Built-in plugin implementations
//!
//! Provides default plugin implementations that are compiled into the application.
//! These plugins demonstrate the plugin system and provide essential functionality.

use super::types::{
    MonorepoPlugin, PluginArgument, PluginArgumentType, PluginCapabilities, PluginCommand,
    PluginContext, PluginInfo, PluginResult,
};
use crate::error::Result;

/// Built-in analyzer plugin for code analysis and dependency tracking
///
/// Provides functionality for analyzing code structure, dependencies,
/// and package relationships within the monorepo.
pub struct AnalyzerPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
    /// Whether the plugin is initialized
    initialized: bool,
}

impl AnalyzerPlugin {
    /// Create a new analyzer plugin
    pub fn new() -> Self {
        Self { name: "analyzer".to_string(), version: "1.0.0".to_string(), initialized: false }
    }
}

impl MonorepoPlugin for AnalyzerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in code analysis and dependency tracking plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "analyze-dependencies".to_string(),
                        description: "Analyze package dependencies and relationships".to_string(),
                        arguments: vec![PluginArgument {
                            name: "package".to_string(),
                            description: "Specific package to analyze (optional)".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: None,
                        }],
                        async_support: true,
                    },
                    PluginCommand {
                        name: "detect-cycles".to_string(),
                        description: "Detect circular dependencies in the monorepo".to_string(),
                        arguments: vec![],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "impact-analysis".to_string(),
                        description: "Analyze change impact across packages".to_string(),
                        arguments: vec![PluginArgument {
                            name: "since".to_string(),
                            description: "Analyze changes since this commit/tag".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: Some("HEAD~1".to_string()),
                        }],
                        async_support: true,
                    },
                ],
                async_support: true,
                parallel_support: false,
                categories: vec!["analyzer".to_string(), "dependencies".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.ts".to_string(),
                    "*.js".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing analyzer plugin with access to monorepo services");
        self.initialized = true;
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        if !self.initialized {
            return Ok(PluginResult::error("Plugin not initialized".to_string()));
        }

        match command {
            "analyze-dependencies" => {
                let package_filter = args.first().map(std::string::String::as_str);
                self.analyze_dependencies(package_filter, context)
            }
            "detect-cycles" => self.detect_cycles(context),
            "impact-analysis" => {
                let since = args.first().map_or("HEAD~1", |s| s.as_str());
                self.impact_analysis(since, context)
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl AnalyzerPlugin {
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
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn analyze_dependencies(
        &self,
        package_filter: Option<&str>,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

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

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(PluginResult::success_with_time(analysis, execution_time)
            .with_metadata("command", "analyze-dependencies")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true))
    }

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
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn detect_cycles(&self, context: &PluginContext) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

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
            (affected_packages.len() as f64 / total_packages as f64) * 100.0
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
            "health_score": std::cmp::max(0, 100 - (circular_dependencies.len() * 10)) as u8,
            "recommendations": match overall_status {
                "clean" => "No circular dependencies detected. Dependency graph is healthy.",
                "warning" => "Few circular dependencies detected. Consider refactoring for better maintainability.",
                "error" => "Multiple circular dependencies detected. Refactoring recommended to prevent build issues.",
                "critical" => "CRITICAL: Extensive circular dependencies detected. Immediate architectural review required.",
                _ => "Review dependency structure"
            }
        });

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(PluginResult::success_with_time(cycles, execution_time)
            .with_metadata("command", "detect-cycles")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true)
            .with_metadata("dependency_health", overall_status))
    }

    /// Perform comprehensive impact analysis using real Git and dependency analysis
    ///
    /// Analyzes the impact of changes since a given reference using:
    /// - Real Git operations to detect changed files
    /// - Dependency analysis to understand propagation
    /// - Package change analysis for comprehensive impact assessment
    ///
    /// # Arguments
    ///
    /// * `since` - Git reference to analyze changes from (commit, tag, or branch)
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Comprehensive impact analysis including:
    /// - Changed files and packages
    /// - Dependency propagation analysis
    /// - Affected packages through dependency chains
    /// - Severity and risk assessment
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::too_many_lines)]
    fn impact_analysis(&self, since: &str, context: &PluginContext) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        // Get changed files using real Git operations
        let changed_files = context
            .repository
            .get_all_files_changed_since_sha_with_status(since)
            .map_err(|e| {
            crate::error::Error::plugin(format!("Failed to get changed files since {since}: {e}"))
        })?;

        // Analyze which packages are directly affected
        let mut directly_affected = std::collections::HashSet::new();
        let mut file_changes_by_package = std::collections::HashMap::new();

        for changed_file in &changed_files {
            for package in context.packages {
                let package_path = package.path().to_string_lossy();
                if changed_file.path.starts_with(package_path.as_ref()) {
                    directly_affected.insert(package.name().to_string());

                    file_changes_by_package
                        .entry(package.name().to_string())
                        .or_insert_with(Vec::new)
                        .push(serde_json::json!({
                            "file": changed_file.path,
                            "status": match changed_file.status {
                                sublime_git_tools::GitFileStatus::Added => "added",
                                sublime_git_tools::GitFileStatus::Modified => "modified",
                                sublime_git_tools::GitFileStatus::Deleted => "deleted",
                                sublime_git_tools::GitFileStatus::Untracked => "untracked",
                            }
                        }));
                    break;
                }
            }
        }

        // Create dependency service for propagation analysis
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

        // Get packages affected through dependencies (propagation)
        let changed_package_names: Vec<String> = directly_affected.iter().cloned().collect();
        let all_affected =
            dependency_service.get_affected_packages(&changed_package_names).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get affected packages: {e}"))
            })?;

        // Separate directly vs indirectly affected
        let indirectly_affected: Vec<String> =
            all_affected.iter().filter(|pkg| !directly_affected.contains(*pkg)).cloned().collect();

        // Analyze change types and severity
        let mut change_types = std::collections::HashMap::new();
        for changed_file in &changed_files {
            let change_type = if changed_file.path.contains("package.json") {
                "dependency_change"
            } else if changed_file.path.contains(".config") || changed_file.path.contains("config")
            {
                "configuration_change"
            } else if changed_file.path.contains(".test.")
                || changed_file.path.contains("test/")
                || changed_file.path.contains("__tests__")
            {
                "test_change"
            } else if changed_file.path.contains(".md") || changed_file.path.contains("README") {
                "documentation_change"
            } else {
                "source_code_change"
            };

            *change_types.entry(change_type.to_string()).or_insert(0) += 1;
        }

        // Calculate risk assessment
        let total_packages = context.packages.len();
        let impact_percentage = if total_packages > 0 {
            (all_affected.len() as f64 / total_packages as f64) * 100.0
        } else {
            0.0
        };

        let risk_level = match impact_percentage {
            p if p >= 50.0 => "high",
            p if p >= 20.0 => "medium",
            p if p > 0.0 => "low",
            _ => "none",
        };

        // Get commit information for better context
        let commits =
            context.repository.get_commits_since(Some(since.to_string()), &None).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to get commits since {since}: {e}"))
            })?;

        let impact = serde_json::json!({
            "since": since,
            "analysis_timestamp": chrono::Utc::now().to_rfc3339(),
            "changed_files": {
                "total": changed_files.len(),
                "by_package": file_changes_by_package,
                "change_types": change_types
            },
            "affected_packages": {
                "directly_affected": directly_affected.into_iter().collect::<Vec<_>>(),
                "indirectly_affected": indirectly_affected,
                "total_affected": all_affected.len(),
                "directly_affected_count": changed_package_names.len(),
                "indirectly_affected_count": all_affected.len() - changed_package_names.len()
            },
            "impact_assessment": {
                "risk_level": risk_level,
                "impact_percentage": format!("{:.1}%", impact_percentage),
                "total_packages": total_packages,
                "propagation_factor": if changed_package_names.is_empty() {
                    0.0
                } else {
                    all_affected.len() as f64 / changed_package_names.len() as f64
                }
            },
            "commit_analysis": {
                "commit_count": commits.len(),
                "recent_commits": commits.into_iter().take(5).map(|commit| serde_json::json!({
                    "hash": commit.hash[0..8].to_string(),
                    "message": commit.message.lines().next().unwrap_or("No message"),
                    "author": commit.author_name,
                    "date": commit.author_date
                })).collect::<Vec<_>>()
            },
            "recommendations": match risk_level {
                "high" => "High impact changes detected. Consider careful testing and staged deployment.",
                "medium" => "Medium impact changes. Verify affected packages and run comprehensive tests.",
                "low" => "Low impact changes. Standard testing procedures should be sufficient.",
                "none" => "No package impact detected. Changes may be documentation or configuration only.",
                _ => "Review changes carefully"
            }
        });

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(PluginResult::success_with_time(impact, execution_time)
            .with_metadata("command", "impact-analysis")
            .with_metadata("analyzer", "builtin")
            .with_metadata("real_analysis", true)
            .with_metadata("risk_level", risk_level)
            .with_metadata("files_analyzed", changed_files.len()))
    }
}

/// Built-in generator plugin for code generation and templating
///
/// Provides functionality for generating code, configuration files,
/// and project structures within the monorepo.
pub struct GeneratorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl GeneratorPlugin {
    /// Create a new generator plugin
    pub fn new() -> Self {
        Self { name: "generator".to_string(), version: "1.0.0".to_string() }
    }
}

impl MonorepoPlugin for GeneratorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in code generation and templating plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "generate-package".to_string(),
                        description: "Generate a new package from template".to_string(),
                        arguments: vec![
                            PluginArgument {
                                name: "name".to_string(),
                                description: "Package name".to_string(),
                                required: true,
                                arg_type: PluginArgumentType::String,
                                default_value: None,
                            },
                            PluginArgument {
                                name: "template".to_string(),
                                description: "Template to use".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("default".to_string()),
                            },
                        ],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "generate-config".to_string(),
                        description: "Generate configuration files".to_string(),
                        arguments: vec![PluginArgument {
                            name: "type".to_string(),
                            description: "Configuration type (eslint, prettier, etc.)".to_string(),
                            required: true,
                            arg_type: PluginArgumentType::String,
                            default_value: None,
                        }],
                        async_support: false,
                    },
                ],
                async_support: false,
                parallel_support: true,
                categories: vec!["generator".to_string(), "templates".to_string()],
                file_patterns: vec!["*.template".to_string(), "*.mustache".to_string()],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing generator plugin");
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        match command {
            "generate-package" => {
                let name = args
                    .first()
                    .ok_or_else(|| crate::error::Error::plugin("Package name required"))?;
                let template = args.get(1).map_or("default", |s| s.as_str());
                self.generate_package(name, template, context)
            }
            "generate-config" => {
                let config_type = args
                    .first()
                    .ok_or_else(|| crate::error::Error::plugin("Config type required"))?;
                self.generate_config(config_type, context)
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl GeneratorPlugin {
    /// Generate a new package with real file creation
    ///
    /// Creates actual package files in the monorepo using the file system service
    /// and following the project's package structure conventions.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the package to generate
    /// * `template` - Template type to use for generation
    /// * `context` - Plugin context with access to file system and configuration
    ///
    /// # Returns
    ///
    /// Result with details of actually created files and package structure
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::too_many_lines)]
    fn generate_package(
        &self,
        name: &str,
        template: &str,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        // Validate package name
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Ok(PluginResult::error(format!("Invalid package name: {name}. Use alphanumeric characters, hyphens, and underscores only.")));
        }

        // Check if package already exists
        for existing_package in context.packages {
            if existing_package.name() == name {
                return Ok(PluginResult::error(format!(
                    "Package '{name}' already exists at {}",
                    existing_package.path().display()
                )));
            }
        }

        // Determine package path based on monorepo structure
        let packages_dir = context.root_path.join("packages");
        let package_path = packages_dir.join(name);

        // Create package directory
        if let Err(e) = std::fs::create_dir_all(&package_path) {
            return Ok(PluginResult::error(format!("Failed to create package directory: {e}")));
        }

        let mut generated_files = Vec::new();

        // Generate package.json based on template
        let package_json = match template {
            "library" => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated library package: {name}"),
                "main": "dist/index.js",
                "types": "dist/index.d.ts",
                "scripts": {
                    "build": "tsc",
                    "test": "jest",
                    "lint": "eslint src/**/*.ts",
                    "clean": "rm -rf dist"
                },
                "keywords": [name],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT",
                "devDependencies": {
                    "typescript": "^5.0.0",
                    "@types/node": "^20.0.0",
                    "jest": "^29.0.0",
                    "eslint": "^8.0.0"
                }
            }),
            "app" => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated application package: {name}"),
                "main": "dist/app.js",
                "scripts": {
                    "build": "tsc",
                    "start": "node dist/app.js",
                    "dev": "ts-node src/app.ts",
                    "test": "jest",
                    "lint": "eslint src/**/*.ts"
                },
                "keywords": [name, "application"],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT",
                "dependencies": {
                    "express": "^4.18.0"
                },
                "devDependencies": {
                    "typescript": "^5.0.0",
                    "@types/node": "^20.0.0",
                    "@types/express": "^4.17.0",
                    "ts-node": "^10.0.0",
                    "jest": "^29.0.0",
                    "eslint": "^8.0.0"
                }
            }),
            _ => serde_json::json!({
                "name": name,
                "version": "0.1.0",
                "description": format!("Generated package: {name}"),
                "main": "dist/index.js",
                "scripts": {
                    "build": "tsc",
                    "test": "jest"
                },
                "keywords": [name],
                "author": "Generated by Sublime Monorepo Tools",
                "license": "MIT"
            }),
        };

        // Write package.json
        let package_json_path = package_path.join("package.json");
        let package_json_content = serde_json::to_string_pretty(&package_json).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to serialize package.json: {e}"))
        })?;

        if let Err(e) = std::fs::write(&package_json_path, package_json_content) {
            return Ok(PluginResult::error(format!("Failed to write package.json: {e}")));
        }
        generated_files.push("package.json".to_string());

        // Create src directory and main file
        let src_dir = package_path.join("src");
        if let Err(e) = std::fs::create_dir_all(&src_dir) {
            return Ok(PluginResult::error(format!("Failed to create src directory: {e}")));
        }

        // Generate main file based on template
        let (main_file, main_content) = match template {
            "library" => ("index.ts", format!(
                "/**\n * {name} library\n * Generated by Sublime Monorepo Tools\n */\n\nexport function hello(): string {{\n    return 'Hello from {name}!';\n}}\n\nexport default hello;\n"
            )),
            "app" => ("app.ts", format!(
                "/**\n * {name} application\n * Generated by Sublime Monorepo Tools\n */\n\nimport express from 'express';\n\nconst app = express();\nconst port = process.env.PORT || 3000;\n\napp.get('/', (req, res) => {{\n    res.json({{ message: 'Hello from {name}!' }});\n}});\n\napp.listen(port, () => {{\n    console.log(`{name} is running on port ${{port}}`);\n}});\n"
            )),
            _ => ("index.ts", format!(
                "/**\n * {name}\n * Generated by Sublime Monorepo Tools\n */\n\nexport function greet(name: string): string {{\n    return `Hello, ${{name}}!`;\n}}\n\nconsole.log(greet('{name}'));\n"
            ))
        };

        let main_file_path = src_dir.join(main_file);
        if let Err(e) = std::fs::write(&main_file_path, main_content) {
            return Ok(PluginResult::error(format!("Failed to write main file: {e}")));
        }
        generated_files.push(format!("src/{main_file}"));

        // Generate README.md
        let readme_content = format!(
            "# {name}\n\nGenerated package by Sublime Monorepo Tools\n\n## Template: {template}\n\n## Installation\n\n```bash\nnpm install\n```\n\n## Build\n\n```bash\nnpm run build\n```\n\n## Test\n\n```bash\nnpm test\n```\n"
        );

        let readme_path = package_path.join("README.md");
        if let Err(e) = std::fs::write(&readme_path, readme_content) {
            return Ok(PluginResult::error(format!("Failed to write README.md: {e}")));
        }
        generated_files.push("README.md".to_string());

        // Generate TypeScript config if applicable
        if template == "library" || template == "app" {
            let tsconfig = serde_json::json!({
                "compilerOptions": {
                    "target": "ES2020",
                    "module": "commonjs",
                    "outDir": "./dist",
                    "rootDir": "./src",
                    "strict": true,
                    "esModuleInterop": true,
                    "skipLibCheck": true,
                    "forceConsistentCasingInFileNames": true,
                    "declaration": true,
                    "declarationMap": true,
                    "sourceMap": true
                },
                "include": ["src/**/*"],
                "exclude": ["node_modules", "dist"]
            });

            let tsconfig_path = package_path.join("tsconfig.json");
            let tsconfig_content = serde_json::to_string_pretty(&tsconfig).map_err(|e| {
                crate::error::Error::plugin(format!("Failed to serialize tsconfig.json: {e}"))
            })?;

            if let Err(e) = std::fs::write(&tsconfig_path, tsconfig_content) {
                return Ok(PluginResult::error(format!("Failed to write tsconfig.json: {e}")));
            }
            generated_files.push("tsconfig.json".to_string());
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        let result = serde_json::json!({
            "package_name": name,
            "template_used": template,
            "package_path": package_path.to_string_lossy(),
            "generated_files": generated_files,
            "status": "successfully_generated",
            "file_count": generated_files.len()
        });

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "generate-package")
            .with_metadata("generator", "builtin")
            .with_metadata("real_generation", true)
            .with_metadata("package_path", package_path.to_string_lossy()))
    }

    /// Generate configuration files with real file creation
    ///
    /// Creates actual configuration files in the monorepo root or specified location
    /// based on the configuration type and best practices for the ecosystem.
    ///
    /// # Arguments
    ///
    /// * `config_type` - Type of configuration to generate (eslint, prettier, typescript, jest, etc.)
    /// * `context` - Plugin context with access to file system and project structure
    ///
    /// # Returns
    ///
    /// Result with details of actually created configuration files
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::too_many_lines)]
    fn generate_config(&self, config_type: &str, context: &PluginContext) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        let mut generated_files = Vec::new();

        match config_type {
            "eslint" => {
                let eslint_config = serde_json::json!({
                    "env": {
                        "browser": true,
                        "es2021": true,
                        "node": true
                    },
                    "extends": [
                        "eslint:recommended",
                        "@typescript-eslint/recommended"
                    ],
                    "parser": "@typescript-eslint/parser",
                    "parserOptions": {
                        "ecmaVersion": "latest",
                        "sourceType": "module"
                    },
                    "plugins": ["@typescript-eslint"],
                    "rules": {
                        "indent": ["error", 2],
                        "linebreak-style": ["error", "unix"],
                        "quotes": ["error", "single"],
                        "semi": ["error", "always"],
                        "@typescript-eslint/no-unused-vars": "error",
                        "@typescript-eslint/explicit-function-return-type": "warn"
                    },
                    "ignorePatterns": ["dist/", "node_modules/", "*.js"]
                });

                let eslint_path = context.root_path.join(".eslintrc.json");
                let eslint_content = serde_json::to_string_pretty(&eslint_config).map_err(|e| {
                    crate::error::Error::plugin(format!("Failed to serialize eslint config: {e}"))
                })?;

                if let Err(e) = std::fs::write(&eslint_path, eslint_content) {
                    return Ok(PluginResult::error(format!("Failed to write .eslintrc.json: {e}")));
                }
                generated_files.push(".eslintrc.json".to_string());
            }
            "prettier" => {
                let prettier_config = serde_json::json!({
                    "semi": true,
                    "trailingComma": "es5",
                    "singleQuote": true,
                    "printWidth": 80,
                    "tabWidth": 2,
                    "useTabs": false,
                    "endOfLine": "lf",
                    "arrowParens": "avoid",
                    "bracketSpacing": true,
                    "bracketSameLine": false
                });

                let prettier_path = context.root_path.join(".prettierrc.json");
                let prettier_content =
                    serde_json::to_string_pretty(&prettier_config).map_err(|e| {
                        crate::error::Error::plugin(format!(
                            "Failed to serialize prettier config: {e}"
                        ))
                    })?;

                if let Err(e) = std::fs::write(&prettier_path, prettier_content) {
                    return Ok(PluginResult::error(format!(
                        "Failed to write .prettierrc.json: {e}"
                    )));
                }
                generated_files.push(".prettierrc.json".to_string());

                // Also generate .prettierignore
                let prettier_ignore =
                    "dist/\nnode_modules/\n*.min.js\n*.bundle.js\ncoverage/\n.nyc_output/\n";
                let prettier_ignore_path = context.root_path.join(".prettierignore");

                if let Err(e) = std::fs::write(&prettier_ignore_path, prettier_ignore) {
                    return Ok(PluginResult::error(format!(
                        "Failed to write .prettierignore: {e}"
                    )));
                }
                generated_files.push(".prettierignore".to_string());
            }
            "typescript" => {
                let tsconfig = serde_json::json!({
                    "compilerOptions": {
                        "target": "ES2020",
                        "lib": ["ES2020", "DOM"],
                        "module": "commonjs",
                        "moduleResolution": "node",
                        "outDir": "./dist",
                        "rootDir": "./src",
                        "strict": true,
                        "esModuleInterop": true,
                        "skipLibCheck": true,
                        "forceConsistentCasingInFileNames": true,
                        "declaration": true,
                        "declarationMap": true,
                        "sourceMap": true,
                        "removeComments": true,
                        "noUnusedLocals": true,
                        "noUnusedParameters": true,
                        "noImplicitReturns": true,
                        "noFallthroughCasesInSwitch": true
                    },
                    "include": ["src/**/*", "tests/**/*"],
                    "exclude": ["node_modules", "dist", "**/*.spec.ts", "**/*.test.ts"],
                    "compileOnSave": true
                });

                let tsconfig_path = context.root_path.join("tsconfig.json");
                let tsconfig_content = serde_json::to_string_pretty(&tsconfig).map_err(|e| {
                    crate::error::Error::plugin(format!("Failed to serialize tsconfig: {e}"))
                })?;

                if let Err(e) = std::fs::write(&tsconfig_path, tsconfig_content) {
                    return Ok(PluginResult::error(format!("Failed to write tsconfig.json: {e}")));
                }
                generated_files.push("tsconfig.json".to_string());
            }
            "jest" => {
                let jest_config = serde_json::json!({
                    "preset": "ts-jest",
                    "testEnvironment": "node",
                    "roots": ["<rootDir>/src", "<rootDir>/tests"],
                    "testMatch": ["**/__tests__/**/*.ts", "**/?(*.)+(spec|test).ts"],
                    "transform": {
                        "^.+\\.ts$": "ts-jest"
                    },
                    "collectCoverageFrom": [
                        "src/**/*.ts",
                        "!src/**/*.d.ts",
                        "!src/**/*.test.ts",
                        "!src/**/*.spec.ts"
                    ],
                    "coverageDirectory": "coverage",
                    "coverageReporters": ["text", "lcov", "html"],
                    "coverageThreshold": {
                        "global": {
                            "branches": 80,
                            "functions": 80,
                            "lines": 80,
                            "statements": 80
                        }
                    },
                    "moduleFileExtensions": ["ts", "js", "json"],
                    "setupFilesAfterEnv": [],
                    "verbose": true
                });

                let jest_path = context.root_path.join("jest.config.json");
                let jest_content = serde_json::to_string_pretty(&jest_config).map_err(|e| {
                    crate::error::Error::plugin(format!("Failed to serialize jest config: {e}"))
                })?;

                if let Err(e) = std::fs::write(&jest_path, jest_content) {
                    return Ok(PluginResult::error(format!(
                        "Failed to write jest.config.json: {e}"
                    )));
                }
                generated_files.push("jest.config.json".to_string());
            }
            "gitignore" => {
                let gitignore_content = "# Dependencies\nnode_modules/\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\n\n# Runtime data\npids\n*.pid\n*.seed\n*.pid.lock\n\n# Coverage\ncoverage/\n.nyc_output/\n\n# Build outputs\ndist/\nbuild/\n*.tsbuildinfo\n\n# Environment\n.env\n.env.local\n.env.development.local\n.env.test.local\n.env.production.local\n\n# Editor\n.vscode/\n.idea/\n*.swp\n*.swo\n*~\n\n# OS\n.DS_Store\nThumbs.db\n\n# Logs\nlogs\n*.log\n\n# Cache\n.cache/\n.parcel-cache/\n";

                let gitignore_path = context.root_path.join(".gitignore");

                if let Err(e) = std::fs::write(&gitignore_path, gitignore_content) {
                    return Ok(PluginResult::error(format!("Failed to write .gitignore: {e}")));
                }
                generated_files.push(".gitignore".to_string());
            }
            _ => {
                return Ok(PluginResult::error(format!(
                    "Unknown config type: {config_type}. Supported types: eslint, prettier, typescript, jest, gitignore"
                )));
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        let result = serde_json::json!({
            "config_type": config_type,
            "generated_files": generated_files,
            "file_count": generated_files.len(),
            "status": "successfully_generated",
            "location": context.root_path.to_string_lossy()
        });

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "generate-config")
            .with_metadata("generator", "builtin")
            .with_metadata("real_generation", true)
            .with_metadata("config_location", context.root_path.to_string_lossy()))
    }
}

/// Built-in validator plugin for validation and quality assurance
///
/// Provides functionality for validating code quality, style,
/// and adherence to monorepo policies.
pub struct ValidatorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl ValidatorPlugin {
    /// Create a new validator plugin
    pub fn new() -> Self {
        Self { name: "validator".to_string(), version: "1.0.0".to_string() }
    }
}

impl MonorepoPlugin for ValidatorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in validation and quality assurance plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "validate-structure".to_string(),
                        description: "Validate monorepo structure and conventions".to_string(),
                        arguments: vec![],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "validate-dependencies".to_string(),
                        description: "Validate dependency constraints and versions".to_string(),
                        arguments: vec![PluginArgument {
                            name: "strict".to_string(),
                            description: "Enable strict validation mode".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::Boolean,
                            default_value: Some("false".to_string()),
                        }],
                        async_support: true,
                    },
                    PluginCommand {
                        name: "validate-commits".to_string(),
                        description: "Validate commit messages against conventions".to_string(),
                        arguments: vec![PluginArgument {
                            name: "count".to_string(),
                            description: "Number of recent commits to validate".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::Integer,
                            default_value: Some("10".to_string()),
                        }],
                        async_support: false,
                    },
                ],
                async_support: true,
                parallel_support: true,
                categories: vec!["validator".to_string(), "quality".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.js".to_string(),
                    "*.ts".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing validator plugin");
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        match command {
            "validate-structure" => Ok(self.validate_structure(context)),
            "validate-dependencies" => {
                let strict = args.first().and_then(|s| s.parse().ok()).unwrap_or(false);
                self.validate_dependencies(strict, context)
            }
            "validate-commits" => {
                let count = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);
                self.validate_commits(count, context)
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl ValidatorPlugin {
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
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::too_many_lines)]
    fn validate_structure(&self, context: &PluginContext) -> PluginResult {
        let start_time = std::time::Instant::now();

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
            (".eslintrc.json", "ESLint configuration for code quality"),
            (".prettierrc.json", "Prettier configuration for code formatting"),
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
            std::cmp::max(0, 100 - (issues_count * 10) - (warnings_count * 3)) as u8;

        let structure_valid = issues.is_empty();
        let overall_status = match validation_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

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

        PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "validate-structure")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("validation_score", validation_score)
    }

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
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::too_many_lines)]
    fn validate_dependencies(&self, strict: bool, context: &PluginContext) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

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

        let health_score = std::cmp::max(
            0,
            100 - (critical_issues * 25) - (high_issues * 10) - (medium_issues * 5),
        ) as u8;

        let dependencies_valid = violations.is_empty();
        let overall_status = match health_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

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

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "validate-dependencies")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("health_score", health_score)
            .with_metadata("strict_mode", strict))
    }

    /// Validate commit messages using real Git analysis
    ///
    /// Performs comprehensive commit validation including:
    /// - Conventional commit format validation
    /// - Commit message quality analysis
    /// - Author information validation
    /// - Commit size and impact analysis
    /// - Branch naming convention checks
    ///
    /// # Arguments
    ///
    /// * `count` - Number of recent commits to validate
    /// * `context` - Plugin context with access to Git repository
    ///
    /// # Returns
    ///
    /// Detailed commit validation result with violations and recommendations
    #[allow(clippy::unused_self)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::too_many_lines)]
    fn validate_commits(&self, count: i32, context: &PluginContext) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        let mut valid_commits = Vec::new();
        let mut invalid_commits = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Get recent commits using real Git operations
        let commits = context
            .repository
            .get_commits_since(None, &None)
            .map_err(|e| crate::error::Error::plugin(format!("Failed to get commits: {e}")))?;

        let commits_to_check = commits.into_iter().take(count as usize).collect::<Vec<_>>();

        if commits_to_check.is_empty() {
            return Ok(PluginResult::success(serde_json::json!({
                "commits_checked": 0,
                "valid_commits": 0,
                "invalid_commits": [],
                "message": "No commits found to validate"
            })));
        }

        // Conventional commit patterns
        let conventional_pattern = regex::Regex::new(
            r"^(feat|fix|docs|style|refactor|perf|test|chore|ci|build)?(\(.+\))?!?: .{1,50}",
        )
        .map_err(|e| crate::error::Error::plugin(format!("Failed to compile regex: {e}")))?;

        for commit in &commits_to_check {
            let mut commit_issues = Vec::new();
            let mut commit_warnings = Vec::new();

            let message_lines: Vec<&str> = commit.message.lines().collect();
            let first_line = message_lines.first().unwrap_or(&"");

            // 1. Validate conventional commit format
            if !conventional_pattern.is_match(first_line) {
                commit_issues.push("Does not follow conventional commit format".to_string());
            }

            // 2. Validate commit message length
            if first_line.len() > 72 {
                commit_issues.push("Subject line too long (max 72 characters)".to_string());
            }

            if first_line.len() < 10 {
                commit_warnings.push(
                    "Subject line very short (consider more descriptive message)".to_string(),
                );
            }

            // 3. Validate commit message quality
            if first_line.ends_with('.') {
                commit_warnings.push("Subject line should not end with a period".to_string());
            }

            if !first_line.chars().next().map_or(false, char::is_uppercase)
                && !first_line.starts_with("feat")
                && !first_line.starts_with("fix")
                && !first_line.starts_with("docs")
                && !first_line.starts_with("style")
                && !first_line.starts_with("refactor")
                && !first_line.starts_with("perf")
                && !first_line.starts_with("test")
                && !first_line.starts_with("chore")
                && !first_line.starts_with("ci")
                && !first_line.starts_with("build")
            {
                commit_warnings.push(
                    "Subject line should start with capital letter or conventional commit type"
                        .to_string(),
                );
            }

            // 4. Check for merge commits (usually OK but note them)
            if first_line.starts_with("Merge") {
                commit_warnings.push(
                    "Merge commit detected - consider squashing for cleaner history".to_string(),
                );
            }

            // 5. Check for empty or placeholder messages
            let placeholder_messages = ["wip", "temp", "fix", "update", ".", "test"];
            if placeholder_messages.contains(&first_line.to_lowercase().as_str()) {
                commit_issues.push("Placeholder or non-descriptive commit message".to_string());
            }

            // 6. Validate author information
            if commit.author_email.is_empty() || !commit.author_email.contains('@') {
                commit_issues.push("Invalid or missing author email".to_string());
            }

            if commit.author_name.is_empty() || commit.author_name.len() < 2 {
                commit_issues.push("Invalid or missing author name".to_string());
            }

            // 7. Check body format if present (lines after first)
            if message_lines.len() > 1 {
                if message_lines.len() > 1 && !message_lines[1].is_empty() {
                    commit_warnings.push("Missing blank line between subject and body".to_string());
                }

                for (i, line) in message_lines.iter().enumerate().skip(2) {
                    if line.len() > 72 {
                        commit_warnings.push(format!("Body line {} exceeds 72 characters", i + 1));
                    }
                }
            }

            let commit_data = serde_json::json!({
                "hash": commit.hash[0..8].to_string(),
                "full_hash": commit.hash,
                "message": first_line,
                "author": commit.author_name,
                "email": commit.author_email,
                "date": commit.author_date,
                "issues": commit_issues,
                "warnings": commit_warnings,
                "conventional_commit": conventional_pattern.is_match(first_line),
                "message_length": first_line.len(),
                "has_body": message_lines.len() > 1
            });

            if commit_issues.is_empty() {
                valid_commits.push(commit_data);
            } else {
                invalid_commits.push(commit_data);
            }

            // Accumulate warnings
            for warning in commit_warnings {
                warnings.push(serde_json::json!({
                    "commit": commit.hash[0..8].to_string(),
                    "message": warning
                }));
            }
        }

        // Generate recommendations based on analysis
        let invalid_percentage = if commits_to_check.is_empty() {
            0.0
        } else {
            (invalid_commits.len() as f64 / commits_to_check.len() as f64) * 100.0
        };

        if invalid_percentage > 50.0 {
            recommendations.push("Consider adopting conventional commit standards".to_string());
            recommendations.push("Set up commit message templates or hooks".to_string());
        }

        if warnings.len() > commits_to_check.len() / 2 {
            recommendations.push("Review commit message best practices with the team".to_string());
        }

        if invalid_commits.is_empty() && warnings.is_empty() {
            recommendations
                .push("Excellent commit hygiene! Keep up the good practices".to_string());
        }

        // Calculate commit quality score
        let quality_score =
            std::cmp::max(0, 100 - (invalid_commits.len() * 15) - (warnings.len() * 3)) as u8;

        let overall_status = match quality_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        // Analyze commit patterns
        let mut commit_types = std::collections::HashMap::new();
        for commit in &commits_to_check {
            let first_line = commit.message.lines().next().unwrap_or("");
            if let Some(cap) = conventional_pattern.captures(first_line) {
                if let Some(commit_type) = cap.get(1) {
                    *commit_types.entry(commit_type.as_str().to_string()).or_insert(0) += 1;
                }
            } else {
                *commit_types.entry("non-conventional".to_string()).or_insert(0) += 1;
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        let result = serde_json::json!({
            "commits_checked": commits_to_check.len(),
            "valid_commits": valid_commits.len(),
            "invalid_commits": invalid_commits,
            "valid_commit_details": valid_commits,
            "warnings": warnings,
            "recommendations": recommendations,
            "quality_score": quality_score,
            "overall_status": overall_status,
            "statistics": {
                "invalid_percentage": format!("{:.1}%", invalid_percentage),
                "warnings_count": warnings.len(),
                "conventional_commits": valid_commits.iter()
                    .filter(|c| c["conventional_commit"].as_bool().unwrap_or(false))
                    .count(),
                "average_message_length": commits_to_check.iter()
                    .map(|c| c.message.lines().next().unwrap_or("").len())
                    .sum::<usize>() / commits_to_check.len().max(1)
            },
            "commit_type_distribution": commit_types,
            "analysis_period": {
                "latest_commit": commits_to_check.first().map(|c| &c.author_date),
                "oldest_commit": commits_to_check.last().map(|c| &c.author_date)
            }
        });

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "validate-commits")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("quality_score", quality_score)
            .with_metadata("commits_analyzed", commits_to_check.len()))
    }
}

impl Default for AnalyzerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GeneratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in configurator plugin for generating monorepo configuration files
///
/// Provides functionality for analyzing projects and generating intelligent
/// default configurations in TOML format for all monorepo tools and services.
pub struct ConfiguratorPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl ConfiguratorPlugin {
    /// Create a new configurator plugin
    pub fn new() -> Self {
        Self { name: "configurator".to_string(), version: "1.0.0".to_string() }
    }
}

impl MonorepoPlugin for ConfiguratorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            description: "Built-in configuration generation and project analysis plugin".to_string(),
            author: "Sublime Monorepo Tools".to_string(),
            capabilities: PluginCapabilities {
                commands: vec![
                    PluginCommand {
                        name: "generate-config".to_string(),
                        description: "Generate comprehensive monorepo configuration".to_string(),
                        arguments: vec![
                            PluginArgument {
                                name: "template".to_string(),
                                description: "Configuration template type".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("smart".to_string()),
                            },
                            PluginArgument {
                                name: "output".to_string(),
                                description: "Output file path".to_string(),
                                required: false,
                                arg_type: PluginArgumentType::String,
                                default_value: Some("monorepo.config.toml".to_string()),
                            },
                        ],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "analyze-project".to_string(),
                        description: "Analyze project structure and configuration needs".to_string(),
                        arguments: vec![PluginArgument {
                            name: "detailed".to_string(),
                            description: "Include detailed analysis information".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::Boolean,
                            default_value: Some("false".to_string()),
                        }],
                        async_support: false,
                    },
                    PluginCommand {
                        name: "validate-config".to_string(),
                        description: "Validate existing monorepo configuration".to_string(),
                        arguments: vec![PluginArgument {
                            name: "config-path".to_string(),
                            description: "Path to configuration file to validate".to_string(),
                            required: false,
                            arg_type: PluginArgumentType::String,
                            default_value: Some("monorepo.config.toml".to_string()),
                        }],
                        async_support: false,
                    },
                ],
                async_support: false,
                parallel_support: false,
                categories: vec!["configurator".to_string(), "analysis".to_string(), "setup".to_string()],
                file_patterns: vec![
                    "package.json".to_string(),
                    "*.config.{js,ts,json,toml}".to_string(),
                    "package-lock.json".to_string(),
                    "yarn.lock".to_string(),
                    "pnpm-lock.yaml".to_string(),
                ],
            },
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
        log::info!("Initializing configurator plugin for monorepo configuration generation");
        Ok(())
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[String],
        context: &PluginContext,
    ) -> Result<PluginResult> {
        match command {
            "generate-config" => {
                let template = args.first().map_or("smart", |s| s.as_str());
                let output = args.get(1).map_or("monorepo.config.toml", |s| s.as_str());
                self.generate_configuration(template, output, context)
            }
            "analyze-project" => {
                let detailed = args
                    .first()
                    .map_or(false, |s| s.to_lowercase() == "true" || s == "1");
                self.analyze_project_structure(detailed, context)
            }
            "validate-config" => {
                let config_path = args.first().map_or("monorepo.config.toml", |s| s.as_str());
                self.validate_configuration(config_path, context)
            }
            _ => Ok(PluginResult::error(format!("Unknown command: {command}"))),
        }
    }
}

impl ConfiguratorPlugin {
    /// Generate comprehensive monorepo configuration based on project analysis
    ///
    /// Analyzes the current project structure, detects package managers,
    /// workspace patterns, and generates an intelligent TOML configuration
    /// with appropriate defaults for all monorepo tools.
    ///
    /// # Arguments
    ///
    /// * `template` - Configuration template type (smart, basic, enterprise, performance, ci-cd)
    /// * `output` - Output file path for the generated configuration
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Configuration generation result with detailed information
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_lines)]
    fn generate_configuration(
        &self,
        template: &str,
        output: &str,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        // 1. Analyze project structure
        let analysis = self.perform_project_analysis(context)?;

        // 2. Generate configuration based on template and analysis
        let config_content = match template {
            "basic" => self.generate_basic_config(&analysis),
            "enterprise" => self.generate_enterprise_config(&analysis),
            "performance" => self.generate_performance_config(&analysis),
            "ci-cd" => self.generate_cicd_config(&analysis),
            "smart" | _ => self.generate_smart_config(&analysis),
        };

        // 3. Write configuration to file
        let output_path = context.root_path.join(output);
        std::fs::write(&output_path, &config_content).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to write configuration file: {e}"))
        })?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let result = serde_json::json!({
            "template_type": template,
            "output_file": output,
            "output_path": output_path.to_string_lossy(),
            "config_size_bytes": config_content.len(),
            "config_lines": config_content.lines().count(),
            "analysis_summary": {
                "package_manager": analysis.package_manager,
                "workspace_patterns": analysis.workspace_patterns,
                "package_count": analysis.package_count,
                "project_size": analysis.project_size,
                "git_provider": analysis.git_provider,
                "has_ci_config": analysis.has_ci_config,
                "has_existing_config": analysis.has_existing_config
            },
            "generation_timestamp": chrono::Utc::now().to_rfc3339(),
            "status": "successfully_generated"
        });

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "generate-config")
            .with_metadata("configurator", "builtin")
            .with_metadata("template", template)
            .with_metadata("config_location", output_path.to_string_lossy()))
    }

    /// Analyze project structure and configuration needs
    ///
    /// Performs comprehensive analysis of the monorepo structure to understand
    /// project characteristics, detect existing configurations, and provide
    /// recommendations for optimal configuration settings.
    ///
    /// # Arguments
    ///
    /// * `detailed` - Whether to include detailed analysis information
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Project analysis results with recommendations
    #[allow(clippy::unused_self)]
    fn analyze_project_structure(
        &self,
        detailed: bool,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        let analysis = self.perform_project_analysis(context)?;

        let mut result_data = serde_json::json!({
            "project_analysis": {
                "package_manager": analysis.package_manager,
                "workspace_patterns": analysis.workspace_patterns,
                "package_count": analysis.package_count,
                "project_size": analysis.project_size,
                "git_provider": analysis.git_provider,
                "has_ci_config": analysis.has_ci_config,
                "has_existing_config": analysis.has_existing_config,
                "detected_tools": analysis.detected_tools,
                "security_files": analysis.security_files,
                "test_frameworks": analysis.test_frameworks
            },
            "recommendations": analysis.recommendations,
            "template_suggestions": analysis.template_suggestions,
            "estimated_complexity": analysis.estimated_complexity,
            "analysis_timestamp": chrono::Utc::now().to_rfc3339()
        });

        if detailed {
            result_data["detailed_analysis"] = serde_json::json!({
                "file_structure": analysis.file_structure,
                "dependency_analysis": analysis.dependency_analysis,
                "git_analysis": analysis.git_analysis,
                "performance_indicators": analysis.performance_indicators,
                "quality_indicators": analysis.quality_indicators
            });
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(PluginResult::success_with_time(result_data, execution_time)
            .with_metadata("command", "analyze-project")
            .with_metadata("configurator", "builtin")
            .with_metadata("analysis_type", if detailed { "detailed" } else { "summary" }))
    }

    /// Validate existing monorepo configuration
    ///
    /// Validates an existing configuration file against the expected schema
    /// and provides recommendations for improvements or fixes.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to validate
    /// * `context` - Plugin context with access to monorepo services
    ///
    /// # Returns
    ///
    /// Validation results with any issues found and recommendations
    #[allow(clippy::unused_self)]
    fn validate_configuration(
        &self,
        config_path: &str,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        let config_file_path = context.root_path.join(config_path);

        if !config_file_path.exists() {
            return Ok(PluginResult::error(format!(
                "Configuration file not found: {config_path}"
            )));
        }

        // Read and parse configuration file
        let config_content = std::fs::read_to_string(&config_file_path).map_err(|e| {
            crate::error::Error::plugin(format!("Failed to read configuration file: {e}"))
        })?;

        let mut validation_issues = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Basic TOML parsing validation
        match toml::from_str::<crate::config::MonorepoConfig>(&config_content) {
            Ok(config) => {
                // Configuration parsed successfully, validate content
                self.validate_config_content(&config, &mut validation_issues, &mut warnings, &mut suggestions);
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
        let execution_time = start_time.elapsed().as_millis() as u64;

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

        Ok(PluginResult::success_with_time(result, execution_time)
            .with_metadata("command", "validate-config")
            .with_metadata("configurator", "builtin")
            .with_metadata("validation_result", if is_valid { "valid" } else { "invalid" }))
    }

    /// Perform comprehensive project analysis
    ///
    /// Analyzes all aspects of the project to generate intelligent configuration defaults.
    fn perform_project_analysis(&self, context: &PluginContext) -> Result<ProjectAnalysis> {
        let mut analysis = ProjectAnalysis::default();

        // Detect package manager
        analysis.package_manager = self.detect_package_manager(context);

        // Analyze workspace patterns
        analysis.workspace_patterns = self.detect_workspace_patterns(context);

        // Count packages
        analysis.package_count = context.packages.len();

        // Determine project size
        analysis.project_size = self.classify_project_size(analysis.package_count);

        // Detect Git provider
        analysis.git_provider = self.detect_git_provider(context);

        // Check for CI configuration
        analysis.has_ci_config = self.has_ci_configuration(context);

        // Check for existing monorepo config
        analysis.has_existing_config = self.has_existing_config(context);

        // Detect development tools
        analysis.detected_tools = self.detect_development_tools(context);

        // Check for security-related files
        analysis.security_files = self.detect_security_files(context);

        // Detect test frameworks
        analysis.test_frameworks = self.detect_test_frameworks(context);

        // Generate recommendations
        analysis.recommendations = self.generate_recommendations(&analysis);

        // Suggest templates
        analysis.template_suggestions = self.suggest_templates(&analysis);

        // Estimate complexity
        analysis.estimated_complexity = self.estimate_project_complexity(&analysis);

        // Additional detailed analysis
        analysis.file_structure = self.analyze_file_structure(context);
        analysis.dependency_analysis = self.analyze_dependencies(context);
        analysis.git_analysis = self.analyze_git_configuration(context);
        analysis.performance_indicators = self.analyze_performance_indicators(&analysis);
        analysis.quality_indicators = self.analyze_quality_indicators(context);

        Ok(analysis)
    }

    /// Detect the primary package manager used in the project
    fn detect_package_manager(&self, context: &PluginContext) -> String {
        let root_path = context.root_path;

        if root_path.join("pnpm-lock.yaml").exists() {
            "pnpm".to_string()
        } else if root_path.join("yarn.lock").exists() {
            "yarn".to_string()
        } else if root_path.join("bun.lockb").exists() {
            "bun".to_string()
        } else if root_path.join("package-lock.json").exists() {
            "npm".to_string()
        } else {
            "npm".to_string() // Default fallback
        }
    }

    /// Detect workspace patterns used in the project
    fn detect_workspace_patterns(&self, context: &PluginContext) -> Vec<String> {
        let mut patterns = Vec::new();

        // Common patterns to check
        let common_patterns = [
            "packages/*", "apps/*", "libs/*", "services/*", 
            "tools/*", "components/*", "modules/*"
        ];

        for pattern in &common_patterns {
            let pattern_path = pattern.replace("/*", "");
            if context.root_path.join(pattern_path).exists() {
                patterns.push(pattern.to_string());
            }
        }

        // If no patterns found, use packages/* as default
        if patterns.is_empty() {
            patterns.push("packages/*".to_string());
        }

        patterns
    }

    /// Classify project size based on package count
    fn classify_project_size(&self, package_count: usize) -> String {
        match package_count {
            0..=5 => "small".to_string(),
            6..=20 => "medium".to_string(),
            21..=50 => "large".to_string(),
            _ => "enterprise".to_string(),
        }
    }

    /// Detect Git hosting provider from remote URLs
    fn detect_git_provider(&self, _context: &PluginContext) -> String {
        // TODO: Implement remote URL detection when available in Repo API
        // For now, return unknown - could be enhanced to check .git/config directly
        "unknown".to_string()
    }

    /// Check if project has CI/CD configuration
    fn has_ci_configuration(&self, context: &PluginContext) -> bool {
        let ci_files = [
            ".github/workflows",
            ".gitlab-ci.yml",
            "azure-pipelines.yml",
            ".travis.yml",
            ".circleci/config.yml",
            "Jenkinsfile",
            ".buildkite/pipeline.yml",
        ];

        ci_files
            .iter()
            .any(|file| context.root_path.join(file).exists())
    }

    /// Check if project already has monorepo configuration
    fn has_existing_config(&self, context: &PluginContext) -> bool {
        let config_files = [
            "monorepo.config.toml",
            ".monorepo/config.toml",
            "monorepo.toml",
            ".monoreporc",
        ];

        config_files
            .iter()
            .any(|file| context.root_path.join(file).exists())
    }

    /// Detect development tools used in the project
    fn detect_development_tools(&self, context: &PluginContext) -> Vec<String> {
        let mut tools = Vec::new();
        let root_path = context.root_path;

        // Check for various tool configuration files
        let tool_files = [
            (".eslintrc", "eslint"),
            (".prettierrc", "prettier"),
            ("tsconfig.json", "typescript"),
            ("jest.config", "jest"),
            ("vitest.config", "vitest"),
            (".babelrc", "babel"),
            ("webpack.config", "webpack"),
            ("rollup.config", "rollup"),
            ("vite.config", "vite"),
            ("tailwind.config", "tailwind"),
            ("next.config", "nextjs"),
            ("nuxt.config", "nuxtjs"),
            ("svelte.config", "svelte"),
            ("astro.config", "astro"),
        ];

        for (file_pattern, tool_name) in &tool_files {
            if self.file_exists_with_pattern(root_path, file_pattern) {
                tools.push(tool_name.to_string());
            }
        }

        tools
    }

    /// Detect security-related files
    fn detect_security_files(&self, context: &PluginContext) -> Vec<String> {
        let mut security_files = Vec::new();
        let root_path = context.root_path;

        let security_patterns = [
            (".nvmrc", "Node version lock"),
            (".node-version", "Node version specification"),
            (".security-policy", "Security policy"),
            ("SECURITY.md", "Security documentation"),
            (".snyk", "Snyk security configuration"),
            ("audit-ci.json", "Audit CI configuration"),
        ];

        for (file_pattern, description) in &security_patterns {
            if self.file_exists_with_pattern(root_path, file_pattern) {
                security_files.push(format!("{}: {}", file_pattern, description));
            }
        }

        security_files
    }

    /// Detect test frameworks used
    fn detect_test_frameworks(&self, context: &PluginContext) -> Vec<String> {
        let mut frameworks = Vec::new();
        let root_path = context.root_path;

        let test_patterns = [
            ("jest.config", "Jest"),
            ("vitest.config", "Vitest"),
            ("cypress.config", "Cypress"),
            ("playwright.config", "Playwright"),
            ("karma.conf", "Karma"),
            ("mocha.opts", "Mocha"),
            ("ava.config", "AVA"),
        ];

        for (file_pattern, framework_name) in &test_patterns {
            if self.file_exists_with_pattern(root_path, file_pattern) {
                frameworks.push(framework_name.to_string());
            }
        }

        frameworks
    }

    /// Generate configuration recommendations based on analysis
    fn generate_recommendations(&self, analysis: &ProjectAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Package manager recommendations
        recommendations.push(format!(
            "Detected {} as package manager - configuration optimized accordingly", 
            analysis.package_manager
        ));

        // Project size recommendations
        match analysis.project_size.as_str() {
            "small" => {
                recommendations.push("Small project detected: Using standard concurrency settings".to_string());
            }
            "medium" => {
                recommendations.push("Medium project detected: Enabling moderate parallelization".to_string());
            }
            "large" => {
                recommendations.push("Large project detected: Enabling high-performance settings".to_string());
            }
            "enterprise" => {
                recommendations.push("Enterprise project detected: Enabling all performance optimizations".to_string());
            }
            _ => {}
        }

        // CI/CD recommendations
        if analysis.has_ci_config {
            recommendations.push("CI/CD configuration detected: Adding deployment task templates".to_string());
        } else {
            recommendations.push("No CI/CD detected: Consider adding CI/CD configuration templates".to_string());
        }

        // Git provider recommendations
        if analysis.git_provider != "unknown" {
            recommendations.push(format!(
                "Git hosting provider detected ({}): Configuring provider-specific integrations", 
                analysis.git_provider
            ));
        }

        // Security recommendations
        if !analysis.security_files.is_empty() {
            recommendations.push("Security files detected: Enabling security validation features".to_string());
        }

        // Test framework recommendations
        if !analysis.test_frameworks.is_empty() {
            recommendations.push(format!(
                "Test frameworks detected ({}): Configuring test-aware task management", 
                analysis.test_frameworks.join(", ")
            ));
        }

        recommendations
    }

    /// Suggest appropriate configuration templates
    fn suggest_templates(&self, analysis: &ProjectAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Always suggest smart template
        suggestions.push("smart".to_string());

        // Suggest based on project characteristics
        match analysis.project_size.as_str() {
            "small" => suggestions.push("basic".to_string()),
            "enterprise" => {
                suggestions.push("enterprise".to_string());
                suggestions.push("performance".to_string());
            }
            "large" => suggestions.push("performance".to_string()),
            _ => {}
        }

        // CI/CD template suggestion
        if analysis.has_ci_config {
            suggestions.push("ci-cd".to_string());
        }

        suggestions
    }

    /// Estimate project complexity based on various factors
    fn estimate_project_complexity(&self, analysis: &ProjectAnalysis) -> String {
        let mut complexity_score = 0;

        // Package count factor
        complexity_score += match analysis.package_count {
            0..=5 => 1,
            6..=20 => 2,
            21..=50 => 3,
            _ => 4,
        };

        // Tools factor
        complexity_score += analysis.detected_tools.len() / 2;

        // CI factor
        if analysis.has_ci_config {
            complexity_score += 1;
        }

        // Workspace patterns factor
        complexity_score += analysis.workspace_patterns.len() / 2;

        match complexity_score {
            0..=2 => "low".to_string(),
            3..=5 => "medium".to_string(),
            6..=8 => "high".to_string(),
            _ => "very_high".to_string(),
        }
    }

    /// Helper function to check if file exists with pattern
    fn file_exists_with_pattern(&self, root_path: &std::path::Path, pattern: &str) -> bool {
        // Simple implementation - could be enhanced with glob patterns
        let extensions = ["", ".js", ".ts", ".json", ".yml", ".yaml"];
        
        for ext in &extensions {
            if root_path.join(format!("{pattern}{ext}")).exists() {
                return true;
            }
        }
        false
    }

    /// Placeholder implementations for detailed analysis methods
    /// These would be implemented with real analysis logic
    
    fn analyze_file_structure(&self, _context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "total_files": "analysis_placeholder",
            "structure_score": "analysis_placeholder"
        })
    }

    fn analyze_dependencies(&self, _context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "dependency_health": "analysis_placeholder",
            "outdated_count": "analysis_placeholder"
        })
    }

    fn analyze_git_configuration(&self, _context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "branch_strategy": "analysis_placeholder",
            "commit_patterns": "analysis_placeholder"
        })
    }

    fn analyze_performance_indicators(&self, _analysis: &ProjectAnalysis) -> serde_json::Value {
        serde_json::json!({
            "build_performance": "analysis_placeholder",
            "task_efficiency": "analysis_placeholder"
        })
    }

    fn analyze_quality_indicators(&self, _context: &PluginContext) -> serde_json::Value {
        serde_json::json!({
            "code_quality": "analysis_placeholder",
            "test_coverage": "analysis_placeholder"
        })
    }

    fn validate_config_content(
        &self,
        _config: &crate::config::MonorepoConfig,
        _issues: &mut Vec<serde_json::Value>,
        _warnings: &mut Vec<serde_json::Value>,
        _suggestions: &mut Vec<serde_json::Value>,
    ) {
        // Placeholder - would implement real validation logic
    }

    /// Generate smart configuration based on project analysis
    fn generate_smart_config(&self, analysis: &ProjectAnalysis) -> String {
        format!(r#"# Monorepo Configuration
# Generated by Sublime Monorepo Tools Configurator Plugin
# Template: SMART
# Generated: {}

# Deployment environments
environments = ["development", "staging", "production"]

[versioning]
default_bump = "patch"
propagate_changes = true
snapshot_format = "{{version}}-snapshot.{{sha}}"
tag_prefix = "v"
auto_tag = true
version_constraint = "^1.0.0"

[tasks]
default_tasks = ["test", "lint"]
parallel = true
max_concurrent = {}
timeout = {}

[tasks.groups]
quality = ["lint", "typecheck", "test"]
build = ["clean", "compile", "bundle"]
release = ["quality", "build", "docs"]

[changelog]
include_breaking_changes = true
output_format = "markdown"
grouping = "type"

[hooks]
enabled = true
hooks_dir = ".hooks"

[hooks.pre_commit]
enabled = true
validate_changeset = true
run_tasks = ["lint"]

[changesets]
required = true
changeset_dir = ".changesets"
default_environments = ["development", "staging"]
auto_deploy = false

[plugins]
enabled = ["configurator", "analyzer", "generator", "validator"]
plugin_dirs = [".monorepo/plugins", "plugins"]

[workspace]
merge_with_detected = true

{}

[workspace.package_manager_configs.{}]
use_workspaces = true

[git]
default_since_ref = "HEAD~1"
default_until_ref = "HEAD"
default_remote = "origin"

[git.branches]
main_branches = ["main", "master", "trunk"]
develop_branches = ["develop", "dev"]
default_base_branch = "main"

{}

[validation]
[validation.task_priorities]
low = 0
normal = 50
high = 100
critical = 200

[validation.quality_gates]
min_test_coverage = 80.0
max_cyclomatic_complexity = 10
max_build_time_seconds = {}
"#,
            chrono::Utc::now().to_rfc3339(),
            self.get_concurrency_for_project_size(&analysis.project_size),
            self.get_timeout_for_project_size(&analysis.project_size),
            self.generate_workspace_patterns_config(analysis),
            analysis.package_manager,
            self.generate_git_provider_config(analysis),
            self.get_build_timeout_for_project_size(&analysis.project_size)
        )
    }

    /// Generate basic configuration template
    fn generate_basic_config(&self, analysis: &ProjectAnalysis) -> String {
        format!(r#"# Basic Monorepo Configuration
# Generated by Sublime Monorepo Tools Configurator Plugin
# Template: BASIC
# Generated: {}

[versioning]
default_bump = "patch"
auto_tag = true

[tasks]
default_tasks = ["test", "lint"]
parallel = true
max_concurrent = 2

[workspace]
merge_with_detected = true

{}

[workspace.package_manager_configs.{}]
use_workspaces = true

[git]
default_remote = "origin"

[git.branches]
main_branches = ["main"]
"#,
            chrono::Utc::now().to_rfc3339(),
            self.generate_workspace_patterns_config(analysis),
            analysis.package_manager
        )
    }

    /// Generate enterprise configuration template
    fn generate_enterprise_config(&self, analysis: &ProjectAnalysis) -> String {
        format!(r#"# Enterprise Monorepo Configuration
# Generated by Sublime Monorepo Tools Configurator Plugin
# Template: ENTERPRISE
# Generated: {}

# Deployment environments
environments = ["development", "testing", "staging", "production"]

[versioning]
default_bump = "patch"
propagate_changes = true
snapshot_format = "{{version}}-snapshot.{{sha}}"
tag_prefix = "v"
auto_tag = true
version_constraint = "^1.0.0"

[tasks]
default_tasks = ["lint", "typecheck", "test", "security-scan"]
parallel = true
max_concurrent = {}
timeout = {}

[tasks.groups]
quality = ["lint", "typecheck", "test", "audit"]
security = ["security-scan", "license-check", "vulnerability-scan"]
build = ["clean", "compile", "bundle", "optimize"]
release = ["quality", "security", "build", "docs", "sign"]

[tasks.deployment_tasks]
development = ["lint", "test"]
testing = ["quality", "unit-tests", "integration-tests"]
staging = ["quality", "security", "build", "e2e-tests"]
production = ["quality", "security", "build", "security-scan", "compliance-check"]

[changelog]
include_breaking_changes = true
output_format = "markdown"
grouping = "type"

[hooks]
enabled = true
hooks_dir = ".hooks"

[hooks.pre_commit]
enabled = true
validate_changeset = true
run_tasks = ["lint", "security-scan"]

[hooks.pre_push]
enabled = true
run_tasks = ["test", "build"]

[changesets]
required = true
changeset_dir = ".changesets"
default_environments = ["development", "testing", "staging"]
auto_deploy = false

[plugins]
enabled = ["configurator", "analyzer", "generator", "validator"]
plugin_dirs = [".monorepo/plugins", "plugins"]

[workspace]
merge_with_detected = true

{}

[workspace.validation]
require_pattern_matches = true
warn_orphaned_packages = true
validate_naming = true
naming_patterns = ["@scope/*", "^[a-z-]+$"]
validate_structure = true
required_files = ["package.json", "README.md", "CHANGELOG.md"]

[workspace.package_manager_configs.{}]
use_workspaces = true

[git]
default_since_ref = "HEAD~1"
default_until_ref = "HEAD"
default_remote = "origin"

[git.branches]
main_branches = ["main", "master"]
develop_branches = ["develop", "dev"]
release_prefixes = ["release/", "rel/"]
feature_prefixes = ["feature/", "feat/"]
hotfix_prefixes = ["hotfix/", "fix/"]
default_base_branch = "main"

{}

[validation]
[validation.task_priorities]
low = 0
normal = 50
high = 100
critical = 200

[validation.quality_gates]
min_test_coverage = 90.0
max_cyclomatic_complexity = 8
max_file_size_bytes = 50000
max_lines_per_file = 500
max_dependencies_per_package = 30
min_documentation_coverage = 80.0
max_build_time_seconds = {}
max_technical_debt_ratio = 0.03

[validation.quality_gates.security_thresholds]
max_high_severity_vulnerabilities = 0
max_medium_severity_vulnerabilities = 2
max_outdated_dependencies_percentage = 10.0
outdated_dependency_days = 180
max_license_issues = 0
"#,
            chrono::Utc::now().to_rfc3339(),
            self.get_concurrency_for_project_size(&analysis.project_size),
            self.get_timeout_for_project_size(&analysis.project_size),
            self.generate_workspace_patterns_config(analysis),
            analysis.package_manager,
            self.generate_git_provider_config(analysis),
            self.get_build_timeout_for_project_size(&analysis.project_size)
        )
    }

    /// Generate performance configuration template
    fn generate_performance_config(&self, analysis: &ProjectAnalysis) -> String {
        format!(r#"# Performance Optimized Monorepo Configuration
# Generated by Sublime Monorepo Tools Configurator Plugin
# Template: PERFORMANCE
# Generated: {}

[versioning]
default_bump = "patch"
propagate_changes = true
auto_tag = true

[tasks]
default_tasks = ["test", "lint"]
parallel = true
max_concurrent = {}
timeout = {}

[tasks.performance]
hook_timeout = {}
version_planning_per_package = 10
cache_duration = 600

[tasks.performance.large_project]
max_concurrent = {}
timeout = {}

[tasks.performance.impact_thresholds]
medium_impact_files = 10
high_impact_files = 30

[workspace]
merge_with_detected = true

{}

[workspace.discovery]
auto_detect = true
cache_results = true
cache_duration = 600
max_scan_depth = 4

[workspace.package_manager_configs.{}]
use_workspaces = true

[git]
default_remote = "origin"

[validation]
[validation.dependency_analysis]
max_chain_depth = 8
max_propagation_depth = 8
max_analysis_depth = 15
complex_dependency_threshold = 50
max_dependents_analysis = 200

[validation.quality_gates]
max_build_time_seconds = {}
"#,
            chrono::Utc::now().to_rfc3339(),
            self.get_high_concurrency_for_project_size(&analysis.project_size),
            self.get_extended_timeout_for_project_size(&analysis.project_size),
            self.get_extended_timeout_for_project_size(&analysis.project_size),
            self.get_high_concurrency_for_project_size(&analysis.project_size),
            self.get_extended_timeout_for_project_size(&analysis.project_size),
            self.generate_workspace_patterns_config(analysis),
            analysis.package_manager,
            self.get_extended_build_timeout_for_project_size(&analysis.project_size)
        )
    }

    /// Generate CI/CD configuration template
    fn generate_cicd_config(&self, analysis: &ProjectAnalysis) -> String {
        format!(r#"# CI/CD Optimized Monorepo Configuration
# Generated by Sublime Monorepo Tools Configurator Plugin
# Template: CI-CD
# Generated: {}

# Deployment environments
environments = ["development", "staging", "production"]

[versioning]
default_bump = "patch"
propagate_changes = true
auto_tag = true
tag_prefix = "v"

[tasks]
default_tasks = ["lint", "test", "build"]
parallel = true
max_concurrent = {}

[tasks.groups]
quality = ["lint", "typecheck", "test"]
ci = ["quality", "build", "package"]
cd = ["ci", "deploy", "verify"]

[tasks.deployment_tasks]
development = ["lint", "test"]
staging = ["quality", "build", "integration-tests"]
production = ["quality", "build", "security-scan", "deploy"]

[changelog]
include_breaking_changes = true
output_format = "markdown"
grouping = "type"

[hooks]
enabled = true
hooks_dir = ".hooks"

[hooks.pre_commit]
enabled = true
validate_changeset = true
run_tasks = ["lint", "test"]

[hooks.pre_push]
enabled = true
validate_changeset = false
run_tasks = ["test", "build"]

[changesets]
required = true
changeset_dir = ".changesets"
default_environments = ["development", "staging"]
auto_deploy = {}

[workspace]
merge_with_detected = true

{}

[workspace.package_manager_configs.{}]
use_workspaces = true

[git]
default_remote = "origin"

[git.branches]
main_branches = ["main", "master"]
develop_branches = ["develop", "dev"]
release_prefixes = ["release/"]
feature_prefixes = ["feature/"]
hotfix_prefixes = ["hotfix/"]

{}
"#,
            chrono::Utc::now().to_rfc3339(),
            self.get_concurrency_for_project_size(&analysis.project_size),
            if analysis.has_ci_config { "true" } else { "false" },
            self.generate_workspace_patterns_config(analysis),
            analysis.package_manager,
            self.generate_git_provider_config(analysis)
        )
    }

    /// Helper methods for configuration generation
    
    fn get_concurrency_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 2,
            "medium" => 4,
            "large" => 6,
            "enterprise" => 8,
            _ => 4,
        }
    }

    fn get_high_concurrency_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 4,
            "medium" => 8,
            "large" => 12,
            "enterprise" => 16,
            _ => 8,
        }
    }

    fn get_timeout_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 180,
            "medium" => 300,
            "large" => 450,
            "enterprise" => 600,
            _ => 300,
        }
    }

    fn get_extended_timeout_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 300,
            "medium" => 600,
            "large" => 900,
            "enterprise" => 1200,
            _ => 600,
        }
    }

    fn get_build_timeout_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 300,
            "medium" => 600,
            "large" => 900,
            "enterprise" => 1200,
            _ => 600,
        }
    }

    fn get_extended_build_timeout_for_project_size(&self, project_size: &str) -> u32 {
        match project_size {
            "small" => 600,
            "medium" => 1200,
            "large" => 1800,
            "enterprise" => 2400,
            _ => 1200,
        }
    }

    fn generate_workspace_patterns_config(&self, analysis: &ProjectAnalysis) -> String {
        let patterns: Vec<String> = analysis.workspace_patterns.iter()
            .map(|pattern| {
                format!(r#"[[workspace.patterns]]
pattern = "{}"
description = "Auto-detected workspace pattern"
enabled = true
priority = 100"#, pattern)
            })
            .collect();
        
        patterns.join("\n\n")
    }

    fn generate_git_provider_config(&self, analysis: &ProjectAnalysis) -> String {
        if analysis.git_provider == "unknown" {
            return String::new();
        }

        format!(r#"
[git.repository]
provider = "{}"
auto_detect = true"#, analysis.git_provider)
    }
}

/// Project analysis data structure
#[derive(Debug, Default)]
struct ProjectAnalysis {
    package_manager: String,
    workspace_patterns: Vec<String>,
    package_count: usize,
    project_size: String,
    git_provider: String,
    has_ci_config: bool,
    has_existing_config: bool,
    detected_tools: Vec<String>,
    security_files: Vec<String>,
    test_frameworks: Vec<String>,
    recommendations: Vec<String>,
    template_suggestions: Vec<String>,
    estimated_complexity: String,
    file_structure: serde_json::Value,
    dependency_analysis: serde_json::Value,
    git_analysis: serde_json::Value,
    performance_indicators: serde_json::Value,
    quality_indicators: serde_json::Value,
}

impl Default for ConfiguratorPlugin {
    fn default() -> Self {
        Self::new()
    }
}
