use crate::error::{PkgError, Result};
use crate::registry::package::PackageRegistry;
use semver::Version;
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A local registry implementation for testing and offline use
#[derive(Debug, Clone)]
pub struct LocalRegistry {
    packages: Arc<Mutex<HashMap<String, HashMap<String, Value>>>>,
}

impl Default for LocalRegistry {
    fn default() -> Self {
        Self { packages: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl LocalRegistry {
    /// Create a new empty local registry
    pub fn new() -> Self {
        Self { packages: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Add a package with specific versions to the registry
    pub fn add_package(&self, name: &str, versions: Vec<&str>) -> Result<()> {
        if let Ok(mut packages) = self.packages.lock() {
            let mut package_versions = HashMap::new();
            for version in versions {
                // Validate version is semver
                let _ = Version::parse(version).map_err(|e| PkgError::VersionParseError {
                    version: version.to_string(),
                    source: e,
                })?;

                // Create package info
                let package_info = json!({
                    "name": name,
                    "version": version,
                    "dependencies": {},
                    "devDependencies": {}
                });

                package_versions.insert(version.to_string(), package_info);
            }

            packages.insert(name.to_string(), package_versions);
            Ok(())
        } else {
            Err(PkgError::Other {
                message: "Failed to acquire lock on LocalRegistry packages".to_string(),
            })
        }
    }

    /// Set dependencies for a specific package version
    pub fn set_dependencies(
        &self,
        name: &str,
        version: &str,
        dependencies: &HashMap<String, String>,
    ) -> Result<()> {
        if let Ok(mut packages) = self.packages.lock() {
            if let Some(package_versions) = packages.get_mut(name) {
                if let Some(package_info) = package_versions.get_mut(version) {
                    let deps_object = json!(dependencies);
                    package_info["dependencies"] = deps_object;
                    Ok(())
                } else {
                    Err(PkgError::Other {
                        message: format!("Version {version} not found for package {name}"),
                    })
                }
            } else {
                Err(PkgError::Other {
                    message: format!("Package {name} not found in local registry"),
                })
            }
        } else {
            Err(PkgError::Other {
                message: "Failed to acquire lock on LocalRegistry packages".to_string(),
            })
        }
    }

    /// Get all packages in the registry
    pub fn get_all_packages(&self) -> Vec<String> {
        if let Ok(packages) = self.packages.lock() {
            packages.keys().cloned().collect()
        } else {
            Vec::new() // Return empty list if we can't acquire the lock
        }
    }
}

impl PackageRegistry for LocalRegistry {
    fn get_latest_version(&self, package_name: &str) -> Result<Option<String>> {
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
            Err(PkgError::Other {
                message: "Failed to acquire lock on LocalRegistry packages".to_string(),
            })
        }
    }

    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let packages = self.packages.lock().unwrap();

        if let Some(versions) = packages.get(package_name) {
            Ok(versions.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_package_info(&self, package_name: &str, version: &str) -> Result<Value> {
        let packages = self.packages.lock().unwrap();

        if let Some(versions) = packages.get(package_name) {
            if let Some(package_info) = versions.get(version) {
                Ok(package_info.clone())
            } else {
                Err(PkgError::Other {
                    message: format!("Version {version} not found for package {package_name}"),
                })
            }
        } else {
            Err(PkgError::Other {
                message: format!("Package {package_name} not found in local registry"),
            })
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
