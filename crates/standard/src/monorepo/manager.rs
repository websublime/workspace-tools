//! # Package Manager Implementation
//!
//! ## What
//! This file implements functionality for detecting and working with different
//! package managers like npm, yarn, pnpm, bun, and jsr.
//!
//! ## How
//! The implementation provides methods for detecting which package manager is used
//! in a project based on lock files and accessing package manager specific information.
//!
//! ## Why
//! Different package managers have different lock files and commands. This abstraction
//! allows the crate to work with any supported package manager in a consistent way.

use super::types::{PackageManager, PackageManagerKind};
use crate::error::{MonorepoError, MonorepoResult};
use std::path::{Path, PathBuf};

impl PackageManager {
    /// Creates a new PackageManager instance with the specified kind and root directory.
    ///
    /// # Arguments
    ///
    /// * `kind` - The package manager type
    /// * `root` - The root directory path where the package manager's files are located
    ///
    /// # Returns
    ///
    /// A new PackageManager instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::types::{PackageManager, PackageManagerKind};
    ///
    /// let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");
    /// ```
    #[must_use]
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self {
        Self { kind, root: root.into() }
    }

    /// Detects which package manager is being used in the specified path.
    ///
    /// Checks for lock files in the following order:
    /// 1. Bun (bun.lockb)
    /// 2. pnpm (pnpm-lock.yaml)
    /// 3. Yarn (yarn.lock)
    /// 4. npm (package-lock.json or npm-shrinkwrap.json)
    /// 5. JSR (jsr.json)
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to check for package manager lock files
    ///
    /// # Returns
    ///
    /// * `Ok(PackageManager)` - If a package manager is detected
    /// * `Err(MonorepoError)` - If no package manager could be detected
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::types::PackageManager;
    /// use sublime_standard_tools::error::MonorepoResult;
    ///
    /// # fn example() -> MonorepoResult<()> {
    /// let project_dir = Path::new(".");
    /// let package_manager = PackageManager::detect(project_dir)?;
    /// println!("Detected package manager: {:?}", package_manager.kind());
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect(path: impl AsRef<Path>) -> MonorepoResult<Self> {
        let path = path.as_ref();

        if path.join(PackageManagerKind::Bun.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Bun, path));
        }
        if path.join(PackageManagerKind::Pnpm.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Pnpm, path));
        }
        if path.join(PackageManagerKind::Yarn.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Yarn, path));
        }
        if path.join(PackageManagerKind::Npm.lock_file()).exists()
            || path.join("npm-shrinkwrap.json").exists()
        // Check both npm lock files
        {
            return Ok(Self::new(PackageManagerKind::Npm, path));
        }
        if path.join(PackageManagerKind::Jsr.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Jsr, path));
        }

        Err(MonorepoError::ManagerNotFound)
    }

    /// Returns the kind of package manager.
    ///
    /// # Returns
    ///
    /// The PackageManagerKind enum value representing the type of package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::types::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Yarn, "project/path");
    /// assert_eq!(manager.kind(), PackageManagerKind::Yarn);
    /// ```
    #[must_use]
    pub fn kind(&self) -> PackageManagerKind {
        self.kind
    }

    /// Returns the root directory path of the package manager.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::types::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");
    /// assert_eq!(manager.root(), Path::new("/project/root"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the full path to the lock file for this package manager.
    ///
    /// For npm, this handles the special case where both package-lock.json
    /// and npm-shrinkwrap.json might exist.
    ///
    /// # Returns
    ///
    /// A PathBuf with the complete path to the lock file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::monorepo::types::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Yarn, "/project/root");
    /// assert_eq!(manager.lock_file_path(), PathBuf::from("/project/root/yarn.lock"));
    /// ```
    #[must_use]
    pub fn lock_file_path(&self) -> PathBuf {
        // Handle npm's alternative lock file if necessary, though detect prioritizes package-lock.json
        let lock_file = if self.kind == PackageManagerKind::Npm
            && !self.root.join(PackageManagerKind::Npm.lock_file()).exists()
            && self.root.join("npm-shrinkwrap.json").exists()
        {
            "npm-shrinkwrap.json"
        } else {
            self.kind.lock_file()
        };
        self.root.join(lock_file)
    }
}
