//! Version resolver implementation for package version management.
//!
//! **What**: Provides the main `VersionResolver` struct that orchestrates version resolution,
//! dependency propagation, and version application for Node.js packages in both monorepo and
//! single-package configurations.
//!
//! **How**: The resolver detects the project structure (monorepo or single package) using
//! `MonorepoDetector` from `sublime_standard_tools`, loads all packages in the workspace,
//! and provides methods for version resolution and application. It uses the configured
//! versioning strategy (independent or unified) to determine version updates.
//!
//! **Why**: To provide a unified interface for version management that automatically adapts
//! to the project structure and handles the complexity of version resolution, dependency
//! propagation, and file updates in a safe and predictable manner.

use crate::config::PackageToolsConfig;
use crate::error::{VersionError, VersionResult};
use crate::types::{PackageInfo, VersioningStrategy};
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, WorkspacePackage};

/// Main version resolver for managing package versions.
///
/// The `VersionResolver` is the central component for version management in Node.js projects.
/// It automatically detects whether the project is a monorepo or single package, loads all
/// packages, and provides methods for version resolution and application.
///
/// # Type Parameters
///
/// * `F` - The filesystem implementation type, defaults to `FileSystemManager`
///
/// # Fields
///
/// * `workspace_root` - Root directory of the workspace/project
/// * `strategy` - Versioning strategy (independent or unified)
/// * `fs` - Filesystem implementation for I/O operations
/// * `config` - Complete package tools configuration
/// * `is_monorepo` - Whether the project is detected as a monorepo
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::VersionResolver;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from("/path/to/workspace");
/// let fs = FileSystemManager::new();
/// let config = PackageToolsConfig::default();
///
/// let resolver = VersionResolver::new(
///     workspace_root,
///     fs,
///     config,
/// ).await?;
///
/// // Check if monorepo
/// if resolver.is_monorepo() {
///     println!("Detected monorepo structure");
/// } else {
///     println!("Detected single package");
/// }
///
/// // TODO: will be implemented on story 5.4
/// // let resolution = resolver.resolve_versions(&changeset).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VersionResolver<F: AsyncFileSystem = FileSystemManager> {
    /// Root directory of the workspace/project.
    workspace_root: PathBuf,
    /// Versioning strategy (independent or unified).
    strategy: VersioningStrategy,
    /// Filesystem implementation for I/O operations.
    fs: F,
    /// Complete package tools configuration.
    config: PackageToolsConfig,
    /// Whether the project is detected as a monorepo.
    is_monorepo: bool,
}

impl VersionResolver<FileSystemManager> {
    /// Creates a new `VersionResolver` with the default filesystem.
    ///
    /// This constructor uses `FileSystemManager` from `sublime_standard_tools` for
    /// filesystem operations. It automatically detects the project structure and
    /// validates the workspace root.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace/project (by value)
    /// * `config` - Complete package tools configuration
    ///
    /// # Returns
    ///
    /// Returns a new `VersionResolver` instance or an error if:
    /// - The workspace root does not exist
    /// - The workspace root is not a directory
    /// - Package detection fails
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidWorkspaceRoot` if the workspace root is invalid.
    /// Returns `VersionError::PackageJsonError` if package.json cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let config = PackageToolsConfig::default();
    ///
    /// let resolver = VersionResolver::new(workspace_root, config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(workspace_root: PathBuf, config: PackageToolsConfig) -> VersionResult<Self> {
        let fs = FileSystemManager::new();
        Self::with_filesystem(workspace_root, fs, config).await
    }
}

impl<F: AsyncFileSystem + Clone + Send + Sync + 'static> VersionResolver<F> {
    /// Creates a new `VersionResolver` with a custom filesystem implementation.
    ///
    /// This constructor allows injecting a custom filesystem implementation, which is
    /// useful for testing with mock filesystems or using alternative I/O implementations.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace/project
    /// * `fs` - Filesystem implementation for I/O operations
    /// * `config` - Complete package tools configuration
    ///
    /// # Returns
    ///
    /// Returns a new `VersionResolver` instance or an error if workspace validation fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::InvalidWorkspaceRoot` if the workspace root is invalid.
    /// Returns `VersionError::PackageJsonError` if package.json cannot be read in single-package mode.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let resolver = VersionResolver::with_filesystem(
    ///     workspace_root,
    ///     fs,
    ///     config,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_filesystem(
        workspace_root: PathBuf,
        fs: F,
        config: PackageToolsConfig,
    ) -> VersionResult<Self> {
        // Validate workspace root exists
        if !fs.exists(&workspace_root).await {
            return Err(VersionError::InvalidWorkspaceRoot {
                path: workspace_root,
                reason: "Path does not exist".to_string(),
            });
        }

        // Detect if monorepo or single package
        // This will also validate that the workspace root is a valid directory
        let is_monorepo = Self::detect_monorepo(&workspace_root, &fs).await?;

        // Extract strategy from config
        let strategy = match config.version.strategy {
            crate::config::VersioningStrategy::Independent => VersioningStrategy::Independent,
            crate::config::VersioningStrategy::Unified => VersioningStrategy::Unified,
        };

        Ok(Self { workspace_root, strategy, fs, config, is_monorepo })
    }

    /// Returns whether the project is detected as a monorepo.
    ///
    /// This method provides access to the monorepo detection result, which is computed
    /// during initialization.
    ///
    /// # Returns
    ///
    /// Returns `true` if the project is a monorepo, `false` for single packages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: &VersionResolver) {
    /// if resolver.is_monorepo() {
    ///     println!("Working with a monorepo");
    /// } else {
    ///     println!("Working with a single package");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.is_monorepo
    }

    /// Returns the workspace root path.
    ///
    /// This method provides access to the workspace root directory path.
    ///
    /// # Returns
    ///
    /// Returns a reference to the workspace root path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: &VersionResolver) {
    /// let root = resolver.workspace_root();
    /// println!("Workspace root: {}", root.display());
    /// # }
    /// ```
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Returns the configured versioning strategy.
    ///
    /// This method provides access to the versioning strategy (independent or unified)
    /// that was configured for this resolver.
    ///
    /// # Returns
    ///
    /// Returns the versioning strategy.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # use sublime_pkg_tools::types::VersioningStrategy;
    /// # async fn example(resolver: &VersionResolver) {
    /// match resolver.strategy() {
    ///     VersioningStrategy::Independent => println!("Using independent versioning"),
    ///     VersioningStrategy::Unified => println!("Using unified versioning"),
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn strategy(&self) -> VersioningStrategy {
        self.strategy
    }

    /// Returns a reference to the filesystem implementation.
    ///
    /// This method provides access to the underlying filesystem implementation,
    /// which is useful for advanced operations or testing.
    ///
    /// # Returns
    ///
    /// Returns a reference to the filesystem.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # use sublime_standard_tools::filesystem::AsyncFileSystem;
    /// # async fn example(resolver: &VersionResolver) -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = resolver.filesystem();
    /// let exists = fs.exists(resolver.workspace_root()).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn filesystem(&self) -> &F {
        &self.fs
    }

    /// Returns a reference to the package tools configuration.
    ///
    /// This method provides access to the complete configuration including
    /// version, dependency, changeset, and other settings.
    ///
    /// # Returns
    ///
    /// Returns a reference to the configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: &VersionResolver) {
    /// let config = resolver.config();
    /// println!("Default bump: {}", config.version.default_bump);
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }

    /// Discovers all packages in the workspace.
    ///
    /// This method loads package information for all packages in the workspace:
    /// - For monorepos: Uses `MonorepoDetector` to find all workspace packages
    /// - For single packages: Loads the single package.json at the root
    ///
    /// # Returns
    ///
    /// Returns a vector of `PackageInfo` instances, one for each package found.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageNotFound` if no packages are found.
    /// Returns `VersionError::PackageJsonError` if package.json files cannot be read or parsed.
    /// Returns `VersionError::FileSystemError` if filesystem operations fail.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: &VersionResolver) -> Result<(), Box<dyn std::error::Error>> {
    /// let packages = resolver.discover_packages().await?;
    /// for package in &packages {
    ///     println!("Found package: {}", package.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_packages(&self) -> VersionResult<Vec<PackageInfo>> {
        if self.is_monorepo {
            self.discover_monorepo_packages().await
        } else {
            self.discover_single_package().await
        }
    }

    /// Detects whether the workspace is a monorepo.
    ///
    /// This method uses `MonorepoDetector` from `sublime_standard_tools` to determine
    /// if the workspace contains multiple packages (monorepo) or a single package.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory to check
    /// * `fs` - Filesystem implementation
    ///
    /// # Returns
    ///
    /// Returns `true` if monorepo is detected, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::FileSystemError` if detection fails.
    async fn detect_monorepo(workspace_root: &Path, fs: &F) -> VersionResult<bool> {
        let detector = MonorepoDetector::with_filesystem(fs.clone());

        // Check if this is a monorepo root
        let monorepo_kind = detector.is_monorepo_root(workspace_root).await.map_err(|e| {
            VersionError::FileSystemError {
                path: workspace_root.to_path_buf(),
                reason: format!("Failed to detect monorepo: {}", e),
            }
        })?;

        Ok(monorepo_kind.is_some())
    }

    /// Discovers packages in a monorepo workspace.
    ///
    /// This method uses `MonorepoDetector` to find all workspace packages and
    /// converts them to `PackageInfo` instances.
    ///
    /// # Returns
    ///
    /// Returns a vector of `PackageInfo` instances for all packages in the monorepo.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageNotFound` if no packages are found.
    /// Returns `VersionError::PackageJsonError` if package.json files cannot be read or parsed.
    async fn discover_monorepo_packages(&self) -> VersionResult<Vec<PackageInfo>> {
        let detector = MonorepoDetector::with_filesystem(self.fs.clone());

        // Detect the monorepo structure
        let monorepo = detector.detect_monorepo(&self.workspace_root).await.map_err(|e| {
            VersionError::FileSystemError {
                path: self.workspace_root.clone(),
                reason: format!("Failed to detect monorepo: {}", e),
            }
        })?;

        // Get all workspace packages
        let workspace_packages = monorepo.packages();

        if workspace_packages.is_empty() {
            return Err(VersionError::PackageNotFound {
                name: "any package".to_string(),
                workspace_root: self.workspace_root.clone(),
            });
        }

        // Convert workspace packages to PackageInfo
        let mut packages = Vec::with_capacity(workspace_packages.len());
        for workspace_package in workspace_packages {
            let package_info = self
                .load_package_info(
                    &workspace_package.absolute_path,
                    Some(workspace_package.clone()),
                )
                .await?;
            packages.push(package_info);
        }

        Ok(packages)
    }

    /// Discovers the single package at the workspace root.
    ///
    /// This method loads the package.json file from the workspace root and
    /// creates a `PackageInfo` instance for it.
    ///
    /// # Returns
    ///
    /// Returns a vector containing a single `PackageInfo` instance.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageJsonError` if package.json cannot be read or parsed.
    async fn discover_single_package(&self) -> VersionResult<Vec<PackageInfo>> {
        let package_info = self.load_package_info(&self.workspace_root, None).await?;
        Ok(vec![package_info])
    }

    /// Loads package information from a package directory.
    ///
    /// This method reads and parses the package.json file at the given path and
    /// creates a `PackageInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `package_path` - Directory containing the package.json file
    /// * `workspace_package` - Optional workspace package information (for monorepos)
    ///
    /// # Returns
    ///
    /// Returns a `PackageInfo` instance for the package.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::PackageJsonError` if the package.json file cannot be
    /// read, parsed, or is missing required fields.
    async fn load_package_info(
        &self,
        package_path: &Path,
        workspace_package: Option<WorkspacePackage>,
    ) -> VersionResult<PackageInfo> {
        let package_json_path = package_path.join("package.json");

        // Read package.json file
        let content = self.fs.read_file_string(&package_json_path).await.map_err(|e| {
            VersionError::PackageJsonError {
                path: package_json_path.clone(),
                reason: format!("Failed to read file: {}", e),
            }
        })?;

        // Parse package.json
        let package_json: package_json::PackageJson =
            serde_json::from_str(&content).map_err(|e| VersionError::PackageJsonError {
                path: package_json_path.clone(),
                reason: format!("Failed to parse JSON: {}", e),
            })?;

        // Create PackageInfo with workspace package info if available
        Ok(PackageInfo::new(package_json, workspace_package, package_path.to_path_buf()))
    }
}
