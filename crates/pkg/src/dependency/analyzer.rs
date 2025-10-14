use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    config::DependencyConfig,
    dependency::{
        graph::{DependencyGraph, DependencyType},
        propagator::{PropagatedUpdate, PropagationReason},
    },
    error::DependencyError,
    PackageResult, ResolvedVersion, VersionBump,
};
use sublime_standard_tools::{
    filesystem::AsyncFileSystem, monorepo::MonorepoDescriptor, project::Dependencies,
};

/// Service for dependency analysis and propagation.
///
/// Provides comprehensive dependency analysis including cycle detection,
/// propagation calculations, and integration with monorepo structures.
pub struct DependencyAnalyzer<F: AsyncFileSystem> {
    /// The dependency graph
    pub(crate) graph: DependencyGraph,
    /// Configuration for dependency analysis
    pub(crate) config: DependencyConfig,
    /// Filesystem for reading package.json files
    #[allow(dead_code)]
    pub(crate) filesystem: F,
}

/// Builder for creating dependency graphs from monorepo structures.
pub struct DependencyGraphBuilder<F: AsyncFileSystem> {
    /// Filesystem for reading package.json files
    filesystem: F,
    /// Configuration for dependency analysis
    config: DependencyConfig,
}

impl<F: AsyncFileSystem> DependencyAnalyzer<F> {
    /// Creates a new dependency analyzer with the given graph and configuration.
    ///
    /// # Arguments
    ///
    /// * `graph` - The dependency graph to analyze
    /// * `config` - Configuration for dependency analysis
    /// * `filesystem` - Filesystem for reading package.json files
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::dependency::{DependencyAnalyzer, DependencyGraph};
    /// use sublime_pkg_tools::config::DependencyConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    ///
    /// let graph = DependencyGraph::new();
    /// let config = DependencyConfig::default();
    /// let fs = FileSystemManager::new();
    /// let analyzer = DependencyAnalyzer::new(graph, config, fs);
    /// ```
    #[must_use]
    pub fn new(graph: DependencyGraph, config: DependencyConfig, filesystem: F) -> Self {
        Self { graph, config, filesystem }
    }

    /// Analyzes dependency propagation for a set of changed packages.
    ///
    /// This method calculates which packages need updates based on the changes
    /// to their dependencies, respecting configuration settings for depth limits
    /// and dependency type inclusion.
    ///
    /// # Arguments
    ///
    /// * `changed_packages` - Map of package names to their version bumps and new versions
    ///
    /// # Returns
    ///
    /// Vector of propagated updates that should be applied
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum propagation depth is exceeded
    /// - Circular dependencies are detected and fail_on_circular is true
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_pkg_tools::dependency::DependencyAnalyzer;
    /// # use sublime_pkg_tools::config::DependencyConfig;
    /// # use sublime_pkg_tools::{VersionBump, ResolvedVersion};
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::collections::HashMap;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut changed_packages = HashMap::new();
    /// changed_packages.insert("pkg-a".to_string(), (VersionBump::Minor, ResolvedVersion::Release("1.1.0".parse().unwrap())));
    ///
    /// # let graph = sublime_pkg_tools::dependency::DependencyGraph::new();
    /// # let config = DependencyConfig::default();
    /// # let fs = FileSystemManager::new();
    /// # let analyzer = DependencyAnalyzer::new(graph, config, fs);
    /// let propagated = analyzer.analyze_propagation(&changed_packages).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_propagation(
        &self,
        changed_packages: &HashMap<String, (VersionBump, ResolvedVersion)>,
    ) -> PackageResult<Vec<PropagatedUpdate>> {
        if !self.config.propagate_updates {
            return Ok(Vec::new());
        }

        let mut propagated_updates = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Initialize queue with directly changed packages
        for (package_name, (bump, new_version)) in changed_packages {
            queue.push_back((package_name.clone(), *bump, new_version.clone(), 0_u32));
        }

        while let Some((package_name, original_bump, new_version, depth)) = queue.pop_front() {
            if depth >= self.config.max_propagation_depth {
                if self.config.max_propagation_depth > 0 {
                    return Err(DependencyError::max_depth_exceeded(
                        self.config.max_propagation_depth as usize,
                    )
                    .into());
                }
                continue;
            }

            if visited.contains(&package_name) {
                continue;
            }
            visited.insert(package_name.clone());

            // Get all packages that depend on this package
            let dependents = self.graph.get_dependents(&package_name);

            for dependent_name in dependents {
                if changed_packages.contains_key(&dependent_name) {
                    // Skip packages that already have direct changes
                    continue;
                }

                let Some(dependent_node) = self.graph.get_package(&dependent_name) else {
                    continue;
                };

                // Determine the type of dependency relationship
                let dependency_type = self.get_dependency_type(&dependent_name, &package_name);

                // Skip if this dependency type is not configured for propagation
                if !self.should_include_dependency_type(dependency_type) {
                    continue;
                }

                // Get current version requirement from dependent package
                let old_version = self.get_current_dependency_version(
                    &dependent_name,
                    &package_name,
                    dependency_type,
                );

                // Calculate suggested bump based on configuration
                let suggested_bump =
                    self.calculate_propagation_bump(&original_bump, dependency_type);

                // Create propagation reason
                let reason = match dependency_type {
                    DependencyType::Runtime => PropagationReason::DependencyUpdate {
                        dependency: package_name.clone(),
                        old_version,
                        new_version: new_version.to_string(),
                    },
                    DependencyType::Development => PropagationReason::DevDependencyUpdate {
                        dependency: package_name.clone(),
                        old_version,
                        new_version: new_version.to_string(),
                    },
                    DependencyType::Optional => PropagationReason::OptionalDependencyUpdate {
                        dependency: package_name.clone(),
                        old_version,
                        new_version: new_version.to_string(),
                    },
                    DependencyType::Peer => PropagationReason::PeerDependencyUpdate {
                        dependency: package_name.clone(),
                        old_version,
                        new_version: new_version.to_string(),
                    },
                };

                // Calculate next version for the dependent package
                let next_version =
                    self.calculate_next_version(&dependent_node.version, &suggested_bump)?;

                let propagated_update = PropagatedUpdate {
                    package_name: dependent_name.clone(),
                    reason,
                    suggested_bump,
                    current_version: dependent_node.version.clone(),
                    next_version: next_version.clone(),
                };

                propagated_updates.push(propagated_update);

                // Add to queue for further propagation
                queue.push_back((dependent_name, suggested_bump, next_version, depth + 1));
            }
        }

        Ok(propagated_updates)
    }
    /// Validates the dependency graph for consistency.
    ///
    /// Checks for circular dependencies and other graph consistency issues.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Circular dependencies are detected and fail_on_circular is true
    /// - Graph structure is inconsistent
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_pkg_tools::dependency::DependencyAnalyzer;
    /// # use sublime_pkg_tools::config::DependencyConfig;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// #
    /// # let graph = sublime_pkg_tools::dependency::DependencyGraph::new();
    /// # let config = DependencyConfig::default();
    /// # let fs = FileSystemManager::new();
    /// # let analyzer = DependencyAnalyzer::new(graph, config, fs);
    /// match analyzer.validate_graph() {
    ///     Ok(()) => println!("Graph is valid"),
    ///     Err(e) => println!("Graph validation failed: {}", e),
    /// }
    /// ```
    pub fn validate_graph(&self) -> PackageResult<()> {
        if self.config.detect_circular {
            let cycles = self.graph.detect_cycles();
            if !cycles.is_empty() && self.config.fail_on_circular {
                return Err(DependencyError::CircularDependency {
                    cycle: cycles.into_iter().next().unwrap_or_default(),
                }
                .into());
            }
        }
        Ok(())
    }

    /// Gets the dependency graph.
    #[must_use]
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Gets the configuration used by this analyzer.
    #[must_use]
    pub fn config(&self) -> &DependencyConfig {
        &self.config
    }

    /// Determines the dependency relationship type between two packages.
    fn get_dependency_type(&self, dependent: &str, dependency: &str) -> DependencyType {
        if let Some(dependent_node) = self.graph.get_package(dependent) {
            if dependent_node.dependencies.contains_key(dependency) {
                return DependencyType::Runtime;
            }
            if dependent_node.dev_dependencies.contains_key(dependency) {
                return DependencyType::Development;
            }
            if dependent_node.optional_dependencies.contains_key(dependency) {
                return DependencyType::Optional;
            }
            if dependent_node.peer_dependencies.contains_key(dependency) {
                return DependencyType::Peer;
            }
        }
        // Default to runtime dependency if type cannot be determined
        DependencyType::Runtime
    }

    /// Checks if a dependency type should be included in propagation analysis.
    fn should_include_dependency_type(&self, dep_type: DependencyType) -> bool {
        match dep_type {
            DependencyType::Runtime => true,
            DependencyType::Development => self.config.propagate_dev_dependencies,
            DependencyType::Optional => self.config.include_optional_dependencies,
            DependencyType::Peer => self.config.include_peer_dependencies,
        }
    }

    /// Calculates the version bump to apply for dependency propagation.
    fn calculate_propagation_bump(
        &self,
        _original_bump: &VersionBump,
        _dependency_type: DependencyType,
    ) -> VersionBump {
        // Use configured default bump for dependency updates
        match self.config.dependency_update_bump.as_str() {
            "major" => VersionBump::Major,
            "minor" => VersionBump::Minor,
            "patch" => VersionBump::Patch,
            _ => VersionBump::Patch, // Default to patch
        }
    }

    /// Calculates the next version based on current version and bump type.
    fn calculate_next_version(
        &self,
        current_version: &ResolvedVersion,
        bump: &VersionBump,
    ) -> PackageResult<ResolvedVersion> {
        match current_version {
            ResolvedVersion::Release(version) => {
                let new_version = version.bump(*bump);
                Ok(ResolvedVersion::Release(new_version))
            }
            ResolvedVersion::Snapshot(_) => {
                // For snapshots, return as-is since they are ephemeral
                Ok(current_version.clone())
            }
        }
    }

    /// Gets the current version requirement for a dependency from the dependent package.
    fn get_current_dependency_version(
        &self,
        dependent_package: &str,
        dependency_package: &str,
        dependency_type: DependencyType,
    ) -> String {
        if let Some(dependent_node) = self.graph.get_package(dependent_package) {
            let dependencies = dependent_node.get_dependencies(dependency_type);
            if let Some(version_req) = dependencies.get(dependency_package) {
                return version_req.clone();
            }
        }
        "unknown".to_string()
    }
}

impl<F: AsyncFileSystem> DependencyGraphBuilder<F> {
    /// Creates a new dependency graph builder.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem for reading package.json files
    /// * `config` - Configuration for dependency analysis
    #[must_use]
    pub fn new(filesystem: F, config: DependencyConfig) -> Self {
        Self { filesystem, config }
    }

    /// Builds a dependency graph from a monorepo descriptor.
    ///
    /// # Arguments
    ///
    /// * `monorepo` - The monorepo descriptor to analyze
    ///
    /// # Returns
    ///
    /// A complete dependency graph with all packages and their relationships
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package.json files cannot be read
    /// - Package dependencies are malformed
    /// - Graph construction fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_pkg_tools::dependency::DependencyGraphBuilder;
    /// # use sublime_pkg_tools::config::DependencyConfig;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let config = DependencyConfig::default();
    /// let builder = DependencyGraphBuilder::new(fs, config);
    ///
    /// # let monorepo = todo!(); // Would come from MonorepoDetector
    /// let graph = builder.build_from_monorepo(&monorepo).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_from_monorepo(
        &self,
        monorepo: &MonorepoDescriptor,
    ) -> PackageResult<DependencyGraph> {
        let mut graph = DependencyGraph::new();

        // First pass: Add all packages as nodes
        for workspace_package in monorepo.packages() {
            let package_json_path = workspace_package.absolute_path.join("package.json");

            // Read package.json to get full dependency information
            let dependencies = match self.read_package_dependencies(&package_json_path).await {
                Ok(deps) => deps,
                Err(_) => {
                    // If we can't read package.json, create empty dependencies
                    Dependencies {
                        prod: HashMap::new(),
                        dev: HashMap::new(),
                        peer: HashMap::new(),
                        optional: HashMap::new(),
                    }
                }
            };

            let version = workspace_package.version.parse().map_err(|e| {
                DependencyError::invalid_specification(
                    &workspace_package.name,
                    &workspace_package.version,
                    format!("Invalid version: {}", e),
                )
            })?;

            let mut node = crate::dependency::DependencyNode::new(
                workspace_package.name.clone(),
                ResolvedVersion::Release(version),
                workspace_package.absolute_path.clone(),
            );

            // Add all dependencies to the node
            for (name, version_req) in dependencies.prod {
                node.add_dependency(name, version_req);
            }
            for (name, version_req) in dependencies.dev {
                node.add_dev_dependency(name, version_req);
            }
            for (name, version_req) in dependencies.optional {
                node.add_optional_dependency(name, version_req);
            }
            for (name, version_req) in dependencies.peer {
                node.add_peer_dependency(name, version_req);
            }

            graph.add_node(node);
        }

        // Second pass: Add edges between packages
        for workspace_package in monorepo.packages() {
            let package_json_path = workspace_package.absolute_path.join("package.json");

            if let Ok(dependencies) = self.read_package_dependencies(&package_json_path).await {
                // Add runtime dependencies
                for (dep_name, version_req) in &dependencies.prod {
                    if graph.get_package(dep_name).is_some() {
                        let edge = crate::dependency::DependencyEdge::new(
                            DependencyType::Runtime,
                            version_req.clone(),
                        );
                        let _ = graph.add_edge(&workspace_package.name, dep_name, edge);
                    }
                }

                // Add dev dependencies if configured
                if self.config.propagate_dev_dependencies {
                    for (dep_name, version_req) in &dependencies.dev {
                        if graph.get_package(dep_name).is_some() {
                            let edge = crate::dependency::DependencyEdge::new(
                                DependencyType::Development,
                                version_req.clone(),
                            );
                            let _ = graph.add_edge(&workspace_package.name, dep_name, edge);
                        }
                    }
                }

                // Add optional dependencies if configured
                if self.config.include_optional_dependencies {
                    for (dep_name, version_req) in &dependencies.optional {
                        if graph.get_package(dep_name).is_some() {
                            let edge = crate::dependency::DependencyEdge::new(
                                DependencyType::Optional,
                                version_req.clone(),
                            );
                            let _ = graph.add_edge(&workspace_package.name, dep_name, edge);
                        }
                    }
                }

                // Add peer dependencies if configured
                if self.config.include_peer_dependencies {
                    for (dep_name, version_req) in &dependencies.peer {
                        if graph.get_package(dep_name).is_some() {
                            let edge = crate::dependency::DependencyEdge::new(
                                DependencyType::Peer,
                                version_req.clone(),
                            );
                            let _ = graph.add_edge(&workspace_package.name, dep_name, edge);
                        }
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Reads dependencies from a package.json file.
    async fn read_package_dependencies(
        &self,
        package_json_path: &std::path::Path,
    ) -> PackageResult<Dependencies> {
        let content = self.filesystem.read_file_string(package_json_path).await.map_err(|e| {
            DependencyError::resolution_failed(
                package_json_path.display().to_string(),
                format!("Failed to read package.json: {}", e),
            )
        })?;

        let package_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            DependencyError::invalid_specification(
                package_json_path.display().to_string(),
                content.clone(),
                format!("Invalid JSON: {}", e),
            )
        })?;

        let mut dependencies = Dependencies {
            prod: HashMap::new(),
            dev: HashMap::new(),
            peer: HashMap::new(),
            optional: HashMap::new(),
        };

        // Parse production dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.prod.insert(name.clone(), version_str.to_string());
                }
            }
        }

        // Parse dev dependencies
        if let Some(deps) = package_json.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.dev.insert(name.clone(), version_str.to_string());
                }
            }
        }

        // Parse peer dependencies
        if let Some(deps) = package_json.get("peerDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.peer.insert(name.clone(), version_str.to_string());
                }
            }
        }

        // Parse optional dependencies
        if let Some(deps) = package_json.get("optionalDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.optional.insert(name.clone(), version_str.to_string());
                }
            }
        }

        Ok(dependencies)
    }
}
