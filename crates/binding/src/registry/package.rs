//! JavaScript bindings for PackageRegistry.

use crate::errors::handle_pkg_result;
use napi::bindgen_prelude::*;
use napi::Result as NapiResult;
use napi::{Env, JsString};
use napi_derive::napi;
use std::sync::Arc;
use ws_pkg::{LocalRegistry, NpmRegistry, PackageRegistry as WsPackageRegistry, RegistryAuth};

/// JavaScript binding for registry types
#[napi]
pub enum RegistryType {
    /// npm registry
    Npm,
    /// GitHub packages registry
    GitHub,
    /// Custom registry
    Custom,
}

/// JavaScript binding for registry authentication
#[napi(object)]
pub struct RegistryAuthConfig {
    /// Auth token
    pub token: String,
    /// Token type (bearer, basic, etc)
    pub token_type: String,
    /// Whether to always use this auth
    pub always: bool,
}

impl From<RegistryAuthConfig> for RegistryAuth {
    fn from(auth: RegistryAuthConfig) -> Self {
        Self { token: auth.token, token_type: auth.token_type, always: auth.always }
    }
}

/// JavaScript binding for package registry interface
#[napi]
pub struct PackageRegistry {
    pub(crate) inner: Arc<dyn WsPackageRegistry + Send + Sync>,
}

#[napi]
impl PackageRegistry {
    /// Create a new npm registry
    ///
    /// @param {string} baseUrl - The base URL for the npm registry
    /// @returns {PackageRegistry} A new npm registry
    #[napi(factory)]
    pub fn create_npm_registry(base_url: String) -> Self {
        let registry = NpmRegistry::new(&base_url);
        Self { inner: Arc::new(registry) }
    }

    /// Create a new local registry (for testing)
    ///
    /// @returns {PackageRegistry} A new local registry
    #[napi(factory)]
    pub fn create_local_registry() -> Self {
        let registry = LocalRegistry::new();
        Self { inner: Arc::new(registry) }
    }

    /// Get the latest version of a package
    ///
    /// @param {string} packageName - The name of the package
    /// @returns {string | null} The latest version, or null if not found
    #[napi]
    pub fn get_latest_version(&self, package_name: String) -> NapiResult<Option<String>> {
        handle_pkg_result(self.inner.get_latest_version(&package_name))
    }

    /// Get all available versions of a package
    ///
    /// @param {string} packageName - The name of the package
    /// @returns {string[]} Array of available versions
    #[napi]
    pub fn get_all_versions(&self, package_name: String) -> NapiResult<Vec<String>> {
        handle_pkg_result(self.inner.get_all_versions(&package_name))
    }

    /// Get metadata about a package
    ///
    /// @param {string} packageName - The name of the package
    /// @param {string} version - The version to get info for
    /// @returns {Object} Package metadata
    #[napi]
    pub fn get_package_info(
        &self,
        package_name: String,
        version: String,
        env: Env,
    ) -> NapiResult<Object> {
        // Get the package info as a serde_json::Value
        let info = handle_pkg_result(self.inner.get_package_info(&package_name, &version))?;

        // Convert to JavaScript object
        let js_value = env.to_js_value(&info)?;
        js_value.coerce_to_object()
    }

    /// Set authentication for the registry
    ///
    /// @param {RegistryAuthConfig} auth - The authentication configuration
    /// @returns {void}
    #[napi]
    pub fn set_auth(&mut self, auth: RegistryAuthConfig) -> NapiResult<()> {
        // We need to downcast the Arc to the specific registry type
        // This only works for NpmRegistry currently
        if let Some(npm_registry) =
            Arc::get_mut(&mut self.inner).and_then(|r| r.as_any_mut().downcast_mut::<NpmRegistry>())
        {
            npm_registry.set_auth(&auth.token, &auth.token_type);
            Ok(())
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "Registry doesn't support authentication or is locked by multiple references",
            ))
        }
    }

    /// Set the user agent string
    ///
    /// @param {string} userAgent - The user agent string
    /// @returns {void}
    #[napi]
    pub fn set_user_agent(&mut self, user_agent: String) -> NapiResult<()> {
        // We need to downcast the Arc to the specific registry type
        // This only works for NpmRegistry currently
        if let Some(npm_registry) =
            Arc::get_mut(&mut self.inner).and_then(|r| r.as_any_mut().downcast_mut::<NpmRegistry>())
        {
            npm_registry.set_user_agent(&user_agent);
            Ok(())
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "Registry doesn't support setting user agent or is locked by multiple references",
            ))
        }
    }

    /// Clear the registry cache
    ///
    /// @returns {void}
    #[napi]
    pub fn clear_cache(&mut self) -> NapiResult<()> {
        // We need to downcast the Arc to the specific registry type
        // This only works for NpmRegistry currently
        if let Some(npm_registry) =
            Arc::get_mut(&mut self.inner).and_then(|r| r.as_any_mut().downcast_mut::<NpmRegistry>())
        {
            npm_registry.clear_cache();
            Ok(())
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "Registry doesn't support clearing cache or is locked by multiple references",
            ))
        }
    }

    /// Add a package to a local registry (only works with local registries)
    ///
    /// @param {string} name - The package name
    /// @param {string[]} versions - Array of versions to add
    /// @returns {void}
    #[napi]
    pub fn add_package(&self, name: String, versions: Vec<String>) -> NapiResult<()> {
        // We need to downcast to a LocalRegistry
        if let Some(local_registry) = self.inner.as_any().downcast_ref::<LocalRegistry>() {
            // Convert Vec<String> to Vec<&str>
            let versions_refs: Vec<&str> = versions.iter().map(|s| s.as_str()).collect();
            handle_pkg_result(local_registry.add_package(&name, versions_refs))
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "This method only works with local registries",
            ))
        }
    }

    /// Set dependencies for a specific package version in a local registry
    ///
    /// @param {string} name - The package name
    /// @param {string} version - The version
    /// @param {Object} dependencies - Map of dependency names to versions
    /// @returns {void}
    #[napi]
    pub fn set_dependencies(
        &self,
        name: String,
        version: String,
        dependencies: Object,
    ) -> NapiResult<()> {
        // We need to downcast to a LocalRegistry
        if let Some(local_registry) = self.inner.as_any().downcast_ref::<LocalRegistry>() {
            // Convert JS object to HashMap<String, String>
            let mut deps_map = std::collections::HashMap::new();
            let prop_names = dependencies.get_property_names()?;
            let length = prop_names.get_array_length()?;

            for i in 0..length {
                // Get the property name
                let js_key = prop_names.get_element::<JsString>(i)?;
                let key = js_key.into_utf8()?.into_owned()?;

                // Get the value
                let js_value = dependencies.get_named_property::<JsString>(key.as_str())?;
                let value = js_value.into_utf8()?.into_owned()?;

                // Add to map
                deps_map.insert(key, value);
            }

            handle_pkg_result(local_registry.set_dependencies(&name, &version, &deps_map))
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "This method only works with local registries",
            ))
        }
    }

    /// Get all packages in a local registry
    ///
    /// @returns {string[]} Array of all package names
    #[napi]
    pub fn get_all_packages(&self) -> NapiResult<Vec<String>> {
        // We need to downcast to a LocalRegistry
        if let Some(local_registry) = self.inner.as_any().downcast_ref::<LocalRegistry>() {
            Ok(local_registry.get_all_packages())
        } else {
            Err(napi::Error::new(
                napi::Status::GenericFailure,
                "This method only works with local registries",
            ))
        }
    }
}

#[cfg(test)]
mod package_registry_binding_tests {
    use super::*;

    #[test]
    fn test_registry_type_enum() {
        let npm = RegistryType::Npm;
        let github = RegistryType::GitHub;
        let custom = RegistryType::Custom;

        assert!(matches!(npm, RegistryType::Npm));
        assert!(matches!(github, RegistryType::GitHub));
        assert!(matches!(custom, RegistryType::Custom));
    }

    #[test]
    fn test_registry_auth_config() {
        let auth = RegistryAuthConfig {
            token: "my-token".to_string(),
            token_type: "Bearer".to_string(),
            always: false,
        };

        let ws_auth = RegistryAuth::from(auth);
        assert_eq!(ws_auth.token, "my-token");
        assert_eq!(ws_auth.token_type, "Bearer");
        assert!(!ws_auth.always);
    }

    #[test]
    fn test_create_npm_registry() {
        let registry =
            PackageRegistry::create_npm_registry("https://registry.npmjs.org".to_string());
        assert!(registry.inner.as_any().downcast_ref::<NpmRegistry>().is_some());
    }

    #[test]
    fn test_create_local_registry() {
        let registry = PackageRegistry::create_local_registry();
        assert!(registry.inner.as_any().downcast_ref::<LocalRegistry>().is_some());
    }

    #[test]
    fn test_local_registry_operations() {
        let registry = PackageRegistry::create_local_registry();

        // Add a package
        let result = registry
            .add_package("test-pkg".to_string(), vec!["1.0.0".to_string(), "1.1.0".to_string()]);
        assert!(result.is_ok());

        // Get all packages
        let packages = registry.get_all_packages().unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0], "test-pkg");

        // Get all versions
        let versions = registry.get_all_versions("test-pkg".to_string()).unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"1.1.0".to_string()));

        // Get latest version
        let latest = registry.get_latest_version("test-pkg".to_string()).unwrap();
        assert_eq!(latest, Some("1.1.0".to_string()));
    }
}
