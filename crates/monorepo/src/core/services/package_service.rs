//! Package discovery service implementation for Node.js monorepos
//!
//! Handles package discovery, metadata parsing, and package relationship analysis
//! within Node.js monorepos. Provides centralized package management operations.

use super::FileSystemService;
use crate::config::MonorepoConfig;
use crate::core::types::MonorepoPackageInfo;
use crate::error::Result;
use std::path::Path;
use sublime_standard_tools::monorepo::{MonorepoDescriptor, PackageManager};

/// Package discovery and management service for Node.js monorepos
///
/// Provides comprehensive package discovery, metadata parsing, and package
/// relationship analysis for Node.js monorepos. Handles NPM/Yarn/PNPM workspaces.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::{PackageDiscoveryService, FileSystemService};
/// use sublime_monorepo_tools::config::MonorepoConfig;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs_service = FileSystemService::new("/path/to/monorepo")?;
/// let config = MonorepoConfig::default();
/// let package_service = PackageDiscoveryService::new("/path/to/monorepo", &fs_service, &config)?;
///
/// // Discover all packages in the Node.js monorepo
/// let packages = package_service.discover_packages()?;
/// println!("Found {} packages", packages.len());
///
/// // Get package manager information
/// let package_manager = package_service.get_package_manager();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct PackageDiscoveryService {
    /// Monorepo descriptor from standard-tools
    descriptor: MonorepoDescriptor,

    /// Package manager information (NPM/Yarn/PNPM/Bun)
    package_manager: PackageManager,

    /// Discovered packages cache
    packages: Vec<MonorepoPackageInfo>,

    /// Configuration for package discovery
    config: MonorepoConfig,
}

#[allow(dead_code)]
impl PackageDiscoveryService {
    /// Create a new package discovery service for Node.js monorepos
    ///
    /// Initializes the package discovery service with the specified root path,
    /// file system service, and configuration. Performs initial package discovery.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the Node.js monorepo
    /// * `file_system_service` - File system service for package discovery
    /// * `config` - Configuration for package discovery rules
    ///
    /// # Returns
    ///
    /// A new package discovery service with discovered packages.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Root path is not accessible
    /// - Package discovery fails
    /// - Package metadata cannot be parsed
    /// - Configuration is invalid for package discovery
    /// - No Node.js package manager is detected
    pub fn new<P: AsRef<Path>>(
        root_path: P,
        file_system_service: &FileSystemService,
        config: &MonorepoConfig,
    ) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Determine Node.js package manager first
        let package_manager = PackageManager::detect(&root_path).map_err(|e| {
            crate::error::Error::package(format!(
                "Failed to detect Node.js package manager for {}: {}",
                root_path.display(),
                e
            ))
        })?;

        // Determine monorepo kind based on Node.js package manager
        let kind = match package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => {
                sublime_standard_tools::monorepo::MonorepoKind::NpmWorkSpace
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => {
                sublime_standard_tools::monorepo::MonorepoKind::YarnWorkspaces
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => {
                sublime_standard_tools::monorepo::MonorepoKind::PnpmWorkspaces
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Bun => {
                sublime_standard_tools::monorepo::MonorepoKind::BunWorkspaces
            }
            sublime_standard_tools::monorepo::PackageManagerKind::Jsr => {
                return Err(crate::error::Error::package(format!(
                    "Unsupported package manager kind for Node.js monorepo: {:?}",
                    package_manager.kind()
                )));
            }
        };

        // Initialize with empty packages - will be populated later
        let descriptor = MonorepoDescriptor::new(
            kind,
            root_path.clone(),
            Vec::new(), // Packages will be discovered after initialization
        );

        let mut service =
            Self { descriptor, package_manager, packages: Vec::new(), config: config.clone() };

        // Perform initial package discovery
        service.refresh_packages(file_system_service)?;

        Ok(service)
    }

    /// Get the monorepo descriptor
    ///
    /// Provides access to the underlying monorepo descriptor which contains
    /// structural information about the monorepo layout and organization.
    ///
    /// # Returns
    ///
    /// Reference to the monorepo descriptor.
    pub fn descriptor(&self) -> &MonorepoDescriptor {
        &self.descriptor
    }

    /// Get the package manager
    ///
    /// Returns information about the detected Node.js package manager including
    /// type, version, and capabilities.
    ///
    /// # Returns
    ///
    /// Reference to the package manager information.
    pub fn get_package_manager(&self) -> &PackageManager {
        &self.package_manager
    }

    /// Discover all packages in the Node.js monorepo
    ///
    /// Returns the cached list of discovered packages. Use refresh_packages()
    /// to update the cache with current file system state.
    ///
    /// # Returns
    ///
    /// Vector of discovered packages with their metadata.
    #[allow(clippy::unnecessary_wraps)]
    pub fn discover_packages(&self) -> Result<Vec<MonorepoPackageInfo>> {
        Ok(self.packages.clone())
    }

    /// Get packages by name pattern
    ///
    /// Filters the discovered packages by name using a simple pattern match.
    /// Supports exact matches and wildcard patterns.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to match package names against
    ///
    /// # Returns
    ///
    /// Vector of packages matching the pattern.
    pub fn get_packages_by_pattern(&self, pattern: &str) -> Vec<&MonorepoPackageInfo> {
        if pattern.contains('*') {
            // Simple wildcard matching
            let pattern = pattern.replace('*', "");
            self.packages.iter().filter(|pkg| pkg.name().contains(&pattern)).collect()
        } else {
            // Exact match
            self.packages.iter().filter(|pkg| pkg.name() == pattern).collect()
        }
    }

    /// Get package by exact name
    ///
    /// Finds a package with the exact specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - Exact name of the package to find
    ///
    /// # Returns
    ///
    /// Reference to the package if found, None otherwise.
    pub fn get_package_by_name(&self, name: &str) -> Option<&MonorepoPackageInfo> {
        self.packages.iter().find(|pkg| pkg.name() == name)
    }

    /// Get all package names
    ///
    /// Returns a list of all discovered package names in the monorepo.
    ///
    /// # Returns
    ///
    /// Vector of package names.
    pub fn get_package_names(&self) -> Vec<String> {
        self.packages.iter().map(|pkg| pkg.name().to_string()).collect()
    }

    /// Get packages in a specific directory
    ///
    /// Returns packages that are located within or under the specified directory.
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory path relative to monorepo root
    ///
    /// # Returns
    ///
    /// Vector of packages in the specified directory.
    pub fn get_packages_in_directory(&self, directory: &str) -> Vec<&MonorepoPackageInfo> {
        self.packages
            .iter()
            .filter(|pkg| pkg.path().to_string_lossy().starts_with(directory))
            .collect()
    }

    /// Refresh package discovery cache
    ///
    /// Re-scans the file system to update the package cache with any
    /// newly added, removed, or modified packages.
    ///
    /// # Arguments
    ///
    /// * `file_system_service` - File system service for package discovery
    ///
    /// # Returns
    ///
    /// Success if packages were refreshed successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File system cannot be scanned
    /// - Package metadata cannot be parsed
    /// - Discovery configuration is invalid
    #[allow(clippy::unnecessary_wraps)]
    pub fn refresh_packages(&mut self, file_system_service: &FileSystemService) -> Result<()> {
        // Clear existing packages
        self.packages.clear();

        // Use workspace configuration to discover packages
        let workspace_patterns = &self.config.workspace.patterns;

        for pattern in workspace_patterns {
            if pattern.enabled {
                let discovered =
                    self.discover_packages_by_pattern(&pattern.pattern, file_system_service);
                self.packages.extend(discovered);
            }
        }

        // Remove duplicates by name (keep first occurrence)
        self.packages.sort_by(|a, b| a.name().cmp(b.name()));
        self.packages.dedup_by(|a, b| a.name() == b.name());

        Ok(())
    }

    /// Discover packages matching a specific pattern
    ///
    /// Internal method to discover packages matching a glob pattern
    /// within the monorepo structure.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern for package discovery
    /// * `file_system_service` - File system service for directory traversal
    ///
    /// # Returns
    ///
    /// Vector of discovered packages matching the pattern.
    fn discover_packages_by_pattern(
        &self,
        pattern: &str,
        file_system_service: &FileSystemService,
    ) -> Vec<MonorepoPackageInfo> {
        let mut packages = Vec::new();

        // For now, implement a simple directory-based discovery
        // This would be enhanced with proper glob pattern matching
        let search_dirs = if pattern.contains('*') {
            // Extract directory part before wildcard
            let dir_part = pattern.split('*').next().unwrap_or("");
            vec![dir_part.trim_end_matches('/')]
        } else {
            vec![pattern]
        };

        for search_dir in search_dirs {
            if file_system_service.is_dir(search_dir) {
                let discovered = self.scan_directory_for_packages(search_dir, file_system_service);
                packages.extend(discovered);
            }
        }

        packages
    }

    /// Scan directory for Node.js package files
    ///
    /// Internal method to scan a directory and its subdirectories for
    /// package.json files.
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory to scan relative to monorepo root
    /// * `file_system_service` - File system service for directory operations
    ///
    /// # Returns
    ///
    /// Vector of packages found in the directory.
    fn scan_directory_for_packages(
        &self,
        directory: &str,
        file_system_service: &FileSystemService,
    ) -> Vec<MonorepoPackageInfo> {
        let mut packages = Vec::new();

        // Check for Node.js package definition files
        let package_files = ["package.json"];

        for &package_file in &package_files {
            let package_path = if directory.is_empty() {
                package_file.to_string()
            } else {
                format!("{directory}/{package_file}")
            };

            if file_system_service.exists(&package_path) {
                match Self::parse_package_info(&package_path, file_system_service) {
                    Ok(package_info) => packages.push(package_info),
                    Err(e) => {
                        // Log warning but continue discovery
                        eprintln!("Warning: Failed to parse package {package_path}: {e}");
                    }
                }
            }
        }

        // Recursively scan subdirectories if configured
        // Use discovery settings for nested packages
        let include_nested =
            self.config.workspace.patterns.first().map_or(true, |p| p.options.include_nested);

        if include_nested {
            match file_system_service.list_directory(directory) {
                Ok(entries) => {
                    for entry in entries {
                        if let Some(dir_name) = entry.file_name() {
                            if let Some(dir_str) = dir_name.to_str() {
                                if file_system_service.is_dir(format!("{directory}/{dir_str}")) {
                                    let subdir = if directory.is_empty() {
                                        dir_str.to_string()
                                    } else {
                                        format!("{directory}/{dir_str}")
                                    };

                                    let subpackages = self
                                        .scan_directory_for_packages(&subdir, file_system_service);
                                    packages.extend(subpackages);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to list directory {directory}: {e}");
                }
            }
        }

        packages
    }

    /// Parse package information from package.json file
    ///
    /// Internal method to parse package metadata from package.json files.
    ///
    /// # Arguments
    ///
    /// * `package_file_path` - Path to package.json file
    /// * `file_system_service` - File system service for file reading
    ///
    /// # Returns
    ///
    /// Parsed package information.
    ///
    /// # Errors
    ///
    /// Returns an error if the package file cannot be read or parsed.
    fn parse_package_info(
        package_file_path: &str,
        file_system_service: &FileSystemService,
    ) -> Result<MonorepoPackageInfo> {
        let content = file_system_service.read_file_string(package_file_path)?;
        let package_dir = std::path::Path::new(package_file_path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        // Parse package.json only (Node.js monorepo)
        if package_file_path.ends_with("package.json") {
            Self::parse_npm_package(&content, &package_dir)
        } else {
            Err(crate::error::Error::package(format!(
                "Unsupported package file format: {package_file_path}. Only package.json is supported for Node.js monorepos."
            )))
        }
    }

    /// Parse NPM package.json file
    ///
    /// Internal method to parse NPM package metadata from package.json.
    ///
    /// # Arguments
    ///
    /// * `content` - Content of the package.json file
    /// * `package_dir` - Directory containing the package
    ///
    /// # Returns
    ///
    /// Parsed package information.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON parsing fails or required fields are missing.
    fn parse_npm_package(
        content: &str,
        package_dir: &std::path::Path,
    ) -> Result<MonorepoPackageInfo> {
        // Parse JSON using serde_json
        let package_json: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            crate::error::Error::package(format!("Failed to parse package.json: {e}"))
        })?;

        let name = package_json
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::Error::package("Missing 'name' field in package.json".to_string())
            })?
            .to_string();

        let version = package_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::Error::package("Missing 'version' field in package.json".to_string())
            })?
            .to_string();

        // Parse dependencies
        let dependencies = Self::parse_package_json_dependencies(&package_json);
        let dependencies_external = Self::extract_external_dependencies(&dependencies);

        // Create proper package using sublime-package-tools
        let package = sublime_package_tools::Package::new(&name, &version, None).map_err(|e| {
            crate::error::Error::package(format!("Failed to create package {name}: {e}"))
        })?;

        let package_info = sublime_package_tools::Info {
            package: std::rc::Rc::new(std::cell::RefCell::new(package)),
            package_json_path: package_dir.join("package.json").to_string_lossy().to_string(),
            package_path: package_dir.to_string_lossy().to_string(),
            package_relative_path: package_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    crate::error::Error::package("Invalid package directory name".to_string())
                })?
                .to_string(),
            pkg_json: std::rc::Rc::new(std::cell::RefCell::new(package_json.clone())),
        };

        // Parse workspace dependencies from package.json
        let (workspace_dependencies, workspace_dev_dependencies) =
            Self::parse_workspace_dependencies(&package_json);

        let workspace_package = sublime_standard_tools::monorepo::WorkspacePackage {
            name: name.clone(),
            version: version.clone(),
            location: package_dir.to_path_buf(),
            absolute_path: package_dir.to_path_buf(),
            workspace_dependencies,
            workspace_dev_dependencies,
        };

        Ok(MonorepoPackageInfo {
            package_info,
            workspace_package,
            is_internal: true,
            dependents: Vec::new(),
            dependencies,
            dependencies_external,
            version_status: crate::core::types::VersionStatus::Stable,
            changesets: Vec::new(),
        })
    }

    /// Parse dependencies from package.json
    fn parse_package_json_dependencies(
        package_json: &serde_json::Value,
    ) -> Vec<crate::core::types::PackageDependency> {
        let mut dependencies = Vec::new();

        // Parse regular dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(crate::core::types::PackageDependency {
                    name: name.clone(),
                    version_requirement: version.as_str().unwrap_or("*").to_string(),
                    dependency_type: crate::core::types::DependencyType::Runtime,
                    optional: false,
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        // Parse dev dependencies
        if let Some(deps) = package_json.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(crate::core::types::PackageDependency {
                    name: name.clone(),
                    version_requirement: version.as_str().unwrap_or("*").to_string(),
                    dependency_type: crate::core::types::DependencyType::Development,
                    optional: false,
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        // Parse peer dependencies
        if let Some(deps) = package_json.get("peerDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(crate::core::types::PackageDependency {
                    name: name.clone(),
                    version_requirement: version.as_str().unwrap_or("*").to_string(),
                    dependency_type: crate::core::types::DependencyType::Peer,
                    optional: false,
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        // Parse optional dependencies
        if let Some(deps) = package_json.get("optionalDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(crate::core::types::PackageDependency {
                    name: name.clone(),
                    version_requirement: version.as_str().unwrap_or("*").to_string(),
                    dependency_type: crate::core::types::DependencyType::Optional,
                    optional: true,
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        dependencies
    }

    /// Extract external dependencies (not part of the monorepo)
    fn extract_external_dependencies(
        dependencies: &[crate::core::types::PackageDependency],
    ) -> Vec<String> {
        // This would typically check against the list of internal packages
        // For now, assume all are external - this should be improved with workspace detection
        dependencies.iter().map(|dep| dep.name.clone()).collect()
    }

    /// Parse workspace dependencies from package.json
    fn parse_workspace_dependencies(
        package_json: &serde_json::Value,
    ) -> (Vec<String>, Vec<String>) {
        let mut workspace_deps = Vec::new();
        let mut workspace_dev_deps = Vec::new();

        // Check for workspace: protocol in dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    if version_str.starts_with("workspace:") {
                        workspace_deps.push(name.clone());
                    }
                }
            }
        }

        // Check for workspace: protocol in devDependencies
        if let Some(deps) = package_json.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    if version_str.starts_with("workspace:") {
                        workspace_dev_deps.push(name.clone());
                    }
                }
            }
        }

        (workspace_deps, workspace_dev_deps)
    }
}
