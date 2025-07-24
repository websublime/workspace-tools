//! Registry manager for coordinating multiple registries
//!
//! This module provides a manager for coordinating access to multiple package registries,
//! handling scopes, authentication, and default registry selection.

use crate::{
    errors::{PackageRegistryError, RegistryError},
    NpmRegistry, PackageRegistry,
};
use core::fmt;
use std::{collections::HashMap, sync::Arc};

/// Type of registry
///
/// Specifies the type of registry being managed, which affects how
/// it's configured and queried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryType {
    /// npm registry
    Npm,
    /// GitHub packages registry
    GitHub,
    /// Custom registry
    Custom(String),
}

/// Authentication config for registries
///
/// Stores authentication information for accessing protected registries.
#[derive(Debug, Clone)]
pub struct RegistryAuth {
    /// Auth token
    pub token: String,
    /// Token type (bearer, basic, etc)
    pub token_type: String,
    /// Whether to always use this auth
    pub always: bool,
}

/// Registry manager to handle multiple registries
///
/// Manages multiple package registries, including scope associations,
/// authentication, and request routing.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{RegistryManager, RegistryType, RegistryAuth};
///
/// // Create a new manager (automatically includes default npm registry)
/// let mut manager = RegistryManager::new();
///
/// // Add additional registries
/// manager.add_registry("https://npm.pkg.github.com", RegistryType::GitHub);
///
/// // Associate scopes with specific registries
/// manager.associate_scope("@my-org", "https://npm.pkg.github.com").unwrap();
///
/// // Add authentication
/// let auth = RegistryAuth {
///     token: "my-token".to_string(),
///     token_type: "Bearer".to_string(),
///     always: false,
/// };
/// manager.set_auth("https://npm.pkg.github.com", auth).unwrap();
/// ```
#[derive(Clone)]
pub struct RegistryManager {
    registries: HashMap<String, Arc<dyn PackageRegistry + Send + Sync>>,
    scopes: HashMap<String, String>,
    default_registry: String,
    auth_configs: HashMap<String, RegistryAuth>,
}

/// Default implementation for RegistryManager
///
/// Creates a new RegistryManager with the default configuration.
impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug implementation for RegistryManager
///
/// Provides a custom debug representation that includes key information
/// without exposing sensitive data like auth tokens.
impl fmt::Debug for RegistryManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistryManager")
            .field("default_registry", &self.default_registry)
            .field("registry_urls", &self.registry_urls())
            .field("scopes", &self.scopes)
            .field("auth_configs_count", &self.auth_configs.len())
            // Indicate that we're intentionally not including all fields
            .finish_non_exhaustive()
    }
}

impl RegistryManager {
    /// Create a new registry manager with default npm registry
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::RegistryManager;
    ///
    /// let manager = RegistryManager::new();
    /// assert_eq!(manager.default_registry(), "https://registry.npmjs.org");
    /// ```
    pub fn new() -> Self {
        let mut registries = HashMap::new();
        let default_registry = "https://registry.npmjs.org".to_string();

        registries.insert(
            default_registry.clone(),
            Arc::new(NpmRegistry::new(&default_registry)) as Arc<dyn PackageRegistry + Send + Sync>,
        );

        Self { registries, scopes: HashMap::new(), default_registry, auth_configs: HashMap::new() }
    }

    /// Add a registry
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the registry
    /// * `registry_type` - Type of the registry
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{RegistryManager, RegistryType};
    ///
    /// let mut manager = RegistryManager::new();
    ///
    /// // Add npm registry
    /// manager.add_registry("https://registry.npmjs.org", RegistryType::Npm);
    ///
    /// // Add GitHub registry
    /// manager.add_registry("https://npm.pkg.github.com", RegistryType::GitHub);
    ///
    /// // Add custom registry
    /// manager.add_registry("https://my-custom-registry.com", RegistryType::Custom("MyClient/1.0".to_string()));
    /// ```
    pub fn add_registry(&mut self, url: &str, registry_type: RegistryType) -> &Self {
        let registry: Arc<dyn PackageRegistry + Send + Sync> = match registry_type {
            RegistryType::Npm => Arc::new(NpmRegistry::new(url)),
            RegistryType::GitHub => {
                let mut npm = NpmRegistry::new(url);
                npm.set_user_agent("GitHub Package Registry Client");
                Arc::new(npm)
            }
            RegistryType::Custom(client_name) => {
                let mut npm = NpmRegistry::new(url);
                npm.set_user_agent(&client_name);
                Arc::new(npm)
            }
        };

        self.registries.insert(url.to_string(), registry);
        self
    }

    /// Add a custom registry instance
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the registry
    /// * `registry` - Custom registry implementation
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining
    pub fn add_registry_instance(
        &mut self,
        url: &str,
        registry: Arc<dyn PackageRegistry + Send + Sync>,
    ) -> &Self {
        self.registries.insert(url.to_string(), registry);
        self
    }

    /// Set authentication for a registry
    ///
    /// # Arguments
    ///
    /// * `registry_url` - URL of the registry to authenticate with
    /// * `auth` - Authentication configuration
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining, or an error if the registry doesn't exist
    /// or doesn't support authentication
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::UrlNotFound` if the registry URL doesn't exist,
    /// or `RegistryError::UrlNotSupported` if the registry doesn't support
    pub fn set_auth(
        &mut self,
        registry_url: &str,
        auth: RegistryAuth,
    ) -> Result<&Self, RegistryError> {
        if let Some(registry) = self.registries.get_mut(registry_url) {
            if let Some(npm_registry) =
                Arc::get_mut(registry).and_then(|r| r.as_any_mut().downcast_mut::<NpmRegistry>())
            {
                npm_registry.set_auth(&auth.token, &auth.token_type);
                self.auth_configs.insert(registry_url.to_string(), auth);
                return Ok(self);
            }

            return Err(RegistryError::UrlNotSupported(registry_url.to_string()));
        }

        Err(RegistryError::UrlNotFound(registry_url.to_string()))
    }

    /// Associate a scope with a specific registry
    ///
    /// # Arguments
    ///
    /// * `scope` - Package scope (with or without @ prefix)
    /// * `registry_url` - URL of the registry to use for this scope
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining, or an error if the registry doesn't exist
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::UrlNotFound` if the registry URL doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::RegistryManager;
    ///
    /// let mut manager = RegistryManager::new();
    ///
    /// // Associate a scope with default registry
    /// manager.associate_scope("@my-org", "https://registry.npmjs.org").unwrap();
    ///
    /// // Check that the scope is registered
    /// assert!(manager.has_scope("@my-org"));
    /// ```
    pub fn associate_scope(
        &mut self,
        scope: &str,
        registry_url: &str,
    ) -> Result<&Self, RegistryError> {
        if !self.registries.contains_key(registry_url) {
            return Err(RegistryError::UrlNotFound(registry_url.to_string()));
        }

        let clean_scope = scope.trim_start_matches('@');
        self.scopes.insert(clean_scope.to_string(), registry_url.to_string());
        Ok(self)
    }

    /// Set the default registry
    ///
    /// # Arguments
    ///
    /// * `registry_url` - URL of the registry to set as default
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining, or an error if the registry doesn't exist
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::UrlNotFound` if the registry URL doesn't exist
    pub fn set_default_registry(&mut self, registry_url: &str) -> Result<&Self, RegistryError> {
        if !self.registries.contains_key(registry_url) {
            return Err(RegistryError::UrlNotFound(registry_url.to_string()));
        }

        self.default_registry = registry_url.to_string();
        Ok(self)
    }

    /// Get the appropriate registry for a package
    ///
    /// Determines the correct registry to use based on package scope,
    /// falling back to the default registry if the package isn't scoped.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    ///
    /// # Returns
    ///
    /// Reference to the appropriate registry
    #[must_use]
    pub fn get_registry_for_package(
        &self,
        package_name: &str,
    ) -> Arc<dyn PackageRegistry + Send + Sync> {
        // Check if this is a scoped package
        if let Some(scope) = package_name.strip_prefix('@') {
            if let Some(scope_name) = scope.split('/').next() {
                if let Some(registry_url) = self.scopes.get(scope_name) {
                    if let Some(registry) = self.registries.get(registry_url) {
                        return Arc::clone(registry);
                    }
                }
            }
        }

        // Fall back to default registry
        Arc::clone(&self.registries[&self.default_registry])
    }

    /// Get the latest version of a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    ///
    /// # Returns
    ///
    /// Latest version string, or `None` if the package doesn't exist,
    /// or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    pub async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_latest_version(package_name).await
    }

    /// Get all available versions of a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    ///
    /// # Returns
    ///
    /// List of available versions, or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The response cannot be parsed as JSON
    pub async fn get_all_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_all_versions(package_name).await
    }

    /// Get metadata about a package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `version` - Specific version to retrieve
    ///
    /// # Returns
    ///
    /// Package metadata as JSON, or a `PackageRegistryError` if the query fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Network request to the registry fails
    /// - The specified package or version is not found
    /// - The response cannot be parsed as JSON
    pub async fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<serde_json::Value, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_package_info(package_name, version).await
    }

    /// Load configuration from .npmrc file
    ///
    /// Parses a .npmrc file and configures the registry manager accordingly,
    /// including registries, scopes, and authentication.
    ///
    /// # Arguments
    ///
    /// * `npmrc_path` - Optional path to .npmrc file. If None, looks in user's home directory.
    ///
    /// # Returns
    ///
    /// Reference to self for method chaining, or an error if loading fails
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::NpmRcFailure` if reading the .npmrc file fails
    pub async fn load_from_npmrc(&mut self, npmrc_path: Option<&str>) -> Result<&Self, RegistryError> {
        let path = if let Some(path_str) = npmrc_path {
            std::path::PathBuf::from(path_str)
        } else {
            // Try to find the user's home directory using environment variables
            let home =
                std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();

            if home.is_empty() {
                return Ok(self); // Can't locate home directory
            }

            std::path::PathBuf::from(home).join(".npmrc")
        };

        if !path.exists() {
            return Ok(self);
        }

        let content = tokio::fs::read_to_string(&path).await.map_err(|e| RegistryError::NpmRcFailure {
            path: path.display().to_string(),
            error: e,
        })?;

        // Parse .npmrc file
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Handle exact match for the default registry
                if key == "registry" {
                    self.add_registry(value, RegistryType::Npm);
                    self.set_default_registry(value)?;
                    continue;
                }

                // Handle registry scopes
                if let Some(registry_key) = key.strip_suffix(":registry") {
                    if registry_key.starts_with('@') {
                        let scope = registry_key.trim_start_matches('@');
                        self.add_registry(value, RegistryType::Npm);
                        self.associate_scope(scope, value)?;
                    }
                }

                // Handle auth tokens
                if let Some(auth_key) = key.strip_suffix(":_authToken") {
                    if let Some(registry_url) = self.resolve_registry_url(auth_key) {
                        let auth = RegistryAuth {
                            token: value.to_string(),
                            token_type: "Bearer".to_string(),
                            always: false,
                        };
                        self.add_registry(&registry_url, RegistryType::Npm);
                        self.set_auth(&registry_url, auth)?;
                    }
                }
            }
        }

        Ok(self)
    }

    /// Get the default registry URL
    ///
    /// # Returns
    ///
    /// URL of the default registry
    #[must_use]
    pub fn default_registry(&self) -> &str {
        &self.default_registry
    }

    /// Check if a scope is associated with a registry
    ///
    /// # Arguments
    ///
    /// * `scope` - Package scope (with or without @ prefix)
    ///
    /// # Returns
    ///
    /// `true` if the scope is associated with a registry, `false` otherwise
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        let clean_scope = scope.trim_start_matches('@');
        self.scopes.contains_key(clean_scope)
    }

    /// Get the registry URL associated with a scope
    ///
    /// # Arguments
    ///
    /// * `scope` - Package scope (with or without @ prefix)
    ///
    /// # Returns
    ///
    /// URL of the associated registry, or `None` if the scope isn't associated
    #[must_use]
    pub fn get_registry_for_scope(&self, scope: &str) -> Option<&str> {
        let clean_scope = scope.trim_start_matches('@');
        self.scopes.get(clean_scope).map(std::string::String::as_str)
    }

    /// Get all registry URLs
    ///
    /// # Returns
    ///
    /// List of all registered registry URLs
    #[must_use]
    pub fn registry_urls(&self) -> Vec<&str> {
        self.registries.keys().map(std::string::String::as_str).collect()
    }

    /// Resolve a registry URL from an authentication key
    ///
    /// # Arguments
    ///
    /// * `auth_key` - Authentication key from .npmrc
    ///
    /// # Returns
    ///
    /// Resolved registry URL, or `None` if resolution fails
    fn resolve_registry_url(&self, auth_key: &str) -> Option<String> {
        if auth_key.starts_with("//") {
            // This is a direct registry URL
            return Some(format!("https:{auth_key}"));
        }

        // Otherwise, look for a scope
        for (scope, registry) in &self.scopes {
            if auth_key.contains(scope) {
                return Some(registry.clone());
            }
        }

        None
    }
}
