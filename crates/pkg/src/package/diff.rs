use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{ChangeType, DependencyChange, Package, PackageError};

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
        for dep_rc in previous.dependencies() {
            let dep = dep_rc.borrow();

            if let Ok(fixed_version) = dep.fixed_version() {
                prev_deps.insert(dep.name().to_string(), fixed_version.to_string());
            }
        }

        // Fill the current dependencies map
        for dep_rc in current.dependencies() {
            let dep = dep_rc.borrow();

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
