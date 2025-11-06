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
use crate::types::{Changeset, DependencyType, PackageInfo, VersioningStrategy};
use crate::version::application::ApplyResult;
use crate::version::graph::DependencyGraph;
use crate::version::propagation::DependencyPropagator;
use crate::version::resolution::{PackageUpdate, VersionResolution, resolve_versions};
use package_json::PackageJson;
use std::collections::HashMap;
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
/// // Resolve versions from changeset
/// let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
/// changeset.add_package("@myorg/package");
///
/// let resolution = resolver.resolve_versions(&changeset).await?;
/// for update in &resolution.updates {
///     println!("{}: {} -> {}", update.name, update.current_version, update.next_version);
/// }
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

    /// Resolves versions for packages in a changeset.
    ///
    /// This method calculates the next versions for all packages in the changeset based on
    /// their current versions and the bump type specified in the changeset. It uses the
    /// configured versioning strategy (independent or unified) to determine how versions
    /// are calculated.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset containing packages and bump type
    ///
    /// # Returns
    ///
    /// Returns a `VersionResolution` containing all package updates with their current
    /// and next versions, along with the reason for each update.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - A package in the changeset is not found in the workspace
    /// - A package has an invalid version in package.json
    /// - Version bump calculation fails
    /// - Filesystem operations fail
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let config = PackageToolsConfig::default();
    /// let resolver = VersionResolver::new(workspace_root, config).await?;
    ///
    /// let mut changeset = Changeset::new(
    ///     "main",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    /// changeset.add_package("@myorg/core");
    /// changeset.add_package("@myorg/utils");
    ///
    /// let resolution = resolver.resolve_versions(&changeset).await?;
    ///
    /// for update in &resolution.updates {
    ///     println!("{}: {} -> {}",
    ///         update.name,
    ///         update.current_version,
    ///         update.next_version
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_versions(
        &self,
        changeset: &Changeset,
    ) -> VersionResult<VersionResolution> {
        // Discover all packages in the workspace
        let package_list = self.discover_packages().await?;

        // Build dependency graph for propagation (before consuming package_list)
        let (graph, circular_deps) = if self.config.dependency.propagation_bump != "none" {
            let g = DependencyGraph::from_packages(&package_list)?;
            let cycles = g.detect_cycles();
            (Some(g), cycles)
        } else {
            (None, Vec::new())
        };

        // Build a map of package name to package info (consuming package_list)
        let mut packages = HashMap::new();
        for package_info in package_list {
            let name = package_info.name().to_string();
            packages.insert(name, package_info);
        }

        // Step 1: Resolve direct version changes from changeset
        let mut resolution = resolve_versions(changeset, &packages, self.strategy).await?;

        // Step 2: Add circular dependencies to resolution
        resolution.circular_dependencies = circular_deps;

        // Step 3: Apply dependency propagation if configured
        if let Some(graph) = graph {
            // Create propagator and apply propagation
            let propagator = DependencyPropagator::new(&graph, &packages, &self.config.dependency);
            propagator.propagate(&mut resolution)?;
        }

        Ok(resolution)
    }

    /// Applies version changes from a changeset to package.json files.
    ///
    /// This method resolves versions for all packages in the changeset and optionally
    /// writes the updated versions and dependency references to package.json files.
    /// When `dry_run` is true, no files are modified and the method only returns
    /// what would be changed.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset containing packages and version bump information
    /// * `dry_run` - If true, only preview changes without modifying files (default: true)
    ///
    /// # Returns
    ///
    /// Returns an `ApplyResult` containing the resolution details, list of modified
    /// files (empty for dry-run), and summary statistics.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Version resolution fails (package not found, invalid version, etc.)
    /// - File reading or writing fails (dry_run = false)
    /// - JSON serialization fails
    /// - Backup or rollback operations fail
    ///
    /// # Examples
    ///
    /// ## Preview changes (dry-run)
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let config = PackageToolsConfig::default();
    /// let resolver = VersionResolver::new(workspace_root, config).await?;
    ///
    /// let mut changeset = Changeset::new(
    ///     "main",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    /// changeset.add_package("@myorg/core");
    ///
    /// // Preview changes without modifying files
    /// let result = resolver.apply_versions(&changeset, true).await?;
    ///
    /// assert!(result.dry_run);
    /// assert!(result.modified_files.is_empty());
    ///
    /// println!("Would update {} packages:", result.summary.packages_updated);
    /// for update in &result.resolution.updates {
    ///     println!("  {}: {} -> {}",
    ///         update.name,
    ///         update.current_version,
    ///         update.next_version
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Apply changes to files
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let config = PackageToolsConfig::default();
    /// let resolver = VersionResolver::new(workspace_root, config).await?;
    ///
    /// let mut changeset = Changeset::new(
    ///     "main",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    /// changeset.add_package("@myorg/core");
    ///
    /// // Apply changes to package.json files
    /// let result = resolver.apply_versions(&changeset, false).await?;
    ///
    /// assert!(!result.dry_run);
    /// println!("Modified {} files:", result.modified_files.len());
    /// for file in &result.modified_files {
    ///     println!("  - {}", file.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_versions(
        &self,
        changeset: &Changeset,
        dry_run: bool,
    ) -> VersionResult<ApplyResult> {
        // First, resolve versions to get all the updates
        let resolution = self.resolve_versions(changeset).await?;

        // If dry-run, return early without modifying files
        if dry_run {
            return Ok(ApplyResult::new(true, resolution, vec![]));
        }

        // Discover all packages again to have full package info with paths
        let package_list = self.discover_packages().await?;
        let mut packages = HashMap::new();
        for package_info in package_list {
            let name = package_info.name().to_string();
            packages.insert(name, package_info);
        }

        // Track modified files and backups for rollback
        let mut modified_files = Vec::new();
        let mut backups: Vec<(PathBuf, Vec<u8>)> = Vec::new();

        // Apply updates to each package
        let apply_result = self
            .apply_updates_to_packages(&resolution, &packages, &mut modified_files, &mut backups)
            .await;

        // If there was an error, restore backups
        if let Err(e) = apply_result {
            self.restore_backups(&backups).await?;
            return Err(e);
        }

        Ok(ApplyResult::new(false, resolution, modified_files))
    }

    /// Applies version updates to all packages in the resolution.
    ///
    /// This internal method iterates through all package updates and writes
    /// the new versions and dependency references to package.json files.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The version resolution containing all updates
    /// * `packages` - Map of package names to package information
    /// * `modified_files` - Vector to track modified file paths
    /// * `backups` - Vector to store backup data for rollback
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    async fn apply_updates_to_packages(
        &self,
        resolution: &VersionResolution,
        packages: &HashMap<String, PackageInfo>,
        modified_files: &mut Vec<PathBuf>,
        backups: &mut Vec<(PathBuf, Vec<u8>)>,
    ) -> VersionResult<()> {
        for update in &resolution.updates {
            let package_info =
                packages.get(&update.name).ok_or_else(|| VersionError::PackageNotFound {
                    name: update.name.clone(),
                    workspace_root: self.workspace_root.clone(),
                })?;

            let package_json_path = self.write_package_json(package_info, update, backups).await?;

            modified_files.push(package_json_path);
        }

        Ok(())
    }

    /// Writes updated version and dependencies to a package.json file.
    ///
    /// This method reads the current package.json, creates a backup, updates
    /// the version field and dependency references, then writes the file back
    /// with preserved formatting.
    ///
    /// # Arguments
    ///
    /// * `package` - Information about the package to update
    /// * `update` - The version update to apply
    /// * `backups` - Vector storing backup data
    ///
    /// # Returns
    ///
    /// Returns the path to the modified package.json file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File reading fails
    /// - JSON parsing fails
    /// - Backup creation fails
    /// - File writing fails
    async fn write_package_json(
        &self,
        package: &PackageInfo,
        update: &PackageUpdate,
        backups: &mut Vec<(PathBuf, Vec<u8>)>,
    ) -> VersionResult<PathBuf> {
        let package_json_path = package.path().join("package.json");

        // Read current package.json content
        let current_content = self.fs.read_file(&package_json_path).await.map_err(|e| {
            VersionError::FileSystemError {
                path: package_json_path.clone(),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        // Create backup before modifying
        backups.push((package_json_path.clone(), current_content.clone()));

        // Parse package.json
        let mut pkg_json: PackageJson = serde_json::from_slice(&current_content).map_err(|e| {
            VersionError::PackageJsonError {
                path: package_json_path.clone(),
                reason: format!("Failed to parse JSON: {}", e),
            }
        })?;

        // Update version
        pkg_json.version = update.next_version.to_string();

        // Update dependency references
        for dep_update in &update.dependency_updates {
            // Skip workspace protocols and local references
            if Self::is_skipped_version_spec(&dep_update.old_version_spec) {
                continue;
            }

            match dep_update.dependency_type {
                DependencyType::Regular => {
                    if let Some(deps) = &mut pkg_json.dependencies {
                        deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
                DependencyType::Dev => {
                    if let Some(dev_deps) = &mut pkg_json.dev_dependencies {
                        dev_deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
                DependencyType::Peer => {
                    if let Some(peer_deps) = &mut pkg_json.peer_dependencies {
                        peer_deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
                DependencyType::Optional => {
                    if let Some(optional_deps) = &mut pkg_json.optional_dependencies {
                        optional_deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
            }
        }

        // Serialize with pretty formatting
        let json_string =
            serde_json::to_string_pretty(&pkg_json).map_err(|e| VersionError::ApplyFailed {
                path: package_json_path.clone(),
                reason: format!("Failed to serialize JSON: {}", e),
            })?;

        // Write to file using filesystem manager
        self.fs.write_file_string(&package_json_path, &json_string).await.map_err(|e| {
            VersionError::ApplyFailed {
                path: package_json_path.clone(),
                reason: format!("Failed to write package.json: {}", e),
            }
        })?;

        Ok(package_json_path)
    }

    /// Checks if a version spec should be skipped (workspace protocols and local references).
    ///
    /// This method identifies version specifications that should not be updated:
    /// - `workspace:*` - workspace protocol
    /// - `file:` - file protocol
    /// - `link:` - link protocol
    /// - `portal:` - portal protocol
    ///
    /// # Arguments
    ///
    /// * `version_spec` - The version specification string to check
    ///
    /// # Returns
    ///
    /// Returns true if the version spec should be skipped.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(VersionResolver::is_skipped_version_spec("workspace:*"));
    /// assert!(VersionResolver::is_skipped_version_spec("file:../local-pkg"));
    /// assert!(VersionResolver::is_skipped_version_spec("link:../linked"));
    /// assert!(VersionResolver::is_skipped_version_spec("portal:../portal"));
    /// assert!(!VersionResolver::is_skipped_version_spec("^1.2.3"));
    /// assert!(!VersionResolver::is_skipped_version_spec("~2.0.0"));
    /// ```
    pub(crate) fn is_skipped_version_spec(version_spec: &str) -> bool {
        version_spec.starts_with("workspace:")
            || version_spec.starts_with("file:")
            || version_spec.starts_with("link:")
            || version_spec.starts_with("portal:")
    }

    /// Restores backed-up files after a failed operation.
    ///
    /// This method is called when an error occurs during version application
    /// to roll back any files that were modified before the error occurred.
    ///
    /// # Arguments
    ///
    /// * `backups` - Vector of (path, content) tuples to restore
    ///
    /// # Errors
    ///
    /// Returns an error if any restore operation fails.
    async fn restore_backups(&self, backups: &[(PathBuf, Vec<u8>)]) -> VersionResult<()> {
        for (path, content) in backups {
            self.fs.write_file(path, content).await.map_err(|e| VersionError::ApplyFailed {
                path: path.clone(),
                reason: format!("Failed to restore backup: {}", e),
            })?;
        }
        Ok(())
    }
}
