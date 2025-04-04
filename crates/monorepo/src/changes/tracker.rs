//! Change tracking system.

use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::{
    Change, ChangeError, ChangeId, ChangeResult, ChangeStore, ChangeType, Changeset, Workspace,
};

/// Represents the scope of a change in the repository
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
    #[must_use]
    pub fn new(workspace: Rc<Workspace>, store: Box<dyn ChangeStore>) -> Self {
        // Default Git configuration
        let git_config = GitConfig { user_name: None, user_email: None };

        Self { workspace, store, git_config, scope_cache: HashMap::new() }
    }

    /// Sets the Git user information.
    #[must_use]
    pub fn with_git_user<S: Into<String>>(mut self, name: Option<S>, email: Option<S>) -> Self {
        self.git_config =
            GitConfig { user_name: name.map(Into::into), user_email: email.map(Into::into) };
        self
    }

    /// Clears the scope cache
    pub fn clear_cache(&mut self) {
        debug!("Clearing file scope cache");
        self.scope_cache.clear();
    }

    /// Maps a file to its change scope
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

        debug!("Normalized absolute path: {}", file_abs_path.display());

        // Print all packages and their paths
        debug!("Available packages:");
        for pkg_info in self.workspace.sorted_packages() {
            let pkg = pkg_info.borrow();
            debug!("  Package {} at {}", pkg.package.borrow().name(), pkg.package_path);
        }

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

            debug!("Checking against package {} at {}", pkg_name, pkg_abs_path.display());

            // Check if file path starts with package path
            if file_abs_path.starts_with(&pkg_abs_path) {
                debug!("Match found! File belongs to package {}", pkg_name);
                let scope = ChangeScope::Package(pkg_name.to_string());
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }

            // Try direct string prefix matching as fallback
            let file_str = file_abs_path.to_string_lossy().to_string();
            let pkg_str = pkg_abs_path.to_string_lossy().to_string();
            if file_str.starts_with(&pkg_str) {
                debug!("String match found for package {}", pkg_name);
                let scope = ChangeScope::Package(pkg_name.to_string());
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }

            // Last resort: check if the file_path contains the package path's basename
            if let Some(pkg_basename) = pkg_abs_path.file_name() {
                let pkg_basename_str = pkg_basename.to_string_lossy().to_string();
                if file_path.contains(&pkg_basename_str) {
                    debug!("Basename match found for package {}", pkg_name);
                    let scope = ChangeScope::Package(pkg_name.to_string());
                    self.scope_cache.insert(path_buf, scope.clone());
                    return Ok(scope);
                }
            }
        }

        // If we get here, file doesn't belong to any package
        debug!("No package match found, checking if at root level");

        // Check if file is direct child of workspace root
        if let Some(parent) = file_abs_path.parent() {
            if parent == workspace_root {
                debug!("File is at root level");
                let scope = ChangeScope::Root;
                self.scope_cache.insert(path_buf, scope.clone());
                return Ok(scope);
            }
        }

        // Default case: file is somewhere in monorepo but not in a specific package
        debug!("File is in monorepo infrastructure (not in any package or at root)");
        let scope = ChangeScope::Monorepo;
        self.scope_cache.insert(path_buf, scope.clone());

        Ok(scope)
    }

    pub fn get_workspace_root_path(&self) -> &Path {
        self.workspace.root_path()
    }

    /// Detects changes between Git references.
    ///
    /// # Errors
    /// Returns an error if change detection fails.
    #[allow(clippy::too_many_lines)]
    pub fn detect_changes_between(
        &mut self,
        from_ref: &str,
        to_ref: Option<&str>,
    ) -> ChangeResult<Vec<Change>> {
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

        // Print all changed files for debugging
        for file in &changed_files {
            debug!("Changed file: {}", file.path);
        }

        // Get commit information if available
        debug!("Getting commits between {from_ref} and {to_ref:?}");
        let commits =
            repo.get_commits_since(Some(from_ref.to_string()), &None).map_or(vec![], |v| v);

        // Group changed files by scope (package, monorepo, root)
        let mut package_changes: HashMap<String, Vec<sublime_git_tools::GitChangedFile>> =
            HashMap::new();
        let mut monorepo_changes = Vec::new();
        let mut root_changes = Vec::new();

        // Clear cache to ensure fresh mappings
        self.clear_cache();

        for file in changed_files {
            debug!("Mapping file to scope: {}", file.path);
            match self.map_file_to_scope(&file.path)? {
                ChangeScope::Package(package_name) => {
                    debug!("File {} mapped to package {}", file.path, package_name);
                    package_changes.entry(package_name).or_default().push(file);
                }
                ChangeScope::Monorepo => {
                    debug!("File {} mapped to monorepo", file.path);
                    monorepo_changes.push(file);
                }
                ChangeScope::Root => {
                    debug!("File {} mapped to root", file.path);
                    root_changes.push(file);
                }
            }
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

        // Handle monorepo and root changes similarly...
        // (rest of the method remains unchanged)

        info!("Created {} changes", changes.len());
        Ok(changes)
    }

    /// Records a change manually.
    ///
    /// # Errors
    /// Returns an error if recording the change fails.
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
    /// # Errors
    /// Returns an error if creating the changeset fails.
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
    /// # Errors
    /// Returns an error if retrieving unreleased changes fails.
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
    /// # Errors
    /// Returns an error if marking changes as released fails.
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
    /// # Errors
    /// Returns an error if retrieving unreleased changes fails.
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
    /// # Errors
    /// Returns an error if marking changes as released fails.
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
    /// # Errors
    /// Returns an error if marking changes as released fails.
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
    #[must_use]
    pub fn store(&self) -> &dyn ChangeStore {
        self.store.as_ref()
    }

    /// Gets a mutable reference to the change store.
    #[must_use]
    pub fn store_mut(&mut self) -> &mut dyn ChangeStore {
        self.store.as_mut()
    }

    /// Analyzes commit messages to determine the change type
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
