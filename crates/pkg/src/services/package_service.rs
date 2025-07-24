//! # Package Service
//!
//! Business logic service for package operations with async filesystem integration.
//!
//! ## Overview
//!
//! This service extracts all business logic from the Package struct, making Package
//! a pure data structure while providing enterprise-grade package management operations.
//!
//! ## Key Features
//!
//! - **Async-First Architecture**: All operations use async patterns with filesystem integration
//! - **Version Management**: Advanced version updating with validation
//! - **Dependency Operations**: Add, update, and manage package dependencies
//! - **Resolution Integration**: Apply dependency resolution results
//! - **Enterprise Patterns**: Clean separation of concerns and robust error handling
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::{PackageService, Package, BumpStrategy};
//! use sublime_standard_tools::filesystem::AsyncFileSystem;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create service with filesystem integration
//! let fs = AsyncFileSystem::new();
//! let service = PackageService::new(fs);
//!
//! // Load and update a package
//! let mut pkg = Package::new("my-app", "1.0.0", None)?;
//! service.update_package_version(&mut pkg, "1.1.0").await?;
//!
//! // Add a dependency
//! service.add_dependency_to_package(&mut pkg, "react", "^18.0.0").await?;
//!
//! // Update dependency version
//! service.update_dependency_version(&mut pkg, "react", "^18.2.0").await?;
//! # Ok(())
//! # }
//! ```

use crate::{
    errors::{DependencyResolutionError, VersionError},
    dependency::resolution::ResolutionResult,
    Dependency, Package,
};
use sublime_standard_tools::filesystem::AsyncFileSystem;
use std::collections::HashMap;
use std::path::Path;

/// Enterprise-grade package service for business logic operations
///
/// This service handles all package-related business operations that were previously
/// embedded in the Package struct, providing a clean separation between data and logic.
///
/// ## Architecture
///
/// - **Single Responsibility**: Only handles package business logic
/// - **Async Integration**: Works with AsyncFileSystem for I/O operations
/// - **Stateless**: No internal state, all operations work on provided Package instances
/// - **Error Handling**: Comprehensive error handling with detailed error types
///
/// ## Examples
///
/// ```rust
/// use sublime_package_tools::{PackageService, Package};
/// use sublime_standard_tools::filesystem::AsyncFileSystem;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = AsyncFileSystem::new();
/// let service = PackageService::new(fs);
///
/// let mut package = Package::new("my-package", "1.0.0", None)?;
/// service.update_package_version(&mut package, "1.1.0").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageService<F> {
    /// Filesystem integration for reading/writing package files
    filesystem: F,
}

impl<F> PackageService<F>
where
    F: AsyncFileSystem + Clone,
{
    /// Create a new package service with filesystem integration
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for I/O operations
    ///
    /// # Returns
    ///
    /// A new PackageService instance ready for package operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::PackageService;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// let fs = AsyncFileSystem::new();
    /// let service = PackageService::new(fs);
    /// ```
    #[must_use]
    pub fn new(filesystem: F) -> Self {
        Self { filesystem }
    }

    /// Update a package version with validation
    ///
    /// This method validates the new version string and updates the package
    /// instance. In a complete implementation, this would also update the
    /// package.json file on disk.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package to update
    /// * `new_version` - The new version string (must be valid semver)
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `VersionError` if the version is invalid
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - The version string is not valid semver
    /// - Filesystem operations fail (in complete implementation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let service = PackageService::new(fs);
    ///
    /// let mut package = Package::new("my-app", "1.0.0", None)?;
    /// service.update_package_version(&mut package, "1.1.0").await?;
    /// assert_eq!(package.version_str(), "1.1.0");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_package_version(
        &self,
        package: &mut Package,
        new_version: &str,
    ) -> Result<(), VersionError> {
        // Validate version string by parsing it
        let _ = semver::Version::parse(new_version)?;
        
        // Update the package
        package.version = new_version.to_string();
        
        // Write updated package.json to filesystem
        // Note: In a real implementation, we would need to know the package path
        // For now, this is a placeholder showing the pattern
        
        Ok(())
    }
    
    /// Update package version with explicit path
    ///
    /// This method updates both the in-memory package and the package.json file on disk.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package to update
    /// * `new_version` - The new version string (must be valid semver)
    /// * `package_json_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if version is invalid or I/O fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let service = PackageService::new(fs);
    ///
    /// let mut package = Package::new("my-app", "1.0.0", None)?;
    /// service.update_package_version_with_path(
    ///     &mut package, 
    ///     "1.1.0",
    ///     Path::new("./package.json")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_package_version_with_path(
        &self,
        package: &mut Package,
        new_version: &str,
        package_json_path: &Path,
    ) -> Result<(), VersionError> {
        // Validate version string by parsing it
        let _ = semver::Version::parse(new_version)?;
        
        // Read current package.json
        let mut package_json = self.read_package_json(package_json_path).await?;
        
        // Update version in package.json
        package_json["version"] = serde_json::Value::String(new_version.to_string());
        
        // Write updated package.json
        self.write_package_json(package_json_path, &package_json).await?;
        
        // Update the in-memory package
        package.version = new_version.to_string();
        
        Ok(())
    }

    /// Add a dependency to a package
    ///
    /// Creates a new dependency and adds it to the package. In a complete
    /// implementation, this would also update the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to add
    /// * `version_req` - Version requirement string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if dependency creation fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Version requirement string is invalid
    /// - Filesystem operations fail (in complete implementation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    ///
    /// let mut package = Package::new("my-app", "1.0.0", None)?;
    /// service.add_dependency_to_package(&mut package, "react", "^18.0.0").await?;
    /// assert_eq!(package.dependencies().len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_dependency_to_package(
        &self,
        package: &mut Package,
        dep_name: &str,
        version_req: &str,
    ) -> Result<(), VersionError> {
        // Create the new dependency
        let dependency = Dependency::new(dep_name, version_req)?;
        
        // Add to package
        package.dependencies.push(dependency);
        
        Ok(())
    }
    
    /// Add a dependency to a package with explicit path
    ///
    /// This method adds a dependency to both the in-memory package and the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to add
    /// * `version_req` - Version requirement string
    /// * `package_json_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if dependency creation fails or I/O fails
    pub async fn add_dependency_to_package_with_path(
        &self,
        package: &mut Package,
        dep_name: &str,
        version_req: &str,
        package_json_path: &Path,
    ) -> Result<(), VersionError> {
        // Create the new dependency
        let dependency = Dependency::new(dep_name, version_req)?;
        
        // Read current package.json
        let mut package_json = self.read_package_json(package_json_path).await?;
        
        // Ensure dependencies object exists
        if !package_json.get("dependencies").is_some() {
            package_json["dependencies"] = serde_json::json!({});
        }
        
        // Add dependency to package.json
        if let Some(deps) = package_json.get_mut("dependencies").and_then(|v| v.as_object_mut()) {
            deps.insert(dep_name.to_string(), serde_json::Value::String(version_req.to_string()));
        }
        
        // Write updated package.json
        self.write_package_json(package_json_path, &package_json).await?;
        
        // Add to in-memory package
        package.dependencies.push(dependency);
        
        Ok(())
    }

    /// Update a dependency version in a package
    ///
    /// Finds the specified dependency and updates its version requirement.
    /// In a complete implementation, this would also update the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to update
    /// * `new_version` - New version requirement string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if dependency not found or version invalid
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Dependency with specified name not found
    /// - New version requirement string is invalid
    /// - Filesystem operations fail (in complete implementation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package, Dependency};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    ///
    /// let dep = Dependency::new("react", "^17.0.0")?;
    /// let mut package = Package::new("my-app", "1.0.0", Some(vec![dep]))?;
    ///
    /// service.update_dependency_version(&mut package, "react", "^18.0.0").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_dependency_version(
        &self,
        package: &mut Package,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError> {
        // Find the dependency
        let package_name_clone = package.name().to_string();
        let dependency = package.dependencies.iter_mut()
            .find(|dep| dep.name() == dep_name)
            .ok_or_else(|| DependencyResolutionError::DependencyNotFound {
                name: dep_name.to_string(),
                package: package_name_clone,
            })?;

        // Update the dependency version
        dependency.update_version(new_version)
            .map_err(|e| DependencyResolutionError::VersionParseError(e.to_string()))?;

        Ok(())
    }
    
    /// Update a dependency version with explicit path
    ///
    /// This method updates a dependency in both the in-memory package and the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to update
    /// * `new_version` - New version requirement string
    /// * `package_json_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if dependency not found, version invalid, or I/O fails
    pub async fn update_dependency_version_with_path(
        &self,
        package: &mut Package,
        dep_name: &str,
        new_version: &str,
        package_json_path: &Path,
    ) -> Result<(), DependencyResolutionError> {
        // Update in-memory package first
        self.update_dependency_version(package, dep_name, new_version).await?;
        
        // Read current package.json
        let mut package_json = self.read_package_json(package_json_path).await
            .map_err(|e| DependencyResolutionError::VersionParseError(e.to_string()))?;
        
        // Update dependency in package.json
        if let Some(deps) = package_json.get_mut("dependencies").and_then(|v| v.as_object_mut()) {
            if deps.contains_key(dep_name) {
                deps.insert(dep_name.to_string(), serde_json::Value::String(new_version.to_string()));
            } else {
                return Err(DependencyResolutionError::DependencyNotFound {
                    name: dep_name.to_string(),
                    package: package.name().to_string(),
                });
            }
        }
        
        // Write updated package.json
        self.write_package_json(package_json_path, &package_json).await
            .map_err(|e| DependencyResolutionError::VersionParseError(e.to_string()))?;
        
        Ok(())
    }

    /// Apply dependency resolution results to packages
    ///
    /// Takes a resolution result and applies all the recommended updates
    /// to the specified packages. This is typically used after running
    /// dependency conflict resolution.
    ///
    /// # Arguments
    ///
    /// * `packages` - Mutable references to packages to update
    /// * `resolution` - Resolution result containing recommended updates
    ///
    /// # Returns
    ///
    /// `Ok(())` if all updates applied successfully, or error if any fail
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Any package references in resolution are not found
    /// - Any version updates are invalid
    /// - Filesystem operations fail (in complete implementation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package, ResolutionResult};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    ///
    /// let mut packages = vec![
    ///     Package::new("app1", "1.0.0", None)?,
    ///     Package::new("app2", "1.0.0", None)?,
    /// ];
    ///
    /// let resolution = ResolutionResult {
    ///     resolved_versions: HashMap::new(),
    ///     updates_required: Vec::new(),
    /// };
    ///
    /// service.apply_resolution_to_packages(&mut packages, &resolution).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_resolution_to_packages(
        &self,
        packages: &mut [Package],
        resolution: &ResolutionResult,
    ) -> Result<(), DependencyResolutionError> {
        // Create a lookup map for packages by name
        let mut package_map: HashMap<String, &mut Package> = HashMap::new();
        for package in packages.iter_mut() {
            package_map.insert(package.name().to_string(), package);
        }

        // Apply all updates from the resolution
        for update in &resolution.updates_required {
            let package_name = &update.package_name;
            let dep_name = &update.dependency_name;
            let new_version = &update.new_version;

            // Find the package
            let package = package_map.get_mut(package_name)
                .ok_or_else(|| DependencyResolutionError::DependencyNotFound {
                    name: package_name.clone(),
                    package: "resolution".to_string(),
                })?;

            // Update the dependency
            self.update_dependency_version(package, dep_name, new_version).await?;
        }

        Ok(())
    }

    /// Remove a dependency from a package
    ///
    /// Removes the specified dependency from the package's dependency list.
    /// In a complete implementation, this would also update the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to remove
    ///
    /// # Returns
    ///
    /// `Ok(true)` if dependency was found and removed, `Ok(false)` if not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package, Dependency};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    ///
    /// let dep = Dependency::new("lodash", "^4.17.21")?;
    /// let mut package = Package::new("my-app", "1.0.0", Some(vec![dep]))?;
    ///
    /// let removed = service.remove_dependency_from_package(&mut package, "lodash").await?;
    /// assert!(removed);
    /// assert_eq!(package.dependencies().len(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_dependency_from_package(
        &self,
        package: &mut Package,
        dep_name: &str,
    ) -> Result<bool, VersionError> {
        let initial_len = package.dependencies.len();
        package.dependencies.retain(|dep| dep.name() != dep_name);
        let removed = package.dependencies.len() < initial_len;

        Ok(removed)
    }
    
    /// Remove a dependency from a package with explicit path
    ///
    /// This method removes a dependency from both the in-memory package and the package.json file.
    ///
    /// # Arguments
    ///
    /// * `package` - Mutable reference to the package
    /// * `dep_name` - Name of the dependency to remove
    /// * `package_json_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// `Ok(true)` if dependency was found and removed, `Ok(false)` if not found
    pub async fn remove_dependency_from_package_with_path(
        &self,
        package: &mut Package,
        dep_name: &str,
        package_json_path: &Path,
    ) -> Result<bool, VersionError> {
        let initial_len = package.dependencies.len();
        package.dependencies.retain(|dep| dep.name() != dep_name);
        let removed = package.dependencies.len() < initial_len;

        if removed {
            // Read current package.json
            let mut package_json = self.read_package_json(package_json_path).await?;
            
            // Remove dependency from package.json
            if let Some(deps) = package_json.get_mut("dependencies").and_then(|v| v.as_object_mut()) {
                deps.remove(dep_name);
            }
            
            // Write updated package.json
            self.write_package_json(package_json_path, &package_json).await?;
        }

        Ok(removed)
    }

    /// Get package information summary
    ///
    /// Returns a summary of the package including dependency counts and version info.
    ///
    /// # Arguments
    ///
    /// * `package` - Reference to the package to analyze
    ///
    /// # Returns
    ///
    /// A `PackageInfo` struct containing summary information
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    /// let package = Package::new("my-app", "1.0.0", None)?;
    ///
    /// let info = service.get_package_info(&package).await;
    /// println!("Package {} has {} dependencies", info.name, info.dependency_count);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub async fn get_package_info(&self, package: &Package) -> PackageInfo {
        PackageInfo {
            name: package.name().to_string(),
            version: package.version_str(),
            dependency_count: package.dependencies().len(),
            has_dependencies: !package.dependencies().is_empty(),
        }
    }

    /// Validate package structure and dependencies
    ///
    /// Performs comprehensive validation of the package including version format,
    /// dependency versions, and structural integrity.
    ///
    /// # Arguments
    ///
    /// * `package` - Reference to the package to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = ();
    /// let service = PackageService::new(fs);
    /// let package = Package::new("my-app", "1.0.0", None)?;
    ///
    /// let validation = service.validate_package(&package).await;
    /// if validation.is_valid {
    ///     println!("Package is valid");
    /// } else {
    ///     for issue in validation.issues {
    ///         println!("Issue: {}", issue);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub async fn validate_package(&self, package: &Package) -> ValidationResult {
        let mut issues = Vec::new();

        // Validate package version
        if semver::Version::parse(package.version()).is_err() {
            issues.push(format!("Invalid package version: {}", package.version()));
        }

        // Validate dependency versions
        for dep in package.dependencies() {
            if let Err(_) = semver::VersionReq::parse(&dep.version().to_string()) {
                issues.push(format!("Invalid dependency version for {}: {}", dep.name(), dep.version()));
            }
        }

        // Check for empty package name
        if package.name().is_empty() {
            issues.push("Package name cannot be empty".to_string());
        }

        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
        }
    }
    
    /// Read package.json file from filesystem
    async fn read_package_json(&self, path: &Path) -> Result<serde_json::Value, VersionError> {
        let content = self.filesystem.read_file_string(path).await
            .map_err(|e| VersionError::IO(format!("Failed to read package.json: {e}")))?;
        
        serde_json::from_str(&content)
            .map_err(|e| VersionError::IO(format!("Failed to parse package.json: {e}")))
    }
    
    /// Write package.json file to filesystem
    async fn write_package_json(&self, path: &Path, package_json: &serde_json::Value) -> Result<(), VersionError> {
        let content = serde_json::to_string_pretty(package_json)
            .map_err(|e| VersionError::IO(format!("Failed to serialize package.json: {e}")))?;
        
        self.filesystem.write_file_string(path, &content).await
            .map_err(|e| VersionError::IO(format!("Failed to write package.json: {e}")))
    }
    
    /// Load a Package from a package.json file
    ///
    /// # Arguments
    ///
    /// * `package_json_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// A `Package` instance loaded from the file
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File cannot be read
    /// - JSON is invalid
    /// - Required fields are missing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::PackageService;
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let service = PackageService::new(fs);
    ///
    /// let package = service.load_package_from_file(Path::new("./package.json")).await?;
    /// println!("Loaded package: {} v{}", package.name(), package.version());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_package_from_file(&self, package_json_path: &Path) -> Result<Package, VersionError> {
        let package_json = self.read_package_json(package_json_path).await?;
        
        // Extract required fields
        let name = package_json.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| VersionError::IO("Missing 'name' field in package.json".to_string()))?;
        
        let version = package_json.get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| VersionError::IO("Missing 'version' field in package.json".to_string()))?;
        
        // Extract dependencies
        let mut dependencies = Vec::new();
        if let Some(deps_obj) = package_json.get("dependencies").and_then(|v| v.as_object()) {
            for (dep_name, dep_version) in deps_obj {
                if let Some(version_str) = dep_version.as_str() {
                    dependencies.push(Dependency::new(dep_name, version_str)?);
                }
            }
        }
        
        Package::new(name, version, if dependencies.is_empty() { None } else { Some(dependencies) })
    }
    
    /// Save a Package to a package.json file
    ///
    /// # Arguments
    ///
    /// * `package` - The package to save
    /// * `package_json_path` - Path where to save the package.json file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or error if I/O fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{PackageService, Package};
    /// use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystem::new();
    /// let service = PackageService::new(fs);
    ///
    /// let package = Package::new("my-app", "1.0.0", None)?;
    /// service.save_package_to_file(&package, Path::new("./package.json")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_package_to_file(&self, package: &Package, package_json_path: &Path) -> Result<(), VersionError> {
        // Build package.json structure
        let mut package_json = serde_json::json!({
            "name": package.name(),
            "version": package.version_str(),
        });
        
        // Add dependencies if any
        if !package.dependencies().is_empty() {
            let mut deps = serde_json::Map::new();
            for dep in package.dependencies() {
                deps.insert(dep.name().to_string(), serde_json::Value::String(dep.version().to_string()));
            }
            package_json["dependencies"] = serde_json::Value::Object(deps);
        }
        
        // Write to file
        self.write_package_json(package_json_path, &package_json).await
    }
}

/// Package information summary
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Number of dependencies
    pub dependency_count: usize,
    /// Whether package has any dependencies
    pub has_dependencies: bool,
}

/// Package validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the package passed all validations
    pub is_valid: bool,
    /// List of validation issues found
    pub issues: Vec<String>,
}