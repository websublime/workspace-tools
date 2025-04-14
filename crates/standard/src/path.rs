//! # Project Path Detection Module
//!
//! This module provides functionality for detecting the root directory of a project
//! by searching for package manager lock files in the current directory and its ancestors.
//!
//! The module is particularly useful for tools that need to operate at the project root
//! level regardless of which subdirectory they're invoked from.

use std::{
    env,
    path::{Path, PathBuf},
};

/// Attempts to determine the root path of the current project.
///
/// This function tries to find a project root directory by examining the file system
/// for common package manager lock files. It will start from either a specified directory
/// or the current working directory, and traverse upward through parent directories until
/// a lock file is found or the file system root is reached.
///
/// # Arguments
///
/// * `root` - An optional starting directory to begin the search from.
///            If `None`, the current working directory is used.
///
/// # Returns
///
/// * `Some(PathBuf)` - The path to the detected project root directory
/// * `None` - This function always returns `Some` as it falls back to the
///            current directory if no project root is detected
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// // Detect project root from current directory
/// let root = get_project_root_path(None);
///
/// // Start detection from a specific directory
/// let specific_root = get_project_root_path(Some(PathBuf::from("/path/to/start")));
/// ```
pub fn get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf> {
    let env_dir = match root {
        Some(dir) => Ok(dir),
        None => env::current_dir(),
    };

    let Ok(current_dir) = env_dir else { return None };
    let current_path = current_dir.as_path();

    // Just use walk_reverse_dir directly, which returns None when it can't find project indicators
    walk_reverse_dir(current_path)
}

/// Recursively walks up the directory tree to find a project root.
///
/// This function searches for common package manager lock files to identify
/// the root directory of a project. It checks the current directory for
/// specific files, and if none are found, it recursively checks parent
/// directories until either a project root is found or the file system
/// root is reached.
///
/// # Lock Files Detected
///
/// The following lock files are recognized as markers of a project root:
/// - `package-lock.json` (npm)
/// - `npm-shrinkwrap.json` (npm)
/// - `yarn.lock` (yarn)
/// - `pnpm-lock.yaml` (pnpm)
/// - `bun.lockb` (bun)
///
/// # Arguments
///
/// * `path` - The directory path to check for project root indicators
///
/// # Returns
///
/// * `Some(PathBuf)` - The path where a lock file was found (project root)
/// * `None` - If no project root indicators were found in this path or any ancestors
///
/// # Implementation Details
///
/// The function uses a depth-first search, working upward through the directory
/// hierarchy. It stops and returns as soon as any recognized lock file is found.
fn walk_reverse_dir(path: &Path) -> Option<PathBuf> {
    let current_path = path.to_path_buf();
    let map_files = vec![
        ("package-lock.json", "npm"),
        ("npm-shrinkwrap.json", "npm"),
        ("yarn.lock", "yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        ("bun.lockb", "bun"),
    ];

    for (file, _) in map_files {
        let lock_file = current_path.join(file);

        if lock_file.exists() {
            return Some(current_path);
        }
    }

    if let Some(parent) = path.parent() {
        return walk_reverse_dir(parent);
    }

    None
}
