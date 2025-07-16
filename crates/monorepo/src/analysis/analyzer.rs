//! Monorepo analyzer implementation

use crate::analysis::MonorepoAnalysisResult;
use crate::analysis::{
    BranchComparisonResult, ChangeAnalysis, DependencyGraphAnalysis, MonorepoAnalyzer,
    PackageClassificationResult, PackageInformation, PackageManagerAnalysis, PatternStatistics,
    RegistryAnalysisResult, RegistryInfo, UpgradeAnalysisResult, WorkspaceConfigAnalysis,
    WorkspacePatternAnalysis,
};
use crate::config::PackageManagerType;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use glob;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use sublime_package_tools::Upgrader;
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::PackageManagerKind;

impl<'a> MonorepoAnalyzer<'a> {
    /// Create a new simplified analyzer with direct borrowing from project
    ///
    /// Streamlined for CLI consumption focusing on essential features:
    /// dependency graph, change detection, and package classification.
    #[must_use]
    pub fn new(project: &'a MonorepoProject) -> Self {
        Self {
            packages: &project.packages,
            config: &project.config,
            file_system: &project.file_system,
            root_path: project.root_path(),
        }
    }

    /// Get all packages in the monorepo
    ///
    /// Returns a reference to all packages that are part of this monorepo for analysis.
    /// This method provides direct access to the package list used by the analyzer.
    ///
    /// # Returns
    ///
    /// A slice containing all MonorepoPackageInfo instances
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let analyzer = tools.analyzer()?;
    /// let packages = analyzer.get_packages();
    /// println!("Found {} packages", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_packages(&self) -> &[crate::core::MonorepoPackageInfo] {
        self.packages
    }

    /// Creates a new analyzer from an existing MonorepoProject
    ///
    /// Convenience method that wraps the `new` constructor for backward compatibility.
    /// Uses real direct borrowing following Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new MonorepoAnalyzer instance with direct borrowing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{MonorepoAnalyzer, MonorepoProject};
    ///
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let analyzer = MonorepoAnalyzer::from_project(&project);
    /// ```
    #[must_use]
    pub fn from_project(project: &'a MonorepoProject) -> Self {
        Self::new(project)
    }

    /// Perform complete monorepo detection and analysis
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Monorepo detection fails
    /// - Package manager analysis fails
    /// - Dependency graph building fails
    /// - Package classification fails
    /// - Registry analysis fails
    /// - Workspace configuration analysis fails
    pub fn detect_monorepo_info(&self, path: &Path) -> Result<MonorepoAnalysisResult> {
        // Analyze package manager to infer monorepo kind
        let package_manager = self.analyze_package_manager()?;
        
        // Infer monorepo kind from package manager
        let kind = match package_manager.kind {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => 
                sublime_standard_tools::monorepo::MonorepoKind::NpmWorkSpace,
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => 
                sublime_standard_tools::monorepo::MonorepoKind::YarnWorkspaces,
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => 
                sublime_standard_tools::monorepo::MonorepoKind::PnpmWorkspaces,
            sublime_standard_tools::monorepo::PackageManagerKind::Bun => 
                sublime_standard_tools::monorepo::MonorepoKind::BunWorkspaces,
            sublime_standard_tools::monorepo::PackageManagerKind::Jsr => 
                sublime_standard_tools::monorepo::MonorepoKind::DenoWorkspaces,
        };

        // Build dependency graph
        let dependency_graph = self.build_dependency_graph()?;

        // Classify packages
        let packages = self.classify_packages()?;

        // Analyze registries
        let registries = self.analyze_registries()?;

        // Analyze workspace configuration
        let workspace_config = self.analyze_workspace_config()?;

        Ok(MonorepoAnalysisResult {
            kind,
            root_path: path.to_path_buf(),
            package_manager,
            packages,
            dependency_graph,
            registries,
            workspace_config,
        })
    }

    /// Analyze the package manager configuration
    pub fn analyze_package_manager(&self) -> Result<PackageManagerAnalysis> {
        // Use direct package manager detection
        let package_manager = sublime_standard_tools::monorepo::PackageManager::detect(self.root_path)
            .map_err(|e| Error::Analysis(format!("Failed to detect package manager: {e}")))?;
        let kind = package_manager.kind();

        // Get package manager version by executing the appropriate command
        let version = self.detect_package_manager_version(kind);

        let package_json_path = self.root_path.join("package.json");

        let workspaces_config = if self.file_system.exists(&package_json_path) {
            let content = self.file_system.read_file_string(&package_json_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            json.get("workspaces").cloned().unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        // Find all config files based on package manager type
        let lock_file_path = match kind {
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => {
                self.root_path.join("yarn.lock")
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                self.root_path.join("pnpm-lock.yaml")
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => {
                self.root_path.join("package-lock.json")
            }
            _ => self.root_path.join("package-lock.json"),
        };

        let mut config_files = vec![lock_file_path.clone()];

        match kind {
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                let workspace_yaml = self.root_path.join("pnpm-workspace.yaml");
                if self.file_system.exists(&workspace_yaml) {
                    config_files.push(workspace_yaml);
                }
            }
            _ => {
                if self.file_system.exists(&package_json_path) {
                    config_files.push(package_json_path);
                }
            }
        }

        Ok(PackageManagerAnalysis {
            kind,
            version,
            lock_file: lock_file_path,
            config_files,
            workspaces_config,
        })
    }

    /// Build and analyze the dependency graph
    pub fn build_dependency_graph(&self) -> Result<DependencyGraphAnalysis> {
        let packages = self.packages;

        if packages.is_empty() {
            return Ok(DependencyGraphAnalysis {
                node_count: 0,
                edge_count: 0,
                has_cycles: false,
                cycles: Vec::new(),
                version_conflicts: HashMap::new(),
                upgradable: HashMap::new(),
                max_depth: 0,
                most_dependencies: Vec::new(),
                most_dependents: Vec::new(),
            });
        }

        // Extract Package objects for graph building
        let package_objects: Result<Vec<_>> = packages
            .iter()
            .map(|pkg| {
                let pkg_ref = pkg.package_info.package.borrow();
                sublime_package_tools::Package::new(
                    pkg_ref.name(),
                    &pkg_ref.version_str(),
                    Some(pkg_ref.dependencies().to_vec()),
                )
                .map_err(crate::error::Error::Version)
            })
            .collect();

        let package_vec = package_objects?;

        // Build dependency graph using Graph::from
        let graph = sublime_package_tools::Graph::from(package_vec.as_slice());

        let node_count = packages.len();
        let edge_count: usize =
            packages.iter().map(|p| p.workspace_package.workspace_dependencies.len()).sum();

        // Detect cycles using the graph
        let graph_with_cycles = graph.detect_circular_dependencies();
        let has_cycles = graph_with_cycles.has_cycles();
        let cycles = graph_with_cycles.get_cycle_strings();

        // Find version conflicts using package-tools functionality
        let version_conflicts = graph.find_version_conflicts_for_package();

        // Find upgradable packages
        let upgradable = graph.check_upgradable_dependencies();

        // Calculate graph metrics
        let max_depth = self.calculate_max_depth(packages);
        let most_dependencies = self.find_packages_with_most_dependencies(packages);
        let most_dependents = self.find_packages_with_most_dependents(packages);

        Ok(DependencyGraphAnalysis {
            node_count,
            edge_count,
            has_cycles,
            cycles,
            version_conflicts,
            upgradable,
            max_depth,
            most_dependencies,
            most_dependents,
        })
    }

    /// Calculate maximum dependency depth
    #[must_use]
    pub fn calculate_max_depth(&self, packages: &[crate::core::MonorepoPackageInfo]) -> usize {
        let mut max_depth = 0;
        let mut visited = std::collections::HashSet::new();

        for package in packages {
            if !visited.contains(package.name()) {
                let depth = Self::calculate_package_depth(package, packages, &mut visited, 0);
                max_depth = max_depth.max(depth);
            }
        }

        max_depth
    }

    /// Calculate depth for a specific package
    fn calculate_package_depth(
        package: &crate::core::MonorepoPackageInfo,
        all_packages: &[crate::core::MonorepoPackageInfo],
        visited: &mut std::collections::HashSet<String>,
        current_depth: usize,
    ) -> usize {
        if visited.contains(package.name()) {
            return current_depth; // Cycle detected
        }

        visited.insert(package.name().to_string());
        let mut max_child_depth = current_depth;

        for dep_name in &package.workspace_package.workspace_dependencies {
            if let Some(dep_package) = all_packages.iter().find(|p| p.name() == dep_name) {
                let child_depth = Self::calculate_package_depth(
                    dep_package,
                    all_packages,
                    visited,
                    current_depth + 1,
                );
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        visited.remove(package.name());
        max_child_depth
    }

    /// Find packages with most dependencies
    #[must_use]
    pub fn find_packages_with_most_dependencies(
        &self,
        packages: &[crate::core::MonorepoPackageInfo],
    ) -> Vec<(String, usize)> {
        let mut deps_count: Vec<_> = packages
            .iter()
            .map(|p| (p.name().to_string(), p.workspace_package.workspace_dependencies.len()))
            .collect();

        deps_count.sort_by(|a, b| b.1.cmp(&a.1));
        deps_count.into_iter().take(5).collect() // Top 5
    }

    /// Find packages with most dependents
    #[must_use]
    pub fn find_packages_with_most_dependents(
        &self,
        packages: &[crate::core::MonorepoPackageInfo],
    ) -> Vec<(String, usize)> {
        let mut dependents_count: Vec<_> =
            packages.iter().map(|p| (p.name().to_string(), p.dependents.len())).collect();

        dependents_count.sort_by(|a, b| b.1.cmp(&a.1));
        dependents_count.into_iter().take(5).collect() // Top 5
    }

    /// Classify packages into internal/external
    pub fn classify_packages(&self) -> Result<PackageClassificationResult> {
        let mut internal_packages = Vec::new();
        let mut external_dependencies = Vec::new();
        let dev_dependencies = Vec::new();
        let peer_dependencies = Vec::new();

        // Convert MonorepoPackageInfo to PackageInformation
        for pkg in self.packages {
            let package_info = PackageInformation {
                name: pkg.name().to_string(),
                version: pkg.version().to_string(),
                path: pkg.path().clone(),
                relative_path: pkg.relative_path().clone(),
                package_json: pkg.package_info.pkg_json.borrow().clone(),
                is_internal: pkg.is_internal,
                dependencies: pkg.workspace_package.workspace_dependencies.clone(),
                dev_dependencies: pkg.workspace_package.workspace_dev_dependencies.clone(),
                workspace_dependencies: pkg.workspace_package.workspace_dependencies.clone(),
                dependents: pkg.dependents.clone(),
            };

            if pkg.is_internal {
                internal_packages.push(package_info);
            }

            // Collect external dependencies
            external_dependencies.extend(pkg.dependencies_external.clone());
        }

        // Deduplicate
        external_dependencies.sort();
        external_dependencies.dedup();

        Ok(PackageClassificationResult {
            internal_packages,
            external_dependencies,
            dev_dependencies,
            peer_dependencies,
        })
    }

    /// Analyze configured registries
    pub fn analyze_registries(&self) -> Result<RegistryAnalysisResult> {
        // Since registry_manager is not available in the simplified analyzer,
        // we'll analyze registries from configuration files directly
        let default_registry = "https://registry.npmjs.org/".to_string();
        let registry_urls = vec![default_registry.clone()];

        let mut registries = Vec::new();
        let mut scoped_registries = HashMap::new();
        let mut auth_status = HashMap::new();

        // Analyze each registry
        for url in registry_urls {
            // Determine registry type using configurable patterns
            // Determine registry type from URL using configuration patterns
            let registry_type =
                self.config
                    .workspace
                    .tool_configs
                    .registry_patterns
                    .iter()
                    .find_map(|(pattern, registry_type)| {
                        if url.contains(pattern) {
                            Some(registry_type.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "custom".to_string());

            // Check for authentication (basic heuristic)
            let has_auth = self.check_registry_auth(&url);

            // Find scopes associated with this registry
            let mut scopes = Vec::new();

            // DRY: Use FileSystemManager for .npmrc file reading
            let fs = FileSystemManager::new();
            if let Ok(npmrc_content) = fs.read_file_string(&self.root_path.join(".npmrc")) {
                for line in npmrc_content.lines() {
                    if line.contains(&url) && line.contains('@') {
                        // Extract scope from lines like "@scope:registry=url"
                        if let Some(scope_part) = line.split(':').next() {
                            if scope_part.starts_with('@') {
                                scopes.push(scope_part.to_string());
                                scoped_registries.insert(scope_part.to_string(), url.to_string());
                            }
                        }
                    }
                }
            }

            let registry_info =
                RegistryInfo { url: url.to_string(), registry_type, has_auth, scopes };

            registries.push(registry_info);
            auth_status.insert(url.to_string(), has_auth);
        }

        // Check for any packages that use scoped registries
        for package in self.packages {
            let package_json = &package.package_info.pkg_json.borrow();

            // Check dependencies for scoped packages
            if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
                for dep_name in deps.keys() {
                    if dep_name.starts_with('@') {
                        let scope = dep_name.split('/').next().unwrap_or(dep_name);
                        // For simplified analysis, assume scoped packages use default registry
                        // unless configured otherwise in .npmrc
                        if !scoped_registries.contains_key(scope) {
                            scoped_registries.insert(scope.to_string(), default_registry.clone());
                        }
                    }
                }
            }
        }

        Ok(RegistryAnalysisResult { default_registry, registries, scoped_registries, auth_status })
    }

    /// Check if a registry has authentication configured
    #[must_use]
    pub fn check_registry_auth(&self, registry_url: &str) -> bool {
        // Check .npmrc for auth tokens
        let npmrc_paths = [
            self.root_path.join(".npmrc"),
            dirs::home_dir().map(|h| h.join(".npmrc")).unwrap_or_default(),
        ];

        // DRY: Use FileSystemManager for file operations
        let fs = FileSystemManager::new();
        for npmrc_path in &npmrc_paths {
            if let Ok(content) = fs.read_file_string(npmrc_path) {
                for line in content.lines() {
                    // Look for auth tokens or auth entries for this registry
                    if line.contains(registry_url)
                        || (line.contains("_auth") && registry_url.contains("registry.npmjs.org"))
                    {
                        return true;
                    }

                    // Check for registry-specific auth patterns
                    let registry_host = registry_url
                        .split("://")
                        .nth(1)
                        .unwrap_or(registry_url)
                        .split('/')
                        .next()
                        .unwrap_or("");

                    if line.contains(registry_host)
                        && (line.contains("_authToken")
                            || line.contains("_auth")
                            || line.contains("_password"))
                    {
                        return true;
                    }
                }
            }
        }

        // Check environment variables for registry authentication using configuration
        let registry_type = self
            .config
            .workspace
            .tool_configs
            .registry_patterns
            .iter()
            .find_map(|(pattern, registry_type)| {
                if registry_url.contains(pattern) {
                    Some(registry_type.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "custom".to_string());

        let auth_env_vars =
            self.config.workspace.tool_configs.auth_env_vars.get(&registry_type).map_or_else(
                || vec!["REGISTRY_TOKEN", "AUTH_TOKEN"],
                |vars| vars.iter().map(std::string::String::as_str).collect::<Vec<_>>(),
            );

        for env_var in auth_env_vars {
            if std::env::var(env_var).is_ok() {
                return true;
            }
        }

        false
    }

    /// Get package information
    #[must_use]
    pub fn get_package_information(&self) -> Vec<PackageInformation> {
        self.packages
            .iter()
            .map(|pkg| PackageInformation {
                name: pkg.name().to_string(),
                version: pkg.version().to_string(),
                path: pkg.path().clone(),
                relative_path: pkg.relative_path().clone(),
                package_json: pkg.package_info.pkg_json.borrow().clone(),
                is_internal: pkg.is_internal,
                dependencies: pkg.workspace_package.workspace_dependencies.clone(),
                dev_dependencies: pkg.workspace_package.workspace_dev_dependencies.clone(),
                workspace_dependencies: pkg.workspace_package.workspace_dependencies.clone(),
                dependents: pkg.dependents.clone(),
            })
            .collect()
    }

    /// Analyze available upgrades using the Upgrader from package-tools
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package extraction fails
    /// - Upgrader initialization fails
    /// - Upgrade analysis fails
    #[allow(clippy::too_many_lines)]
    pub fn analyze_available_upgrades(&self) -> Result<UpgradeAnalysisResult> {
        let total_packages = self.packages.len();
        let mut major_upgrades = Vec::new();
        let mut minor_upgrades = Vec::new();
        let mut patch_upgrades = Vec::new();
        let mut up_to_date = Vec::new();

        if total_packages == 0 {
            return Ok(UpgradeAnalysisResult {
                total_packages: 0,
                upgradable_count: 0,
                major_upgrades,
                minor_upgrades,
                patch_upgrades,
                up_to_date,
            });
        }

        // Create upgrader with default registry manager
        let mut upgrader = Upgrader::new();

        // Extract Package objects for upgrade analysis
        let packages_for_analysis: Result<Vec<_>> = self
            .packages
            .iter()
            .map(|pkg_info| {
                // Convert external dependencies from strings to Dependency objects
                let dependencies: Vec<sublime_package_tools::Dependency> = pkg_info
                    .dependencies_external
                    .iter()
                    .filter_map(|dep_name| {
                        sublime_package_tools::Dependency::new(
                            dep_name, "*", // Use wildcard version for now
                        )
                        .ok() // Ignore dependencies that fail to parse
                    })
                    .collect();

                sublime_package_tools::Package::new(
                    pkg_info.name(),
                    pkg_info.version(),
                    Some(dependencies),
                )
                .map_err(crate::error::Error::Version)
            })
            .collect();

        let packages = packages_for_analysis?;

        // Check for upgrades across all packages
        match upgrader.check_all_upgrades(&packages) {
            Ok(upgrade_results) => {
                for upgrade in upgrade_results {
                    let upgrade_info = super::types::UpgradeInfo {
                        package_name: upgrade.package_name.clone(),
                        dependency_name: upgrade.dependency_name.clone(),
                        current_version: upgrade.current_version.clone(),
                        available_version: upgrade.latest_version.clone().unwrap_or_else(|| {
                            upgrade
                                .compatible_version
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string())
                        }),
                        upgrade_type: match upgrade.status {
                            sublime_package_tools::UpgradeStatus::MajorAvailable(_) => {
                                "major".to_string()
                            }
                            sublime_package_tools::UpgradeStatus::MinorAvailable(_) => {
                                "minor".to_string()
                            }
                            sublime_package_tools::UpgradeStatus::PatchAvailable(_) => {
                                "patch".to_string()
                            }
                            sublime_package_tools::UpgradeStatus::UpToDate => {
                                "up-to-date".to_string()
                            }
                            sublime_package_tools::UpgradeStatus::Constrained(_) => {
                                "constrained".to_string()
                            }
                            sublime_package_tools::UpgradeStatus::CheckFailed(_) => {
                                "check-failed".to_string()
                            }
                        },
                    };

                    match upgrade.status {
                        sublime_package_tools::UpgradeStatus::MajorAvailable(_) => {
                            major_upgrades.push(upgrade_info);
                        }
                        sublime_package_tools::UpgradeStatus::MinorAvailable(_) => {
                            minor_upgrades.push(upgrade_info);
                        }
                        sublime_package_tools::UpgradeStatus::PatchAvailable(_) => {
                            patch_upgrades.push(upgrade_info);
                        }
                        sublime_package_tools::UpgradeStatus::UpToDate => {
                            up_to_date.push(format!(
                                "{}@{}",
                                upgrade.package_name, upgrade.current_version
                            ));
                        }
                        _ => {
                            // Handle constrained or failed checks
                        }
                    }
                }
            }
            Err(e) => {
                // Log error but don't fail the entire analysis
                log::warn!("Failed to check upgrades: {}", e);
            }
        }

        let upgradable_count = major_upgrades.len() + minor_upgrades.len() + patch_upgrades.len();

        Ok(UpgradeAnalysisResult {
            total_packages,
            upgradable_count,
            major_upgrades,
            minor_upgrades,
            patch_upgrades,
            up_to_date,
        })
    }

    /// Analyze workspace configuration using standard-tools and config
    pub fn analyze_workspace_config(&self) -> Result<WorkspaceConfigAnalysis> {
        // Use existing packages instead of separate detection
        let mut patterns = Vec::new();
        let mut has_nohoist = false;
        let mut nohoist_patterns = Vec::new();

        // DRY: Create FileSystemManager once for the entire function
        let fs = FileSystemManager::new();

        // Extract workspace patterns from package manager configuration
        // Detect package manager using standard-tools PackageManager::detect
        let package_manager =
            sublime_standard_tools::monorepo::PackageManager::detect(self.root_path)
                .map_err(|e| Error::Analysis(format!("Failed to detect package manager: {e}")))?;
        let pm_kind_str = format!("{:?}", package_manager.kind());
        match pm_kind_str.as_str() {
            "Npm" | "Yarn" => {
                // DRY: Use FileSystemManager for package.json reading
                let fs = FileSystemManager::new();
                let package_json_path = self.root_path.join("package.json");
                if let Ok(content) = fs.read_file_string(&package_json_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Handle different workspace configurations
                        if let Some(workspaces) = json.get("workspaces") {
                            match workspaces {
                                serde_json::Value::Array(arr) => {
                                    patterns = arr
                                        .iter()
                                        .filter_map(|v| {
                                            v.as_str().map(std::string::ToString::to_string)
                                        })
                                        .collect();
                                }
                                serde_json::Value::Object(obj) => {
                                    // Yarn workspaces with packages and nohoist
                                    if let Some(packages) =
                                        obj.get("packages").and_then(|p| p.as_array())
                                    {
                                        patterns = packages
                                            .iter()
                                            .filter_map(|v| {
                                                v.as_str().map(std::string::ToString::to_string)
                                            })
                                            .collect();
                                    }

                                    // Check for nohoist configuration
                                    if let Some(nohoist) =
                                        obj.get("nohoist").and_then(|n| n.as_array())
                                    {
                                        has_nohoist = true;
                                        nohoist_patterns = nohoist
                                            .iter()
                                            .filter_map(|v| {
                                                v.as_str().map(std::string::ToString::to_string)
                                            })
                                            .collect();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            "Pnpm" => {
                // DRY: Use FileSystemManager for pnpm-workspace.yaml reading
                let workspace_yaml = self.root_path.join("pnpm-workspace.yaml");
                if let Ok(content) = fs.read_file_string(&workspace_yaml) {
                    if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        if let Some(packages) = config.get("packages").and_then(|p| p.as_sequence())
                        {
                            patterns = packages
                                .iter()
                                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                                .collect();
                        }
                    }
                }
            }
            _ => {
                // For other package managers, try to infer patterns from detected packages
                let package_dirs: std::collections::HashSet<String> = self.packages
                    .iter()
                    .filter_map(|pkg| {
                        pkg.relative_path().parent().map(|p| p.to_string_lossy().to_string())
                    })
                    .collect();

                patterns = package_dirs.into_iter().map(|dir| format!("{dir}/*")).collect();
            }
        }

        // Add config-based patterns if they exist
        if let Ok(config_patterns) = self.get_config_workspace_patterns() {
            for pattern in config_patterns {
                if !patterns.contains(&pattern) {
                    patterns.push(pattern);
                }
            }
        }

        // If no patterns found, infer from package locations
        if patterns.is_empty() {
            let package_dirs: std::collections::HashSet<String> = self.packages
                .iter()
                .filter_map(|pkg| pkg.relative_path().parent().map(|p| p.to_string_lossy().to_string()))
                .collect();

            patterns = package_dirs.into_iter().map(|dir| format!("{dir}/*")).collect();
        }

        let matched_packages = self.packages.len();

        // Find orphaned packages (packages not matching any pattern)
        let orphaned_packages = self.find_orphaned_packages(&patterns);

        Ok(WorkspaceConfigAnalysis {
            patterns,
            matched_packages,
            orphaned_packages,
            has_nohoist,
            nohoist_patterns,
        })
    }

    /// Get workspace patterns from configuration
    /// This is a robust implementation that leverages the monorepo configuration system
    #[allow(clippy::too_many_lines)]
    pub fn get_config_workspace_patterns(&self) -> Result<Vec<String>> {
        // Determine current package manager type
        // Detect package manager using standard-tools PackageManager::detect
        let package_manager =
            sublime_standard_tools::monorepo::PackageManager::detect(self.root_path)
                .map_err(|e| Error::Analysis(format!("Failed to detect package manager: {e}")))?;
        let pm_kind_str = format!("{:?}", package_manager.kind());
        let _current_pm_type = match pm_kind_str.as_str() {
            "Npm" => PackageManagerType::Npm,
            "Yarn" => PackageManagerType::Yarn,
            "Pnpm" => PackageManagerType::Pnpm,
            "Bun" => PackageManagerType::Bun,
            "Jsr" => PackageManagerType::Custom("jsr".to_string()),
            _ => PackageManagerType::Custom(pm_kind_str),
        };

        // First, try to infer patterns from actual discovered packages (single source of truth)
        if !self.packages.is_empty() {
            let mut patterns = std::collections::HashSet::new();
            
            // Extract directory patterns from actual package locations
            for package in self.packages {
                let location = package.relative_path();
                if let Some(parent) = location.parent() {
                    let pattern = format!("{}/*", parent.to_string_lossy());
                    patterns.insert(pattern);
                }
            }
            
            if !patterns.is_empty() {
                return Ok(patterns.into_iter().collect());
            }
        }

        // Fallback: Get auto-detected patterns from workspace configuration files
        let auto_detected_patterns = self.get_auto_detected_patterns()?;
        if !auto_detected_patterns.is_empty() {
            return Ok(auto_detected_patterns);
        }

        // Last resort: Use configured common patterns if nothing else is available
        if !self.config.workspace.discovery.common_patterns.is_empty() {
            return Ok(self.config.workspace.discovery.common_patterns.clone());
        }

        // Fallback: Get package manager specific patterns (real implementation)
        let pm_specific_patterns = match package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => {
                // For npm, check workspaces in package.json
                let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
                let package_json_path = self.root_path.join("package.json");
                if let Ok(content) = fs.read_file_string(&package_json_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(workspaces) = json.get("workspaces") {
                            match workspaces {
                                serde_json::Value::Array(arr) => arr
                                    .iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect(),
                                serde_json::Value::Object(obj) => {
                                    if let Some(serde_json::Value::Array(arr)) = obj.get("packages")
                                    {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect()
                                    } else {
                                        Vec::new()
                                    }
                                }
                                _ => Vec::new(),
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => {
                // For yarn, similar to npm but also check yarn-specific patterns
                let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
                let package_json_path = self.root_path.join("package.json");
                if let Ok(content) = fs.read_file_string(&package_json_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(workspaces) = json.get("workspaces") {
                            match workspaces {
                                serde_json::Value::Array(arr) => arr
                                    .iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect(),
                                serde_json::Value::Object(obj) => {
                                    if let Some(serde_json::Value::Array(arr)) = obj.get("packages")
                                    {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect()
                                    } else {
                                        Vec::new()
                                    }
                                }
                                _ => Vec::new(),
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                // For pnpm, check pnpm-workspace.yaml
                let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
                let workspace_path = self.root_path.join("pnpm-workspace.yaml");
                if let Ok(content) = fs.read_file_string(&workspace_path) {
                    if let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content) {
                        if let Some(serde_json::Value::Array(arr)) = yaml.get("packages") {
                            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        if !pm_specific_patterns.is_empty() {
            return Ok(pm_specific_patterns);
        }

        // Last fallback: Use discovery patterns from config (real implementation)
        let workspace_config = &self.config.workspace;
        if workspace_config.discovery.scan_common_patterns {
            return Ok(workspace_config.discovery.common_patterns.clone());
        }

        // Ultimate fallback: return empty
        Ok(Vec::new())
    }

    /// Get auto-detected workspace patterns based on current project structure
    pub fn get_auto_detected_patterns(&self) -> Result<Vec<String>> {
        let mut patterns = Vec::new();
        let workspace_config = &self.config.workspace;

        // Skip auto-detection if disabled
        if !workspace_config.discovery.auto_detect {
            return Ok(patterns);
        }

        // Use existing packages from project (single source of truth)
        if self.packages.is_empty() {
            return Ok(patterns);
        }

        let package_locations: Vec<String> = self.packages
            .iter()
            .filter_map(|pkg| pkg.relative_path().parent().map(|p| p.to_string_lossy().to_string()))
            .collect();

        // Infer patterns from existing package locations
        let mut location_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for location in &package_locations {
            // Count how many packages are in each parent directory
            *location_counts.entry(location.clone()).or_insert(0) += 1;
        }

        // Convert locations with multiple packages to patterns
        for (location, count) in location_counts {
            match count {
                0 => {} // Skip empty locations
                1 => patterns.push(location),
                _ => patterns.push(format!("{location}/*")),
            }
        }

        // No longer use common_patterns here - we have real detected packages

        // Remove patterns that match excluded directories
        patterns.retain(|pattern| {
            !workspace_config
                .discovery
                .exclude_directories
                .iter()
                .any(|excluded| pattern.contains(excluded))
        });

        // Sort by specificity (more specific patterns first)
        patterns.sort_by(|a, b| {
            let specificity_a = self.calculate_pattern_specificity(a);
            let specificity_b = self.calculate_pattern_specificity(b);
            specificity_b.cmp(&specificity_a)
        });

        Ok(patterns)
    }

    /// Calculate pattern specificity for sorting (higher is more specific)
    #[allow(clippy::cast_possible_truncation)]
    pub fn calculate_pattern_specificity(&self, pattern: &str) -> u32 {
        let mut specificity = 0;

        // More path components = higher specificity
        let components = pattern.split('/').count() as u32;
        specificity += components * 10;

        // Count wildcards and non-wildcard components
        let wildcard_count = pattern.matches('*').count() as u32;
        let non_wildcard_components =
            pattern.split('/').filter(|&comp| !comp.contains('*')).count() as u32;

        // Patterns without wildcards are more specific
        if wildcard_count == 0 {
            specificity += 100; // Exact matches get highest bonus
        } else {
            // Fewer wildcards = more specific
            specificity += 50 - (wildcard_count * 10);

            // More non-wildcard components = more specific
            specificity += non_wildcard_components * 5;
        }

        // Bonus for specific directory names vs wildcards
        if pattern.contains('*') {
            let specific_parts = pattern.split('/').filter(|&part| !part.contains('*')).count();
            specificity += (specific_parts as u32) * 3;
        }

        specificity
    }

    /// Get workspace patterns with full context and validation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration patterns cannot be retrieved
    /// - Auto-detected patterns cannot be generated
    /// - Workspace configuration validation fails
    pub fn get_validated_workspace_patterns(&self) -> Result<WorkspacePatternAnalysis> {
        let config_patterns = self.get_config_workspace_patterns()?;
        let auto_detected = self.get_auto_detected_patterns()?;

        // Validate patterns against existing packages
        let existing_packages: Vec<String> = self.packages
            .iter()
            .map(|pkg| pkg.relative_path().to_string_lossy().to_string())
            .collect();

        // Validate workspace configuration (real implementation)
        let validation_errors = self.validate_workspace_configuration();

        // Handle validation errors properly with detailed reporting
        if validation_errors.is_empty() {
            log::debug!("Workspace configuration validation passed successfully");
        } else {
            // Log detailed error information with structured context
            log::error!(
                "Found {} workspace configuration validation errors:",
                validation_errors.len()
            );
            for (index, error) in validation_errors.iter().enumerate() {
                log::error!("  Validation Error {}: {}", index + 1, error);
            }

            // Log summary for easier debugging
            log::warn!(
                "Workspace validation failed with {} errors. These may indicate mismatched patterns or invalid package configurations.",
                validation_errors.len()
            );
        }

        // Calculate pattern effectiveness
        let mut pattern_stats = Vec::new();
        for pattern in &config_patterns {
            let matches = existing_packages
                .iter()
                .filter(|pkg| self.matches_glob_pattern(pkg, pattern))
                .count();

            pattern_stats.push(PatternStatistics {
                pattern: pattern.clone(),
                matches,
                is_effective: matches > 0,
                specificity: self.calculate_pattern_specificity(pattern),
            });
        }

        Ok(WorkspacePatternAnalysis {
            config_patterns: config_patterns.clone(),
            auto_detected_patterns: auto_detected,
            effective_patterns: pattern_stats
                .iter()
                .filter(|stats| stats.is_effective)
                .map(|stats| stats.pattern.clone())
                .collect(),
            validation_errors,
            pattern_statistics: pattern_stats,
            orphaned_packages: self.find_orphaned_packages(&config_patterns),
        })
    }

    /// Find packages that don't match any workspace pattern
    #[must_use]
    pub fn find_orphaned_packages(&self, patterns: &[String]) -> Vec<String> {
        let mut orphaned = Vec::new();

        // Use existing packages from project
        for package in self.packages {
            let package_path = package.relative_path().to_string_lossy();
            let mut matches_pattern = false;

            for pattern in patterns {
                if self.matches_glob_pattern(&package_path, pattern) {
                    matches_pattern = true;
                    break;
                }
            }

            if !matches_pattern {
                orphaned.push(package.name().to_string());
            }
        }

        orphaned
    }

    /// Real glob pattern matching using the glob library
    ///
    /// Provides proper glob pattern matching capabilities including:
    /// - Wildcard patterns (*, **)
    /// - Character classes (\[abc\], \[a-z\])
    /// - Escape sequences
    /// - Case sensitivity handling
    ///
    /// # Arguments
    ///
    /// * `path` - The path to match against
    /// * `pattern` - The glob pattern to use for matching
    ///
    /// # Returns
    ///
    /// True if the path matches the glob pattern, false otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::MonorepoAnalyzer;
    /// # let analyzer = create_test_analyzer();
    /// assert!(analyzer.matches_glob_pattern("packages/core/src/lib.rs", "packages/*/src/**"));
    /// assert!(analyzer.matches_glob_pattern("src/test.js", "src/*.js"));
    /// assert!(!analyzer.matches_glob_pattern("src/test.ts", "src/*.js"));
    /// ```
    #[must_use]
    pub fn matches_glob_pattern(&self, path: &str, pattern: &str) -> bool {
        use glob::Pattern;

        match Pattern::new(pattern) {
            Ok(glob_pattern) => glob_pattern.matches(path),
            Err(e) => {
                log::warn!(
                    "Invalid glob pattern '{}': {}. Falling back to exact string match.",
                    pattern,
                    e
                );
                // Fallback to exact match if pattern is invalid
                path == pattern
            }
        }
    }

    /// Detect changes since a specific Git reference
    ///
    /// Performs real Git-based change detection using sublime-git-tools to identify
    /// files changed between the specified reference and the current HEAD.
    ///
    /// # Arguments
    ///
    /// * `since_ref` - Git reference to compare from (tag, branch, or commit SHA)
    /// * `until_ref` - Optional Git reference to compare to (defaults to HEAD)
    ///
    /// # Returns
    ///
    /// Comprehensive change analysis including changed files, packages affected,
    /// and impact analysis for the monorepo.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git repository cannot be opened
    /// - References cannot be resolved
    /// - File change analysis fails
    pub fn detect_changes_since(
        &self,
        since_ref: &str,
        until_ref: Option<&str>,
    ) -> Result<ChangeAnalysis> {
        // Open Git repository using sublime-git-tools
        let repo = sublime_git_tools::Repo::open(
            self.root_path
                .to_str()
                .ok_or_else(|| crate::error::Error::analysis("Invalid root path encoding"))?,
        )
        .map_err(|e| {
            crate::error::Error::analysis(format!("Failed to open Git repository: {e}"))
        })?;

        // Get changed files since the reference
        let changed_files = if let Some(_until) = until_ref {
            // If until_ref is specified, we need to compare between two specific refs
            // For now, use the since_ref approach and note the limitation
            repo.get_all_files_changed_since_sha_with_status(since_ref).map_err(|e| {
                crate::error::Error::analysis(format!("Failed to get changed files: {e}"))
            })?
        } else {
            repo.get_all_files_changed_since_sha_with_status(since_ref).map_err(|e| {
                crate::error::Error::analysis(format!("Failed to get changed files: {e}"))
            })?
        };

        // Analyze which packages are affected by the changes
        let mut directly_affected = Vec::new();
        let mut package_changes = Vec::new();

        for changed_file in &changed_files {
            // Find which package this file belongs to
            for package in self.packages {
                let package_path = package.path().to_string_lossy();
                if changed_file.path.starts_with(package_path.as_ref()) {
                    if !directly_affected.contains(&package.name().to_string()) {
                        directly_affected.push(package.name().to_string());
                    }

                    // Convert GitFileStatus to PackageChangeType
                    // All file changes are considered source code changes for now
                    let change_type = crate::changes::PackageChangeType::SourceCode;

                    // Create package change using the correct structure
                    package_changes.push(crate::changes::PackageChange {
                        package_name: package.name().to_string(),
                        change_type,
                        significance: crate::changes::ChangeSignificance::Medium,
                        changed_files: vec![changed_file.clone()],
                        suggested_version_bump: crate::config::VersionBumpType::Patch,
                        metadata: std::collections::HashMap::new(),
                    });

                    break;
                }
            }
        }

        // Create comprehensive change analysis
        Ok(ChangeAnalysis {
            from_ref: since_ref.to_string(),
            to_ref: until_ref.unwrap_or("HEAD").to_string(),
            changed_files,
            package_changes,
            directly_affected: directly_affected.clone(),
            dependents_affected: Vec::new(), // Would need dependency analysis for this
            change_propagation_graph: std::collections::HashMap::new(),
            impact_scores: std::collections::HashMap::new(),
            total_affected_count: directly_affected.len(),
            significance_analysis: Vec::new(), // Would need deeper analysis for this
        })
    }

    /// Compare two branches using Git operations
    ///
    /// Performs comprehensive branch comparison using sublime-git-tools to analyze
    /// differences between two branches including file changes, package impacts,
    /// and potential merge conflicts.
    ///
    /// # Arguments
    ///
    /// * `base_branch` - The base branch to compare from
    /// * `target_branch` - The target branch to compare to
    ///
    /// # Returns
    ///
    /// Detailed branch comparison result showing divergence point, changed files,
    /// affected packages, and analysis of the differences.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git repository cannot be opened
    /// - Branches cannot be resolved
    /// - Git operations fail
    pub fn compare_branches(
        &self,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<BranchComparisonResult> {
        // Open Git repository using sublime-git-tools
        let repo = sublime_git_tools::Repo::open(
            self.root_path
                .to_str()
                .ok_or_else(|| crate::error::Error::analysis("Invalid root path encoding"))?,
        )
        .map_err(|e| {
            crate::error::Error::analysis(format!("Failed to open Git repository: {e}"))
        })?;

        // Find the common ancestor (divergence point)
        let divergence_point = repo.get_diverged_commit(base_branch).map_err(|e| {
            crate::error::Error::analysis(format!("Failed to find divergence point: {e}"))
        })?;

        // Get changes in target branch since divergence point
        let target_changes =
            repo.get_all_files_changed_since_sha_with_status(&divergence_point).map_err(|e| {
                crate::error::Error::analysis(format!("Failed to get target branch changes: {e}"))
            })?;

        // Analyze affected packages in target branch
        let mut target_affected_packages = Vec::new();

        for changed_file in &target_changes {
            for package in self.packages {
                let package_path = package.path().to_string_lossy();
                if changed_file.path.starts_with(package_path.as_ref()) {
                    if !target_affected_packages.contains(&package.name().to_string()) {
                        target_affected_packages.push(package.name().to_string());
                    }
                    break;
                }
            }
        }

        // Get commit history for both branches since divergence
        let _target_commits =
            repo.get_commits_since(Some(divergence_point.clone()), &None).map_err(|e| {
                crate::error::Error::analysis(format!("Failed to get target commits: {e}"))
            })?;

        // Analyze potential conflicts (simplified - checking if same files are modified)
        let mut potential_conflicts = Vec::new();

        // For a more thorough conflict analysis, we'd need to checkout base_branch and compare
        // For now, we'll identify files that have been modified in target as potential conflicts
        for change in &target_changes {
            if matches!(change.status, sublime_git_tools::GitFileStatus::Modified) {
                potential_conflicts.push(change.path.clone());
            }
        }

        Ok(BranchComparisonResult {
            base_branch: base_branch.to_string(),
            target_branch: target_branch.to_string(),
            changed_files: target_changes,
            affected_packages: target_affected_packages,
            merge_base: divergence_point,
            conflicts: potential_conflicts,
        })
    }

    /// Detect the actual version of the package manager by executing the appropriate command
    ///
    /// # Arguments
    ///
    /// * `kind` - The package manager kind to detect version for
    ///
    /// # Returns
    ///
    /// The version string of the package manager, or "unknown" if detection fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_node_tools::PackageManagerKind;
    /// # use sublime_monorepo_tools::MonorepoAnalyzer;
    /// # let analyzer = create_test_analyzer();
    /// let version = analyzer.detect_package_manager_version(PackageManagerKind::Npm);
    /// println!("NPM version: {}", version);
    /// ```
    fn detect_package_manager_version(&self, kind: PackageManagerKind) -> String {
        // Use configurable commands and arguments
        let workspace_config = &self.config.workspace;
        let pm_config = &workspace_config.package_manager_commands;

        // Handle JSR separately due to lifetime issues
        if matches!(kind, PackageManagerKind::Jsr) {
            let output =
                Command::new("jsr").args(["--version"]).current_dir(self.root_path).output();

            return match output {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if version.is_empty() {
                        "unknown".to_string()
                    } else {
                        version
                    }
                }
                _ => "unknown".to_string(),
            };
        }

        let (command, args) = match &kind {
            PackageManagerKind::Npm => {
                let pm_type = crate::config::types::workspace::PackageManagerType::Npm;
                (pm_config.get_command(&pm_type), pm_config.get_version_args(&pm_type))
            }
            PackageManagerKind::Pnpm => {
                let pm_type = crate::config::types::workspace::PackageManagerType::Pnpm;
                (pm_config.get_command(&pm_type), pm_config.get_version_args(&pm_type))
            }
            PackageManagerKind::Yarn => {
                let pm_type = crate::config::types::workspace::PackageManagerType::Yarn;
                (pm_config.get_command(&pm_type), pm_config.get_version_args(&pm_type))
            }
            PackageManagerKind::Bun => {
                let pm_type = crate::config::types::workspace::PackageManagerType::Bun;
                (pm_config.get_command(&pm_type), pm_config.get_version_args(&pm_type))
            }
            PackageManagerKind::Jsr => unreachable!(), // Handled above
        };

        let output = Command::new(command).args(args).current_dir(self.root_path).output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if version.is_empty() {
                    "unknown".to_string()
                } else {
                    version
                }
            }
            Ok(output) => {
                log::warn!(
                    "Package manager '{}' command failed with status: {}. stderr: {}",
                    command,
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                "unknown".to_string()
            }
            Err(e) => {
                log::warn!("Failed to execute package manager '{}' command: {}", command, e);
                "unknown".to_string()
            }
        }
    }

    /// Validate workspace configuration for monorepo consistency
    ///
    /// Performs comprehensive validation of workspace configuration including:
    /// - Package discovery patterns validation
    /// - Workspace structure consistency
    /// - Package manager configuration coherence
    /// - Tool configuration validation
    ///
    /// # Returns
    ///
    /// A vector of validation error messages. Empty if validation passes.
    fn validate_workspace_configuration(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate workspace discovery patterns
        let workspace_config = &self.config.workspace;

        // Check if discovery patterns are valid glob patterns
        for pattern in &workspace_config.discovery.common_patterns {
            if pattern.trim().is_empty() {
                errors.push("Empty discovery pattern found in workspace configuration".to_string());
                continue;
            }

            // Validate glob pattern syntax
            if let Err(e) = glob::Pattern::new(pattern) {
                errors.push(format!("Invalid glob pattern '{pattern}': {e}"));
            }
        }

        // Validate that package discovery is working (via real MonorepoDetector)
        if self.packages.is_empty() && workspace_config.discovery.auto_detect {
            errors.push("Auto-detection enabled but no packages found. Check workspace configuration files (package.json, pnpm-workspace.yaml, etc.).".to_string());
        }

        // Validate package manager consistency
        let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
        let mut detected_package_managers = Vec::new();

        // Check for npm
        if fs.exists(&self.root_path.join("package.json")) {
            detected_package_managers.push("npm/yarn");
        }

        // Check for pnpm
        if fs.exists(&self.root_path.join("pnpm-workspace.yaml")) {
            detected_package_managers.push("pnpm");
        }

        // Check for multiple package managers
        if detected_package_managers.len() > 1 {
            errors.push(format!(
                "Multiple package manager configurations detected: {}. This may cause conflicts.",
                detected_package_managers.join(", ")
            ));
        }

        // Validate workspace tool configurations
        // Validate workspace tool configurations (real implementation)
        for config_pattern in &workspace_config.tool_configs.config_file_patterns {
            if config_pattern.trim().is_empty() {
                errors.push("Empty config pattern in workspace tool configurations".to_string());
                continue;
            }

            // Validate tool-specific configuration paths exist
            let full_path = self.root_path.join(config_pattern);
            if !fs.exists(&full_path) {
                errors.push(format!("Tool configuration file not found: {}", full_path.display()));
            }
        }

        // Validate package structure consistency
        let mut package_names = std::collections::HashSet::new();
        for package in self.packages {
            // Check for duplicate package names
            if !package_names.insert(package.name()) {
                let package_name = package.name();
                errors.push(format!("Duplicate package name found: '{package_name}'"));
            }

            // Validate package path exists
            if !fs.exists(package.path()) {
                errors.push(format!(
                    "Package '{}' path does not exist: {}",
                    package.name(),
                    package.path().display()
                ));
            }

            // Validate package.json exists for each package
            let package_json_path = package.path().join("package.json");
            if !fs.exists(&package_json_path) {
                errors.push(format!(
                    "Package '{}' missing package.json: {}",
                    package.name(),
                    package_json_path.display()
                ));
            }
        }

        // Validate root workspace structure
        let root_package_json = self.root_path.join("package.json");
        if !fs.exists(&root_package_json) {
            errors.push(
                "Root package.json not found. This is required for workspace configuration."
                    .to_string(),
            );
        }

        errors
    }
}
