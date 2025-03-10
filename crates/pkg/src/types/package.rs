//! Package type and related functionality.

use crate::error::{PkgError, Result};
use crate::registry::ResolutionResult;
use crate::types::dependency::Dependency;
use crate::DependencyRegistry;
use semver::Version;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// Parse package scope, name, and version from a string
pub struct PackageScopeMetadata {
    pub full: String,
    pub name: String,
    pub version: String,
    pub path: Option<String>,
}

/// Package represents a package with dependencies
#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    version: Rc<RefCell<Version>>,
    dependencies: Vec<Rc<RefCell<Dependency>>>,
}

impl Package {
    /// Create a new package with name, version, and optional dependencies
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Result<Self> {
        let parsed_version = version
            .parse()
            .map_err(|e| PkgError::VersionParseError { version: version.to_string(), source: e })?;

        Ok(Self {
            name: name.to_string(),
            version: Rc::new(RefCell::new(parsed_version)),
            dependencies: dependencies.unwrap_or_default(),
        })
    }

    /// Create a new package using the dependency registry
    pub fn new_with_registry(
        name: &str,
        version: &str,
        dependencies: Option<Vec<(&str, &str)>>,
        registry: &mut DependencyRegistry,
    ) -> Result<Self> {
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
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package version
    pub fn version(&self) -> Version {
        self.version.borrow().clone()
    }

    /// Get the package version as a string
    pub fn version_str(&self) -> String {
        self.version.borrow().to_string()
    }

    /// Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<()> {
        let parsed_version = new_version.parse().map_err(|e| PkgError::VersionParseError {
            version: new_version.to_string(),
            source: e,
        })?;
        *self.version.borrow_mut() = parsed_version;
        Ok(())
    }

    /// Get the package dependencies
    pub fn dependencies(&self) -> &[Rc<RefCell<Dependency>>] {
        &self.dependencies
    }

    /// Update a dependency version
    pub fn update_dependency_version(&self, dep_name: &str, new_version: &str) -> Result<()> {
        for dep in &self.dependencies {
            let name = dep.borrow().name().to_string();
            if name == dep_name {
                dep.borrow().update_version(new_version)?;
                return Ok(());
            }
        }

        Err(PkgError::DependencyNotFound { name: dep_name.to_string(), package: self.name.clone() })
    }

    /// Add a dependency to the package
    pub fn add_dependency(&mut self, dependency: Rc<RefCell<Dependency>>) {
        self.dependencies.push(dependency);
    }

    /// Update package dependencies based on resolution result
    pub fn update_dependencies_from_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<Vec<(String, String, String)>> {
        let mut updated_deps = Vec::new();

        for dep in &self.dependencies {
            let name = dep.borrow().name().to_string();
            if let Some(resolved_version) = resolution.resolved_versions.get(&name) {
                let current_version = dep.borrow().version_str();

                // Only update if the versions are different
                if current_version != *resolved_version
                    && !current_version.contains(resolved_version)
                {
                    dep.borrow().update_version(resolved_version)?;
                    updated_deps.push((name, current_version, resolved_version.clone()));
                }
            }
        }

        Ok(updated_deps)
    }
}

/// PackageInfo represents a package with its metadata
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub package: Rc<RefCell<Package>>,
    pub package_json_path: String,
    pub package_path: String,
    pub package_relative_path: String,
    pub pkg_json: Rc<RefCell<Value>>,
}

impl PackageInfo {
    /// Create a new package info
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

    /// Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<()> {
        // Update Package version
        self.package.borrow().update_version(new_version)?;

        // Update JSON
        if let Some(obj) = self.pkg_json.borrow_mut().as_object_mut() {
            obj.insert("version".to_string(), Value::String(new_version.to_string()));
        }

        Ok(())
    }

    /// Update a dependency version
    pub fn update_dependency_version(&self, dep_name: &str, new_version: &str) -> Result<()> {
        // First, modify the package dependency separately from JSON
        {
            let update_result =
                self.package.borrow().update_dependency_version(dep_name, new_version);
            if let Err(PkgError::DependencyNotFound { .. }) = update_result {
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
            && self.package.borrow().update_dependency_version(dep_name, new_version).is_err()
        {
            return Err(PkgError::DependencyNotFound {
                name: dep_name.to_string(),
                package: self.package.borrow().name().to_string(),
            });
        }

        Ok(())
    }

    /// Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(&self, resolution: &ResolutionResult) -> Result<()> {
        // First, update the package's dependencies (handles regular dependencies)
        let _ = { self.package.borrow().update_dependencies_from_resolution(resolution)? }; // Package borrow is dropped here

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
    pub fn write_package_json(&self) -> Result<()> {
        let json_content = serde_json::to_string_pretty(&*self.pkg_json.borrow()).map_err(|e| {
            PkgError::JsonParseError {
                path: Some(self.package_json_path.clone().into()),
                source: e,
            }
        })?;

        std::fs::write(&self.package_json_path, json_content).map_err(|e| PkgError::IoError {
            path: Some(self.package_json_path.clone().into()),
            source: e,
        })
    }
}

/// Parse package scope, name, and version from a string
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata> {
    // Must start with @ to be a scoped package
    if !pkg_name.starts_with('@') {
        return None;
    }

    let full = pkg_name.to_string();
    let mut name = String::new();
    let mut version = "latest".to_string();
    let mut path = None;

    // First check for colon format: @scope/name:version
    if pkg_name.contains(':') {
        let parts: Vec<&str> = pkg_name.split(':').collect();
        name = parts[0].to_string();
        if parts.len() > 1 {
            version = parts[1].to_string();
        }
    }
    // Handle @ format: @scope/name@version or @scope/name@version@path
    else {
        let parts: Vec<&str> = pkg_name.split('@').collect();

        // First part is empty because it starts with @
        if parts.len() >= 2 {
            // Format: @scope/name
            name = format!("@{}", parts[1]);

            // Check if there's a version
            if parts.len() >= 3 {
                // Format: @scope/name@version
                version = parts[2].to_string();

                // Check if there's a path
                if parts.len() >= 4 {
                    // Format: @scope/name@version@path
                    path = Some(parts[3].to_string());
                }
            }
        }
    }

    Some(PackageScopeMetadata { full, name, version, path })
}
