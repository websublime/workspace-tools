//! Changelog generator for creating and managing changelogs.
//!
//! **What**: Provides the core `ChangelogGenerator` structure that orchestrates changelog
//! generation for both single-package and monorepo projects.
//!
//! **How**: This module integrates Git repository access, filesystem operations, and
//! configuration to generate changelogs from commit history. It supports multiple formats
//! (Keep a Changelog, Conventional Commits, custom templates) and handles version detection
//! from Git tags.
//!
//! **Why**: To provide a unified interface for changelog generation that works consistently
//! across different project structures and changelog formats, automating the creation of
//! human-readable release documentation.
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changelog::ChangelogGenerator;
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_git_tools::Repo;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let git_repo = Repo::open(".")?;
//! let fs = FileSystemManager::new();
//! let config = PackageToolsConfig::default();
//!
//! let generator = ChangelogGenerator::new(
//!     workspace_root,
//!     git_repo,
//!     fs,
//!     config.changelog
//! ).await?;
//!
//! // TODO: will be implemented in story 8.4
//! // let changelog = generator.generate_for_version("my-package", "2.0.0").await?;
//! # Ok(())
//! # }
//! ```

use crate::config::ChangelogConfig;
use crate::error::{ChangelogError, ChangelogResult};
use std::path::PathBuf;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

/// Main changelog generator for creating and managing changelogs.
///
/// The `ChangelogGenerator` is the primary entry point for all changelog operations.
/// It provides methods to generate changelogs from version history, update existing
/// changelog files, and parse commit messages using conventional commit format.
///
/// # Architecture
///
/// The generator integrates several components:
/// - **Git Repository**: Access to commit history, tags, and version detection
/// - **File System**: Reading and writing changelog files
/// - **Configuration**: Changelog format, templates, and generation options
///
/// # Supported Formats
///
/// - **Keep a Changelog**: Standard format following <https://keepachangelog.com>
/// - **Conventional Commits**: Automatic grouping by commit type (feat, fix, etc.)
/// - **Custom Templates**: User-defined formats with variable substitution
///
/// # Monorepo Support
///
/// The generator supports three monorepo modes:
/// - **Per-Package**: Each package has its own CHANGELOG.md in its directory
/// - **Root**: Single CHANGELOG.md at the repository root
/// - **Both**: Maintain both per-package and root changelogs
///
/// # Examples
///
/// ## Creating a generator
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::ChangelogGenerator;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_git_tools::Repo;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from("/path/to/workspace");
/// let git_repo = Repo::open("/path/to/workspace")?;
/// let fs = FileSystemManager::new();
/// let config = PackageToolsConfig::default();
///
/// let generator = ChangelogGenerator::new(
///     workspace_root,
///     git_repo,
///     fs,
///     config.changelog,
/// ).await?;
///
/// println!("Generator ready for workspace: {}", generator.workspace_root().display());
/// # Ok(())
/// # }
/// ```
///
/// ## Accessing configuration
///
/// ```rust,ignore
/// # use sublime_pkg_tools::changelog::ChangelogGenerator;
/// # use sublime_pkg_tools::config::{PackageToolsConfig, ChangelogFormat};
/// # use sublime_git_tools::Repo;
/// # use sublime_standard_tools::filesystem::FileSystemManager;
/// # use std::path::PathBuf;
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let workspace_root = PathBuf::from(".");
/// # let git_repo = Repo::open(".")?;
/// # let fs = FileSystemManager::new();
/// # let config = PackageToolsConfig::default();
/// #
/// let generator = ChangelogGenerator::new(
///     workspace_root,
///     git_repo,
///     fs,
///     config.changelog,
/// ).await?;
///
/// let format = generator.config().format;
/// println!("Using changelog format: {:?}", format);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ChangelogGenerator {
    /// Root path of the workspace or repository.
    ///
    /// This is used as the base path for resolving package locations and
    /// changelog file paths.
    workspace_root: PathBuf,

    /// Git repository for accessing commit history and tags.
    ///
    /// The repository provides access to commits, tags, and version detection
    /// needed for changelog generation.
    git_repo: Repo,

    /// File system manager for reading and writing changelog files.
    ///
    /// The file system abstraction enables testing with mock implementations
    /// and provides async file operations.
    fs: FileSystemManager,

    /// Changelog generation configuration.
    ///
    /// Controls the format, templates, exclusion rules, and other aspects
    /// of changelog generation.
    config: ChangelogConfig,
}

impl ChangelogGenerator {
    /// Creates a new `ChangelogGenerator` instance.
    ///
    /// This constructor initializes the generator with the provided workspace root,
    /// Git repository, file system manager, and configuration. It validates that
    /// the workspace root exists and is a valid directory.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The root path of the workspace or repository
    /// * `git_repo` - Git repository instance for accessing commit history
    /// * `fs` - File system manager for file operations
    /// * `config` - Changelog generation configuration
    ///
    /// # Returns
    ///
    /// Returns a `ChangelogResult<Self>` containing the new generator instance,
    /// or an error if initialization fails.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - The workspace root path is invalid or doesn't exist
    /// - The workspace root is not a directory
    /// - The configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// assert!(generator.workspace_root().exists());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        git_repo: Repo,
        fs: FileSystemManager,
        config: ChangelogConfig,
    ) -> ChangelogResult<Self> {
        // Validate workspace root exists and is a directory
        if !fs.exists(&workspace_root).await {
            return Err(ChangelogError::InvalidPath {
                path: workspace_root.clone(),
                reason: "Workspace root does not exist".to_string(),
            });
        }

        // Check if path is a directory by attempting to read it
        // If read_dir fails, it's likely not a directory
        if fs.read_dir(&workspace_root).await.is_err() {
            return Err(ChangelogError::InvalidPath {
                path: workspace_root.clone(),
                reason: "Workspace root is not a directory".to_string(),
            });
        }

        Ok(Self { workspace_root, git_repo, fs, config })
    }

    /// Returns a reference to the workspace root path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// println!("Workspace: {}", generator.workspace_root().display());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Returns a reference to the Git repository.
    ///
    /// This provides access to the underlying Git repository for operations
    /// that need direct access to commits, tags, or other Git functionality.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// let current_sha = generator.git_repo().get_current_sha()?;
    /// println!("Current commit: {}", current_sha);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn git_repo(&self) -> &Repo {
        &self.git_repo
    }

    /// Returns a reference to the file system manager.
    ///
    /// This provides access to the filesystem operations for reading and
    /// writing files during changelog generation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// let fs = generator.fs();
    /// let exists = fs.exists(&PathBuf::from("CHANGELOG.md")).await;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn fs(&self) -> &FileSystemManager {
        &self.fs
    }

    /// Returns a reference to the changelog configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// let config = generator.config();
    /// println!("Changelog format: {:?}", config.format);
    /// println!("Include links: {}", config.include_commit_links);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &ChangelogConfig {
        &self.config
    }

    /// Checks if changelog generation is enabled in the configuration.
    ///
    /// This is a convenience method to check the `enabled` flag in the
    /// configuration without directly accessing the config field.
    ///
    /// # Returns
    ///
    /// Returns `true` if changelog generation is enabled, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// if generator.is_enabled() {
    ///     println!("Changelog generation is enabled");
    /// } else {
    ///     println!("Changelog generation is disabled");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Gets the repository URL from configuration or Git remote.
    ///
    /// This method returns the repository URL needed for generating links to
    /// commits and issues. It first checks the configuration for an explicit
    /// repository URL, and if not found, attempts to detect it from the Git
    /// remote configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(url))` if a repository URL is found, `Ok(None)` if no
    /// URL is configured, or an error if Git operations fail.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - Git remote operations fail
    /// - The remote URL is in an invalid format
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let git_repo = Repo::open(".")?;
    /// # let fs = FileSystemManager::new();
    /// # let config = PackageToolsConfig::default();
    /// #
    /// let generator = ChangelogGenerator::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config.changelog,
    /// ).await?;
    ///
    /// if let Some(url) = generator.get_repository_url()? {
    ///     println!("Repository URL: {}", url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_repository_url(&self) -> ChangelogResult<Option<String>> {
        // Return configured URL if available
        if let Some(ref url) = self.config.repository_url {
            return Ok(Some(url.clone()));
        }

        // TODO: will be implemented in story 8.3
        // Attempt to detect from git remote
        // This would call git_repo methods to get remote URLs
        Ok(None)
    }
}
