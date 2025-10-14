//! Package.json operations module for sublime_pkg_tools.
//!
//! This module provides comprehensive support for reading, parsing, modifying,
//! and validating package.json files. It integrates with the sublime_standard_tools
//! filesystem abstraction and preserves file formatting during modifications.
//!
//! # What
//!
//! Implements package.json operations required for version management and
//! dependency updates:
//! - Reading and parsing package.json files from the filesystem
//! - Modifying version fields and dependency sections
//! - Preserving file formatting, indentation, and comments
//! - Validating package.json structure and required fields
//! - Supporting both single packages and monorepo workspaces
//!
//! # How
//!
//! Uses serde_json for parsing with custom formatting preservation logic.
//! Integrates with FileSystemManager for cross-platform file operations.
//! Leverages MonorepoDetector from standard crate for workspace discovery.
//! Provides type-safe representations of package.json structure and validates
//! against Node.js package.json specifications.
//!
//! # Why
//!
//! Package.json files are central to Node.js project management. This module
//! ensures safe, reliable modifications while preserving the human-readable
//! formatting that developers depend on. It supports the version management
//! workflows required by changeset operations while reusing battle-tested
//! monorepo detection from the standard crate.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::package::{PackageJson, PackageJsonEditor, Package};
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let package_path = Path::new("./package.json");
//!
//! // Read and parse package.json
//! let package_json = PackageJson::read_from_path(&fs, package_path).await?;
//! println!("Package: {} v{}", package_json.name, package_json.version);
//!
//! // Create editor for modifications
//! let mut editor = PackageJsonEditor::new(&fs, package_path).await?;
//! editor.set_version("1.2.4")?;
//! editor.update_dependency("lodash", "^4.17.21")?;
//! editor.save().await?;
//!
//! // Create Package representation
//! let package = Package::from_path(&fs, Path::new("./")).await?;
//! println!("Found package: {}", package.name);
//! # Ok(())
//! # }
//! ```

use futures::future::BoxFuture;
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

mod editor;
mod json;
#[allow(clippy::module_inception)]
mod package;
mod validation;

#[cfg(test)]
mod tests;

pub use editor::{PackageJsonEditor, PackageJsonModification};
pub use json::{
    BugsInfo, Dependencies, DependencyType, PackageJson, PackageJsonMetadata, PersonOrString,
    Repository, Scripts, WorkspaceConfig,
};
pub use package::{Package, PackageInfo};
pub use validation::{PackageJsonValidator, ValidationIssue, ValidationResult, ValidationSeverity};

use crate::error::{PackageError, PackageResult};
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Convenience function to read a package.json file from a path.
///
/// This function provides a simple interface for reading and parsing package.json
/// files without needing to create a PackageJson instance manually.
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use for reading
/// * `path` - Path to the package.json file
///
/// # Returns
///
/// A parsed PackageJson instance
///
/// # Errors
///
/// Returns an error if:
/// - File cannot be read from the filesystem
/// - JSON content is malformed
/// - Required fields are missing
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::read_package_json;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let package = read_package_json(&fs, Path::new("./package.json")).await?;
/// println!("Package name: {}", package.name);
/// # Ok(())
/// # }
/// ```
pub async fn read_package_json<F>(filesystem: &F, path: &Path) -> PackageResult<PackageJson>
where
    F: AsyncFileSystem + Send + Sync,
{
    PackageJson::read_from_path(filesystem, path).await
}

/// Convenience function to validate a package.json file.
///
/// This function provides a simple interface for validating package.json files
/// without needing to create validator instances manually.
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use for reading
/// * `path` - Path to the package.json file
///
/// # Returns
///
/// A validation result with any issues found
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::validate_package_json;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let result = validate_package_json(&fs, Path::new("./package.json")).await?;
///
/// if result.has_errors() {
///     for error in result.errors() {
///         eprintln!("Error: {}", error.message);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn validate_package_json<F>(
    filesystem: &F,
    path: &Path,
) -> PackageResult<ValidationResult>
where
    F: AsyncFileSystem + Send + Sync,
{
    let validator = PackageJsonValidator::new()?;
    validator.validate_file(filesystem, path).await
}

/// Convenience function to create a Package from a directory path.
///
/// This function automatically locates the package.json file in the given
/// directory and creates a Package instance from it.
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use
/// * `directory` - Path to the directory containing package.json
///
/// # Returns
///
/// A Package instance representing the package in that directory
///
/// # Errors
///
/// Returns an error if:
/// - No package.json file is found in the directory
/// - Package.json cannot be parsed
/// - Required package information is missing
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::create_package_from_directory;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let package = create_package_from_directory(&fs, Path::new("./packages/auth")).await?;
/// println!("Found package: {} at {}", package.name, package.path.display());
/// # Ok(())
/// # }
/// ```
pub async fn create_package_from_directory<F>(
    filesystem: &F,
    directory: &Path,
) -> PackageResult<Package>
where
    F: AsyncFileSystem + Send + Sync,
{
    Package::from_path(filesystem, directory).await
}

/// Checks if a directory contains a valid package.json file.
///
/// This function provides a quick way to determine if a directory is a
/// valid Node.js package without fully parsing the package.json.
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use
/// * `directory` - Path to check for package.json
///
/// # Returns
///
/// True if the directory contains a readable package.json file
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::is_package_directory;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
///
/// if is_package_directory(&fs, Path::new("./packages/auth")).await {
///     println!("Found a package directory");
/// }
/// # Ok(())
/// # }
/// ```
pub async fn is_package_directory<F>(filesystem: &F, directory: &Path) -> bool
where
    F: AsyncFileSystem + Send + Sync,
{
    let package_json_path = directory.join("package.json");
    filesystem.exists(&package_json_path).await
}

/// Finds all package directories in a given root directory.
///
/// This function uses the MonorepoDetector from sublime_standard_tools to discover
/// packages in monorepo structures, falling back to simple directory scanning
/// for non-monorepo projects. This provides more robust workspace detection
/// while maintaining backward compatibility.
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use
/// * `root` - Root directory to search from
/// * `max_depth` - Maximum search depth (Some for compatibility, but ignored in monorepo detection)
///
/// # Returns
///
/// A vector of paths to directories containing package.json files
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::find_package_directories;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let packages = find_package_directories(&fs, Path::new("./"), Some(3)).await?;
///
/// for package_dir in packages {
///     println!("Found package at: {}", package_dir.display());
/// }
/// # Ok(())
/// # }
/// ```
pub async fn find_package_directories<F>(
    filesystem: &F,
    root: &Path,
    max_depth: Option<usize>,
) -> PackageResult<Vec<std::path::PathBuf>>
where
    F: AsyncFileSystem + Send + Sync + Clone,
{
    // First, try to use MonorepoDetector for enhanced workspace discovery
    let detector = MonorepoDetector::with_filesystem(filesystem.clone());

    // Check if this is a monorepo root
    match detector.is_monorepo_root(root).await {
        Ok(Some(_monorepo_kind)) => {
            // Use MonorepoDetector for workspace package discovery
            match detector.detect_packages(root).await {
                Ok(workspace_packages) => {
                    let package_paths: Vec<std::path::PathBuf> =
                        workspace_packages.into_iter().map(|pkg| pkg.absolute_path).collect();
                    return Ok(package_paths);
                }
                Err(e) => {
                    log::warn!("MonorepoDetector failed, falling back to manual scan: {}", e);
                    // Fall through to manual scanning
                }
            }
        }
        Ok(None) => {
            // Not a monorepo, check if root itself is a package
            if is_package_directory(filesystem, root).await {
                return Ok(vec![root.to_path_buf()]);
            }
            // Fall through to manual scanning for loose packages
        }
        Err(e) => {
            log::warn!("MonorepoDetector check failed, falling back to manual scan: {}", e);
            // Fall through to manual scanning
        }
    }

    // Fallback: use original recursive scanning approach
    let mut packages = Vec::new();
    find_packages_recursive(filesystem, root, root, max_depth, 0, &mut packages).await?;
    Ok(packages)
}

/// Recursive helper function for finding package directories (fallback implementation).
///
/// This function is used as a fallback when MonorepoDetector cannot be used or fails.
/// It provides the original manual scanning behavior for backward compatibility.
fn find_packages_recursive<'a, F>(
    filesystem: &'a F,
    current: &'a Path,
    _root: &'a Path,
    max_depth: Option<usize>,
    current_depth: usize,
    packages: &'a mut Vec<std::path::PathBuf>,
) -> BoxFuture<'a, PackageResult<()>>
where
    F: AsyncFileSystem + Send + Sync,
{
    Box::pin(async move {
        // Check depth limit
        if let Some(max) = max_depth {
            if current_depth >= max {
                return Ok(());
            }
        }

        // Check if current directory is a package
        if is_package_directory(filesystem, current).await {
            packages.push(current.to_path_buf());
        }

        // Skip node_modules directories to avoid scanning dependencies
        if current.file_name().and_then(|name| name.to_str()) == Some("node_modules") {
            return Ok(());
        }

        // Skip common non-package directories for performance
        if let Some(dir_name) = current.file_name().and_then(|name| name.to_str()) {
            match dir_name {
                "node_modules" | ".git" | ".svn" | ".hg" | "target" | "build" | "dist"
                | ".next" => {
                    return Ok(());
                }
                _ => {}
            }
        }

        // Recursively search subdirectories
        if filesystem.exists(current).await {
            let entries = filesystem.read_dir(current).await.map_err(|e| {
                PackageError::operation(
                    "find_package_directories",
                    format!("Failed to read directory {}: {}", current.display(), e),
                )
            })?;

            for path in entries {
                if filesystem.exists(&path).await {
                    find_packages_recursive(
                        filesystem,
                        &path,
                        _root,
                        max_depth,
                        current_depth + 1,
                        packages,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    })
}
