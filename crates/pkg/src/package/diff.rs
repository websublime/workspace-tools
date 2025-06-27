//! # Package Diff Module
//!
//! This module provides functionality for comparing different versions of packages.
//!
//! When working with package updates or version changes, it's important to understand
//! what has changed between versions. The `PackageDiff` struct captures differences
//! between package versions, including version changes and dependency modifications.
//!
//! This is particularly useful for:
//! - Generating changelogs
//! - Determining the impact of package updates
//! - Identifying breaking changes
//! - Tracking dependency evolution

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::{ChangeType, DependencyChange, Package, errors::PackageError};
use semver::Version;
use serde::{Deserialize, Serialize};

/// Represents the differences between two versions of a package.
///
/// This structure captures the changes between two package versions,
/// including version changes and all dependency modifications (additions,
/// removals, and updates).
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{Package, PackageDiff, Dependency};
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create two package versions for comparison
/// let old_pkg = Package::new(
///     "my-app",
///     "1.0.0",
///     Some(vec![
///         Rc::new(RefCell::new(Dependency::new("react", "^16.0.0")?)),
///         Rc::new(RefCell::new(Dependency::new("express", "^4.16.0")?)),
///     ])
/// )?;
///
/// let new_pkg = Package::new(
///     "my-app",
///     "2.0.0",
///     Some(vec![
///         Rc::new(RefCell::new(Dependency::new("react", "^17.0.0")?)),
///         Rc::new(RefCell::new(Dependency::new("lodash", "^4.17.21")?)), // Added
///         // express removed
///     ])
/// )?;
///
/// // Generate diff
/// let diff = PackageDiff::between(&old_pkg, &new_pkg)?;
///
/// // Use diff information
/// println!("Package {} updated from {} to {}",
///          diff.package_name, diff.previous_version, diff.current_version);
///
/// if diff.breaking_change {
///     println!("Warning: Breaking change detected!");
/// }
///
/// // Display dependency changes
/// println!("{}", diff);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDiff {
    /// Name of the package
    pub package_name: String,
    /// Version before the change
    pub previous_version: String,
    /// Version after the change
    pub current_version: String,
    /// Changes to the dependencies
    pub dependency_changes: Vec<DependencyChange>,
    /// Whether the package version change is breaking (major version bump)
    pub breaking_change: bool,
}

impl fmt::Display for PackageDiff {
    /// Formats the package diff as a human-readable string.
    ///
    /// Includes package name, version changes, breaking change warning,
    /// and a list of all dependency modifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, PackageDiff};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let old_pkg = Package::new("pkg", "1.0.0", None)?;
    /// # let new_pkg = Package::new("pkg", "2.0.0", None)?;
    /// let diff = PackageDiff::between(&old_pkg, &new_pkg)?;
    /// println!("{}", diff);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Output format:
    /// ```text
    /// Package: pkg (1.0.0→2.0.0)
    /// ⚠️  Breaking change: Major version bump
    /// No dependency changes
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Package: {} ({}→{})",
            self.package_name, self.previous_version, self.current_version
        )?;

        if self.breaking_change {
            writeln!(f, "⚠️  Breaking change: Major version bump")?;
        }

        if self.dependency_changes.is_empty() {
            writeln!(f, "No dependency changes")?;
        } else {
            writeln!(f, "Dependency changes:")?;

            for change in &self.dependency_changes {
                let symbol = match change.change_type {
                    ChangeType::Added => "+",
                    ChangeType::Removed => "-",
                    ChangeType::Updated => "↑",
                    ChangeType::Unchanged => " ",
                };

                match change.change_type {
                    ChangeType::Added => {
                        writeln!(
                            f,
                            "  {} {} added ({})",
                            symbol,
                            change.name,
                            change.current_version.as_deref().unwrap_or("unknown")
                        )?;
                    }
                    ChangeType::Removed => {
                        writeln!(
                            f,
                            "  {} {} removed (was {})",
                            symbol,
                            change.name,
                            change.previous_version.as_deref().unwrap_or("unknown")
                        )?;
                    }
                    ChangeType::Updated => {
                        writeln!(
                            f,
                            "  {} {} updated: {} → {}{}",
                            symbol,
                            change.name,
                            change.previous_version.as_deref().unwrap_or("unknown"),
                            change.current_version.as_deref().unwrap_or("unknown"),
                            if change.breaking { " ⚠️" } else { "" }
                        )?;
                    }
                    ChangeType::Unchanged => {}
                }
            }
        }

        Ok(())
    }
}

impl PackageDiff {
    /// Generate a diff between two packages.
    ///
    /// Compares two packages and identifies all differences between them,
    /// including version changes and dependency modifications.
    ///
    /// # Arguments
    ///
    /// * `previous` - The previous version of the package
    /// * `current` - The current version of the package
    ///
    /// # Returns
    ///
    /// A `PackageDiff` containing all identified differences between the packages.
    ///
    /// # Errors
    ///
    /// Returns `PackageError::PackageBetweenFailure` if the packages have different names,
    /// as diffs can only be generated for different versions of the same package.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, PackageDiff, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create two package versions
    /// let old_pkg = Package::new(
    ///     "my-lib",
    ///     "0.1.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("dep1", "^1.0.0")?)),
    ///     ])
    /// )?;
    ///
    /// let new_pkg = Package::new(
    ///     "my-lib",
    ///     "0.2.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("dep1", "^1.5.0")?)),
    ///         Rc::new(RefCell::new(Dependency::new("dep2", "^2.0.0")?)),
    ///     ])
    /// )?;
    ///
    /// // Generate diff
    /// let diff = PackageDiff::between(&old_pkg, &new_pkg)?;
    ///
    /// // Check diff contents
    /// assert_eq!(diff.package_name, "my-lib");
    /// assert_eq!(diff.previous_version, "0.1.0");
    /// assert_eq!(diff.current_version, "0.2.0");
    /// assert_eq!(diff.dependency_changes.len(), 2); // 1 updated, 1 added
    /// # Ok(())
    /// # }
    /// ```
    pub fn between(previous: &Package, current: &Package) -> Result<Self, PackageError> {
        if previous.name() != current.name() {
            return Err(PackageError::PackageBetweenFailure(format!(
                "Cannot diff different packages: {} vs {}",
                previous.name(),
                current.name()
            )));
        }

        // Get previous and current versions
        let prev_version = previous.version_str();
        let curr_version = current.version_str();

        // Determine if the package version change is breaking
        let breaking_change = if let (Ok(prev_ver), Ok(curr_ver)) =
            (Version::parse(&prev_version), Version::parse(&curr_version))
        {
            curr_ver.major > prev_ver.major
        } else {
            // If we can't parse the version, conservatively assume it might be breaking
            true
        };

        // Create maps of dependencies for easy comparison
        let mut prev_deps = HashMap::new();
        let mut curr_deps = HashMap::new();

        // Fill the previous dependencies map
        for dep in previous.dependencies() {
            if let Ok(fixed_version) = dep.fixed_version() {
                prev_deps.insert(dep.name().to_string(), fixed_version.to_string());
            }
        }

        // Fill the current dependencies map
        for dep in current.dependencies() {
            if let Ok(fixed_version) = dep.fixed_version() {
                curr_deps.insert(dep.name().to_string(), fixed_version.to_string());
            }
        }

        // Find all unique dependency names
        let mut all_dep_names = HashSet::new();
        for name in prev_deps.keys() {
            all_dep_names.insert(name.clone());
        }
        for name in curr_deps.keys() {
            all_dep_names.insert(name.clone());
        }

        // Sort dependency names for consistent output
        let mut all_dep_names: Vec<String> = all_dep_names.into_iter().collect();
        all_dep_names.sort();

        // Generate dependency changes
        let mut dependency_changes = Vec::new();
        for name in all_dep_names {
            let prev_ver = prev_deps.get(&name);
            let curr_ver = curr_deps.get(&name);

            let change_type = match (prev_ver, curr_ver) {
                (Some(_), None) => ChangeType::Removed,
                (None, Some(_)) => ChangeType::Added,
                (Some(p), Some(c)) if p != c => ChangeType::Updated,
                _ => ChangeType::Unchanged,
            };

            // Only include changes (ignore unchanged deps)
            if change_type != ChangeType::Unchanged {
                dependency_changes.push(DependencyChange::new(
                    &name,
                    prev_ver.map(String::as_str),
                    curr_ver.map(String::as_str),
                    change_type,
                ));
            }
        }

        Ok(Self {
            package_name: previous.name().to_string(),
            previous_version: prev_version,
            current_version: curr_version,
            dependency_changes,
            breaking_change,
        })
    }

    /// Count the number of breaking changes in dependencies.
    ///
    /// Returns the count of dependency changes that are marked as breaking.
    /// A dependency change is considered breaking if:
    /// - It's a removal
    /// - It's an update that increases the major version number
    ///
    /// # Returns
    ///
    /// The number of breaking dependency changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, PackageDiff, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create packages with breaking dependency changes
    /// let old_pkg = Package::new(
    ///     "my-app",
    ///     "1.0.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("react", "^16.0.0")?)),
    ///         Rc::new(RefCell::new(Dependency::new("dep-to-remove", "^1.0.0")?)),
    ///     ])
    /// )?;
    ///
    /// let new_pkg = Package::new(
    ///     "my-app",
    ///     "1.1.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("react", "^17.0.0")?)), // Major bump (breaking)
    ///         // dep-to-remove is removed (breaking)
    ///     ])
    /// )?;
    ///
    /// let diff = PackageDiff::between(&old_pkg, &new_pkg)?;
    /// assert_eq!(diff.count_breaking_changes(), 2); // 1 major update, 1 removal
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn count_breaking_changes(&self) -> usize {
        self.dependency_changes.iter().filter(|c| c.breaking).count()
    }

    /// Count the changes by type.
    ///
    /// Groups dependency changes by their type (added, removed, updated)
    /// and returns a count for each type.
    ///
    /// # Returns
    ///
    /// A hashmap with counts for each change type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Package, PackageDiff, ChangeType, Dependency};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create packages with multiple types of changes
    /// let old_pkg = Package::new(
    ///     "my-app",
    ///     "1.0.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("dep1", "^1.0.0")?)),
    ///         Rc::new(RefCell::new(Dependency::new("dep2", "^1.0.0")?)),
    ///         Rc::new(RefCell::new(Dependency::new("dep-to-remove", "^1.0.0")?)),
    ///     ])
    /// )?;
    ///
    /// let new_pkg = Package::new(
    ///     "my-app",
    ///     "1.1.0",
    ///     Some(vec![
    ///         Rc::new(RefCell::new(Dependency::new("dep1", "^1.5.0")?)), // Updated
    ///         Rc::new(RefCell::new(Dependency::new("dep2", "^1.0.0")?)), // Unchanged (not included)
    ///         Rc::new(RefCell::new(Dependency::new("new-dep", "^1.0.0")?)), // Added
    ///         // dep-to-remove is removed
    ///     ])
    /// )?;
    ///
    /// let diff = PackageDiff::between(&old_pkg, &new_pkg)?;
    /// let counts = diff.count_changes_by_type();
    ///
    /// assert_eq!(*counts.get(&ChangeType::Added).unwrap_or(&0), 1);
    /// assert_eq!(*counts.get(&ChangeType::Removed).unwrap_or(&0), 1);
    /// assert_eq!(*counts.get(&ChangeType::Updated).unwrap_or(&0), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn count_changes_by_type(&self) -> HashMap<ChangeType, usize> {
        let mut counts = HashMap::new();

        for change in &self.dependency_changes {
            *counts.entry(change.change_type.clone()).or_insert(0) += 1;
        }

        counts
    }
}
