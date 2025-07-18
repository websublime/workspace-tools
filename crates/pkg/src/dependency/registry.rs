//! # Dependency Registry Module
//!
//! This module provides a registry for managing and reusing dependency instances.
//!
//! ## Overview
//!
//! The `Registry` is a central repository for managing dependencies, ensuring:
//! - Dependencies with the same name are consistently represented
//! - Version conflicts between dependencies can be detected and resolved
//! - Memory usage is optimized by reusing dependency instances
//!
//! ## Usage
//!
//! The registry is particularly useful when building packages that might share dependencies,
//! as it ensures that references to the same dependency are consistent across the system.
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{Dependency, Registry, Package};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a registry
//! let mut registry = Registry::new();
//!
//! // Get dependencies from the registry
//! let react_dep = registry.get_or_create("react", "^17.0.2")?;
//! let router_dep = registry.get_or_create("react-router", "^6.0.0")?;
//!
//! // Create a package using these dependencies
//! let pkg = Package::new_with_registry(
//!     "my-app",
//!     "1.0.0",
//!     Some(vec![("react", "^17.0.2"), ("react-router", "^6.0.0")]),
//!     &mut registry
//! )?;
//!
//! // The dependencies in the package will be the same instances from the registry
//! # Ok(())
//! # }
//! ```

use super::{resolution::ResolutionResult, update::Update as DependencyUpdate};
use crate::{
    errors::{PackageRegistryError, VersionError},
    package::registry::PackageRegistryClone,
    Dependency, Version,
};
use semver::{Version as SemverVersion, VersionReq};
use std::collections::HashMap;

/// A registry for managing and reusing dependency instances.
///
/// The `Registry` maintains a collection of dependencies, ensuring that
/// the same dependency (by name) is consistently represented throughout the system.
/// It also provides functionality for resolving version conflicts between dependencies.
///
/// When a package registry is provided, the registry can query external sources
/// to find the highest compatible versions for dependency resolution.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{Registry, Package};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut registry = Registry::new();
///
/// // When we create packages with the same dependencies
/// let pkg1 = Package::new_with_registry(
///     "pkg1",
///     "1.0.0",
///     Some(vec![("react", "^17.0.0")]),
///     &mut registry
/// )?;
///
/// let pkg2 = Package::new_with_registry(
///     "pkg2",
///     "1.0.0",
///     Some(vec![("react", "^17.0.0")]),
///     &mut registry
/// )?;
///
/// // The registry ensures they share the same dependency instance
/// // This means updates to one will affect the other
/// # Ok(())
/// # }
/// ```
///
/// ## With Package Registry
///
/// ```
/// use sublime_package_tools::{Registry, NpmRegistry};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create registry with npm package registry for enhanced version resolution
/// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
/// let mut dependency_registry = Registry::with_package_registry(
///     Box::new(npm_registry)
/// );
///
/// // Now the registry can query npm for available versions when resolving conflicts
/// let react_dep = dependency_registry.get_or_create("react", "^17.0.0")?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Registry {
    /// Collection of managed dependencies indexed by name
    dependencies: HashMap<String, Dependency>,
    /// Optional package registry for querying external package sources
    package_registry: Option<Box<dyn PackageRegistryClone>>,
}

impl Clone for Box<dyn PackageRegistryClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl std::fmt::Debug for dyn PackageRegistryClone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageRegistry").finish()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Creates a new, empty dependency registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Registry;
    ///
    /// let registry = Registry::new();
    /// ```
    pub fn new() -> Self {
        Self { dependencies: HashMap::new(), package_registry: None }
    }

    /// Creates a new dependency registry with a package registry for enhanced version resolution.
    ///
    /// When a package registry is provided, the dependency registry can query external sources
    /// (like npm) to find the highest compatible versions during dependency resolution.
    ///
    /// # Arguments
    ///
    /// * `package_registry` - A boxed package registry implementation for querying external sources
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_package_registry(package_registry: Box<dyn PackageRegistryClone>) -> Self {
        Self { dependencies: HashMap::new(), package_registry: Some(package_registry) }
    }

    /// Sets the package registry for this dependency registry.
    ///
    /// This allows adding package registry functionality to an existing dependency registry.
    ///
    /// # Arguments
    ///
    /// * `package_registry` - A boxed package registry implementation for querying external sources
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// registry.set_package_registry(Box::new(npm_registry));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_package_registry(&mut self, package_registry: Box<dyn PackageRegistryClone>) {
        self.package_registry = Some(package_registry);
    }

    /// Gets an existing dependency or creates a new one.
    ///
    /// If the dependency with the given name already exists in the registry,
    /// a clone of the existing dependency is returned. Otherwise, a new dependency
    /// is created and added to the registry.
    ///
    /// If the dependency exists but with a different version requirement,
    /// the higher version will be used and the registry will be updated.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency
    /// * `version` - The version requirement string
    ///
    /// # Returns
    ///
    /// A cloned `Dependency` instance.
    ///
    /// # Errors
    ///
    /// Returns a `VersionError` if the version string is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Registry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // First call creates a new dependency
    /// let dep1 = registry.get_or_create("react", "^17.0.2")?;
    /// assert_eq!(dep1.name(), "react");
    /// assert_eq!(dep1.version().to_string(), "^17.0.2");
    ///
    /// // Second call returns a dependency with the same values
    /// let dep2 = registry.get_or_create("react", "^17.0.2")?;
    /// assert_eq!(dep2.name(), "react");
    /// assert_eq!(dep2.version().to_string(), "^17.0.2");
    ///
    /// // With a higher version, the registry is updated
    /// let dep3 = registry.get_or_create("react", "^18.0.0")?;
    /// assert_eq!(dep3.version().to_string(), "^18.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_or_create(&mut self, name: &str, version: &str) -> Result<Dependency, VersionError> {
        if let Some(existing_dep) = self.dependencies.get_mut(name) {
            let current_version = existing_dep.version().to_string();

            // If the new version requirement is different, update it
            // Note: We might want to keep the higher version when there's a conflict
            if current_version != version {
                // Parse both versions to compare them properly
                let current_clean = current_version.trim_start_matches('^').trim_start_matches('~');
                let new_clean = version.trim_start_matches('^').trim_start_matches('~');

                if let (Ok(curr_ver), Ok(new_ver)) =
                    (semver::Version::parse(current_clean), semver::Version::parse(new_clean))
                {
                    // Update to the higher version
                    if new_ver > curr_ver {
                        existing_dep.update_version(version)?;
                    }
                } else {
                    // If we can't parse, just update to the new version
                    existing_dep.update_version(version)?;
                }
            }

            return Ok(existing_dep.clone());
        }

        let dep = Dependency::new(name, version)?;
        self.dependencies.insert(name.to_string(), dep.clone());
        Ok(dep)
    }

    /// Gets an existing dependency by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency to retrieve
    ///
    /// # Returns
    ///
    /// A reference-counted cell containing the dependency if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Registry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // Add a dependency
    /// registry.get_or_create("react", "^17.0.2")?;
    ///
    /// // Retrieve the dependency
    /// if let Some(dep) = registry.get("react") {
    ///     println!("Found: {}", dep.name());
    /// }
    ///
    /// // Non-existent dependency
    /// assert!(registry.get("missing").is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Dependency> {
        self.dependencies.get(name).cloned()
    }

    /// Resolve version conflicts between dependencies.
    ///
    /// This method analyzes all dependencies in the registry and tries to find
    /// a consistent version that satisfies all requirements for each package.
    /// If conflicts are found, it attempts to resolve them by finding the highest
    /// compatible version.
    ///
    /// # Returns
    ///
    /// A `ResolutionResult` containing resolved versions and required updates,
    /// or an error if resolution fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if version parsing fails during resolution.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Registry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // Add two dependencies with the same name but different versions
    /// registry.get_or_create("lodash", "^4.17.0")?;
    /// registry.get_or_create("lodash", "^4.17.21")?;
    ///
    /// // Resolve conflicts
    /// let result = registry.resolve_version_conflicts()?;
    ///
    /// // Check resolved versions
    /// if let Some(version) = result.resolved_versions.get("lodash") {
    ///     println!("Resolved lodash to: {}", version);
    /// }
    ///
    /// // Check updates needed
    /// for update in result.updates_required {
    ///     println!("Update required: {} from {} to {}",
    ///              update.dependency_name,
    ///              update.current_version,
    ///              update.new_version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::uninlined_format_args)]
    #[allow(clippy::inefficient_to_string)]
    pub fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError> {
        let mut resolved_versions: HashMap<String, String> = HashMap::new();
        let mut updates_required: Vec<DependencyUpdate> = Vec::new();

        // Group all dependencies by name
        let mut dependency_requirements: HashMap<String, Vec<(String, VersionReq)>> =
            HashMap::new();

        // Collect all version requirements for each dependency
        for (name, dep) in &self.dependencies {
            let version_req = dep.version();
            dependency_requirements
                .entry(name.clone())
                .or_default()
                .push((dep.fixed_version()?.to_string(), version_req.clone()));
        }

        // For each dependency, find the highest available version that satisfies all requirements
        for (name, requirements) in &dependency_requirements {
            // For test purposes, extract the underlying version numbers
            let mut versions = Vec::new();
            for (ver_str, _) in requirements {
                // Clean up version string
                let clean_ver = ver_str.trim_start_matches('^').trim_start_matches('~');

                // Parse into semver::Version for proper comparison
                if let Ok(ver) = Version::parse(clean_ver) {
                    versions.push((clean_ver, ver));
                }
            }

            // Sort versions by the actual parsed Version objects
            versions.sort_by(|(_, a), (_, b)| a.cmp(b));

            // Take the highest version (last after sorting)
            if let Some((highest_str, _)) = versions.last() {
                resolved_versions.insert(name.clone(), highest_str.to_string());

                // Check if updates are required
                for (version_str, _) in requirements {
                    let clean_version = version_str.trim_start_matches('^').trim_start_matches('~');
                    if clean_version != *highest_str {
                        updates_required.push(DependencyUpdate {
                            package_name: String::new(), // Can't know without more context
                            dependency_name: name.clone(),
                            current_version: version_str.clone(),
                            new_version: highest_str.to_string(),
                        });
                    }
                }
            }
        }

        Ok(ResolutionResult { resolved_versions, updates_required })
    }

    /// Get all versions of a package from the package registry.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package to query
    ///
    /// # Returns
    ///
    /// A list of available versions or an error if the query fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    ///
    /// if let Ok(versions) = registry.get_package_versions("react") {
    ///     println!("Found {} versions of react", versions.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_package_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        if let Some(ref registry) = self.package_registry {
            registry.get_all_versions(package_name)
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if the registry has package registry capabilities.
    ///
    /// # Returns
    ///
    /// `true` if a package registry is configured, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// assert!(!registry.has_package_registry());
    ///
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// registry.set_package_registry(Box::new(npm_registry));
    /// assert!(registry.has_package_registry());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn has_package_registry(&self) -> bool {
        self.package_registry.is_some()
    }

    /// Find highest version that is compatible with all requirements.
    ///
    /// This method first checks existing dependencies in the registry, then falls back
    /// to querying the package registry (if available) to find all available versions
    /// and select the highest one that satisfies all requirements.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency
    /// * `requirements` - A list of version requirements to satisfy
    ///
    /// # Returns
    ///
    /// A string representing the highest version that satisfies all requirements.
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if querying the package registry fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, Dependency};
    /// use semver::VersionReq;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// registry.get_or_create("react", "^17.0.0")?;
    ///
    /// // Find a version compatible with both requirements
    /// let req1 = VersionReq::parse("^17.0.0")?;
    /// let req2 = VersionReq::parse("^17.0.2")?;
    /// let version = registry.find_highest_compatible_version(
    ///     "react",
    ///     &[&req1, &req2]
    /// )?;
    ///
    /// // The method will return the highest compatible version
    /// assert!(!version.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With Package Registry
    ///
    /// ```no_run
    /// use sublime_package_tools::{Registry, NpmRegistry};
    /// use semver::VersionReq;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create registry with npm for enhanced resolution
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    ///
    /// let req = VersionReq::parse("^17.0.0")?;
    ///
    /// // This will query npm to find the highest version matching the requirement
    /// let version = registry.find_highest_compatible_version("react", &[&req])?;
    /// println!("Found compatible version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unnecessary_wraps)]
    pub fn find_highest_compatible_version(
        &self,
        name: &str,
        requirements: &[&VersionReq],
    ) -> Result<String, PackageRegistryError> {
        // First priority: Query package registry for all available versions
        match self.get_package_versions(name) {
            Ok(available_versions) => {
                if !available_versions.is_empty() {
                    // Parse and filter versions that satisfy all requirements
                    let mut compatible_versions: Vec<SemverVersion> = available_versions
                        .iter()
                        .filter_map(|version_str| {
                            // Clean version string (remove ^ or ~ prefixes if present)
                            let clean_version =
                                version_str.trim_start_matches('^').trim_start_matches('~');

                            SemverVersion::parse(clean_version).ok()
                        })
                        .filter(|version| {
                            // Check if this version satisfies all requirements
                            requirements.iter().all(|req| req.matches(version))
                        })
                        .collect();

                    // Sort versions in ascending order, then take the highest (last)
                    compatible_versions.sort();

                    if let Some(highest_version) = compatible_versions.last() {
                        return Ok(highest_version.to_string());
                    }
                }
            }
            Err(registry_error) => {
                // Log the registry error but continue to fallback
                // In a production system, you might want to log this error
                eprintln!("Warning: Failed to query package registry for {name}: {registry_error}");
            }
        }

        // Fallback: Check existing dependencies in the registry
        if let Some(dep) = self.dependencies.get(name) {
            let version_str = dep.version().to_string();

            // Handle ^ or ~ prefix
            let clean_version = version_str.trim_start_matches('^').trim_start_matches('~');

            if let Ok(version) = SemverVersion::parse(clean_version) {
                // Check if this version satisfies all requirements
                if requirements.iter().all(|req| req.matches(&version)) {
                    return Ok(clean_version.to_string());
                }
            }
        }

        // Final fallback: return base version
        Ok("0.0.0".to_string())
    }

    /// Apply the resolution result to update all dependencies.
    ///
    /// This method updates the version requirements for all dependencies
    /// according to the resolved versions in the provided resolution result.
    ///
    /// # Arguments
    ///
    /// * `result` - The resolution result containing resolved versions and updates
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if updating any version fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if updating any version fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, Update, ResolutionResult};
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// registry.get_or_create("react", "^16.0.0")?;
    ///
    /// // Create a resolution result
    /// let mut resolved_versions = HashMap::new();
    /// resolved_versions.insert("react".to_string(), "17.0.0".to_string());
    ///
    /// let updates = vec![
    ///     Update {
    ///         package_name: "test-app".to_string(),
    ///         dependency_name: "react".to_string(),
    ///         current_version: "^16.0.0".to_string(),
    ///         new_version: "^17.0.0".to_string(),
    ///     }
    /// ];
    ///
    /// let result = ResolutionResult {
    ///     resolved_versions,
    ///     updates_required: updates,
    /// };
    ///
    /// // Apply updates
    /// registry.apply_resolution_result(&result)?;
    ///
    /// // Verify updates
    /// let dep = registry.get("react").unwrap();
    /// assert_eq!(dep.version().to_string(), "^17.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_resolution_result(
        &mut self,
        result: &ResolutionResult,
    ) -> Result<(), VersionError> {
        for update in &result.updates_required {
            if let Some(dep) = self.dependencies.get_mut(&update.dependency_name) {
                dep.update_version(&update.new_version)?;
            }
        }
        Ok(())
    }
}

