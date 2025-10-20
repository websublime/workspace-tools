//! Changeset manager for high-level changeset operations.
//!
//! **What**: Provides a high-level API for creating, loading, updating, and managing changesets,
//! orchestrating storage operations and validation.
//!
//! **How**: The `ChangesetManager` wraps a storage implementation and provides methods for all
//! changeset lifecycle operations including CRUD operations, validation, and future git integration.
//! It ensures changesets are valid before saving and provides clear error messages.
//!
//! **Why**: To provide a clean, ergonomic API for changeset management that handles validation,
//! error handling, and storage coordination, making it easy to work with changesets throughout
//! the application.

use crate::config::ChangesetConfig;
use crate::error::{ChangesetError, ChangesetResult};
use crate::types::{Changeset, VersionBump};
use std::path::PathBuf;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;

use super::storage::{ChangesetStorage, FileBasedChangesetStorage};

/// Manager for high-level changeset operations.
///
/// The `ChangesetManager` provides a comprehensive API for creating, reading, updating,
/// and deleting changesets. It orchestrates validation, storage operations, and will
/// integrate with Git for automatic package detection in future stories.
///
/// # Type Parameters
///
/// * `S` - The storage implementation (must implement `ChangesetStorage`)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changeset::ChangesetManager;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let fs = FileSystemManager::new();
/// let config = PackageToolsConfig::default();
///
/// let manager = ChangesetManager::new(workspace_root, fs, config).await?;
///
/// // Create a new changeset
/// let changeset = manager.create(
///     "feature-branch",
///     sublime_pkg_tools::types::VersionBump::Minor,
///     vec!["production".to_string()]
/// ).await?;
///
/// // Load and modify
/// let mut changeset = manager.load("feature-branch").await?;
/// changeset.add_package("my-package");
/// manager.update(&changeset).await?;
///
/// // List all pending changesets
/// let pending = manager.list_pending().await?;
/// println!("Found {} pending changesets", pending.len());
///
/// // Delete when no longer needed
/// manager.delete("feature-branch").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ChangesetManager<S: ChangesetStorage> {
    /// The storage implementation for persisting changesets.
    storage: S,
    /// Optional Git repository for commit integration (will be used in Story 6.4).
    git_repo: Option<Repo>,
    /// Configuration for changeset validation and behavior.
    config: ChangesetConfig,
}

impl ChangesetManager<FileBasedChangesetStorage<FileSystemManager>> {
    /// Creates a new `ChangesetManager` with file-based storage.
    ///
    /// This is a convenience constructor that creates a `FileBasedChangesetStorage`
    /// using the provided filesystem and configuration paths.
    ///
    /// # Parameters
    ///
    /// * `workspace_root` - The root directory of the workspace
    /// * `fs` - The filesystem implementation to use
    /// * `config` - The package tools configuration
    ///
    /// # Returns
    ///
    /// A new `ChangesetManager` instance ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Git repository cannot be opened (non-fatal, git_repo will be None)
    /// - Storage initialization fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ChangesetManager::new(
    ///     PathBuf::from("."),
    ///     FileSystemManager::new(),
    ///     PackageToolsConfig::default()
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        workspace_root: impl Into<PathBuf>,
        fs: FileSystemManager,
        config: crate::config::PackageToolsConfig,
    ) -> ChangesetResult<Self> {
        let workspace_root = workspace_root.into();
        let changeset_config = config.changeset;

        // Create storage with configured paths
        let storage = FileBasedChangesetStorage::new(
            workspace_root.clone(),
            changeset_config.path.clone(),
            changeset_config.history_path.clone(),
            fs,
        );

        // Attempt to open Git repository (non-fatal if it fails)
        let git_repo = Repo::open(workspace_root.to_string_lossy().as_ref()).ok();

        Ok(Self { storage, git_repo, config: changeset_config })
    }
}

impl<S: ChangesetStorage> ChangesetManager<S> {
    /// Creates a new `ChangesetManager` with a custom storage implementation.
    ///
    /// This constructor allows full control over the storage implementation,
    /// Git repository, and configuration. It's primarily useful for testing
    /// or when using custom storage backends.
    ///
    /// # Parameters
    ///
    /// * `storage` - The storage implementation to use
    /// * `git_repo` - Optional Git repository for commit integration
    /// * `config` - Changeset configuration for validation
    ///
    /// # Returns
    ///
    /// A new `ChangesetManager` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changeset::{ChangesetManager, FileBasedChangesetStorage};
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # fn example() {
    /// let storage = FileBasedChangesetStorage::new(
    ///     PathBuf::from("."),
    ///     ".changesets",
    ///     ".changesets/history",
    ///     FileSystemManager::new()
    /// );
    ///
    /// let manager = ChangesetManager::with_storage(
    ///     storage,
    ///     None,
    ///     ChangesetConfig::default()
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn with_storage(storage: S, git_repo: Option<Repo>, config: ChangesetConfig) -> Self {
        Self { storage, git_repo, config }
    }

    /// Creates a new changeset.
    ///
    /// This method creates a new changeset with the specified branch name, version bump type,
    /// and target environments. It validates the branch name, checks for existing changesets,
    /// validates the environments, and saves the changeset to storage.
    ///
    /// # Parameters
    ///
    /// * `branch` - The branch name (must be non-empty and valid)
    /// * `bump` - The version bump type (Major, Minor, Patch, or None)
    /// * `environments` - The target environments for this changeset
    ///
    /// # Returns
    ///
    /// The newly created `Changeset` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The branch name is empty or invalid
    /// - A changeset already exists for this branch
    /// - The environments are invalid (not in available environments)
    /// - Storage operation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    /// use sublime_pkg_tools::types::VersionBump;
    ///
    /// # async fn example(manager: ChangesetManager<impl ChangesetStorage>) -> Result<(), Box<dyn std::error::Error>> {
    /// let changeset = manager.create(
    ///     "feature/new-api",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string(), "staging".to_string()]
    /// ).await?;
    ///
    /// println!("Created changeset for branch: {}", changeset.branch);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(
        &self,
        branch: impl Into<String>,
        bump: VersionBump,
        environments: Vec<String>,
    ) -> ChangesetResult<Changeset> {
        let branch_name = branch.into();

        // Validate branch name
        if branch_name.is_empty() {
            return Err(ChangesetError::InvalidBranch {
                branch: branch_name,
                reason: "Branch name cannot be empty".to_string(),
            });
        }

        // Check if changeset already exists
        if self.storage.exists(&branch_name).await? {
            let path = std::env::current_dir()
                .unwrap_or_default()
                .join(&self.config.path)
                .join(&branch_name);
            return Err(ChangesetError::AlreadyExists { branch: branch_name, path });
        }

        // Validate environments
        self.validate_environments(&environments)?;

        // Create new changeset
        let changeset = Changeset::new(branch_name, bump, environments);

        // Note: We don't validate for packages here because changesets start empty
        // and packages are added later through update operations

        // Save to storage
        self.storage.save(&changeset).await?;

        Ok(changeset)
    }

    /// Loads an existing changeset by branch name.
    ///
    /// # Parameters
    ///
    /// * `branch` - The branch name to load
    ///
    /// # Returns
    ///
    /// The loaded `Changeset` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The changeset does not exist for the given branch
    /// - Storage operation fails
    /// - Deserialization fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # async fn example(manager: ChangesetManager<impl ChangesetStorage>) -> Result<(), Box<dyn std::error::Error>> {
    /// let changeset = manager.load("feature/new-api").await?;
    /// println!("Loaded changeset with {} packages", changeset.packages.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
        self.storage.load(branch).await
    }

    /// Updates an existing changeset.
    ///
    /// This method validates the changeset, updates its `updated_at` timestamp,
    /// and saves it back to storage. The changeset must already exist.
    ///
    /// # Parameters
    ///
    /// * `changeset` - The changeset to update
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Validation fails
    /// - The changeset is empty (no packages)
    /// - Storage operation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # async fn example(manager: ChangesetManager<impl ChangesetStorage>) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut changeset = manager.load("feature/new-api").await?;
    ///
    /// // Modify the changeset
    /// changeset.add_package("new-package");
    /// changeset.add_commit("abc123def");
    ///
    /// // Save changes
    /// manager.update(&changeset).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, changeset: &Changeset) -> ChangesetResult<()> {
        // Validate the changeset
        let available_envs: Vec<&str> =
            self.config.available_environments.iter().map(|s| s.as_str()).collect();
        changeset.validate(&available_envs)?;

        // Create a mutable copy to touch it
        let mut updated_changeset = changeset.clone();
        updated_changeset.touch();

        // Save to storage
        self.storage.save(&updated_changeset).await?;

        Ok(())
    }

    /// Deletes a pending changeset.
    ///
    /// This permanently removes the changeset from storage. Use with caution as
    /// this operation cannot be undone.
    ///
    /// # Parameters
    ///
    /// * `branch` - The branch name of the changeset to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The changeset does not exist
    /// - Storage operation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # async fn example(manager: ChangesetManager<impl ChangesetStorage>) -> Result<(), Box<dyn std::error::Error>> {
    /// manager.delete("feature/abandoned").await?;
    /// println!("Changeset deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, branch: &str) -> ChangesetResult<()> {
        self.storage.delete(branch).await
    }

    /// Lists all pending changesets.
    ///
    /// This method loads all changesets that have not yet been archived.
    ///
    /// # Returns
    ///
    /// A vector of all pending `Changeset` instances.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Storage operation fails
    /// - Any changeset fails to load or deserialize
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # async fn example(manager: ChangesetManager<impl ChangesetStorage>) -> Result<(), Box<dyn std::error::Error>> {
    /// let pending = manager.list_pending().await?;
    /// for changeset in pending {
    ///     println!("Branch: {}, Packages: {}", changeset.branch, changeset.packages.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_pending(&self) -> ChangesetResult<Vec<Changeset>> {
        self.storage.list_pending().await
    }

    /// Validates that all environments are in the available environments list.
    ///
    /// # Parameters
    ///
    /// * `environments` - The environments to validate
    ///
    /// # Errors
    ///
    /// Returns an error if any environment is not in the available environments list.
    fn validate_environments(&self, environments: &[String]) -> ChangesetResult<()> {
        for env in environments {
            if !self.config.available_environments.contains(env) {
                return Err(ChangesetError::InvalidEnvironment {
                    environment: env.clone(),
                    available: self.config.available_environments.clone(),
                });
            }
        }
        Ok(())
    }

    /// Returns a reference to the storage implementation.
    ///
    /// This method provides access to the underlying storage for advanced use cases.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # fn example(manager: ChangesetManager<impl ChangesetStorage>) {
    /// let storage = manager.storage();
    /// // Use storage directly if needed
    /// # }
    /// ```
    #[must_use]
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Returns a reference to the Git repository, if available.
    ///
    /// # Returns
    ///
    /// `Some(&Repo)` if a Git repository is available, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # fn example(manager: ChangesetManager<impl ChangesetStorage>) {
    /// if let Some(repo) = manager.git_repo() {
    ///     println!("Git repository available at: {:?}", repo.get_repo_path());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn git_repo(&self) -> Option<&Repo> {
        self.git_repo.as_ref()
    }

    /// Returns a reference to the changeset configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetManager;
    /// # fn example(manager: ChangesetManager<impl ChangesetStorage>) {
    /// let config = manager.config();
    /// println!("Available environments: {:?}", config.available_environments);
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &ChangesetConfig {
        &self.config
    }
}
