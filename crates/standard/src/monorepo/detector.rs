//! # Monorepo Detection Implementation
//!
//! ## What
//! This file implements the `MonorepoDetector` struct, providing methods to identify
//! and analyze monorepo structures in a filesystem. It supports detection of various
//! monorepo types including npm, yarn, pnpm, bun, and others.
//!
//! ## How
//! The implementation uses a filesystem abstraction to scan for package manager lock files
//! and workspace configuration files. It employs various heuristics to detect monorepo
//! structures and builds a comprehensive representation of packages and their relationships.
//!
//! ## Why
//! Accurate detection of monorepo structures is essential for proper project analysis,
//! dependency management, and command execution. This implementation provides a consistent
//! approach to working with different monorepo types through a unified interface.

use package_json::{PackageJson, PackageJsonManager};

use super::{
    MonorepoDescriptor, MonorepoDetector, MonorepoKind, PackageManagerKind, PnpmWorkspaceConfig,
    WorkspacePackage,
};
use crate::error::{Error, FileSystemError, MonorepoError, Result, WorkspaceError};
use crate::filesystem::{FileSystem, FileSystemManager};
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new `MonorepoDetector` with the default filesystem implementation.
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the `FileSystemManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// let detector = MonorepoDetector::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { fs: FileSystemManager::new() }
    }
}

impl<F: FileSystem> MonorepoDetector<F> {
    /// Creates a new `MonorepoDetector` with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the provided filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// let fs = FileSystemManager::new();
    /// let detector = MonorepoDetector::with_filesystem(fs);
    /// ```
    #[must_use]
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    /// Checks if a path is the root of a monorepo by examining lock files.
    ///
    /// This method looks for lock files specific to different package managers
    /// to determine if the directory is a monorepo root and which type.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - An I/O error occurs while accessing the filesystem
    /// - The path cannot be read or accessed
    /// - Configuration files cannot be parsed
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MonorepoKind))` - If the path is a monorepo root, with the detected type
    /// * `Ok(None)` - If the path is not a monorepo root
    /// * `Err(Error)` - If an error occurred during detection
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// if let Some(kind) = detector.is_monorepo_root(".")? {
    ///     println!("Found monorepo of type: {}", kind.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>> {
        let path = path.as_ref();

        let npm_lock_path = path.join(PackageManagerKind::Npm.lock_file());
        if self.fs.exists(&npm_lock_path) {
            return Ok(Some(MonorepoKind::NpmWorkSpace));
        }

        let bun_lock_file = path.join(PackageManagerKind::Bun.lock_file());
        if self.fs.exists(&bun_lock_file) {
            return Ok(Some(MonorepoKind::BunWorkspaces));
        }

        let jsr_lock_file = path.join(PackageManagerKind::Jsr.lock_file());
        if self.fs.exists(&jsr_lock_file) {
            return Ok(Some(MonorepoKind::DenoWorkspaces));
        }

        let pnpm_lock_file = path.join(PackageManagerKind::Pnpm.lock_file());
        if self.fs.exists(&pnpm_lock_file) {
            return Ok(Some(MonorepoKind::PnpmWorkspaces));
        }

        let yarn_lock_file = path.join(PackageManagerKind::Yarn.lock_file());
        if self.fs.exists(&yarn_lock_file) {
            return Ok(Some(MonorepoKind::YarnWorkspaces));
        }

        Ok(None)
    }

    /// Finds the nearest monorepo root by walking up from the given path.
    ///
    /// This method starts at the provided path and checks each parent directory
    /// until it finds a monorepo root or reaches the filesystem root.
    ///
    /// # Arguments
    ///
    /// * `start_path` - The path to start searching from
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - An I/O error occurs while accessing the filesystem
    /// - Any path in the traversal cannot be read or accessed
    /// - Configuration files cannot be parsed during detection
    ///
    /// # Returns
    ///
    /// * `Ok(Some((PathBuf, MonorepoKind)))` - The path and kind of monorepo if found
    /// * `Ok(None)` - If no monorepo root was found
    /// * `Err(Error)` - If an error occurred during detection
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// if let Some((root, kind)) = detector.find_monorepo_root("src/components")? {
    ///     println!("Found monorepo root at: {}", root.display());
    ///     println!("Type: {}", kind.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>> {
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

    /// Detects and analyzes a monorepo at the given path.
    ///
    /// This method identifies the monorepo type, locates all packages,
    /// and creates a comprehensive description of the monorepo structure.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for a monorepo structure
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The path is not a monorepo root
    /// - An I/O error occurs while scanning the filesystem
    /// - Package configuration files cannot be read or parsed
    /// - Workspace configuration files are invalid
    /// - Glob patterns in workspace configurations are malformed
    ///
    /// # Returns
    ///
    /// * `Ok(MonorepoDescriptor)` - A descriptor containing all monorepo information
    /// * `Err(Error)` - If the path is not a monorepo or an error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let monorepo = detector.detect_monorepo(".")?;
    ///
    /// println!("Detected {} monorepo with {} packages",
    ///          monorepo.kind().name(),
    ///          monorepo.packages().len());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::manual_let_else)]
    pub fn detect_monorepo(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor> {
        let path = path.as_ref();

        // Find monorepo root
        let (root, kind) = if let Some((root, kind)) = self.find_monorepo_root(path)? {
            (root, kind)
        } else {
            return Err(Error::Monorepo(MonorepoError::Detection {
                source: FileSystemError::NotFound { path: path.to_path_buf() },
            }));
        };

        // Get package locations
        let packages = match &kind {
            MonorepoKind::DenoWorkspaces => self.detect_npm_packages(&root)?,
            MonorepoKind::YarnWorkspaces => self.detect_npm_packages(&root)?,
            MonorepoKind::PnpmWorkspaces => self.detect_pnpm_packages(&root)?,
            MonorepoKind::BunWorkspaces => self.detect_npm_packages(&root)?,
            MonorepoKind::NpmWorkSpace => self.detect_npm_packages(&root)?,
            MonorepoKind::Custom { name: _, config_file: _ } => {
                self.detect_custom_packages(&root)?
            }
        };

        // Create monorepo info
        Ok(MonorepoDescriptor::new(kind, root, packages))
    }

    /// Checks if a directory contains multiple packages based on common patterns.
    ///
    /// This is a heuristic method that examines common monorepo directory structures
    /// to determine if a directory likely contains multiple packages.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for multiple packages
    ///
    /// # Returns
    ///
    /// `true` if the path likely contains multiple packages, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    ///
    /// let detector = MonorepoDetector::new();
    /// let has_packages = detector.has_multiple_packages(Path::new("."));
    /// println!("Contains multiple packages: {}", has_packages);
    /// ```
    pub fn has_multiple_packages(&self, path: &Path) -> bool {
        // Common package directory patterns
        let package_dirs = [
            path.join("packages"),
            path.join("apps"),
            path.join("libs"),
            path.join("components"),
            path.join("modules"),
            path.join("web"),
            path.join("ui"),
            path.join("pkgs"),
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

    /// Detects packages in an npm/yarn monorepo by parsing package.json workspaces.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The list of packages found in the monorepo
    /// * `Err(Error)` - If an error occurred during package detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let packages = detector.detect_npm_packages(Path::new("/project/root"))?;
    /// println!("Found {} packages", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    fn detect_npm_packages(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
        let package_json_path = root.join("package.json");
        let mut manager = PackageJsonManager::with_file_path(&package_json_path);
        let package_json = manager
            .read_ref()
            .map_err(|e| Error::Workspace(WorkspaceError::InvalidPackageJson(e.to_string())))?;

        let workspaces_config = package_json.workspaces.as_ref().ok_or_else(|| {
            Error::Workspace(WorkspaceError::WorkspaceConfigMissing(
                "No workspaces field in package.json".to_string(),
            ))
        })?;

        self.find_packages_from_patterns(root, workspaces_config)
    }

    /// Detects packages in a pnpm monorepo by parsing pnpm-workspace.yaml.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The list of packages found in the monorepo
    /// * `Err(Error)` - If an error occurred during package detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let packages = detector.detect_pnpm_packages(Path::new("/project/root"))?;
    /// println!("Found {} packages", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    fn detect_pnpm_packages(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
        let pnpm_path = root.join("pnpm-workspace.yaml");
        let pnpm_content = self.fs.read_file_string(&pnpm_path)?;

        let pnpm_config: PnpmWorkspaceConfig = serde_yaml::from_str(&pnpm_content)
            .map_err(|e| Error::Workspace(WorkspaceError::InvalidPnpmWorkspace(e.to_string())))?;

        self.find_packages_from_patterns(root, &pnpm_config.packages)
    }

    /// Detects packages in an unrecognized monorepo by using common patterns.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The list of packages found in the monorepo
    /// * `Err(Error)` - If an error occurred during package detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let packages = detector.detect_custom_packages(Path::new("/project/root"))?;
    /// println!("Found {} packages", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    fn detect_custom_packages(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
        // Check for common monorepo directories
        let common_patterns =
            ["packages/*", "apps/*", "libs/*", "modules/*", "components/*", "services/*"];

        // Start with an empty set of patterns
        let mut patterns = Vec::new();

        // Add patterns for directories that exist
        for pattern in common_patterns {
            if let Some(base_dir) = pattern.split('/').next() {
                if self.fs.exists(&root.join(base_dir)) {
                    patterns.push(pattern.to_string());
                }
            }
        }

        // If no common patterns are found, scan for package.json files
        if patterns.is_empty() {
            return self.find_packages_by_scanning(root);
        }

        self.find_packages_from_patterns(root, &patterns)
    }

    /// Finds packages in a monorepo by matching glob patterns.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    /// * `patterns` - Glob patterns that match package directories
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The list of packages found in the monorepo
    /// * `Err(Error)` - If an error occurred during package detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let patterns = vec!["packages/*".to_string(), "apps/*".to_string()];
    /// let packages = detector.find_packages_from_patterns(Path::new("/project/root"), &patterns)?;
    /// println!("Found {} packages", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn find_packages_from_patterns(
        &self,
        root: &Path,
        patterns: &[String],
    ) -> Result<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();
        let mut package_paths = HashSet::new();

        for pattern in patterns {
            // Convert to absolute pattern
            let abs_pattern = root.join(pattern).to_string_lossy().into_owned();

            // Use glob to find matching paths
            for entry in glob(&abs_pattern).map_err(|e| {
                Error::Workspace(WorkspaceError::InvalidWorkspacesPattern(e.to_string()))
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

    /// Finds packages by scanning the directory hierarchy for package.json files.
    ///
    /// This is used when no explicit workspace patterns are available.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The list of packages found in the monorepo
    /// * `Err(Error)` - If an error occurred during package detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let packages = detector.find_packages_by_scanning(Path::new("/project/root"))?;
    /// println!("Found {} packages by scanning", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::implicit_clone)]
    pub(crate) fn find_packages_by_scanning(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
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
                // Create an owned copy of the path for error reporting
                let path_buf = path.to_path_buf();

                // Get parent directory as an owned PathBuf to avoid lifetime issues
                let package_dir = match path.parent() {
                    Some(parent) => parent.to_path_buf(),
                    None => {
                        return Err(Error::FileSystem(FileSystemError::NotFound { path: path_buf }))
                    }
                };

                // Skip the root package.json and node_modules
                if package_dir == root || package_dir.to_string_lossy().contains("node_modules") {
                    continue;
                }

                if !package_paths.contains(&package_dir) {
                    package_paths.insert(package_dir);
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

    /// Reads and parses a package.json file to create a `WorkspacePackage`.
    ///
    /// # Arguments
    ///
    /// * `package_json_path` - The path to the package.json file
    /// * `root` - The root directory of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(WorkspacePackage)` - The parsed package information
    /// * `Err(Error)` - If the package.json is invalid or can't be read
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::{Path, PathBuf};
    /// # use sublime_standard_tools::monorepo::MonorepoDetector;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let root = PathBuf::from("/project/root");
    /// let pkg_json_path = root.join("packages/ui/package.json");
    ///
    /// if let Ok(package) = detector.read_package_json(&pkg_json_path, &root) {
    ///     println!("Package: {} v{}", package.name, package.version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn read_package_json(
        &self,
        package_json_path: &Path,
        root: &Path,
    ) -> Result<WorkspacePackage> {
        let content = self.fs.read_file_string(package_json_path)?;
        let package_json = serde_json::from_str::<PackageJson>(&content).map_err(|e| {
            FileSystemError::Validation {
                path: package_json_path.to_path_buf(),
                reason: format!("Invalid package.json format: {e}"),
            }
        })?;

        let package_dir = package_json_path.parent().ok_or_else(|| {
            Error::FileSystem(FileSystemError::NotFound { path: package_json_path.to_path_buf() })
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
        let mut workspace_dev_dependencies = Vec::new();

        // Add direct dependencies
        if let Some(dependencies) = &package_json.dependencies {
            for dep_name in dependencies.keys() {
                workspace_dependencies.push(dep_name.clone());
            }
        }

        // Add dev dependencies
        if let Some(dev_dependencies) = &package_json.dev_dependencies {
            for dep_name in dev_dependencies.keys() {
                workspace_dev_dependencies.push(dep_name.clone());
            }
        }

        Ok(WorkspacePackage {
            name: package_json.name,
            version: package_json.version,
            location,
            absolute_path: package_dir.to_path_buf(),
            workspace_dependencies,
            workspace_dev_dependencies,
        })
    }
}
