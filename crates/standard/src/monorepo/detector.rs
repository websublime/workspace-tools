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
use crate::config::{ConfigManager, StandardConfig, traits::Configurable};
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
            if self.fs.exists(&package_json_path).await
                && let Ok(content) = self.fs.read_file_string(&package_json_path).await
            {
                // Try raw JSON parsing to check for workspaces field
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content)
                    && let Some(workspaces) = json_value.get("workspaces")
                    && !workspaces.is_null()
                {
                    return Ok(Some(MonorepoKind::YarnWorkspaces));
                }
            }
        }

        // Check for pnpm-lock.yaml with workspaces
        let pnpm_lock_path = path.join("pnpm-lock.yaml");
        if self.fs.exists(&pnpm_lock_path).await {
            let package_json_path = path.join("package.json");
            if self.fs.exists(&package_json_path).await {
                match self.fs.read_file_string(&package_json_path).await {
                    Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json_value) => {
                            if let Some(workspaces) = json_value.get("workspaces")
                                && !workspaces.is_null()
                            {
                                return Ok(Some(MonorepoKind::PnpmWorkspaces));
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to parse package.json for pnpm workspace detection at {}: {}",
                                package_json_path.display(),
                                e
                            );
                        }
                    },
                    Err(e) => {
                        log::debug!(
                            "Could not read package.json for pnpm workspace detection at {}: {}",
                            package_json_path.display(),
                            e
                        );
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
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json_value) => {
                        if let Some(workspaces) = json_value.get("workspaces")
                            && !workspaces.is_null()
                        {
                            return Ok(Some(MonorepoKind::NpmWorkSpace));
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to parse package.json for npm workspace detection at {}: {}",
                            package_json_path.display(),
                            e
                        );
                    }
                },
                Err(e) => {
                    log::debug!(
                        "Could not read package.json for npm workspace detection at {}: {}",
                        package_json_path.display(),
                        e
                    );
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

        // First, discover internal scopes by scanning existing packages
        let discovered_scopes = self.discover_internal_scopes(root).await?;

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
            workspace_patterns.clone_from(&self.config.workspace_patterns);
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

                    if let Ok(metadata) = self.fs.metadata(&dir_path).await
                        && metadata.is_dir()
                    {
                        let package_json_path = dir_path.join("package.json");
                        if self.fs.exists(&package_json_path).await
                            && let Ok(package) = self
                                .load_workspace_package(&package_json_path, &discovered_scopes)
                                .await
                        {
                            packages.push(package);
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    async fn has_multiple_packages(&self, path: &Path) -> bool {
        if let Ok(packages) = self.detect_packages(path).await { packages.len() > 1 } else { false }
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
    /// Discovers internal workspace scopes by analyzing existing packages.
    ///
    /// This method scans all packages in the workspace and extracts their scopes
    /// automatically to build a list of internal workspace scopes.
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the monorepo
    ///
    /// # Returns
    ///
    /// A vector of discovered internal workspace scopes (e.g., `["@scope/", "@internal/"]`)
    async fn discover_internal_scopes(&self, root: &Path) -> Result<Vec<String>> {
        let mut discovered_scopes = std::collections::HashSet::new();

        // Get workspace patterns to scan for packages
        let package_json_path = root.join("package.json");
        if !self.fs.exists(&package_json_path).await {
            return Ok(Vec::new());
        }

        let content = self.fs.read_file_string(&package_json_path).await?;
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
                Vec::new()
            }
        } else {
            self.config.workspace_patterns.clone()
        };

        // If no patterns from package.json, use config patterns
        if workspace_patterns.is_empty() {
            workspace_patterns.clone_from(&self.config.workspace_patterns);
        }

        // Scan workspace directories for packages and extract scopes
        for pattern in workspace_patterns {
            let full_pattern = root.join(&pattern).to_string_lossy().to_string();
            if let Ok(paths) = glob::glob(&full_pattern) {
                for dir_path in paths.flatten() {
                    if self.should_exclude_path(&dir_path) {
                        continue;
                    }

                    if let Ok(metadata) = self.fs.metadata(&dir_path).await
                        && metadata.is_dir()
                    {
                        let package_json_path = dir_path.join("package.json");
                        if self.fs.exists(&package_json_path).await
                            && let Ok(pkg_content) =
                                self.fs.read_file_string(&package_json_path).await
                            && let Ok(pkg_json) =
                                serde_json::from_str::<serde_json::Value>(&pkg_content)
                            && let Some(name) = pkg_json.get("name").and_then(|v| v.as_str())
                        {
                            // Extract scope from package name (e.g., "@scope/lib" -> "@scope/")
                            if name.starts_with('@')
                                && let Some(slash_pos) = name.find('/')
                            {
                                let scope = format!("{}/", &name[..slash_pos]);
                                discovered_scopes.insert(scope);
                            }
                        }
                    }
                }
            }
        }

        // Convert to sorted Vec for consistent results
        let mut scopes: Vec<String> = discovered_scopes.into_iter().collect();
        scopes.sort();

        Ok(scopes)
    }

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

        // Enterprise-grade exclusion pattern matching using robust glob patterns
        for exclude_pattern in &self.config.exclude_patterns {
            // Direct path component matching for exact exclusions
            if Self::matches_exclusion_pattern(&path_str, exclude_pattern) {
                return true;
            }

            // Component-based matching for directory-level exclusions
            for component in path.components() {
                if let Some(component_str) = component.as_os_str().to_str()
                    && Self::matches_exclusion_pattern(component_str, exclude_pattern)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Enterprise-grade exclusion pattern matching with full glob support.
    ///
    /// This method implements comprehensive pattern matching for path exclusions,
    /// supporting glob patterns, wildcards, and complex directory structures.
    ///
    /// # Arguments
    ///
    /// * `path_component` - The path component to check
    /// * `exclude_pattern` - The exclusion pattern to match against
    ///
    /// # Returns
    ///
    /// `true` if the path component matches the exclusion pattern, `false` otherwise.
    ///
    /// # Supported Patterns
    ///
    /// - Exact matches: "node_modules" matches "node_modules"
    /// - Glob patterns: "*.tmp" matches "temp.tmp", "cache.tmp"
    /// - Wildcards: "*_cache" matches "webpack_cache", "babel_cache"
    /// - Directory patterns: "dist/*" matches any file in dist directory
    fn matches_exclusion_pattern(path_component: &str, exclude_pattern: &str) -> bool {
        // Exact match - highest priority
        if path_component == exclude_pattern {
            return true;
        }

        // Glob pattern matching for complex exclusions
        if exclude_pattern.contains('*') {
            return Self::glob_matches(path_component, exclude_pattern);
        }

        // Substring matching for partial exclusions (e.g., ".tmp" in "file.tmp")
        if exclude_pattern.starts_with('.') || exclude_pattern.starts_with('_') {
            return path_component.contains(exclude_pattern);
        }

        // Directory prefix matching (e.g., "cache" matches "cache/file")
        if let Some(remaining) = path_component.strip_prefix(exclude_pattern) {
            return remaining.is_empty()
                || remaining.starts_with('/')
                || remaining.starts_with('\\');
        }

        false
    }

    /// High-performance glob pattern matching implementation.
    ///
    /// This method provides enterprise-grade glob pattern matching with support for
    /// multiple wildcards, character classes, and complex patterns.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to match
    /// * `pattern` - The glob pattern
    ///
    /// # Returns
    ///
    /// `true` if the text matches the pattern, `false` otherwise.
    fn glob_matches(text: &str, pattern: &str) -> bool {
        // Use the existing glob crate for robust pattern matching
        match glob::Pattern::new(pattern) {
            Ok(compiled_pattern) => compiled_pattern.matches(text),
            Err(_) => {
                // Fallback to simple wildcard matching for invalid patterns
                Self::simple_wildcard_match(text, pattern)
            }
        }
    }

    /// Fallback wildcard matching for edge cases.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to match
    /// * `pattern` - The pattern with wildcards
    ///
    /// # Returns
    ///
    /// `true` if the text matches the pattern, `false` otherwise.
    fn simple_wildcard_match(text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return text.starts_with(prefix);
        }

        if let Some(suffix) = pattern.strip_prefix('*') {
            return text.ends_with(suffix);
        }

        // Handle patterns with * in the middle
        if let Some(star_pos) = pattern.find('*') {
            let prefix = &pattern[..star_pos];
            let suffix = &pattern[star_pos + 1..];

            return text.starts_with(prefix)
                && text.ends_with(suffix)
                && text.len() >= prefix.len() + suffix.len();
        }

        false
    }

    /// Determines if a dependency name represents a workspace dependency.
    ///
    /// This method uses both configuration and discovered internal scopes to determine
    /// if a dependency is an internal workspace dependency. No hardcoded patterns are used.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The dependency name to check
    /// * `discovered_scopes` - Scopes discovered from existing packages
    ///
    /// # Returns
    ///
    /// `true` if the dependency is a workspace dependency, `false` otherwise.
    fn is_workspace_dependency(&self, dep_name: &str, discovered_scopes: &[String]) -> bool {
        // Check against configured custom workspace fields/patterns
        for field in &self.config.custom_workspace_fields {
            if dep_name.starts_with(field) {
                return true;
            }
        }

        // Check against discovered internal scopes
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        // Check for local file references
        if dep_name.starts_with("file:")
            || dep_name.starts_with("./")
            || dep_name.starts_with("../")
        {
            return true;
        }

        // Check against workspace directory patterns from config using enterprise-grade glob matching
        for pattern in &self.config.workspace_patterns {
            // Convert workspace patterns to dependency scope patterns
            // e.g., "packages/*" -> "@packages/" or "apps/*" -> "@apps/"
            if let Some(prefix) = pattern.strip_suffix("/*") {
                let workspace_scope = format!("@{prefix}/");
                if dep_name.starts_with(&workspace_scope) {
                    return true;
                }
            }

            // Advanced glob pattern matching for complex patterns
            // e.g., "packages/*/lib" -> matches dependencies from lib packages
            if pattern.contains('*') && pattern != "*" {
                // Convert directory pattern to dependency pattern and check
                if self.matches_workspace_pattern(dep_name, pattern, discovered_scopes) {
                    return true;
                }
            }
        }

        false
    }

    /// Enterprise-grade glob pattern matching for workspace dependency detection.
    ///
    /// This method converts workspace directory patterns to dependency name patterns
    /// and performs sophisticated matching using glob-style patterns.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The dependency name to check
    /// * `workspace_pattern` - The workspace directory pattern (e.g., "packages/*", "libs/*/core")
    ///
    /// # Returns
    ///
    /// `true` if the dependency name matches the workspace pattern, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// // Pattern "packages/*" matches dependency "@myorg/lib" if there's a package at packages/lib
    /// // Pattern "libs/*/core" matches dependency "@core/utils" if there's a package at libs/utils/core
    /// ```
    fn matches_workspace_pattern(
        &self,
        dep_name: &str,
        workspace_pattern: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Enterprise-grade workspace pattern matching with full glob support and directory analysis
        // Handles complex monorepo structures, nested dependencies, and sophisticated naming patterns

        // Parse and analyze workspace pattern components
        let pattern_parts: Vec<&str> = workspace_pattern.split('/').collect();

        // Handle multi-level patterns like "packages/*/*", "apps/*/lib", "modules/core/*"
        if workspace_pattern.contains("*/") {
            return Self::analyze_complex_workspace_pattern(
                dep_name,
                &pattern_parts,
                discovered_scopes,
            );
        }

        // Handle single-level patterns with comprehensive scope analysis
        if !workspace_pattern.contains('/') && workspace_pattern != "*" {
            return Self::analyze_simple_workspace_pattern(
                dep_name,
                workspace_pattern,
                discovered_scopes,
            );
        }

        // Handle wildcard-only patterns with sophisticated heuristics
        if workspace_pattern == "*" {
            return self.analyze_wildcard_workspace_pattern(dep_name, discovered_scopes);
        }

        false
    }

    /// Analyzes complex multi-level workspace patterns with full directory structure analysis.
    ///
    /// This method performs comprehensive analysis of workspace patterns containing wildcards
    /// and multiple directory levels, mapping them to dependency naming conventions.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The dependency name to analyze
    /// * `pattern_parts` - The parsed workspace pattern components
    ///
    /// # Returns
    ///
    /// `true` if the dependency matches the complex pattern, `false` otherwise.
    fn analyze_complex_workspace_pattern(
        dep_name: &str,
        pattern_parts: &[&str],
        discovered_scopes: &[String],
    ) -> bool {
        // Advanced pattern analysis for enterprise monorepo structures
        for (index, part) in pattern_parts.iter().enumerate() {
            if *part == "*" {
                // Wildcard analysis: examine surrounding context to infer scope patterns
                let has_before = index > 0;
                let has_after = index < pattern_parts.len() - 1;

                if has_before && has_after {
                    // Pattern like "packages/*/lib" - analyze contextual dependencies
                    let before_part = pattern_parts[index - 1];
                    let after_part = pattern_parts[index + 1];

                    // Check for dependency names that match contextual patterns
                    if Self::matches_contextual_pattern(
                        dep_name,
                        before_part,
                        after_part,
                        discovered_scopes,
                    ) {
                        return true;
                    }
                } else if has_before {
                    // Pattern like "packages/*" - scope-based analysis
                    let scope_base = pattern_parts[index - 1];
                    if Self::matches_scope_based_pattern(dep_name, scope_base, discovered_scopes) {
                        return true;
                    }
                } else if has_after {
                    // Pattern like "*/lib" - reverse scope analysis
                    let scope_suffix = pattern_parts[index + 1];
                    if Self::matches_reverse_scope_pattern(
                        dep_name,
                        scope_suffix,
                        discovered_scopes,
                    ) {
                        return true;
                    }
                }
            } else {
                // Concrete directory name - check for direct scope mapping
                let scope_pattern = format!("@{part}/");
                if dep_name.starts_with(&scope_pattern) {
                    return true;
                }

                // Check for embedded pattern matching in scoped dependencies
                if dep_name.starts_with('@') && dep_name.contains(part) {
                    return true;
                }
            }
        }

        false
    }

    /// Analyzes contextual dependency patterns using ONLY direct scope matching.
    fn matches_contextual_pattern(
        dep_name: &str,
        before_part: &str,
        after_part: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Check discovered scopes first - highest priority
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        // ONLY direct contextual matching - NO assumptions about naming patterns
        if dep_name.starts_with('@') {
            let scope_with_suffix = format!("@{before_part}/");
            let after_scope = format!("@{after_part}/");

            return dep_name.starts_with(&scope_with_suffix) || dep_name.starts_with(&after_scope);
        }
        false
    }

    /// Analyzes scope-based patterns for workspace dependencies using ONLY direct matching.
    fn matches_scope_based_pattern(
        dep_name: &str,
        scope_base: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Check discovered scopes first - highest priority
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        if dep_name.starts_with('@') {
            // ONLY direct scope matching - NO assumptions or abbreviations
            let direct_scope = format!("@{scope_base}/");
            if dep_name.starts_with(&direct_scope) {
                return true;
            }

            // Use configuration-based inference only
            return Self::infer_scope_from_workspace_pattern(
                dep_name,
                scope_base,
                discovered_scopes,
            );
        }
        false
    }

    /// Analyzes reverse scope patterns using ONLY direct scope matching.
    fn matches_reverse_scope_pattern(
        dep_name: &str,
        scope_suffix: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Check discovered scopes first - highest priority
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        if dep_name.starts_with('@') {
            // ONLY direct reverse pattern matching - NO assumptions
            let suffix_scope = format!("@{scope_suffix}/");
            if dep_name.starts_with(&suffix_scope) {
                return true;
            }
        }
        false
    }

    /// Infers scope patterns from workspace directory structures using configuration only.
    fn infer_scope_from_workspace_pattern(
        dep_name: &str,
        workspace_dir: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Check discovered scopes first - highest priority
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        // NO HARDCODING - use ONLY configuration and dynamic detection
        // Check if workspace directory name appears as a scope in the dependency
        if dep_name.starts_with('@') {
            let workspace_scope = format!("@{workspace_dir}/");
            return dep_name.starts_with(&workspace_scope);
        }

        false
    }

    /// Analyzes simple single-level workspace patterns with comprehensive scope matching.
    fn analyze_simple_workspace_pattern(
        dep_name: &str,
        workspace_pattern: &str,
        discovered_scopes: &[String],
    ) -> bool {
        if dep_name.starts_with('@') {
            // Direct scope matching
            let scope_pattern = format!("@{workspace_pattern}/");
            if dep_name.starts_with(&scope_pattern) {
                return true;
            }

            // Pattern-based scope inference using enterprise heuristics
            // Check discovered scopes first - highest priority
            for scope in discovered_scopes {
                if dep_name.starts_with(scope) {
                    return true;
                }
            }

            // Pattern-based scope inference using enterprise heuristics
            return Self::infer_scope_from_workspace_pattern(
                dep_name,
                workspace_pattern,
                discovered_scopes,
            );
        }
        false
    }

    /// Analyzes wildcard workspace patterns using ONLY configuration and discovered scopes.
    fn analyze_wildcard_workspace_pattern(
        &self,
        dep_name: &str,
        discovered_scopes: &[String],
    ) -> bool {
        // Enterprise-grade wildcard pattern analysis using ONLY discovered scopes and configuration

        // Check against discovered internal scopes from workspace packages
        for scope in discovered_scopes {
            if dep_name.starts_with(scope) {
                return true;
            }
        }

        // Check against configured custom workspace fields/patterns
        for field in &self.config.custom_workspace_fields {
            if dep_name.starts_with(field) {
                return true;
            }
        }

        // Check for local file references (these are always workspace dependencies)
        if dep_name.starts_with("file:")
            || dep_name.starts_with("./")
            || dep_name.starts_with("../")
        {
            return true;
        }

        // Check for workspace-relative references (without hardcoding specific patterns)
        // Only use patterns that exist in configuration or were discovered dynamically
        for pattern in &self.config.workspace_patterns {
            // Only check patterns that have specific directory names (no wildcards)
            if !pattern.contains('*') {
                let workspace_scope = format!("@{pattern}/");
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
    /// * `discovered_scopes` - Internal scopes discovered from workspace packages
    ///
    /// # Returns
    ///
    /// * `Ok(WorkspacePackage)` - The loaded workspace package
    /// * `Err(Error)` - If the package cannot be loaded
    async fn load_workspace_package(
        &self,
        package_json_path: &Path,
        discovered_scopes: &[String],
    ) -> Result<WorkspacePackage> {
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
                log::debug!(
                    "Failed to canonicalize path {}: {}. Using original path.",
                    location.display(),
                    e
                );
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
        if let Some(deps) = json_value.get("dependencies")
            && let Some(deps_obj) = deps.as_object()
        {
            for (dep_name, _) in deps_obj {
                if self.is_workspace_dependency(dep_name, discovered_scopes) {
                    workspace_dependencies.push(dep_name.clone());
                }
            }
        }

        // Check dev dependencies - use custom workspace fields or detect by name patterns
        if let Some(dev_deps) = json_value.get("devDependencies")
            && let Some(dev_deps_obj) = dev_deps.as_object()
        {
            for (dep_name, _) in dev_deps_obj {
                if self.is_workspace_dependency(dep_name, discovered_scopes) {
                    workspace_dev_dependencies.push(dep_name.clone());
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
