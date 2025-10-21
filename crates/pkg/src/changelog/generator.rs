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

use crate::changelog::version_detection::{find_previous_version, parse_version_tag, VersionTag};
use crate::changelog::{Changelog, ChangelogCollector, ChangelogMetadata};
use crate::config::ChangelogConfig;
use crate::error::{ChangelogError, ChangelogResult};
use crate::types::VersionBump;
use chrono::Utc;
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
}
