//! Automatic changeset creation for dependency upgrades.
//!
//! **What**: Provides functionality to automatically create or update changesets after
//! applying dependency upgrades, tracking which packages were affected and ensuring
//! proper version bump configuration.
//!
//! **How**: This module integrates with the changeset manager to create changesets
//! with affected packages, configurable version bump types, and proper metadata.
//! It supports both creating new changesets and updating existing ones based on
//! the current git branch.
//!
//! **Why**: To enable automated tracking of dependency upgrades through the changeset
//! workflow, ensuring upgrades are properly versioned and documented without manual
//! intervention.

use crate::changeset::ChangesetManager;
use crate::error::{UpgradeError, UpgradeResult};
use crate::types::VersionBump;
use std::collections::HashSet;
use std::path::Path;
use sublime_git_tools::Repo;

/// Creates or updates a changeset for applied upgrades.
///
/// This function automatically creates a changeset when dependency upgrades are applied,
/// or updates an existing changeset if one already exists for the current branch.
///
/// # Changeset Details
///
/// The created/updated changeset will have:
/// - **Branch**: Current git branch name
/// - **Bump**: Configurable version bump (typically `patch` for dependency upgrades)
/// - **Environments**: Empty (user can update later)
/// - **Packages**: All packages that had dependencies upgraded
/// - **Changes**: Empty (commits are added later via `add_commits_from_git`)
///
/// # Arguments
///
/// * `changeset_manager` - Manager instance for changeset operations
/// * `affected_packages` - Set of package names that were modified
/// * `workspace_root` - Root directory of the workspace
/// * `bump_type_str` - String representation of version bump type ("major", "minor", "patch", "none")
///
/// # Returns
///
/// The changeset ID (branch name) on success, or `None` if changeset creation is disabled
/// or if the current branch cannot be determined.
///
/// # Errors
///
/// Returns `UpgradeError` if:
/// - Version bump type string is invalid
/// - Git repository operations fail
/// - Changeset creation/update fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::application::create_changeset_for_upgrades;
/// use sublime_pkg_tools::changeset::ChangesetManager;
/// use std::collections::HashSet;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = ChangesetManager::new(&PathBuf::from(".")).await?;
/// let mut packages = HashSet::new();
/// packages.insert("@myorg/core".to_string());
/// packages.insert("@myorg/utils".to_string());
///
/// let changeset_id = create_changeset_for_upgrades(
///     &manager,
///     packages,
///     &PathBuf::from("."),
///     "patch"
/// ).await?;
///
/// if let Some(id) = changeset_id {
///     println!("Created changeset: {}", id);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn create_changeset_for_upgrades<S>(
    changeset_manager: &ChangesetManager<S>,
    affected_packages: HashSet<String>,
    workspace_root: &Path,
    bump_type_str: &str,
) -> UpgradeResult<Option<String>>
where
    S: crate::changeset::ChangesetStorage + Send + Sync,
{
    // Return early if no packages were affected
    if affected_packages.is_empty() {
        return Ok(None);
    }

    // Parse version bump type
    let bump =
        VersionBump::parse(bump_type_str).map_err(|e| UpgradeError::ChangesetCreationFailed {
            reason: format!("Invalid bump type '{}': {}", bump_type_str, e),
        })?;

    // Get current branch from git
    let branch_name = match get_current_branch(workspace_root) {
        Ok(branch) => branch,
        Err(e) => {
            // If we can't get the branch, log the error and return None
            // This allows upgrades to succeed even if changeset creation fails
            eprintln!("Warning: Could not determine current git branch: {}", e);
            return Ok(None);
        }
    };

    // Convert HashSet to Vec for changeset
    let package_list: Vec<String> = affected_packages.into_iter().collect();

    // Check if changeset already exists for this branch
    let changeset_exists = changeset_manager
        .storage()
        .exists(&branch_name)
        .await
        .map_err(|e| UpgradeError::ChangesetCreationFailed { reason: e.to_string() })?;

    if changeset_exists {
        // Update existing changeset with new packages
        update_existing_changeset(changeset_manager, &branch_name, package_list).await?;
    } else {
        // Create new changeset
        create_new_changeset(changeset_manager, &branch_name, bump, package_list).await?;
    }

    Ok(Some(branch_name))
}

/// Creates a new changeset for upgrades.
///
/// # Arguments
///
/// * `changeset_manager` - Manager instance for changeset operations
/// * `branch_name` - Name of the git branch
/// * `bump` - Version bump type
/// * `packages` - List of affected package names
///
/// # Errors
///
/// Returns error if changeset creation fails
async fn create_new_changeset<S>(
    changeset_manager: &ChangesetManager<S>,
    branch_name: &str,
    bump: VersionBump,
    packages: Vec<String>,
) -> UpgradeResult<()>
where
    S: crate::changeset::ChangesetStorage + Send + Sync,
{
    // Create changeset with empty environments (user can update later)
    let mut changeset = changeset_manager
        .create(branch_name, bump, vec![])
        .await
        .map_err(|e| UpgradeError::ChangesetCreationFailed { reason: e.to_string() })?;

    // Add packages to the changeset (sorted for consistency)
    let mut sorted_packages = packages;
    sorted_packages.sort();
    changeset.packages = sorted_packages;

    // Update the changeset with the packages
    changeset_manager
        .storage()
        .save(&changeset)
        .await
        .map_err(|e| UpgradeError::ChangesetCreationFailed { reason: e.to_string() })?;

    Ok(())
}

/// Updates an existing changeset with additional packages.
///
/// This function merges new packages with existing ones, avoiding duplicates.
///
/// # Arguments
///
/// * `changeset_manager` - Manager instance for changeset operations
/// * `branch_name` - Name of the git branch
/// * `new_packages` - List of newly affected package names
///
/// # Errors
///
/// Returns error if loading or updating the changeset fails
async fn update_existing_changeset<S>(
    changeset_manager: &ChangesetManager<S>,
    branch_name: &str,
    new_packages: Vec<String>,
) -> UpgradeResult<()>
where
    S: crate::changeset::ChangesetStorage + Send + Sync,
{
    // Load existing changeset
    let mut changeset = changeset_manager
        .load(branch_name)
        .await
        .map_err(|e| UpgradeError::ChangesetCreationFailed { reason: e.to_string() })?;

    // Merge packages (avoid duplicates)
    let mut existing_packages: HashSet<String> = changeset.packages.into_iter().collect();
    for package in new_packages {
        existing_packages.insert(package);
    }

    // Convert back to Vec
    changeset.packages = existing_packages.into_iter().collect();
    changeset.packages.sort(); // Keep packages sorted for consistency

    // Update timestamp
    changeset.updated_at = chrono::Utc::now();

    // Save updated changeset
    changeset_manager
        .storage()
        .save(&changeset)
        .await
        .map_err(|e| UpgradeError::ChangesetCreationFailed { reason: e.to_string() })?;

    Ok(())
}

/// Gets the current git branch name.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
///
/// # Returns
///
/// The current branch name
///
/// # Errors
///
/// Returns error if:
/// - Git repository cannot be opened
/// - Current branch cannot be determined (e.g., detached HEAD)
fn get_current_branch(workspace_root: &Path) -> UpgradeResult<String> {
    // Open git repository
    let workspace_str = workspace_root.to_str().ok_or_else(|| {
        UpgradeError::ChangesetCreationFailed { reason: "Invalid workspace path".to_string() }
    })?;
    let repo = Repo::open(workspace_str).map_err(|e| UpgradeError::ChangesetCreationFailed {
        reason: format!("Failed to open git repository: {}", e),
    })?;

    // Get current branch
    let branch = repo.get_current_branch().map_err(|e| UpgradeError::ChangesetCreationFailed {
        reason: format!("Failed to get current branch: {}", e),
    })?;

    Ok(branch)
}
