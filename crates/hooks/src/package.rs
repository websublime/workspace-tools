use crate::HookResult;
use sublime_package_tools::{
    Package,
    PackageRegistry,
    LocalRegistry,
    Version,
    PackageInfo,
    PackageError,
    RegistryManager,
    build_dependency_graph_from_package_infos,
};
use std::path::Path;
use crate::{error::HookError};
use std::sync::Arc;

#[derive(Debug)]
pub struct PackageManager {
    registry: Box<dyn PackageRegistry>,
}

impl PackageManager {
    /// Creates a new package manager
    pub fn new(workspace_root: &Path) -> HookResult<Self> {
        let mut manager = RegistryManager::new();
        let local_registry = LocalRegistry::default();
        manager.add_registry_instance("local", Arc::new(local_registry));
        Ok(Self {
            registry: Box::new(local_registry),
        })
    }

    /// Gets a package by name
    pub fn get_package(&self, name: &str) -> HookResult<PackageInfo> {
        let info = self.registry.get_package_info(name, "*").map_err(|e| HookError::Hook(e.to_string()))?;
        Ok(info)
    }

    /// Gets the current version of a package
    pub fn get_package_version(&self, name: &str) -> HookResult<String> {
        let package = self.get_package(name)?;
        let version = package.package.borrow().version_str();
        Ok(version)
    }

    /// Gets all dependent packages that need to be updated
    pub fn get_dependents(&self, package: &str) -> HookResult<Vec<String>> {
        let dependents = self.registry.get_package_dependents(package)?;
        Ok(dependents.into_iter().map(|p| p.name().to_string()).collect())
    }

    /// Records a version decision for a package
    pub fn record_version_decision(&self, package: &str, version_bump: Version) -> HookResult<()> {
        let mut package = self.get_package(package)?;
        if let Some(obj) = package.pkg_json.borrow_mut().as_object_mut() {
            obj.insert("version_decision".to_string(), version_bump.to_string().into());
            package.write_package_json().map_err(|e| HookError::Hook(e.to_string()))?;
        }
        Ok(())
    }

    /// Gets the recorded version decision for a package
    pub fn get_version_decision(&self, package: &str) -> HookResult<Option<Version>> {
        let package = self.get_package(package)?;
        Ok(package.pkg_json.borrow()
            .get("version_decision")
            .and_then(|v| v.as_str())
            .and_then(|v| Version::from_str(v).ok()))
    }

    /// Checks if a package has dependents that would be affected by a version bump
    pub fn has_affected_dependents(&self, package: &str) -> HookResult<bool> {
        let package = self.get_package(package)?;
        let mut packages = Vec::new();
        let graph = build_dependency_graph_from_package_infos(&[package], &mut packages);
        Ok(!graph.get_dependents(&package.package.borrow().name())
            .map_err(|e| HookError::Hook(e.to_string()))?
            .is_empty())
    }
} 