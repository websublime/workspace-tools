use std::{cell::RefCell, rc::Rc};

use serde_json::Value;

use crate::{DependencyResolutionError, Package, PackageError, ResolutionResult, VersionError};

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
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError> {
        // First, modify the package dependency separately from JSON
        {
            let update_result =
                self.package.borrow().update_dependency_version(dep_name, new_version);
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
            && self.package.borrow().update_dependency_version(dep_name, new_version).is_err()
        {
            return Err(DependencyResolutionError::DependencyNotFound {
                name: dep_name.to_string(),
                package: self.package.borrow().name().to_string(),
            });
        }

        Ok(())
    }

    /// Apply dependency resolution across all packages
    pub fn apply_dependency_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<(), VersionError> {
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
    pub fn write_package_json(&self) -> Result<(), PackageError> {
        let json_content = serde_json::to_string_pretty(&*self.pkg_json.borrow())
            .map_err(|e| PackageError::into_parse_error(e, self.package_json_path.clone()))?;

        std::fs::write(&self.package_json_path, json_content)
            .map_err(|e| PackageError::into_io_error(e, self.package_json_path.clone()))?;

        Ok(())
    }
}
