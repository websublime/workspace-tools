use std::{collections::HashMap, sync::Arc};

use crate::{NpmRegistry, PackageRegistry, PackageRegistryError, RegistryError};

/// Type of registry
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
#[derive(Clone)]
pub struct RegistryManager {
    registries: HashMap<String, Arc<dyn PackageRegistry + Send + Sync>>,
    scopes: HashMap<String, String>,
    default_registry: String,
    auth_configs: HashMap<String, RegistryAuth>,
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryManager {
    /// Create a new registry manager with default npm registry
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

    // Add a method to directly add a registry instance
    pub fn add_registry_instance(
        &mut self,
        url: &str,
        registry: Arc<dyn PackageRegistry + Send + Sync>,
    ) -> &Self {
        self.registries.insert(url.to_string(), registry);
        self
    }

    /// Set authentication for a registry
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
    pub fn set_default_registry(&mut self, registry_url: &str) -> Result<&Self, RegistryError> {
        if !self.registries.contains_key(registry_url) {
            return Err(RegistryError::UrlNotFound(registry_url.to_string()));
        }

        self.default_registry = registry_url.to_string();
        Ok(self)
    }

    /// Get the appropriate registry for a package
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
    pub fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_latest_version(package_name)
    }

    /// Get all available versions of a package
    pub fn get_all_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_all_versions(package_name)
    }

    /// Get metadata about a package
    pub fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<serde_json::Value, PackageRegistryError> {
        let registry = self.get_registry_for_package(package_name);
        registry.get_package_info(package_name, version)
    }

    /// Load configuration from .npmrc file
    pub fn load_from_npmrc(&mut self, npmrc_path: Option<&str>) -> Result<&Self, RegistryError> {
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

        let content = std::fs::read_to_string(&path).map_err(|e| RegistryError::NpmRcFailure {
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

                // Handle registry scopes
                if let Some(registry_key) = key.strip_suffix(":registry") {
                    if registry_key.starts_with('@') {
                        let scope = registry_key.trim_start_matches('@');
                        self.add_registry(value, RegistryType::Npm);
                        self.associate_scope(scope, value)?;
                    } else if registry_key == "registry" {
                        self.add_registry(value, RegistryType::Npm);
                        self.set_default_registry(value)?;
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
    pub fn default_registry(&self) -> &str {
        &self.default_registry
    }

    /// Check if a scope is associated with a registry
    pub fn has_scope(&self, scope: &str) -> bool {
        let clean_scope = scope.trim_start_matches('@');
        self.scopes.contains_key(clean_scope)
    }

    /// Get the registry URL associated with a scope
    pub fn get_registry_for_scope(&self, scope: &str) -> Option<&str> {
        let clean_scope = scope.trim_start_matches('@');
        self.scopes.get(clean_scope).map(std::string::String::as_str)
    }

    /// Get all registry URLs
    pub fn registry_urls(&self) -> Vec<&str> {
        self.registries.keys().map(std::string::String::as_str).collect()
    }

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
