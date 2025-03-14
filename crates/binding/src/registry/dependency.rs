//! Dependency registry for tracking and resolving shared dependencies.

use crate::errors::handle_pkg_result;
use crate::types::dependency::Dependency;
use napi::bindgen_prelude::*;
use napi::JsString;
use napi::Result as NapiResult;
use napi_derive::napi;
use std::cell::RefCell;
use ws_pkg::registry::{
    DependencyRegistry as WsDependencyRegistry, DependencyUpdate,
    ResolutionResult as WsResolutionResult,
};

/// JavaScript binding for DependencyResolutionError
#[napi]
pub enum ResolutionErrorType {
    /// Error parsing a version
    VersionParseError,
    /// Incompatible versions of the same dependency
    IncompatibleVersions,
    /// No valid version found
    NoValidVersion,
}

/// JavaScript binding for dependency update information
#[napi(object)]
pub struct DependencyUpdateInfo {
    /// Package containing the dependency
    pub package_name: String,

    /// Dependency name
    pub dependency_name: String,

    /// Current version
    pub current_version: String,

    /// New version to update to
    pub new_version: String,
}

impl From<DependencyUpdate> for DependencyUpdateInfo {
    fn from(update: DependencyUpdate) -> Self {
        Self {
            package_name: update.package_name,
            dependency_name: update.dependency_name,
            current_version: update.current_version,
            new_version: update.new_version,
        }
    }
}

/// JavaScript binding for the result of dependency resolution
#[napi(object)]
pub struct ResolutionResult {
    /// Resolved versions for each package
    pub resolved_versions: Object,

    /// Dependencies that need updates
    pub updates_required: Vec<DependencyUpdateInfo>,
}

/// DependencyRegistry class
/// A registry to manage shared dependency instances
///
/// @class DependencyRegistry - The DependencyRegistry class.
/// @example
///
/// ```typescript
/// const registry = new DependencyRegistry();
/// const dep1 = registry.getOrCreate("foo", "^1.0.0");
/// const dep2 = registry.getOrCreate("bar", "^2.0.0");
///
/// // Resolve version conflicts
/// const result = registry.resolveVersionConflicts();
/// console.log(result.resolvedVersions);
/// ```
#[napi]
pub struct DependencyRegistry {
    pub(crate) inner: RefCell<WsDependencyRegistry>,
}

#[napi]
#[allow(clippy::new_without_default)]
impl DependencyRegistry {
    /// Create a new dependency registry
    ///
    /// @returns {DependencyRegistry} A new empty registry
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: RefCell::new(WsDependencyRegistry::new()) }
    }

    /// Get or create a dependency in the registry
    ///
    /// @param {string} name - The name of the dependency
    /// @param {string} version - The version or version requirement
    /// @returns {Dependency} The dependency instance
    #[napi(js_name = "getOrCreate", ts_return_type = "Dependency")]
    pub fn get_or_create(&self, name: String, version: String) -> NapiResult<Dependency> {
        let mut registry = self.inner.borrow_mut();
        let dep_rc = handle_pkg_result(registry.get_or_create(&name, &version))?;
        Ok(Dependency { inner: dep_rc })
    }

    /// Get a dependency by name
    ///
    /// @param {string} name - The name of the dependency
    /// @returns {Dependency | null} The dependency instance if found, null otherwise
    #[napi]
    pub fn get(&self, name: String) -> Option<Dependency> {
        let registry = self.inner.borrow();
        registry.get(&name).map(|dep_rc| Dependency { inner: dep_rc })
    }

    /// Resolve version conflicts between dependencies
    ///
    /// @returns {ResolutionResult} The result of dependency resolution
    #[napi(js_name = "resolveVersionConflicts", ts_return_type = "ResolutionResult")]
    pub fn resolve_version_conflicts(&self, env: Env) -> NapiResult<ResolutionResult> {
        let registry = self.inner.borrow();
        let result = handle_pkg_result(registry.resolve_version_conflicts())?;

        // Create a JavaScript Object for resolved_versions
        let mut resolved_versions_obj = env.create_object()?;
        for (key, value) in &result.resolved_versions {
            resolved_versions_obj.set_named_property(key, value.as_str())?;
        }

        // Convert updates to DependencyUpdateInfo structs
        let updates_required =
            result.updates_required.into_iter().map(DependencyUpdateInfo::from).collect();

        Ok(ResolutionResult { resolved_versions: resolved_versions_obj, updates_required })
    }

    /// Apply a resolution result to update all dependencies
    ///
    /// @param {ResolutionResult} result - The resolution result to apply
    /// @returns {void}
    #[napi(js_name = "applyResolutionResult", ts_return_type = "void")]
    pub fn apply_resolution_result(&self, result: ResolutionResult) -> NapiResult<()> {
        let mut registry = self.inner.borrow_mut();

        // Extract resolved versions from JS Object into HashMap
        let mut resolved_versions = std::collections::HashMap::new();
        let prop_names = result.resolved_versions.get_property_names()?;
        let length = prop_names.get_array_length()?;

        for i in 0..length {
            // Get the property name as a JavaScript string first
            let js_key = prop_names.get_element::<JsString>(i)?;
            // Convert JavaScript string to Rust String
            let key = js_key.into_utf8()?.into_owned()?;
            // Get the value as a JavaScript string
            let js_value = result.resolved_versions.get_named_property::<JsString>(key.as_str())?;
            // Convert JavaScript string to Rust String
            let value = js_value.into_utf8()?.into_owned()?;
            // Store in our HashMap
            resolved_versions.insert(key, value);
        }

        // Convert updates to DependencyUpdate structs
        let updates_required = result
            .updates_required
            .into_iter()
            .map(|update| DependencyUpdate {
                package_name: update.package_name,
                dependency_name: update.dependency_name,
                current_version: update.current_version,
                new_version: update.new_version,
            })
            .collect();

        let ws_result = WsResolutionResult { resolved_versions, updates_required };

        handle_pkg_result(registry.apply_resolution_result(&ws_result))
    }

    /// Find highest version that is compatible with all requirements
    ///
    /// @param {string} name - The name of the dependency
    /// @param {string[]} requirements - List of version requirements
    /// @returns {string | null} The highest compatible version, if any
    #[napi(js_name = "findHighestCompatibleVersion", ts_return_type = "string | null")]
    pub fn find_highest_compatible_version(
        &self,
        name: String,
        requirements: Vec<String>,
    ) -> NapiResult<Option<String>> {
        let registry = self.inner.borrow();

        // Convert string requirements to VersionReq
        let version_reqs = requirements
            .iter()
            .map(|req| {
                semver::VersionReq::parse(req).map_err(|e| {
                    let err = ws_pkg::PkgError::VersionReqParseError {
                        requirement: req.clone(),
                        source: e,
                    };
                    crate::pkg_error_to_napi_error(err)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Create slice of references
        let version_req_refs: Vec<&semver::VersionReq> = version_reqs.iter().collect();

        // Find highest compatible version
        let result = registry.find_highest_compatible_version(&name, &version_req_refs);
        Ok(result)
    }
}

#[cfg(test)]
mod registry_binding_tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_dependency_registry_creation() {
        let registry = DependencyRegistry::new();
        assert!(registry.inner.borrow().get("nonexistent").is_none());
    }

    #[test]
    fn test_get_or_create() {
        let registry = DependencyRegistry::new();

        // Create a new dependency
        let dep = registry.get_or_create("test-dep".to_string(), "^1.0.0".to_string()).unwrap();
        assert_eq!(dep.name(), "test-dep");
        assert_eq!(dep.version(), "^1.0.0");

        // Get the same dependency again
        let dep2 = registry.get_or_create("test-dep".to_string(), "^1.0.0".to_string()).unwrap();
        assert_eq!(dep2.name(), "test-dep");

        // These should be the same internal object (RC points to same data)
        assert!(Rc::ptr_eq(&dep.inner, &dep2.inner));
    }

    #[test]
    fn test_get_dependency() {
        let registry = DependencyRegistry::new();

        // Create a dependency
        let _ = registry.get_or_create("test-dep".to_string(), "^1.0.0".to_string()).unwrap();

        // Get it directly
        let dep = registry.get("test-dep".to_string()).unwrap();
        assert_eq!(dep.name(), "test-dep");

        // Try getting a non-existent dependency
        assert!(registry.get("nonexistent".to_string()).is_none());
    }

    #[test]
    fn test_dependency_update_info() {
        // Test the DependencyUpdateInfo struct
        let update = DependencyUpdateInfo {
            package_name: "pkg1".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            new_version: "^2.0.0".to_string(),
        };

        assert_eq!(update.package_name, "pkg1");
        assert_eq!(update.dependency_name, "dep1");
        assert_eq!(update.current_version, "^1.0.0");
        assert_eq!(update.new_version, "^2.0.0");
    }

    #[test]
    fn test_from_dependency_update() {
        // Test conversion from ws_pkg's DependencyUpdate
        let ws_update = ws_pkg::registry::DependencyUpdate {
            package_name: "pkg1".to_string(),
            dependency_name: "dep1".to_string(),
            current_version: "^1.0.0".to_string(),
            new_version: "^2.0.0".to_string(),
        };

        let update = DependencyUpdateInfo::from(ws_update);

        assert_eq!(update.package_name, "pkg1");
        assert_eq!(update.dependency_name, "dep1");
        assert_eq!(update.current_version, "^1.0.0");
        assert_eq!(update.new_version, "^2.0.0");
    }

    // Note: We can't test resolve_version_conflicts and apply_resolution_result in unit tests
    // since they require a JavaScript environment (Env). These should be tested with integration tests.
}
