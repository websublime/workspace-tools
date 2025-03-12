use crate::errors::handle_pkg_result;
use napi::Result as NapiResult;
use napi_derive::napi;
use std::cell::RefCell;
use std::rc::Rc;
use ws_pkg::Dependency as WsPkgDependency;

/// Dependency class.
/// Represents a package dependency.
///
/// @class Dependency - The Dependency class.
/// @property {string} name - The name of the dependency.
/// @property {string} version - The version of the dependency.
///
/// @example
///
/// ```typescript
/// const dep = new Dependency("foo", "1.0.0");
/// console.log(dep.name); // foo
/// console.log(dep.version); // 1.0.0
/// ```
#[napi]
pub struct Dependency {
    pub(crate) inner: Rc<RefCell<WsPkgDependency>>,
}

#[napi]
impl Dependency {
    /// Create a new dependency with a name and version
    ///
    /// @param {string} name - The name of the dependency package.
    /// @param {string} version - The version of the dependency.
    ///
    /// @returns {Dependency} The new dependency.
    #[napi(constructor)]
    pub fn new(name: String, version: String) -> Self {
        match WsPkgDependency::new(&name, &version) {
            Ok(inner) => Self { inner: Rc::new(RefCell::new(inner)) },
            Err(err) => {
                // Since constructors can't return Result<T, E>, we need to panic
                // napi-rs will convert this panic to a JavaScript exception
                let js_error = crate::pkg_error_to_napi_error(err);
                panic!("{}", js_error.reason);
            }
        }
    }

    /// Gets the name of the dependency.
    ///
    /// @returns {string} The name of the dependency.
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.borrow().name().to_string()
    }

    /// Gets the version of the dependency.
    ///
    /// @returns {string} The version of the dependency.
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.inner.borrow().version_str()
    }

    /// Updates dependency version
    ///
    /// @param {string} version - The new version of the dependency.
    /// @returns {Promise<void>} A promise that resolves when the version is updated.
    #[napi]
    pub fn update_version(&self, version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.borrow().update_version(&version))
    }
}

#[cfg(test)]
mod dependency_binding_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_dependency_creation_invalid() {
        // Test invalid version - should panic
        let _ = Dependency::new("dep1".to_string(), "invalid".to_string());
    }

    #[test]
    fn test_dependency_creation_valid() {
        // Test successful dependency creation
        let dep = Dependency::new("dep1".to_string(), "^1.0.0".to_string());
        assert_eq!(dep.name(), "dep1");
        assert_eq!(dep.version(), "^1.0.0");
    }

    #[test]
    fn test_dependency_update_version() {
        let dep = Dependency::new("dep1".to_string(), "^1.0.0".to_string());

        // Test valid version update
        let result = dep.update_version("^2.0.0".to_string());
        assert!(result.is_ok());
        assert_eq!(dep.version(), "^2.0.0");

        // Test invalid version update
        let result = dep.update_version("invalid".to_string());
        assert!(result.is_err());
    }
}
