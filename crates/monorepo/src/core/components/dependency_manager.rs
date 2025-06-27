//! Package dependency management component
//!
//! Handles all dependency-related operations including adding, removing,
//! and querying dependencies and dependents for packages.

use super::super::types::MonorepoPackageInfo;
use std::collections::HashSet;

/// Component for managing package dependencies
pub struct PackageDependencyManager {
    package: MonorepoPackageInfo,
}

impl PackageDependencyManager {
    /// Create a new package dependency manager
    #[must_use]
    pub fn new(package: MonorepoPackageInfo) -> Self {
        Self { package }
    }

    /// Get immutable reference to the package
    #[must_use]
    pub fn package(&self) -> &MonorepoPackageInfo {
        &self.package
    }

    /// Get list of packages that depend on this package
    #[must_use]
    pub fn dependents(&self) -> &[String] {
        &self.package.dependents
    }

    /// Get external dependencies (not in the monorepo)
    #[must_use]
    pub fn external_dependencies(&self) -> &[String] {
        &self.package.dependencies_external
    }

    /// Add a dependent package
    ///
    /// # Arguments
    /// * `dependent_name` - Name of package that depends on this one
    pub fn add_dependent(&mut self, dependent_name: String) {
        if !self.package.dependents.contains(&dependent_name) {
            self.package.dependents.push(dependent_name);
        }
    }

    /// Remove a dependent package
    ///
    /// # Arguments
    /// * `dependent_name` - Name of package to remove from dependents
    ///
    /// # Returns
    /// True if the dependent was removed, false if not found
    pub fn remove_dependent(&mut self, dependent_name: &str) -> bool {
        let initial_len = self.package.dependents.len();
        self.package.dependents.retain(|dep| dep != dependent_name);
        self.package.dependents.len() < initial_len
    }

    /// Add an external dependency
    ///
    /// # Arguments
    /// * `dependency_name` - Name of external dependency to add
    pub fn add_external_dependency(&mut self, dependency_name: String) {
        if !self.package.dependencies_external.contains(&dependency_name) {
            self.package.dependencies_external.push(dependency_name);
        }
    }

    /// Remove an external dependency
    ///
    /// # Arguments
    /// * `dependency_name` - Name of external dependency to remove
    ///
    /// # Returns
    /// True if the dependency was removed, false if not found
    pub fn remove_external_dependency(&mut self, dependency_name: &str) -> bool {
        let initial_len = self.package.dependencies_external.len();
        self.package.dependencies_external.retain(|dep| dep != dependency_name);
        self.package.dependencies_external.len() < initial_len
    }

    /// Check if a package depends on this one
    #[must_use]
    pub fn has_dependent(&self, package_name: &str) -> bool {
        self.package.dependents.contains(&package_name.to_string())
    }

    /// Check if this package has a specific external dependency
    #[must_use]
    pub fn has_external_dependency(&self, dependency_name: &str) -> bool {
        self.package.dependencies_external.contains(&dependency_name.to_string())
    }

    /// Get all unique external dependencies
    #[must_use]
    pub fn unique_external_dependencies(&self) -> HashSet<&String> {
        self.package.dependencies_external.iter().collect()
    }

    /// Get all unique dependents
    #[must_use]
    pub fn unique_dependents(&self) -> HashSet<&String> {
        self.package.dependents.iter().collect()
    }

    /// Clear all dependents
    pub fn clear_dependents(&mut self) {
        self.package.dependents.clear();
    }

    /// Clear all external dependencies
    pub fn clear_external_dependencies(&mut self) {
        self.package.dependencies_external.clear();
    }

    /// Update dependents list with new set
    ///
    /// # Arguments
    /// * `new_dependents` - New list of dependent package names
    pub fn update_dependents(&mut self, new_dependents: Vec<String>) {
        self.package.dependents = new_dependents;
    }

    /// Update external dependencies list with new set
    ///
    /// # Arguments
    /// * `new_dependencies` - New list of external dependency names
    pub fn update_external_dependencies(&mut self, new_dependencies: Vec<String>) {
        self.package.dependencies_external = new_dependencies;
    }

    /// Get dependency statistics
    #[must_use]
    pub fn dependency_stats(&self) -> DependencyStats {
        DependencyStats {
            dependents_count: self.package.dependents.len(),
            external_deps_count: self.package.dependencies_external.len(),
            unique_dependents_count: self.unique_dependents().len(),
            unique_external_deps_count: self.unique_external_dependencies().len(),
        }
    }

    /// Validate dependency consistency
    ///
    /// # Returns
    /// List of validation errors found
    #[must_use]
    pub fn validate_dependencies(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for duplicate dependents
        let mut seen_dependents = HashSet::new();
        for dependent in &self.package.dependents {
            if !seen_dependents.insert(dependent) {
                errors.push(format!("Duplicate dependent: {dependent}"));
            }
        }

        // Check for duplicate external dependencies
        let mut seen_external = HashSet::new();
        for dep in &self.package.dependencies_external {
            if !seen_external.insert(dep) {
                errors.push(format!("Duplicate external dependency: {dep}"));
            }
        }

        // Check for empty names
        for dependent in &self.package.dependents {
            if dependent.trim().is_empty() {
                errors.push("Empty dependent name found".to_string());
            }
        }

        for dep in &self.package.dependencies_external {
            if dep.trim().is_empty() {
                errors.push("Empty external dependency name found".to_string());
            }
        }

        errors
    }

    /// Consume the manager and return the updated package
    #[must_use]
    pub fn into_package(self) -> MonorepoPackageInfo {
        self.package
    }
}

/// Statistics about package dependencies
#[derive(Debug, Clone)]
pub struct DependencyStats {
    /// Number of dependent packages
    pub dependents_count: usize,
    /// Number of external dependencies
    pub external_deps_count: usize,
    /// Number of unique dependents (after deduplication)
    pub unique_dependents_count: usize,
    /// Number of unique external dependencies (after deduplication)
    pub unique_external_deps_count: usize,
}