//! JavaScript bindings for registry manager.

use crate::errors::handle_pkg_result;
use crate::registry::package::{PackageRegistry, RegistryAuthConfig, RegistryType};
use napi::bindgen_prelude::*;
use napi::Env;
use napi::Result as NapiResult;
use napi_derive::napi;
use std::sync::Arc;
use ws_pkg::{
    NpmRegistry, PackageRegistry as WsPackageRegistry, RegistryAuth,
    RegistryManager as WsRegistryManager, RegistryType as WsRegistryType,
};

/// JavaScript binding for registry manager
#[napi]
pub struct RegistryManager {
    inner: WsRegistryManager,
}

#[napi]
#[allow(clippy::new_without_default)]
impl RegistryManager {
    /// Create a new registry manager
    ///
    /// @returns {RegistryManager} A new registry manager
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: WsRegistryManager::new() }
    }

    /// Add a registry
    ///
    /// @param {string} url - The registry URL
    /// @param {RegistryType} registryType - The type of registry
    /// @param {string} [clientName] - The client name for custom registries
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn add_registry(
        &mut self,
        url: String,
        registry_type: RegistryType,
        client_name: Option<String>,
    ) -> NapiResult<()> {
        let ws_registry_type = match registry_type {
            RegistryType::Npm => WsRegistryType::Npm,
            RegistryType::GitHub => WsRegistryType::GitHub,
            RegistryType::Custom => {
                WsRegistryType::Custom(client_name.unwrap_or_else(|| "custom-client".to_string()))
            }
        };

        handle_pkg_result(self.inner.add_registry(&url, ws_registry_type))
    }

    /// Set authentication for a registry
    ///
    /// @param {string} registryUrl - The registry URL
    /// @param {RegistryAuthConfig} auth - The authentication configuration
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn set_auth(&mut self, registry_url: String, auth: RegistryAuthConfig) -> NapiResult<()> {
        let ws_auth =
            RegistryAuth { token: auth.token, token_type: auth.token_type, always: auth.always };

        handle_pkg_result(self.inner.set_auth(&registry_url, ws_auth))
    }

    /// Associate a scope with a specific registry
    ///
    /// @param {string} scope - The package scope (with or without @ prefix)
    /// @param {string} registryUrl - The registry URL
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn associate_scope(&mut self, scope: String, registry_url: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.associate_scope(&scope, &registry_url))
    }

    /// Set the default registry
    ///
    /// @param {string} registryUrl - The registry URL
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn set_default_registry(&mut self, registry_url: String) -> NapiResult<()> {
        handle_pkg_result(self.inner.set_default_registry(&registry_url))
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

    /// Load configuration from .npmrc file
    ///
    /// @param {string} [npmrcPath] - Optional path to .npmrc file
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn load_from_npmrc(&mut self, npmrc_path: Option<String>) -> NapiResult<()> {
        handle_pkg_result(self.inner.load_from_npmrc(npmrc_path.as_deref()))
    }

    /// Get the default registry URL
    ///
    /// @returns {string} The default registry URL
    #[napi(getter)]
    pub fn default_registry(&self) -> String {
        self.inner.default_registry().to_string()
    }

    /// Check if a scope is associated with a registry
    ///
    /// @param {string} scope - The package scope
    /// @returns {boolean} True if the scope is associated with a registry
    #[napi]
    pub fn has_scope(&self, scope: String) -> bool {
        self.inner.has_scope(&scope)
    }

    /// Get the registry URL associated with a scope
    ///
    /// @param {string} scope - The package scope
    /// @returns {string | null} The registry URL, or null if not found
    #[napi]
    pub fn get_registry_for_scope(&self, scope: String) -> Option<String> {
        self.inner.get_registry_for_scope(&scope).map(String::from)
    }

    /// Get all registry URLs
    ///
    /// @returns {string[]} Array of all registry URLs
    #[napi]
    pub fn registry_urls(&self) -> Vec<String> {
        self.inner.registry_urls().into_iter().map(String::from).collect()
    }

    /// Add a registry instance directly
    ///
    /// @param {string} url - The registry URL
    /// @param {PackageRegistry} registry - The registry instance
    /// @returns {void}
    #[napi(ts_return_type = "void")]
    pub fn add_registry_instance(
        &mut self,
        url: String,
        registry: &PackageRegistry,
    ) -> NapiResult<()> {
        // Since we can't directly use the registry.inner because of Arc typing issues,
        // and we can't access private fields from other crates,
        // we'll create a new registry instance with the URL

        // Create a new registry based on the provided URL
        let new_registry: Arc<dyn WsPackageRegistry + Send + Sync> =
            if registry.inner.as_any().downcast_ref::<NpmRegistry>().is_some() {
                // Create a new npm registry with the given URL
                Arc::new(NpmRegistry::new(&url))
            } else if registry.inner.as_any().downcast_ref::<ws_pkg::LocalRegistry>().is_some() {
                // Create a new local registry
                Arc::new(ws_pkg::LocalRegistry::default())
            } else {
                return Err(napi::Error::new(
                    napi::Status::GenericFailure,
                    "Unknown registry type",
                ));
            };

        handle_pkg_result(self.inner.add_registry_instance(&url, new_registry))
    }
}

#[cfg(test)]
mod registry_manager_binding_tests {
    use super::*;

    #[test]
    fn test_registry_manager_creation() {
        let manager = RegistryManager::new();
        // Default registry should be npm
        assert_eq!(manager.default_registry(), "https://registry.npmjs.org");
    }

    #[test]
    fn test_add_registry() {
        let mut manager = RegistryManager::new();

        // Add a custom registry
        let result = manager.add_registry(
            "https://custom-registry.com".to_string(),
            RegistryType::Custom,
            Some("test-client".to_string()),
        );
        assert!(result.is_ok());

        // Verify it was added
        let urls = manager.registry_urls();
        assert!(urls.contains(&"https://custom-registry.com".to_string()));
    }

    #[test]
    fn test_associate_scope() {
        let mut manager = RegistryManager::new();

        // Add a registry first
        let result = manager.add_registry(
            "https://custom-registry.com".to_string(),
            RegistryType::Custom,
            Some("test-client".to_string()),
        );
        assert!(result.is_ok());

        // Associate a scope
        let result =
            manager.associate_scope("@test".to_string(), "https://custom-registry.com".to_string());
        assert!(result.is_ok());

        // Verify the association
        assert!(manager.has_scope("@test".to_string()));
        assert_eq!(
            manager.get_registry_for_scope("@test".to_string()),
            Some("https://custom-registry.com".to_string())
        );
    }

    #[test]
    fn test_set_default_registry() {
        let mut manager = RegistryManager::new();

        // Add a registry
        let result = manager.add_registry(
            "https://custom-registry.com".to_string(),
            RegistryType::Custom,
            Some("test-client".to_string()),
        );
        assert!(result.is_ok());

        // Set as default
        let result = manager.set_default_registry("https://custom-registry.com".to_string());
        assert!(result.is_ok());

        // Verify the default was set
        assert_eq!(manager.default_registry(), "https://custom-registry.com");
    }

    #[test]
    fn test_set_auth() {
        let mut manager = RegistryManager::new();

        // Add a registry
        let result = manager.add_registry(
            "https://custom-registry.com".to_string(),
            RegistryType::Custom,
            Some("test-client".to_string()),
        );
        assert!(result.is_ok());

        // Set auth
        let auth = RegistryAuthConfig {
            token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
            always: false,
        };
        let result = manager.set_auth("https://custom-registry.com".to_string(), auth);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod registry_integration_tests {
    use crate::{DependencyRegistry, PackageRegistry, RegistryManager};

    #[test]
    fn test_local_registry_with_dependency_registry() {
        // Create a local registry
        let registry = PackageRegistry::create_local_registry();

        // Add some packages
        registry
            .add_package("test-pkg".to_string(), vec!["1.0.0".to_string(), "1.1.0".to_string()])
            .unwrap();

        // Create a dependency registry
        let dep_registry = DependencyRegistry::new();

        // Create a dependency that would use our local registry
        let dependency =
            dep_registry.get_or_create("test-pkg".to_string(), "^1.0.0".to_string()).unwrap();

        assert_eq!(dependency.name(), "test-pkg");
        assert_eq!(dependency.version(), "^1.0.0");
    }

    #[test]
    fn test_registry_manager_with_package_registry() {
        // Create a registry manager
        let manager = RegistryManager::new();

        // Create a local registry
        let _ = PackageRegistry::create_local_registry();

        // This is a simplified test that just ensures the types can be created
        assert_eq!(manager.default_registry(), "https://registry.npmjs.org");
    }
}
