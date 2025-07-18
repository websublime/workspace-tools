//! # Package Info Module
//!
//! This module provides functionality for working with package information,
//! including both parsed package data and the raw package.json content.
//!
//! The `Info` structure bridges the structured `Package` representation
//! with the raw JSON content of a package.json file, allowing operations on both
//! and keeping them synchronized. This is particularly useful for tools that need
//! to read, modify, and write package.json files while preserving formatting
//! and non-standard fields.

use crate::{
    errors::{DependencyResolutionError, PackageError, VersionError},
    Package, ResolutionResult,
};
use serde_json::Value;
use std::{cell::RefCell, rc::Rc};

/// Represents a package along with its JSON data and file paths.
///
/// This structure holds both a structured representation of a package
/// and its raw package.json content, allowing operations to maintain
/// consistency between the two. It also tracks file paths for reading
/// and writing package.json files.
///
/// # Examples
///
/// ```no_run
/// use sublime_package_tools::{Package, Info, Dependency};
/// use serde_json::{json, Value};
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a package
/// let pkg = Package::new(
///     "my-app",
///     "1.0.0",
///     Some(vec![
///         Rc::new(RefCell::new(Dependency::new("react", "^17.0.0")?))
///     ])
/// )?;
///
/// // Create package.json content
/// let pkg_json = json!({
///     "name": "my-app",
///     "version": "1.0.0",
///     "dependencies": {
///         "react": "^17.0.0"
///     }
/// });
///
/// // Create Info
/// let pkg_info = Info::new(
///     pkg,
///     "/path/to/package.json",
///     "/path/to/package",
///     "relative/path/to/package",
///     pkg_json
/// );
///
/// // Update version in both package and JSON
/// pkg_info.update_version("1.1.0")?;
///
/// // Write changes back to disk
/// pkg_info.write_package_json()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Info {
    /// The parsed package structure
    pub package: Rc<RefCell<Package>>,
    /// Absolute path to the package.json file
    pub package_json_path: String,
    /// Absolute path to the package directory
    pub package_path: String,
    /// Relative path to the package directory from the workspace root
    pub package_relative_path: String,
    /// Raw package.json content as JSON
    pub pkg_json: Rc<RefCell<Value>>,
}

impl Info {
    /// Create a new package info.
    ///
    /// # Arguments
    ///
    /// * `package` - The parsed package structure
    /// * `package_json_path` - Absolute path to the package.json file
    /// * `package_path` - Absolute path to the package directory
    /// * `package_relative_path` - Relative path to the package directory from the workspace root
    /// * `pkg_json` - Raw package.json content as JSON
    ///
    /// # Returns
    ///
    /// A new `Info` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Package, Info};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a package
    /// let pkg = Package::new("test-pkg", "1.0.0", None)?;
    ///
    /// // Create a Info
    /// let pkg_info = Info::new(
    ///     pkg,
    ///     "/path/to/package.json",
    ///     "/path/to/package",
    ///     "packages/test-pkg",
    ///     json!({
    ///         "name": "test-pkg",
    ///         "version": "1.0.0"
    ///     })
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        package: Package,
        package_json_path: String,
        package_path: String,
        package_relative_path: String,
        pkg_json: Value,
    ) -> Self {
        Self {
            package: Rc::new(RefCell::new(package)),
            package_json_path,
            package_path,
            package_relative_path,
            pkg_json: Rc::new(RefCell::new(pkg_json)),
        }
    }

    /// Update the package version.
    ///
    /// Updates both the structured package object and the raw package.json content
    /// with the new version.
    ///
    /// # Arguments
    ///
    /// * `new_version` - The new version string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `VersionError` if the version is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Package, Info};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pkg = Package::new("test-pkg", "1.0.0", None)?;
    /// let pkg_info = Info::new(
    ///     pkg,
    ///     "package.json".to_string(),
    ///     ".".to_string(),
    ///     ".".to_string(),
    ///     json!({"name": "test-pkg", "version": "1.0.0"})
    /// );
    ///
    /// // Update version
    /// pkg_info.update_version("2.0.0")?;
    ///
    /// // Verify both are updated
    /// assert_eq!(pkg_info.package.borrow().version_str(), "2.0.0");
    /// assert_eq!(pkg_info.pkg_json.borrow()["version"], "2.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError> {
        // Update Package version
        self.package.borrow().update_version(new_version)?;

        // Update JSON
        if let Some(obj) = self.pkg_json.borrow_mut().as_object_mut() {
            obj.insert("version".to_string(), Value::String(new_version.to_string()));
        }

        Ok(())
    }

    /// Update a dependency version
    ///
    /// This method updates the version of a dependency in both the package object and
    /// the package.json object. It works for both regular dependencies and devDependencies.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The name of the dependency to update
    /// * `new_version` - The new version string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `DependencyResolutionError` if the dependency is not found
    /// or the version is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Package, Info};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pkg = Package::new("test-pkg", "1.0.0", None)?;
    /// # let pkg_info = Info::new(
    /// #     pkg,
    /// #     "package.json".to_string(),
    /// #     ".".to_string(),
    /// #     ".".to_string(),
    /// #     json!({"name": "test-pkg", "version": "1.0.0", "dependencies": {"react": "^16.0.0"}})
    /// # );
    ///
    /// // Update dependency version
    /// pkg_info.update_dependency_version("react", "^17.0.0")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError> {
        // First, modify the package dependency separately from JSON
        {
            let update_result =
                self.package.borrow_mut().update_dependency_version(dep_name, new_version);
            if let Err(DependencyResolutionError::DependencyNotFound { .. }) = update_result {
                // If not found in regular dependencies, that's ok - it might be in devDependencies only
            } else if let Err(e) = update_result {
                // For any other error, return it
                return Err(e);
            }
        } // Package borrow is dropped here

        // Now update the JSON, after the package borrow is dropped
        let mut json_updated = false;

        if let Some(obj) = self.pkg_json.borrow_mut().as_object_mut() {
            // Try updating in dependencies
            if let Some(deps) = obj.get_mut("dependencies").and_then(|v| v.as_object_mut()) {
                if deps.contains_key(dep_name) {
                    deps.insert(dep_name.to_string(), Value::String(new_version.to_string()));
                    json_updated = true;
                }
            }

            // Also try in devDependencies
            if let Some(dev_deps) = obj.get_mut("devDependencies").and_then(|v| v.as_object_mut()) {
                if dev_deps.contains_key(dep_name) {
                    dev_deps.insert(dep_name.to_string(), Value::String(new_version.to_string()));
                    json_updated = true;
                }
            }
        }

        // If we didn't update JSON but also didn't find it in package, it's a genuine "not found"
        if !json_updated
            && self.package.borrow_mut().update_dependency_version(dep_name, new_version).is_err()
        {
            return Err(DependencyResolutionError::DependencyNotFound {
                name: dep_name.to_string(),
                package: self.package.borrow().name().to_string(),
            });
        }

        Ok(())
    }

    /// Apply dependency resolution across all packages
    ///
    /// This method applies a resolution result to update all dependencies in the package,
    /// modifying both the Package object and package.json content for dependencies and devDependencies.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The resolution result containing the resolved versions
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `VersionError` if any version update fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Package, Info, ResolutionResult};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pkg = Package::new("test-pkg", "1.0.0", None)?;
    /// # let pkg_info = Info::new(
    /// #     pkg,
    /// #     "package.json".to_string(),
    /// #     ".".to_string(),
    /// #     ".".to_string(),
    /// #     json!({"name": "test-pkg", "version": "1.0.0", "dependencies": {"react": "^16.0.0"}})
    /// # );
    ///
    /// // Create resolution result
    /// let mut resolved_versions = HashMap::new();
    /// resolved_versions.insert("react".to_string(), "^17.0.0".to_string());
    ///
    /// let resolution = ResolutionResult {
    ///     resolved_versions,
    ///     updates_required: vec![]
    /// };
    ///
    /// // Apply resolution
    /// pkg_info.apply_dependency_resolution(&resolution)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_dependency_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<(), VersionError> {
        // First, update the package's dependencies (handles regular dependencies)
        let _ = { self.package.borrow_mut().update_dependencies_from_resolution(resolution)? }; // Package borrow is dropped here

        // Now update package.json for both dependencies and devDependencies
        if let Some(pkg_json_obj) = self.pkg_json.borrow_mut().as_object_mut() {
            // Update all dependencies in the resolved versions map
            for (dep_name, new_version) in &resolution.resolved_versions {
                // Check and update in dependencies section
                if let Some(deps) =
                    pkg_json_obj.get_mut("dependencies").and_then(Value::as_object_mut)
                {
                    if deps.contains_key(dep_name) {
                        deps.insert(dep_name.clone(), Value::String(new_version.clone()));
                    }
                }

                // Also check and update in devDependencies section
                if let Some(dev_deps) =
                    pkg_json_obj.get_mut("devDependencies").and_then(Value::as_object_mut)
                {
                    if dev_deps.contains_key(dep_name) {
                        dev_deps.insert(dep_name.clone(), Value::String(new_version.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Write the package.json file to disk
    ///
    /// This method writes the current state of the package.json object back to disk,
    /// persisting any changes made to the package.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `PackageError` if writing fails
    ///
    /// # Errors
    ///
    /// Can return `PackageError::PackageJsonParseFailure` if JSON serialization fails or
    /// `PackageError::PackageJsonIoFailure` if writing to disk fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_package_tools::{Package, Info};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let pkg = Package::new("test-pkg", "1.0.0", None)?;
    /// # let pkg_info = Info::new(
    /// #     pkg,
    /// #     "package.json".to_string(),
    /// #     ".".to_string(),
    /// #     ".".to_string(),
    /// #     json!({"name": "test-pkg", "version": "1.0.0"})
    /// # );
    ///
    /// // Make changes
    /// pkg_info.update_version("1.1.0")?;
    ///
    /// // Write changes to disk
    /// pkg_info.write_package_json()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_package_json(&self) -> Result<(), PackageError> {
        let json_content = serde_json::to_string_pretty(&*self.pkg_json.borrow())
            .map_err(|e| PackageError::into_parse_error(e, self.package_json_path.clone()))?;

        std::fs::write(&self.package_json_path, json_content)
            .map_err(|e| PackageError::into_io_error(e, self.package_json_path.clone()))?;

        Ok(())
    }
}

