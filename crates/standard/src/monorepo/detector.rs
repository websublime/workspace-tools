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
use crate::config::{traits::Configurable, ConfigManager, StandardConfig};
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
/// if let Some(kind) = detector.is_monorepo_root(Path::new(".")).await? {
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
    /// match detector.is_monorepo_root(Path::new(".")).await? {
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
    async fn is_monorepo_root(&self, path: &Path) -> Result<Option<MonorepoKind>>;

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
    /// match detector.find_monorepo_root(Path::new(".")).await? {
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
    async fn find_monorepo_root(
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
    /// let monorepo = detector.detect_monorepo(Path::new(".")).await?;
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
    async fn detect_monorepo(&self, path: &Path) -> Result<MonorepoDescriptor>;

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
    /// let packages = detector.detect_packages(Path::new(".")).await?;
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
    async fn detect_packages(&self, root: &Path) -> Result<Vec<WorkspacePackage>>;

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
    /// if detector.has_multiple_packages(Path::new(".")).await {
    ///     println!("This directory contains multiple packages");
    /// } else {
    ///     println!("This directory does not contain multiple packages");
    /// }
    /// # }
    /// ```
    async fn has_multiple_packages(&self, path: &Path) -> bool;
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
/// let monorepo = detector.detect_monorepo(Path::new(".")).await?;
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
    /// let results = detector.detect_packages_multiple(&roots).await;
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
    async fn detect_packages_multiple(&self, roots: &[&Path])
        -> Vec<Result<Vec<WorkspacePackage>>>;

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
    /// let packages = detector.detect_packages_parallel(Path::new("."), 50).await?;
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
    async fn detect_packages_parallel(
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
/// if let Some(kind) = detector.is_monorepo_root(Path::new(".")).await? {
///     println!("Found {} monorepo", kind.name());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MonorepoDetector<F: AsyncFileSystem = FileSystemManager> {
    /// Async filesystem implementation for file operations
    fs: F,
    /// Configuration for monorepo detection and analysis
    config: crate::config::MonorepoConfig,
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new `MonorepoDetector` with the default async filesystem implementation
    /// and default monorepo configuration.
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the `FileSystemManager` and default configuration.
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
        Self { fs: FileSystemManager::new(), config: crate::config::MonorepoConfig::default() }
    }

    /// Creates a new `MonorepoDetector` with the default filesystem and custom monorepo configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The monorepo configuration to use
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the provided configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    /// use sublime_standard_tools::config::MonorepoConfig;
    ///
    /// let config = MonorepoConfig {
    ///     max_search_depth: 10,
    ///     ..MonorepoConfig::default()
    /// };
    /// let detector = MonorepoDetector::new_with_config(config);
    /// ```
    #[must_use]
    pub fn new_with_config(config: crate::config::MonorepoConfig) -> Self {
        Self { fs: FileSystemManager::new(), config }
    }

    /// Creates a new `MonorepoDetector` that automatically loads configuration from project files.
    ///
    /// This method searches for configuration files (repo.config.*) in the specified path and
    /// loads the monorepo configuration from them. If no config files are found, it uses
    /// default configuration with environment variable overrides.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The path to search for configuration files
    ///
    /// # Returns
    ///
    /// * `Ok(MonorepoDetector)` - A detector with loaded configuration
    /// * `Err(Error)` - If configuration loading fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = MonorepoDetector::new_with_project_config(Path::new(".")).await?;
    /// // Configuration loaded from repo.config.toml/yml/json or defaults + env vars
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files exist but cannot be parsed.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self> {
        let fs = FileSystemManager::new();
        let config = Self::load_project_config(&fs, project_root, None).await?;

        Ok(Self { fs, config: config.monorepo })
    }

    /// Loads configuration from project files in the specified directory.
    ///
    /// This method searches for configuration files in the following order:
    /// - repo.config.toml
    /// - repo.config.yml/yaml
    /// - repo.config.json
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    /// * `project_root` - The directory to search for configuration files
    /// * `base_config` - Optional base configuration to merge with
    ///
    /// # Returns
    ///
    /// * `Ok(StandardConfig)` - The loaded and merged configuration
    /// * `Err(Error)` - If configuration loading fails
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files exist but cannot be parsed.
    async fn load_project_config(
        fs: &FileSystemManager,
        project_root: &Path,
        base_config: Option<StandardConfig>,
    ) -> Result<StandardConfig> {
        let mut builder = ConfigManager::<StandardConfig>::builder().with_defaults();

        // Check for repo.config.* files in order of preference
        let config_files = [
            project_root.join("repo.config.toml"),
            project_root.join("repo.config.yml"),
            project_root.join("repo.config.yaml"),
            project_root.join("repo.config.json"),
        ];

        // Add existing config files to the builder
        for config_file in &config_files {
            if fs.exists(config_file).await {
                builder = builder.with_file(config_file);
            }
        }

        let manager = builder
            .build(fs.clone())
            .map_err(|e| Error::operation(format!("Failed to create config manager: {e}")))?;

        let mut config = manager
            .load()
            .await
            .map_err(|e| Error::operation(format!("Failed to load configuration: {e}")))?;

        // Merge with base config if provided
        if let Some(base) = base_config {
            config
                .merge_with(base)
                .map_err(|e| Error::operation(format!("Failed to merge configurations: {e}")))?;
        }

        Ok(config)
    }
}

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Creates a new `MonorepoDetector` with a custom async filesystem implementation
    /// and default monorepo configuration.
    ///
    /// # Arguments
    ///
    /// * `fs` - The async filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the provided async filesystem and default configuration.
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
        Self { fs, config: crate::config::MonorepoConfig::default() }
    }

    /// Creates a new `MonorepoDetector` with a custom async filesystem implementation
    /// and custom monorepo configuration.
    ///
    /// # Arguments
    ///
    /// * `fs` - The async filesystem implementation to use
    /// * `config` - The monorepo configuration to use
    ///
    /// # Returns
    ///
    /// A new `MonorepoDetector` instance using the provided filesystem and configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::monorepo::MonorepoDetector;
    /// use sublime_standard_tools::config::MonorepoConfig;
    ///
    /// let fs = FileSystemManager::new();
    /// let config = MonorepoConfig {
    ///     max_search_depth: 8,
    ///     ..MonorepoConfig::default()
    /// };
    /// let detector = MonorepoDetector::with_filesystem_and_config(fs, config);
    /// ```
    #[must_use]
    pub fn with_filesystem_and_config(fs: F, config: crate::config::MonorepoConfig) -> Self {
        Self { fs, config }
    }
}

#[async_trait]
impl<F: AsyncFileSystem + Clone> MonorepoDetectorTrait for MonorepoDetector<F> {
    async fn is_monorepo_root(&self, path: &Path) -> Result<Option<MonorepoKind>> {
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
                match self.fs.read_file_string(&package_json_path).await {
                    Ok(content) => {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(json_value) => {
                                if let Some(workspaces) = json_value.get("workspaces") {
                                    if !workspaces.is_null() {
                                        return Ok(Some(MonorepoKind::PnpmWorkspaces));
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse package.json for pnpm workspace detection at {}: {}", package_json_path.display(), e);
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("Could not read package.json for pnpm workspace detection at {}: {}", package_json_path.display(), e);
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
            match self.fs.read_file_string(&package_json_path).await {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json_value) => {
                            if let Some(workspaces) = json_value.get("workspaces") {
                                if !workspaces.is_null() {
                                    return Ok(Some(MonorepoKind::NpmWorkSpace));
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to parse package.json for npm workspace detection at {}: {}", package_json_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Could not read package.json for npm workspace detection at {}: {}", package_json_path.display(), e);
                }
            }
        }

        Ok(None)
    }

    async fn find_monorepo_root(
        &self,
        start_path: &Path,
    ) -> Result<Option<(PathBuf, MonorepoKind)>> {
        let mut current_path = start_path.to_path_buf();

        loop {
            if let Some(kind) = self.is_monorepo_root(&current_path).await? {
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

    async fn detect_monorepo(&self, path: &Path) -> Result<MonorepoDescriptor> {
        // First, determine if this is a monorepo root
        let kind = self.is_monorepo_root(path).await?.ok_or_else(|| {
            use crate::error::{FileSystemError, MonorepoError};
            Error::Monorepo(MonorepoError::Detection {
                source: FileSystemError::NotFound { path: path.to_path_buf() },
            })
        })?;

        // Detect packages within the monorepo
        let packages = self.detect_packages(path).await?;

        Ok(MonorepoDescriptor::new(
            kind,
            path.to_path_buf(),
            packages,
            None, // package_manager
            None, // package_json
            ProjectValidationStatus::NotValidated,
        ))
    }

    #[allow(clippy::assigning_clones)]
    async fn detect_packages(&self, root: &Path) -> Result<Vec<WorkspacePackage>> {
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

        // Extract workspace patterns from package.json
        let mut workspace_patterns = if let Some(workspaces) = json_value.get("workspaces") {
            if let Some(array) = workspaces.as_array() {
                array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            } else {
                Vec::new() // No workspaces array
            }
        } else {
            Vec::new() // No workspaces defined
        };

        // If no patterns from package.json, use config patterns
        if workspace_patterns.is_empty() {
            workspace_patterns = self.config.workspace_patterns.clone();
        } else {
            // Use a HashSet to merge patterns without duplicates
            let mut unique_patterns = std::collections::HashSet::new();

            // Add package.json patterns
            for pattern in &workspace_patterns {
                unique_patterns.insert(pattern.clone());
            }

            // Add config patterns
            for pattern in &self.config.workspace_patterns {
                unique_patterns.insert(pattern.clone());
            }

            // Convert back to Vec
            workspace_patterns = unique_patterns.into_iter().collect();
        }

        // Early return if still no patterns
        if workspace_patterns.is_empty() {
            return Ok(packages);
        }

        // Find all package.json files in workspace directories
        for pattern in workspace_patterns {
            let full_pattern = root.join(&pattern).to_string_lossy().to_string();
            if let Ok(paths) = glob::glob(&full_pattern) {
                for dir_path in paths.flatten() {
                    // Check if the path should be excluded based on config
                    if self.should_exclude_path(&dir_path) {
                        continue;
                    }

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

    async fn has_multiple_packages(&self, path: &Path) -> bool {
        if let Ok(packages) = self.detect_packages(path).await {
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

    async fn detect_packages_multiple(
        &self,
        roots: &[&Path],
    ) -> Vec<Result<Vec<WorkspacePackage>>> {
        let mut results = Vec::with_capacity(roots.len());

        // Process all roots concurrently
        let futures = roots.iter().map(|root| self.detect_packages(root));

        // Collect all results
        for future in futures {
            results.push(future.await);
        }

        results
    }

    async fn detect_packages_parallel(
        &self,
        root: &Path,
        _max_concurrent: usize,
    ) -> Result<Vec<WorkspacePackage>> {
        // For now, use the standard detect_packages
        // In a more sophisticated implementation, we would use semaphores
        // to limit concurrency
        self.detect_packages(root).await
    }
}

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Determines if a path should be excluded from package detection.
    ///
    /// This method uses the monorepo configuration exclude patterns to determine
    /// if a path should be ignored during package discovery.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check against exclude patterns
    ///
    /// # Returns
    ///
    /// `true` if the path should be excluded, `false` otherwise.
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check against configured exclude patterns
        for exclude_pattern in &self.config.exclude_patterns {
            // Simple pattern matching - in a real implementation this would use glob patterns
            if path_str.contains(exclude_pattern) {
                return true;
            }

            // Check if any component of the path matches the exclude pattern
            for component in path.components() {
                if let Some(component_str) = component.as_os_str().to_str() {
                    if component_str == exclude_pattern {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Determines if a dependency name represents a workspace dependency.
    ///
    /// This method uses the monorepo configuration to determine if a dependency
    /// is an internal workspace dependency based on configured patterns.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The dependency name to check
    ///
    /// # Returns
    ///
    /// `true` if the dependency is a workspace dependency, `false` otherwise.
    fn is_workspace_dependency(&self, dep_name: &str) -> bool {
        // Check against configured custom workspace fields/patterns
        for field in &self.config.custom_workspace_fields {
            if dep_name.starts_with(field) {
                return true;
            }
        }

        // Default heuristics for detecting workspace dependencies:
        // 1. Scoped packages that start with common workspace prefixes
        // 2. Packages that match workspace patterns
        // 3. Local/file references

        // Check for scoped packages with common workspace patterns
        if dep_name.starts_with('@') {
            // Common scoped package patterns that often indicate workspace dependencies
            let scoped_patterns = [
                "@workspace/",
                "@internal/",
                "@company/",
                "@org/",
                "@app/",
                "@shared/",
                "@common/",
                "@core/",
                "@utils/",
                "@lib/",
                "@libs/",
                "@packages/",
                "@components/",
                "@services/",
                "@tools/",
                "@monorepo/",
            ];

            for pattern in &scoped_patterns {
                if dep_name.starts_with(pattern) {
                    return true;
                }
            }
        }

        // Check for local file references
        if dep_name.starts_with("file:")
            || dep_name.starts_with("./")
            || dep_name.starts_with("../")
        {
            return true;
        }

        // Check against workspace directory patterns from config
        for pattern in &self.config.workspace_patterns {
            // Simple pattern matching - in a real implementation this would use glob patterns
            if let Some(prefix) = pattern.strip_suffix("/*") {
                let workspace_scope = format!("@{prefix}/");
                if dep_name.starts_with(&workspace_scope) {
                    return true;
                }
            }
        }

        false
    }

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

        let absolute_path = match location.canonicalize() {
            Ok(canonical) => canonical,
            Err(e) => {
                log::debug!("Failed to canonicalize path {}: {}. Using original path.", location.display(), e);
                location.clone()
            }
        };

        let name = json_value.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

        let version =
            json_value.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

        // Extract workspace dependencies
        let mut workspace_dependencies = Vec::new();
        let mut workspace_dev_dependencies = Vec::new();

        // Check regular dependencies - use custom workspace fields or detect by name patterns
        if let Some(deps) = json_value.get("dependencies") {
            if let Some(deps_obj) = deps.as_object() {
                for (dep_name, _) in deps_obj {
                    if self.is_workspace_dependency(dep_name) {
                        workspace_dependencies.push(dep_name.clone());
                    }
                }
            }
        }

        // Check dev dependencies - use custom workspace fields or detect by name patterns
        if let Some(dev_deps) = json_value.get("devDependencies") {
            if let Some(dev_deps_obj) = dev_deps.as_object() {
                for (dep_name, _) in dev_deps_obj {
                    if self.is_workspace_dependency(dep_name) {
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
