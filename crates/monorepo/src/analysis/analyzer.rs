//! Monorepo analyzer implementation

use crate::analysis::{
    BranchComparisonResult, ChangeAnalysis, DependencyGraphAnalysis, DiffAnalyzer,
    MonorepoAnalyzer, PackageClassificationResult, PackageInformation, PackageManagerAnalysis,
    PatternStatistics, RegistryAnalysisResult, RegistryInfo, UpgradeAnalysisResult,
    WorkspaceConfigAnalysis, WorkspacePatternAnalysis,
};
use crate::config::PackageManagerType;
use crate::core::MonorepoProject;
use crate::error::Result;
use crate::MonorepoAnalysisResult;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use sublime_package_tools::Upgrader;
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, PackageManagerKind};

impl MonorepoAnalyzer {
    /// Create a new analyzer for a monorepo project
    #[must_use]
    pub fn new(project: Arc<MonorepoProject>) -> Self {
        Self { project }
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
        // Use standard-tools detector
        let detector = MonorepoDetector::new();
        let descriptor = detector.detect_monorepo(path)?;

        // Analyze package manager
        let package_manager = self.analyze_package_manager()?;

        // Build dependency graph
        let dependency_graph = self.build_dependency_graph()?;

        // Classify packages
        let packages = self.classify_packages()?;

        // Analyze registries
        let registries = self.analyze_registries()?;

        // Analyze workspace configuration
        let workspace_config = self.analyze_workspace_config()?;

        Ok(MonorepoAnalysisResult {
            kind: descriptor.kind().clone(),
            root_path: descriptor.root().to_path_buf(),
            package_manager,
            packages,
            dependency_graph,
            registries,
            workspace_config,
        })
    }

    /// Analyze the package manager configuration
    pub fn analyze_package_manager(&self) -> Result<PackageManagerAnalysis> {
        let pm = &self.project.package_manager;
        let kind = pm.kind();

        // Get package manager version by executing the appropriate command
        let version = self.detect_package_manager_version(kind);

        // DRY: Use FileSystemManager instead of manual std::fs operations
        let fs = FileSystemManager::new();
        let package_json_path = pm.root().join("package.json");
        let workspaces_config = if fs.exists(&package_json_path) {
            let content = fs.read_file_string(&package_json_path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            json.get("workspaces").cloned().unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        // Find all config files
        let mut config_files = vec![pm.lock_file_path()];

        match kind {
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                let workspace_yaml = pm.root().join("pnpm-workspace.yaml");
                if fs.exists(&workspace_yaml) {
                    config_files.push(workspace_yaml);
                }
            }
            _ => {
                if fs.exists(&package_json_path) {
                    config_files.push(package_json_path);
                }
            }
        }

        Ok(PackageManagerAnalysis {
            kind,
            version,
            lock_file: pm.lock_file_path(),
            config_files,
            workspaces_config,
        })
    }

    /// Build and analyze the dependency graph
    pub fn build_dependency_graph(&self) -> Result<DependencyGraphAnalysis> {
        let packages = &self.project.packages;

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

        // Build dependency graph using DependencyGraph::from
        let graph = sublime_package_tools::DependencyGraph::from(package_vec.as_slice());

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
        for pkg in &self.project.packages {
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
        let rm = &self.project.registry_manager;

        let default_registry = rm.default_registry().to_string();
        let registry_urls = rm.registry_urls();

        let mut registries = Vec::new();
        let mut scoped_registries = HashMap::new();
        let mut auth_status = HashMap::new();

        // Analyze each registry
        for url in registry_urls {
            // Determine registry type using configurable patterns
            let registry_type =
                self.project.config.workspace.tool_configs.get_registry_type(url).to_string();

            // Check for authentication (basic heuristic)
            let has_auth = self.check_registry_auth(url);

            // Find scopes associated with this registry
            let mut scopes = Vec::new();

            // DRY: Use FileSystemManager for .npmrc file reading
            let fs = FileSystemManager::new();
            if let Ok(npmrc_content) = fs.read_file_string(&self.project.root_path().join(".npmrc"))
            {
                for line in npmrc_content.lines() {
                    if line.contains(url) && line.contains('@') {
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
        for package in &self.project.packages {
            let package_json = &package.package_info.pkg_json.borrow();

            // Check dependencies for scoped packages
            if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
                for dep_name in deps.keys() {
                    if dep_name.starts_with('@') {
                        let scope = dep_name.split('/').next().unwrap_or(dep_name);
                        if rm.has_scope(scope) {
                            if let Some(scope_registry) = rm.get_registry_for_scope(scope) {
                                scoped_registries
                                    .insert(scope.to_string(), scope_registry.to_string());
                            }
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
            self.project.root_path().join(".npmrc"),
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

        // Check environment variables using configurable auth patterns
        let tool_config = &self.project.config.workspace.tool_configs;
        let registry_type = tool_config.get_registry_type(registry_url);
        let auth_env_vars = tool_config.get_auth_env_vars(registry_type);

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
        self.project
            .packages
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
    pub fn analyze_available_upgrades(&self) -> Result<UpgradeAnalysisResult> {
        let total_packages = self.project.packages.len();
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

        // Create upgrader with the project's registry manager
        let mut upgrader = Upgrader::with_registry_manager(self.project.registry_manager.clone());

        // Extract Package objects for upgrade analysis
        let packages_for_analysis: Result<Vec<_>> = self
            .project
            .packages
            .iter()
            .map(|pkg_info| {
                let pkg_ref = pkg_info.package_info.package.borrow();
                sublime_package_tools::Package::new(
                    pkg_ref.name(),
                    &pkg_ref.version_str(),
                    Some(pkg_ref.dependencies().to_vec()),
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
        let descriptor = &self.project.descriptor;
        let mut patterns = Vec::new();
        let mut has_nohoist = false;
        let mut nohoist_patterns = Vec::new();

        // DRY: Create FileSystemManager once for the entire function
        let fs = FileSystemManager::new();

        // Extract workspace patterns from package manager configuration
        match self.project.package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm
            | sublime_standard_tools::monorepo::PackageManagerKind::Yarn => {
                // DRY: Use FileSystemManager for package.json reading
                let fs = FileSystemManager::new();
                let package_json_path = self.project.package_manager.root().join("package.json");
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
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                // DRY: Use FileSystemManager for pnpm-workspace.yaml reading
                let workspace_yaml =
                    self.project.package_manager.root().join("pnpm-workspace.yaml");
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
                let package_dirs: std::collections::HashSet<String> = descriptor
                    .packages()
                    .iter()
                    .filter_map(|pkg| {
                        pkg.location.parent().map(|p| p.to_string_lossy().to_string())
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
            let package_dirs: std::collections::HashSet<String> = descriptor
                .packages()
                .iter()
                .filter_map(|pkg| pkg.location.parent().map(|p| p.to_string_lossy().to_string()))
                .collect();

            patterns = package_dirs.into_iter().map(|dir| format!("{dir}/*")).collect();
        }

        let matched_packages = descriptor.packages().len();

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
    pub fn get_config_workspace_patterns(&self) -> Result<Vec<String>> {
        // Determine current package manager type
        let current_pm_type = match self.project.package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => PackageManagerType::Npm,
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => PackageManagerType::Yarn,
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => PackageManagerType::Pnpm,
            sublime_standard_tools::monorepo::PackageManagerKind::Bun => PackageManagerType::Bun,
            sublime_standard_tools::monorepo::PackageManagerKind::Jsr => {
                PackageManagerType::Custom("jsr".to_string())
            }
        };

        // Get auto-detected patterns from the current workspace structure
        let auto_detected_patterns = self.get_auto_detected_patterns()?;

        // Get effective patterns combining config and auto-detected
        let effective_patterns = self.project.config_manager.get_effective_workspace_patterns(
            auto_detected_patterns,
            Some(current_pm_type.clone()),
            None, // No specific environment
        )?;

        // If we have effective patterns, return them
        if !effective_patterns.is_empty() {
            return Ok(effective_patterns);
        }

        // Fallback: Try package manager specific patterns
        let pm_specific_patterns =
            self.project.config_manager.get_package_manager_patterns(current_pm_type)?;

        if !pm_specific_patterns.is_empty() {
            return Ok(pm_specific_patterns);
        }

        // Last fallback: Use discovery patterns from config
        let workspace_config = self.project.config_manager.get_workspace()?;
        if workspace_config.discovery.scan_common_patterns {
            return Ok(workspace_config.discovery.common_patterns);
        }

        // Ultimate fallback: return empty
        Ok(Vec::new())
    }

    /// Get auto-detected workspace patterns based on current project structure
    pub fn get_auto_detected_patterns(&self) -> Result<Vec<String>> {
        let mut patterns = Vec::new();
        let workspace_config = self.project.config_manager.get_workspace()?;

        // Skip auto-detection if disabled
        if !workspace_config.discovery.auto_detect {
            return Ok(patterns);
        }

        // Get existing package locations
        let package_locations: Vec<String> = self
            .project
            .descriptor
            .packages()
            .iter()
            .filter_map(|pkg| pkg.location.parent().map(|p| p.to_string_lossy().to_string()))
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

        // Add common patterns if configured and they exist
        if workspace_config.discovery.scan_common_patterns {
            for common_pattern in &workspace_config.discovery.common_patterns {
                if self.pattern_has_matches(common_pattern)? && !patterns.contains(common_pattern) {
                    patterns.push(common_pattern.clone());
                }
            }
        }

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

    /// Check if a pattern has any matches in the current workspace
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::manual_strip)]
    fn pattern_has_matches(&self, pattern: &str) -> Result<bool> {
        let packages = self.project.descriptor.packages();

        for package in packages {
            let package_path = package.location.to_string_lossy();
            if self.matches_glob_pattern(&package_path, pattern) {
                return Ok(true);
            }
        }

        // Also check if the pattern directory structure exists
        let pattern_path = if pattern.ends_with("/*") {
            self.project.root_path().join(&pattern[..pattern.len() - 2])
        } else {
            self.project.root_path().join(pattern)
        };

        Ok(pattern_path.exists())
    }

    /// Calculate pattern specificity for sorting (higher is more specific)
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
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
        let existing_packages: Vec<String> = self
            .project
            .descriptor
            .packages()
            .iter()
            .map(|pkg| pkg.location.to_string_lossy().to_string())
            .collect();

        let validation_errors =
            self.project.config_manager.validate_workspace_config(&existing_packages)?;

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

        for package in self.project.descriptor.packages() {
            let package_path = package.location.to_string_lossy();
            let mut matches_pattern = false;

            for pattern in patterns {
                if self.matches_glob_pattern(&package_path, pattern) {
                    matches_pattern = true;
                    break;
                }
            }

            if !matches_pattern {
                orphaned.push(package.name.clone());
            }
        }

        orphaned
    }

    /// Real glob pattern matching using the glob library
    ///
    /// Provides proper glob pattern matching capabilities including:
    /// - Wildcard patterns (*, **)
    /// - Character classes ([abc], [a-z])
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

    /// Detect changes since a specific reference using `DiffAnalyzer`
    ///
    /// This is a synchronous operation as it only performs git operations
    /// and data analysis, which are CPU-bound operations.
    pub fn detect_changes_since(
        &self,
        since_ref: &str,
        until_ref: Option<&str>,
    ) -> Result<ChangeAnalysis> {
        let diff_analyzer = DiffAnalyzer::new(Arc::clone(&self.project));
        diff_analyzer.detect_changes_since(since_ref, until_ref)
    }

    /// Compare two branches using `DiffAnalyzer`
    ///
    /// This is a synchronous operation as it only performs git operations
    /// and data analysis, which are CPU-bound operations.
    pub fn compare_branches(
        &self,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<BranchComparisonResult> {
        let diff_analyzer = DiffAnalyzer::new(Arc::clone(&self.project));
        diff_analyzer.compare_branches(base_branch, target_branch)
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
        let pm_config = &self.project.config.workspace.package_manager_commands;

        // Handle JSR separately due to lifetime issues
        if matches!(kind, PackageManagerKind::Jsr) {
            let output = Command::new("jsr")
                .args(["--version"])
                .current_dir(self.project.root_path())
                .output();

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

        let output =
            Command::new(command).args(args).current_dir(self.project.root_path()).output();

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
}
