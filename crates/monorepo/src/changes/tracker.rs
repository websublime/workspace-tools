//! Change tracking system.
//!
//! This module provides the `ChangeTracker` type, which is the central component
//! for detecting, recording, and managing changes across packages in a monorepo.
//! It integrates with Git to automatically detect changes between commits and
//! provides utilities to map files to the packages they belong to.

use crate::{
    Change, ChangeError, ChangeId, ChangeResult, ChangeStore, ChangeType, Changeset, Workspace,
};
use log::{debug, info};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Represents the scope of a change in the repository
///
/// This enum categorizes where in the repository a change occurred,
/// differentiating between changes to specific packages, monorepo
/// infrastructure, or the root level.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::ChangeScope;
///
/// // Change to a specific package
/// let package_change = ChangeScope::Package("ui-components".to_string());
///
/// // Change to monorepo infrastructure
/// let infra_change = ChangeScope::Monorepo;
///
/// // Change at the root level
/// let root_change = ChangeScope::Root;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeScope {
    /// Change specific to a package
    Package(String),

    /// Change affecting the monorepo infrastructure (outside packages but not root)
    Monorepo,

    /// Change at the root level
    Root,
}

/// Change tracking system.
///
/// The `ChangeTracker` provides functionality for:
/// - Detecting changes between Git commits or branches
/// - Mapping files to packages
/// - Recording changes manually or automatically
/// - Managing the lifecycle of changes from creation to release
///
/// It works with a pluggable `ChangeStore` implementation for persistence.
///
/// # Examples
///
/// ```no_run
/// use std::rc::Rc;
/// use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace};
/// use sublime_git_tools::Repo;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create workspace
/// let workspace = Workspace::new(std::path::Path::new(".").to_path_buf(), Default::default(), None)?;
/// let workspace_rc = Rc::new(workspace);
///
/// // Create change store and tracker
/// let store = Box::new(MemoryChangeStore::new());
/// let mut tracker = ChangeTracker::new(workspace_rc, store);
///
/// // Detect changes since last tag
/// let changes = tracker.detect_changes_between("v1.0.0", None)?;
/// println!("Detected {} changes", changes.len());
///
/// // Get unreleased changes
/// let unreleased = tracker.unreleased_changes()?;
/// for (package, changes) in unreleased {
///     println!("Package {} has {} unreleased changes", package, changes.len());
/// }
/// # Ok(())
/// # }
/// ```
pub struct ChangeTracker {
    /// Workspace reference
    workspace: Rc<Workspace>,
    /// Storage for changes
    store: Box<dyn ChangeStore>,
    /// Git configuration
    git_config: GitConfig,
    /// Cache for file to scope mapping
    scope_cache: HashMap<PathBuf, ChangeScope>,
}

/// Git configuration for change tracking.
///
/// Stores Git user information that may be used when creating changes.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GitConfig {
    /// Name for Git commits
    user_name: Option<String>,
    /// Email for Git commits
    user_email: Option<String>,
}

impl ChangeTracker {
    /// Creates a new change tracker.
    ///
    /// # Arguments
    ///
    /// * `workspace` - Reference to the monorepo workspace
    /// * `store` - Change store implementation for persistence
    ///
    /// # Returns
    ///
    /// A new `ChangeTracker` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::rc::Rc;
    /// use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create workspace
    /// let workspace = Workspace::new(std::path::Path::new(".").to_path_buf(), Default::default(), None)?;
    /// let workspace_rc = Rc::new(workspace);
    ///
    /// // Create tracker with memory store
    /// let store = Box::new(MemoryChangeStore::new());
    /// let tracker = ChangeTracker::new(workspace_rc, store);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(workspace: Rc<Workspace>, store: Box<dyn ChangeStore>) -> Self {
        // Default Git configuration
        let git_config = GitConfig { user_name: None, user_email: None };

        Self { workspace, store, git_config, scope_cache: HashMap::new() }
    }

    /// Sets the Git user information.
    ///
    /// # Arguments
    ///
    /// * `name` - Optional Git user name
    /// * `email` - Optional Git user email
    ///
    /// # Returns
    ///
    /// The modified `ChangeTracker` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::rc::Rc;
    /// use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace = Rc::new(Workspace::new(std::path::Path::new(".").to_path_buf(), Default::default(), None)?);
    /// # let store = Box::new(MemoryChangeStore::new());
    /// let tracker = ChangeTracker::new(workspace, store)
    ///     .with_git_user(Some("John Doe"), Some("john@example.com"));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_git_user<S: Into<String>>(mut self, name: Option<S>, email: Option<S>) -> Self {
        self.git_config =
            GitConfig { user_name: name.map(Into::into), user_email: email.map(Into::into) };
        self
    }

    /// Clears the scope cache
    ///
    /// Clears the internal cache that maps file paths to package scopes.
    /// This is useful after file system changes to ensure fresh mapping.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::rc::Rc;
    /// # use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace};
    /// # fn example(mut tracker: ChangeTracker) {
    /// // Clear the cache after file changes
    /// tracker.clear_cache();
    /// # }
    /// ```
    pub fn clear_cache(&mut self) {
        debug!("Clearing file scope cache");
        self.scope_cache.clear();
    }

    /// Maps a file to its change scope
    ///
    /// Determines whether a file belongs to a specific package,
    /// the monorepo infrastructure, or the root level.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to map
    ///
    /// # Returns
    ///
    /// The `ChangeScope` for the file.
    ///
    /// # Errors
    ///
    /// While technically this can return a `ChangeError`, the implementation
    /// currently always returns `Ok`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::rc::Rc;
    /// # use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace, ChangeScope};
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Map a file path to its scope
    /// let scope = tracker.map_file_to_scope("packages/ui/package.json")?;
    ///
    /// // Check the scope type
    /// match scope {
    ///     ChangeScope::Package(name) => println!("File belongs to package: {}", name),
    ///     ChangeScope::Monorepo => println!("File belongs to monorepo infrastructure"),
    ///     ChangeScope::Root => println!("File is at root level"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unnecessary_wraps)]
    pub fn map_file_to_scope(&mut self, file_path: &str) -> ChangeResult<ChangeScope> {
        // Convert input to PathBuf for cache lookup
        let path_buf = PathBuf::from(file_path);

        // Check cache first for performance
        if let Some(scope) = self.scope_cache.get(&path_buf) {
            debug!("Cache hit: file {} mapped to {:?}", file_path, scope);
            return Ok(scope.clone());
        }

        debug!("Mapping file {}", file_path);

        // Normalize path: Make absolute if relative
        let workspace_root = self.workspace.root_path();
        let file_abs_path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            workspace_root.join(file_path)
        };

        // NEW: Canonicalize the file path to resolve symlinks
        let file_canonical_path = match std::fs::canonicalize(&file_abs_path) {
            Ok(p) => p,
            Err(_) => file_abs_path.clone(), // Fallback to original if we can't canonicalize
        };

        debug!("Canonical path: {}", file_canonical_path.display());

        // Try to find the package this file belongs to
        for pkg_info in self.workspace.sorted_packages() {
            let pkg = pkg_info.borrow();
            let package_borrow = pkg.package.borrow();
            let pkg_name = package_borrow.name();
            let pkg_path_str = &pkg.package_path;

            // Convert to absolute path
            let pkg_abs_path = if Path::new(pkg_path_str).is_absolute() {
                PathBuf::from(pkg_path_str)
            } else {
                workspace_root.join(pkg_path_str)
            };

            // NEW: Canonicalize the package path
            let pkg_canonical_path = match std::fs::canonicalize(&pkg_abs_path) {
                Ok(p) => p,
                Err(_) => pkg_abs_path.clone(), // Fallback to original if we can't canonicalize
            };

            debug!("Checking against package {} at {}", pkg_name, pkg_canonical_path.display());

            // Check if canonical file path starts with canonical package path
            if file_canonical_path.starts_with(&pkg_canonical_path) {
                debug!("Match found! File belongs to package {}", pkg_name);
                let scope = ChangeScope::Package(pkg_name.to_string());
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }

            // Try string prefix matching as fallback
            let file_str = file_canonical_path.to_string_lossy().to_string();
            let pkg_str = pkg_canonical_path.to_string_lossy().to_string();
            if file_str.starts_with(&pkg_str) {
                debug!("String match found for package {}", pkg_name);
                let scope = ChangeScope::Package(pkg_name.to_string());
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }
        }

        debug!("No package match found, checking if at root level");

        // Canonicalize the workspace root too for consistency
        let root_canonical = match std::fs::canonicalize(workspace_root) {
            Ok(p) => p,
            Err(_) => workspace_root.to_path_buf(),
        };

        if let Some(parent) = file_canonical_path.parent() {
            if parent == root_canonical {
                debug!("File is at root level");
                let scope = ChangeScope::Root;
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }
        }

        debug!("File is in monorepo infrastructure (not in any package or at root)");
        let scope = ChangeScope::Monorepo;
        self.scope_cache.insert(path_buf, scope.clone());

        Ok(scope)
    }

    /// Gets the workspace root path
    ///
    /// # Returns
    ///
    /// The path to the workspace root directory.
    pub fn get_workspace_root_path(&self) -> &Path {
        self.workspace.root_path()
    }

    /// Detects changes between Git references.
    ///
    /// Analyzes the changes between two Git references (commits, tags, or branches)
    /// and generates change records.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - The starting Git reference (e.g., a commit SHA, tag, or branch name)
    /// * `to_ref` - Optional ending Git reference (defaults to HEAD if None)
    ///
    /// # Returns
    ///
    /// A vector of detected changes.
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail or no changes are detected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::rc::Rc;
    /// # use sublime_monorepo_tools::{ChangeTracker, MemoryChangeStore, Workspace};
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Detect changes since a tag
    /// let changes = tracker.detect_changes_between("v1.0.0", None)?;
    /// println!("Found {} changes", changes.len());
    ///
    /// // Detect changes between two tags
    /// let changes = tracker.detect_changes_between("v1.0.0", Some("v2.0.0"))?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::too_many_lines)]
    pub fn detect_changes_between(
        &mut self,
        from_ref: &str,
        to_ref: Option<&str>,
    ) -> ChangeResult<Vec<Change>> {
        // Clear cache first to avoid borrowing issues
        self.clear_cache();

        info!("Detecting changes from {} to {:?}", from_ref, to_ref);

        // Ensure we have a Git repository
        let repo = self.workspace.git_repo().ok_or(ChangeError::NoGitRepository)?;

        // Get the changed files with status
        let changed_files = repo
            .get_all_files_changed_since_sha_with_status(from_ref)
            .map_err(ChangeError::GitError)?;

        // Early return if no changes
        if changed_files.is_empty() {
            info!("No changes found between references");
            return Err(ChangeError::NoChangesFound);
        }

        info!("Found {} changed files", changed_files.len());

        // Get commit information if available
        debug!("Getting commits between {from_ref} and {to_ref:?}");
        let commits =
            repo.get_commits_since(Some(from_ref.to_string()), &None).map_or(vec![], |v| v);

        // Group changed files by package
        let mut package_changes: HashMap<String, Vec<sublime_git_tools::GitChangedFile>> =
            HashMap::new();
        let mut monorepo_changes = Vec::new();
        let mut root_changes = Vec::new();

        let workspace_root = self.workspace.root_path().to_path_buf();

        // Map files to packages
        for file in changed_files {
            debug!("Processing changed file: {}", file.path);

            // Convert to absolute path
            let file_abs_path = if Path::new(&file.path).is_absolute() {
                PathBuf::from(&file.path)
            } else {
                workspace_root.join(&file.path)
            };

            // First try direct mapping to discovered packages
            match self.map_file_to_scope(&file.path) {
                Ok(ChangeScope::Package(pkg_name)) => {
                    debug!("File {} directly mapped to package {}", file.path, pkg_name);
                    package_changes.entry(pkg_name).or_default().push(file);
                    continue;
                }
                Ok(ChangeScope::Root) => {
                    debug!("File {} is at root level", file.path);
                    root_changes.push(file);
                    continue;
                }
                _ => {
                    // Continue with more attempts to assign to a package
                }
            }

            // If direct mapping failed, try to find the nearest package.json
            if let Some(package_name) =
                Self::find_package_name_from_nearest_package_json(&file_abs_path)
            {
                debug!("File {} inferred to belong to package {}", file.path, package_name);
                package_changes.entry(package_name).or_default().push(file);
                continue;
            }

            // If that also failed, it's a monorepo file
            debug!("File {} is a monorepo file", file.path);
            monorepo_changes.push(file);
        }

        // Create changes from detected file changes
        let mut changes = Vec::new();

        // Handle package changes
        for (package_name, files) in package_changes {
            debug!("Creating change for package {} with {} files", package_name, files.len());

            // Get relevant commits for these files
            let relevant_commits: Vec<sublime_git_tools::RepoCommit> =
                ChangeTracker::get_commits_for_files(&commits, &files);

            // Determine change type from commits
            let change_type = ChangeTracker::determine_change_type_from_commits(&relevant_commits);

            // Check if any commit indicates a breaking change
            let is_breaking = change_type == ChangeType::Breaking;

            // Create change
            let mut change = Change::new(
                package_name,
                change_type,
                ChangeTracker::generate_change_description(&relevant_commits, &files),
                is_breaking,
            );

            // Set author
            if let Some(ref name) = self.git_config.user_name {
                change = change.with_author(name);
            } else if !relevant_commits.is_empty() {
                change = change.with_author(&relevant_commits[0].author_name);
            }

            changes.push(change);
        }

        // Handle monorepo changes
        if !monorepo_changes.is_empty() {
            debug!("Creating monorepo change with {} files", monorepo_changes.len());

            let relevant_commits =
                ChangeTracker::get_commits_for_files(&commits, &monorepo_changes);
            let change_type = ChangeTracker::determine_change_type_from_commits(&relevant_commits);
            let is_breaking = change_type == ChangeType::Breaking;

            let mut change = Change::new(
                "monorepo",
                change_type,
                &ChangeTracker::generate_change_description(&relevant_commits, &monorepo_changes),
                is_breaking,
            );

            if let Some(ref name) = self.git_config.user_name {
                change = change.with_author(name);
            } else if !relevant_commits.is_empty() {
                change = change.with_author(&relevant_commits[0].author_name);
            }

            changes.push(change);
        }

        // Handle root changes
        if !root_changes.is_empty() {
            debug!("Creating root change with {} files", root_changes.len());

            let relevant_commits = ChangeTracker::get_commits_for_files(&commits, &root_changes);
            let change_type = ChangeTracker::determine_change_type_from_commits(&relevant_commits);
            let is_breaking = change_type == ChangeType::Breaking;

            let mut change = Change::new(
                "root",
                change_type,
                &ChangeTracker::generate_change_description(&relevant_commits, &root_changes),
                is_breaking,
            );

            if let Some(ref name) = self.git_config.user_name {
                change = change.with_author(name);
            } else if !relevant_commits.is_empty() {
                change = change.with_author(&relevant_commits[0].author_name);
            }

            changes.push(change);
        }

        info!("Created {} changes", changes.len());
        Ok(changes)
    }

    /// Records a change manually.
    ///
    /// # Arguments
    ///
    /// * `change` - The change to record
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if recording fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the package does not exist or if storing the change fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{ChangeTracker, Change, ChangeType};
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a change
    /// let change = Change::new(
    ///     "ui-components",
    ///     ChangeType::Feature,
    ///     "Add new button component",
    ///     false
    /// );
    ///
    /// // Record the change
    /// tracker.record_change(change)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn record_change(&mut self, change: Change) -> ChangeResult<()> {
        // Validate that the package exists in the workspace
        if self.workspace.get_package(&change.package).is_none() {
            return Err(ChangeError::InvalidPackage(format!(
                "Package '{}' not found in workspace",
                change.package
            )));
        }

        // Create a changeset with this single change
        let changeset = Changeset::new::<String>(None, vec![change]);

        // Store the changeset
        self.store.store_changeset(&changeset)
    }

    /// Creates and records a changeset.
    ///
    /// # Arguments
    ///
    /// * `summary` - Optional summary for the changeset
    /// * `changes` - The changes to include in the changeset
    ///
    /// # Returns
    ///
    /// The created changeset.
    ///
    /// # Errors
    ///
    /// Returns an error if any package does not exist or if storing the changeset fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{ChangeTracker, Change, ChangeType};
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create multiple changes
    /// let change1 = Change::new("ui-components", ChangeType::Feature, "Add button", false);
    /// let change2 = Change::new("api", ChangeType::Fix, "Fix validation", false);
    ///
    /// // Create and store a changeset
    /// let changeset = tracker.create_changeset(
    ///     Some("PR #123: UI and API improvements".to_string()),
    ///     vec![change1, change2]
    /// )?;
    ///
    /// println!("Created changeset with ID: {}", changeset.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_changeset(
        &mut self,
        summary: Option<String>,
        changes: Vec<Change>,
    ) -> ChangeResult<Changeset> {
        // Validate that all packages exist
        for change in &changes {
            if self.workspace.get_package(&change.package).is_none() {
                return Err(ChangeError::InvalidPackage(format!(
                    "Package '{}' not found in workspace",
                    change.package
                )));
            }
        }

        // Create changeset
        let changeset = Changeset::new(summary, changes);

        // Store the changeset
        self.store.store_changeset(&changeset)?;

        Ok(changeset)
    }

    /// Gets unreleased changes for all packages.
    ///
    /// # Returns
    ///
    /// A hashmap where keys are package names and values are vectors of unreleased changes.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving unreleased changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get all unreleased changes
    /// let unreleased = tracker.unreleased_changes()?;
    ///
    /// // Process them
    /// for (package, changes) in unreleased {
    ///     println!("Package {} has {} unreleased changes", package, changes.len());
    ///
    ///     for change in changes {
    ///         println!("  - {}", change.summary());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn unreleased_changes(&self) -> ChangeResult<HashMap<String, Vec<Change>>> {
        // Get all changes by package
        let all_by_package = self.store.get_all_changes_by_package()?;

        // Filter to only unreleased changes
        let mut unreleased = HashMap::new();
        for (package, changes) in all_by_package {
            let unreleased_changes: Vec<Change> =
                changes.into_iter().filter(|c| c.release_version.is_none()).collect();

            if !unreleased_changes.is_empty() {
                unreleased.insert(package, unreleased_changes);
            }
        }

        Ok(unreleased)
    }

    /// Marks changes as released.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package to mark changes for
    /// * `version` - Version string to assign to the changes
    /// * `dry_run` - If true, only preview changes without applying them
    ///
    /// # Returns
    ///
    /// A vector of changes that were or would be marked as released.
    ///
    /// # Errors
    ///
    /// Returns an error if the package does not exist or if marking changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Preview changes that would be marked as released
    /// let dry_run = tracker.mark_released("ui-components", "2.0.0", true)?;
    /// println!("Would mark {} changes as released", dry_run.len());
    ///
    /// // Actually mark changes as released
    /// let released = tracker.mark_released("ui-components", "2.0.0", false)?;
    /// println!("Marked {} changes as released", released.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn mark_released(
        &mut self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>> {
        // Validate package exists
        if self.workspace.get_package(package).is_none() {
            return Err(ChangeError::InvalidPackage(format!(
                "Package '{package}' not found in workspace"
            )));
        }

        // Mark changes as released
        self.store.mark_changes_as_released(package, version, dry_run)
    }

    /// Gets unreleased changes for all packages filtered by environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment to filter changes for (e.g., "production", "staging")
    ///
    /// # Returns
    ///
    /// A hashmap where keys are package names and values are vectors of unreleased changes
    /// that apply to the specified environment.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving unreleased changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get unreleased changes for production
    /// let prod_changes = tracker.unreleased_changes_for_environment("production")?;
    ///
    /// // Get unreleased changes for staging
    /// let staging_changes = tracker.unreleased_changes_for_environment("staging")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn unreleased_changes_for_environment(
        &self,
        environment: &str,
    ) -> ChangeResult<HashMap<String, Vec<Change>>> {
        let all_unreleased = self.unreleased_changes()?;

        let mut filtered = HashMap::new();
        for (package, changes) in all_unreleased {
            let env_changes: Vec<Change> =
                changes.into_iter().filter(|c| c.applies_to_environment(environment)).collect();

            if !env_changes.is_empty() {
                filtered.insert(package, env_changes);
            }
        }

        Ok(filtered)
    }

    /// Marks changes as released for a specific environment.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package to mark changes for
    /// * `version` - Version string to assign to the changes
    /// * `environment` - Environment name to filter changes by
    /// * `dry_run` - If true, only preview changes without applying them
    ///
    /// # Returns
    ///
    /// A vector of changes that were or would be marked as released.
    ///
    /// # Errors
    ///
    /// Returns an error if the package does not exist or if marking changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Mark production changes as released
    /// let released = tracker.mark_released_for_environment(
    ///     "ui-components",
    ///     "2.0.0",
    ///     "production",
    ///     false
    /// )?;
    /// println!("Marked {} changes as released for production", released.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn mark_released_for_environment(
        &mut self,
        package: &str,
        version: &str,
        environment: &str,
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>> {
        // Get all unreleased changes for this package
        let unreleased = self.store().get_unreleased_changes(package)?;

        // Filter to only those that apply to this environment
        let applicable_changes: Vec<Change> =
            unreleased.into_iter().filter(|c| c.applies_to_environment(environment)).collect();

        if applicable_changes.is_empty() {
            return Ok(Vec::new());
        }

        // Get the IDs of changes to mark as released
        let change_ids: Vec<ChangeId> = applicable_changes.iter().map(|c| c.id.clone()).collect();

        // Mark these specific changes as released
        self.mark_specific_changes_as_released(package, version, &change_ids, dry_run)
    }

    /// Marks specific changes as released by ID.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package containing the changes
    /// * `version` - Version string to assign to the changes
    /// * `change_ids` - IDs of the specific changes to mark
    /// * `dry_run` - If true, only preview changes without applying them
    ///
    /// # Returns
    ///
    /// A vector of changes that were or would be marked as released.
    ///
    /// # Errors
    ///
    /// Returns an error if the package does not exist or if marking changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{ChangeTracker, ChangeId};
    /// # use std::str::FromStr;
    /// # fn example(mut tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get specific change IDs
    /// let id1 = ChangeId::from_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// let id2 = ChangeId::from_str("550e8400-e29b-41d4-a716-446655440001")?;
    ///
    /// // Mark only these changes as released
    /// let released = tracker.mark_specific_changes_as_released(
    ///     "ui-components",
    ///     "2.0.0",
    ///     &[id1, id2],
    ///     false
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn mark_specific_changes_as_released(
        &mut self,
        package: &str,
        version: &str,
        change_ids: &[ChangeId],
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>> {
        // Validate package exists
        if self.workspace.get_package(package).is_none() {
            return Err(ChangeError::InvalidPackage(format!(
                "Package '{package}' not found in workspace"
            )));
        }

        // Mark only the specified changes as released
        let mut updated_changes = Vec::new();
        let mut updated_changesets = Vec::new();

        // Find all changesets with these specific changes
        let all_changesets = self.store.get_all_changesets()?;

        for changeset in all_changesets {
            let mut has_updates = false;
            let mut updated_changeset = changeset.clone();

            // Update changes within this changeset
            for change in &mut updated_changeset.changes {
                if change.package == package
                    && change.release_version.is_none()
                    && change_ids.contains(&change.id)
                {
                    change.release_version = Some(version.to_string());
                    updated_changes.push(change.clone());
                    has_updates = true;
                }
            }

            // If changes were updated, queue for storage
            if has_updates {
                updated_changesets.push(updated_changeset);
            }
        }

        // Store updated changesets if not a dry run
        if !dry_run {
            for changeset in updated_changesets {
                self.store.store_changeset(&changeset)?;
            }
        }

        Ok(updated_changes)
    }

    /// Gets the change store.
    ///
    /// # Returns
    ///
    /// A reference to the underlying change store implementation.
    #[must_use]
    pub fn store(&self) -> &dyn ChangeStore {
        self.store.as_ref()
    }

    /// Gets a mutable reference to the change store.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying change store implementation.
    #[must_use]
    pub fn store_mut(&mut self) -> &mut dyn ChangeStore {
        self.store.as_mut()
    }

    /// Generate a report of package changes with cycle information
    ///
    /// Creates a text report showing all changes grouped by package,
    /// with optional cycle information for visualization.
    ///
    /// # Arguments
    ///
    /// * `with_cycle_info` - Whether to include dependency cycle information
    ///
    /// # Returns
    ///
    /// A formatted string report.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// // Generate a basic report
    /// let basic_report = tracker.generate_changes_report(false)?;
    ///
    /// // Generate a report with cycle information
    /// let detailed_report = tracker.generate_changes_report(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_changes_report(&self, with_cycle_info: bool) -> ChangeResult<String> {
        let mut report = String::new();

        // Get all changes by package
        let changes_by_package = self.store().get_all_changes_by_package()?;

        if changes_by_package.is_empty() {
            report.push_str("No changes found.\n");
            return Ok(report);
        }

        // Get cycle information if requested
        let mut cycle_groups = Vec::new();
        let mut cycle_membership = HashMap::new();

        if with_cycle_info {
            let sorted_with_cycles = self.workspace.get_sorted_packages_with_circulars();
            cycle_groups = sorted_with_cycles
                .circular
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|p| p.borrow().package.borrow().name().to_string())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            // Build cycle membership lookup
            for (i, group) in cycle_groups.iter().enumerate() {
                for pkg_name in group {
                    cycle_membership.insert(pkg_name.clone(), i);
                }
            }
        }

        // First display cycle information if requested
        if with_cycle_info && !cycle_groups.is_empty() {
            report.push_str("Circular dependency groups detected:\n");

            for (i, group) in cycle_groups.iter().enumerate() {
                report.push_str(&format!("  Group {}: {}\n", i + 1, group.join(" → ")));
            }
            report.push('\n');
        }

        // Report changes by package
        report.push_str("Package Changes:\n");

        // Sort packages for consistent output, putting cycle members first
        let mut packages: Vec<_> = changes_by_package.keys().collect();
        packages.sort_by(|a, b| {
            let a_in_cycle = cycle_membership.contains_key(*a);
            let b_in_cycle = cycle_membership.contains_key(*b);

            if a_in_cycle && !b_in_cycle {
                std::cmp::Ordering::Less
            } else if !a_in_cycle && b_in_cycle {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        for package_name in packages {
            let changes = &changes_by_package[package_name];

            // Indicate if package is part of a cycle
            let cycle_indicator = if let Some(&cycle_idx) = cycle_membership.get(package_name) {
                format!(" (part of cycle group {})", cycle_idx + 1)
            } else {
                String::new()
            };

            report.push_str(&format!("\n{package_name}{cycle_indicator}:\n"));

            if changes.is_empty() {
                report.push_str("  No changes\n");
                continue;
            }

            // Group by change type
            let mut by_type: HashMap<&ChangeType, Vec<&Change>> = HashMap::new();

            for change in changes {
                by_type.entry(&change.change_type).or_default().push(change);
            }

            // Print changes by type
            for (change_type, type_changes) in by_type {
                report.push_str(&format!("  {change_type}:\n"));

                for change in type_changes {
                    let breaking = if change.breaking { " [BREAKING]" } else { "" };
                    report.push_str(&format!("    - {}{}\n", change.description, breaking));
                }
            }
        }

        Ok(report)
    }

    /// Visualize the dependency graph including cycles
    ///
    /// Creates an ASCII art visualization of the package dependency graph,
    /// highlighting cycles.
    ///
    /// # Arguments
    ///
    /// * `include_cycles` - Whether to include cycle details
    ///
    /// # Returns
    ///
    /// A formatted string visualization.
    ///
    /// # Errors
    ///
    /// Returns an error if graph generation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::ChangeTracker;
    /// # fn example(tracker: ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = tracker.visualize_dependency_graph(true)?;
    /// println!("{}", graph);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::single_char_add_str)]
    pub fn visualize_dependency_graph(&self, include_cycles: bool) -> ChangeResult<String> {
        let mut output = String::new();

        // Get cycle information
        let sorted_with_cycles = self.workspace.get_sorted_packages_with_circulars();
        let cycle_groups = sorted_with_cycles
            .circular
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|p| p.borrow().package.borrow().name().to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Build cycle membership lookup
        let mut cycle_membership = HashMap::new();
        for (i, group) in cycle_groups.iter().enumerate() {
            for pkg_name in group {
                cycle_membership.insert(pkg_name.clone(), i);
            }
        }

        // Generate ASCII art representation of the dependency graph
        output.push_str("Package Dependency Graph:\n");
        output.push_str("========================\n\n");

        // Display cycle information if requested
        if include_cycles && !cycle_groups.is_empty() {
            output.push_str("Cycle Groups:\n");

            for (i, group) in cycle_groups.iter().enumerate() {
                output.push_str(&format!("  Group {}: {}\n", i + 1, group.join(" → ")));
            }
            output.push_str("\n");
        }

        // Create a simplified graph visualization
        let packages = self.workspace.sorted_packages();

        // Process each package
        for pkg_info in &packages {
            let pkg = pkg_info.borrow();
            let package = pkg.package.borrow();
            let pkg_name = package.name().to_string();

            // Skip root or special packages if desired
            if pkg_name == "root" || pkg_name == "monorepo" {
                continue;
            }

            // Check if this package is in a cycle
            let cycle_marker = if let Some(&cycle_idx) = cycle_membership.get(&pkg_name) {
                format!(" (cycle group {})", cycle_idx + 1)
            } else {
                String::new()
            };

            output.push_str(&format!("\n{pkg_name}{cycle_marker}\n"));

            // List dependencies
            let deps = package.dependencies();
            if deps.is_empty() {
                output.push_str("  No dependencies\n");
            } else {
                output.push_str("  Dependencies:\n");

                for dep in deps {
                    let dep_name = dep.borrow().name().to_string();

                    // Check if dependency is in a cycle
                    let dep_cycle = if let Some(&cycle_idx) = cycle_membership.get(&dep_name) {
                        format!(" (cycle group {})", cycle_idx + 1)
                    } else {
                        String::new()
                    };

                    // Check if this forms a cyclic dependency
                    let is_cyclic = cycle_membership.get(&pkg_name)
                        == cycle_membership.get(&dep_name)
                        && cycle_membership.contains_key(&pkg_name);

                    let cyclic_marker = if is_cyclic { " ⟲" } else { "" };

                    output.push_str(&format!("    - {dep_name}{dep_cycle}{cyclic_marker}\n"));
                }
            }
        }

        Ok(output)
    }

    // Helper method to find the nearest package.json and extract its name
    fn find_package_name_from_nearest_package_json(file_path: &Path) -> Option<String> {
        let mut current = file_path;

        // Traverse up the directory tree looking for package.json
        while let Some(parent) = current.parent() {
            let package_json_path = parent.join("package.json");

            if package_json_path.exists() {
                // Found a package.json, try to read it
                if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                    // Try to parse as JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Check if it has a name field
                        if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                            return Some(name.to_string());
                        }
                    }
                }

                // If we found a package.json but couldn't get a name, stop looking
                break;
            }

            // Move up one level
            current = parent;
        }

        None
    }

    /// Analyzes commit messages to determine the change type
    ///
    /// Examines the commit messages to identify the type of change
    /// (feature, fix, breaking, etc.) based on conventional commit prefixes.
    ///
    /// # Arguments
    ///
    /// * `commits` - List of Git commit data
    ///
    /// # Returns
    ///
    /// The determined `ChangeType`
    #[allow(clippy::too_many_lines)]
    fn determine_change_type_from_commits(commits: &[sublime_git_tools::RepoCommit]) -> ChangeType {
        // If no commits, default to Chore
        if commits.is_empty() {
            debug!("No commits provided, defaulting to Chore");
            return ChangeType::Chore;
        }

        // For debugging, print all commit messages
        for (i, commit) in commits.iter().enumerate() {
            debug!("Commit {}: {}", i, commit.message);
        }

        // Look for conventional commit prefixes in the commit messages
        // Prioritize the most significant change type
        let mut has_feature = false;
        let mut has_fix = false;
        let mut has_breaking = false;
        let mut has_perf = false;
        let mut has_docs = false;
        let mut has_test = false;
        let mut has_ci = false;
        let mut has_build = false;
        let mut has_refactor = false;
        let mut has_style = false;
        let mut has_revert = false;

        for commit in commits {
            let message = commit.message.to_lowercase();
            debug!("Analyzing commit message: {}", message);

            // Check for breaking changes (highest priority)
            if message.contains("breaking")
                || message.contains("major")
                || message.contains("!:")
                || message.contains("!)")
                || message.contains("breaking change")
            {
                debug!("Found breaking change indicator");
                has_breaking = true;
            }

            // Check conventional commit prefixes
            if message.starts_with("feat")
                || message.contains("feat:")
                || message.contains("feature")
            {
                debug!("Found feature indicator");
                has_feature = true;
            } else if message.starts_with("fix") || message.contains("fix:") {
                debug!("Found fix indicator");
                has_fix = true;
            } else if message.starts_with("perf")
                || message.contains("perf:")
                || message.contains("performance")
            {
                debug!("Found performance indicator");
                has_perf = true;
            } else if message.starts_with("docs")
                || message.contains("docs:")
                || message.contains("documentation")
            {
                debug!("Found documentation indicator");
                has_docs = true;
            } else if message.starts_with("test") || message.contains("test:") {
                debug!("Found test indicator");
                has_test = true;
            } else if message.starts_with("ci") || message.contains("ci:") {
                debug!("Found CI indicator");
                has_ci = true;
            } else if message.starts_with("build") || message.contains("build:") {
                debug!("Found build indicator");
                has_build = true;
            } else if message.starts_with("refactor") || message.contains("refactor:") {
                debug!("Found refactor indicator");
                has_refactor = true;
            } else if message.starts_with("style") || message.contains("style:") {
                debug!("Found style indicator");
                has_style = true;
            } else if message.starts_with("revert") || message.contains("revert:") {
                debug!("Found revert indicator");
                has_revert = true;
            }
        }

        // Determine the most significant change type based on priority
        let result = if has_breaking {
            debug!("Breaking change detected from commit message");
            ChangeType::Breaking
        } else if has_feature {
            debug!("Feature change detected from commit message");
            ChangeType::Feature
        } else if has_fix {
            debug!("Fix change detected from commit message");
            ChangeType::Fix
        } else if has_perf {
            debug!("Performance change detected from commit message");
            ChangeType::Performance
        } else if has_docs {
            debug!("Documentation change detected from commit message");
            ChangeType::Documentation
        } else if has_test {
            debug!("Test change detected from commit message");
            ChangeType::Test
        } else if has_ci {
            debug!("CI change detected from commit message");
            ChangeType::CI
        } else if has_build {
            debug!("Build change detected from commit message");
            ChangeType::Build
        } else if has_refactor {
            debug!("Refactor change detected from commit message");
            ChangeType::Refactor
        } else if has_style {
            debug!("Style change detected from commit message");
            ChangeType::Style
        } else if has_revert {
            debug!("Revert change detected from commit message");
            ChangeType::Revert
        } else {
            debug!("No specific change type detected from commit message, defaulting to Chore");
            ChangeType::Chore
        };

        debug!("Determined change type: {:?}", result);
        result
    }

    /// Gets the commits related to specific files
    ///
    /// # Arguments
    ///
    /// * `all_commits` - List of all commits
    /// * `_files` - List of changed files
    ///
    /// # Returns
    ///
    /// A list of commits related to the specified files
    fn get_commits_for_files(
        all_commits: &[sublime_git_tools::RepoCommit],
        _files: &[sublime_git_tools::GitChangedFile],
    ) -> Vec<sublime_git_tools::RepoCommit> {
        // In a real implementation, we'd want to filter the commits to only those that affect these files
        // This would require more sophisticated Git operations
        // For now, we'll just return all commits as an approximation
        debug!("Getting commits for files. All commits count: {}", all_commits.len());
        for (i, commit) in all_commits.iter().enumerate() {
            debug!("Commit {}: {} - {}", i, commit.hash, commit.message);
        }
        all_commits.to_vec()
    }

    /// Generates a descriptive message for the change based on commits
    ///
    /// # Arguments
    ///
    /// * `commits` - List of commits
    /// * `files` - List of changed files
    ///
    /// # Returns
    ///
    /// A descriptive string for the change
    fn generate_change_description(
        commits: &[sublime_git_tools::RepoCommit],
        files: &[sublime_git_tools::GitChangedFile],
    ) -> String {
        if commits.is_empty() {
            return format!("Changes detected ({} files)", files.len());
        }

        // Try to use the first commit message as the change description
        let first_commit = &commits[0];
        let message = first_commit.message.lines().next().unwrap_or("").trim();

        if message.is_empty() {
            format!("Changes detected ({} files)", files.len())
        } else if commits.len() == 1 {
            message.to_string()
        } else {
            format!("{} (and {} more commits)", message, commits.len() - 1)
        }
    }
}
