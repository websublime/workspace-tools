//! # Package Module
//!
//! This module provides the `Package` structure for representing Node.js packages.
//!
//! A `Package` represents a Node.js package with name, version, and dependencies.
//! It's now a pure data structure - all business logic has been moved to `PackageService`.
//!
//! ## Key Features
//!
//! - **Pure Data Structure**: No business logic, only data and simple accessors
//! - **Public Fields**: Direct access to package data for performance
//! - **Node Trait**: Implements dependency graph operations
//! - **Simple Constructors**: Easy creation with validation
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{Package, PackageService};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a package (pure data)
//! let mut pkg = Package::new("my-app", "1.0.0", None)?;
//!
//! // Access package information directly
//! println!("Package: {} v{}", pkg.name, pkg.version);
//!
//! // Use PackageService for business operations
//! # let fs = ();
//! let service = PackageService::new(fs);
//! # Ok(())
//! # }
//! ```

use crate::{
    errors::VersionError,
    Dependency, Node,
};

/// A package with its dependencies and version information
///
/// This is now a pure data structure representing an NPM package. All business logic
/// has been extracted to `PackageService` to maintain clean separation of concerns.
///
/// ## Fields
///
/// All fields are public for direct access and performance:
/// - `name`: Package name
/// - `version`: Package version string  
/// - `dependencies`: List of package dependencies
///
/// ## Examples
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
///
/// // Direct field access
/// println!("Package {} has {} dependencies", pkg_with_deps.name, pkg_with_deps.dependencies.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Package version string (semver)
    pub version: String,
    /// List of package dependencies
    pub dependencies: Vec<Dependency>,
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
        self.name() == dependency.name() && dependency.version().matches(&semver::Version::parse(&self.version).unwrap_or_else(|_| semver::Version::new(0, 0, 0)))
    }

    fn identifier(&self) -> Self::Identifier {
        self.name().to_string()
    }
}

impl Package {
    /// Create a new package with name, version, and optional dependencies
    ///
    /// This constructor validates the version string but performs no other business logic.
    /// For complex package operations, use `PackageService`.
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
    /// assert_eq!(pkg.name, "my-package");
    /// assert_eq!(pkg.version, "1.0.0");
    /// assert_eq!(pkg.dependencies.len(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Dependency>>,
    ) -> Result<Self, VersionError> {
        // Validate version string by parsing it
        let _ = semver::Version::parse(version)?;

        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
            dependencies: dependencies.unwrap_or_default(),
        })
    }

    /// Create a new package using the dependency registry
    ///
    /// This constructor creates dependencies using a registry to ensure consistent
    /// dependency instances across packages.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name
    /// * `version` - The package version as a semver string
    /// * `deps` - Optional list of (name, version) tuples for dependencies
    /// * `registry` - Mutable reference to dependency registry
    ///
    /// # Returns
    ///
    /// A new `Package` instance with dependencies from the registry
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The version string is not a valid semantic version
    /// - Any dependency version is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Registry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// let pkg = Package::new_with_registry(
    ///     "my-app",
    ///     "1.0.0",
    ///     Some(vec![("react", "^17.0.2"), ("lodash", "^4.17.21")]),
    ///     &mut registry
    /// )?;
    /// assert_eq!(pkg.dependencies.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_registry(
        name: &str,
        version: &str,
        deps: Option<Vec<(&str, &str)>>,
        registry: &mut crate::Registry,
    ) -> Result<Self, VersionError> {
        let dependencies = if let Some(dep_specs) = deps {
            let mut deps = Vec::new();
            for (dep_name, dep_version) in dep_specs {
                let dep = registry.get_or_create(dep_name, dep_version)?;
                deps.push(dep);
            }
            deps
        } else {
            Vec::new()
        };

        Self::new(name, version, Some(dependencies))
    }

    /// Get the package name
    ///
    /// # Returns
    ///
    /// A reference to the package name string
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("my-package", "1.0.0", None)?;
    /// assert_eq!(pkg.name(), "my-package");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package version
    ///
    /// # Returns
    ///
    /// A reference to the package version string
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("my-package", "1.0.0", None)?;
    /// assert_eq!(pkg.version(), "1.0.0");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the package version as an owned string
    ///
    /// # Returns
    ///
    /// The package version as an owned string
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Package;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("my-package", "1.0.0", None)?;
    /// let version_string = pkg.version_str();
    /// assert_eq!(version_string, "1.0.0");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn version_str(&self) -> String {
        self.version.clone()
    }

    /// Get the package dependencies
    ///
    /// # Returns
    ///
    /// A slice containing the package's dependencies
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, Dependency};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("react", "^17.0.2")?;
    /// let pkg = Package::new("my-app", "1.0.0", Some(vec![dep]))?;
    /// assert_eq!(pkg.dependencies().len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}