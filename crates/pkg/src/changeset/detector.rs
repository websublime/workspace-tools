//! # Package change detector for changeset creation
//!
//! ## What
//! Provides functionality to detect which packages are affected by file changes
//! in Git. Maps changed file paths to package directories and identifies
//! package ownership for change tracking.
//!
//! ## How
//! - Reuses MonorepoDetector from sublime_standard_tools for workspace detection
//! - Maps file paths to their containing packages
//! - Groups changed files by package
//! - Handles edge cases like root-level changes
//!
//! ## Why
//! Accurate package detection is critical for creating changesets that
//! correctly identify which packages need version bumps based on
//! file changes in Git history. Reusing MonorepoDetector ensures consistency
//! across the codebase.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

use crate::{
    error::{ChangesetError, ChangesetResult},
    package::{Package, PackageJson},
};

/// Detector for mapping file changes to affected packages.
///
/// Analyzes workspace structure and determines which packages are affected
/// by a set of changed files. Supports both monorepo and single-package
/// project structures.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::PackageChangeDetector;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let detector = PackageChangeDetector::new(
///     PathBuf::from("/workspace"),
///     FileSystemManager::new(),
/// );
///
/// let changed_files = vec![
///     PathBuf::from("packages/auth/src/lib.rs"),
///     PathBuf::from("packages/user/src/lib.rs"),
/// ];
///
/// let affected = detector.detect_affected_packages(&changed_files).await?;
/// println!("Affected packages: {:?}", affected.keys());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageChangeDetector<F = FileSystemManager>
where
    F: AsyncFileSystem + Send + Sync + Clone,
{
    /// Root path of the workspace
    pub(crate) workspace_root: PathBuf,
    /// Filesystem implementation for I/O operations
    pub(crate) filesystem: F,
    /// MonorepoDetector from standard tools for workspace analysis
    monorepo_detector: MonorepoDetector<F>,
}

impl PackageChangeDetector<FileSystemManager> {
    /// Creates a new package change detector with default filesystem.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `filesystem` - Filesystem implementation
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let detector = PackageChangeDetector::new(
    ///     PathBuf::from("/workspace"),
    ///     FileSystemManager::new(),
    /// );
    /// ```
    #[must_use]
    pub fn new(workspace_root: PathBuf, filesystem: FileSystemManager) -> Self {
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        Self { workspace_root, filesystem, monorepo_detector }
    }
}

impl<F> PackageChangeDetector<F>
where
    F: AsyncFileSystem + Send + Sync + Clone,
{
    /// Creates a detector with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `filesystem` - Custom filesystem implementation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use std::path::PathBuf;
    ///
    /// # fn example<F: sublime_standard_tools::filesystem::AsyncFileSystem + Send + Sync>(fs: F) {
    /// let detector = PackageChangeDetector::with_filesystem(
    ///     PathBuf::from("/workspace"),
    ///     fs,
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn with_filesystem(workspace_root: PathBuf, filesystem: F) -> Self {
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        Self { workspace_root, filesystem, monorepo_detector }
    }

    /// Detects which packages are affected by the given changed files.
    ///
    /// Groups changed files by their containing package. Files that don't
    /// belong to any package are grouped under a special "root" package
    /// (if a root package.json exists).
    ///
    /// # Arguments
    ///
    /// * `changed_files` - List of file paths that have changed
    ///
    /// # Returns
    ///
    /// A map from package name to the list of changed files in that package.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Unable to detect workspace structure
    /// - Unable to read package.json files
    /// - No packages found for the changed files
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = PackageChangeDetector::new(
    ///     PathBuf::from("/workspace"),
    ///     FileSystemManager::new(),
    /// );
    ///
    /// let changed_files = vec![
    ///     PathBuf::from("packages/auth/src/index.ts"),
    ///     PathBuf::from("packages/user/src/index.ts"),
    /// ];
    ///
    /// let affected = detector.detect_affected_packages(&changed_files).await?;
    /// for (package_name, files) in affected {
    ///     println!("{}: {} files changed", package_name, files.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_affected_packages(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<HashMap<String, Vec<PathBuf>>> {
        if changed_files.is_empty() {
            return Ok(HashMap::new());
        }

        // Detect if this is a monorepo or single-package project
        let is_monorepo = self.is_monorepo().await?;

        if is_monorepo {
            self.detect_affected_packages_monorepo(changed_files).await
        } else {
            self.detect_affected_packages_single(changed_files).await
        }
    }

    /// Checks if the workspace is a monorepo.
    ///
    /// Uses MonorepoDetector from sublime_standard_tools for consistent detection.
    ///
    /// # Returns
    ///
    /// `true` if the workspace is a monorepo, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns error if unable to detect workspace structure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = PackageChangeDetector::new(
    ///     PathBuf::from("/workspace"),
    ///     FileSystemManager::new(),
    /// );
    ///
    /// if detector.is_monorepo().await? {
    ///     println!("This is a monorepo");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_monorepo(&self) -> ChangesetResult<bool> {
        // Use MonorepoDetector from standard tools for consistent detection
        let monorepo_kind =
            self.monorepo_detector.is_monorepo_root(&self.workspace_root).await.map_err(|e| {
                ChangesetError::package_detection_failed(format!(
                    "Failed to detect monorepo: {}",
                    e
                ))
            })?;

        Ok(monorepo_kind.is_some())
    }

    /// Gets the package that contains the given file path.
    ///
    /// Searches upward from the file path to find the nearest package.json.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    ///
    /// The package containing the file, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns error if unable to read package.json files.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::{Path, PathBuf};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = PackageChangeDetector::new(
    ///     PathBuf::from("/workspace"),
    ///     FileSystemManager::new(),
    /// );
    ///
    /// let file_path = Path::new("packages/auth/src/index.ts");
    /// if let Some(package) = detector.get_package_for_file(file_path).await? {
    ///     println!("File belongs to package: {}", package.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package_for_file(&self, file_path: &Path) -> ChangesetResult<Option<Package>> {
        // Make path absolute if it isn't already
        let abs_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.workspace_root.join(file_path)
        };

        // Start from the file's directory and search upward for package.json
        let mut current_dir = if abs_path.is_file() {
            abs_path
                .parent()
                .ok_or_else(|| {
                    ChangesetError::package_detection_failed(format!(
                        "Unable to get parent directory for: {}",
                        file_path.display()
                    ))
                })?
                .to_path_buf()
        } else {
            abs_path.clone()
        };

        // Search upward until we find a package.json or reach the workspace root
        loop {
            let package_json_path = current_dir.join("package.json");

            if self.filesystem.exists(&package_json_path).await {
                // Found a package.json, try to read it
                match PackageJson::read_from_path(&self.filesystem, &package_json_path).await {
                    Ok(package_json) => {
                        let package = Package { metadata: package_json, path: current_dir.clone() };
                        return Ok(Some(package));
                    }
                    Err(e) => {
                        return Err(ChangesetError::package_json_read_failed(
                            package_json_path,
                            format!("{}", e),
                        ));
                    }
                }
            }

            // Move to parent directory
            match current_dir.parent() {
                Some(parent) if parent.starts_with(&self.workspace_root) => {
                    current_dir = parent.to_path_buf();
                }
                _ => {
                    // Reached workspace root or above without finding package.json
                    break;
                }
            }
        }

        Ok(None)
    }

    /// Detects affected packages in a monorepo structure.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - List of changed file paths
    ///
    /// # Returns
    ///
    /// Map of package names to their changed files
    ///
    /// # Errors
    ///
    /// Returns error if unable to detect packages or read package.json files.
    async fn detect_affected_packages_monorepo(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<HashMap<String, Vec<PathBuf>>> {
        // Use MonorepoDetector to get all packages
        // Note: This should only be called after is_monorepo() returns true
        let monorepo_descriptor =
            self.monorepo_detector.detect_monorepo(&self.workspace_root).await.map_err(|e| {
                ChangesetError::package_detection_failed(format!(
                    "Failed to detect monorepo packages: {}",
                    e
                ))
            })?;

        let packages = monorepo_descriptor.packages();

        if packages.is_empty() {
            return Err(ChangesetError::package_detection_failed(
                "No packages found in monorepo workspace",
            ));
        }

        // Build a map of package paths to package names
        let mut package_map: HashMap<PathBuf, String> = HashMap::new();
        for pkg in packages {
            let package_json_path = pkg.absolute_path.join("package.json");
            match PackageJson::read_from_path(&self.filesystem, &package_json_path).await {
                Ok(pkg_json) => {
                    package_map.insert(pkg.absolute_path.clone(), pkg_json.name.clone());
                }
                Err(e) => {
                    return Err(ChangesetError::package_json_read_failed(
                        package_json_path,
                        format!("{}", e),
                    ));
                }
            }
        }

        // Group changed files by package
        let mut result: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut unmatched_files = Vec::new();

        for file_path in changed_files {
            let abs_path = if file_path.is_absolute() {
                file_path.clone()
            } else {
                self.workspace_root.join(file_path)
            };

            // Find which package this file belongs to
            let mut matched = false;
            for (pkg_path, pkg_name) in &package_map {
                if abs_path.starts_with(pkg_path) {
                    result.entry(pkg_name.clone()).or_default().push(file_path.clone());
                    matched = true;
                    break;
                }
            }

            if !matched {
                unmatched_files.push(file_path.clone());
            }
        }

        // Handle root-level changes (files not in any package)
        if !unmatched_files.is_empty() {
            let root_package_json = self.workspace_root.join("package.json");
            if self.filesystem.exists(&root_package_json).await {
                match PackageJson::read_from_path(&self.filesystem, &root_package_json).await {
                    Ok(pkg_json) => {
                        result.insert(pkg_json.name.clone(), unmatched_files);
                    }
                    Err(_) => {
                        // Root package.json exists but couldn't be read - ignore these files
                    }
                }
            }
            // If no root package.json, ignore unmatched files
        }

        if result.is_empty() {
            return Err(ChangesetError::package_detection_failed(
                "No packages affected by the changed files",
            ));
        }

        Ok(result)
    }

    /// Detects affected packages in a single-package structure.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - List of changed file paths
    ///
    /// # Returns
    ///
    /// Map with a single entry for the root package
    ///
    /// # Errors
    ///
    /// Returns error if unable to read the root package.json.
    async fn detect_affected_packages_single(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<HashMap<String, Vec<PathBuf>>> {
        let package_json_path = self.workspace_root.join("package.json");

        if !self.filesystem.exists(&package_json_path).await {
            return Err(ChangesetError::package_detection_failed(
                "No package.json found at workspace root",
            ));
        }

        let package_json = PackageJson::read_from_path(&self.filesystem, &package_json_path)
            .await
            .map_err(|e| {
                ChangesetError::package_json_read_failed(
                    package_json_path.clone(),
                    format!("{}", e),
                )
            })?;

        let mut result = HashMap::new();
        result.insert(package_json.name.clone(), changed_files.to_vec());

        Ok(result)
    }

    /// Lists all packages in the workspace.
    ///
    /// # Returns
    ///
    /// List of all packages found in the workspace
    ///
    /// # Errors
    ///
    /// Returns error if unable to detect packages or read package.json files.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::PackageChangeDetector;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = PackageChangeDetector::new(
    ///     PathBuf::from("/workspace"),
    ///     FileSystemManager::new(),
    /// );
    ///
    /// let packages = detector.list_all_packages().await?;
    /// for package in packages {
    ///     println!("Found package: {}", package.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_all_packages(&self) -> ChangesetResult<Vec<Package>> {
        let is_monorepo = self.is_monorepo().await?;

        if is_monorepo {
            self.list_packages_monorepo().await
        } else {
            self.list_packages_single().await
        }
    }

    /// Lists all packages in a monorepo.
    ///
    /// Note: This should only be called after is_monorepo() returns true.
    async fn list_packages_monorepo(&self) -> ChangesetResult<Vec<Package>> {
        // Use MonorepoDetector to get all packages
        // This will error if not a monorepo, so ensure is_monorepo() is checked first
        let monorepo_descriptor =
            self.monorepo_detector.detect_monorepo(&self.workspace_root).await.map_err(|e| {
                ChangesetError::package_detection_failed(format!(
                    "Failed to detect monorepo packages: {}",
                    e
                ))
            })?;

        let workspace_packages = monorepo_descriptor.packages();
        let mut packages = Vec::new();

        for pkg in workspace_packages {
            let package_json_path = pkg.absolute_path.join("package.json");
            match PackageJson::read_from_path(&self.filesystem, &package_json_path).await {
                Ok(package_json) => {
                    packages
                        .push(Package { metadata: package_json, path: pkg.absolute_path.clone() });
                }
                Err(e) => {
                    return Err(ChangesetError::package_json_read_failed(
                        package_json_path,
                        format!("{}", e),
                    ));
                }
            }
        }

        Ok(packages)
    }

    /// Lists the single package at the workspace root.
    async fn list_packages_single(&self) -> ChangesetResult<Vec<Package>> {
        let package_json_path = self.workspace_root.join("package.json");

        if !self.filesystem.exists(&package_json_path).await {
            return Err(ChangesetError::package_detection_failed(
                "No package.json found at workspace root",
            ));
        }

        let package_json = PackageJson::read_from_path(&self.filesystem, &package_json_path)
            .await
            .map_err(|e| {
                ChangesetError::package_json_read_failed(
                    package_json_path.clone(),
                    format!("{}", e),
                )
            })?;

        Ok(vec![Package { metadata: package_json, path: self.workspace_root.clone() }])
    }
}
