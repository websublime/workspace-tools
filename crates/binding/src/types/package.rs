//! JavaScript bindings for the ws_pkg::Package type.

use crate::errors::handle_pkg_result;
use crate::types::dependency::Dependency;
use napi::Result as NapiResult;
use napi_derive::napi;
use std::rc::Rc;
use ws_pkg::Package as WsPkgPackage;

/// JavaScript binding for ws_pkg::Package
#[napi]
pub struct Package {
    pub(crate) inner: WsPkgPackage,
}

#[napi]
impl Package {
    /// Create a new package with a name and version
    #[napi(constructor)]
    pub fn new(name: String, version: String) -> Self {
        match WsPkgPackage::new(&name, &version, None) {
            Ok(inner) => Self { inner },
            Err(err) => {
                // Since constructors can't return Result<T, E>, we need to panic
                // napi-rs will convert this panic to a JavaScript exception
                let js_error = crate::pkg_error_to_napi_error(err);
                panic!("{}", js_error.reason);
            }
        }
    }

    /// Get the package name
    #[napi]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the package version
    #[napi]
    pub fn version(&self) -> String {
        self.inner.version_str()
    }

    /// Update the package version
    #[napi]
    pub fn update_version(&self, version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.update_version(&version))
    }

    /// Get all dependencies of this package
    ///
    /// This method returns an array of Dependency objects that can be used in JavaScript.
    /// Note: Due to technical limitations, this method requires special handling in JavaScript.
    #[napi]
    pub fn dependencies(&self) -> Vec<Dependency> {
        let mut deps = Vec::new();

        for dep_rc in self.inner.dependencies() {
            deps.push(Dependency { inner: Rc::clone(dep_rc) });
        }

        deps
    }

    /// Add a dependency to this package
    #[napi]
    pub fn add_dependency(&mut self, dependency: &Dependency) {
        self.inner.add_dependency(Rc::clone(&dependency.inner));
    }

    /// Update a dependency's version
    #[napi]
    pub fn update_dependency_version(&self, name: String, version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.update_dependency_version(&name, &version))
    }

    /// Get a dependency by name
    #[napi]
    pub fn get_dependency(&self, name: String) -> Option<Dependency> {
        for dep_rc in self.inner.dependencies() {
            let dep = dep_rc.borrow();
            if dep.name() == name {
                return Some(Dependency { inner: Rc::clone(dep_rc) });
            }
        }
        None
    }

    /// Get the number of dependencies
    #[napi]
    pub fn dependency_count(&self) -> u32 {
        self.inner.dependencies().len() as u32
    }
}

#[cfg(test)]
mod package_binding_tests {
    use super::*;
    use crate::types::dependency::Dependency;

    #[test]
    #[should_panic]
    fn test_package_creation_invalid() {
        // Test invalid version - should panic
        let _ = Package::new("test-pkg".to_string(), "invalid".to_string());
    }

    #[test]
    fn test_package_creation_valid() {
        // Test successful package creation
        let pkg = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        assert_eq!(pkg.name(), "test-pkg");
        assert_eq!(pkg.version(), "1.0.0");
    }

    #[test]
    fn test_package_update_version() {
        let pkg = Package::new("test-pkg".to_string(), "1.0.0".to_string());

        // Test valid version update
        let result = pkg.update_version("2.0.0".to_string());
        assert!(result.is_ok());
        assert_eq!(pkg.version(), "2.0.0");

        // Test invalid version update
        let result = pkg.update_version("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_and_get_dependencies() {
        let mut pkg = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        let dep = Dependency::new("dep1".to_string(), "^1.0.0".to_string());

        // Add dependency
        pkg.add_dependency(&dep);

        // Check dependencies
        let deps = pkg.dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name(), "dep1");
        assert_eq!(deps[0].version(), "^1.0.0");

        // Check dependency count
        assert_eq!(pkg.dependency_count(), 1);

        // Check get_dependency
        let retrieved_dep = pkg.get_dependency("dep1".to_string());
        assert!(retrieved_dep.is_some());
        assert_eq!(retrieved_dep.unwrap().name(), "dep1");
    }

    #[test]
    fn test_update_dependency_version() {
        let mut pkg = Package::new("test-pkg".to_string(), "1.0.0".to_string());
        let dep = Dependency::new("dep1".to_string(), "^1.0.0".to_string());

        // Add dependency
        pkg.add_dependency(&dep);

        // Update dependency version
        let result = pkg.update_dependency_version("dep1".to_string(), "^2.0.0".to_string());
        assert!(result.is_ok());

        // Verify the update using dependencies()
        let deps = pkg.dependencies();
        assert_eq!(deps[0].version(), "^2.0.0");

        // Test updating non-existent dependency
        let result = pkg.update_dependency_version("nonexistent".to_string(), "^1.0.0".to_string());
        assert!(result.is_err());
    }
}
