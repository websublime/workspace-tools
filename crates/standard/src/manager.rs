//! # Package Manager Module
//!
//! This module provides functionality for detecting and working with
//! Node.js package managers (npm, yarn, pnpm, and bun).
//!
//! It includes the `CorePackageManager` enum for representing different
//! package managers and functionality to detect which manager is being
//! used in a project by examining lock files.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::{Display, Formatter, Result as StdResult},
    path::Path,
};
use thiserror::Error;

/// Represents supported package managers for JavaScript/TypeScript projects.
///
/// This enum defines the package managers that can be detected and used
/// within the application. It's serializable and deserializable with
/// lowercase string representations for interoperability with other systems.
///
/// # Variants
///
/// * `Npm` - Node Package Manager, the default package manager for Node.js
/// * `Yarn` - Facebook's alternative package manager with features like workspaces
/// * `Pnpm` - Performance NPM, a disk space efficient package manager
/// * `Bun` - All-in-one JavaScript runtime and package manager
///
/// # Serialization
///
/// The enum serializes to lowercase strings ("npm", "yarn", "pnpm", "bun")
/// to maintain compatibility with JSON and other string-based formats.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorePackageManager {
    /// Node Package Manager (npm) - the default package manager that comes with Node.js
    Npm,
    /// Yarn - Facebook's alternative to npm with additional features like workspaces
    Yarn,
    /// Performance NPM (pnpm) - a disk space efficient alternative that uses symlinks
    Pnpm,
    /// Bun - a modern all-in-one JavaScript runtime, transpiler, bundler and package manager
    Bun,
}

/// Error that can occur when trying to parse a package manager from a string.
///
/// This enum represents the different error cases that might occur during
/// the parsing or conversion process from strings to the `CorePackageManager` enum.
///
/// # Variants
///
/// * `ParsePackageManagerError` - Occurs when attempting to parse an unsupported
///   package manager name string.
#[derive(Error, Debug)]
pub enum CorePackageManagerError {
    /// Error returned when an unsupported package manager name is provided.
    /// The inner string contains the name of the unsupported package manager.
    #[error("Unsupported package manager: {0}")]
    ParsePackageManagerError(String),
}

/// Converts a string slice to a `CorePackageManager`.
///
/// This implementation allows the conversion from a string reference to
/// the corresponding `CorePackageManager` enum variant.
///
/// # Arguments
///
/// * `manager` - A string slice containing the package manager name
///
/// # Returns
///
/// * `Ok(CorePackageManager)` - If the string matches a supported package manager
/// * `Err(CorePackageManagerError)` - If the string doesn't match any supported package manager
///
/// # Examples
///
/// ```
/// use std::convert::TryFrom;
///
/// let npm = CorePackageManager::try_from("npm").expect("Valid package manager");
/// assert!(CorePackageManager::try_from("invalid").is_err());
/// ```
impl TryFrom<&str> for CorePackageManager {
    type Error = CorePackageManagerError;

    fn try_from(manager: &str) -> Result<Self, Self::Error> {
        match manager {
            "npm" => Ok(Self::Npm),
            "yarn" => Ok(Self::Yarn),
            "pnpm" => Ok(Self::Pnpm),
            "bun" => Ok(Self::Bun),
            _ => Err(CorePackageManagerError::ParsePackageManagerError(manager.to_string())),
        }
    }
}

/// Converts a `String` to a `CorePackageManager`.
///
/// This implementation delegates to the `&str` implementation to avoid duplication.
/// It allows direct conversion from owned strings to `CorePackageManager` variants.
///
/// # Arguments
///
/// * `manager` - A `String` containing the package manager name
///
/// # Returns
///
/// * `Ok(CorePackageManager)` - If the string matches a supported package manager
/// * `Err(CorePackageManagerError)` - If the string doesn't match any supported package manager
///
/// # Examples
///
/// ```
/// use std::convert::TryFrom;
///
/// let yarn = CorePackageManager::try_from(String::from("yarn")).expect("Valid package manager");
/// assert!(CorePackageManager::try_from(String::from("webpack")).is_err());
/// ```
impl TryFrom<String> for CorePackageManager {
    type Error = CorePackageManagerError;

    fn try_from(manager: String) -> Result<Self, Self::Error> {
        Self::try_from(manager.as_str())
    }
}

/// Implements string reference conversion for `CorePackageManagerError`.
///
/// This implementation allows the error to be represented as a static string,
/// which can be useful for error categorization and handling.
///
/// # Returns
///
/// A static string identifier for the error type:
/// * `"UnsupportedPackageManager"` - For all parse errors
///
/// # Examples
///
/// ```
/// use std::convert::TryFrom;
/// use std::convert::AsRef;
///
/// let err = CorePackageManager::try_from("invalid").unwrap_err();
/// assert_eq!(err.as_ref(), "UnsupportedPackageManager");
/// ```
impl AsRef<str> for CorePackageManagerError {
    fn as_ref(&self) -> &str {
        match self {
            CorePackageManagerError::ParsePackageManagerError(_) => "UnsupportedPackageManager",
        }
    }
}

/// Implements the `Clone` trait for `CorePackageManagerError`.
///
/// This implementation allows `CorePackageManagerError` instances to be duplicated,
/// which is useful when you need to pass or store the error in multiple places.
///
/// # Implementation Details
///
/// The `clone` method creates a new instance of `CorePackageManagerError` with the
/// same variant and inner data as the original. For the `ParsePackageManagerError`
/// variant, it clones the contained string that holds the name of the unsupported
/// package manager.
///
/// # Examples
///
/// ```
/// use std::convert::TryFrom;
///
/// let original_err = CorePackageManager::try_from("invalid").unwrap_err();
/// let cloned_err = original_err.clone();
///
/// // Both errors represent the same underlying issue
/// assert_eq!(format!("{}", original_err), format!("{}", cloned_err));
/// ```
impl Clone for CorePackageManagerError {
    fn clone(&self) -> Self {
        match self {
            CorePackageManagerError::ParsePackageManagerError(err) => {
                CorePackageManagerError::ParsePackageManagerError(err.clone())
            }
        }
    }
}

/// Implements the `Display` trait for `CorePackageManager`.
///
/// This allows `CorePackageManager` instances to be formatted as strings
/// using the `{}` format specifier. The implementation returns the lowercase
/// string representation of the package manager.
///
/// # Examples
///
/// ```
/// let manager = CorePackageManager::Yarn;
/// println!("Using package manager: {}", manager); // Prints "Using package manager: yarn"
/// ```
impl Display for CorePackageManager {
    fn fmt(&self, f: &mut Formatter) -> StdResult {
        match self {
            Self::Npm => write!(f, "npm"),
            Self::Yarn => write!(f, "yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Bun => write!(f, "bun"),
        }
    }
}

/// Detects which package manager is being used in a workspace by examining lock files.
///
/// This function identifies the package manager by checking for the presence of
/// specific lock files in the provided directory path. If no lock file is found,
/// it recursively searches parent directories until it reaches the root of the file system.
///
/// # Lock files checked
///
/// * `package-lock.json` - Indicates npm
/// * `npm-shrinkwrap.json` - Indicates npm
/// * `yarn.lock` - Indicates Yarn
/// * `pnpm-lock.yaml` - Indicates pnpm
/// * `bun.lockb` - Indicates Bun
///
/// # Arguments
///
/// * `path` - A reference to a `Path` representing the directory to check
///
/// # Returns
///
/// * `Some(CorePackageManager)` - If a package manager is detected
/// * `None` - If no package manager could be detected in the path or any of its parents
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let project_dir = Path::new("/path/to/project");
/// if let Some(manager) = detect_package_manager(project_dir) {
///     println!("Detected package manager: {}", manager);
/// } else {
///     println!("No package manager detected");
/// }
/// ```
pub fn detect_package_manager(path: &Path) -> Option<CorePackageManager> {
    let package_manager_files = HashMap::from([
        ("package-lock.json", CorePackageManager::Npm),
        ("npm-shrinkwrap.json", CorePackageManager::Npm),
        ("yarn.lock", CorePackageManager::Yarn),
        ("pnpm-lock.yaml", CorePackageManager::Pnpm),
        ("bun.lockb", CorePackageManager::Bun),
    ]);

    for (file, package_manager) in package_manager_files {
        let lock_file = path.join(file);

        if lock_file.exists() {
            return Some(package_manager);
        }
    }

    if let Some(parent) = path.parent() {
        return detect_package_manager(parent);
    }

    None
}
