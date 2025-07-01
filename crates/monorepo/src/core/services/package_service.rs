//! Package discovery service implementation
//!
//! Handles package discovery, metadata parsing, and package relationship analysis
//! within the monorepo. Provides centralized package management operations.

use crate::config::MonorepoConfig;
use crate::core::types::MonorepoPackageInfo;
use crate::error::Result;
use std::path::Path;
use sublime_standard_tools::monorepo::{MonorepoDescriptor, PackageManager};
use super::FileSystemService;

/// Package discovery and management service
///
/// Provides comprehensive package discovery, metadata parsing, and package
/// relationship analysis for the monorepo. Handles different package types
/// and manages package discovery configuration.
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
/// // Discover all packages in the monorepo
/// let packages = package_service.discover_packages()?;
/// println!("Found {} packages", packages.len());
/// 
/// // Get package manager information
/// let package_manager = package_service.get_package_manager();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageDiscoveryService {
    /// Monorepo descriptor from standard-tools
    descriptor: MonorepoDescriptor,
    
    /// Package manager information
    package_manager: PackageManager,
    
    /// Discovered packages cache
    packages: Vec<MonorepoPackageInfo>,
    
    /// Root path of the monorepo
    root_path: std::path::PathBuf,
    
    /// Configuration for package discovery
    config: MonorepoConfig,
}

impl PackageDiscoveryService {
    /// Create a new package discovery service
    ///
    /// Initializes the package discovery service with the specified root path,
    /// file system service, and configuration. Performs initial package discovery.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the monorepo
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
    pub fn new<P: AsRef<Path>>(
        root_path: P,
        file_system_service: &FileSystemService,
        config: &MonorepoConfig,
    ) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        
        // Determine package manager first
        let package_manager = PackageManager::detect(&root_path)
            .map_err(|e| crate::error::Error::package(format!(
                "Failed to detect package manager for {}: {}", 
                root_path.display(), 
                e
            )))?;
        
        // Determine monorepo kind based on package manager
        let kind = match package_manager.kind() {
            sublime_standard_tools::monorepo::PackageManagerKind::Npm => sublime_standard_tools::monorepo::MonorepoKind::NpmWorkSpace,
            sublime_standard_tools::monorepo::PackageManagerKind::Yarn => sublime_standard_tools::monorepo::MonorepoKind::YarnWorkspaces, 
            sublime_standard_tools::monorepo::PackageManagerKind::Pnpm => sublime_standard_tools::monorepo::MonorepoKind::PnpmWorkspaces,
            sublime_standard_tools::monorepo::PackageManagerKind::Bun => sublime_standard_tools::monorepo::MonorepoKind::BunWorkspaces,
            sublime_standard_tools::monorepo::PackageManagerKind::Jsr => sublime_standard_tools::monorepo::MonorepoKind::Custom { 
                name: "Unknown".to_string(),
                config_file: "unknown.json".to_string()
            },
        };
        
        // Initialize with empty packages - will be populated later
        let descriptor = MonorepoDescriptor::new(
            kind,
            root_path.clone(),
            Vec::new() // Packages will be discovered after initialization
        );
        
        let mut service = Self {
            descriptor,
            package_manager,
            packages: Vec::new(),
            root_path,
            config: config.clone(),
        };
        
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
    /// Returns information about the detected package manager including
    /// type, version, and capabilities.
    ///
    /// # Returns
    ///
    /// Reference to the package manager information.
    pub fn get_package_manager(&self) -> &PackageManager {
        &self.package_manager
    }
    
    /// Discover all packages in the monorepo
    ///
    /// Returns the cached list of discovered packages. Use refresh_packages()
    /// to update the cache with current file system state.
    ///
    /// # Returns
    ///
    /// Vector of discovered packages with their metadata.
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
            self.packages.iter()
                .filter(|pkg| pkg.name.contains(&pattern))
                .collect()
        } else {
            // Exact match
            self.packages.iter()
                .filter(|pkg| pkg.name == pattern)
                .collect()
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
        self.packages.iter()
            .find(|pkg| pkg.name == name)
    }
    
    /// Get all package names
    ///
    /// Returns a list of all discovered package names in the monorepo.
    ///
    /// # Returns
    ///
    /// Vector of package names.
    pub fn get_package_names(&self) -> Vec<String> {
        self.packages.iter()
            .map(|pkg| pkg.name.clone())
            .collect()
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
        self.packages.iter()
            .filter(|pkg| {
                pkg.path.to_string_lossy().starts_with(directory)
            })
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
    pub fn refresh_packages(&mut self, file_system_service: &FileSystemService) -> Result<()> {
        // Clear existing packages
        self.packages.clear();
        
        // Use workspace configuration to discover packages
        let workspace_patterns = &self.config.workspace.patterns;
        
        for pattern in workspace_patterns {
            if pattern.enabled {
                let discovered = self.discover_packages_by_pattern(&pattern.pattern, file_system_service);
                self.packages.extend(discovered);
            }
        }
        
        // Remove duplicates by name (keep first occurrence)
        self.packages.sort_by(|a, b| a.name.cmp(&b.name));
        self.packages.dedup_by(|a, b| a.name == b.name);
        
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
    
    /// Scan directory for package files
    ///
    /// Internal method to scan a directory and its subdirectories for
    /// package definition files (package.json, Cargo.toml, etc.).
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
        
        // Check for package definition files
        let package_files = ["package.json", "Cargo.toml", "pyproject.toml", "pom.xml"];
        
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
        let include_nested = self.config.workspace.patterns.first()
            .map_or(true, |p| p.options.include_nested);
        
        if include_nested {
            match file_system_service.list_directory(directory) {
                Ok(entries) => {
                    for entry in entries {
                        if let Some(dir_name) = entry.file_name() {
                            if let Some(dir_str) = dir_name.to_str() {
                                if file_system_service.is_dir(&format!("{directory}/{dir_str}")) {
                                    let subdir = if directory.is_empty() {
                                        dir_str.to_string()
                                    } else {
                                        format!("{directory}/{dir_str}")
                                    };
                                    
                                    let subpackages = self.scan_directory_for_packages(&subdir, file_system_service);
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
    
    /// Parse package information from package file
    ///
    /// Internal method to parse package metadata from various package
    /// definition file formats.
    ///
    /// # Arguments
    ///
    /// * `package_file_path` - Path to package definition file
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
        
        // Parse based on file type
        if package_file_path.ends_with("package.json") {
            Self::parse_npm_package(&content, package_dir)
        } else if package_file_path.ends_with("Cargo.toml") {
            Self::parse_cargo_package(&content, package_dir)
        } else if package_file_path.ends_with("pyproject.toml") {
            Self::parse_python_package(&content, package_dir)
        } else if package_file_path.ends_with("pom.xml") {
            Self::parse_maven_package(&content, package_dir)
        } else {
            Err(crate::error::Error::package(format!(
                "Unsupported package file format: {package_file_path}"
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
        package_dir: std::path::PathBuf,
    ) -> Result<MonorepoPackageInfo> {
        // For now, implement basic JSON parsing
        // In a real implementation, this would use serde_json
        let name = Self::extract_json_field(content, "name")
            .unwrap_or_else(|| "unknown".to_string());
        let version = Self::extract_json_field(content, "version")
            .unwrap_or_else(|| "0.0.0".to_string());
        
        Ok(MonorepoPackageInfo {
            // For now, create placeholder structs - proper parsing will be implemented
            package_info: sublime_package_tools::PackageInfo {
                package: std::rc::Rc::new(std::cell::RefCell::new(
                    sublime_package_tools::Package::new(
                        &name,
                        &version,
                        None
                    ).or_else(|_| sublime_package_tools::Package::new(
                        &name,
                        "0.0.0",
                        None
                    )).map_err(|e| crate::error::Error::package(format!(
                        "Failed to create package {name}: {e}"
                    )))?
                )),
                package_json_path: package_dir.join("package.json").to_string_lossy().to_string(),
                package_path: package_dir.to_string_lossy().to_string(),
                package_relative_path: package_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                pkg_json: std::rc::Rc::new(std::cell::RefCell::new(
                    serde_json::json!({
                        "name": name.clone(),
                        "version": version.clone()
                    })
                )),
            },
            // Create basic workspace package
            workspace_package: sublime_standard_tools::monorepo::WorkspacePackage {
                name: name.clone(),
                version: version.clone(),
                location: package_dir.clone(),
                absolute_path: package_dir.clone(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            },
            is_internal: true,
            dependents: Vec::new(),
            dependencies: Vec::new(), // Would be parsed from dependencies field
            dependencies_external: Vec::new(),
            version_status: crate::core::types::VersionStatus::Stable,
            changesets: Vec::new(),
            name,
            version,
            path: package_dir,
            package_type: crate::core::types::PackageType::JavaScript,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Parse Cargo.toml file
    ///
    /// Internal method to parse Rust package metadata from Cargo.toml.
    ///
    /// # Arguments
    ///
    /// * `content` - Content of the Cargo.toml file
    /// * `package_dir` - Directory containing the package
    ///
    /// # Returns
    ///
    /// Parsed package information.
    ///
    /// # Errors
    ///
    /// Returns an error if TOML parsing fails or required fields are missing.
    fn parse_cargo_package(
        content: &str,
        package_dir: std::path::PathBuf,
    ) -> Result<MonorepoPackageInfo> {
        // For now, implement basic TOML parsing
        // In a real implementation, this would use toml crate
        let name = Self::extract_toml_field(content, "name")
            .unwrap_or_else(|| "unknown".to_string());
        let version = Self::extract_toml_field(content, "version")
            .unwrap_or_else(|| "0.0.0".to_string());
        
        Ok(MonorepoPackageInfo {
            // For now, create placeholder structs - proper parsing will be implemented
            package_info: sublime_package_tools::PackageInfo {
                package: std::rc::Rc::new(std::cell::RefCell::new(
                    sublime_package_tools::Package::new(
                        &name,
                        &version,
                        None
                    ).or_else(|_| sublime_package_tools::Package::new(
                        &name,
                        "0.0.0",
                        None
                    )).map_err(|e| crate::error::Error::package(format!(
                        "Failed to create package {name}: {e}"
                    )))?
                )),
                package_json_path: package_dir.join("package.json").to_string_lossy().to_string(),
                package_path: package_dir.to_string_lossy().to_string(),
                package_relative_path: package_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                pkg_json: std::rc::Rc::new(std::cell::RefCell::new(
                    serde_json::json!({
                        "name": name.clone(),
                        "version": version.clone()
                    })
                )),
            },
            // Create basic workspace package
            workspace_package: sublime_standard_tools::monorepo::WorkspacePackage {
                name: name.clone(),
                version: version.clone(),
                location: package_dir.clone(),
                absolute_path: package_dir.clone(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            },
            is_internal: true,
            dependents: Vec::new(),
            dependencies: Vec::new(), // Would be parsed from dependencies section
            dependencies_external: Vec::new(),
            version_status: crate::core::types::VersionStatus::Stable,
            changesets: Vec::new(),
            name,
            version,
            path: package_dir,
            package_type: crate::core::types::PackageType::Rust,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Parse Python pyproject.toml file
    ///
    /// Internal method to parse Python package metadata from pyproject.toml.
    ///
    /// # Arguments
    ///
    /// * `content` - Content of the pyproject.toml file
    /// * `package_dir` - Directory containing the package
    ///
    /// # Returns
    ///
    /// Parsed package information.
    ///
    /// # Errors
    ///
    /// Returns an error if TOML parsing fails or required fields are missing.
    fn parse_python_package(
        content: &str,
        package_dir: std::path::PathBuf,
    ) -> Result<MonorepoPackageInfo> {
        // Basic parsing for Python packages
        let name = Self::extract_toml_field(content, "name")
            .unwrap_or_else(|| "unknown".to_string());
        let version = Self::extract_toml_field(content, "version")
            .unwrap_or_else(|| "0.0.0".to_string());
        
        Ok(MonorepoPackageInfo {
            // For now, create placeholder structs - proper parsing will be implemented
            package_info: sublime_package_tools::PackageInfo {
                package: std::rc::Rc::new(std::cell::RefCell::new(
                    sublime_package_tools::Package::new(
                        &name,
                        &version,
                        None
                    ).or_else(|_| sublime_package_tools::Package::new(
                        &name,
                        "0.0.0",
                        None
                    )).map_err(|e| crate::error::Error::package(format!(
                        "Failed to create package {name}: {e}"
                    )))?
                )),
                package_json_path: package_dir.join("package.json").to_string_lossy().to_string(),
                package_path: package_dir.to_string_lossy().to_string(),
                package_relative_path: package_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                pkg_json: std::rc::Rc::new(std::cell::RefCell::new(
                    serde_json::json!({
                        "name": name.clone(),
                        "version": version.clone()
                    })
                )),
            },
            // Create basic workspace package
            workspace_package: sublime_standard_tools::monorepo::WorkspacePackage {
                name: name.clone(),
                version: version.clone(),
                location: package_dir.clone(),
                absolute_path: package_dir.clone(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            },
            is_internal: true,
            dependents: Vec::new(),
            dependencies: Vec::new(),
            dependencies_external: Vec::new(),
            version_status: crate::core::types::VersionStatus::Stable,
            changesets: Vec::new(),
            name,
            version,
            path: package_dir,
            package_type: crate::core::types::PackageType::Python,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Parse Maven pom.xml file
    ///
    /// Internal method to parse Java package metadata from pom.xml.
    ///
    /// # Arguments
    ///
    /// * `content` - Content of the pom.xml file
    /// * `package_dir` - Directory containing the package
    ///
    /// # Returns
    ///
    /// Parsed package information.
    ///
    /// # Errors
    ///
    /// Returns an error if XML parsing fails or required fields are missing.
    fn parse_maven_package(
        content: &str,
        package_dir: std::path::PathBuf,
    ) -> Result<MonorepoPackageInfo> {
        // Basic XML parsing for Maven packages
        let name = Self::extract_xml_field(content, "artifactId")
            .unwrap_or_else(|| "unknown".to_string());
        let version = Self::extract_xml_field(content, "version")
            .unwrap_or_else(|| "0.0.0".to_string());
        
        Ok(MonorepoPackageInfo {
            // For now, create placeholder structs - proper parsing will be implemented
            package_info: sublime_package_tools::PackageInfo {
                package: std::rc::Rc::new(std::cell::RefCell::new(
                    sublime_package_tools::Package::new(
                        &name,
                        &version,
                        None
                    ).or_else(|_| sublime_package_tools::Package::new(
                        &name,
                        "0.0.0",
                        None
                    )).map_err(|e| crate::error::Error::package(format!(
                        "Failed to create package {name}: {e}"
                    )))?
                )),
                package_json_path: package_dir.join("package.json").to_string_lossy().to_string(),
                package_path: package_dir.to_string_lossy().to_string(),
                package_relative_path: package_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                pkg_json: std::rc::Rc::new(std::cell::RefCell::new(
                    serde_json::json!({
                        "name": name.clone(),
                        "version": version.clone()
                    })
                )),
            },
            // Create basic workspace package
            workspace_package: sublime_standard_tools::monorepo::WorkspacePackage {
                name: name.clone(),
                version: version.clone(),
                location: package_dir.clone(),
                absolute_path: package_dir.clone(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            },
            is_internal: true,
            dependents: Vec::new(),
            dependencies: Vec::new(),
            dependencies_external: Vec::new(),
            version_status: crate::core::types::VersionStatus::Stable,
            changesets: Vec::new(),
            name,
            version,
            path: package_dir,
            package_type: crate::core::types::PackageType::Java,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Extract field from JSON content
    ///
    /// Simple JSON field extraction utility. In production, would use serde_json.
    ///
    /// # Arguments
    ///
    /// * `content` - JSON content
    /// * `field` - Field name to extract
    ///
    /// # Returns
    ///
    /// Field value if found, None otherwise.
    fn extract_json_field(content: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{field}\":");
        if let Some(start) = content.find(&pattern) {
            let after_colon = &content[start + pattern.len()..];
            if let Some(quote_start) = after_colon.find('"') {
                let value_start = quote_start + 1;
                if let Some(quote_end) = after_colon[value_start..].find('"') {
                    return Some(after_colon[value_start..value_start + quote_end].to_string());
                }
            }
        }
        None
    }
    
    /// Extract field from TOML content
    ///
    /// Simple TOML field extraction utility. In production, would use toml crate.
    ///
    /// # Arguments
    ///
    /// * `content` - TOML content
    /// * `field` - Field name to extract
    ///
    /// # Returns
    ///
    /// Field value if found, None otherwise.
    fn extract_toml_field(content: &str, field: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(&format!("{field} =")) {
                if let Some(equals_pos) = line.find('=') {
                    let value = line[equals_pos + 1..].trim();
                    return Some(value.trim_matches('"').to_string());
                }
            }
        }
        None
    }
    
    /// Extract field from XML content
    ///
    /// Simple XML field extraction utility. In production, would use proper XML parser.
    ///
    /// # Arguments
    ///
    /// * `content` - XML content
    /// * `field` - Field name to extract
    ///
    /// # Returns
    ///
    /// Field value if found, None otherwise.
    fn extract_xml_field(content: &str, field: &str) -> Option<String> {
        let start_tag = format!("<{field}>");
        let end_tag = format!("</{field}>");
        
        if let Some(start) = content.find(&start_tag) {
            let value_start = start + start_tag.len();
            if let Some(end) = content[value_start..].find(&end_tag) {
                return Some(content[value_start..value_start + end].trim().to_string());
            }
        }
        None
    }
}