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
        // Update Package dependency version
        match self.package.borrow().update_dependency_version(dep_name, new_version) {
            Ok(()) => {
                // Update JSON
                if let Some(obj) = self.pkg_json.borrow_mut().as_object_mut() {
                    if let Some(deps) = obj.get_mut("dependencies").and_then(|v| v.as_object_mut())
                    {
                        if deps.contains_key(dep_name) {
                            deps.insert(
                                dep_name.to_string(),
                                Value::String(new_version.to_string()),
                            );
                        }
                    }
                }

                self.update_dev_dependency_version(dep_name, new_version)?;
                Ok(())
            }
            // Skip error if dependency not found in normal dependencies
            Err(PkgError::DependencyNotFound { .. }) => {
                // Still try to update in package.json in case it's only in devDependencies
                self.update_dev_dependency_version(dep_name, new_version)
            }
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn update_dev_dependency_version(&self, dep_name: &str, new_version: &str) -> Result<()> {
        // Update JSON for devDependencies
        if let Some(obj) = self.pkg_json.borrow_mut().as_object_mut() {
            if let Some(deps) = obj.get_mut("devDependencies").and_then(|v| v.as_object_mut()) {
                if deps.contains_key(dep_name) {
                    deps.insert(dep_name.to_string(), Value::String(new_version.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(&self, resolution: &ResolutionResult) -> Result<()> {
        // Update the package's dependencies
        let updated_deps = self.package.borrow().update_dependencies_from_resolution(resolution)?;

        // Update package.json
        if let Some(pkg_json_obj) = self.pkg_json.borrow_mut().as_object_mut() {
            // Update dependencies section
            if let Some(deps) = pkg_json_obj.get_mut("dependencies").and_then(Value::as_object_mut)
            {
                for (name, _, new_version) in &updated_deps {
                    if deps.contains_key(name) {
                        deps.insert(name.clone(), Value::String(new_version.clone()));
                    }
                }
            }

            // Also check devDependencies
            if let Some(dev_deps) =
                pkg_json_obj.get_mut("devDependencies").and_then(Value::as_object_mut)
            {
                for (name, _, new_version) in &updated_deps {
                    if dev_deps.contains_key(name) {
                        dev_deps.insert(name.clone(), Value::String(new_version.clone()));
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
    // Implementation of parsing package name, scope, and version
    let parts: Vec<&str> = pkg_name.split('@').collect();
    if parts.len() < 2 {
        return None;
    }

    let full = pkg_name.to_string();
    let name_parts: Vec<&str> = parts[1].split(':').collect();
    let name = format!("@{}", name_parts[0]);

    let version =
        if name_parts.len() > 1 { name_parts[1].to_string() } else { "latest".to_string() };

    let path = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };

    Some(PackageScopeMetadata { full, name, version, path })
}
