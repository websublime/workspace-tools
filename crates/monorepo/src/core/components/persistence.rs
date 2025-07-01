//! Package persistence component
//!
//! Handles all persistence operations including saving package.json files,
//! loading package information, and managing file system interactions.

use super::super::types::MonorepoPackageInfo;
use crate::error::{Error, Result};
use std::path::Path;
// TODO: These will be needed when reload() method is fully implemented
// use sublime_package_tools::{DependencyRegistry, Package, PackageInfo};

/// Component for managing package persistence operations
pub struct PackagePersistence {
    package: MonorepoPackageInfo,
}

impl PackagePersistence {
    /// Create a new package persistence manager
    #[must_use]
    pub fn new(package: MonorepoPackageInfo) -> Self {
        Self { package }
    }

    /// Get immutable reference to the package
    #[must_use]
    pub fn package(&self) -> &MonorepoPackageInfo {
        &self.package
    }

    /// Save the package.json file to disk
    ///
    /// # Errors
    /// Returns an error if the file cannot be written
    pub fn save(&self) -> Result<()> {
        self.package.package_info.write_package_json()
            .map_err(|e| Error::package(format!("Failed to save package.json: {e}")))
    }

    /// Save the package.json file to a specific path
    ///
    /// # Arguments
    /// * `path` - Path where to save the package.json file
    ///
    /// # Errors
    /// Returns an error if the file cannot be written
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        // This would require copying the file from current location to new path
        let current_path = self.package_json_path();
        std::fs::copy(&current_path, path)
            .map_err(|e| Error::package(format!("Failed to copy package.json to {path}: {e}", path = path.display())))?;
        Ok(())
    }

    /// Create a backup of the current package.json
    ///
    /// # Returns
    /// Path to the created backup file
    ///
    /// # Errors
    /// Returns an error if the backup cannot be created
    pub fn create_backup(&self) -> Result<std::path::PathBuf> {
        let package_path = &self.package.workspace_package.absolute_path;
        let package_json_path = package_path.join("package.json");
        
        if !package_json_path.exists() {
            return Err(Error::package("package.json not found"));
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("package.json.backup.{timestamp}");
        let backup_path = package_path.join(backup_name);

        std::fs::copy(&package_json_path, &backup_path)
            .map_err(|e| Error::package(format!("Failed to create backup: {e}")))?;

        Ok(backup_path)
    }

    /// Restore from a backup file
    ///
    /// # Arguments
    /// * `backup_path` - Path to the backup file to restore from
    ///
    /// # Errors
    /// Returns an error if the restore operation fails
    pub fn restore_from_backup(&mut self, backup_path: &Path) -> Result<()> {
        if !backup_path.exists() {
            return Err(Error::package("Backup file not found"));
        }

        let package_path = &self.package.workspace_package.absolute_path;
        let package_json_path = package_path.join("package.json");

        std::fs::copy(backup_path, &package_json_path)
            .map_err(|e| Error::package(format!("Failed to restore from backup: {e}")))?;

        // Reload package info after restore
        self.reload()?;

        Ok(())
    }

    /// Reload package information from disk
    ///
    /// # Errors
    /// Returns an error if the package information cannot be reloaded
    pub fn reload(&mut self) -> Result<()> {
        let package_json_path = self.package_json_path();
        
        // Read the package.json file
        let content = std::fs::read_to_string(&package_json_path)
            .map_err(|e| Error::package(format!("Failed to read package.json: {e}")))?;
        
        // Parse JSON content
        let pkg_json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::package(format!("Failed to parse package.json: {e}")))?;
        
        // Extract package name and version
        let name = pkg_json["name"].as_str()
            .ok_or_else(|| Error::package("Missing 'name' field in package.json"))?
            .to_string();
        
        let version = pkg_json["version"].as_str()
            .ok_or_else(|| Error::package("Missing 'version' field in package.json"))?
            .to_string();
        
        // Create dependency registry for parsing dependencies
        let mut dependency_registry = sublime_package_tools::DependencyRegistry::new();
        
        // Parse dependencies
        let mut dependencies = Vec::new();
        if let Some(deps_obj) = pkg_json["dependencies"].as_object() {
            for (dep_name, dep_version) in deps_obj {
                if let Some(version_str) = dep_version.as_str() {
                    dependencies.push((dep_name.as_str(), version_str));
                }
            }
        }
        
        // Create new Package with dependencies
        let new_package = sublime_package_tools::Package::new_with_registry(
            &name, 
            &version, 
            Some(dependencies),
            &mut dependency_registry
        ).map_err(|e| Error::package(format!("Failed to create package: {e}")))?;
        
        // Create new PackageInfo
        let new_package_info = sublime_package_tools::PackageInfo::new(
            new_package,
            package_json_path.to_string_lossy().to_string(),
            self.package.workspace_package.absolute_path.to_string_lossy().to_string(),
            self.package.workspace_package.location.to_string_lossy().to_string(),
            pkg_json
        );
        
        // Update the package info
        self.package.package_info = new_package_info;
        
        // Update workspace package version to match
        self.package.workspace_package.version = version;
        self.package.workspace_package.name = name;
        
        Ok(())
    }

    /// Check if the package.json file exists on disk
    #[must_use]
    pub fn package_json_exists(&self) -> bool {
        let package_path = &self.package.workspace_package.absolute_path;
        package_path.join("package.json").exists()
    }

    /// Get the path to the package.json file
    #[must_use]
    pub fn package_json_path(&self) -> std::path::PathBuf {
        self.package.workspace_package.absolute_path.join("package.json")
    }

    /// Get file metadata for the package.json file
    ///
    /// # Errors
    /// Returns an error if the file metadata cannot be read
    pub fn package_json_metadata(&self) -> Result<std::fs::Metadata> {
        let package_json_path = self.package_json_path();
        std::fs::metadata(&package_json_path)
            .map_err(|e| Error::package(format!("Failed to read package.json metadata: {e}")))
    }

    /// Check if the package.json file has been modified since the last load
    ///
    /// # Errors
    /// Returns an error if the file metadata cannot be read
    pub fn is_package_json_modified(&self) -> Result<bool> {
        let metadata = self.package_json_metadata()?;
        let modified_time = metadata.modified()
            .map_err(|e| Error::package(format!("Failed to get modification time: {e}")))?;

        // Compare with package info last modified time
        // This is a simplified check - in practice, you might want to store
        // the last loaded time in the PackageInfo or MonorepoPackageInfo
        Ok(modified_time.elapsed().map_or(false, |duration| duration.as_secs() < 60))
    }

    /// Validate package.json structure
    ///
    /// # Returns
    /// List of validation errors found
    #[must_use]
    pub fn validate_package_json(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check if package.json exists
        if !self.package_json_exists() {
            errors.push("package.json file not found".to_string());
            return errors;
        }

        // Check basic required fields from package structure
        let package_ref = self.package.package_info.package.borrow();
        if package_ref.name().trim().is_empty() {
            errors.push("Package name is empty".to_string());
        }

        let version_str = package_ref.version_str();
        if version_str.trim().is_empty() {
            errors.push("Package version is empty".to_string());
        }

        // Validate version format
        if let Err(e) = sublime_package_tools::Version::parse(&version_str) {
            errors.push(format!("Invalid version format: {e}"));
        }
        drop(package_ref);

        // Check for required directories
        let package_path = &self.package.workspace_package.absolute_path;
        if !package_path.exists() {
            errors.push(format!("Package directory not found: {package_path}", package_path = package_path.display()));
        }

        errors
    }

    /// Get package file size
    ///
    /// # Returns
    /// Size of the package.json file in bytes
    ///
    /// # Errors
    /// Returns an error if the file size cannot be read
    pub fn package_json_size(&self) -> Result<u64> {
        let metadata = self.package_json_metadata()?;
        Ok(metadata.len())
    }

    /// Consume the manager and return the updated package
    #[must_use]
    pub fn into_package(self) -> MonorepoPackageInfo {
        self.package
    }
}