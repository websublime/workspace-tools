//! # Dependency Module
//!
//! This module provides functionality for working with Node.js package dependencies.
//!
//! The main structure is `Dependency`, which represents a package dependency with
//! semantic versioning requirements. Dependencies are often used with Rc<RefCell<>> to
//! allow sharing and mutation across package structures.
//!
//! ## Key Features
//!
//! - Parse and validate version requirements
//! - Version compatibility checking
//! - Version comparison and update operations
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::Dependency;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new dependency
//! let dep = Dependency::new("react", "^17.0.2")?;
//!
//! // Check if a specific version satisfies requirements
//! assert!(dep.matches("17.0.5")?);
//! assert!(!dep.matches("18.0.0")?);
//!
//! // Get the fixed version (without ^ or ~ operators)
//! let fixed = dep.fixed_version()?;
//! assert_eq!(fixed.to_string(), "17.0.2");
//!
//! // Update version
//! dep.update_version("^18.0.0")?;
//! # Ok(())
//! # }
//! ```

use crate::VersionError;
use semver::{Version, VersionReq};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

/// Represents a package dependency with name and version requirements.
///
/// A dependency consists of:
/// - A name identifier (e.g., "react", "lodash")
/// - A version requirement (e.g., "^17.0.2", "~4.17.21")
///
/// The version is stored in a `Rc<RefCell<>>` to allow shared references
/// and updates when resolving dependency conflicts.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::Dependency;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let dep = Dependency::new("express", "^4.18.1")?;
/// println!("Dependency: {}", dep); // Formats as "express@^4.18.1"
///
/// // Check compatibility
/// if dep.matches("4.18.2")? {
///     println!("Version 4.18.2 is compatible!");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Dependency {
    pub(crate) name: String,
    pub(crate) version: Rc<RefCell<VersionReq>>,
}

impl Display for Dependency {
    /// Formats a dependency as "name@version"
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("lodash", "^4.17.21")?;
    /// assert_eq!(dep.to_string(), "lodash@^4.17.21");
    /// # Ok(())
    /// # }
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}@{}", self.name, self.version.borrow())
    }
}

impl Dependency {
    /// Creates a new dependency with the given name and version requirements.
    ///
    /// # Arguments
    ///
    /// * `name` - The package name (e.g., "react", "lodash")
    /// * `version` - The version requirement (e.g., "^17.0.2", "~4.17.21")
    ///
    /// # Returns
    ///
    /// A new `Dependency` instance or an error if the version format is invalid.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidVersion` if:
    /// - The version string contains `*` or `workspace:*` (internal dependency markers)
    /// - The version string cannot be parsed as a valid semver requirement
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create valid dependencies
    /// let dep1 = Dependency::new("react", "^17.0.2")?;
    /// let dep2 = Dependency::new("express", "~4.18.1")?;
    /// let dep3 = Dependency::new("lodash", ">=4.0.0 <5.0.0")?;
    ///
    /// // Invalid version (internal dependency marker)
    /// let invalid = Dependency::new("internal-pkg", "workspace:*");
    /// assert!(invalid.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: &str, version: &str) -> Result<Self, VersionError> {
        if version.starts_with('*') | version.contains("workspace:*") {
            return Err(VersionError::InvalidVersion(format!(
                "Looks like you are trying to update a internal package: {version}"
            )));
        };

        let version_req = VersionReq::parse(version).map_err(VersionError::from)?;
        Ok(Self { name: name.to_string(), version: Rc::new(RefCell::new(version_req)) })
    }

    /// Returns the name of the dependency.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("express", "^4.18.1")?;
    /// assert_eq!(dep.name(), "express");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the version requirement of the dependency.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    /// use semver::VersionReq;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("react", "^17.0.2")?;
    /// let req: VersionReq = dep.version();
    /// assert_eq!(req.to_string(), "^17.0.2");
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> VersionReq {
        self.version.borrow().clone()
    }

    /// Extracts the fixed version from the version requirement.
    ///
    /// This removes caret (^) or tilde (~) operators from the version,
    /// returning just the base version.
    ///
    /// # Returns
    ///
    /// The parsed `Version` without operators, or an error if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep1 = Dependency::new("react", "^17.0.2")?;
    /// assert_eq!(dep1.fixed_version()?.to_string(), "17.0.2");
    ///
    /// let dep2 = Dependency::new("express", "~4.18.1")?;
    /// assert_eq!(dep2.fixed_version()?.to_string(), "4.18.1");
    /// # Ok(())
    /// # }
    /// ```
    pub fn fixed_version(&self) -> Result<Version, VersionError> {
        let req_str = self.version.borrow().to_string();
        // Remove operators and parse
        let clean_version = req_str.trim_start_matches(|c| "^~=".contains(c)).trim();

        Version::parse(clean_version).map_err(VersionError::from)
    }

    /// Compares the dependency's version with another version string.
    ///
    /// # Arguments
    ///
    /// * `other` - The version string to compare with
    ///
    /// # Returns
    ///
    /// A comparison result:
    /// - `Ordering::Less` - The dependency's version is less than `other`
    /// - `Ordering::Equal` - The versions are equal
    /// - `Ordering::Greater` - The dependency's version is greater than `other`
    ///
    /// # Errors
    ///
    /// Returns a `VersionError` if either version can't be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    /// use std::cmp::Ordering;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("react", "^16.0.0")?;
    ///
    /// assert_eq!(dep.compare_versions("17.0.0")?, Ordering::Less);
    /// assert_eq!(dep.compare_versions("16.0.0")?, Ordering::Equal);
    /// assert_eq!(dep.compare_versions("15.0.0")?, Ordering::Greater);
    /// # Ok(())
    /// # }
    /// ```
    pub fn compare_versions(&self, other: &str) -> Result<Ordering, VersionError> {
        let self_version = self.fixed_version()?;
        let other_version = Version::parse(other)?;

        Ok(self_version.cmp(&other_version))
    }

    /// Updates the version requirement to a new value.
    ///
    /// # Arguments
    ///
    /// * `new_version` - The new version requirement string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if the new version is invalid.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidVersion` if:
    /// - The version string contains `*` or `workspace:*` (internal dependency markers)
    /// - The version string cannot be parsed as a valid semver requirement
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("react", "^16.0.0")?;
    ///
    /// // Update to a newer version
    /// dep.update_version("^17.0.0")?;
    /// assert_eq!(dep.version().to_string(), "^17.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError> {
        if new_version.starts_with('*') | new_version.contains("workspace:*") {
            return Err(VersionError::InvalidVersion(format!(
                "Looks like you are trying to update a internal package: {new_version}"
            )));
        };

        let new_req = VersionReq::parse(new_version).map_err(VersionError::from)?;
        *self.version.borrow_mut() = new_req;
        Ok(())
    }

    /// Checks if a specific version matches this dependency's requirements.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to check against this dependency's requirements
    ///
    /// # Returns
    ///
    /// `true` if the version satisfies the requirements, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns a `VersionError` if the version string can't be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::Dependency;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dep = Dependency::new("react", "^16.0.0")?;
    ///
    /// assert!(dep.matches("16.0.0")?);   // Exact match
    /// assert!(dep.matches("16.8.0")?);   // Compatible with ^16.0.0
    /// assert!(!dep.matches("17.0.0")?);  // Not compatible with ^16.0.0
    /// # Ok(())
    /// # }
    /// ```
    pub fn matches(&self, version: &str) -> Result<bool, VersionError> {
        let version = Version::parse(version).map_err(VersionError::from)?;

        Ok(self.version.borrow().matches(&version))
    }
}
