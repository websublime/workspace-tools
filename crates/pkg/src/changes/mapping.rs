//! File-to-package mapping for changes analysis.
//!
//! **What**: Provides functionality to map changed files to their containing packages,
//! supporting both single-package and monorepo project structures.
//!
//! **How**: Uses the `MonorepoDetector` from `sublime_standard_tools` to detect project
//! structure, then maps each file to its owning package by checking if the file path
//! is under a package directory. Implements caching to optimize repeated lookups.
//!
//! **Why**: To efficiently determine which packages are affected by file changes, enabling
//! accurate change analysis and version calculation in both simple and complex project structures.
//!
//! # Features
//!
//! - **Monorepo Support**: Automatically detects and handles npm/yarn/pnpm/bun workspaces
//! - **Single Package**: Handles standard single-package projects
//! - **Caching**: Caches monorepo structure and file mappings for performance
//! - **Root Files**: Handles files in the workspace root that don't belong to any package
//! - **Path Normalization**: Handles relative and absolute paths correctly
//!
//! # Examples
//!
//! ## Basic file mapping
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changes::mapping::PackageMapper;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//!
//! let mut mapper = PackageMapper::new(workspace_root, fs);
//!
//! let files = vec![
//!     PathBuf::from("packages/core/src/index.ts"),
//!     PathBuf::from("packages/utils/src/helper.ts"),
//! ];
//!
//! let package_files = mapper.map_files_to_packages(&files).await?;
//!
//! for (package_name, files) in package_files {
//!     println!("Package '{}' has {} changed files", package_name, files.len());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Finding package for a single file
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changes::mapping::PackageMapper;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//!
//! let mut mapper = PackageMapper::new(workspace_root, fs);
//!
//! let file = PathBuf::from("packages/core/src/index.ts");
//! if let Some(package_name) = mapper.find_package_for_file(&file).await? {
//!     println!("File belongs to package: {}", package_name);
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{ChangesError, ChangesResult};
use crate::types::PackageInfo;
use package_json::PackageJson;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{
    MonorepoDescriptor, MonorepoDetector, MonorepoDetectorTrait,
};

/// Maps files to their containing packages with caching support.
///
/// The `PackageMapper` analyzes the project structure (single-package or monorepo)
/// and provides efficient mapping from file paths to package names. It caches both
/// the monorepo structure and individual file mappings for optimal performance.
///
/// # Architecture
///
/// - **Lazy Detection**: Monorepo structure is detected on first use
/// - **Two-Level Cache**: Caches both the monorepo descriptor and file-to-package mappings
/// - **Absolute Path Handling**: Normalizes all paths relative to workspace root
///
/// # Cache Strategy
///
/// - **Monorepo Cache**: `Option<MonorepoDescriptor>` cached after first detection
/// - **File Mapping Cache**: `HashMap<PathBuf, Option<String>>` for individual file lookups
/// - **Cache Invalidation**: Create a new mapper instance to invalidate caches
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changes::mapping::PackageMapper;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let fs = FileSystemManager::new();
///
/// let mut mapper = PackageMapper::new(workspace_root.clone(), fs);
///
/// // First call detects monorepo structure and caches it
/// let files = vec![PathBuf::from("packages/core/src/index.ts")];
/// let result = mapper.map_files_to_packages(&files).await?;
///
/// // Subsequent calls use cached structure for better performance
/// let more_files = vec![PathBuf::from("packages/utils/src/helper.ts")];
/// let result2 = mapper.map_files_to_packages(&more_files).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageMapper<F = FileSystemManager>
where
    F: AsyncFileSystem + Clone + Send + Sync + 'static,
{
    /// Root directory of the workspace.
    workspace_root: PathBuf,

    /// Filesystem abstraction for file operations.
    fs: F,

    /// Monorepo detector for structure analysis.
    monorepo_detector: MonorepoDetector<F>,

    /// Cached monorepo descriptor (None means not yet detected or single-package).
    pub(crate) cached_monorepo: Option<Option<MonorepoDescriptor>>,

    /// Cache mapping file paths to package names.
    /// Value is Option<String> where None means file doesn't belong to any package.
    pub(crate) file_cache: HashMap<PathBuf, Option<String>>,
}

impl PackageMapper<FileSystemManager> {
    /// Creates a new `PackageMapper` with the default filesystem.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `fs` - Filesystem instance for file operations
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mapper = PackageMapper::new(workspace_root, fs);
    /// ```
    #[must_use]
    pub fn new(workspace_root: PathBuf, fs: FileSystemManager) -> Self {
        let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());

        Self {
            workspace_root,
            fs,
            monorepo_detector,
            cached_monorepo: None,
            file_cache: HashMap::new(),
        }
    }
}

impl<F> PackageMapper<F>
where
    F: AsyncFileSystem + Clone + Send + Sync + 'static,
{
    /// Creates a new `PackageMapper` with a custom filesystem.
    ///
    /// This allows using mock filesystems for testing.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `fs` - Custom filesystem implementation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use std::path::PathBuf;
    ///
    /// # async fn example<F>(fs: F) -> Result<(), Box<dyn std::error::Error>>
    /// # where F: AsyncFileSystem + Clone + Send + Sync + 'static
    /// # {
    /// let workspace_root = PathBuf::from(".");
    /// let mapper = PackageMapper::with_filesystem(workspace_root, fs);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_filesystem(workspace_root: PathBuf, fs: F) -> Self {
        let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());

        Self {
            workspace_root,
            fs,
            monorepo_detector,
            cached_monorepo: None,
            file_cache: HashMap::new(),
        }
    }

    /// Maps a list of files to their containing packages.
    ///
    /// Returns a HashMap where keys are package names and values are lists of files
    /// belonging to that package. Files that don't belong to any package are omitted.
    ///
    /// # Arguments
    ///
    /// * `files` - List of file paths to map (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// A HashMap mapping package names to lists of file paths.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Monorepo detection fails
    /// - Filesystem operations fail
    /// - package.json files cannot be read or parsed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mut mapper = PackageMapper::new(workspace_root, fs);
    ///
    /// let files = vec![
    ///     PathBuf::from("packages/core/src/index.ts"),
    ///     PathBuf::from("packages/utils/src/helper.ts"),
    ///     PathBuf::from("README.md"),
    /// ];
    ///
    /// let package_files = mapper.map_files_to_packages(&files).await?;
    ///
    /// for (package_name, pkg_files) in package_files {
    ///     println!("Package '{}' has files:", package_name);
    ///     for file in pkg_files {
    ///         println!("  - {}", file.display());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn map_files_to_packages(
        &mut self,
        files: &[PathBuf],
    ) -> ChangesResult<HashMap<String, Vec<PathBuf>>> {
        // Ensure we have detected the project structure
        self.ensure_monorepo_detected().await?;

        let mut package_files: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for file in files {
            // Normalize path relative to workspace root
            let normalized_path = self.normalize_path(file)?;

            // Find which package owns this file
            if let Some(package_name) = self.find_package_for_file(&normalized_path).await? {
                package_files.entry(package_name).or_default().push(normalized_path);
            }
            // Files not belonging to any package are silently omitted
        }

        Ok(package_files)
    }

    /// Finds the package that contains a specific file.
    ///
    /// Returns `Some(package_name)` if the file belongs to a package,
    /// or `None` if the file is outside all packages or is a root file.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(package_name))` if file belongs to a package
    /// - `Ok(None)` if file doesn't belong to any package (e.g., root files)
    /// - `Err(...)` if detection fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Monorepo detection fails
    /// - Filesystem operations fail
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mut mapper = PackageMapper::new(workspace_root, fs);
    ///
    /// // File in a package
    /// let file = PathBuf::from("packages/core/src/index.ts");
    /// if let Some(pkg) = mapper.find_package_for_file(&file).await? {
    ///     println!("File belongs to: {}", pkg);
    /// }
    ///
    /// // Root file
    /// let root_file = PathBuf::from("README.md");
    /// if mapper.find_package_for_file(&root_file).await?.is_none() {
    ///     println!("File is a root file");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_package_for_file(&mut self, file: &Path) -> ChangesResult<Option<String>> {
        // Normalize the path
        let normalized_path = self.normalize_path(file)?;

        // Check cache first
        if let Some(cached_result) = self.file_cache.get(&normalized_path) {
            return Ok(cached_result.clone());
        }

        // Ensure monorepo is detected
        self.ensure_monorepo_detected().await?;

        // Find the package
        let package_name = self.find_package_for_file_impl(&normalized_path).await?;

        // Cache the result
        self.file_cache.insert(normalized_path, package_name.clone());

        Ok(package_name)
    }

    /// Gets all packages in the workspace.
    ///
    /// Returns a list of `PackageInfo` for all packages in the workspace.
    /// For single-package projects, returns a single package.
    /// For monorepos, returns all workspace packages.
    ///
    /// # Returns
    ///
    /// A vector of `PackageInfo` instances for all packages.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Monorepo detection fails
    /// - package.json files cannot be read or parsed
    /// - No packages are found
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mut mapper = PackageMapper::new(workspace_root, fs);
    ///
    /// let packages = mapper.get_all_packages().await?;
    /// println!("Found {} packages:", packages.len());
    /// for pkg in packages {
    ///     println!("  - {} at {}", pkg.name(), pkg.path().display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_packages(&mut self) -> ChangesResult<Vec<PackageInfo>> {
        self.ensure_monorepo_detected().await?;

        if let Some(Some(monorepo)) = &self.cached_monorepo {
            // Monorepo: convert all workspace packages to PackageInfo
            let mut packages = Vec::new();
            for wp in monorepo.packages() {
                packages.push(self.workspace_package_to_package_info(wp).await?);
            }

            if packages.is_empty() {
                return Err(ChangesError::NoPackagesFound {
                    workspace_root: self.workspace_root.clone(),
                });
            }

            Ok(packages)
        } else {
            // Single package: read root package.json
            let package_info = self.read_root_package().await?;
            Ok(vec![package_info])
        }
    }

    /// Clears all caches, forcing re-detection on next operation.
    ///
    /// This is useful when the workspace structure may have changed
    /// (e.g., packages added/removed).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mut mapper = PackageMapper::new(workspace_root, fs);
    ///
    /// // Use mapper...
    /// let _ = mapper.get_all_packages().await?;
    ///
    /// // Clear caches to force re-detection
    /// mapper.clear_cache();
    ///
    /// // Next call will re-detect structure
    /// let _ = mapper.get_all_packages().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_cache(&mut self) {
        self.cached_monorepo = None;
        self.file_cache.clear();
    }

    /// Returns whether this workspace is a monorepo.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if workspace is a monorepo
    /// - `Ok(false)` if workspace is a single-package project
    ///
    /// # Errors
    ///
    /// Returns an error if monorepo detection fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::mapping::PackageMapper;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let mut mapper = PackageMapper::new(workspace_root, fs);
    ///
    /// if mapper.is_monorepo().await? {
    ///     println!("This is a monorepo workspace");
    /// } else {
    ///     println!("This is a single-package project");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_monorepo(&mut self) -> ChangesResult<bool> {
        self.ensure_monorepo_detected().await?;
        Ok(self.cached_monorepo.as_ref().map_or(false, |m| m.is_some()))
    }

    /// Ensures monorepo structure has been detected and cached.
    ///
    /// This method is called internally before operations that need the
    /// project structure information.
    async fn ensure_monorepo_detected(&mut self) -> ChangesResult<()> {
        if self.cached_monorepo.is_none() {
            // Detect monorepo structure
            let monorepo_result =
                self.monorepo_detector.detect_monorepo(&self.workspace_root).await;

            match monorepo_result {
                Ok(descriptor) => {
                    // Successfully detected as monorepo
                    self.cached_monorepo = Some(Some(descriptor));
                }
                Err(_) => {
                    // Not a monorepo (or detection failed) - treat as single package
                    // This is normal for single-package projects
                    self.cached_monorepo = Some(None);
                }
            }
        }

        Ok(())
    }

    /// Internal implementation for finding package for a file.
    ///
    /// This assumes the path is already normalized and monorepo is detected.
    async fn find_package_for_file_impl(&self, file: &Path) -> ChangesResult<Option<String>> {
        if let Some(Some(monorepo)) = &self.cached_monorepo {
            // Monorepo: use find_package_for_path
            // Convert relative path to absolute and canonicalize to handle symlinks
            let absolute_file = if file.is_absolute() {
                file.to_path_buf()
            } else {
                self.workspace_root.join(file)
            };

            // Canonicalize to handle symlinks (e.g., /var -> /private/var on macOS)
            let canonical_file = absolute_file.canonicalize().unwrap_or(absolute_file);

            if let Some(workspace_package) = monorepo.find_package_for_path(&canonical_file) {
                return Ok(Some(workspace_package.name.clone()));
            }

            // File is not under any package (root file)
            Ok(None)
        } else {
            // Single package: all files belong to the root package
            let package_info = self.read_root_package().await?;
            Ok(Some(package_info.name().to_string()))
        }
    }

    /// Normalizes a file path relative to the workspace root.
    ///
    /// Handles both relative and absolute paths, ensuring the returned path
    /// is relative to the workspace root.
    pub(crate) fn normalize_path(&self, path: &Path) -> ChangesResult<PathBuf> {
        if path.is_absolute() {
            // Strip workspace root prefix
            path.strip_prefix(&self.workspace_root).map(|p| p.to_path_buf()).map_err(|_| {
                ChangesError::FileOutsideWorkspace {
                    path: path.to_path_buf(),
                    workspace_root: self.workspace_root.clone(),
                }
            })
        } else {
            // Already relative
            Ok(path.to_path_buf())
        }
    }

    /// Reads the root package.json and creates a PackageInfo.
    async fn read_root_package(&self) -> ChangesResult<PackageInfo> {
        let package_json_path = self.workspace_root.join("package.json");

        // Read package.json
        let content = self.fs.read_file_string(&package_json_path).await.map_err(|e| {
            ChangesError::FileSystemError {
                path: package_json_path.clone(),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        // Parse as PackageJson
        let package_json: PackageJson =
            serde_json::from_str(&content).map_err(|e| ChangesError::PackageJsonParseError {
                path: package_json_path.clone(),
                reason: e.to_string(),
            })?;

        // Create PackageInfo
        Ok(PackageInfo::new(package_json, None, self.workspace_root.clone()))
    }

    /// Converts a WorkspacePackage to PackageInfo by reading its package.json.
    async fn workspace_package_to_package_info(
        &self,
        workspace_package: &sublime_standard_tools::monorepo::WorkspacePackage,
    ) -> ChangesResult<PackageInfo> {
        // Read the package.json for this workspace package
        let package_json_path = workspace_package.absolute_path.join("package.json");

        // Read package.json content
        let content = self.fs.read_file_string(&package_json_path).await.map_err(|e| {
            ChangesError::FileSystemError {
                path: package_json_path.clone(),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        // Parse as PackageJson
        let package_json: PackageJson =
            serde_json::from_str(&content).map_err(|e| ChangesError::PackageJsonParseError {
                path: package_json_path.clone(),
                reason: e.to_string(),
            })?;

        // Create PackageInfo with workspace context
        Ok(PackageInfo::new(
            package_json,
            Some(workspace_package.clone()),
            workspace_package.absolute_path.clone(),
        ))
    }
}
