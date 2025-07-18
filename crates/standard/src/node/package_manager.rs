//! # Node.js Package Manager Abstractions
//!
//! ## What
//! This file defines the fundamental types and abstractions for working with
//! Node.js package managers. It provides the `PackageManagerKind` enum that
//! represents different package manager implementations and the `PackageManager`
//! struct that encapsulates package manager detection and functionality.
//!
//! ## How
//! The module defines types that model the various Node.js package managers
//! available in the ecosystem (npm, yarn, pnpm, bun, jsr). Each package manager
//! has specific characteristics like lock files, commands, and configuration
//! formats that are captured in these abstractions. The types are designed
//! to be generic and reusable across all Node.js project types.
//!
//! ## Why
//! Previously, package manager types were incorrectly placed in the monorepo
//! module, creating conceptual dependencies where simple projects needed to
//! import monorepo-specific logic. This module provides the correct location
//! for these fundamental Node.js concepts, enabling clean separation of concerns
//! and reusability across all project types.

use std::path::{Path, PathBuf};
use crate::error::{Error, MonorepoError, Result};

/// Represents the type of package manager used in a Node.js project.
///
/// Different package managers have different characteristics including lock files,
/// command syntax, workspace configurations, and performance characteristics.
/// This enum captures these variations to enable package-manager-specific
/// processing and functionality.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::node::PackageManagerKind;
///
/// let npm = PackageManagerKind::Npm;
/// assert_eq!(npm.command(), "npm");
/// assert_eq!(npm.lock_file(), "package-lock.json");
///
/// let yarn = PackageManagerKind::Yarn;
/// assert_eq!(yarn.command(), "yarn");
/// assert_eq!(yarn.lock_file(), "yarn.lock");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerKind {
    /// npm package manager (default for Node.js)
    ///
    /// The standard package manager that comes with Node.js installations.
    /// Uses package-lock.json for dependency locking and npm-shrinkwrap.json
    /// for publishing. Supports workspaces since version 7.
    Npm,

    /// Yarn package manager
    ///
    /// A fast, reliable, and secure dependency management tool created by
    /// Facebook. Uses yarn.lock for dependency locking and has built-in
    /// workspace support. Available in both v1 (Classic) and v2+ (Berry).
    Yarn,

    /// pnpm package manager (performance-oriented)
    ///
    /// A fast, disk space efficient package manager that uses hard links
    /// and symlinks to save disk space. Uses pnpm-lock.yaml for dependency
    /// locking and has excellent monorepo/workspace support.
    Pnpm,

    /// Bun package manager and runtime
    ///
    /// A fast all-in-one JavaScript runtime, bundler, test runner, and
    /// package manager. Uses bun.lockb (binary format) for dependency
    /// locking and provides npm-compatible API.
    Bun,

    /// Jsr package manager and runtime
    ///
    /// A modern package registry and runtime for TypeScript and JavaScript.
    /// Designed for modern JavaScript development with native TypeScript
    /// support and web-standard APIs.
    Jsr,
}

impl PackageManagerKind {
    /// Returns the command name for this package manager.
    ///
    /// This is the executable name that would be used in shell commands
    /// to invoke the package manager.
    ///
    /// # Returns
    ///
    /// The command string for the package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.command(), "npm");
    /// assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
    /// assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
    /// assert_eq!(PackageManagerKind::Bun.command(), "bun");
    /// assert_eq!(PackageManagerKind::Jsr.command(), "jsr");
    /// ```
    #[must_use]
    pub fn command(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Jsr => "jsr",
        }
    }

    /// Returns the lock file name for this package manager.
    ///
    /// Lock files contain the exact versions of all dependencies and
    /// are used to ensure reproducible installations across environments.
    ///
    /// # Returns
    ///
    /// The lock file name for the package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
    /// assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
    /// assert_eq!(PackageManagerKind::Pnpm.lock_file(), "pnpm-lock.yaml");
    /// assert_eq!(PackageManagerKind::Bun.lock_file(), "bun.lockb");
    /// assert_eq!(PackageManagerKind::Jsr.lock_file(), "jsr.json");
    /// ```
    #[must_use]
    pub fn lock_file(&self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json",
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
            Self::Jsr => "jsr.json",
        }
    }

    /// Returns a human-readable name for this package manager.
    ///
    /// This provides a consistent display name that can be used in
    /// user interfaces, logging, and error messages.
    ///
    /// # Returns
    ///
    /// A human-readable name for the package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.name(), "npm");
    /// assert_eq!(PackageManagerKind::Yarn.name(), "yarn");
    /// assert_eq!(PackageManagerKind::Pnpm.name(), "pnpm");
    /// assert_eq!(PackageManagerKind::Bun.name(), "bun");
    /// assert_eq!(PackageManagerKind::Jsr.name(), "jsr");
    /// ```
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Jsr => "jsr",
        }
    }

    /// Checks if this package manager supports workspaces natively.
    ///
    /// Workspace support enables monorepo functionality where multiple
    /// packages can be managed within a single repository with shared
    /// dependencies and cross-package linking.
    ///
    /// # Returns
    ///
    /// `true` if the package manager supports workspaces, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::PackageManagerKind;
    ///
    /// assert!(PackageManagerKind::Npm.supports_workspaces());
    /// assert!(PackageManagerKind::Yarn.supports_workspaces());
    /// assert!(PackageManagerKind::Pnpm.supports_workspaces());
    /// assert!(PackageManagerKind::Bun.supports_workspaces());
    /// assert!(!PackageManagerKind::Jsr.supports_workspaces()); // Not primarily a workspace manager
    /// ```
    #[must_use]
    pub fn supports_workspaces(&self) -> bool {
        match self {
            Self::Npm | Self::Yarn | Self::Pnpm | Self::Bun => true,
            Self::Jsr => false,
        }
    }

    /// Returns the workspace configuration file for this package manager.
    ///
    /// Different package managers use different files to configure workspace
    /// behavior and define which directories contain packages.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - The workspace configuration file name
    /// * `None` - If the package manager doesn't support workspaces or uses package.json
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.workspace_config_file(), None); // Uses package.json
    /// assert_eq!(PackageManagerKind::Yarn.workspace_config_file(), None); // Uses package.json
    /// assert_eq!(PackageManagerKind::Pnpm.workspace_config_file(), Some("pnpm-workspace.yaml"));
    /// assert_eq!(PackageManagerKind::Bun.workspace_config_file(), None); // Uses package.json
    /// assert_eq!(PackageManagerKind::Jsr.workspace_config_file(), None);
    /// ```
    #[must_use]
    pub fn workspace_config_file(&self) -> Option<&'static str> {
        match self {
            Self::Npm | Self::Yarn | Self::Bun | Self::Jsr => None, // Uses package.json workspaces field or doesn't support workspaces
            Self::Pnpm => Some("pnpm-workspace.yaml"),
        }
    }
}

/// Represents a package manager detected in a Node.js project.
///
/// This struct encapsulates information about a detected package manager
/// including its type and the root directory where it was found. It provides
/// methods for accessing package manager properties and performing common
/// operations.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
///
/// // Create a package manager representation
/// let manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");
/// assert_eq!(manager.kind(), PackageManagerKind::Npm);
/// assert_eq!(manager.root(), Path::new("/project/root"));
/// assert_eq!(manager.command(), "npm");
/// ```
#[derive(Debug, Clone)]
pub struct PackageManager {
    /// The type of package manager
    pub(crate) kind: PackageManagerKind,
    /// The root directory where the package manager was detected
    pub(crate) root: PathBuf,
}

impl PackageManager {
    /// Creates a new PackageManager instance.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of package manager
    /// * `root` - The root directory path where the package manager is used
    ///
    /// # Returns
    ///
    /// A new PackageManager instance configured with the specified type and root.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Yarn, "/path/to/project");
    /// assert_eq!(manager.kind(), PackageManagerKind::Yarn);
    /// ```
    #[must_use]
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            root: root.into(),
        }
    }

    /// Detects which package manager is being used in the specified path.
    ///
    /// Checks for lock files in order of preference:
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
    /// # Errors
    ///
    /// Returns a [`MonorepoError::ManagerNotFound`] if no package manager
    /// lock files are found in the specified path.
    ///
    /// # Returns
    ///
    /// * `Ok(PackageManager)` - If a package manager is detected
    /// * `Err(Error)` - If no package manager could be detected
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::node::PackageManager;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let project_dir = Path::new(".");
    /// let package_manager = PackageManager::detect(project_dir)?;
    /// println!("Detected package manager: {:?}", package_manager.kind());
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Check lock files in order of preference
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
        {
            return Ok(Self::new(PackageManagerKind::Npm, path));
        }
        if path.join(PackageManagerKind::Jsr.lock_file()).exists() {
            return Ok(Self::new(PackageManagerKind::Jsr, path));
        }

        Err(Error::Monorepo(MonorepoError::ManagerNotFound))
    }

    /// Returns the package manager kind.
    ///
    /// # Returns
    ///
    /// The `PackageManagerKind` representing the type of package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Pnpm, "/project");
    /// assert_eq!(manager.kind(), PackageManagerKind::Pnpm);
    /// ```
    #[must_use]
    pub fn kind(&self) -> PackageManagerKind {
        self.kind
    }

    /// Returns the root directory path.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Bun, "/project/root");
    /// assert_eq!(manager.root(), Path::new("/project/root"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the command name for this package manager.
    ///
    /// This is a convenience method that delegates to the kind's command method.
    ///
    /// # Returns
    ///
    /// The command string for the package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Yarn, "/project");
    /// assert_eq!(manager.command(), "yarn");
    /// ```
    #[must_use]
    pub fn command(&self) -> &'static str {
        self.kind.command()
    }

    /// Returns the lock file name for this package manager.
    ///
    /// This is a convenience method that delegates to the kind's lock_file method.
    ///
    /// # Returns
    ///
    /// The lock file name for the package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Pnpm, "/project");
    /// assert_eq!(manager.lock_file(), "pnpm-lock.yaml");
    /// ```
    #[must_use]
    pub fn lock_file(&self) -> &'static str {
        self.kind.lock_file()
    }

    /// Returns the full path to the lock file.
    ///
    /// This combines the root directory with the lock file name to provide
    /// the complete path where the lock file should be located. For npm,
    /// this handles the special case where both package-lock.json and
    /// npm-shrinkwrap.json might exist.
    ///
    /// # Returns
    ///
    /// A PathBuf representing the full path to the lock file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let manager = PackageManager::new(PackageManagerKind::Npm, "/project");
    /// let expected = PathBuf::from("/project/package-lock.json");
    /// assert_eq!(manager.lock_file_path(), expected);
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

    /// Checks if this package manager supports workspaces.
    ///
    /// This is a convenience method that delegates to the kind's supports_workspaces method.
    ///
    /// # Returns
    ///
    /// `true` if the package manager supports workspaces, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
    /// assert!(npm_manager.supports_workspaces());
    ///
    /// let jsr_manager = PackageManager::new(PackageManagerKind::Jsr, "/project");
    /// assert!(!jsr_manager.supports_workspaces());
    /// ```
    #[must_use]
    pub fn supports_workspaces(&self) -> bool {
        self.kind.supports_workspaces()
    }

    /// Returns the workspace configuration file path if applicable.
    ///
    /// For package managers that use a separate workspace configuration file
    /// (like pnpm with pnpm-workspace.yaml), this returns the full path to
    /// that file. For package managers that use package.json for workspace
    /// configuration, this returns None.
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - If the package manager uses a separate workspace config file
    /// * `None` - If the package manager uses package.json or doesn't support workspaces
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    ///
    /// let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
    /// assert_eq!(npm_manager.workspace_config_path(), None);
    ///
    /// let pnpm_manager = PackageManager::new(PackageManagerKind::Pnpm, "/project");
    /// let expected = Some(PathBuf::from("/project/pnpm-workspace.yaml"));
    /// assert_eq!(pnpm_manager.workspace_config_path(), expected);
    /// ```
    #[must_use]
    pub fn workspace_config_path(&self) -> Option<PathBuf> {
        self.kind
            .workspace_config_file()
            .map(|file| self.root.join(file))
    }
}