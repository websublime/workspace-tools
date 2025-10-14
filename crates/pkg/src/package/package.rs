//! Package representation and information structures.
//!
//! This module provides high-level abstractions for working with Node.js packages,
//! combining package.json metadata with filesystem location information. It offers
//! a convenient interface for package operations without dealing with raw JSON.
//!
//! # What
//!
//! Defines Package struct that combines:
//! - Package metadata from package.json
//! - Filesystem location and path information
//! - Convenience methods for common package operations
//! - Integration with version resolution and dependency analysis
//!
//! # How
//!
//! Builds on the PackageJson structure and adds filesystem context.
//! Provides trait-based access to package information for flexibility.
//! Integrates with FileSystemManager for cross-platform compatibility.
//!
//! # Why
//!
//! Raw package.json files lack context about their location in the filesystem.
//! The Package abstraction provides this context and simplifies common operations
//! like finding dependencies, checking workspace relationships, and resolving paths.

use crate::error::{PackageError, PackageResult};
use crate::package::PackageJson;
use crate::version::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Trait for accessing package information.
///
/// This trait provides a common interface for accessing package metadata
/// regardless of the underlying representation. It allows for flexible
/// implementations and easy testing with mock data.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::package::{Package, PackageInfo};
/// use std::path::Path;
///
/// fn process_package<P: PackageInfo>(package: &P) {
///     println!("Processing package: {} v{}", package.name(), package.version());
///     println!("Located at: {}", package.path().display());
/// }
/// ```
pub trait PackageInfo {
    /// Gets the package name.
    ///
    /// # Returns
    ///
    /// The package name as defined in package.json
    fn name(&self) -> &str;

    /// Gets the package version.
    ///
    /// # Returns
    ///
    /// The current version as defined in package.json
    fn version(&self) -> &Version;

    /// Gets the package description if available.
    ///
    /// # Returns
    ///
    /// The package description or None if not set
    fn description(&self) -> Option<&str>;

    /// Gets the filesystem path to the package directory.
    ///
    /// # Returns
    ///
    /// Path to the directory containing package.json
    fn path(&self) -> &Path;

    /// Gets the absolute path to the package.json file.
    ///
    /// # Returns
    ///
    /// Full path to the package.json file
    fn package_json_path(&self) -> PathBuf {
        self.path().join("package.json")
    }

    /// Checks if this package is private (not published).
    ///
    /// # Returns
    ///
    /// True if the package is marked as private
    fn is_private(&self) -> bool;

    /// Checks if this package is a workspace root.
    ///
    /// # Returns
    ///
    /// True if the package defines workspaces
    fn is_workspace_root(&self) -> bool;

    /// Gets all dependency names regardless of type.
    ///
    /// # Returns
    ///
    /// A vector of dependency names
    fn dependency_names(&self) -> Vec<String>;

    /// Checks if this package has a specific dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name to check
    ///
    /// # Returns
    ///
    /// True if the package depends on the named dependency
    fn has_dependency(&self, name: &str) -> bool;

    /// Gets the license information if available.
    ///
    /// # Returns
    ///
    /// The package license or None if not set
    fn license(&self) -> Option<&str>;

    /// Gets the main entry point if available.
    ///
    /// # Returns
    ///
    /// The main entry point or None if not set
    fn main_entry(&self) -> Option<&str>;

    /// Gets the module entry point if available.
    ///
    /// # Returns
    ///
    /// The ES module entry point or None if not set
    fn module_entry(&self) -> Option<&str>;

    /// Gets the TypeScript types entry point if available.
    ///
    /// # Returns
    ///
    /// The types entry point or None if not set
    fn types_entry(&self) -> Option<&str>;
}

/// Represents a Node.js package with its metadata and filesystem location.
///
/// This structure combines package.json metadata with filesystem context,
/// providing a complete representation of a package. It implements PackageInfo
/// and provides additional methods for package operations.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::Package;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let package = Package::from_path(&fs, Path::new("./packages/auth")).await?;
///
/// println!("Package: {} v{}", package.name(), package.version());
/// println!("Path: {}", package.path().display());
///
/// if package.is_workspace_root() {
///     println!("This is a workspace root package");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package metadata from package.json
    pub metadata: PackageJson,

    /// Filesystem path to the package directory
    pub path: PathBuf,
}

impl Package {
    /// Creates a new Package from metadata and path.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The parsed package.json metadata
    /// * `path` - Path to the package directory
    ///
    /// # Returns
    ///
    /// A new Package instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let metadata = PackageJson::default();
    /// let path = PathBuf::from("./my-package");
    /// let package = Package::new(metadata, path);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(metadata: PackageJson, path: PathBuf) -> Self {
        Self { metadata, path }
    }

    /// Creates a Package by reading from a directory path.
    ///
    /// This method looks for a package.json file in the given directory
    /// and creates a Package instance from it.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem implementation to use
    /// * `directory` - Path to the directory containing package.json
    ///
    /// # Returns
    ///
    /// A Package instance loaded from the directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No package.json file exists in the directory
    /// - The package.json file cannot be read or parsed
    /// - The package.json is missing required fields
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::Package;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let package = Package::from_path(&fs, Path::new("./packages/auth")).await?;
    /// println!("Loaded package: {}", package.name());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_path<F>(filesystem: &F, directory: &Path) -> PackageResult<Self>
    where
        F: AsyncFileSystem + Send + Sync,
    {
        let package_json_path = directory.join("package.json");

        if !filesystem.exists(&package_json_path).await {
            return Err(PackageError::operation(
                "load_package",
                format!("No package.json found in directory: {}", directory.display()),
            ));
        }

        let metadata = PackageJson::read_from_path(filesystem, &package_json_path).await?;
        Ok(Self::new(metadata, directory.to_path_buf()))
    }

    /// Gets access to the raw package.json metadata.
    ///
    /// # Returns
    ///
    /// Reference to the underlying PackageJson structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// let metadata = package.package_json();
    /// println!("Scripts: {:?}", metadata.scripts);
    /// # }
    /// ```
    pub fn package_json(&self) -> &PackageJson {
        &self.metadata
    }

    /// Gets mutable access to the package.json metadata.
    ///
    /// # Returns
    ///
    /// Mutable reference to the underlying PackageJson structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// let metadata = package.package_json_mut();
    /// metadata.description = Some("Updated description".to_string());
    /// # }
    /// ```
    pub fn package_json_mut(&mut self) -> &mut PackageJson {
        &mut self.metadata
    }

    /// Updates the package version.
    ///
    /// # Arguments
    ///
    /// * `new_version` - The new version to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson, PackageInfo};
    /// use sublime_pkg_tools::version::Version;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// let new_version = Version::new(2, 0, 0);
    /// package.set_version(new_version);
    /// assert_eq!(package.version().to_string(), "2.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_version(&mut self, new_version: Version) {
        self.metadata.version = new_version;
    }

    /// Adds or updates a dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The dependency version constraint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson, PackageInfo};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// package.add_dependency("lodash".to_string(), "^4.17.21".to_string());
    /// assert!(package.has_dependency("lodash"));
    /// # }
    /// ```
    pub fn add_dependency(&mut self, name: String, version: String) {
        self.metadata.dependencies.insert(name, version);
    }

    /// Adds or updates a development dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The dependency version constraint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// package.add_dev_dependency("jest".to_string(), "^29.0.0".to_string());
    /// # }
    /// ```
    pub fn add_dev_dependency(&mut self, name: String, version: String) {
        self.metadata.dev_dependencies.insert(name, version);
    }

    /// Removes a dependency of any type.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name to remove
    ///
    /// # Returns
    ///
    /// True if a dependency was removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson, PackageInfo};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("."));
    /// package.add_dependency("lodash".to_string(), "^4.17.21".to_string());
    /// assert!(package.remove_dependency("lodash"));
    /// assert!(!package.has_dependency("lodash"));
    /// # }
    /// ```
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        let mut removed = false;

        if self.metadata.dependencies.remove(name).is_some() {
            removed = true;
        }

        if self.metadata.dev_dependencies.remove(name).is_some() {
            removed = true;
        }

        if self.metadata.peer_dependencies.remove(name).is_some() {
            removed = true;
        }

        if self.metadata.optional_dependencies.remove(name).is_some() {
            removed = true;
        }

        removed
    }

    /// Gets the workspace patterns if this is a workspace root.
    ///
    /// # Returns
    ///
    /// A vector of workspace patterns
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson, WorkspaceConfig};
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let mut metadata = PackageJson::default();
    /// metadata.workspaces = Some(WorkspaceConfig::Packages(vec!["packages/*".to_string()]));
    /// let package = Package::new(metadata, PathBuf::from("."));
    ///
    /// let patterns = package.workspace_patterns();
    /// assert_eq!(patterns, vec!["packages/*"]);
    /// # }
    /// ```
    pub fn workspace_patterns(&self) -> Vec<String> {
        self.metadata.get_workspace_patterns()
    }

    /// Gets the relative path from a workspace root to this package.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// The relative path from workspace root to this package
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use std::path::{Path, PathBuf};
    ///
    /// # fn example() {
    /// let package = Package::new(
    ///     PackageJson::default(),
    ///     PathBuf::from("/workspace/packages/auth")
    /// );
    ///
    /// let relative = package.relative_path_from(Path::new("/workspace"));
    /// // Would return "packages/auth" on Unix systems
    /// # }
    /// ```
    pub fn relative_path_from(&self, workspace_root: &Path) -> Option<PathBuf> {
        self.path.strip_prefix(workspace_root).ok().map(|p| p.to_path_buf())
    }

    /// Saves the package.json to the filesystem.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem implementation to use for writing
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or serialized
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::{Package, PackageJson};
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let mut package = Package::new(PackageJson::default(), PathBuf::from("./test-package"));
    /// package.save(&fs).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save<F>(&self, filesystem: &F) -> PackageResult<()>
    where
        F: AsyncFileSystem + Send + Sync,
    {
        let package_json_path = self.package_json_path();
        let content = self.metadata.to_pretty_json()?;

        filesystem.write_file_string(&package_json_path, &content).await.map_err(|e| {
            PackageError::operation(
                "save_package_json",
                format!("Failed to write {}: {}", package_json_path.display(), e),
            )
        })
    }
}

impl PackageInfo for Package {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &Version {
        &self.metadata.version
    }

    fn description(&self) -> Option<&str> {
        self.metadata.description.as_deref()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn is_private(&self) -> bool {
        self.metadata.private.unwrap_or(false)
    }

    fn is_workspace_root(&self) -> bool {
        self.metadata.is_workspace_root()
    }

    fn dependency_names(&self) -> Vec<String> {
        self.metadata.get_all_dependencies().into_iter().map(|(name, _, _)| name).collect()
    }

    fn has_dependency(&self, name: &str) -> bool {
        self.metadata.get_dependency(name).is_some()
    }

    fn license(&self) -> Option<&str> {
        self.metadata.license.as_deref()
    }

    fn main_entry(&self) -> Option<&str> {
        self.metadata.main.as_deref()
    }

    fn module_entry(&self) -> Option<&str> {
        self.metadata.module.as_deref()
    }

    fn types_entry(&self) -> Option<&str> {
        self.metadata.types.as_deref().or(self.metadata.typings.as_deref())
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.name == other.metadata.name && self.path == other.path
    }
}

impl Eq for Package {}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{} ({})", self.name(), self.version(), self.path().display())
    }
}
