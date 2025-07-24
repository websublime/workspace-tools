//! Local package registry implementation
//!
//! This module provides a local in-memory package registry implementation, primarily
//! useful for testing and simulating registry behavior without network calls.

use async_trait::async_trait;
use crate::external::npm_client::PackageRegistryClone;
use crate::errors::PackageRegistryError;
use super::npm_client::PackageRegistry;
use semver::Version;
use serde_json::{json, Value};
use std::{
    any::Any,
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

/// In-memory package registry implementation
///
/// This registry stores package information locally in memory, making it suitable
/// for testing or creating mocks without requiring actual registry calls.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{LocalRegistry, PackageRegistry};
/// use serde_json::json;
///
/// // Create a local registry
/// let registry = LocalRegistry::default();
///
/// // Query (will be empty until populated)
/// let versions = registry.get_all_versions("test-package").unwrap();
/// assert!(versions.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct LocalRegistry {
    packages: Arc<Mutex<HashMap<String, HashMap<String, Value>>>>,
}

impl Default for LocalRegistry {
    fn default() -> Self {
        Self { packages: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait]
impl PackageRegistry for LocalRegistry {
    async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<Option<String>, PackageRegistryError> {
        if let Ok(packages) = self.packages.lock() {
            if let Some(versions) = packages.get(package_name) {
                if versions.is_empty() {
                    return Ok(None);
                }

                // Parse versions and find latest
                let mut latest_version: Option<(Version, String)> = None;

                for version_str in versions.keys() {
                    if let Ok(version) = Version::parse(version_str) {
                        if let Some((current_latest, _)) = &latest_version {
                            if version > *current_latest {
                                latest_version = Some((version, version_str.clone()));
                            }
                        } else {
                            latest_version = Some((version, version_str.clone()));
                        }
                    }
                }

                Ok(latest_version.map(|(_, version_str)| version_str))
            } else {
                Ok(None)
            }
        } else {
            Err(PackageRegistryError::LockFailure)
        }
    }

    async fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
        let packages = self.packages.lock()?;

        if let Some(versions) = packages.get(package_name) {
            Ok(versions.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_package_info(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Value, PackageRegistryError> {
        let packages = self.packages.lock()?;

        if let Some(versions) = packages.get(package_name) {
            if let Some(package_info) = versions.get(version) {
                Ok(package_info.clone())
            } else {
                Err(PackageRegistryError::NotFound {
                    package_name: package_name.to_string(),
                    version: version.to_string(),
                })
            }
        } else {
            Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            })
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn download_package(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, PackageRegistryError> {
        // For LocalRegistry, we simulate a package download by checking if the package exists
        // and returning mock tarball data
        let packages = self.packages.lock()?;

        if let Some(versions) = packages.get(package_name) {
            if versions.contains_key(version) {
                // Return mock gzip tarball header followed by some data
                // This simulates a real tarball without actual tar content
                let mock_tarball = vec![
                    0x1f, 0x8b, 0x08, 0x00, // gzip magic header
                    0x00, 0x00, 0x00, 0x00, // timestamp
                    0x00, 0xff, // extra flags
                    // Mock tar content would follow here
                    0x70, 0x61, 0x63, 0x6b, 0x61, 0x67, 0x65, 0x2e, 0x6a, 0x73, 0x6f,
                    0x6e, // "package.json"
                ];
                Ok(mock_tarball)
            } else {
                Err(PackageRegistryError::NotFound {
                    package_name: package_name.to_string(),
                    version: version.to_string(),
                })
            }
        } else {
            Err(PackageRegistryError::NotFound {
                package_name: package_name.to_string(),
                version: version.to_string(),
            })
        }
    }

    async fn download_and_extract_package(
        &self,
        package_name: &str,
        version: &str,
        destination: &Path,
    ) -> Result<(), PackageRegistryError> {
        // For LocalRegistry, we simulate extraction by creating a basic directory structure
        use tokio::fs;

        // Check if package exists and get package info (drop lock before async operations)
        let package_info = {
            let packages = self.packages.lock()?;
            if let Some(versions) = packages.get(package_name) {
                if !versions.contains_key(version) {
                    return Err(PackageRegistryError::NotFound {
                        package_name: package_name.to_string(),
                        version: version.to_string(),
                    });
                }
                // Get the package info while we have the lock
                versions.get(version).cloned().unwrap_or_else(|| {
                    json!({
                        "name": package_name,
                        "version": version,
                        "_note": "Mock package created by LocalRegistry"
                    })
                })
            } else {
                return Err(PackageRegistryError::NotFound {
                    package_name: package_name.to_string(),
                    version: version.to_string(),
                });
            }
        }; // Lock is dropped here

        // Create destination directory
        if let Err(e) = fs::create_dir_all(destination).await {
            return Err(PackageRegistryError::DirectoryCreationFailure {
                path: destination.display().to_string(),
                source: e,
            });
        }

        // Create package subdirectory (npm packages are extracted to package/ subdirectory)
        let package_dir = destination.join("package");
        if let Err(e) = fs::create_dir_all(&package_dir).await {
            return Err(PackageRegistryError::DirectoryCreationFailure {
                path: package_dir.display().to_string(),
                source: e,
            });
        }

        // Create a mock package.json file
        let package_json_path = package_dir.join("package.json");
        let package_json_content = serde_json::to_string_pretty(&package_info).map_err(|e| {
            PackageRegistryError::ExtractionFailure {
                package_name: package_name.to_string(),
                version: version.to_string(),
                destination: destination.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;

        fs::write(&package_json_path, package_json_content).await.map_err(|e| {
            PackageRegistryError::ExtractionFailure {
                package_name: package_name.to_string(),
                version: version.to_string(),
                destination: destination.display().to_string(),
                source: e,
            }
        })?;

        Ok(())
    }
}

impl PackageRegistryClone for LocalRegistry {
    fn clone_box(&self) -> Box<dyn PackageRegistryClone> {
        Box::new(self.clone())
    }
}

impl LocalRegistry {
    /// Add a package version to the local registry
    ///
    /// This method is primarily useful for testing scenarios where you need
    /// to populate the registry with known package versions.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `version` - Version string of the package
    /// * `metadata` - Optional JSON metadata for the package version
    ///
    /// # Returns
    ///
    /// `Ok(())` if the package was added successfully, or a `PackageRegistryError` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The internal lock for the package storage fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::LocalRegistry;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = LocalRegistry::default();
    ///
    /// // Add a package version
    /// registry.add_package_version(
    ///     "react",
    ///     "18.2.0",
    ///     Some(json!({"name": "react", "version": "18.2.0"}))
    /// )?;
    ///
    /// // Add another version
    /// registry.add_package_version("react", "17.0.2", None)?;
    ///
    /// // Now the registry contains these versions
    /// let versions = registry.get_all_versions("react")?;
    /// assert_eq!(versions.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_package_version(
        &self,
        package_name: &str,
        version: &str,
        metadata: Option<Value>,
    ) -> Result<(), PackageRegistryError> {
        let mut packages = self.packages.lock()?;

        let package_metadata = metadata.unwrap_or_else(|| {
            json!({
                "name": package_name,
                "version": version
            })
        });

        packages
            .entry(package_name.to_string())
            .or_default()
            .insert(version.to_string(), package_metadata);

        Ok(())
    }

    /// Add multiple versions for a package at once
    ///
    /// This is a convenience method for adding several versions of the same package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `versions` - List of version strings to add
    ///
    /// # Returns
    ///
    /// `Ok(())` if all versions were added successfully, or a `PackageRegistryError` if any operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The internal lock for the package storage fails
    /// - Any individual package version addition fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::LocalRegistry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = LocalRegistry::default();
    ///
    /// // Add multiple versions at once
    /// registry.add_package_versions("lodash", &["4.17.20", "4.17.21"])?;
    ///
    /// let versions = registry.get_all_versions("lodash")?;
    /// assert_eq!(versions.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_package_versions(
        &self,
        package_name: &str,
        versions: &[&str],
    ) -> Result<(), PackageRegistryError> {
        for version in versions {
            self.add_package_version(package_name, version, None)?;
        }
        Ok(())
    }

    /// Clear all packages from the registry
    ///
    /// This method removes all packages and their versions from the registry,
    /// useful for resetting the registry state in tests.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registry was cleared successfully, or a `PackageRegistryError` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The internal lock for the package storage fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::LocalRegistry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = LocalRegistry::default();
    ///
    /// // Add some packages
    /// registry.add_package_version("react", "18.2.0", None)?;
    /// registry.add_package_version("lodash", "4.17.21", None)?;
    ///
    /// // Clear the registry
    /// registry.clear()?;
    ///
    /// // Registry should now be empty
    /// let react_versions = registry.get_all_versions("react")?;
    /// assert!(react_versions.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear(&self) -> Result<(), PackageRegistryError> {
        let mut packages = self.packages.lock()?;
        packages.clear();
        Ok(())
    }

    /// Check if a package exists in the registry
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to check
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the package exists, `Ok(false)` if it doesn't, or a `PackageRegistryError` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The internal lock for the package storage fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::LocalRegistry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = LocalRegistry::default();
    ///
    /// assert!(!registry.has_package("react")?);
    ///
    /// registry.add_package_version("react", "18.2.0", None)?;
    /// assert!(registry.has_package("react")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_package(&self, package_name: &str) -> Result<bool, PackageRegistryError> {
        let packages = self.packages.lock()?;
        Ok(packages.contains_key(package_name))
    }
}
