//! Monorepo analyzer implementation

use crate::analysis::{
    DependencyGraphAnalysis, PackageClassificationResult, PackageInformation,
    PackageManagerAnalysis, RegistryAnalysisResult, RegistryInfo, UpgradeAnalysisResult,
    WorkspaceConfigAnalysis,
};
use crate::core::MonorepoProject;
use crate::error::Result;
use crate::MonorepoAnalysisResult;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use sublime_package_tools::Upgrader;
use sublime_standard_tools::monorepo::MonorepoDetector;

/// Analyzer for comprehensive monorepo analysis
pub struct MonorepoAnalyzer {
    /// Reference to the monorepo project
    project: Arc<MonorepoProject>,
}

impl MonorepoAnalyzer {
    /// Create a new analyzer for a monorepo project
    pub fn new(project: Arc<MonorepoProject>) -> Self {
        Self { project }
    }

    /// Perform complete monorepo detection and analysis
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

        // Get package manager version (would need to execute command)
        let version = "unknown".to_string(); // Placeholder

        // Get workspace configuration from package.json
        let package_json_path = pm.root().join("package.json");
        let workspaces_config = if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path)?;
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
                if workspace_yaml.exists() {
                    config_files.push(workspace_yaml);
                }
            }
            _ => {
                if package_json_path.exists() {
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
    pub fn calculate_max_depth(&self, packages: &[crate::core::MonorepoPackageInfo]) -> usize {
        let mut max_depth = 0;
        let mut visited = std::collections::HashSet::new();

        for package in packages {
            if !visited.contains(package.name()) {
                let depth = self.calculate_package_depth(package, packages, &mut visited, 0);
                max_depth = max_depth.max(depth);
            }
        }

        max_depth
    }

    /// Calculate depth for a specific package
    fn calculate_package_depth(
        &self,
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
                let child_depth = self.calculate_package_depth(
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
            // Determine registry type based on URL
            let registry_type = if url.contains("registry.npmjs.org") {
                "npm"
            } else if url.contains("npm.pkg.github.com") {
                "github"
            } else if url.contains("pkgs.dev.azure.com") {
                "azure"
            } else if url.contains("gitlab.com") {
                "gitlab"
            } else {
                "custom"
            }
            .to_string();

            // Check for authentication (basic heuristic)
            let has_auth = self.check_registry_auth(url);

            // Find scopes associated with this registry
            let mut scopes = Vec::new();

            // Check for common scoped packages patterns
            if let Ok(npmrc_content) =
                std::fs::read_to_string(self.project.root_path().join(".npmrc"))
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
    pub fn check_registry_auth(&self, registry_url: &str) -> bool {
        // Check .npmrc for auth tokens
        let npmrc_paths = [
            self.project.root_path().join(".npmrc"),
            dirs::home_dir().map(|h| h.join(".npmrc")).unwrap_or_default(),
        ];

        for npmrc_path in &npmrc_paths {
            if let Ok(content) = std::fs::read_to_string(npmrc_path) {
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

        // Check environment variables for common auth patterns
        if registry_url.contains("registry.npmjs.org") && std::env::var("NPM_TOKEN").is_ok() {
            return true;
        }

        if registry_url.contains("npm.pkg.github.com")
            && (std::env::var("GITHUB_TOKEN").is_ok() || std::env::var("NPM_TOKEN").is_ok())
        {
            return true;
        }

        false
    }

    /// Get package information
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
            Ok(upgrades) => {
                for upgrade in upgrades {
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

        // Extract workspace patterns from package manager configuration
        match self.project.package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm
            | sublime_standard_tools::monorepo::PackageManagerKind::Yarn => {
                // Read workspace patterns from package.json
                let package_json_path = self.project.package_manager.root().join("package.json");
                if let Ok(content) = std::fs::read_to_string(&package_json_path) {
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
                // Read from pnpm-workspace.yaml
                let workspace_yaml =
                    self.project.package_manager.root().join("pnpm-workspace.yaml");
                if let Ok(content) = std::fs::read_to_string(&workspace_yaml) {
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
    pub fn get_config_workspace_patterns(&self) -> Result<Vec<String>> {
        // Check if custom workspace patterns are defined in monorepo config
        let _config = &self.project.config;

        // For now, return empty as we don't have workspace patterns in config yet
        // This could be extended to read from monorepo configuration
        Ok(Vec::new())
    }

    /// Find packages that don't match any workspace pattern
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

    /// Simple glob pattern matching
    #[allow(clippy::manual_strip)]
    pub fn matches_glob_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob matching - could be enhanced with a proper glob library
        if pattern.ends_with("/*") {
            let base_pattern = &pattern[..pattern.len() - 2];
            path.starts_with(base_pattern)
        } else if pattern.contains('*') {
            // More complex patterns - for now just check if the non-wildcard parts match
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                // Check if path starts with prefix and ends with suffix
                if path.len() >= prefix.len() + suffix.len() {
                    path.starts_with(prefix) && path.ends_with(suffix)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            path == pattern
        }
    }
}
