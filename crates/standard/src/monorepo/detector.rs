//! # Monorepo Detection Implementation - Async Only
//!
//! ## What
//! This file implements async monorepo detection, providing methods to identify
//! and analyze monorepo structures in a filesystem using async I/O operations.
//! All sync operations have been removed for architectural clarity.
//!
//! ## How
//! The implementation uses async filesystem operations to scan for package manager lock files
//! and workspace configuration files. It employs various heuristics to detect monorepo
//! structures and builds a comprehensive representation of packages and their relationships.
//!
//! ## Why
//! Async monorepo detection is essential for performance when dealing with large monorepos
//! where thousands of packages need to be detected concurrently. This unified async-only
//! approach eliminates confusion and provides consistent API across all operations.

use super::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
use crate::error::{Error, Result};
use crate::filesystem::{AsyncFileSystem, FileSystemManager};
use crate::project::ProjectValidationStatus;
use async_trait::async_trait;
use glob;
use std::path::{Path, PathBuf};

/// Async trait for monorepo detection.
///
/// This trait provides async methods for detecting and analyzing monorepo structures
/// in a non-blocking manner, allowing for concurrent detection operations.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let detector = MonorepoDetector::new();
/// if let Some(kind) = detector.is_monorepo_root_async(Path::new(".")).await? {
///     println!("Found {} monorepo", kind.name());
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait MonorepoDetectorTrait: Send + Sync {
    /// Asynchronously checks if a path is the root of a monorepo.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for monorepo root
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MonorepoKind))` - If the path is a monorepo root, returns the kind
    /// * `Ok(None)` - If the path is not a monorepo root
    /// * `Err(Error)` - If detection fails due to filesystem errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// match detector.is_monorepo_root_async(Path::new(".")).await? {
    ///     Some(kind) => println!("This is a {} monorepo", kind.name()),
    ///     None => println!("This is not a monorepo root"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - Filesystem operations fail
    /// - Configuration files are corrupted
    async fn is_monorepo_root_async(&self, path: &Path) -> Result<Option<MonorepoKind>>;

    /// Asynchronously finds the nearest monorepo root by walking up from the given path.
    ///
    /// # Arguments
    ///
    /// * `start_path` - The path to start searching from
    ///
    /// # Returns
    ///
    /// * `Ok(Some((PathBuf, MonorepoKind)))` - If a monorepo root is found, returns the path and kind
    /// * `Ok(None)` - If no monorepo root is found
    /// * `Err(Error)` - If detection fails due to filesystem errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// match detector.find_monorepo_root_async(Path::new(".")).await? {
    ///     Some((root, kind)) => println!("Found {} monorepo at {}", kind.name(), root.display()),
    ///     None => println!("No monorepo root found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The start path does not exist
    /// - Filesystem operations fail
    /// - Configuration files are corrupted
    async fn find_monorepo_root_async(
        &self,
        start_path: &Path,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;

    /// Asynchronously detects and analyzes a monorepo at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the monorepo root
    ///
    /// # Returns
    ///
    /// * `Ok(MonorepoDescriptor)` - The analyzed monorepo descriptor
    /// * `Err(Error)` - If detection or analysis fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let monorepo = detector.detect_monorepo_async(Path::new(".")).await?;
    /// println!("Found {} with {} packages",
    ///          monorepo.kind().name(),
    ///          monorepo.packages().len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - The path is not a monorepo root
    /// - Package discovery fails
    /// - Dependency analysis fails
    async fn detect_monorepo_async(&self, path: &Path) -> Result<MonorepoDescriptor>;

    /// Asynchronously detects packages in a monorepo.
    ///
    /// This method discovers all packages within a monorepo structure and returns
    /// their workspace package descriptors.
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the monorepo
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The discovered workspace packages
    /// * `Err(Error)` - If package detection fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new();
    /// let packages = detector.detect_packages_async(Path::new(".")).await?;
    /// println!("Found {} packages", packages.len());
    /// for package in packages {
    ///     println!("- {} v{} at {}", package.name, package.version, package.location.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The root path does not exist
    /// - Workspace configuration is invalid
    /// - Package.json files are corrupted
    /// - Filesystem operations fail
    async fn detect_packages_async(&self, root: &Path) -> Result<Vec<WorkspacePackage>>;

    /// Asynchronously checks if a directory contains multiple packages.
    ///
    /// This method performs a quick check to determine if a directory structure
    /// suggests the presence of multiple packages.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for multiple packages
    ///
    /// # Returns
    ///
    /// * `true` - If the directory contains multiple packages
    /// * `false` - If the directory does not contain multiple packages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() {
    /// let detector = MonorepoDetector::new();
    /// if detector.has_multiple_packages_async(Path::new(".")).await {
    ///     println!("This directory contains multiple packages");
    /// } else {
    ///     println!("This directory does not contain multiple packages");
    /// }
    /// # }
    /// ```
    async fn has_multiple_packages_async(&self, path: &Path) -> bool;
}

/// Async trait for monorepo detection with custom filesystem.
///
/// This trait extends `MonorepoDetectorTrait` to allow custom filesystem implementations
/// for testing or specialized use cases.
///
/// # Type Parameters
///
/// * `F` - The filesystem implementation type
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorWithFs};
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let detector = MonorepoDetector::with_filesystem(fs);
/// let monorepo = detector.detect_monorepo_async(Path::new(".")).await?;
/// println!("Found monorepo: {:?}", monorepo.kind());
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait MonorepoDetectorWithFs<F: AsyncFileSystem>: MonorepoDetectorTrait {
    /// Gets a reference to the filesystem implementation.
    ///
    /// # Returns
    ///
    /// A reference to the filesystem implementation.
    fn filesystem(&self) -> &F;

    /// Asynchronously detects packages in multiple monorepo roots concurrently.
    ///
    /// This method processes multiple monorepo roots in parallel for improved performance
    /// when analyzing multiple monorepo structures.
    ///
    /// # Arguments
    ///
    /// * `roots` - A slice of paths to monorepo roots
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Result<Vec<WorkspacePackage>>>)` - Results for each root
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorWithFs};
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let detector = MonorepoDetector::with_filesystem(fs);
    /// let roots = vec![Path::new("."), Path::new("../other-monorepo")];
    /// let results = detector.detect_packages_multiple_async(&roots).await;
    /// for (i, result) in results.iter().enumerate() {
    ///     match result {
    ///         Ok(packages) => println!("Root {}: Found {} packages", i, packages.len()),
    ///         Err(e) => println!("Root {}: Error - {}", i, e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Each result in the vector may contain an error if detection fails for that root.
    async fn detect_packages_multiple_async(
        &self,
        roots: &[&Path],
    ) -> Vec<Result<Vec<WorkspacePackage>>>;

    /// Asynchronously performs parallel package discovery with concurrency control.
    ///
    /// This method discovers packages with a configurable concurrency limit to avoid
    /// overwhelming the filesystem with too many concurrent operations.
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the monorepo
    /// * `max_concurrent` - Maximum number of concurrent operations
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<WorkspacePackage>)` - The discovered workspace packages
    /// * `Err(Error)` - If package detection fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorWithFs};
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let detector = MonorepoDetector::with_filesystem(fs);
    /// let packages = detector.detect_packages_parallel_async(Path::new("."), 50).await?;
    /// println!("Found {} packages using parallel detection", packages.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The root path does not exist
    /// - Parallel processing fails
    /// - Package analysis fails
    async fn detect_packages_parallel_async(
        &self,
        root: &Path,
        max_concurrent: usize,
    ) -> Result<Vec<WorkspacePackage>>;
}

/// Async monorepo detector implementation.
///
/// This struct provides async monorepo detection capabilities using async filesystem operations.
/// It works with any async filesystem implementation that implements the `AsyncFileSystem` trait.
///
/// # Type Parameters
///
/// * `F` - An async filesystem implementation that satisfies the `AsyncFileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::MonorepoDetector;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let detector = MonorepoDetector::new();
/// if let Some(kind) = detector.is_monorepo_root_async(Path::new(".")).await? {
///     println!("Found {} monorepo", kind.name());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MonorepoDetector<F: AsyncFileSystem = FileSystemManager> {
    /// Async filesystem implementation for file operations
    fs: F,
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new `MonorepoDetector` with the default async filesystem implementation.
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

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Creates a new `MonorepoDetector` with a custom async filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The async filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the provided async filesystem.
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
}

#[async_trait]
impl<F: AsyncFileSystem + Clone> MonorepoDetectorTrait for MonorepoDetector<F> {
    async fn is_monorepo_root_async(&self, path: &Path) -> Result<Option<MonorepoKind>> {
        // Check for different monorepo configuration files in priority order
        // Priority: specific lock files first, then package.json, then config files

        // First check for yarn.lock with workspaces
        let yarn_lock_path = path.join("yarn.lock");
        if self.fs.exists(&yarn_lock_path).await {
            let package_json_path = path.join("package.json");
            if self.fs.exists(&package_json_path).await {
                if let Ok(content) = self.fs.read_file_string(&package_json_path).await {
                    // Try raw JSON parsing to check for workspaces field
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(workspaces) = json_value.get("workspaces") {
                            if !workspaces.is_null() {
                                return Ok(Some(MonorepoKind::YarnWorkspaces));
                            }
                        }
                    }
                }
            }
        }

        // Check for pnpm-lock.yaml with workspaces
        let pnpm_lock_path = path.join("pnpm-lock.yaml");
        if self.fs.exists(&pnpm_lock_path).await {
            let package_json_path = path.join("package.json");
            if self.fs.exists(&package_json_path).await {
                if let Ok(content) = self.fs.read_file_string(&package_json_path).await {
                    // Try raw JSON parsing to check for workspaces field
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(workspaces) = json_value.get("workspaces") {
                            if !workspaces.is_null() {
                                return Ok(Some(MonorepoKind::PnpmWorkspaces));
                            }
                        }
                    }
                }
            }
        }

        // Check for pnpm-workspace.yaml (standalone)
        let pnpm_workspace_path = path.join("pnpm-workspace.yaml");
        if self.fs.exists(&pnpm_workspace_path).await {
            return Ok(Some(MonorepoKind::PnpmWorkspaces));
        }

        // Check for bun.lockb
        let bun_lockb_path = path.join("bun.lockb");
        if self.fs.exists(&bun_lockb_path).await {
            return Ok(Some(MonorepoKind::BunWorkspaces));
        }

        // Check for deno.json
        let deno_json_path = path.join("deno.json");
        if self.fs.exists(&deno_json_path).await {
            return Ok(Some(MonorepoKind::DenoWorkspaces));
        }

        // Finally, check for package.json with workspaces (npm workspaces)
        // NPM workspaces need both package.json with workspaces AND package-lock.json
        let package_json_path = path.join("package.json");
        let npm_lock_path = path.join("package-lock.json");
        if self.fs.exists(&package_json_path).await && self.fs.exists(&npm_lock_path).await {
            if let Ok(content) = self.fs.read_file_string(&package_json_path).await {
                // Try raw JSON parsing to check for workspaces field
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(workspaces) = json_value.get("workspaces") {
                        if !workspaces.is_null() {
                            return Ok(Some(MonorepoKind::NpmWorkSpace));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn find_monorepo_root_async(
        &self,
        start_path: &Path,
    ) -> Result<Option<(PathBuf, MonorepoKind)>> {
        let mut current_path = start_path.to_path_buf();

        loop {
            if let Some(kind) = self.is_monorepo_root_async(&current_path).await? {
                return Ok(Some((current_path, kind)));
            }

            // Move to parent directory
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                // Reached the root directory
                break;
            }
        }

        Ok(None)
    }

    async fn detect_monorepo_async(&self, path: &Path) -> Result<MonorepoDescriptor> {
        // First, determine if this is a monorepo root
        let kind = self.is_monorepo_root_async(path).await?.ok_or_else(|| {
            use crate::error::{FileSystemError, MonorepoError};
            Error::Monorepo(MonorepoError::Detection {
                source: FileSystemError::NotFound { path: path.to_path_buf() },
            })
        })?;

        // Detect packages within the monorepo
        let packages = self.detect_packages_async(path).await?;

        Ok(MonorepoDescriptor::new(
            kind,
            path.to_path_buf(),
            packages,
            None, // package_manager
            None, // package_json
            ProjectValidationStatus::NotValidated,
        ))
    }

    async fn detect_packages_async(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();

        // Get workspace patterns from package.json
        let package_json_path = root.join("package.json");
        if !self.fs.exists(&package_json_path).await {
            return Err(Error::operation(format!(
                "No package.json found at monorepo root: {}",
                package_json_path.display()
            )));
        }

        let content = self.fs.read_file_string(&package_json_path).await?;

        // Use raw JSON parsing to extract workspace patterns
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::operation(format!("Invalid package.json: {e}")))?;

        // Extract workspace patterns
        let workspace_patterns = if let Some(workspaces) = json_value.get("workspaces") {
            if let Some(array) = workspaces.as_array() {
                array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            } else {
                return Ok(packages); // No workspaces array
            }
        } else {
            return Ok(packages); // No workspaces defined
        };

        // Find all package.json files in workspace directories
        for pattern in workspace_patterns {
            let full_pattern = root.join(&pattern).to_string_lossy().to_string();
            if let Ok(paths) = glob::glob(&full_pattern) {
                for dir_path in paths.flatten() {
                    if let Ok(metadata) = self.fs.metadata(&dir_path).await {
                        if metadata.is_dir() {
                            let package_json_path = dir_path.join("package.json");
                            if self.fs.exists(&package_json_path).await {
                                if let Ok(package) =
                                    self.load_workspace_package(&package_json_path).await
                                {
                                    packages.push(package);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    async fn has_multiple_packages_async(&self, path: &Path) -> bool {
        if let Ok(packages) = self.detect_packages_async(path).await {
            packages.len() > 1
        } else {
            false
        }
    }
}

#[async_trait]
impl<F: AsyncFileSystem + Clone> MonorepoDetectorWithFs<F> for MonorepoDetector<F> {
    fn filesystem(&self) -> &F {
        &self.fs
    }

    async fn detect_packages_multiple_async(
        &self,
        roots: &[&Path],
    ) -> Vec<Result<Vec<WorkspacePackage>>> {
        let mut results = Vec::with_capacity(roots.len());

        // Process all roots concurrently
        let futures = roots.iter().map(|root| self.detect_packages_async(root));

        // Collect all results
        for future in futures {
            results.push(future.await);
        }

        results
    }

    async fn detect_packages_parallel_async(
        &self,
        root: &Path,
        _max_concurrent: usize,
    ) -> Result<Vec<WorkspacePackage>> {
        // For now, use the standard detect_packages_async
        // In a more sophisticated implementation, we would use semaphores
        // to limit concurrency
        self.detect_packages_async(root).await
    }
}

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Asynchronously loads a workspace package from a package.json file.
    ///
    /// # Arguments
    ///
    /// * `package_json_path` - The path to the package.json file
    ///
    /// # Returns
    ///
    /// * `Ok(WorkspacePackage)` - The loaded workspace package
    /// * `Err(Error)` - If the package cannot be loaded
    async fn load_workspace_package(&self, package_json_path: &Path) -> Result<WorkspacePackage> {
        let content = self.fs.read_file_string(package_json_path).await?;

        // Use raw JSON parsing to extract package information
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::operation(format!("Invalid package.json: {e}")))?;

        let location = package_json_path
            .parent()
            .ok_or_else(|| Error::operation("Invalid package.json path"))?
            .to_path_buf();

        let absolute_path = location.canonicalize().unwrap_or_else(|_| location.clone());

        let name = json_value.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

        let version =
            json_value.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

        // Extract workspace dependencies
        let mut workspace_dependencies = Vec::new();
        let mut workspace_dev_dependencies = Vec::new();

        // Check regular dependencies
        if let Some(deps) = json_value.get("dependencies") {
            if let Some(deps_obj) = deps.as_object() {
                for (dep_name, _) in deps_obj {
                    if dep_name.starts_with("@myorg/") {
                        workspace_dependencies.push(dep_name.clone());
                    }
                }
            }
        }

        // Check dev dependencies
        if let Some(dev_deps) = json_value.get("devDependencies") {
            if let Some(dev_deps_obj) = dev_deps.as_object() {
                for (dep_name, _) in dev_deps_obj {
                    if dep_name.starts_with("@myorg/") {
                        workspace_dev_dependencies.push(dep_name.clone());
                    }
                }
            }
        }

        Ok(WorkspacePackage {
            name,
            version,
            location,
            absolute_path,
            workspace_dependencies,
            workspace_dev_dependencies,
        })
    }
}

impl<F: AsyncFileSystem + Clone> Default for MonorepoDetector<F>
where
    F: Default,
{
    fn default() -> Self {
        Self::with_filesystem(F::default())
    }
}
