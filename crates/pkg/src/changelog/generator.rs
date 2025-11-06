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
//! // Generate changelog for a specific version
//! let changelog = generator.generate_for_version(Some("my-package"), "2.0.0", Some("1.0.0"), None).await?;
//! println!("{}", changelog.to_markdown(&config.changelog));
//! # Ok(())
//! # }
//! ```

use crate::changelog::version_detection::{VersionTag, find_previous_version, parse_version_tag};
use crate::changelog::{Changelog, ChangelogCollector, ChangelogMetadata};
use crate::config::ChangelogConfig;
use crate::error::{ChangelogError, ChangelogResult};
use crate::types::VersionBump;
use chrono::Utc;
use std::path::{Path, PathBuf};
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

        // TODO: will be implemented in future story
        // Attempt to detect from git remote
        // This would call git_repo methods to get remote URLs
        Ok(None)
    }

    /// Detects the previous version tag for a package from Git tags.
    ///
    /// This method searches through Git tags to find the most recent version
    /// that is less than the current version. It supports both monorepo
    /// (per-package tags) and single-package (root tags) scenarios based on
    /// the configuration.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name for monorepo scenarios
    /// * `current_version` - The current version to find the previous version for
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(VersionTag))` if a previous version is found,
    /// `Ok(None)` if this is the first version (no previous tags),
    /// or an error if Git operations fail or the version is invalid.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - Git tag retrieval fails
    /// - The current version string is invalid
    /// - Git repository operations fail
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
    /// // Detect previous version for a package
    /// let previous = generator.detect_previous_version(Some("mypackage"), "2.0.0").await?;
    /// if let Some(tag) = previous {
    ///     println!("Previous version: {}", tag.version());
    ///     println!("Previous tag: {}", tag.tag_name());
    /// } else {
    ///     println!("This is the first release");
    /// }
    ///
    /// // Detect previous version for root package
    /// let previous = generator.detect_previous_version(None, "1.5.0").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_previous_version(
        &self,
        package_name: Option<&str>,
        current_version: &str,
    ) -> ChangelogResult<Option<VersionTag>> {
        // Get all tags from Git repository
        let git_tags = self.git_repo.get_remote_or_local_tags(Some(true)).map_err(|e| {
            ChangelogError::GitError {
                operation: "get tags".to_string(),
                reason: e.as_ref().to_string(),
            }
        })?;

        // Extract tag names
        let tag_names: Vec<String> = git_tags.iter().map(|t| t.tag.clone()).collect();

        // Determine which format to use based on package_name
        let format = if package_name.is_some() {
            &self.config.version_tag_format
        } else {
            &self.config.root_tag_format
        };

        // Find previous version
        find_previous_version(&tag_names, current_version, package_name, format)
    }

    /// Parses a Git tag string into a version tag structure.
    ///
    /// This method attempts to parse a tag according to the configured format
    /// (either monorepo or root format). It's useful for validating or extracting
    /// version information from tag strings.
    ///
    /// # Arguments
    ///
    /// * `tag` - The Git tag string to parse
    /// * `package_name` - Optional package name for monorepo tags
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(VersionTag))` if the tag matches the expected format,
    /// `Ok(None)` if the tag doesn't match the format,
    /// or an error if the operation fails.
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
    /// // Parse monorepo tag
    /// let tag = generator.parse_version_tag("mypackage@1.0.0", Some("mypackage"))?;
    /// if let Some(version_tag) = tag {
    ///     println!("Version: {}", version_tag.version());
    /// }
    ///
    /// // Parse root tag
    /// let tag = generator.parse_version_tag("v1.0.0", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_version_tag(
        &self,
        tag: &str,
        package_name: Option<&str>,
    ) -> ChangelogResult<Option<VersionTag>> {
        // Determine which format to use
        let format = if package_name.is_some() {
            &self.config.version_tag_format
        } else {
            &self.config.root_tag_format
        };

        Ok(parse_version_tag(tag, package_name, format))
    }

    /// Gets all version tags from the Git repository.
    ///
    /// This method retrieves all tags from Git and filters them to only
    /// include valid version tags according to the configured format.
    /// Optionally filters by package name for monorepo scenarios.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name to filter monorepo tags
    ///
    /// # Returns
    ///
    /// Returns a vector of `VersionTag` instances, sorted by version (newest first).
    ///
    /// # Errors
    ///
    /// This method returns an error if Git operations fail.
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
    /// // Get all version tags for a package
    /// let tags = generator.get_version_tags(Some("mypackage")).await?;
    /// for tag in tags {
    ///     println!("Tag: {} -> Version: {}", tag.tag_name(), tag.version());
    /// }
    ///
    /// // Get all root version tags
    /// let tags = generator.get_version_tags(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_version_tags(
        &self,
        package_name: Option<&str>,
    ) -> ChangelogResult<Vec<VersionTag>> {
        // Get all tags from Git repository
        let git_tags = self.git_repo.get_remote_or_local_tags(Some(true)).map_err(|e| {
            ChangelogError::GitError {
                operation: "get tags".to_string(),
                reason: e.as_ref().to_string(),
            }
        })?;

        // Extract tag names
        let tag_names: Vec<String> = git_tags.iter().map(|t| t.tag.clone()).collect();

        // Determine which format to use
        let format = if package_name.is_some() {
            &self.config.version_tag_format
        } else {
            &self.config.root_tag_format
        };

        // Parse and filter tags
        let mut version_tags: Vec<VersionTag> = tag_names
            .iter()
            .filter_map(|tag| parse_version_tag(tag, package_name, format))
            .collect();

        // Sort by version (newest first)
        version_tags.sort_by(|a, b| b.cmp(a));

        Ok(version_tags)
    }

    /// Generates a changelog for a specific version.
    ///
    /// This method collects commits between the previous version and the current version,
    /// parses them into changelog entries, groups them by section type, and creates
    /// a complete `Changelog` structure.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name for monorepo scenarios
    /// * `version` - The version to generate the changelog for
    /// * `previous_version` - Optional previous version (auto-detected if None)
    /// * `relative_path` - Optional path filter for monorepo packages
    ///
    /// # Returns
    ///
    /// A `Changelog` instance containing all changes for the specified version.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git operations fail
    /// - Version detection fails
    /// - Commit retrieval fails
    /// - No commits are found between versions
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
    /// // Generate for specific package with auto-detected previous version
    /// let changelog = generator.generate_for_version(
    ///     Some("my-package"),
    ///     "2.0.0",
    ///     None,
    ///     Some("packages/my-package")
    /// ).await?;
    ///
    /// // Generate for root with explicit previous version
    /// let changelog = generator.generate_for_version(
    ///     None,
    ///     "1.5.0",
    ///     Some("1.4.0"),
    ///     None
    /// ).await?;
    ///
    /// println!("Generated changelog with {} entries", changelog.entry_count());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_for_version(
        &self,
        package_name: Option<&str>,
        version: &str,
        previous_version: Option<&str>,
        relative_path: Option<&str>,
    ) -> ChangelogResult<Changelog> {
        // Determine previous version if not provided
        let prev_version = if let Some(prev) = previous_version {
            Some(prev.to_string())
        } else {
            self.detect_previous_version(package_name, version)
                .await?
                .map(|tag| tag.version().to_string())
        };

        // Determine Git references
        let (from_ref, to_ref) =
            self.build_git_refs(package_name, prev_version.as_deref(), version)?;

        // Collect commits using the collector
        let collector = ChangelogCollector::new(&self.git_repo, &self.config);
        let sections =
            collector.collect_between_versions(&from_ref, &to_ref, relative_path).await?;

        // Build changelog metadata
        let metadata = self.build_metadata(
            package_name,
            version,
            prev_version.as_deref(),
            &from_ref,
            &to_ref,
            &sections,
        )?;

        // Create changelog
        let mut changelog =
            Changelog::new(package_name, version, prev_version.as_deref(), Utc::now());

        // Add sections
        for section in sections {
            changelog.add_section(section);
        }

        changelog.metadata = metadata;

        Ok(changelog)
    }

    /// Builds Git references for changelog generation.
    ///
    /// Creates the from and to references based on the configured tag format.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name
    /// * `previous_version` - Optional previous version
    /// * `current_version` - Current version
    ///
    /// # Returns
    ///
    /// A tuple of (from_ref, to_ref) strings.
    ///
    /// # Errors
    ///
    /// Returns an error if no previous version is available.
    fn build_git_refs(
        &self,
        package_name: Option<&str>,
        previous_version: Option<&str>,
        current_version: &str,
    ) -> ChangelogResult<(String, String)> {
        let format = if package_name.is_some() {
            &self.config.version_tag_format
        } else {
            &self.config.root_tag_format
        };

        // Build to_ref (current version tag)
        let to_ref = if let Some(pkg_name) = package_name {
            format.replace("{name}", pkg_name).replace("{version}", current_version)
        } else {
            format.replace("{version}", current_version)
        };

        // Build from_ref (previous version tag or HEAD)
        let from_ref = if let Some(prev_version) = previous_version {
            if let Some(pkg_name) = package_name {
                format.replace("{name}", pkg_name).replace("{version}", prev_version)
            } else {
                format.replace("{version}", prev_version)
            }
        } else {
            // No previous version - use first commit
            return Err(ChangelogError::VersionNotFound {
                reason: "No previous version found for changelog generation".to_string(),
            });
        };

        Ok((from_ref, to_ref))
    }

    /// Builds changelog metadata from collected data.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name
    /// * `version` - Current version
    /// * `previous_version` - Optional previous version
    /// * `from_ref` - Starting Git reference
    /// * `to_ref` - Ending Git reference
    /// * `sections` - Collected changelog sections
    ///
    /// # Returns
    ///
    /// A `ChangelogMetadata` instance.
    fn build_metadata(
        &self,
        _package_name: Option<&str>,
        _version: &str,
        previous_version: Option<&str>,
        from_ref: &str,
        to_ref: &str,
        sections: &[crate::changelog::ChangelogSection],
    ) -> ChangelogResult<ChangelogMetadata> {
        // Calculate total commits
        let total_commits: usize = sections.iter().map(|s| s.entries.len()).sum();

        // Build commit range
        let commit_range = if previous_version.is_some() {
            Some(format!("{}..{}", from_ref, to_ref))
        } else {
            None
        };

        // Determine bump type based on sections
        let bump_type = self.infer_bump_type(sections);

        // Get repository URL
        let repository_url = self.get_repository_url()?;

        Ok(ChangelogMetadata {
            tag: Some(to_ref.to_string()),
            commit_range,
            total_commits,
            repository_url,
            bump_type: Some(bump_type),
        })
    }

    /// Infers the version bump type from changelog sections.
    ///
    /// # Arguments
    ///
    /// * `sections` - Changelog sections
    ///
    /// # Returns
    ///
    /// The inferred `VersionBump`.
    fn infer_bump_type(&self, sections: &[crate::changelog::ChangelogSection]) -> VersionBump {
        use crate::changelog::SectionType;

        // Breaking changes = major bump
        for section in sections {
            if section.section_type == SectionType::Breaking && !section.is_empty() {
                return VersionBump::Major;
            }
        }

        // Features = minor bump
        for section in sections {
            if section.section_type == SectionType::Features && !section.is_empty() {
                return VersionBump::Minor;
            }
        }

        // Any other changes = patch bump
        if sections.iter().any(|s| !s.is_empty()) {
            return VersionBump::Patch;
        }

        // No changes
        VersionBump::None
    }

    /// Updates or creates a CHANGELOG.md file with new changelog content.
    ///
    /// This method either creates a new changelog file with a header or updates an existing
    /// one by prepending the new version section. The operation can be performed in dry-run
    /// mode, which returns the content without writing to the file system.
    ///
    /// # Arguments
    ///
    /// * `package_path` - The path to the package directory
    /// * `changelog` - The changelog data to add
    /// * `dry_run` - If `true`, returns content without writing to disk
    ///
    /// # Returns
    ///
    /// Returns the final changelog content (whether written or not).
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - File system operations fail
    /// - The existing changelog cannot be parsed
    /// - The path is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::{Changelog, ChangelogGenerator};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    /// use chrono::Utc;
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
    /// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
    /// let package_path = PathBuf::from("packages/my-package");
    ///
    /// // Dry run - preview without writing
    /// let content = generator.update_changelog(&package_path, &changelog, true).await?;
    /// println!("Would write:\n{}", content);
    ///
    /// // Actually write the changelog
    /// generator.update_changelog(&package_path, &changelog, false).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_changelog(
        &self,
        package_path: &Path,
        changelog: &Changelog,
        dry_run: bool,
    ) -> ChangelogResult<String> {
        let changelog_path = package_path.join(&self.config.filename);

        // Render new content
        let new_section = changelog.to_markdown(&self.config);

        // Read existing changelog (if exists)
        let existing_content = if self.fs.exists(&changelog_path).await {
            self.fs.read_file_string(&changelog_path).await.map_err(|e| {
                ChangelogError::FileSystemError {
                    path: changelog_path.clone(),
                    reason: e.as_ref().to_string(),
                }
            })?
        } else {
            // Create header for new changelog
            let mut header = self.config.template.header.clone();
            if !header.is_empty() && !header.ends_with('\n') {
                header.push('\n');
            }
            header.push('\n');
            header
        };

        // Prepend new section
        let updated_content = self.prepend_changelog(&existing_content, &new_section);

        // Write if not dry-run
        if !dry_run {
            // Ensure parent directory exists
            if let Some(parent) = changelog_path.parent() {
                self.fs.create_dir_all(parent).await.map_err(|e| {
                    ChangelogError::FileSystemError {
                        path: parent.to_path_buf(),
                        reason: e.as_ref().to_string(),
                    }
                })?;
            }

            self.fs.write_file_string(&changelog_path, &updated_content).await.map_err(|e| {
                ChangelogError::UpdateFailed {
                    path: changelog_path.clone(),
                    reason: e.as_ref().to_string(),
                }
            })?;
        }

        Ok(updated_content)
    }

    /// Parses an existing CHANGELOG.md file.
    ///
    /// This method reads and parses an existing changelog file, extracting version
    /// information and content. It's useful for querying existing changelogs or
    /// checking if a version already exists before updating.
    ///
    /// # Arguments
    ///
    /// * `package_path` - The path to the package directory
    ///
    /// # Returns
    ///
    /// Returns a `ParsedChangelog` containing the structured changelog data.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - The changelog file doesn't exist
    /// - The file cannot be read
    /// - The changelog format is invalid
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
    /// let package_path = PathBuf::from("packages/my-package");
    /// let parsed = generator.parse_changelog(&package_path).await?;
    ///
    /// println!("Found {} versions", parsed.versions.len());
    /// if let Some(latest) = parsed.latest_version() {
    ///     println!("Latest version: {}", latest.version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn parse_changelog(
        &self,
        package_path: &Path,
    ) -> ChangelogResult<crate::changelog::parser::ParsedChangelog> {
        use crate::changelog::parser::ChangelogParser;

        let changelog_path = package_path.join(&self.config.filename);

        // Check if file exists
        if !self.fs.exists(&changelog_path).await {
            return Err(ChangelogError::NotFound { path: changelog_path });
        }

        // Read the file
        let content = self.fs.read_file_string(&changelog_path).await.map_err(|e| {
            ChangelogError::FileSystemError {
                path: changelog_path.clone(),
                reason: e.as_ref().to_string(),
            }
        })?;

        // Parse the content
        let parser = ChangelogParser::new();
        parser.parse(&content)
    }

    /// Prepends a new changelog section to existing content.
    ///
    /// This method intelligently inserts the new version section after the header
    /// but before any existing versions, preserving the overall structure of the
    /// changelog.
    ///
    /// The insertion logic:
    /// 1. Finds the first version header (## )
    /// 2. Inserts the new section before it
    /// 3. If no versions exist, appends to the header
    ///
    /// # Arguments
    ///
    /// * `existing` - The existing changelog content
    /// * `new_section` - The new version section to prepend
    ///
    /// # Returns
    ///
    /// The updated changelog content with the new section prepended.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogGenerator;
    ///
    /// # fn example(generator: &ChangelogGenerator) {
    /// let existing = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Old feature";
    /// let new_section = "## [2.0.0] - 2024-02-01\n\n### Features\n- New feature\n";
    ///
    /// let updated = generator.prepend_changelog(existing, new_section);
    /// assert!(updated.contains("## [2.0.0]"));
    /// assert!(updated.contains("## [1.0.0]"));
    /// # }
    /// ```
    pub(crate) fn prepend_changelog(&self, existing: &str, new_section: &str) -> String {
        let lines: Vec<&str> = existing.lines().collect();

        // Find the position of the first version header (starts with "## ")
        let insert_pos =
            lines.iter().position(|line| line.starts_with("## ")).unwrap_or(lines.len());

        let mut result = String::new();

        // Add lines before insert position (header)
        for (i, line) in lines.iter().enumerate().take(insert_pos) {
            result.push_str(line);
            result.push('\n');

            // Add extra newline after header if this is the last header line
            if i == insert_pos - 1 && insert_pos < lines.len() {
                result.push('\n');
            }
        }

        // If we're at the end (no existing versions), ensure proper spacing
        if insert_pos == lines.len() && !result.is_empty() && !result.ends_with("\n\n") {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
        }

        // Add new section
        result.push_str(new_section);

        // Ensure proper spacing before existing versions
        if insert_pos < lines.len() && !new_section.ends_with("\n\n") {
            if !new_section.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
        }

        // Add remaining lines (existing versions)
        for line in lines.iter().skip(insert_pos) {
            result.push_str(line);
            result.push('\n');
        }

        // Remove trailing newlines beyond two
        while result.ends_with("\n\n\n") {
            result.pop();
        }

        result
    }

    /// Generates changelogs from a changeset and version resolution.
    ///
    /// This method takes a changeset with commits and a version resolution with
    /// package updates, then generates changelog entries for each affected package.
    /// It respects the configured monorepo mode and generates changelogs accordingly.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset containing commits and package information
    /// * `version_resolution` - Resolved versions for all affected packages
    ///
    /// # Returns
    ///
    /// A vector of `GeneratedChangelog` instances, one for each package that
    /// needs a changelog update.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - Monorepo detection fails
    /// - Package information cannot be loaded
    /// - Commit collection fails
    /// - Changelog generation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogGenerator;
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    /// use sublime_pkg_tools::version::VersionResolver;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = ChangelogGenerator::new(workspace_root, git_repo, fs, config).await?;
    /// let changeset_manager = ChangesetManager::new(changeset_storage, git_repo).await?;
    /// let version_resolver = VersionResolver::new(&workspace_root, &config).await?;
    ///
    /// // Load changeset
    /// let changeset = changeset_manager.load("feature-branch").await?;
    ///
    /// // Resolve versions
    /// let resolution = version_resolver.resolve_versions(&changeset).await?;
    ///
    /// // Generate changelogs
    /// let changelogs = generator.generate_from_changeset(&changeset, &resolution).await?;
    ///
    /// for generated in &changelogs {
    ///     println!("Generated changelog for: {:?}", generated.package_name);
    ///     generated.write(&fs).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_from_changeset(
        &self,
        changeset: &crate::types::Changeset,
        version_resolution: &crate::version::VersionResolution,
    ) -> ChangelogResult<Vec<crate::changelog::GeneratedChangelog>> {
        use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

        // Check if this is a monorepo
        let monorepo_detector = MonorepoDetector::with_filesystem(self.fs.clone());
        let is_monorepo = monorepo_detector
            .is_monorepo_root(&self.workspace_root)
            .await
            .map_err(|e| ChangelogError::FileSystemError {
                path: self.workspace_root.clone(),
                reason: e.as_ref().to_string(),
            })?
            .is_some();

        let mut generated_changelogs = Vec::new();

        // Determine which packages need changelogs based on monorepo_mode
        match self.config.monorepo_mode {
            crate::config::MonorepoMode::PerPackage => {
                // Generate changelog for each updated package
                if is_monorepo {
                    generated_changelogs.extend(
                        self.generate_per_package_changelogs(changeset, version_resolution).await?,
                    );
                } else {
                    // Single package - generate root changelog
                    generated_changelogs
                        .extend(self.generate_root_changelog(changeset, version_resolution).await?);
                }
            }
            crate::config::MonorepoMode::Root => {
                // Generate only root changelog
                generated_changelogs
                    .extend(self.generate_root_changelog(changeset, version_resolution).await?);
            }
            crate::config::MonorepoMode::Both => {
                // Generate both per-package and root changelogs
                if is_monorepo {
                    generated_changelogs.extend(
                        self.generate_per_package_changelogs(changeset, version_resolution).await?,
                    );
                }
                generated_changelogs
                    .extend(self.generate_root_changelog(changeset, version_resolution).await?);
            }
        }

        Ok(generated_changelogs)
    }

    /// Generates changelogs for each package in a monorepo.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset with commits
    /// * `version_resolution` - Resolved versions for packages
    ///
    /// # Returns
    ///
    /// A vector of generated changelogs, one per package.
    async fn generate_per_package_changelogs(
        &self,
        _changeset: &crate::types::Changeset,
        version_resolution: &crate::version::VersionResolution,
    ) -> ChangelogResult<Vec<crate::changelog::GeneratedChangelog>> {
        use crate::changelog::GeneratedChangelog;
        use sublime_standard_tools::filesystem::AsyncFileSystem;

        let mut changelogs = Vec::new();

        for update in &version_resolution.updates {
            // Load package.json to get package name
            let package_json_path = update.path.join("package.json");
            let package_json_content =
                self.fs.read_file_string(&package_json_path).await.map_err(|_e| {
                    ChangelogError::PackageNotFound { package: update.name.clone() }
                })?;

            let package_json: package_json::PackageJson =
                serde_json::from_str(&package_json_content).map_err(|e| {
                    ChangelogError::FileSystemError {
                        path: package_json_path.clone(),
                        reason: format!("Failed to parse package.json: {}", e),
                    }
                })?;

            let package_name = package_json.name.clone();

            // Determine relative path for commit filtering
            let relative_path = update
                .path
                .strip_prefix(&self.workspace_root)
                .ok()
                .and_then(|p| p.to_str())
                .map(String::from);

            // Determine previous version for Git refs
            let previous_version = if let Ok(prev_tag) = self
                .detect_previous_version(Some(&package_name), &update.next_version.to_string())
                .await
            {
                prev_tag.map(|t| t.version().to_string())
            } else {
                None
            };

            // Build Git refs and collect commits
            let (sections, from_ref, to_ref) = if let Some(ref prev_version) = previous_version {
                let (from_ref, to_ref) = self.build_git_refs(
                    Some(&package_name),
                    Some(prev_version.as_str()),
                    &update.next_version.to_string(),
                )?;

                // Collect commits for this package
                let collector = ChangelogCollector::new(&self.git_repo, &self.config);
                let sections = collector
                    .collect_between_versions(&from_ref, &to_ref, relative_path.as_deref())
                    .await?;

                (sections, from_ref, to_ref)
            } else {
                // No previous version - try to collect all commits using git log
                // If that fails (empty repo), return empty sections
                let collector = ChangelogCollector::new(&self.git_repo, &self.config);

                // Try to get commits since the beginning
                // Use get_commits_since with None to get all commits
                let commits_result = self.git_repo.get_commits_since(None, &relative_path);

                let sections = if let Ok(commits) = commits_result {
                    collector.process_commits(commits)?
                } else {
                    // Empty repo or no commits accessible
                    Vec::new()
                };

                // Use empty refs for metadata when no previous version
                (sections, String::new(), "HEAD".to_string())
            };

            // Build metadata
            let metadata = self.build_metadata(
                Some(&package_name),
                &update.next_version.to_string(),
                previous_version.as_deref(),
                &from_ref,
                &to_ref,
                &sections,
            )?;

            // Create changelog
            let mut changelog = crate::changelog::Changelog::new(
                Some(&package_name),
                &update.next_version.to_string(),
                previous_version.as_deref(),
                chrono::Utc::now(),
            );

            for section in sections {
                changelog.add_section(section);
            }

            changelog.metadata = metadata;

            // Render to markdown
            let content = changelog.to_markdown(&self.config);

            // Determine changelog path
            let changelog_path = update.path.join(&self.config.filename);
            let existing = self.fs.exists(&changelog_path).await;

            changelogs.push(GeneratedChangelog::new(
                Some(package_name),
                update.path.clone(),
                changelog,
                content,
                existing,
                changelog_path,
            ));
        }

        Ok(changelogs)
    }

    /// Generates a root changelog for the entire workspace.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset with commits
    /// * `version_resolution` - Resolved versions for packages
    ///
    /// # Returns
    ///
    /// A vector containing a single root changelog.
    async fn generate_root_changelog(
        &self,
        _changeset: &crate::types::Changeset,
        version_resolution: &crate::version::VersionResolution,
    ) -> ChangelogResult<Vec<crate::changelog::GeneratedChangelog>> {
        use crate::changelog::GeneratedChangelog;
        use sublime_standard_tools::filesystem::AsyncFileSystem;

        // For root changelog, we need to determine a version
        // Use the highest version from the resolution, or construct one
        let version = if let Some(first_update) = version_resolution.updates.first() {
            first_update.next_version.to_string()
        } else {
            // No updates - shouldn't happen, but handle gracefully
            return Ok(Vec::new());
        };

        // Determine previous version for Git refs
        let previous_version =
            if let Ok(prev_tag) = self.detect_previous_version(None, &version).await {
                prev_tag.map(|t| t.version().to_string())
            } else {
                None
            };

        // Build Git refs and collect commits
        let collector = ChangelogCollector::new(&self.git_repo, &self.config);
        let (sections, from_ref, to_ref) = if let Some(ref prev_version) = previous_version {
            let (from_ref, to_ref) =
                self.build_git_refs(None, Some(prev_version.as_str()), &version)?;
            let sections = collector.collect_between_versions(&from_ref, &to_ref, None).await?;
            (sections, from_ref, to_ref)
        } else {
            // No previous version - try to collect all commits
            let commits_result = self.git_repo.get_commits_since(None, &None);

            let sections = if let Ok(commits) = commits_result {
                collector.process_commits(commits)?
            } else {
                // Empty repo or no commits accessible
                Vec::new()
            };

            // Use empty refs for metadata when no previous version
            (sections, String::new(), "HEAD".to_string())
        };

        // Build metadata
        let metadata = self.build_metadata(
            None,
            &version,
            previous_version.as_deref(),
            &from_ref,
            &to_ref,
            &sections,
        )?;

        // Create changelog
        let mut changelog = crate::changelog::Changelog::new(
            None,
            &version,
            previous_version.as_deref(),
            chrono::Utc::now(),
        );

        for section in sections {
            changelog.add_section(section);
        }

        changelog.metadata = metadata;

        // Render to markdown
        let content = changelog.to_markdown(&self.config);

        // Determine changelog path (at workspace root)
        let changelog_path = self.workspace_root.join(&self.config.filename);
        let existing = self.fs.exists(&changelog_path).await;

        Ok(vec![GeneratedChangelog::new(
            None,
            self.workspace_root.clone(),
            changelog,
            content,
            existing,
            changelog_path,
        )])
    }
}
