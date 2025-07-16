//! # Package Module
//!
//! This module provides the `Package` structure for representing Node.js packages.
//!
//! A `Package` represents a Node.js package with name, version, and dependencies.
//! It's a key building block for dependency analysis and management.
//!
//! ## Key Features
//!
//! - Create and manage Node.js package metadata
//! - Handle package dependencies
//! - Implement the `Node` trait for dependency graph operations
//! - Version management and updates
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{Package, Registry};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a package with dependencies using a registry
//! let mut registry = Registry::new();
//! let pkg = Package::new_with_registry(
//!     "my-app",
//!     "1.0.0",
//!     Some(vec![
//!         ("react", "^17.0.2"),
//!         ("lodash", "^4.17.21")
//!     ]),
//!     &mut registry
//! )?;
//!
//! // Access package information
//! println!("Package: {} v{}", pkg.name(), pkg.version_str());
//!
//! // Update package version
//! pkg.update_version("1.1.0")?;
//!
//! // Update a dependency version
//! pkg.update_dependency_version("react", "^18.0.0")?;
//! # Ok(())
//! # }
//! ```

use crate::{
    errors::{DependencyResolutionError, VersionError},
    Dependency, Registry, Node, ResolutionResult,
};
use semver::Version;
use std::{cell::RefCell, rc::Rc}; // Still needed for version field

/// A package with its dependencies and version information
///
/// Represents an NPM package with its name, version, and dependencies. The `Package`
/// struct is the central data structure for dependency graph analysis and manipulation.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{Package, Dependency};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a package without dependencies
/// let simple_pkg = Package::new("my-app", "1.0.0", None)?;
///
/// // Create a package with dependencies
/// let dep1 = Dependency::new("react", "^17.0.2")?;
/// let dep2 = Dependency::new("lodash", "^4.17.21")?;
///
/// let pkg_with_deps = Package::new(
///     "my-app-with-deps",
///     "1.0.0",
///     Some(vec![dep1, dep2])
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    version: Rc<RefCell<Version>>,
    dependencies: Vec<Dependency>,
}

impl Node for Package {
    type DependencyType = crate::Dependency;
    type Identifier = String;

    fn dependencies(&self) -> Vec<&Self::DependencyType> {
        self.dependencies.iter().collect()
    }

    fn dependencies_vec(&self) -> Vec<Self::DependencyType> {
        self.dependencies.clone()
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        self.name() == dependency.name() && dependency.version().matches(&self.version())
    }

    fn identifier(&self) -> Self::Identifier {
        self.name().to_string()
    }
}

impl Package {
    /// Create a new package with name, version, and optional dependencies
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `version` - The package version as a semver string
    /// * `dependencies` - Optional list of dependencies
    ///
    /// # Returns
    ///
    /// A new `Package` instance, or a `VersionError` if the version string is invalid
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("my-package", "1.0.0", None)?;
    /// assert_eq!(pkg.name(), "my-package");
    /// assert_eq!(pkg.version_str(), "1.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Dependency>>,
    ) -> Result<Self, VersionError> {
        let parsed_version = version.parse()?;

        Ok(Self {
            name: name.to_string(),
            version: Rc::new(RefCell::new(parsed_version)),
            dependencies: dependencies.unwrap_or_default(),
        })
    }

    /// Create a new package using the dependency registry
    ///
    /// Creates a package and registers its dependencies with the provided registry.
    /// This ensures consistent dependency instances across multiple packages.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `version` - The package version as a semver string
    /// * `dependencies` - Optional list of dependency name/version pairs
    /// * `registry` - Registry to manage dependency instances
    ///
    /// # Returns
    ///
    /// A new `Package` instance, or a `VersionError` if any version is invalid
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The package version string is not a valid semantic version
    /// - Any dependency version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Registry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// let pkg = Package::new_with_registry(
    ///     "my-package",
    ///     "1.0.0",
    ///     Some(vec![("react", "^17.0.2"), ("lodash", "^4.17.21")]),
    ///     &mut registry
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_registry(
        name: &str,
        version: &str,
        dependencies: Option<Vec<(&str, &str)>>,
        registry: &mut Registry,
    ) -> Result<Self, VersionError> {
        let deps = if let Some(dep_list) = dependencies {
            let mut deps_vec = Vec::new();
            for (dep_name, dep_version) in dep_list {
                let dep = registry.get_or_create(dep_name, dep_version)?;
                deps_vec.push(dep);
            }
            deps_vec
        } else {
            Vec::new()
        };

        Self::new(name, version, Some(deps))
    }

    /// Get the package name
    ///
    /// # Returns
    ///
    /// The name of the package as a string slice
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package version
    ///
    /// # Returns
    ///
    /// A clone of the package's semantic version
    #[must_use]
    pub fn version(&self) -> Version {
        self.version.borrow().clone()
    }

    /// Get the package version as a string
    ///
    /// # Returns
    ///
    /// The package version formatted as a string
    #[must_use]
    pub fn version_str(&self) -> String {
        self.version.borrow().to_string()
    }

    /// Update the package version
    ///
    /// # Arguments
    ///
    /// * `new_version` - The new version string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `VersionError` if the version is invalid
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The new version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("my-package", "1.0.0", None)?;
    /// pkg.update_version("2.0.0")?;
    ///
    /// assert_eq!(pkg.version_str(), "2.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError> {
        let parsed_version = new_version.parse()?;
        *self.version.borrow_mut() = parsed_version;
        Ok(())
    }

    /// Get the package dependencies
    ///
    /// # Returns
    ///
    /// A slice containing the package's dependencies
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// Update a dependency version
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The name of the dependency to update
    /// * `new_version` - The new version string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `DependencyResolutionError` if the dependency is not found
    /// or the version is invalid
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The dependency with the specified name is not found in the package
    /// - The new version string is not a valid semantic version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create dependency
    /// let dep = Dependency::new("react", "^16.0.0")?;
    ///
    /// // Create package with the dependency
    /// let mut pkg = Package::new("my-app", "1.0.0", Some(vec![dep]))?;
    ///
    /// // Update the dependency version
    /// pkg.update_dependency_version("react", "^17.0.0")?;
    ///
    /// // Verify the update
    /// assert_eq!(dep.version().to_string(), "^17.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_dependency_version(
        &mut self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError> {
        for dep in &mut self.dependencies {
            if dep.name() == dep_name && dep.update_version(new_version).is_ok() {
                return Ok(());
            }
        }

        Err(DependencyResolutionError::DependencyNotFound {
            name: dep_name.to_string(),
            package: self.name.clone(),
        })
    }

    /// Add a dependency to the package
    ///
    /// # Arguments
    ///
    /// * `dependency` - The dependency to add
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a package
    /// let mut pkg = Package::new("my-app", "1.0.0", None)?;
    ///
    /// // Add a dependency
    /// let dep = Dependency::new("react", "^17.0.0")?;
    /// pkg.add_dependency(dep);
    ///
    /// assert_eq!(pkg.dependencies().len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
    }

    /// Update package dependencies based on resolution result
    ///
    /// Updates all package dependencies according to the resolved versions.
    ///
    /// # Arguments
    ///
    /// * `resolution` - Result of dependency resolution containing resolved versions
    ///
    /// # Returns
    ///
    /// A vector of tuples with (dependency_name, old_version, new_version) for dependencies
    /// that were updated, or a `VersionError` if any version update fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Any version string in the resolution result is not a valid semantic version
    /// - The underlying dependency version updates fail
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Dependency, ResolutionResult};
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create package with dependencies
    /// let dep = Dependency::new("react", "^16.0.0")?;
    /// let mut pkg = Package::new("my-app", "1.0.0", Some(vec![dep]))?;
    ///
    /// // Create resolution result
    /// let mut resolved = HashMap::new();
    /// resolved.insert("react".to_string(), "^17.0.0".to_string());
    ///
    /// let resolution = ResolutionResult {
    ///     resolved_versions: resolved,
    ///     updates_required: vec![]
    /// };
    ///
    /// // Apply resolution
    /// let updated = pkg.update_dependencies_from_resolution(&resolution)?;
    ///
    /// assert_eq!(updated.len(), 1);
    /// assert_eq!(dep.version().to_string(), "^17.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_dependencies_from_resolution(
        &mut self,
        resolution: &ResolutionResult,
    ) -> Result<Vec<(String, String, String)>, VersionError> {
        let mut updated_deps = Vec::new();

        for dep in &mut self.dependencies {
            let name = dep.name().to_string();
            if let Some(resolved_version) = resolution.resolved_versions.get(&name) {
                let current_version = dep.fixed_version()?.to_string();

                // Only update if the versions are different
                if current_version != *resolved_version
                    && !current_version.contains(resolved_version)
                {
                    dep.update_version(resolved_version)?;
                    updated_deps.push((name, current_version, resolved_version.clone()));
                }
            }
        }

        Ok(updated_deps)
    }
}
