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
use std::collections::HashMap;

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
    F: Clone,
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
        
        // Update the package (in a complete implementation, this would also
        // update the package.json file using self.filesystem)
        package.version = new_version.to_string();
        
        // TODO: Write updated package.json to filesystem
        // self.filesystem.write_json(package_json_path, updated_content).await?;
        
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
        
        // TODO: Update package.json file
        // self.filesystem.write_json(package_json_path, updated_content).await?;
        
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

        // TODO: Update package.json file
        // self.filesystem.write_json(package_json_path, updated_content).await?;

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

        // TODO: Batch write all updated package.json files
        // This would be more efficient than individual writes

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

        if removed {
            // TODO: Update package.json file
            // self.filesystem.write_json(package_json_path, updated_content).await?;
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