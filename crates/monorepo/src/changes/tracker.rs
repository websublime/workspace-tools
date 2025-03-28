//! Change tracking system.

use std::rc::Rc;
use std::{collections::HashMap, path::Path};

use crate::{Change, ChangeError, ChangeResult, ChangeStore, ChangeType, Changeset, Workspace};

/// Change tracking system.
pub struct ChangeTracker {
    /// Workspace reference
    workspace: Rc<Workspace>,
    /// Storage for changes
    store: Box<dyn ChangeStore>,
    /// Git configuration
    git_config: GitConfig,
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

        Self { workspace, store, git_config }
    }

    /// Sets the Git user information.
    #[must_use]
    pub fn with_git_user<S: Into<String>>(mut self, name: Option<S>, email: Option<S>) -> Self {
        self.git_config =
            GitConfig { user_name: name.map(Into::into), user_email: email.map(Into::into) };
        self
    }

    /// Detects changes between Git references.
    ///
    /// # Errors
    /// Returns an error if change detection fails.
    pub fn detect_changes_between(
        &self,
        from_ref: &str,
        to_ref: Option<&str>,
    ) -> ChangeResult<Vec<Change>> {
        // Ensure we have a Git repository
        let repo = self.workspace.git_repo().ok_or(ChangeError::NoGitRepository)?;

        // Get the changed files with status
        let changed_files = repo
            .get_all_files_changed_since_sha_with_status(from_ref)
            .map_err(ChangeError::GitError)?;

        // Early return if no changes
        if changed_files.is_empty() {
            return Err(ChangeError::NoChangesFound);
        }

        // Get commit information if available
        let commits = if let Some(_to) = to_ref {
            // If we have a to_ref, get commits between from_ref and to_ref
            repo.get_commits_since(Some(from_ref.to_string()), &None)
                .map_err(ChangeError::GitError)?
        } else {
            // Otherwise, get all commits since from_ref
            vec![]
        };

        // Group changed files by package
        let mut changes_by_package: HashMap<String, Vec<sublime_git_tools::GitChangedFile>> =
            HashMap::new();

        for file in changed_files {
            // Find which package this file belongs to
            if let Some(package_name) = self.find_package_for_file(&file.path) {
                changes_by_package.entry(package_name).or_default().push(file);
            }
        }

        if changes_by_package.is_empty() {
            return Err(ChangeError::NoChangesFound);
        }

        // Create changes from detected file changes
        let mut changes = Vec::new();
        for (package_name, files) in changes_by_package {
            // Determine the most appropriate change type based on the files
            let change_type = self.determine_change_type(&files);

            // Create change
            let mut change = Change::new(
                package_name.clone(),
                change_type,
                format!("Changes detected ({} files)", files.len()),
                false, // Not marking as breaking by default
            );

            // Try to set author if available
            if let Some(ref name) = self.git_config.user_name {
                change = change.with_author(name);
            } else if !commits.is_empty() {
                // Use author from first commit
                change = change.with_author(&commits[0].author_name);
            }

            changes.push(change);
        }

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

    // Helper methods

    /// Finds which package a file belongs to.
    fn find_package_for_file(&self, file_path: &str) -> Option<String> {
        // Normalize file path for comparison
        let file_path = Path::new(file_path);
        let absolute_file_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            // If path is relative, make it absolute relative to workspace root
            self.workspace.root_path().join(file_path)
        };

        // Loop through all packages and see if the file is in their directory
        for package_info in self.workspace.sorted_packages() {
            let package_info_borrow = package_info.borrow();
            let package_path = Path::new(&package_info_borrow.package_path);
            let package_borrow = package_info_borrow.package.borrow();
            let package_name = package_borrow.name();

            // Attempt direct path prefix matching
            if let Ok(_relative_path) = file_path.strip_prefix(package_path) {
                return Some(package_name.to_string());
            }

            // Try matching on absolute paths
            if let Ok(_relative_path) = absolute_file_path.strip_prefix(package_path) {
                return Some(package_name.to_string());
            }

            // Check common monorepo structures (packages/pkg-name/*, apps/pkg-name/*)
            let file_path_str = file_path.to_string_lossy();
            let package_in_packages = format!("packages/{package_name}/");
            let package_in_apps = format!("apps/{package_name}/");

            if file_path_str.contains(&*package_in_packages)
                || file_path_str.contains(&*package_in_apps)
            {
                return Some(package_name.to_string());
            }

            // Check if this file is the package.json for this package
            if file_path.ends_with("package.json") {
                let parent_dir = file_path.parent().unwrap_or(file_path);
                let dir_name = parent_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if dir_name == package_name {
                    return Some(package_name.to_string());
                }
            }
        }

        // If no package matches, check if the file is part of the root package
        if let Some(root_package) = self.workspace.get_package("root") {
            let root_name = root_package.borrow().package.borrow().name().to_string();
            return Some(root_name);
        }

        None
    }

    /// Determines the most appropriate change type based on the files changed
    #[allow(clippy::unused_self)]
    fn determine_change_type(&self, files: &[sublime_git_tools::GitChangedFile]) -> ChangeType {
        // Count files of each type
        let mut test_files = 0;
        let mut doc_files = 0;
        let mut ci_files = 0;
        let mut build_files = 0;

        for file in files {
            let path = &file.path;

            // Check file patterns to determine types
            if path.contains("/tests/")
                || path.contains("/__tests__/")
                || path.ends_with(".test.js")
                || path.ends_with(".spec.js")
            {
                test_files += 1;
            } else if path.contains("/docs/")
                || std::path::Path::new(path)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
                || std::path::Path::new(path)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("mdx"))
            {
                doc_files += 1;
            } else if path.contains("/.github/")
                || path.contains("/ci/")
                || path.contains(".github/workflows/")
            {
                ci_files += 1;
            } else if path.contains("/webpack")
                || path.contains("/rollup")
                || path.ends_with("package.json")
            {
                build_files += 1;
            }
        }

        // Determine the dominant change type
        if test_files > 0 && test_files >= files.len() / 2 {
            ChangeType::Test
        } else if doc_files > 0 && doc_files >= files.len() / 2 {
            ChangeType::Documentation
        } else if ci_files > 0 && ci_files >= files.len() / 2 {
            ChangeType::CI
        } else if build_files > 0 && build_files >= files.len() / 2 {
            ChangeType::Build
        } else if files.len() == 1 {
            // If only one file, use the infer logic
            Self::infer_change_type_from_file(&files[0].path)
        } else {
            // Default for mixed or unknown files
            ChangeType::Chore
        }
    }

    /// Tries to infer change type from a file path.
    fn infer_change_type_from_file(file_path: &str) -> ChangeType {
        if file_path.contains("/tests/")
            || file_path.contains("/__tests__/")
            || file_path.ends_with(".test.js")
            || file_path.ends_with(".spec.js")
        {
            ChangeType::Test
        } else if file_path.contains("/docs/")
            || std::path::Path::new(file_path)
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
            || std::path::Path::new(file_path)
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("mdx"))
        {
            ChangeType::Documentation
        } else if file_path.contains("/.github/")
            || file_path.contains("/ci/")
            || file_path.contains(".github/workflows/")
        {
            ChangeType::CI
        } else if file_path.contains("/webpack")
            || file_path.contains("/rollup")
            || file_path.ends_with("package.json")
        {
            ChangeType::Build
        } else {
            // Default to chore for unknown files
            ChangeType::Chore
        }
    }
}
