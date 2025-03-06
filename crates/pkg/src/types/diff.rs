//! Package diff functionality.

use crate::error::{PkgError, Result};
use crate::types::package::Package;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Types of changes that can occur between package versions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// Package was added
    Added,
    /// Package was removed
    Removed,
    /// Package version was updated
    Updated,
    /// No change detected
    Unchanged,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Updated => write!(f, "updated"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

/// Represents a change in a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChange {
    /// Name of the dependency
    pub name: String,
    /// Previous version (None if newly added)
    pub previous_version: Option<String>,
    /// Current version (None if removed)
    pub current_version: Option<String>,
    /// Type of change
    pub change_type: ChangeType,
    /// Whether this is a breaking change based on semver
    pub breaking: bool,
}

impl DependencyChange {
    /// Creates a new dependency change
    pub fn new(
        name: &str,
        previous_version: Option<&str>,
        current_version: Option<&str>,
        change_type: ChangeType,
    ) -> Self {
        // Determine if this is a breaking change based on semver
        let breaking = match (previous_version, current_version) {
            (Some(prev), Some(curr)) => {
                // Clean up the version strings
                let clean_prev = prev.trim_start_matches('^').trim_start_matches('~');
                let clean_curr = curr.trim_start_matches('^').trim_start_matches('~');

                if let (Ok(prev_ver), Ok(curr_ver)) =
                    (Version::parse(clean_prev), Version::parse(clean_curr))
                {
                    // Breaking if major version increases
                    curr_ver.major > prev_ver.major
                } else {
                    // If we can't parse the version, conservatively assume it might be breaking
                    true
                }
            }
            // Only removals are considered breaking changes, not additions
            (Some(_), None) => true, // Removed dependency is breaking
            (None, Some(_) | None) => false, // Added dependency is not breaking
        };

        Self {
            name: name.to_string(),
            previous_version: previous_version.map(String::from),
            current_version: current_version.map(String::from),
            change_type,
            breaking,
        }
    }
}

/// The complete diff between two package versions
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
    /// Generate a diff between two packages
    pub fn between(previous: &Package, current: &Package) -> Result<Self> {
        if previous.name() != current.name() {
            return Err(PkgError::Other {
                message: format!(
                    "Cannot diff different packages: {} vs {}",
                    previous.name(),
                    current.name()
                ),
            });
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
        for dep_rc in previous.dependencies() {
            let dep = dep_rc.borrow();
            prev_deps.insert(dep.name().to_string(), dep.version_str());
        }

        // Fill the current dependencies map
        for dep_rc in current.dependencies() {
            let dep = dep_rc.borrow();
            curr_deps.insert(dep.name().to_string(), dep.version_str());
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

    /// Count the number of breaking changes in dependencies
    pub fn count_breaking_changes(&self) -> usize {
        self.dependency_changes.iter().filter(|c| c.breaking).count()
    }

    /// Count the changes by type
    pub fn count_changes_by_type(&self) -> HashMap<ChangeType, usize> {
        let mut counts = HashMap::new();

        for change in &self.dependency_changes {
            *counts.entry(change.change_type.clone()).or_insert(0) += 1;
        }

        counts
    }
}
