//! Package registry interfaces for querying package repositories.

use crate::error::{PkgError, Result};
use serde_json::Value;

/// Interface to an external package registry that provides version information
pub trait PackageRegistry {
    /// Get the latest version of a package
    fn get_latest_version(&self, package_name: &str) -> Result<Option<String>>;

    /// Get all available versions of a package
    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>>;

    /// Get metadata about a package
    fn get_package_info(&self, package_name: &str, version: &str) -> Result<Value>;
}

/// Registry client that fetches package information from npm
pub struct NpmRegistry {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Default for NpmRegistry {
    fn default() -> Self {
        Self::new("https://registry.npmjs.org")
    }
}

impl NpmRegistry {
    /// Create a new npm registry client with the given base URL
    pub fn new(base_url: &str) -> Self {
        Self { base_url: base_url.to_string(), client: reqwest::blocking::Client::new() }
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
}

impl PackageRegistry for NpmRegistry {
    fn get_latest_version(&self, package_name: &str) -> Result<Option<String>> {
        let url = format!("{}/latest", self.package_url(package_name));

        let response = self.client.get(&url).send().map_err(|e| PkgError::Other {
            message: format!("Failed to fetch from npm registry: {e}"),
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let data: Value = response.json().map_err(|e| PkgError::Other {
            message: format!("Failed to parse npm registry response: {e}"),
        })?;

        Ok(data.get("version").and_then(|v| v.as_str()).map(ToString::to_string))
    }

    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let url = self.package_url(package_name);

        let response = self.client.get(&url).send().map_err(|e| PkgError::Other {
            message: format!("Failed to fetch from npm registry: {e}"),
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let data: Value = response.json().map_err(|e| PkgError::Other {
            message: format!("Failed to parse npm registry response: {e}"),
        })?;

        let versions = data
            .get("versions")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(versions)
    }

    fn get_package_info(&self, package_name: &str, version: &str) -> Result<Value> {
        let url = format!("{}/{}", self.package_url(package_name), version);

        let response = self.client.get(&url).send().map_err(|e| PkgError::Other {
            message: format!("Failed to fetch from npm registry: {e}"),
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PkgError::Other {
                message: format!("Package {package_name}@{version} not found"),
            });
        }

        response.json().map_err(|e| PkgError::Other {
            message: format!("Failed to parse npm registry response: {e}"),
        })
    }
}
