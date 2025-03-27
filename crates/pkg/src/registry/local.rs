use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use semver::Version;
use serde_json::Value;

use crate::{PackageRegistry, PackageRegistryError};

#[derive(Debug, Clone)]
pub struct LocalRegistry {
    packages: Arc<Mutex<HashMap<String, HashMap<String, Value>>>>,
}

impl Default for LocalRegistry {
    fn default() -> Self {
        Self { packages: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl PackageRegistry for LocalRegistry {
    fn get_latest_version(
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

    fn get_all_versions(&self, package_name: &str) -> Result<Vec<String>, PackageRegistryError> {
        let packages = self.packages.lock()?;

        if let Some(versions) = packages.get(package_name) {
            Ok(versions.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_package_info(
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
}
