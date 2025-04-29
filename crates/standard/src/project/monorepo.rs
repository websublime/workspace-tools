//! Monorepo detection and management for Node.js projects.
//!
//! What:
//! This module provides functionality for detecting and working with monorepo
//! structures, including identifying monorepo roots and workspace packages across
//! different monorepo tools (Lerna, Yarn Workspaces, pnpm Workspaces, Nx, Turborepo).
//!
//! Who:
//! Used by developers who need to:
//! - Detect monorepo structures in Node.js projects
//! - Identify all workspace packages within a monorepo
//! - Support different monorepo tools transparently
//! - Work with package interdependencies
//!
//! Why:
//! Proper monorepo detection is essential for:
//! - Supporting modern JavaScript development workflows
//! - Enabling package-specific operations
//! - Managing workspace-level dependencies
//! - Implementing cross-package operations

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

use glob::glob;
use serde::{Deserialize, Serialize};

use super::{FileSystem, FileSystemManager, PackageJson};
use crate::error::{FileSystemError, StandardError, StandardResult};

/// Types of monorepo tools supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonorepoKind {
    /// Lerna-based monorepo
    Lerna,
    /// Yarn Workspaces monorepo
    YarnWorkspaces,
    /// pnpm Workspaces monorepo
    PnpmWorkspaces,
    /// Nx monorepo
    Nx,
    /// Turborepo monorepo
    Turborepo,
    /// Rush monorepo
    Rush,
    /// Custom monorepo (generic structure detection)
    Custom,
}

impl MonorepoKind {
    /// Gets the name of the monorepo tool
    ///
    /// # Returns
    ///
    /// The name of the monorepo tool
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::MonorepoKind;
    ///
    /// assert_eq!(MonorepoKind::Lerna.name(), "Lerna");
    /// assert_eq!(MonorepoKind::YarnWorkspaces.name(), "Yarn Workspaces");
    /// ```
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Lerna => "Lerna",
            Self::YarnWorkspaces => "Yarn Workspaces",
            Self::PnpmWorkspaces => "pnpm Workspaces",
            Self::Nx => "Nx",
            Self::Turborepo => "Turborepo",
            Self::Rush => "Rush",
            Self::Custom => "Custom Monorepo",
        }
    }

    /// Gets the configuration file for this monorepo kind
    ///
    /// # Returns
    ///
    /// The name of the configuration file for this monorepo kind
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::MonorepoKind;
    ///
    /// assert_eq!(MonorepoKind::Lerna.config_file(), "lerna.json");
    /// assert_eq!(MonorepoKind::Nx.config_file(), "nx.json");
    /// ```
    #[must_use]
    pub fn config_file(self) -> &'static str {
        match self {
            Self::Lerna => "lerna.json",
            Self::YarnWorkspaces | Self::Custom => "package.json", // Uses workspaces field in package.json
            Self::PnpmWorkspaces => "pnpm-workspace.yaml",
            Self::Nx => "nx.json",
            Self::Turborepo => "turbo.json",
            Self::Rush => "rush.json",
        }
    }
}

/// Lerna configuration structure
#[derive(Debug, Clone, Deserialize)]
struct LernaConfig {
    /// Package locations (glob patterns)
    #[serde(default)]
    packages: Vec<String>,
    /// Use Yarn Workspaces for package management
    #[serde(default)]
    #[serde(rename = "useWorkspaces")]
    use_workspaces: bool,
}

/// Workspace configuration in package.json (Yarn and npm)
#[derive(Debug, Clone, Deserialize)]
struct WorkspacesConfig {
    /// Package locations (glob patterns)
    #[serde(default)]
    workspaces: Option<WorkspacesPatterns>,
}

/// Workspaces patterns - can be an array or an object with packages array
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum WorkspacesPatterns {
    /// Workspace glob patterns as array
    Array(Vec<String>),
    /// Workspace config with patterns in "packages" field
    Object { packages: Vec<String> },
}

/// pnpm workspace configuration
#[derive(Debug, Clone, Deserialize)]
struct PnpmWorkspaceConfig {
    /// Package locations (glob patterns)
    packages: Vec<String>,
}

/// Nx configuration structure
#[derive(Debug, Clone, Deserialize)]
struct NxConfig {
    /// Package locations or configuration
    #[serde(default)]
    projects: HashMap<String, NxProjectConfig>,
}

/// Nx project configuration
#[derive(Debug, Clone, Deserialize)]
struct NxProjectConfig {
    /// Project root path (may be relative)
    #[serde(default)]
    root: Option<String>,
}

/// Rush configuration structure
#[derive(Debug, Clone, Deserialize)]
struct RushConfig {
    /// Projects in the Rush monorepo
    #[serde(default)]
    projects: Vec<RushProject>,
}

/// Rush project configuration
#[derive(Debug, Clone, Deserialize)]
struct RushProject {
    /// Package name
    #[serde(default)]
    #[serde(rename = "packageName")]
    package_name: String,
    /// Project folder path
    #[serde(rename = "projectFolder")]
    project_folder: String,
}

/// Workspace package information
#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePackage {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub version: String,
    /// Location of the package relative to the monorepo root
    pub location: PathBuf,
    /// Absolute path to the package
    pub absolute_path: PathBuf,
    /// Direct dependencies within the workspace
    pub workspace_dependencies: Vec<String>,
}

/// Monorepo information container
#[derive(Debug, Clone)]
pub struct MonorepoInfo {
    /// Type of monorepo detected
    kind: MonorepoKind,
    /// Root directory of the monorepo
    root: PathBuf,
    /// Package locations (paths relative to root)
    packages: Vec<WorkspacePackage>,
    /// Map of package names to their locations
    name_to_package: HashMap<String, usize>,
}

impl MonorepoInfo {
    /// Creates a new MonorepoInfo instance
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of monorepo detected
    /// * `root` - The root directory of the monorepo
    /// * `packages` - The packages in the monorepo
    ///
    /// # Returns
    ///
    /// A new MonorepoInfo instance
    fn new(kind: MonorepoKind, root: PathBuf, packages: Vec<WorkspacePackage>) -> Self {
        // Build name-to-package map for quick lookups
        let mut name_to_package = HashMap::new();
        for (i, package) in packages.iter().enumerate() {
            name_to_package.insert(package.name.clone(), i);
        }

        Self { kind, root, packages, name_to_package }
    }

    /// Gets the kind of monorepo detected
    ///
    /// # Returns
    ///
    /// The monorepo kind
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// println!("Detected monorepo: {}", monorepo.kind().name());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn kind(&self) -> MonorepoKind {
        self.kind
    }

    /// Gets the root directory of the monorepo
    ///
    /// # Returns
    ///
    /// The root directory path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// println!("Monorepo root: {}", monorepo.root().display());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Gets all packages in the monorepo
    ///
    /// # Returns
    ///
    /// A slice of the workspace packages
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// println!("Found {} packages:", monorepo.packages().len());
    /// for package in monorepo.packages() {
    ///     println!("  - {} at {}", package.name, package.location.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn packages(&self) -> &[WorkspacePackage] {
        &self.packages
    }

    /// Gets a package by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package to find
    ///
    /// # Returns
    ///
    /// The package if found, or None if not
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// if let Some(package) = monorepo.get_package("my-package") {
    ///     println!("Found package at {}", package.absolute_path.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage> {
        self.name_to_package.get(name).map(|&i| &self.packages[i])
    }

    /// Finds all packages that depend on a specific package
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// A vector of packages that depend on the specified package
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// let dependents = monorepo.find_dependents("common-lib");
    /// println!("Packages that depend on common-lib:");
    /// for pkg in dependents {
    ///     println!("  - {}", pkg.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn find_dependents(&self, package_name: &str) -> Vec<&WorkspacePackage> {
        self.packages
            .iter()
            .filter(|pkg| pkg.workspace_dependencies.iter().any(|dep| dep == package_name))
            .collect()
    }

    /// Finds a package containing a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to find a package for
    ///
    /// # Returns
    ///
    /// The package containing the path, or None if not in any package
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new(FileSystemManager::new());
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// let file_path = Path::new("packages/ui-components/src/Button.tsx");
    /// if let Some(package) = monorepo.find_package_for_path(file_path) {
    ///     println!("File belongs to package: {}", package.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn find_package_for_path(&self, path: &Path) -> Option<&WorkspacePackage> {
        // Normalize and make path absolute for comparison
        let abs_path = if path.is_absolute() { path.to_path_buf() } else { self.root.join(path) };

        self.packages.iter().find(|pkg| abs_path.starts_with(&pkg.absolute_path))
    }
}

/// Detector for monorepo structures
#[derive(Debug)]
pub struct MonorepoDetector<F: FileSystem = FileSystemManager> {
    /// File system interface for filesystem operations
    fs: F,
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new MonorepoDetector with the default filesystem manager
    ///
    /// # Returns
    ///
    /// A new MonorepoDetector instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::MonorepoDetector;
    ///
    /// let detector = MonorepoDetector::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { fs: FileSystemManager::new() }
    }
}

impl<F: FileSystem> MonorepoDetector<F> {
    /// Creates a new MonorepoDetector with a custom filesystem
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new MonorepoDetector instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{MonorepoDetector, FileSystemManager};
    ///
    /// let fs = FileSystemManager::new();
    /// let detector = MonorepoDetector::with_filesystem(fs);
    /// ```
    #[must_use]
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    /// Detects if a directory is the root of a monorepo
    ///
    /// # Arguments
    ///
    /// * `path` - The directory to check
    ///
    /// # Returns
    ///
    /// The monorepo kind if detected, or None if not a monorepo
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::MonorepoDetector;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    ///
    /// if let Some(kind) = detector.is_monorepo_root("./my-project").await? {
    ///     println!("Detected {} monorepo", kind.name());
    /// } else {
    ///     println!("Not a monorepo");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> StandardResult<Option<MonorepoKind>> {
        let path = path.as_ref();

        // Check for each monorepo configuration file
        let lerna_path = path.join(MonorepoKind::Lerna.config_file());
        if self.fs.exists(&lerna_path) {
            return Ok(Some(MonorepoKind::Lerna));
        }

        let nx_path = path.join(MonorepoKind::Nx.config_file());
        if self.fs.exists(&nx_path) {
            return Ok(Some(MonorepoKind::Nx));
        }

        let turborepo_path = path.join(MonorepoKind::Turborepo.config_file());
        if self.fs.exists(&turborepo_path) {
            return Ok(Some(MonorepoKind::Turborepo));
        }

        let pnpm_workspace_path = path.join(MonorepoKind::PnpmWorkspaces.config_file());
        if self.fs.exists(&pnpm_workspace_path) {
            return Ok(Some(MonorepoKind::PnpmWorkspaces));
        }

        let rush_path = path.join(MonorepoKind::Rush.config_file());
        if self.fs.exists(&rush_path) {
            return Ok(Some(MonorepoKind::Rush));
        }

        // Check for Yarn Workspaces in package.json
        let package_json_path = path.join("package.json");
        if self.fs.exists(&package_json_path) {
            let content = self.fs.read_file_string(&package_json_path)?;
            if let Ok(package_json) = serde_json::from_str::<WorkspacesConfig>(&content) {
                if package_json.workspaces.is_some() {
                    return Ok(Some(MonorepoKind::YarnWorkspaces));
                }
            }

            // Custom monorepo detection (multiple package.json files or packages directory)
            if self.has_multiple_packages(path) {
                return Ok(Some(MonorepoKind::Custom));
            }
        }

        Ok(None)
    }

    /// Finds the root of a monorepo by scanning parent directories
    ///
    /// # Arguments
    ///
    /// * `start_path` - The starting directory to search from
    ///
    /// # Returns
    ///
    /// The path to the monorepo root and its kind, or None if not in a monorepo
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::MonorepoDetector;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    ///
    /// // Look for a monorepo from the current directory
    /// if let Some((root, kind)) = detector.find_monorepo_root(".").await? {
    ///     println!("Found {} monorepo at {}", kind.name(), root.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> StandardResult<Option<(PathBuf, MonorepoKind)>> {
        let start_path = start_path.as_ref();

        // Check if the current directory is a monorepo root
        if let Some(kind) = self.is_monorepo_root(start_path)? {
            return Ok(Some((start_path.to_path_buf(), kind)));
        }

        // Walk up the directory tree
        let mut current = Some(start_path);
        while let Some(path) = current {
            if let Some(kind) = self.is_monorepo_root(path)? {
                return Ok(Some((path.to_path_buf(), kind)));
            }
            current = path.parent();
        }

        Ok(None)
    }

    /// Detects and analyzes a monorepo structure
    ///
    /// # Arguments
    ///
    /// * `path` - The directory to analyze (can be any subdirectory of a monorepo)
    ///
    /// # Returns
    ///
    /// Monorepo information including root, type, and packages
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::MonorepoDetector;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    ///
    /// // Detect monorepo structure
    /// let monorepo = detector.detect_monorepo(".").await?;
    ///
    /// println!("Detected {} monorepo at {}", monorepo.kind().name(), monorepo.root().display());
    /// println!("Found {} packages", monorepo.packages().len());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::manual_let_else)]
    pub async fn detect_monorepo(&self, path: impl AsRef<Path>) -> StandardResult<MonorepoInfo> {
        let path = path.as_ref();

        // Find monorepo root
        let (root, kind) = if let Some((root, kind)) = self.find_monorepo_root(path)? {
            (root, kind)
        } else {
            return Err(StandardError::operation(format!(
                "No monorepo detected at or above {}",
                path.display()
            )));
        };

        // Get package locations
        let packages = match kind {
            MonorepoKind::Lerna => self.detect_lerna_packages(&root)?,
            MonorepoKind::YarnWorkspaces => self.detect_yarn_packages(&root)?,
            MonorepoKind::PnpmWorkspaces => self.detect_pnpm_packages(&root)?,
            MonorepoKind::Nx => self.detect_nx_packages(&root)?,
            MonorepoKind::Turborepo => self.detect_turborepo_packages(&root)?,
            MonorepoKind::Rush => self.detect_rush_packages(&root)?,
            MonorepoKind::Custom => self.detect_custom_packages(&root)?,
        };

        // Create monorepo info
        Ok(MonorepoInfo::new(kind, root, packages))
    }

    /// Detects if a directory has multiple package.json files
    ///
    /// # Arguments
    ///
    /// * `path` - The directory to check
    ///
    /// # Returns
    ///
    /// True if multiple package.json files are found
    fn has_multiple_packages(&self, path: &Path) -> bool {
        // Common package directory patterns
        let package_dirs = [
            path.join("packages"),
            path.join("apps"),
            path.join("libs"),
            path.join("components"),
            path.join("modules"),
        ];

        // Check if any common package directories exist
        for dir in &package_dirs {
            if self.fs.exists(dir) && dir.is_dir() {
                // Check if at least one subdirectory contains a package.json
                if let Ok(entries) = self.fs.read_dir(dir) {
                    for entry in entries {
                        let pkg_json = entry.join("package.json");
                        if self.fs.exists(&pkg_json) {
                            return true;
                        }
                    }
                }
            }
        }

        // Manual check for multiple package.json files in subdirectories
        let Ok(paths) = self.fs.walk_dir(path) else { return false };

        let mut package_json_count = 0;
        for path in paths {
            if path.file_name().map_or(false, |name| name == "package.json") {
                package_json_count += 1;
                if package_json_count > 1 {
                    return true;
                }
            }
        }

        false
    }

    /// Detects packages in a Lerna monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_lerna_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let lerna_path = root.join("lerna.json");
        let lerna_content = self.fs.read_file_string(&lerna_path)?;
        let lerna_config = serde_json::from_str::<LernaConfig>(&lerna_content)
            .map_err(|e| StandardError::operation(format!("Invalid lerna.json format: {e}")))?;

        // If useWorkspaces is true, delegate to yarn/npm workspaces detection
        if lerna_config.use_workspaces {
            return self.detect_yarn_packages(root);
        }

        // Default patterns if none specified
        let patterns = if lerna_config.packages.is_empty() {
            vec!["packages/*".to_string()]
        } else {
            lerna_config.packages
        };

        self.find_packages_from_patterns(root, &patterns)
    }

    /// Detects packages in a Yarn Workspaces monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_yarn_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let package_json_path = root.join("package.json");
        let package_json_content = self.fs.read_file_string(&package_json_path)?;
        let package_json = serde_json::from_str::<WorkspacesConfig>(&package_json_content)
            .map_err(|e| StandardError::operation(format!("Invalid package.json format: {e}")))?;

        let workspaces_config = package_json.workspaces.ok_or_else(|| {
            StandardError::operation("No workspaces field in package.json".to_string())
        })?;

        let patterns = match workspaces_config {
            WorkspacesPatterns::Array(patterns) => patterns,
            WorkspacesPatterns::Object { packages } => packages,
        };

        self.find_packages_from_patterns(root, &patterns)
    }

    /// Detects packages in a pnpm Workspaces monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_pnpm_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let pnpm_path = root.join("pnpm-workspace.yaml");
        let pnpm_content = self.fs.read_file_string(&pnpm_path)?;

        let pnpm_config: PnpmWorkspaceConfig = serde_yaml::from_str(&pnpm_content)
            .map_err(|e| StandardError::operation(format!("Invalid pnpm-workspace.yaml: {e}")))?;

        self.find_packages_from_patterns(root, &pnpm_config.packages)
    }

    /// Detects packages in an Nx monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_nx_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let nx_path = root.join("nx.json");
        let nx_content = self.fs.read_file_string(&nx_path)?;
        let nx_config = serde_json::from_str::<NxConfig>(&nx_content)
            .map_err(|e| StandardError::operation(format!("Invalid nx.json format: {e}")))?;

        let mut packages = Vec::new();

        // Two cases:
        // 1. Newer Nx has explicit project config with paths
        // 2. Older Nx requires checking workspace.json or angular.json
        if nx_config.projects.is_empty() {
            // Case 2: Try workspace.json or angular.json
            let workspace_files = ["workspace.json", "angular.json"];
            for file in &workspace_files {
                let workspace_path = root.join(file);
                if self.fs.exists(&workspace_path) {
                    let content = self.fs.read_file_string(&workspace_path)?;
                    if let Ok(workspace_config) = serde_json::from_str::<NxConfig>(&content) {
                        for (_name, project_config) in workspace_config.projects {
                            if let Some(project_path) = project_config.root {
                                let package_path = root.join(&project_path);
                                let package_json_path = package_path.join("package.json");

                                if self.fs.exists(&package_json_path) {
                                    if let Ok(package) =
                                        self.read_package_json(&package_json_path, root)
                                    {
                                        packages.push(package);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Case 1: Extract from nx.json projects
            for (_name, project_config) in nx_config.projects {
                if let Some(project_path) = project_config.root {
                    let package_path = root.join(&project_path);
                    let package_json_path = package_path.join("package.json");

                    if self.fs.exists(&package_json_path) {
                        if let Ok(package) = self.read_package_json(&package_json_path, root) {
                            packages.push(package);
                        }
                    }
                }
            }
        }

        // If still empty, fall back to common patterns
        if packages.is_empty() {
            let patterns =
                vec!["packages/*".to_string(), "apps/*".to_string(), "libs/*".to_string()];
            packages = self.find_packages_from_patterns(root, &patterns)?;
        }

        Ok(packages)
    }

    /// Detects packages in a Turborepo monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_turborepo_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        // Turborepo uses npm/yarn/pnpm workspaces under the hood
        // First try to detect from package.json
        if let Ok(packages) = self.detect_yarn_packages(root) {
            return Ok(packages);
        }

        // If no workspaces in package.json, try pnpm
        if self.fs.exists(&root.join("pnpm-workspace.yaml")) {
            return self.detect_pnpm_packages(root);
        }

        // Fall back to common patterns
        let patterns = vec!["packages/*".to_string(), "apps/*".to_string()];
        self.find_packages_from_patterns(root, &patterns)
    }

    /// Detects packages in a Rush monorepo
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_rush_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let rush_path = root.join("rush.json");
        let rush_content = self.fs.read_file_string(&rush_path)?;
        let rush_config = serde_json::from_str::<RushConfig>(&rush_content)
            .map_err(|e| StandardError::operation(format!("Invalid rush.json format: {e}")))?;

        let mut packages = Vec::new();

        for project in rush_config.projects {
            let package_path = root.join(&project.project_folder);
            let package_json_path = package_path.join("package.json");

            if self.fs.exists(&package_json_path) {
                if let Ok(package) = self.read_package_json(&package_json_path, root) {
                    packages.push(package);
                }
            }
        }

        Ok(packages)
    }

    /// Detects packages in a custom monorepo structure
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn detect_custom_packages(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        // Check for common monorepo directories
        let common_patterns =
            ["packages/*", "apps/*", "libs/*", "modules/*", "components/*", "services/*"];

        // Start with an empty set of patterns
        let mut patterns = Vec::new();

        // Add patterns for directories that exist
        for pattern in common_patterns {
            let base_dir = pattern.split('/').next().unwrap();
            if self.fs.exists(&root.join(base_dir)) {
                patterns.push(pattern.to_string());
            }
        }

        // If no common patterns are found, scan for package.json files
        if patterns.is_empty() {
            return self.find_packages_by_scanning(root);
        }

        self.find_packages_from_patterns(root, &patterns)
    }

    /// Finds packages using glob patterns
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    /// * `patterns` - Glob patterns for package locations
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn find_packages_from_patterns(
        &self,
        root: &Path,
        patterns: &[String],
    ) -> StandardResult<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();
        let mut package_paths = HashSet::new();

        for pattern in patterns {
            // Convert to absolute pattern
            let abs_pattern = root.join(pattern).to_string_lossy().into_owned();

            // Use glob to find matching paths
            for entry in glob(&abs_pattern).map_err(|e| {
                StandardError::operation(format!("Invalid glob pattern '{pattern}': {e}"))
            })? {
                match entry {
                    Ok(path) => {
                        if path.is_dir() {
                            let package_json_path = path.join("package.json");
                            if self.fs.exists(&package_json_path) && !package_paths.contains(&path)
                            {
                                package_paths.insert(path.clone());
                                if let Ok(package) =
                                    self.read_package_json(&package_json_path, root)
                                {
                                    packages.push(package);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Error processing path for pattern {}: {}", pattern, e);
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Finds packages by scanning the directory structure
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of workspace packages
    fn find_packages_by_scanning(&self, root: &Path) -> StandardResult<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();
        let root_package_json = root.join("package.json");

        // If there's a root package.json, we'll use it to exclude the root from being counted as a package
        let mut root_package_name = None;
        if self.fs.exists(&root_package_json) {
            if let Ok(content) = self.fs.read_file_string(&root_package_json) {
                if let Ok(json) = serde_json::from_str::<PackageJson>(&content) {
                    root_package_name = Some(json.name.clone());
                }
            }
        }

        // Walk the directory and find all package.json files
        let paths = self.fs.walk_dir(root)?;
        let mut package_paths = HashSet::new();

        for path in paths {
            if path.file_name().map_or(false, |name| name == "package.json") {
                let package_dir = path.parent().ok_or_else(|| {
                    StandardError::operation("Failed to get parent directory".to_string())
                })?;

                // Skip the root package.json and node_modules
                if package_dir == root || package_dir.to_string_lossy().contains("node_modules") {
                    continue;
                }

                if !package_paths.contains(package_dir) {
                    package_paths.insert(package_dir.to_path_buf());
                    if let Ok(package) = self.read_package_json(&path, root) {
                        // Skip the root package
                        if let Some(ref name) = root_package_name {
                            if &package.name == name {
                                continue;
                            }
                        }
                        packages.push(package);
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Reads a package.json file and creates a WorkspacePackage
    ///
    /// # Arguments
    ///
    /// * `package_json_path` - Path to the package.json file
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// A workspace package
    fn read_package_json(
        &self,
        package_json_path: &Path,
        root: &Path,
    ) -> StandardResult<WorkspacePackage> {
        let content = self.fs.read_file_string(package_json_path)?;
        let package_json = serde_json::from_str::<PackageJson>(&content).map_err(|e| {
            FileSystemError::Validation {
                path: package_json_path.to_path_buf(),
                reason: format!("Invalid package.json format: {e}"),
            }
        })?;

        let package_dir = package_json_path.parent().ok_or_else(|| {
            StandardError::operation("Failed to get package directory".to_string())
        })?;

        // Get the relative path from the root
        let location = if package_dir.is_absolute() && root.is_absolute() {
            package_dir
                .strip_prefix(root)
                .map_or_else(|_| package_dir.to_path_buf(), std::path::Path::to_path_buf)
        } else {
            // This is a best-effort approach if paths aren't absolute
            let package_path =
                fs::canonicalize(package_dir).unwrap_or_else(|_| package_dir.to_path_buf());
            let root_path = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

            package_path
                .strip_prefix(&root_path)
                .map_or_else(|_| package_dir.to_path_buf(), std::path::Path::to_path_buf)
        };

        // Extract workspace dependencies
        let mut workspace_dependencies = Vec::new();

        // Add direct dependencies
        for dep_name in package_json.dependencies.keys() {
            workspace_dependencies.push(dep_name.clone());
        }

        // Add dev dependencies
        for dep_name in package_json.dev_dependencies.keys() {
            workspace_dependencies.push(dep_name.clone());
        }

        Ok(WorkspacePackage {
            name: package_json.name,
            version: package_json.version,
            location,
            absolute_path: package_dir.to_path_buf(),
            workspace_dependencies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Test is_monorepo_root function
    #[tokio::test]
    async fn test_is_monorepo_root() -> StandardResult<()> {
        let detector = MonorepoDetector::new();

        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Not a monorepo yet
        assert!(detector.is_monorepo_root(root_path)?.is_none());

        // Create a lerna.json file
        std::fs::write(
            root_path.join("lerna.json"),
            r#"{ "packages": ["packages/*"], "version": "0.0.0" }"#,
        )?;

        // Now it should be detected as a Lerna monorepo
        assert_eq!(detector.is_monorepo_root(root_path)?, Some(MonorepoKind::Lerna));

        Ok(())
    }

    // Test find_monorepo_root function
    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_find_monorepo_root() -> StandardResult<()> {
        let detector = MonorepoDetector::new();

        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Create a subdirectory structure
        let packages_dir = root_path.join("packages");
        let package_dir = packages_dir.join("test-package");
        std::fs::create_dir_all(&package_dir)?;

        // Create a package.json file with workspaces
        std::fs::write(
            root_path.join("package.json"),
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )?;

        // Test finding from root
        let result = detector.find_monorepo_root(root_path)?;
        assert!(result.is_some());
        let (found_root, kind) = result.unwrap();
        assert_eq!(found_root, root_path);
        assert_eq!(kind, MonorepoKind::YarnWorkspaces);

        // Test finding from subdirectory
        let result = detector.find_monorepo_root(&package_dir)?;
        assert!(result.is_some());
        let (found_root, kind) = result.unwrap();
        assert_eq!(found_root, root_path);
        assert_eq!(kind, MonorepoKind::YarnWorkspaces);

        Ok(())
    }

    // Test full monorepo detection
    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_detect_monorepo() -> StandardResult<()> {
        let detector = MonorepoDetector::new();

        // Create a temporary directory for a Yarn Workspaces monorepo
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Create package structure
        let packages_dir = root_path.join("packages");
        let package_a_dir = packages_dir.join("package-a");
        let package_b_dir = packages_dir.join("package-b");
        std::fs::create_dir_all(&package_a_dir)?;
        std::fs::create_dir_all(&package_b_dir)?;

        // Create root package.json with workspaces
        std::fs::write(
            root_path.join("package.json"),
            r#"{
                "name": "monorepo-root",
                "private": true,
                "workspaces": ["packages/*"]
            }"#,
        )?;

        // Create package-a package.json
        std::fs::write(
            package_a_dir.join("package.json"),
            r#"{
                "name": "package-a",
                "version": "1.0.0",
                "dependencies": {
                    "package-b": "^1.0.0"
                }
            }"#,
        )?;

        // Create package-b package.json
        std::fs::write(
            package_b_dir.join("package.json"),
            r#"{
                "name": "package-b",
                "version": "1.0.0"
            }"#,
        )?;

        // Detect the monorepo
        let monorepo = detector.detect_monorepo(root_path).await?;

        // Verify monorepo information
        assert_eq!(monorepo.kind(), MonorepoKind::YarnWorkspaces);
        assert_eq!(monorepo.root(), root_path);
        assert_eq!(monorepo.packages().len(), 2);

        // Check that we can find a package by name
        let package_a = monorepo.get_package("package-a");
        assert!(package_a.is_some());
        assert_eq!(package_a.unwrap().name, "package-a");

        // Check that package-a depends on package-b
        let package_a = monorepo.get_package("package-a").unwrap();
        assert!(package_a.workspace_dependencies.contains(&"package-b".to_string()));

        // Find dependents of package-b
        let dependents = monorepo.find_dependents("package-b");
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].name, "package-a");

        Ok(())
    }

    // Test finding packages from patterns
    #[tokio::test]
    async fn test_find_packages_from_patterns() -> StandardResult<()> {
        let detector = MonorepoDetector::new();

        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Create package structure with multiple patterns
        let packages_dir = root_path.join("packages");
        let apps_dir = root_path.join("apps");
        let libs_dir = root_path.join("libs");

        std::fs::create_dir_all(&packages_dir.join("ui"))?;
        std::fs::create_dir_all(&apps_dir.join("web"))?;
        std::fs::create_dir_all(&libs_dir.join("utils"))?;

        // Create package.json files
        std::fs::write(
            packages_dir.join("ui").join("package.json"),
            r#"{ "name": "ui", "version": "1.0.0" }"#,
        )?;

        std::fs::write(
            apps_dir.join("web").join("package.json"),
            r#"{ "name": "web", "version": "1.0.0" }"#,
        )?;

        std::fs::write(
            libs_dir.join("utils").join("package.json"),
            r#"{ "name": "utils", "version": "1.0.0" }"#,
        )?;

        // Find packages using patterns
        let patterns = vec!["packages/*".to_string(), "apps/*".to_string(), "libs/*".to_string()];

        let packages = detector.find_packages_from_patterns(root_path, &patterns)?;

        // Verify packages
        assert_eq!(packages.len(), 3);

        // Check package names were detected correctly
        let package_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
        assert!(package_names.contains(&"ui".to_string()));
        assert!(package_names.contains(&"web".to_string()));
        assert!(package_names.contains(&"utils".to_string()));

        Ok(())
    }

    // Test custom monorepo detection
    #[tokio::test]
    async fn test_custom_monorepo_detection() -> StandardResult<()> {
        let detector = MonorepoDetector::new();

        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Create an unconventional structure with package.json files
        let module_a_dir = root_path.join("module-a");
        let module_b_dir = root_path.join("module-b");
        std::fs::create_dir_all(&module_a_dir)?;
        std::fs::create_dir_all(&module_b_dir)?;

        // Create root package.json (no workspaces)
        std::fs::write(
            root_path.join("package.json"),
            r#"{ "name": "custom-monorepo", "private": true }"#,
        )?;

        // Create module package.json files
        std::fs::write(
            module_a_dir.join("package.json"),
            r#"{ "name": "module-a", "version": "1.0.0" }"#,
        )?;

        std::fs::write(
            module_b_dir.join("package.json"),
            r#"{ "name": "module-b", "version": "1.0.0" }"#,
        )?;

        // Detect as custom monorepo
        assert_eq!(detector.is_monorepo_root(root_path)?, Some(MonorepoKind::Custom));

        // Test full detection
        let monorepo = detector.detect_monorepo(root_path).await?;
        assert_eq!(monorepo.kind(), MonorepoKind::Custom);
        assert_eq!(monorepo.packages().len(), 2);

        Ok(())
    }
}
