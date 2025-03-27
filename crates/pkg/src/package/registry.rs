use reqwest::blocking::{Client, RequestBuilder};
use serde_json::Value;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{CacheEntry, PackageRegistryError};

pub trait PackageRegistry {
    /// Get the latest version of a package
    fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError>;

    /// Get all available versions of a package
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError>;

    /// Get metadata about a package
    fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError>;

    /// Get the registry as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get the registry as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct NpmRegistry {
    base_url: String,
    client: Client,
    user_agent: String,
    cache_ttl: Duration,
    versions_cache: Arc<Mutex<HashMap<String, CacheEntry<Vec<String>>>>>,
    latest_version_cache: Arc<Mutex<HashMap<String, CacheEntry<Option<String>>>>>,
    auth_token: Option<String>,
    auth_type: Option<String>,
}

impl Default for NpmRegistry {
    fn default() -> Self {
        Self::new("https://registry.npmjs.org")
    }
}

impl PackageRegistry for NpmRegistry {
    fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        // Check cache first
        {
            if let Ok(cache) = self.latest_version_cache.lock() {
                if let Some(cache_entry) = cache.get(package_name) {
                    if cache_entry.is_valid(self.cache_ttl) {
                        return Ok(cache_entry.get());
                    }
                }
            }
        }

        let url = format!("{}/latest", self.package_url(package_name));

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let data: Value = response.json().map_err(PackageRegistryError::FetchFailure)?;

        let version = data.get("version").and_then(|v| v.as_str()).map(ToString::to_string);

        // Cache the result
        if let Ok(mut cache) = self.latest_version_cache.lock() {
            cache.insert(package_name.to_string(), CacheEntry::new(version.clone()));
        }

        Ok(version)
    }

    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
        // Check cache first
        {
            if let Ok(cache) = self.versions_cache.lock() {
                if let Some(cache_entry) = cache.get(package_name) {
                    if cache_entry.is_valid(self.cache_ttl) {
                        return Ok(cache_entry.get());
                    }
                }
            }
        }

        let url = self.package_url(package_name);

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let data: Value = response.json().map_err(PackageRegistryError::JsonParseFailure)?;

        let versions = data
            .get("versions")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect::<Vec<String>>())
            .unwrap_or_default();

        // Cache the result
        if let Ok(mut cache) = self.versions_cache.lock() {
            cache.insert(package_name.to_string(), CacheEntry::new(versions.clone()));
        }

        Ok(versions)
    }

    fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError> {
        let url = format!("{}/{}", self.package_url(package_name), version);

        let response =
            self.build_request(&url).send().map_err(PackageRegistryError::FetchFailure)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            });
        }

        response.json().map_err(PackageRegistryError::JsonParseFailure)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NpmRegistry {
    /// Create a new npm registry client with the given base URL
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
            user_agent: "ws-pkg/0.1.0".to_string(),
            cache_ttl: Duration::from_secs(300), // 5 minutes default
            versions_cache: Arc::new(Mutex::new(HashMap::new())),
            latest_version_cache: Arc::new(Mutex::new(HashMap::new())),
            auth_token: None,
            auth_type: None,
        }
    }

    /// Set the user agent string
    pub fn set_user_agent(&mut self, user_agent: &str) -> &mut Self {
        self.user_agent = user_agent.to_string();
        self
    }

    /// Set authentication
    pub fn set_auth(&mut self, token: &str, auth_type: &str) -> &mut Self {
        self.auth_token = Some(token.to_string());
        self.auth_type = Some(auth_type.to_string());
        self
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: Duration) -> &mut Self {
        self.cache_ttl = ttl;
        self
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.versions_cache = Arc::new(Mutex::new(HashMap::new()));
        self.latest_version_cache = Arc::new(Mutex::new(HashMap::new()));
    }

    /// Build a URL for the package
    fn package_url(&self, package_name: &str) -> String {
        // Handle scoped packages correctly
        let encoded_name = if package_name.starts_with('@') {
            // URL encode the @ character and the / character
            package_name.replace('@', "%40").replace('/', "%2F")
        } else {
            package_name.to_string()
        };

        format!("{}/{}", self.base_url, encoded_name)
    }

    /// Build a request with appropriate headers
    fn build_request(&self, url: &str) -> RequestBuilder {
        let mut builder = self.client.get(url).header("User-Agent", &self.user_agent);

        // Add auth if configured
        if let (Some(token), Some(auth_type)) = (&self.auth_token, &self.auth_type) {
            let auth_header = if auth_type.eq_ignore_ascii_case("bearer") {
                format!("Bearer {token}")
            } else if auth_type.eq_ignore_ascii_case("basic") {
                format!("Basic {token}")
            } else {
                token.clone()
            };

            builder = builder.header("Authorization", auth_header);
        }

        builder
    }
}
