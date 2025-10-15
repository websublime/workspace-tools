//! Package.json operations module for sublime_pkg_tools.
//!
//! This module provides package.json-specific operations for Node.js projects.
//! It focuses on parsing, modifying, and validating package.json files while
//! delegating project structure detection to the `sublime_standard_tools` crate.
//!
//! # What
//!
//! Implements package.json-specific operations:
//! - Reading and parsing package.json files with type-safe representations
//! - Modifying version fields and dependency sections with format preservation
//! - Validating package.json structure against Node.js specifications
//! - Supporting both single packages and monorepo workspaces
//!
//! # How
//!
//! Uses serde_json for parsing with custom formatting preservation logic.
//! Integrates with FileSystemManager for cross-platform file operations.
//! Provides type-safe representations of package.json structure and validates
//! against Node.js package.json specifications.
//!
//! # Why
//!
//! Package.json files are central to Node.js project management. This module
//! ensures safe, reliable modifications while preserving the human-readable
//! formatting that developers depend on. It focuses on package.json-specific
//! concerns while delegating project and monorepo detection to the standard crate.
//!
//! # Separation of Concerns
//!
//! This module focuses exclusively on package.json operations. For project
//! structure detection and package discovery, use the appropriate tools from
//! `sublime_standard_tools`:
//!
//! - **Project Detection**: Use `ProjectDetector` from `sublime_standard_tools::project`
//! - **Monorepo Detection**: Use `MonorepoDetector` from `sublime_standard_tools::monorepo`
//! - **Finding Packages**: Use `MonorepoDetector::detect_packages()`
//! - **Filesystem Operations**: Use `AsyncFileSystem` from `sublime_standard_tools::filesystem`
//!
//! # Examples
//!
//! ## Reading and Parsing package.json
//!
//! ```rust
//! use sublime_pkg_tools::package::PackageJson;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let path = Path::new("./package.json");
//!
//! // Read and parse package.json directly
//! let package_json = PackageJson::read_from_path(&fs, path).await?;
//! println!("Package: {} v{}", package_json.name, package_json.version);
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating a Package from a Directory
//!
//! ```rust
//! use sublime_pkg_tools::package::Package;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let dir = Path::new("./packages/auth");
//!
//! // Create Package from directory
//! let package = Package::from_path(&fs, dir).await?;
//! println!("Package: {} at {}", package.name(), package.path().display());
//! # Ok(())
//! # }
//! ```
//!
//! ## Modifying package.json
//!
//! ```rust
//! use sublime_pkg_tools::package::PackageJsonEditor;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let path = Path::new("./package.json");
//!
//! // Create editor for modifications (preserves formatting)
//! let mut editor = PackageJsonEditor::new(fs, path).await?;
//! editor.set_version("1.2.4")?;
//! editor.update_dependency("lodash", "^4.17.21")?;
//! editor.save().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Validating package.json
//!
//! ```rust
//! use sublime_pkg_tools::package::validate_package_json;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let path = Path::new("./package.json");
//!
//! // Validate against Node.js specifications
//! let result = validate_package_json(&fs, path).await?;
//!
//! if result.has_errors() {
//!     for error in result.errors() {
//!         eprintln!("Error: {}", error.message);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Finding Packages in a Workspace
//!
//! For finding packages, use `sublime_standard_tools`:
//!
//! ```rust
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
//! use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
//! use sublime_pkg_tools::package::Package;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let root = Path::new("./");
//!
//! // Detect project type
//! let project_detector = ProjectDetector::with_filesystem(fs.clone());
//! let project = project_detector.detect(root, None).await?;
//!
//! if project.as_project_info().kind().is_monorepo() {
//!     // Monorepo - find all packages
//!     let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());
//!     let packages = monorepo_detector.detect_packages(root).await?;
//!
//!     for pkg in packages {
//!         println!("Found: {} at {}", pkg.name, pkg.absolute_path.display());
//!     }
//! } else {
//!     // Single package
//!     let package = Package::from_path(&fs, root).await?;
//!     println!("Single package: {}", package.name());
//! }
//! # Ok(())
//! # }
//! ```

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

use crate::error::PackageResult;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Validates a package.json file against Node.js specifications.
///
/// This function provides package.json-specific validation, applying Node.js
/// ecosystem rules and requirements. This is the only convenience function
/// maintained in this module as validation rules are specific to the
/// package.json specification and do not duplicate functionality from
/// `sublime_standard_tools`.
///
/// For other package.json operations, use the types directly:
/// - **Reading package.json**: Use `PackageJson::read_from_path()`
/// - **Creating Package**: Use `Package::from_path()`
/// - **Checking for package.json**: Use `AsyncFileSystem::exists(&path.join("package.json"))`
/// - **Finding packages**: Use `MonorepoDetector::detect_packages()` from `sublime_standard_tools`
/// - **Project detection**: Use `ProjectDetector::detect()` from `sublime_standard_tools`
///
/// # Arguments
///
/// * `filesystem` - The filesystem implementation to use for reading
/// * `path` - Path to the package.json file to validate
///
/// # Returns
///
/// A `ValidationResult` containing any issues found during validation.
/// The result includes errors (must be fixed) and warnings (should be reviewed).
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read from the filesystem
/// - The file is not valid JSON
/// - The validator cannot be initialized
///
/// # Examples
///
/// ## Basic Validation
///
/// ```rust
/// use sublime_pkg_tools::package::validate_package_json;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let path = Path::new("./package.json");
///
/// let result = validate_package_json(&fs, path).await?;
///
/// if result.has_errors() {
///     eprintln!("Validation failed with {} errors:", result.error_count());
///     for error in result.errors() {
///         eprintln!("  - {}: {}", error.field, error.message);
///     }
/// }
///
/// if result.has_warnings() {
///     println!("Validation warnings:");
///     for warning in result.warnings() {
///         println!("  - {}: {}", warning.field, warning.message);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Validation in CI/CD
///
/// ```rust
/// use sublime_pkg_tools::package::validate_package_json;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let path = Path::new("./package.json");
///
/// let result = validate_package_json(&fs, path).await?;
///
/// // Fail CI if there are any errors
/// if result.has_errors() {
///     eprintln!("Package.json validation failed!");
///     for error in result.errors() {
///         eprintln!("ERROR: {}", error.message);
///     }
///     std::process::exit(1);
/// }
///
/// // Log warnings but don't fail
/// for warning in result.warnings() {
///     println!("WARNING: {}", warning.message);
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Validating Multiple Packages
///
/// ```rust
/// use sublime_pkg_tools::package::validate_package_json;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let root = Path::new("./");
///
/// // Find all packages in monorepo
/// let detector = MonorepoDetector::with_filesystem(fs.clone());
/// let packages = detector.detect_packages(root).await?;
///
/// // Validate each package
/// for pkg in packages {
///     let pkg_json_path = pkg.absolute_path.join("package.json");
///
///     match validate_package_json(&fs, &pkg_json_path).await {
///         Ok(result) => {
///             if result.has_errors() {
///                 eprintln!("❌ {} has {} validation errors", pkg.name, result.error_count());
///             } else {
///                 println!("✅ {} is valid", pkg.name);
///             }
///         }
///         Err(e) => {
///             eprintln!("❌ Failed to validate {}: {}", pkg.name, e);
///         }
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
