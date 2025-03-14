//! JavaScript bindings for the ws_pkg::Package type.

use crate::errors::handle_pkg_result;
use crate::registry::dependency::{DependencyRegistry, ResolutionResult};
use crate::types::dependency::Dependency;
use napi::{Env, JsString, Result as NapiResult};
use napi_derive::napi;
use std::cell::RefCell;
use std::rc::Rc;
use ws_pkg::registry::ResolutionResult as WsResolutionResult;
use ws_pkg::types::package::package_scope_name_version;
use ws_pkg::types::package::PackageInfo as WsPackageInfo;
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

    /// Create a new package with dependencies using the dependency registry
    ///
    /// @param {string} name - The name of the package
    /// @param {string} version - The version of the package
    /// @param {Array<[string, string]>} dependencies - Array of [name, version] tuples for dependencies
    /// @param {DependencyRegistry} registry - The dependency registry to use
    /// @returns {Package} The new package
    #[napi(ts_return_type = "Package")]
    pub fn with_registry(
        name: String,
        version: String,
        dependencies: Option<Vec<(String, String)>>,
        registry: &DependencyRegistry,
    ) -> NapiResult<Package> {
        // Create a package first
        let pkg = handle_pkg_result(WsPkgPackage::new(&name, &version, None))?;
        let mut package = Package { inner: pkg };

        // Then add dependencies if provided
        if let Some(deps) = dependencies {
            for (dep_name, dep_version) in deps {
                let dep = registry.get_or_create(dep_name, dep_version)?;
                package.add_dependency(&dep);
            }
        }

        Ok(package)
    }

    /// Get the package name
    ///
    /// @returns {string} The package name
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the package version
    ///
    /// @returns {string} The package version
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.inner.version_str()
    }

    /// Update the package version
    ///
    /// @param {string} version - The new version to set
    #[napi(ts_return_type = "void")]
    pub fn update_version(&self, version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.update_version(&version))
    }

    /// Get all dependencies of this package
    ///
    /// This method returns an array of Dependency objects that can be used in JavaScript.
    /// Note: Due to technical limitations, this method requires special handling in JavaScript.
    ///
    /// @returns {Array<Dependency>} An array of Dependency objects
    #[napi]
    pub fn dependencies(&self) -> Vec<Dependency> {
        let mut deps = Vec::new();

        for dep_rc in self.inner.dependencies() {
            deps.push(Dependency { inner: Rc::clone(dep_rc) });
        }

        deps
    }

    /// Add a dependency to this package
    ///
    /// @param {Dependency} dependency - The dependency to add
    #[napi(ts_return_type = "void")]
    pub fn add_dependency(&mut self, dependency: &Dependency) {
        self.inner.add_dependency(Rc::clone(&dependency.inner));
    }

    /// Update a dependency's version
    ///
    /// @param {string} name - The name of the dependency to update
    /// @param {string} version - The new version of the dependency
    #[napi(ts_return_type = "void")]
    pub fn update_dependency_version(&self, name: String, version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.update_dependency_version(&name, &version))
    }

    /// Get a dependency by name
    ///
    /// @param {string} name - The name of the dependency to get
    /// @returns {Dependency | null} A dependency or null if not found
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
    ///
    /// @returns {number} The number of dependencies
    #[napi(getter)]
    pub fn dependency_count(&self) -> u32 {
        self.inner.dependencies().len() as u32
    }

    /// Update dependencies based on a resolution result
    ///
    /// This method updates all dependencies in the package according to the
    /// resolution result.
    ///
    /// @param {ResolutionResult} resolution - The resolution result to apply
    /// @param {Env} env - The NAPI environment
    /// @returns {Array<[string, string, string]>} Array of [name, oldVersion, newVersion] tuples for updated deps
    #[napi(
        js_name = "updateDependenciesFromResolution",
        ts_return_type = "Array<[string, string, string]>"
    )]
    pub fn update_dependencies_from_resolution(
        &self,
        resolution: ResolutionResult,
    ) -> NapiResult<Vec<(String, String, String)>> {
        // Extract resolved versions from JS Object into HashMap
        let mut resolved_versions = std::collections::HashMap::new();
        let prop_names = resolution.resolved_versions.get_property_names()?;
        let length = prop_names.get_array_length()?;

        for i in 0..length {
            // Get the property name as a JavaScript string first
            let js_key = prop_names.get_element::<JsString>(i)?;
            // Convert JavaScript string to Rust String
            let key = js_key.into_utf8()?.into_owned()?;
            // Get the value as a JavaScript string
            let js_value =
                resolution.resolved_versions.get_named_property::<JsString>(key.as_str())?;
            // Convert JavaScript string to Rust String
            let value = js_value.into_utf8()?.into_owned()?;
            // Store in our HashMap
            resolved_versions.insert(key, value);
        }

        // Create a Rust ResolutionResult to pass to the package method
        let ws_resolution = WsResolutionResult {
            resolved_versions,
            updates_required: vec![], // We don't actually need this for updating
        };

        // Call the update method on the package
        let updated_deps =
            handle_pkg_result(self.inner.update_dependencies_from_resolution(&ws_resolution))?;

        // Return the updated dependencies info
        Ok(updated_deps)
    }

    /// Check for dependency version conflicts in this package
    ///
    /// @param {DependencyRegistry} registry - The dependency registry to use
    /// @returns {Array<[string, Array<string>]>} Map of dependency names to conflicting version requirements
    #[napi(js_name = "findVersionConflicts", ts_return_type = "Array<[string, Array<string>]>")]
    pub fn find_version_conflicts(&self, env: Env) -> NapiResult<napi::bindgen_prelude::Object> {
        // Create a JavaScript object to hold the conflicts
        let mut conflicts_obj = env.create_object()?;

        // Check each dependency against the registry
        let deps = self.inner.dependencies();
        // Registry is not actually used, so we can remove this line
        // let reg = registry.inner.borrow();

        // Create a map to track versions seen for each dependency
        let mut versions_by_dep = std::collections::HashMap::new();

        // Collect version requirements for each dependency
        for dep_rc in deps {
            let dep = dep_rc.borrow();
            let name = dep.name().to_string();
            let version = dep.version_str();

            versions_by_dep.entry(name).or_insert_with(Vec::new).push(version);
        }

        // Find conflicts (where a dependency has multiple different version requirements)
        for (name, versions) in versions_by_dep {
            if versions.len() > 1 {
                // Create a JavaScript array for this dependency's versions
                let mut versions_array = env.create_array_with_length(versions.len())?;
                for (i, version) in versions.iter().enumerate() {
                    // Create a JavaScript string from the Rust string
                    let js_version = env.create_string(version)?;
                    versions_array.set_element(i as u32, js_version)?;
                }

                // Add this entry to the conflicts object
                conflicts_obj.set_named_property(&name, versions_array)?;
            }
        }

        Ok(conflicts_obj)
    }

    /// Generate combined dependency information for all packages
    ///
    /// @param {Package[]} packages - Array of packages to analyze
    /// @param {DependencyRegistry} registry - The dependency registry to use
    /// @returns {DependencyInfo} Object with dependency information
    #[napi(js_name = "generateDependencyInfo", ts_return_type = "DependencyInfo")]
    pub fn generate_dependency_info(
        env: Env,
        packages: Vec<&Package>,
    ) -> NapiResult<napi::bindgen_prelude::Object> {
        // Create result object
        let mut result = env.create_object()?;

        // Track all dependencies and their versions across packages
        let mut all_deps = std::collections::HashMap::new();
        let mut packages_by_dep = std::collections::HashMap::new();

        // Collect information from all packages
        for pkg in &packages {
            let pkg_name = pkg.inner.name();

            for dep_rc in pkg.inner.dependencies() {
                let dep = dep_rc.borrow();
                let dep_name = dep.name().to_string();
                let version = dep.version_str();

                all_deps.entry(dep_name.clone()).or_insert_with(Vec::new).push(version);

                packages_by_dep.entry(dep_name).or_insert_with(Vec::new).push(pkg_name.to_string());
            }
        }

        // Store the total count before moving all_deps
        let total_deps_count = all_deps.len() as i32;

        // Create "dependencies" object with all dependency info
        let mut deps_obj = env.create_object()?;
        for (dep_name, versions) in all_deps {
            let mut dep_info = env.create_object()?;

            // Add versions array
            let mut versions_array = env.create_array_with_length(versions.len())?;
            for (i, version) in versions.iter().enumerate() {
                let js_version = env.create_string(version)?;
                versions_array.set_element(i as u32, js_version)?;
            }
            dep_info.set_named_property("versions", versions_array)?;

            // Add packages array
            if let Some(pkgs) = packages_by_dep.get(&dep_name) {
                let mut pkgs_array = env.create_array_with_length(pkgs.len())?;
                for (i, pkg) in pkgs.iter().enumerate() {
                    let js_pkg = env.create_string(pkg)?;
                    pkgs_array.set_element(i as u32, js_pkg)?;
                }
                dep_info.set_named_property("packages", pkgs_array)?;
            }

            // Add to main deps object
            deps_obj.set_named_property(&dep_name, dep_info)?;
        }

        // Add dependencies object to result
        result.set_named_property("dependencies", deps_obj)?;

        // Add total count using the value saved before
        result.set_named_property("totalDependencies", total_deps_count)?;

        Ok(result)
    }
}

/// JavaScript binding for ws_pkg::PackageInfo
/// Represents a package with its metadata
///
/// @class PackageInfo - The PackageInfo class.
/// @example
///
/// ```typescript
/// const pkgInfo = new PackageInfo(package, "/path/to/package.json", "/path/to/package", "./relative/path", packageJson);
/// console.log(pkgInfo.packageJsonPath); // /path/to/package.json
/// ```
#[napi]
pub struct PackageInfo {
    pub(crate) inner: Rc<RefCell<WsPackageInfo>>,
}

#[napi]
impl PackageInfo {
    /// Create a new package info object
    ///
    /// @param {Package} pkg - The package object
    /// @param {string} packageJsonPath - Path to the package.json file
    /// @param {string} packagePath - Path to the package directory
    /// @param {string} packageRelativePath - Relative path to the package directory
    /// @param {Object} packageJson - The package.json content
    /// @returns {PackageInfo} The new package info
    #[napi(constructor)]
    pub fn new(
        pkg: &Package,
        package_json_path: String,
        package_path: String,
        package_relative_path: String,
        package_json: napi::bindgen_prelude::Object,
        env: Env,
    ) -> Self {
        // Handle conversion safely, panic on error since constructors can't return Result
        let js_unknown = package_json.into_unknown();

        // Convert to serde_json::Value, using our error handling
        let pkg_json_value: serde_json::Value = match env.from_js_value(js_unknown) {
            Ok(val) => val,
            Err(e) => {
                let pkg_error = ws_pkg::PkgError::Other {
                    message: format!("Failed to convert package_json to serde_json::Value: {}", e),
                };
                let js_error = crate::pkg_error_to_napi_error(pkg_error);
                panic!("{}", js_error.reason);
            }
        };

        // Create a new WsPackageInfo
        let ws_package_info = WsPackageInfo::new(
            pkg.inner.clone(),
            package_json_path,
            package_path,
            package_relative_path,
            pkg_json_value,
        );

        Self { inner: Rc::new(RefCell::new(ws_package_info)) }
    }

    /// Get the package json path
    ///
    /// @returns {string} The path to package.json
    #[napi(getter)]
    pub fn package_json_path(&self) -> String {
        self.inner.borrow().package_json_path.clone()
    }

    /// Get the package path
    ///
    /// @returns {string} The path to the package
    #[napi(getter)]
    pub fn package_path(&self) -> String {
        self.inner.borrow().package_path.clone()
    }

    /// Get the relative package path
    ///
    /// @returns {string} The relative path to the package
    #[napi(getter)]
    pub fn package_relative_path(&self) -> String {
        self.inner.borrow().package_relative_path.clone()
    }

    /// Get the package
    ///
    /// @returns {Package} The package
    #[napi(getter)]
    pub fn package(&self) -> Package {
        Package { inner: self.inner.borrow().package.borrow().clone() }
    }

    /// Update the package version
    ///
    /// @param {string} newVersion - The new version to set
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn update_version(&self, new_version: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.borrow().update_version(&new_version))
    }

    /// Update a dependency version
    ///
    /// @param {string} depName - The name of the dependency to update
    /// @param {string} newVersion - The new version to set
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn update_dependency_version(
        &self,
        dep_name: String,
        new_version: String,
    ) -> NapiResult<()> {
        handle_pkg_result(self.inner.borrow().update_dependency_version(&dep_name, &new_version))
    }

    /// Apply dependency resolution across all packages
    ///
    /// @param {ResolutionResult} resolution - The resolution result to apply
    /// @returns {void}
    #[napi(js_name = "applyDependencyResolution", ts_return_type = "void")]
    pub fn apply_dependency_resolution(&self, resolution: ResolutionResult) -> NapiResult<()> {
        // Extract resolved versions from JS Object into HashMap
        let mut resolved_versions = std::collections::HashMap::new();
        let prop_names = resolution.resolved_versions.get_property_names()?;
        let length = prop_names.get_array_length()?;

        for i in 0..length {
            // Get the property name as a JavaScript string first
            let js_key = prop_names.get_element::<JsString>(i)?;
            // Convert JavaScript string to Rust String
            let key = js_key.into_utf8()?.into_owned()?;
            // Get the value as a JavaScript string
            let js_value =
                resolution.resolved_versions.get_named_property::<JsString>(key.as_str())?;
            // Convert JavaScript string to Rust String
            let value = js_value.into_utf8()?.into_owned()?;
            // Store in our HashMap
            resolved_versions.insert(key, value);
        }

        // Convert updates to DependencyUpdate structs
        let updates_required = resolution
            .updates_required
            .into_iter()
            .map(|update| ws_pkg::registry::DependencyUpdate {
                package_name: update.package_name,
                dependency_name: update.dependency_name,
                current_version: update.current_version,
                new_version: update.new_version,
            })
            .collect();

        let ws_result = WsResolutionResult { resolved_versions, updates_required };

        handle_pkg_result(self.inner.borrow().apply_dependency_resolution(&ws_result))
    }

    /// Write the package.json file to disk
    ///
    /// @returns {void}
    #[napi(js_name = "writePackageJson", ts_return_type = "void")]
    pub fn write_package_json(&self) -> NapiResult<()> {
        handle_pkg_result(self.inner.borrow().write_package_json())
    }

    /// Get the package.json content
    ///
    /// @returns {Object} The package.json content
    #[napi(getter, ts_return_type = "Result<string, unknown>")]
    pub fn package_json(&self, env: Env) -> NapiResult<napi::bindgen_prelude::Object> {
        // Get the package.json content as a serde_json::Value
        let pkg_json = self.inner.borrow().pkg_json.borrow().clone();

        // Convert the serde_json::Value to a JavaScript object
        let js_value = env.to_js_value(&pkg_json)?;

        // Convert to a JavaScript object
        js_value.coerce_to_object()
    }
}

/// Parse a scoped package name with optional version and path
///
/// Handles formats like:
/// - @scope/name
/// - @scope/name@version
/// - @scope/name@version@path
/// - @scope/name:version
///
/// @param {string} pkg_name - The scoped package name to parse
/// @returns {Object | null} An object with parsed components or null if not a valid scoped package
#[napi(ts_return_type = "ScopedPackageInfo | null")]
pub fn parse_scoped_package(
    pkg_name: String,
    env: Env,
) -> NapiResult<Option<napi::bindgen_prelude::Object>> {
    // Call ws_pkg function to parse the package name
    if let Some(metadata) = package_scope_name_version(&pkg_name) {
        // Create a JavaScript object for the result
        let mut result = env.create_object()?;

        // Set properties on the object
        result.set_named_property("full", env.create_string(&metadata.full)?)?;
        result.set_named_property("name", env.create_string(&metadata.name)?)?;
        result.set_named_property("version", env.create_string(&metadata.version)?)?;

        // Handle optional path
        if let Some(path) = metadata.path {
            result.set_named_property("path", env.create_string(&path)?)?;
        } else {
            result.set_named_property("path", env.get_null()?)?;
        }

        Ok(Some(result))
    } else {
        // Return null for non-scoped packages
        Ok(None)
    }
}

#[cfg(test)]
mod package_binding_tests {
    use super::*;
    use crate::registry::dependency::DependencyRegistry;
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

    #[test]
    fn test_with_registry() {
        // Create a registry
        let registry = DependencyRegistry::new();

        // Add some dependencies to the registry
        let _ = registry.get_or_create("dep1".to_string(), "^1.0.0".to_string()).unwrap();
        let _ = registry.get_or_create("dep2".to_string(), "^2.0.0".to_string()).unwrap();

        // Create a package with dependencies from the registry
        let package = Package::with_registry(
            "test-pkg".to_string(),
            "1.0.0".to_string(),
            Some(vec![
                ("dep1".to_string(), "^1.0.0".to_string()),
                ("dep2".to_string(), "^2.0.0".to_string()),
            ]),
            &registry,
        )
        .unwrap();

        // Verify package properties
        assert_eq!(package.name(), "test-pkg");
        assert_eq!(package.version(), "1.0.0");

        // Verify dependencies
        assert_eq!(package.dependency_count(), 2);

        // Check specific dependencies
        let dep1 = package.get_dependency("dep1".to_string()).unwrap();
        let dep2 = package.get_dependency("dep2".to_string()).unwrap();

        assert_eq!(dep1.name(), "dep1");
        assert_eq!(dep1.version(), "^1.0.0");
        assert_eq!(dep2.name(), "dep2");
        assert_eq!(dep2.version(), "^2.0.0");
    }

    #[test]
    fn test_underlying_package_scope_name_version() {
        // Test successful parsing
        let result = package_scope_name_version("@scope/name@1.0.0");
        assert!(result.is_some());
        let metadata = result.unwrap();
        assert_eq!(metadata.full, "@scope/name@1.0.0");
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.path, None);

        // Test with path
        let result = package_scope_name_version("@scope/name@1.0.0@/some/path");
        assert!(result.is_some());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.path, Some("/some/path".to_string()));

        // Test colon format
        let result = package_scope_name_version("@scope/name:1.0.0");
        assert!(result.is_some());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");

        // Test non-scoped package - should return None
        let result = package_scope_name_version("regular-package@1.0.0");
        assert!(result.is_none());
    }
}
